use std::{
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, broadcast::Sender, Mutex};
use tokio_util::sync::CancellationToken;

use crate::{
    audio_quality::instrumented_sink::InstrumentedSink,
    music_handler::status::PlaybackStatus,
    playback::{service::TrackSource, trace::PlaybackTrace, PlaybackService},
};

/// OutputStream is !Send on macOS due to cpal's CoreAudio internals,
/// but it's only created on the main thread and never touched again.
/// This wrapper safely enables Tauri's `manage` (`Send + Sync`).
struct SendOutputStream(OutputStream);
unsafe impl Send for SendOutputStream {}
unsafe impl Sync for SendOutputStream {}

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
    /// 后端播放状态（唯一状态源），供 `get_playback_state` 快照查询。
    pub status: Arc<PlaybackStatus>,
    /// 带可观测性装饰的 Sink；快照查询从这里读位置，探针逻辑在装饰器内部。
    pub telemetry: Arc<InstrumentedSink>,
    // 保持 OutputStream 存活；音频输出句柄释放即断音。
    _stream_handle: SendOutputStream,
}

impl MusicHandler {
    pub fn new(app_handle: AppHandle, playback: Arc<PlaybackService>) -> Self {
        let (event_sender, event_receiver) = broadcast::channel(100);
        let stream_handle = SendOutputStream(OutputStreamBuilder::open_default_stream().unwrap());
        log::info!(
            "audio output opened with config: {:?}",
            stream_handle.0.config()
        );
        let sink = Sink::connect_new(&stream_handle.0.mixer());
        sink.set_volume(1.0);
        let telemetry = Arc::new(InstrumentedSink::new(sink));
        let suppress_end = Arc::new(AtomicBool::new(false));

        // 下载期间（sink 为空）的 seek 先排队，加载完成后立即应用。
        let pending_seek = Arc::new(Mutex::new(None::<Duration>));
        let status = Arc::new(PlaybackStatus::new());

        Self::spawn_handle_event(
            app_handle.clone(),
            event_receiver,
            Arc::clone(&telemetry),
            Arc::clone(&suppress_end),
            Arc::clone(&pending_seek),
            Arc::clone(&status),
            playback,
        );
        Self::spawn_progress(
            app_handle,
            Arc::clone(&telemetry),
            suppress_end,
            Arc::clone(&status),
        );
        Self {
            event_sender,
            status,
            telemetry,
            _stream_handle: stream_handle,
        }
    }

