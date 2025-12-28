use rodio::buffer::SamplesBuffer;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamBuilder;
use rodio::Sink;
use rodio::Source;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use std::time::Duration;
use tauri_plugin_http::reqwest;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use tokio::sync::{broadcast, Mutex};

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
    pub fn new(track_map: Arc<SyncMutex<HashMap<String, Track>>>) -> Self {
        let (event_sender, mut event_receiver) = broadcast::channel(100);
        let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();

        let sink = Arc::new(Mutex::new(Sink::connect_new(&stream_handle.mixer())));
        let sink_clone = Arc::clone(&sink);
        let track_map_clone = Arc::clone(&track_map);
        // spawn a thread to handle the music events
        tokio::spawn(async move {
            // receive events from the channel
            while let Ok(event) = event_receiver.recv().await {
                println!("{:?}", event);
                match event {
                    MusicState::Play(id, tx) => {
                        {
                            let sink = sink_clone.lock().await;
                            sink.clear();
                        }
                        let track_src = track_map_clone
                            .lock()
                            .unwrap()
                            .get(&id)
                            .unwrap()
                            .src
                            .clone();
                        let result = online_play(track_src, sink_clone.clone()).await;
                        if let Some(sender) = tx.lock().unwrap().take() {
                            let _ = sender.send(result);
                        }
                    }
                    MusicState::Seek(time) => {
                        let sink = sink_clone.lock().await;
                        if sink.empty() {
                            println!("Debug: Sink is empty, cannot seek yet.");
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

async fn online_play(track_src: TrackSrc, sink: Arc<Mutex<Sink>>) -> Result<(), String> {
    // TODO:缓存

    let req = match track_src {
        TrackSrc::Url(url) => reqwest::Client::default().get(url),
        TrackSrc::UrlWithHead(url, head) => reqwest::Client::builder()
            .default_headers(head)
            .build()
            .unwrap()
            .get(url),
    };
    let response = req
        .send()
        .await
        .map_err(|e| format!("request error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("response status fail: {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read response data error: {}", e))?;
    let source = {
        let cursor = std::io::Cursor::new(bytes);
        let decoder = Decoder::new(cursor).map_err(|e| format!("decode error: {}", e))?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();

        // 关键步骤：convert_samples() 会把所有数据解压出来
        // collect() 会把它们存入 Vec<f32>
        let samples: Vec<f32> = decoder.collect();

        // 创建支持完美 Seek 的 Buffer
        Ok::<_, String>(SamplesBuffer::new(channels, sample_rate, samples))
    }
    .unwrap();
    // match Decoder::new(cursor) {
    // Ok(s) => s,
    // Err(e) => {
    //     return Err(format!("decode online song error: {}", e));
    // }
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
