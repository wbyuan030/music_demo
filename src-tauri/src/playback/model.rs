use std::{fmt, str::FromStr};

use crate::types::TrackView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Youtube,
    Bilibili,
    Wechat,
    // ==== sync-generated:begin source_kinds ====
    Audius,
    // ==== sync-generated:end source_kinds ====
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Youtube => "youtube",
            Self::Bilibili => "bilibili",
            Self::Wechat => "wechat",
            // ==== sync-generated:begin source_kind_as_str ====
            Self::Audius => "audius",
            // ==== sync-generated:end source_kind_as_str ====
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackId {
    pub source: SourceKind,
    pub remote_id: String,
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source {
            SourceKind::Youtube => write!(f, "yt:{}", self.remote_id),
            SourceKind::Bilibili => write!(f, "bili:{}", self.remote_id),
            SourceKind::Wechat => write!(f, "{}", self.remote_id),
            // ==== sync-generated:begin track_id_display ====
            SourceKind::Audius => write!(f, "au:{}", self.remote_id),
            // ==== sync-generated:end track_id_display ====
        }
    }
}

impl FromStr for TrackId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(remote_id) = value.strip_prefix("yt:") {
            if !remote_id.is_empty() {
                return Ok(Self {
                    source: SourceKind::Youtube,
                    remote_id: remote_id.to_string(),
                });
            }
        }
        if let Some(remote_id) = value.strip_prefix("bili:") {
            if !remote_id.is_empty() {
                return Ok(Self {
                    source: SourceKind::Bilibili,
                    remote_id: remote_id.to_string(),
                });
            }
        }
        // ==== sync-generated:begin track_id_fromstr ====
        if let Some(remote_id) = value.strip_prefix("au:") {
            if !remote_id.is_empty() {
                return Ok(Self {
                    source: SourceKind::Audius,
                    remote_id: remote_id.to_string(),
                });
            }
        }
        // ==== sync-generated:end track_id_fromstr ====
        Err(())
    }
}

#[derive(Debug, Clone)]
pub enum SourceRef {
    Youtube { video_id: String },
    Bilibili { bvid: String },
    Wechat { url: String },
    // ==== sync-generated:begin source_ref ====
    Audius { id: String },
    // ==== sync-generated:end source_ref ====
}

impl SourceRef {
    pub fn kind(&self) -> SourceKind {
        match self {
            Self::Youtube { .. } => SourceKind::Youtube,
            Self::Bilibili { .. } => SourceKind::Bilibili,
            Self::Wechat { .. } => SourceKind::Wechat,
            // ==== sync-generated:begin source_ref_kind ====
            Self::Audius { .. } => SourceKind::Audius,
            // ==== sync-generated:end source_ref_kind ====
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayableEntry {
    pub view: TrackView,
    pub source_ref: SourceRef,
}

impl PlayableEntry {
    pub fn id(&self) -> &str {
        &self.view.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_id_round_trips_stable_extractor_ids() {
        let youtube = TrackId {
            source: SourceKind::Youtube,
            remote_id: "video".to_string(),
        };
        assert_eq!(youtube.to_string(), "yt:video");
        assert_eq!("yt:video".parse::<TrackId>(), Ok(youtube));

        let bilibili = TrackId {
            source: SourceKind::Bilibili,
            remote_id: "BV1".to_string(),
        };
        assert_eq!(bilibili.to_string(), "bili:BV1");
        assert_eq!("bili:BV1".parse::<TrackId>(), Ok(bilibili));
        assert!("legacy-uuid".parse::<TrackId>().is_err());
    }
}
