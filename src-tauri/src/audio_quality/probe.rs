use std::time::{Duration, Instant};

/// 欠载事件
#[derive(Debug, Clone)]
pub struct StallEvent {
    /// 从播放开始到欠载的时间（秒）
    pub elapsed_sec: f64,
    /// 期望位置与实际位置的差值
    pub gap: Duration,
}

/// 探针报告
#[derive(Debug, Clone)]
pub struct ProbeReport {
    pub stall_count: usize,
    pub severe_count: usize,
    pub max_gap: Duration,
    pub avg_gap: Duration,
    pub stall_timeline: Vec<(f64, Duration)>,
}

impl ProbeReport {
    /// 格式化为 JSON 字符串（手动构建，避免 serde 依赖）
    pub fn to_json_string(&self) -> String {
        let timeline: Vec<String> = self
            .stall_timeline
            .iter()
            .map(|(sec, gap)| format!("[{:.3},{:.3}]", sec, gap.as_secs_f64()))
            .collect();
        format!(
            r#"{{"stall_count":{},"severe_count":{},"max_gap":{:.3},"avg_gap":{:.3},"stall_timeline":[{}]}}"#,
            self.stall_count,
            self.severe_count,
            self.max_gap.as_secs_f64(),
            self.avg_gap.as_secs_f64(),
            timeline.join(",")
        )
    }
}

/// 在线播放探针
///
/// 集成到 spawn_progress 中，每 500ms 调用 tick(current_pos)，
/// 计算期望位置与实际位置的 gap，检测欠载事件。
pub struct PlaybackProbe {
    last_pos: Duration,
    last_wall_clock: Instant,
    total_elapsed: f64,
    stall_threshold: Duration,
    severe_threshold: Duration,
    pub stall_events: Vec<StallEvent>,
    pub max_gap: Duration,
    trace_id: Option<String>,
    track_id: Option<String>,
    last_metric_at: Instant,
}

impl PlaybackProbe {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::without_trace()
    }
    pub fn with_trace(track_id: impl Into<String>, trace_id: impl Into<String>) -> Self {
        let mut probe = Self::without_trace();
        probe.track_id = Some(track_id.into());
        probe.trace_id = Some(trace_id.into());
        probe
    }

    fn without_trace() -> Self {
        let now = Instant::now();
        Self {
            last_pos: Duration::ZERO,
            last_wall_clock: now,
            total_elapsed: 0.0,
            stall_threshold: Duration::from_millis(100),
            severe_threshold: Duration::from_millis(500),
            stall_events: Vec::new(),
            max_gap: Duration::ZERO,
            trace_id: None,
            track_id: None,
            last_metric_at: now,
        }
    }

    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    pub fn track_id(&self) -> Option<&str> {
        self.track_id.as_deref()
    }

    pub fn should_emit_metric(&mut self) -> bool {
        if self.last_metric_at.elapsed() >= Duration::from_secs(5) {
            self.last_metric_at = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn elapsed_sec(&self) -> f64 {
        self.total_elapsed
    }

    /// 每次进度回调时调用
    /// `current_pos`: sink.get_pos() 返回的当前播放位置
    /// `paused`: 暂停时位置冻结是正常行为，不构成欠载；只重置基线，避免恢复后误报。
    pub fn tick(&mut self, current_pos: Duration, paused: bool) {
        if paused {
            self.last_pos = current_pos;
            self.last_wall_clock = Instant::now();
            return;
        }
        let elapsed = self.last_wall_clock.elapsed();
        self.total_elapsed += elapsed.as_secs_f64();

        let expected = self.last_pos + elapsed;
        let actual = current_pos;

        // gap = expected - actual: 如果实际位置落后于期望位置，说明发生了欠载
        let gap = if expected > actual {
            expected - actual
        } else {
            Duration::ZERO
        };

        if gap > self.stall_threshold {
            self.stall_events.push(StallEvent {
                elapsed_sec: self.total_elapsed,
                gap,
            });
            if gap > self.max_gap {
                self.max_gap = gap;
            }
            if gap > self.severe_threshold {
                match (self.trace_id(), self.track_id()) {
                    (Some(trace_id), Some(track_id)) => log::warn!(
                        target: "playback_trace",
                        "trace_id={} track_id={} stage=underrun severity=severe gap_ms={} elapsed_ms={}",
                        trace_id,
                        track_id,
                        gap.as_millis(),
                        (self.total_elapsed * 1_000.0) as u128
                    ),
                    _ => log::warn!(
                        target: "playback_trace",
                        "stage=underrun severity=severe gap_ms={} elapsed_ms={}",
                        gap.as_millis(),
                        (self.total_elapsed * 1_000.0) as u128
                    ),
                }
            }
        }

        self.last_pos = actual;
        self.last_wall_clock = Instant::now();
    }

    /// 生成报告
    pub fn report(&self) -> ProbeReport {
        let severe_count = self
            .stall_events
            .iter()
            .filter(|e| e.gap > self.severe_threshold)
            .count();
        let avg_gap = if self.stall_events.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.stall_events.iter().map(|e| e.gap).sum();
            total / self.stall_events.len() as u32
        };
        let timeline = self
            .stall_events
            .iter()
            .map(|e| (e.elapsed_sec, e.gap))
            .collect();

        ProbeReport {
            stall_count: self.stall_events.len(),
            severe_count,
            max_gap: self.max_gap,
            avg_gap,
            stall_timeline: timeline,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn probe_no_stall() {
        let mut probe = PlaybackProbe::new();
        for i in 1..=10 {
            let pos = Duration::from_secs_f64(i as f64 * 0.5);
            probe.last_wall_clock = std::time::Instant::now() - Duration::from_secs_f64(0.5);
            probe.tick(pos, false);
        }
        let report = probe.report();
        assert_eq!(report.stall_count, 0, "expected no stalls");
    }

    #[test]
    fn probe_detects_stall() {
        let mut probe = PlaybackProbe::new();
        // 正常播放 3 次
        for i in 1..=3 {
            let pos = Duration::from_secs_f64(i as f64 * 0.5);
            probe.last_wall_clock = std::time::Instant::now() - Duration::from_secs_f64(0.5);
            probe.tick(pos, false);
        }
        // 第 4 次：时间过了 500ms 但位置没变（卡在 1.5s）
        probe.last_wall_clock = std::time::Instant::now() - Duration::from_secs_f64(0.5);
        probe.tick(Duration::from_secs_f64(1.5), false);
        let report = probe.report();
        assert!(report.stall_count > 0, "expected at least one stall, got 0");
    }

    #[test]
    fn paused_ticks_do_not_count_as_underrun() {
        let mut probe = PlaybackProbe::new();
        // 暂停 10 秒：位置冻结，但不应产生任何欠载
        for _ in 1..=20 {
            probe.last_wall_clock = std::time::Instant::now() - Duration::from_millis(500);
            probe.tick(Duration::from_secs_f64(1.5), true);
        }
        let report = probe.report();
        assert_eq!(report.stall_count, 0, "pause must not count as underrun");
        // 暂停期间 elapsed 不累积；恢复后基线已重置，紧接的 tick 不误报
        assert_eq!(probe.elapsed_sec(), 0.0);
        probe.last_wall_clock = std::time::Instant::now();
        probe.tick(Duration::from_secs_f64(1.5), false);
        let report = probe.report();
        assert_eq!(report.stall_count, 0, "resume right after pause: no stall");
    }
}
