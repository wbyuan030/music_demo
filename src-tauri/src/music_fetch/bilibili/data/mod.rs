pub mod search;
use derive_new::new;
#[derive(Default, new)]
pub struct MediaItem {
    pub cid: Option<String>,
    pub bvid: Option<String>,
    pub aid: Option<String>,
}

pub enum MediaQuality {
    Low,
    Standard,
    High,
    Super,
}

impl MediaQuality {
    pub fn to_string(&self) -> String {
        match self {
            MediaQuality::Low => "low".to_string(),
            MediaQuality::Standard => "standard".to_string(),
            MediaQuality::High => "high".to_string(),
            MediaQuality::Super => "super".to_string(),
        }
    }
}
