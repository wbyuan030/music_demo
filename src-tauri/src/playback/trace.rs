use std::{fmt::Display, time::Instant};

/// Correlates one frontend play request across cache, resolver, download, and decoder stages.
#[derive(Clone)]
pub(crate) struct PlaybackTrace {
    id: String,
    track_id: String,
    started_at: Instant,
}

impl PlaybackTrace {
    pub(crate) fn new(track_id: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().simple().to_string(),
            track_id: track_id.into(),
            started_at: Instant::now(),
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    fn format_event(&self, stage: &str, elapsed_ms: u128, details: impl Display) -> String {
        format!(
            "trace_id={} track_id={} stage={} elapsed_ms={} {}",
            self.id, self.track_id, stage, elapsed_ms, details
        )
    }

    pub(crate) fn event(&self, stage: &str, details: impl Display) {
        log::info!(
            target: "playback_trace",
            "{}",
            self.format_event(stage, self.started_at.elapsed().as_millis(), details)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::PlaybackTrace;

    #[test]
    fn trace_ids_are_unique() {
        let first = PlaybackTrace::new("yt:first");
        let second = PlaybackTrace::new("yt:first");
        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn trace_line_contains_correlation_and_stage_fields() {
        let trace = PlaybackTrace::new("yt:first");
        let line = trace.format_event("decode", 42, "status=ok");
        assert!(line.contains("trace_id="));
        assert!(line.contains("track_id=yt:first"));
        assert!(line.contains("stage=decode"));
        assert!(line.contains("elapsed_ms=42"));
        assert!(line.ends_with("status=ok"));
    }
}