    fn spawn_handle_event(
        app_handle: AppHandle,
        mut event_receiver: broadcast::Receiver<MusicState>,
        telemetry: Arc<InstrumentedSink>,
        suppress_end: Arc<AtomicBool>,
        pending_seek: Arc<Mutex<Option<Duration>>>,
        status: Arc<PlaybackStatus>,
        playback: Arc<PlaybackService>,
    ) {
        tokio::spawn(async move {
            let mut active_task: Option<(CancellationToken, tokio::task::JoinHandle<()>)> = None;
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    MusicState::Play(id) => {
                        // 新曲目重置排队 seek；锁顺序统一：pending_seek 先于 sink。
                        *pending_seek.lock().await = None;
                        status.set_loading(&id);
                        let trace = PlaybackTrace::new(&id);
                        trace.event("request", "status=received");
                        log::info!("playback request received: {} trace_id={}", id, trace.id());
                        if let Some((cancel, _)) = active_task.take() {
                            cancel.cancel();
                            trace.event("previous_task", "status=cancel_requested");
                        }
                        {
                            if !telemetry.empty() {
                                suppress_end.store(true, Ordering::Release);
                            }
                            telemetry.clear();
                        }
                        telemetry.begin_track(&id, trace.id());

                        let cancel = CancellationToken::new();
                        let task_cancel = cancel.clone();
                        let task_telemetry = Arc::clone(&telemetry);
                        let task_suppress_end = Arc::clone(&suppress_end);
                        let task_playback = Arc::clone(&playback);
                        let task_app_handle = app_handle.clone();
                        let task_trace = trace.clone();
                        let task_pending_seek = Arc::clone(&pending_seek);
                        let task_status = Arc::clone(&status);
                        let task = tokio::spawn(async move {
                            let source = match task_playback
                                .load_track_source_with_trace(&id, &task_cancel, &task_trace)
                                .await
                            {
                                Ok(source) => {
                                    task_trace.event("load", "status=ok");
                                    source
                                }
                                Err(crate::playback::resolver::PlaybackError::Cancelled) => {
                                    task_trace.event("load", "status=cancelled");
                                    return;
                                }
                                Err(error) => {
                                    task_trace.event(
                                        "load",
                                        format_args!("status=error error={}", error),
                                    );
                                    log::error!("playback load failed for {}: {}", id, error);
                                    task_status.set_failed(error.to_string());
                                    let _ = task_app_handle.emit("play_failed", error.to_string());
                                    return;
                                }
                            };

                            // 解码器从这里拿输入：完整文件，或边下边播的 spool（读会阻塞在下载边界）。
                            let (path, spool_state, spool_reader) = match source {
                                TrackSource::File(path) => (path, None, None),
                                TrackSource::Progressive {
                                    reader,
                                    state,
                                    path,
                                } => {
                                    task_trace.event("stream", "status=progressive");
                                    (path, Some(state), Some(reader))
                                }
                            };

                            let decode_started = Instant::now();
                            task_trace.event("decode", "status=start");
                            let decoder = match tokio::task::spawn_blocking({
                                let path = path.clone();
                                let spool_reader = spool_reader;
                                move || {
                                    // 两种来源统一成同一个 reader 类型。
                                    let reader: Box<dyn crate::playback::spool::ReadSeek> =
                                        match spool_reader {
                                            Some(reader) => Box::new(reader),
                                            None => {
                                                Box::new(File::open(&path).map_err(|error| {
                                                    format!("open audio cache: {}", error)
                                                })?)
                                            }
                                        };
                                    Decoder::try_from(std::io::BufReader::new(reader))
                                        .map_err(|error| format!("decode audio: {}", error))
                                }
                            })
                            .await
                            {
                                Ok(Ok(decoder)) => {
                                    if let Some(state) = &spool_state {
                                        state.mark_decoded();
                                    }
                                    let duration_ms =
                                        decoder.total_duration().map(|d| d.as_millis());
                                    task_trace.event(
                                        "decode",
                                        format_args!(
                                            "status=ok duration_ms={} channels={} sample_rate={} duration_elapsed_ms={}",
                                            duration_ms
                                                .map(|duration| duration.to_string())
                                                .unwrap_or_else(|| "unknown".to_string()),
                                            decoder.channels(),
                                            decoder.sample_rate(),
                                            decode_started.elapsed().as_millis()
                                        ),
                                    );
                                    decoder
                                }
                                Ok(Err(error)) => {
                                    if let Some(state) = &spool_state {
                                        state.mark_decode_failed();
                                    }
                                    let _ = tokio::fs::remove_file(&path).await;
                                    task_trace.event(
                                        "decode",
                                        format_args!(
                                            "status={} error={}",
                                            if task_cancel.is_cancelled() {
                                                "cancelled"
                                            } else {
                                                "error"
                                            },
                                            error
                                        ),
                                    );
                                    if !task_cancel.is_cancelled() {
                                        log::error!("playback decode failed for {}: {}", id, error);
                                        task_status.set_failed(error.to_string());
                                        let _ =
                                            task_app_handle.emit("play_failed", error.to_string());
                                    }
                                    return;
                                }
                                Err(error) => {
                                    if let Some(state) = &spool_state {
                                        state.mark_decode_failed();
                                    }
                                    let _ = tokio::fs::remove_file(&path).await;
                                    task_trace.event(
                                        "decode",
                                        format_args!(
                                            "status={} error=decoder_task_failed:{}",
                                            if task_cancel.is_cancelled() {
                                                "cancelled"
                                            } else {
                                                "error"
                                            },
                                            error
                                        ),
                                    );
                                    if !task_cancel.is_cancelled() {
                                        log::error!("decoder task failed for {}: {}", id, error);
                                        task_status.set_failed(error.to_string());
                                        let _ =
                                            task_app_handle.emit("play_failed", error.to_string());
                                    }
                                    return;
                                }
                            };

                            if task_cancel.is_cancelled() {
                                let _ = tokio::fs::remove_file(&path).await;
                                task_trace.event("sink", "status=cancelled_before_append");
                                return;
                            }

                            {
                                let paused_before = task_telemetry.is_paused();
                                task_telemetry.append(Box::new(decoder));
                                task_suppress_end.store(false, Ordering::Release);
                                if paused_before {
                                    task_telemetry.play();
                                }
                                task_trace.event(
                                    "sink",
                                    format_args!(
                                        "status=appended paused_before={} empty_after={}",
                                        paused_before,
                                        task_telemetry.empty()
                                    ),
                                );
                            }
                            // 下载期间排队的 seek：append 后立即应用，再通知 play_start。
                            if let Some(pos) = task_pending_seek.lock().await.take() {
                                match task_telemetry.try_seek(pos) {
                                    Ok(()) => task_trace.event(
                                        "control",
                                        format_args!(
                                            "action=seek status=applied_pending requested_sec={:.3}",
                                            pos.as_secs_f32()
                                        ),
                                    ),
                                    Err(error) => task_trace.event(
                                        "control",
                                        format_args!("action=seek status=apply_failed error={}", error),
                                    ),
                                }
                            }
                            task_status.set_playing();
                            task_trace.event("play_start", "status=emitted");
                            log::info!("playback started: {}", id);
                            let _ = task_app_handle.emit("play_start", ());

                            task_trace.event("persist", "status=start");
                            if let Err(error) = task_playback.persist_played(&id, &path).await {
                                task_trace
                                    .event("persist", format_args!("status=error error={}", error));
                                log::error!(
                                    "persist played track failed after playback started for {}: {}",
                                    id,
                                    error
                                );
                            } else {
                                task_trace.event("persist", "status=ok");
                            }
                            if task_cancel.is_cancelled() {
                                task_telemetry.clear();
                                task_trace.event("request", "status=cancelled_after_append");
                                if let Err(error) = task_playback.discard_played(&id, &path).await {
                                    log::error!(
                                        "discard cancelled playback failed for {}: {}",
                                        id,
                                        error
                                    );
                                }
                                return;
                            }
                            let _ = task_app_handle.emit("db_tracks_changed", "recent");
                        });
                        active_task = Some((cancel, task));
                    }
                    MusicState::Seek(time) => {
                        let pos = Duration::from_secs_f32(time);
                        // 锁顺序统一：pending_seek 先于 sink。
                        let mut pending = pending_seek.lock().await;
                        if telemetry.empty() {
                            // 下载/加载中：排队，加载完成后应用（后到的 seek 覆盖先到的）。
                            *pending = Some(pos);
                            log::info!(
                                target: "playback_trace",
                                "stage=control action=seek status=queued reason=empty_sink requested_sec={:.3}",
                                time
                            );
                            continue;
                        }
                        *pending = None;
                        if let Err(error) = telemetry.try_seek(pos) {
                            log::error!("seek error: {}", error);
                        } else {
                            log::info!(
                                target: "playback_trace",
                                "stage=control action=seek status=ok requested_sec={:.3}",
                                time
                            );
                        }
                    }
                    MusicState::Recovery => {
                        status.set_playing();
                        telemetry.play();
                        log::info!(
                            target: "playback_trace",
                            "stage=control action=recovery paused_after={}",
                            telemetry.is_paused()
                        );
                    }
                    MusicState::Pause => {
                        status.set_paused();
                        telemetry.pause();
                        log::info!(
                            target: "playback_trace",
                            "stage=control action=pause paused_after={}",
                            telemetry.is_paused()
                        );
                    }
                    MusicState::Volume(volume) => {
                        let effective_volume = volume / 50.0;
                        telemetry.set_volume(effective_volume);
                        log::info!(
                            target: "playback_trace",
                            "stage=control action=volume requested={} effective={:.3}",
                            volume,
                            effective_volume
                        );
                    }
                    MusicState::Quit => {
                        if let Some((cancel, _)) = active_task.take() {
                            cancel.cancel();
                        }
                        status.set_idle();
                        telemetry.stop();
                        log::info!(target: "playback_trace", "stage=control action=quit");
                    }
                }
            }
        });
    }

    fn spawn_progress(
        app_handle: AppHandle,
        telemetry: Arc<InstrumentedSink>,
        suppress_end: Arc<AtomicBool>,
        status: Arc<PlaybackStatus>,
    ) {
        tokio::spawn(async move {
            let mut curr_state = false;
            loop {
                // 业务逻辑只感知播放接口；tick/指标日志/汇总都在装饰器内完成。
                if telemetry.empty() {
                    if curr_state {
                        if suppress_end.swap(false, Ordering::AcqRel) {
                            curr_state = false;
                        } else {
                            if let Some(report) = telemetry.take_report() {
                                if report.stall_count > 0 {
                                    let _ = app_handle
                                        .emit("play_probe_report", report.to_json_string());
                                }
                            }
                            status.set_idle();
                            let _ = app_handle.emit("play_end", ());
                            curr_state = false;
                        }
                    }
                } else {
                    let position = telemetry.get_pos();
                    let _ = app_handle.emit("play_progress", position);
                    curr_state = true;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }
}
