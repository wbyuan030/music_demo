use rodio::OutputStream;
use rodio::OutputStreamBuilder;
use rodio::Sink;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};
use tracing::error;

use crate::audio_quality::probe::PlaybackProbe;
use crate::global::get_track_state;
use crate::music_handler::utils::parse_track_request;
use crate::music_handler::utils::play;

#[derive(Debug, Clone)]
pub enum MusicState {
    Play(String),
    Recovery,
    Pause,
    Volume(f32),
    Quit,
    Seek(f32),
}

pub struct MusicHandler {
    pub event_sender: Sender<MusicState>,
    #[allow(dead_code)]
    pub sink: Arc<Mutex<Sink>>,
    #[allow(dead_code)]
    stream_handle: OutputStream,
}
impl MusicHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        let (event_sender, event_receiver) = broadcast::channel(100);
        let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();

        let sink = Arc::new(Mutex::new(Sink::connect_new(&stream_handle.mixer())));

        let probe = Arc::new(Mutex::new(None::<PlaybackProbe>));

        let progress_sink = Arc::clone(&sink);
        let handler_sink = Arc::clone(&sink);
        let app_handle_event = app_handle.clone();
        let app_progress_cloned = app_handle.clone();
        let probe_handler = Arc::clone(&probe);
        let probe_progress = Arc::clone(&probe);
        Self::spawn_handle_event(app_handle_event, event_receiver, handler_sink, probe_handler);
        Self::spawn_progress(app_progress_cloned, progress_sink, probe_progress);
        Self {
            event_sender,
            sink,
            stream_handle,
        }
    }

    fn spawn_handle_event(
        app_handle: AppHandle,
        mut event_receiver: broadcast::Receiver<MusicState>,
        sink: Arc<Mutex<Sink>>,
        probe: Arc<Mutex<Option<PlaybackProbe>>>,
    ) {
        let track_map = get_track_state().unwrap();
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    MusicState::Play(id) => {
                        // 重置探针
                        *probe.lock().await = Some(PlaybackProbe::new());

                        sink.lock().await.clear();
                        let track = match track_map.lock().await.get(&id) {
                            Some(track) => Some(track.clone()),
                            None => None,
                        };
                        let track_data = match parse_track_request(id, track, app_handle.clone()).await {
                            Ok(data) => data,
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        };
                        let _ = match play(track_data, sink.clone()).await {
                            Ok(()) => app_handle.emit("play_start", ()),
                            Err(e) => app_handle.emit("play_start", e.to_string()),
                        };
                    }
                    MusicState::Seek(time) => {
                        if sink.lock().await.empty() {
                            continue;
                        }
                        let pos = Duration::from_secs_f32(time);
                        match sink.lock().await.try_seek(pos) {
                            Ok(_) => (),
                            Err(e) => eprintln!("seek error: {}", e),
                        }
                    }
                    MusicState::Recovery => {
                        sink.lock().await.play();
                    }
                    MusicState::Pause => {
                        sink.lock().await.pause();
                    }
                    MusicState::Volume(volume) => {
                        sink.lock().await.set_volume(volume / 50.0);
                    }
                    MusicState::Quit => {
                        sink.lock().await.stop();
                    }
                }
            }
        });
    }
    fn spawn_progress(app_handle: AppHandle, sink: Arc<Mutex<Sink>>, probe: Arc<Mutex<Option<PlaybackProbe>>>) {
        tokio::spawn(async move {
            let mut curr_state = false;
            loop {
                let sink = sink.lock().await;
                if sink.empty() {
                    if curr_state == true {
                        // 播放结束，emit 探针报告
                        if let Some(ref p) = *probe.lock().await {
                            let report = p.report();
                            if report.stall_count > 0 {
                                let _ = app_handle.emit("play_probe_report", report.to_json_string());
                            }
                        }
                        let _ = app_handle.emit("play_end", ());
                        curr_state = false
                    }
                } else {
                    let pos = sink.get_pos();
                    // 探针 tick
                    if let Some(ref mut p) = *probe.lock().await {
                        p.tick(pos);
                    }
                    let _ = app_handle.emit("play_progress", pos);
                    curr_state = true
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }
}
