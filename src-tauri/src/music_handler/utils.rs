use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use rodio::{Decoder, Sink};
use tokio::sync::Mutex;

use crate::{
    global::get_db,
    storage::{add_recent_track, get_track_db_item, TrackDbItem},
    types::{Track, TrackSrc},
};
fn get_cache_dir() -> Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    dir.push("music_cache");
    if !dir.exists() {
        fs::create_dir_all(dir.clone())?
    }
    Ok(dir)
}

pub async fn parse_track_request(id: String, mut track: Option<Track>) -> Result<Vec<u8>> {
    let db = get_db();
    let db_item = { get_track_db_item(db, id.clone())? };
    match db_item {
        Some(item) => {
            let path = PathBuf::from(item.src.clone());
            match fs::read(path) {
                Ok(d) => return Ok(d),
                Err(e) => {
                    eprintln!("{}", e);
                    track = match item.to_track().await {
                        Some(d) => Some(d),
                        None => return Err(anyhow!("Cannot Parse Track From Db")),
                    };
                }
            };
        }
        None => (),
    }
    let track = match track {
        Some(track) => track,
        None => return Err(anyhow!("track not found")),
    };
    let track_src = track.src.clone();
    let cache_dir = get_cache_dir()?;
    let file_name = format!("{}.bin", id);
    let file_path = cache_dir.join(file_name);
    let req = match track_src {
        TrackSrc::Wechat(url) => reqwest::Client::default().get(url),
        TrackSrc::Bilibili(url, head) => reqwest::Client::builder()
            .default_headers(head)
            .build()
            .unwrap()
            .get(url),
    };
    let response = req.send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("response status fail: {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow!("read response data error: {}", e))?;
    let data = bytes.to_vec();
    fs::write(file_path.clone(), &data)?;
    let track_db_item = TrackDbItem {
        id: id.clone(),
        title: track.title.clone(),
        src: file_path.to_str().unwrap().to_string(),
        artist: track.artist.clone(),
        cover_url: track.cover_url.clone(),
        duration: track.duration,
        meta: track.meta.clone(),
    };
    {
        add_recent_track(db, track_db_item)?;
    };
    Ok(data)
}

pub async fn play(bytes: Vec<u8>, sink: Arc<Mutex<Sink>>) -> Result<(), String> {
    let source = tokio::task::spawn_blocking(move || {
        let cursor = std::io::Cursor::new(bytes);
        Decoder::new(cursor).map_err(|e| format!("Decode error: {}", e))
    })
    .await
    .map_err(|e| format!("Join error: {}", e))??;
    let sink_lock = sink.lock().await;
    sink_lock.append(source);
    if sink_lock.is_paused() {
        sink_lock.play();
    }
    Ok(())
}
