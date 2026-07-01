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
}

impl PlaybackProbe {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_pos: Duration::ZERO,
            last_wall_clock: now,
            total_elapsed: 0.0,
            stall_threshold: Duration::from_millis(100),
            severe_threshold: Duration::from_millis(500),
            stall_events: Vec::new(),
            max_gap: Duration::ZERO,
        }
    }

    /// 每次进度回调时调用
    /// `current_pos`: sink.get_pos() 返回的当前播放位置
    pub fn tick(&mut self, current_pos: Duration) {
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
                tracing::warn!(
                    "severe audio underrun: gap={:?} at {:.1}s",
                    gap,
                    self.total_elapsed
                );
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
            let total: Duration = self
                .stall_events
                .iter()
                .map(|e| e.gap)
                .sum();
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
            probe.tick(pos);
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
            probe.tick(pos);
        }
        // 第 4 次：时间过了 500ms 但位置没变（卡在 1.5s）
        probe.last_wall_clock = std::time::Instant::now() - Duration::from_secs_f64(0.5);
        probe.tick(Duration::from_secs_f64(1.5));
        let report = probe.report();
        assert!(report.stall_count > 0, "expected at least one stall, got 0");
    }
}
