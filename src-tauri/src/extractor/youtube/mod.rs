pub mod api;
pub mod commands;
pub mod player;
pub mod search;
pub mod types;

/// Extract a YouTube video ID from various URL formats.
pub fn extract_video_id(url: &str) -> Option<String> {
    // youtu.be/VIDEO_ID
    if let Some(id) = url
        .strip_prefix("https://youtu.be/")
        .or_else(|| url.strip_prefix("http://youtu.be/"))
    {
        return Some(id.split(&['?', '#'][..]).next()?.to_string());
    }

    // youtube.com/watch?v=VIDEO_ID
    if url.contains("youtube.com/watch") || url.contains("music.youtube.com/watch") {
        let query = url.split('?').nth(1)?;
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next()? == "v" {
                return Some(parts.next()?.to_string());
            }
        }
    }

    // youtube.com/embed/VIDEO_ID
    if url.contains("youtube.com/embed/") {
        let id = url.split("/embed/").nth(1)?;
        return Some(id.split(&['?', '#', '/'][..]).next()?.to_string());
    }

    // youtube.com/shorts/VIDEO_ID
    if url.contains("youtube.com/shorts/") {
        let id = url.split("/shorts/").nth(1)?;
        return Some(id.split(&['?', '#', '/'][..]).next()?.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_full_url() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_short() {
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_music() {
        assert_eq!(
            extract_video_id(
                "https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=RDAMVMdQw4w9WgXcQ"
            ),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_with_params() {
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ?t=30"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_none() {
        assert_eq!(extract_video_id("https://example.com"), None);
    }
}
