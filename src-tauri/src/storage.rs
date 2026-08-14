use anyhow::{anyhow, Result};
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::*;
use native_model::native_model;
use native_model::Model;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::{uuid, Uuid};

use crate::playback::model::PlayableEntry;
use crate::types::TrackMeta;

const URL_NAMESPACE: Uuid = uuid!("49be3fd4-a796-4392-9ce8-b7af0d3866f3");

pub const MAX_RECENT_TRACK_COUNT: u16 = 100;
// cover url 或许也可以缓存

trait DbCheck {
    fn exists_in_db(&self, rw: &RwTransaction) -> Result<bool>;
}

pub fn get_uuid_from_url(url: &str) -> Uuid {
    Uuid::new_v5(&URL_NAMESPACE, url.as_bytes())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct TrackDbItem {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    #[primary_key]
    pub id: String,
    pub src: String,
    pub meta: TrackMeta,
}

impl DbCheck for TrackDbItem {
    fn exists_in_db(&self, rw: &RwTransaction) -> Result<bool> {
        let res = rw.get().primary::<TrackDbItem>(self.id.clone())?;
        Ok(res.is_some())
    }
}

impl TrackDbItem {
    pub fn to_playable_entry(&self) -> Result<PlayableEntry> {
        let source_ref = self
            .meta
            .to_source_ref()
            .ok_or_else(|| anyhow!("unknown track source metadata for {}", self.id))?;
        Ok(PlayableEntry {
            view: crate::types::TrackView {
                title: self.title.clone(),
                artist: self.artist.clone(),
                cover_url: self.cover_url.clone(),
                duration: self.duration,
                id: self.id.clone(),
            },
            source_ref,
        })
    }
}

pub fn upsert_track_entry(
    db: &Database,
    entry: &PlayableEntry,
    cached_path: Option<&Path>,
) -> Result<TrackDbItem> {
    let item = TrackDbItem {
        title: entry.view.title.clone(),
        artist: entry.view.artist.clone(),
        cover_url: entry.view.cover_url.clone(),
        duration: entry.view.duration,
        id: entry.view.id.clone(),
        src: cached_path
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        meta: TrackMeta::from_source_ref(&entry.source_ref),
    };

    let rw = db.rw_transaction()?;
    if let Some(existing) = rw.get().primary::<TrackDbItem>(item.id.clone())? {
        rw.remove(existing)?;
    }
    rw.insert::<TrackDbItem>(item.clone())?;
    rw.commit()?;
    Ok(item)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct LikedTrack {
    #[primary_key]
    pub id: String,
    #[secondary_key]
    pub added_at: i64,
}
impl DbCheck for LikedTrack {
    fn exists_in_db(&self, rw: &RwTransaction) -> Result<bool> {
        let res = rw.get().primary::<LikedTrack>(self.id.clone())?;
        Ok(res.is_some())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct RecentTrack {
    #[primary_key]
    pub id: String,
    #[secondary_key]
    pub added_at: i64,
}

impl DbCheck for RecentTrack {
    fn exists_in_db(&self, rw: &RwTransaction) -> Result<bool> {
        let res = rw.get().primary::<RecentTrack>(self.id.clone())?;
        Ok(res.is_some())
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct Playlist {
    #[primary_key]
    pub id: String,
    pub name: String,
    #[secondary_key]
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct PlaylistTrack {
    #[primary_key]
    pub id: String,
    #[secondary_key]
    pub playlist_id: String,
    pub track_id: String,
    #[secondary_key]
    pub position: i64,
}

#[derive(Debug, PartialEq)]
pub struct PlaylistData {
    pub playlist: Playlist,
    pub tracks: Vec<TrackDbItem>,
}

fn validate_playlist_name(name: String) -> Result<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(anyhow!("playlist name cannot be empty"));
    }
    Ok(name)
}

fn playlist_data_from_transaction(
    r: &RTransaction,
    playlist: Playlist,
) -> Result<PlaylistData> {
    let mut playlist_tracks: Vec<PlaylistTrack> = r
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .filter(|track| track.playlist_id == playlist.id)
        .collect();
    playlist_tracks.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.id.cmp(&right.id))
    });

    let tracks = playlist_tracks
        .into_iter()
        .filter_map(|playlist_track| {
            r.get()
                .primary::<TrackDbItem>(playlist_track.track_id)
                .ok()
                .flatten()
        })
        .collect();
    Ok(PlaylistData { playlist, tracks })
}

pub fn list_playlists(db: &Database) -> Result<Vec<PlaylistData>> {
    let r = db.r_transaction()?;
    let mut playlists: Vec<Playlist> = r
        .scan()
        .primary::<Playlist>()?
        .all()?
        .filter_map(|item| item.ok())
        .collect();
    playlists.sort_by_key(|playlist| playlist.created_at);
    playlists
        .into_iter()
        .map(|playlist| playlist_data_from_transaction(&r, playlist))
        .collect()
}

pub fn create_playlist(db: &Database, name: String) -> Result<PlaylistData> {
    let name = validate_playlist_name(name)?;
    let playlist = Playlist {
        id: Uuid::new_v4().to_string(),
        name,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
    };
    let rw = db.rw_transaction()?;
    rw.insert::<Playlist>(playlist.clone())?;
    rw.commit()?;
    let r = db.r_transaction()?;
    playlist_data_from_transaction(&r, playlist)
}

pub fn rename_playlist(db: &Database, id: String, name: String) -> Result<PlaylistData> {
    let name = validate_playlist_name(name)?;
    let rw = db.rw_transaction()?;
    let mut playlist = rw
        .get()
        .primary::<Playlist>(id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", id))?;
    playlist.name = name;
    rw.remove(playlist.clone())?;
    rw.insert::<Playlist>(playlist.clone())?;
    rw.commit()?;
    let r = db.r_transaction()?;
    playlist_data_from_transaction(&r, playlist)
}

pub fn delete_playlist(db: &Database, id: String) -> Result<()> {
    let rw = db.rw_transaction()?;
    let playlist = rw
        .get()
        .primary::<Playlist>(id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", id))?;
    let tracks: Vec<PlaylistTrack> = rw
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .filter(|track| track.playlist_id == id)
        .collect();
    for track in tracks {
        rw.remove(track)?;
    }
    rw.remove(playlist)?;
    rw.commit()?;
    Ok(())
}

pub fn add_playlist_track(
    db: &Database,
    playlist_id: String,
    track_id: String,
) -> Result<PlaylistData> {
    let r = db.r_transaction()?;
    let playlist = r
        .get()
        .primary::<Playlist>(playlist_id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", playlist_id))?;
    if r.get().primary::<TrackDbItem>(track_id.clone())?.is_none() {
        return Err(anyhow!("track not found: {}", track_id));
    }
    let existing: Vec<PlaylistTrack> = r
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .filter(|track| track.playlist_id == playlist_id && track.track_id == track_id)
        .collect();
    if !existing.is_empty() {
        return playlist_data_from_transaction(&r, playlist);
    }
    let position = r
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .filter(|track| track.playlist_id == playlist_id)
        .map(|track| track.position)
        .max()
        .map_or(0, |position| position + 1);
    drop(r);

    let playlist_track = PlaylistTrack {
        id: Uuid::new_v4().to_string(),
        playlist_id: playlist_id.clone(),
        track_id,
        position,
    };
    let rw = db.rw_transaction()?;
    rw.insert::<PlaylistTrack>(playlist_track)?;
    rw.commit()?;
    let r = db.r_transaction()?;
    let playlist = r
        .get()
        .primary::<Playlist>(playlist_id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", playlist_id))?;
    playlist_data_from_transaction(&r, playlist)
}

pub fn remove_playlist_track(
    db: &Database,
    playlist_id: String,
    track_id: String,
) -> Result<PlaylistData> {
    let r = db.r_transaction()?;
    let playlist = r
        .get()
        .primary::<Playlist>(playlist_id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", playlist_id))?;
    if r.get().primary::<TrackDbItem>(track_id.clone())?.is_none() {
        return Err(anyhow!("track not found: {}", track_id));
    }
    let playlist_track = r
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .find(|track| track.playlist_id == playlist_id && track.track_id == track_id)
        .ok_or_else(|| anyhow!("track not in playlist: {}", track_id))?;
    drop(r);

    let rw = db.rw_transaction()?;
    rw.remove(playlist_track)?;
    rw.commit()?;
    let r = db.r_transaction()?;
    let playlist = r
        .get()
        .primary::<Playlist>(playlist_id)?
        .ok_or_else(|| anyhow!("playlist not found"))?;
    playlist_data_from_transaction(&r, playlist)
}

pub fn reorder_playlist_track(
    db: &Database,
    playlist_id: String,
    track_id: String,
    position: i64,
) -> Result<PlaylistData> {
    let rw = db.rw_transaction()?;
    let playlist = rw
        .get()
        .primary::<Playlist>(playlist_id.clone())?
        .ok_or_else(|| anyhow!("playlist not found: {}", playlist_id))?;
    let mut playlist_tracks: Vec<PlaylistTrack> = rw
        .scan()
        .primary::<PlaylistTrack>()?
        .all()?
        .filter_map(|item| item.ok())
        .filter(|track| track.playlist_id == playlist_id)
        .collect();
    playlist_tracks.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.id.cmp(&right.id))
    });

    let current_index = playlist_tracks
        .iter()
        .position(|track| track.track_id == track_id)
        .ok_or_else(|| anyhow!("track not in playlist: {}", track_id))?;
    if position < 0 || position >= playlist_tracks.len() as i64 {
        return Err(anyhow!(
            "position out of bounds: {} (playlist has {} tracks)",
            position,
            playlist_tracks.len()
        ));
    }
    let moved_track = playlist_tracks.remove(current_index);
    playlist_tracks.insert(position as usize, moved_track);

    for track in &playlist_tracks {
        rw.remove(track.clone())?;
    }
    for (index, mut track) in playlist_tracks.into_iter().enumerate() {
        track.position = index as i64;
        rw.insert::<PlaylistTrack>(track)?;
    }
    rw.commit()?;

    let r = db.r_transaction()?;
    let playlist = r
        .get()
        .primary::<Playlist>(playlist_id)?
        .ok_or_else(|| anyhow!("playlist not found"))?;
    playlist_data_from_transaction(&r, playlist)
}


//TODO: 获取出错的处理逻辑现在是直接filter，感觉得打一些log.
pub fn list_liked_track(db: &Database) -> Result<Vec<TrackDbItem>> {
    let r = db.r_transaction()?;
    let mut table_result: Vec<LikedTrack> = r
        .scan()
        .primary::<LikedTrack>()?
        .all()?
        .filter_map(|item| match item {
            Ok(d) => Some(d),
            Err(_) => None,
        })
        .collect();
    table_result.sort_by(|a, b| a.added_at.cmp(&b.added_at));
    table_result.reverse();
    let track_list: Vec<TrackDbItem> = table_result
        .iter()
        .filter_map(|d| match r.get().primary::<TrackDbItem>(&*d.id) {
            Ok(liked_item_box) => match liked_item_box {
                Some(liked_item) => match r.get().primary::<TrackDbItem>(&*liked_item.id) {
                    Ok(track) => track,
                    Err(_) => None,
                },
                None => None,
            },
            Err(_) => None,
        })
        .collect();
    Ok(track_list)
}
fn _add_track<T: ToInput + DbCheck>(rw: &RwTransaction, item: T) -> Result<()> {
    match item.exists_in_db(rw)? {
        true => return Ok(()),
        false => rw.insert::<T>(item)?,
    };
    Ok(())
}
fn _get_track_by_id<T: ToInput>(r: &RTransaction, id: String) -> Result<Option<T>> {
    let item = r.get().primary::<T>(id)?;
    Ok(item)
}

pub fn get_track_by_id(db: &Database, id: String) -> Result<Option<TrackDbItem>> {
    let r = db.r_transaction()?;
    _get_track_by_id::<TrackDbItem>(&r, id)
}

//TODO: 加log
fn _delete_track_by_id<T: ToInput>(rw: &RwTransaction, id: String) -> Result<()> {
    let to_deleted_item = rw.get().primary::<T>(id)?;
    match to_deleted_item {
        Some(item) => {
            rw.remove(item)?;
            Ok(())
        }
        None => Ok(()),
    }
}

fn _get_liked_track_by_id(db: &Database, id: String) -> Result<Option<TrackDbItem>> {
    let r = db.r_transaction()?;
    let track = _get_track_by_id::<TrackDbItem>(&r, id)?;
    Ok(track)
}
pub fn list_recent_track(db: &Database) -> Result<Vec<TrackDbItem>> {
    let r = db.r_transaction()?;
    let mut table_result: Vec<RecentTrack> = r
        .scan()
        .primary::<RecentTrack>()?
        .all()?
        .filter_map(|item| match item {
            Ok(d) => Some(d),
            Err(_) => None,
        })
        .collect();
    table_result.sort_by(|a, b| a.added_at.cmp(&b.added_at));
    table_result.reverse();
    let track_list: Vec<TrackDbItem> = table_result
        .iter()
        .filter_map(|d| match r.get().primary::<TrackDbItem>(&*d.id) {
            Ok(recent_item_box) => match recent_item_box {
                Some(recent_item) => Some(recent_item),
                None => None,
            },
            Err(_) => None,
        })
        .collect();
    Ok(track_list)
}

pub fn toggle_liked_by_id(db: &Database, id: String) -> Result<()> {
    let track = _get_liked_track_by_id(db, id.clone())?;
    match track {
        Some(_) => _remove_liked_track(db, id)?,
        None => {
            let track = _get_track_by_id(&db.r_transaction()?, id.clone())?
                .ok_or_else(|| anyhow!("track not found: {}", id))?;
            _add_liked_track(db, track)?;
        }
    };
    Ok(())
}

fn _add_liked_track(db: &Database, track: TrackDbItem) -> Result<()> {
    let liked_track = LikedTrack {
        id: track.id.clone(),
        added_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
    };
    let rw = db.rw_transaction()?;
    _add_track::<LikedTrack>(&rw, liked_track)?;
    _add_track::<TrackDbItem>(&rw, track)?;
    rw.commit()?;
    Ok(())
}
//TODO: 这么写不太优雅，需要重构
fn _remove_liked_track(db: &Database, id: String) -> Result<()> {
    let rw = db.rw_transaction()?;
    _delete_track_by_id::<LikedTrack>(&rw, id.clone())?;
    let r = db.r_transaction()?;
    match _get_track_by_id::<RecentTrack>(&r, id.clone())? {
        Some(_) => (),
        None => _delete_track_by_id::<TrackDbItem>(&rw, id.clone())?,
    };
    rw.commit()?;
    Ok(())
}

//TODO: 这么写不太优雅，需要重构
pub fn _remove_recent_track(db: &Database, id: String) -> Result<()> {
    let rw = db.rw_transaction()?;
    let r = db.r_transaction()?;
    _delete_track_by_id::<RecentTrack>(&rw, id.clone())?;
    match _get_track_by_id::<LikedTrack>(&r, id.clone())? {
        Some(_) => (),
        None => _delete_track_by_id::<TrackDbItem>(&rw, id.clone())?,
    };
    rw.commit()?;
    Ok(())
}

pub fn _clear_recent_track(db: &Database) -> Result<()> {
    let track_list = list_recent_track(db)?;
    let rw = db.rw_transaction()?;
    let r = db.r_transaction()?;
    for data in track_list {
        match _get_track_by_id::<LikedTrack>(&r, data.id.clone())? {
            Some(_) => (),
            None => _delete_track_by_id::<TrackDbItem>(&rw, data.id.clone())?,
        };
        _delete_track_by_id::<RecentTrack>(&rw, data.id.clone())?;
    }
    rw.commit()?;
    Ok(())
}
pub fn add_recent_track(db: &Database, track: TrackDbItem) -> Result<()> {
    match _get_track_by_id::<RecentTrack>(&db.r_transaction()?, track.id.clone())? {
        Some(mut t) => {
            let rw = db.rw_transaction()?;
            _delete_track_by_id::<RecentTrack>(&rw, t.id.clone())?;
            t.added_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
            _add_track::<RecentTrack>(&rw, t)?;
            rw.commit()?;
            return Ok(());
        }
        None => (),
    };

    let rw = db.rw_transaction()?;
    let total_count = rw.len().primary::<RecentTrack>()?;
    if total_count >= MAX_RECENT_TRACK_COUNT as u64 {
        let mut recent_tracks = list_recent_track(db)?;
        for _ in 0..(total_count - MAX_RECENT_TRACK_COUNT as u64) + 1 {
            let to_removed = recent_tracks.pop().unwrap();
            match _get_track_by_id::<LikedTrack>(&db.r_transaction()?, to_removed.id.clone())? {
                Some(_) => (),
                None => _delete_track_by_id::<TrackDbItem>(&rw, to_removed.id.clone())?,
            };
            _delete_track_by_id::<RecentTrack>(&rw, to_removed.id.clone())?;
        }
    }

    // 添加逻辑
    _add_track::<TrackDbItem>(&rw, track.clone())?;
    let new_recent = RecentTrack {
        id: track.id.clone(),
        added_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
    };
    _add_track::<RecentTrack>(&rw, new_recent)?;
    rw.commit()?;
    Ok(())
}
pub static TRACK_MODEL: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<TrackDbItem>().unwrap();
    models.define::<LikedTrack>().unwrap();
    models.define::<RecentTrack>().unwrap();
    models.define::<Playlist>().unwrap();
    models.define::<PlaylistTrack>().unwrap();
    models
});

mod test {

    use super::*;

    #[test]
    fn test_uuid() {
        let url = "https://www.baidu.com";
        let uuid_a = super::get_uuid_from_url(url);
        let uuid_b = super::get_uuid_from_url(url);
        assert_eq!(uuid_a.to_string(), uuid_b.to_string());
    }
    fn _get_track_list_length(r: &RTransaction) -> Result<usize> {
        let table_result = r
            .scan()
            .primary::<TrackDbItem>()?
            .all()
            .unwrap()
            .filter_map(|d| match d {
                Ok(r) => Some(r),
                Err(e) => {
                    dbg!(e);
                    None
                }
            })
            .count();
        Ok(table_result)
    }
    fn _get_track_list(r: &RTransaction) -> Result<Vec<TrackDbItem>> {
        let table_result: Vec<TrackDbItem> = r
            .scan()
            .primary::<TrackDbItem>()?
            .all()
            .unwrap()
            .filter_map(|d| match d {
                Ok(r) => Some(r),
                Err(e) => {
                    dbg!(e);
                    None
                }
            })
            .collect();
        Ok(table_result)
    }
    #[test]
    fn test_debug_localdb() {
        let path = "./test_debug_local.db";
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }
        let db = native_db::Builder::new()
            .create(&super::TRACK_MODEL, path)
            .unwrap();
        let r = db.r_transaction().unwrap();
        let track_list = _get_track_list(&r).unwrap();
        println!("{:?}", track_list);
        drop(r);
        drop(db);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_crud_in_track() {
        // init
        if std::path::Path::new("./test_track.db").exists() {
            std::fs::remove_file("./test_track.db").unwrap();
        }
        let db = native_db::Builder::new()
            .create(&super::TRACK_MODEL, "./test_track.db")
            .unwrap();
        // create
        let item = super::TrackDbItem {
            title: "title".to_string(),
            artist: "artist".to_string(),
            cover_url: "cover_url".to_string(),
            duration: 10.0,
            id: 0.to_string(),
            src: "src".to_string(),
            meta: TrackMeta {
                source: "".to_string(),
                value: crate::types::MetaValue::Wechat("".to_string()),
            },
        };

        add_recent_track(&db, item.clone()).unwrap();
        _add_liked_track(&db, item.clone()).unwrap();
        {
            let recent_track_list = list_recent_track(&db).unwrap();
            assert_eq!(recent_track_list.iter().len(), 1);
            let liked_track_list = list_liked_track(&db).unwrap();
            assert_eq!(liked_track_list.iter().len(), 1);
            let r = db.r_transaction().unwrap();
            let table_length = _get_track_list_length(&r).unwrap();
            assert_eq!(table_length, 1);
        }
        _remove_liked_track(&db, item.id.clone()).unwrap();
        {
            let recent_track_list = list_recent_track(&db).unwrap();
            assert_eq!(recent_track_list.iter().len(), 1);
            let liked_track_list = list_liked_track(&db).unwrap();
            assert_eq!(liked_track_list.iter().len(), 0);
            let r = db.r_transaction().unwrap();
            let table_length = _get_track_list_length(&r).unwrap();
            assert_eq!(table_length, 1);
        }
        for i in 1..120 {
            // sleep(time::Duration::from_millis(1000));
            let mut to_insert_item = item.clone();
            to_insert_item.id = i.to_string();
            add_recent_track(&db, to_insert_item.clone()).unwrap();
            let rw = db.rw_transaction().unwrap();
            let recent_entry: RecentTrack = rw
                .get()
                .primary::<RecentTrack>(to_insert_item.id.clone())
                .unwrap()
                .unwrap(); // 刚插进去的，肯定unwrap能出来
            let mut mock_entry = recent_entry.clone();
            mock_entry.added_at = i as i64;
            rw.update(recent_entry, mock_entry).unwrap();
            rw.commit().unwrap();
            if i % 10 == 0 {
                _add_liked_track(&db, to_insert_item.clone()).unwrap();
            }
        }

        {
            let recent_track_list = list_recent_track(&db).unwrap();
            assert_eq!(recent_track_list.iter().len(), 100);
            let liked_track_list = list_liked_track(&db).unwrap();
            assert_eq!(liked_track_list.iter().len(), 11);
            let r = db.r_transaction().unwrap();
            let table_length = _get_track_list_length(&r).unwrap();

            assert_eq!(table_length, 102);
        }

        // read
        // let r = db.r_transaction().unwrap();
        // let table_result: Vec<_> = r
        //     .scan()
        //     .primary::<TrackDbItem>()
        //     .unwrap()
        //     .all()
        //     .unwrap()
        //     .collect();
        // for item in table_result.iter() {
        //     println!("{:?}", *item);
        // }
        // // remove
        // let rw = db.rw_transaction().unwrap();
        // for item in table_result {
        //     rw.remove(item.unwrap()).unwrap();
        // }
        // rw.commit().unwrap();
    }
    #[test]
    fn upsert_playable_entry_preserves_stable_source_without_recent() {
        let path = "./test_upsert.db";
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }
        let db = native_db::Builder::new()
            .create(&super::TRACK_MODEL, path)
            .unwrap();
        let entry = crate::playback::model::PlayableEntry {
            view: crate::types::TrackView::new(
                "title".to_string(),
                "artist".to_string(),
                "cover".to_string(),
                12.0,
                "yt:test-video".to_string(),
            ),
            source_ref: crate::playback::model::SourceRef::Youtube {
                video_id: "test-video".to_string(),
            },
        };

        let item = upsert_track_entry(&db, &entry, None).unwrap();
        assert_eq!(item.src, "");
        assert_eq!(
            item.meta.value,
            crate::types::MetaValue::Extractor("yt:test-video".to_string())
        );
        assert!(list_recent_track(&db).unwrap().is_empty());

        let cached = std::path::Path::new("/tmp/test-upsert.audio");
        let updated = upsert_track_entry(&db, &entry, Some(cached)).unwrap();
        assert_eq!(updated.src, cached.to_string_lossy());
        assert_eq!(
            get_track_by_id(&db, entry.view.id.clone()).unwrap(),
            Some(updated)
        );

        drop(db);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn playlist_lifecycle_preserves_track_data() {
        let path = std::env::temp_dir().join(format!(
            "music-demo-playlist-{}.db",
            Uuid::new_v4()
        ));
        let path_string = path.to_string_lossy().into_owned();
        let db = native_db::Builder::new()
            .create(&super::TRACK_MODEL, &path_string)
            .unwrap();
        let track = super::TrackDbItem {
            title: "Playlist song".to_string(),
            artist: "Artist".to_string(),
            cover_url: "cover".to_string(),
            duration: 42.0,
            id: "playlist-track".to_string(),
            src: String::new(),
            meta: TrackMeta {
                source: "extractor".to_string(),
                value: crate::types::MetaValue::Extractor("yt:playlist-track".to_string()),
            },
        };
        let rw = db.rw_transaction().unwrap();
        rw.insert::<super::TrackDbItem>(track.clone()).unwrap();
        rw.commit().unwrap();

        let playlist = super::create_playlist(&db, "Focus".to_string()).unwrap();
        assert!(super::create_playlist(&db, " ".to_string()).is_err());
        let populated =
            super::add_playlist_track(&db, playlist.playlist.id.clone(), track.id.clone())
                .unwrap();
        assert_eq!(populated.tracks, vec![track.clone()]);

        super::delete_playlist(&db, playlist.playlist.id).unwrap();
        assert!(super::list_playlists(&db).unwrap().is_empty());
        assert_eq!(
            super::get_track_by_id(&db, track.id.clone()).unwrap(),
            Some(track)
        );

        drop(db);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn playlist_reorder_keeps_positions_contiguous() {
        let path = std::env::temp_dir().join(format!(
            "music-demo-playlist-order-{}.db",
            Uuid::new_v4()
        ));
        let path_string = path.to_string_lossy().into_owned();
        let db = native_db::Builder::new()
            .create(&super::TRACK_MODEL, &path_string)
            .unwrap();
        let tracks: Vec<_> = (0..3)
            .map(|index| super::TrackDbItem {
                title: format!("Song {index}"),
                artist: "Artist".to_string(),
                cover_url: "cover".to_string(),
                duration: 30.0,
                id: format!("playlist-order-{index}"),
                src: String::new(),
                meta: TrackMeta {
                    source: "extractor".to_string(),
                    value: crate::types::MetaValue::Extractor(format!("yt:playlist-order-{index}")),
                },
            })
            .collect();
        let rw = db.rw_transaction().unwrap();
        for track in &tracks {
            rw.insert::<super::TrackDbItem>(track.clone()).unwrap();
        }
        rw.commit().unwrap();

        let playlist = super::create_playlist(&db, "Order".to_string()).unwrap();
        for track in &tracks {
            super::add_playlist_track(&db, playlist.playlist.id.clone(), track.id.clone())
                .unwrap();
        }
        let reordered = super::reorder_playlist_track(
            &db,
            playlist.playlist.id.clone(),
            tracks[0].id.clone(),
            2,
        )
        .unwrap();
        let ordered_ids: Vec<_> = reordered
            .tracks
            .iter()
            .map(|track| track.id.as_str())
            .collect();
        assert_eq!(
            ordered_ids,
            vec!["playlist-order-1", "playlist-order-2", "playlist-order-0"]
        );
        assert!(super::reorder_playlist_track(
            &db,
            playlist.playlist.id,
            tracks[0].id.clone(),
            3
        )
        .is_err());

        drop(db);
        std::fs::remove_file(path).unwrap();
    }
}
