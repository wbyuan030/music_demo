use std::time::Duration;

use parking_lot::Mutex;
use rodio::{Sink, Source};

use super::probe::{PlaybackProbe, ProbeReport};

/// Sink 的可观测性装饰器。
///
/// 业务代码只感知播放接口（append/seek/pause/...）与两个生命周期钩子
/// （`begin_track` / `take_report`）；探针的 tick、暂停感知、5s 指标日志
/// 和播放结束汇总全部收在本装饰器内，与业务逻辑解耦。
pub struct InstrumentedSink {
    inner: Mutex<Sink>,
    probe: Mutex<Option<PlaybackProbe>>,
}

impl InstrumentedSink {
    pub fn new(inner: Sink) -> Self {
        Self {
            inner: Mutex::new(inner),
            probe: Mutex::new(None),
        }
    }

    /// 开始一次播放的观测（在 play 请求时调用）。
    pub fn begin_track(&self, track_id: &str, trace_id: &str) {
        *self.probe.lock() = Some(PlaybackProbe::with_trace(track_id, trace_id));
    }

    /// 读取播放位置；内部完成暂停感知的 tick 与每 5s 一次的进度指标日志。
    pub fn get_pos(&self) -> Duration {
        let inner = self.inner.lock();
        let position = inner.get_pos();
        let paused = inner.is_paused();
        drop(inner);

        if let Some(probe) = self.probe.lock().as_mut() {
            probe.tick(position, paused);
            if probe.should_emit_metric() {
                let report = probe.report();
                if let (Some(trace_id), Some(track_id)) = (probe.trace_id(), probe.track_id()) {
                    log::info!(
                        target: "playback_trace",
                        "trace_id={} track_id={} stage=progress elapsed_ms={} position_ms={} stall_count={} severe_count={} max_gap_ms={} avg_gap_ms={}",
                        trace_id,
                        track_id,
                        (probe.elapsed_sec() * 1_000.0) as u128,
                        position.as_millis(),
                        report.stall_count,
                        report.severe_count,
                        report.max_gap.as_millis(),
                        report.avg_gap.as_millis()
                    );
                }
            }
        }
        position
    }

    /// 自然结束时取汇总并清理探针；没有进行中的观测时返回 None。
    /// `stage=play_end` 汇总日志在此输出。
    pub fn take_report(&self) -> Option<ProbeReport> {
        let probe = self.probe.lock().take()?;
        let report = probe.report();
        let trace_id = probe.trace_id().unwrap_or("-");
        let track_id = probe.track_id().unwrap_or("-");
        log::info!(
            target: "playback_trace",
            "trace_id={} track_id={} stage=play_end elapsed_ms={} status=natural_end stall_count={} severe_count={} max_gap_ms={} avg_gap_ms={}",
            trace_id,
            track_id,
            (probe.elapsed_sec() * 1_000.0) as u128,
            report.stall_count,
            report.severe_count,
            report.max_gap.as_millis(),
            report.avg_gap.as_millis()
        );
        Some(report)
    }

    // ── 转发的播放接口 ────────────────────────────────────────────────

    pub fn append(&self, source: Box<dyn Source + Send>) {
        self.inner.lock().append(source);
    }

    pub fn clear(&self) {
        self.inner.lock().clear();
    }

    pub fn empty(&self) -> bool {
        self.inner.lock().empty()
    }

    pub fn is_paused(&self) -> bool {
        self.inner.lock().is_paused()
    }

    pub fn play(&self) {
        self.inner.lock().play();
    }

    pub fn pause(&self) {
        self.inner.lock().pause();
    }

    pub fn stop(&self) {
        self.inner.lock().stop();
    }

    pub fn set_volume(&self, volume: f32) {
        self.inner.lock().set_volume(volume);
    }

    pub fn try_seek(&self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.inner.lock().try_seek(pos)
    }
}
