use std::{fs, path::PathBuf, sync::Arc};

use tauri::Emitter;
use anyhow::{anyhow, Result};
use rodio::{Decoder, Sink, Source};
use tokio::sync::Mutex;

use crate::{
    global::get_db,
    storage::{add_recent_track, get_track_by_id, TrackDbItem},
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

pub async fn parse_track_request(id: String, mut track: Option<Track>, app_handle: tauri::AppHandle) -> Result<Vec<u8>> {
    let db = get_db();
    let db_item = { get_track_by_id(db, id.clone())? };
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
        app_handle.emit("db_tracks_changed", "recent").ok();
    };
    Ok(data)
}


pub async fn play(bytes: Vec<u8>, sink: Arc<Mutex<Sink>>) -> Result<(), String> {
    let (samples, channels, sample_rate) = tokio::task::spawn_blocking(move || {
        let cursor = std::io::Cursor::new(bytes);
        let decoder = Decoder::new(cursor).map_err(|e| format!("Decode error: {}", e))?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let samples: Vec<f32> = decoder.collect();
        Ok::<_, String>((samples, channels, sample_rate))
    })
    .await
    .map_err(|e| format!("Join error: {}", e))??;
    let sink_lock = sink.lock().await;
    sink_lock.append(rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples));
    if sink_lock.is_paused() {
        sink_lock.play();
    }
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;
    use rodio::Source;
    use std::fs;

    /// 扫描 /tmp/music_cache/ 中的缓存音频 → 解码 → 报告采样率/声道/时长
    /// 断言：解码不崩溃、采样率合理
    #[test]
    fn diagnose_cached_audio_quality() {
        let cache_dir = std::env::temp_dir().join("music_cache");
        if !cache_dir.exists() {
            println!("缓存目录不存在: {:?} — 先播放一首歌再跑此测试", cache_dir);
            return;
        }

        let mut found = 0;
        for entry in fs::read_dir(&cache_dir).expect("无法读取缓存目录") {
            let entry = entry.expect("读取目录项失败");
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "bin") {
                continue;
            }

            let bytes = fs::read(&path).expect("读取缓存文件失败");
            let cursor = std::io::Cursor::new(bytes);
            let decoder = match Decoder::new(cursor) {
                Ok(d) => d,
                Err(e) => {
                    println!("❌ 解码失败: {:?} — {}", path.file_name().unwrap(), e);
                    continue;
                }
            };

            let sample_rate = decoder.sample_rate();
            let channels = decoder.channels();
            let total_samples = decoder.count();
            let duration = total_samples as f64 / sample_rate as f64 / channels as f64;

            found += 1;
            println!(
                "📁 {:?} | {} Hz | {}ch | {:.1}s",
                path.file_name().unwrap(),
                sample_rate,
                channels,
                duration
            );

            assert!(total_samples > 0, "空音频文件");
            assert!(sample_rate > 0, "采样率为 0");
            assert!(channels >= 1 && channels <= 2, "异常声道数");
            assert!(duration > 0.1, "时长过短: {:.2}s", duration);
        }

        if found == 0 {
            println!("未找到 .bin 缓存文件 — 先播放一首歌再跑此测试");
        } else {
            println!("\n共分析 {} 个缓存文件", found);
        }
    }
}
