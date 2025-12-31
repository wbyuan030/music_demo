use anyhow::anyhow;
use anyhow::Result as AnyResult;
use native_db::Database;
use rodio::buffer::SamplesBuffer;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamBuilder;
use rodio::Sink;
use rodio::Source;
use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use std::time::Duration;
use tauri_plugin_http::reqwest;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use tokio::sync::{broadcast, Mutex};

use crate::storage;
use crate::storage::TrackDbItem;
use crate::storage::_add_recent_track;
use crate::storage::get_track_db_item;
use crate::types::Track;
use crate::types::TrackSrc;

pub struct TrackStore {
    pub tracks: Arc<SyncMutex<HashMap<String, Track>>>,
}

pub struct Music {
    pub event_sender: Sender<MusicState>,
    pub sink: Arc<Mutex<Sink>>,
    #[allow(dead_code)]
    stream_handle: OutputStream,
}

#[derive(Debug, Clone)]
pub enum MusicState {
    Play(
        String,
        Arc<SyncMutex<Option<oneshot::Sender<Result<(), String>>>>>,
    ),
    Recovery,
    Pause,
    Volume(f32),
    Quit,
    Seek(f32),
}

impl Music {
    pub fn new(
        track_map: Arc<SyncMutex<HashMap<String, Track>>>,
        track_db: Arc<SyncMutex<Database<'static>>>,
    ) -> Self {
        let (event_sender, mut event_receiver) = broadcast::channel(100);
        let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();

        let sink = Arc::new(Mutex::new(Sink::connect_new(&stream_handle.mixer())));
        let sink_clone = Arc::clone(&sink);
        let track_map_clone = Arc::clone(&track_map);
        let track_db_clone = Arc::clone(&track_db);
        // spawn a thread to handle the music events
        tokio::spawn(async move {
            // receive events from the channel
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    MusicState::Play(id, tx) => {
                        {
                            let sink = sink_clone.lock().await;
                            sink.clear();
                        }
                        let track = track_map_clone.lock().unwrap().get(&id).unwrap().clone();

                        let track_data = get_online_track(id, track, track_db_clone.clone())
                            .await
                            .unwrap();
                        let result = play(track_data, sink_clone.clone()).await;
                        if let Some(sender) = tx.lock().unwrap().take() {
                            let _ = sender.send(result);
                        }
                    }
                    MusicState::Seek(time) => {
                        let sink = sink_clone.lock().await;
                        if sink.empty() {
                            continue;
                        }
                        let pos = Duration::from_secs_f32(time);
                        match sink.try_seek(pos) {
                            Ok(_) => (),
                            Err(e) => eprintln!("seek error: {}", e),
                        }
                    }
                    MusicState::Recovery => {
                        let sink = sink_clone.lock().await;
                        sink.play();
                    }
                    MusicState::Pause => {
                        let sink = sink_clone.lock().await;
                        sink.pause();
                    }
                    MusicState::Volume(volume) => {
                        let sink = sink_clone.lock().await;
                        sink.set_volume(volume / 50.0);
                    }
                    MusicState::Quit => {
                        let sink = sink_clone.lock().await;
                        sink.stop();
                    }
                }
            }
        });

        Self {
            event_sender,
            sink,
            stream_handle,
        }
    }
}
fn get_cache_dir() -> AnyResult<PathBuf> {
    let mut dir = std::env::temp_dir();
    dir.push("music_cache");
    if !dir.exists() {
        fs::create_dir_all(dir.clone())?
    }
    Ok(dir)
}
async fn get_online_track(
    id: String,
    track: Track,
    db: Arc<SyncMutex<Database<'static>>>,
) -> AnyResult<Vec<u8>> {
    let db_item = {
        let db_touse = db.lock().unwrap();
        get_track_db_item(&db_touse, id.clone())?
    };
    match db_item {
        Some(item) => {
            let path = PathBuf::from(item.src);
            let data = fs::read(path)?;
            return Ok(data);
        }
        None => (),
    }
    let track_src = track.src.clone();
    let cache_dir = get_cache_dir()?;
    let file_name = format!("{}.bin", id);
    let file_path = cache_dir.join(file_name);
    let req = match track_src {
        TrackSrc::Url(url) => reqwest::Client::default().get(url),
        TrackSrc::UrlWithHead(url, head) => reqwest::Client::builder()
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
    };
    {
        let db_touse = db.lock().unwrap();
        _add_recent_track(&db_touse, track_db_item)?;
    };
    Ok(data)
}

async fn play(bytes: Vec<u8>, sink: Arc<Mutex<Sink>>) -> Result<(), String> {
    let source = {
        let cursor = std::io::Cursor::new(bytes);
        let decoder = Decoder::new(cursor).map_err(|e| format!("decode error: {}", e))?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();

        let samples: Vec<f32> = decoder.collect();

        // 创建支持 Seek 的 Buffer
        Ok::<_, String>(SamplesBuffer::new(channels, sample_rate, samples))
    }
    .unwrap();

    let sink_lock = sink.lock().await;
    sink_lock.append(source);
    if sink_lock.is_paused() {
        sink_lock.play();
    }

    Ok(())
}

#[tauri::command]
pub async fn handle_event(
    sender: tauri::State<'_, Sender<MusicState>>,
    event: String,
) -> Result<(), String> {
    let event: serde_json::Value =
        serde_json::from_str(&event).map_err(|e| format!("JSON解析错误:{}", e))?;

    if let Some(act) = event["action"].as_str() {
        match act {
            "play" => {
                let (tx, rx) = oneshot::channel();
                let tx_wrapper = Arc::new(SyncMutex::new(Some(tx)));

                event["id"]
                    .as_str()
                    .map(|id| sender.send(MusicState::Play(id.to_owned(), tx_wrapper)));
                match rx.await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Error in play: {}", e)),
                }
            }
            "recovery" => {
                let _ = Some(sender.send(MusicState::Recovery));
                Ok(())
            }
            "pause" => {
                let _ = Some(sender.send(MusicState::Pause));
                Ok(())
            }
            "volume" => {
                let _ = event["volume"]
                    .as_f64()
                    .map(|vol| sender.send(MusicState::Volume(vol as f32)));
                Ok(())
            }
            "quit" => {
                let _ = Some(sender.send(MusicState::Quit));
                Ok(())
            }
            "seek" => {
                let _ = event["time"]
                    .as_f64()
                    .map(|t| sender.send(MusicState::Seek(t as f32)));
                Ok(())
            }
            _ => Ok(()),
        }
    } else {
        Err(format!("Unknown action"))
    }
}
