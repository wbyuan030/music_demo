use anyhow::Result;
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::*;
use native_model::native_model;
use native_model::Model;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::{uuid, Uuid};

use crate::types::{Track, TrackMeta};

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
    pub async fn to_track(&self) -> Option<Track> {
        let src = match self.meta.parse().await {
            Some(d) => d,
            None => return None,
        };
        return Some(Track {
            src: src,
            title: self.title.clone(),
            duration: self.duration,
            artist: self.artist.clone(),
            cover_url: self.cover_url.clone(),
            meta: self.meta.clone(),
        });
    }
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
    let track = _get_liked_track_by_id(&db, id.clone())?;
    match track {
        Some(_) => _remove_liked_track(&db, id.clone())?,
        None => _add_liked_track(
            &db,
            _get_track_by_id(&db.r_transaction()?, id.clone())?.unwrap(),
        )?,
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
pub static TRACK_MODEL: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<TrackDbItem>().unwrap();
    models.define::<LikedTrack>().unwrap();
    models.define::<RecentTrack>().unwrap();
    models
});

mod test {

    use super::*;
    use crate::storage::TrackDbItem;

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
        let mut db = Builder::new()
            .create(&super::TRACK_MODEL, "./local.db")
            .unwrap();
        let r = db.r_transaction().unwrap();
        let track_list = _get_track_list(&r).unwrap();
        println!("{:?}", track_list);
    }

    #[test]
    fn test_crud_in_track() {
        // init
        if exists("./test_track.db").unwrap() {
            remove_file("./test_track.db").unwrap();
        }
        let mut db = Builder::new()
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
                value: MetaValue::Wechat("".to_string()),
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
}
