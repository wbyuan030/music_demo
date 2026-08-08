use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{Image, Track};
use crate::extractor::protocol::ExtractError;

use super::api::InnertubeClient;
use super::types::*;

/// Search YouTube Music for tracks matching `query`.
/// Returns application-level Track models.
pub async fn search_music(
    ctx: &ExtractorContext,
    query: &str,
    section: Option<&str>,
) -> Result<Vec<Track>, ExtractError> {
    let params = section.and_then(|s| {
        // Known section params for YouTube Music search.
        // Songs: EgWKAQIIAWoKEAoQAxAEEAkQBQ==
        // Albums: EgWKAQIYAWoKEAoQAxAEEAkQBQ==
        // Videos: EgWKAQIQAWoKEAoQAxAEEAkQBQ==
        // Artists: EgWKAQIgAWoKEAoQAxAEEAkQBQ==
        match s.to_lowercase().as_str() {
            "songs" => Some("EgWKAQIIAWoKEAoQAxAEEAkQBQ=="),
            "albums" => Some("EgWKAQIYAWoKEAoQAxAEEAkQBQ=="),
            "videos" => Some("EgWKAQIQAWoKEAoQAxAEEAkQBQ=="),
            "artists" => Some("EgWKAQIgAWoKEAoQAxAEEAkQBQ=="),
            _ => None,
        }
    });

    let resp = InnertubeClient::search(ctx, query, params).await?;

    let items = parse_search_response(&resp);
    Ok(items)
}

/// Parse search results from an InnerTube response into Track models.
fn parse_search_response(resp: &InnerTubeResponse) -> Vec<Track> {
    let mut tracks: Vec<Track> = Vec::new();

    // Try tabbed search results (YouTube Music).
    if let Some(contents) = &resp.contents {
        if let Some(tabbed) = &contents.tabbed_search {
            for tab in &tabbed.tabs {
                if let Some(renderer) = &tab.tab_renderer {
                    if let Some(content) = &renderer.content {
                        if let Some(section_list) = &content.section_list {
                            for section in &section_list.contents {
                                if let Some(shelf) = &section.music_shelf {
                                    for item in &shelf.contents {
                                        if let Some(t) = parse_music_list_item(&item.renderer) {
                                            tracks.push(t);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Try standard search results (YouTube main).
        if let Some(two_col) = &contents.two_column_search {
            if let Some(primary) = &two_col.primary_contents {
                if let Some(section_list) = &primary.section_list {
                    for section in &section_list.contents {
                        if let Some(item_section) = &section.item_section {
                            for item in &item_section.contents {
                                if let Some(t) = parse_music_list_item(&item.music_renderer) {
                                    tracks.push(t);
                                }
                                if let Some(renderer) = &item.video_renderer {
                                    if let Some(t) = parse_video_renderer(renderer) {
                                        tracks.push(t);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tracks
}

/// Parse a YouTube Music list item into a Track.
fn parse_music_list_item(item: &Option<MusicResponsiveListItemRenderer>) -> Option<Track> {
    let renderer = item.as_ref()?;

    // Video ID (primary key for tracks).
    let video_id = renderer
        .playlist_item_data
        .as_ref()
        .and_then(|d| d.video_id.clone())
        .or_else(|| {
            renderer
                .navigation_endpoint
                .as_ref()
                .and_then(|e| e.watch_endpoint.as_ref())
                .and_then(|w| w.video_id.clone())
        })?;

    let id = format!("yt:{}", video_id);

    // Title from first flex column.
    let title = renderer
        .flex_columns
        .first()
        .and_then(|col| col.renderer.as_ref())
        .map(|r| r.text.flatten())
        .unwrap_or_default();

    // Subtitle runs from the second flex column.
    let subtitle_runs: Vec<String> = renderer
        .flex_columns
        .get(1)
        .and_then(|col| col.renderer.as_ref())
        .and_then(|r| r.text.runs.as_ref())
        .map(|runs| {
            runs.iter()
                .map(|run| run.text.trim().to_string())
                .filter(|t| !t.is_empty() && t != "•")
                .collect()
        })
        .unwrap_or_default();

    // Parse duration from subtitle: last entry that looks like "M:SS" or "H:MM:SS".
    let duration_ms = subtitle_runs
        .iter()
        .rev()
        .find_map(|t| super::api::parse_duration_to_ms(t));

    // Artist: first non-duration, non-numeric entry in subtitle.
    let artists: Vec<String> = subtitle_runs
        .iter()
        .take_while(|t| super::api::parse_duration_to_ms(t).is_none())
        .filter(|t| !t.parse::<f64>().is_ok())
        .take(1)
        .map(|t| t.clone())
        .collect();

    // Album: entries between artist and duration, filter out play counts.
    let album = subtitle_runs
        .iter()
        .skip(if artists.is_empty() { 0 } else { 1 })
        .take_while(|t| super::api::parse_duration_to_ms(t).is_none())
        .filter(|t| !t.parse::<f64>().is_ok())
        .filter(|t| !t.ends_with("plays"))
        .next()
        .cloned();

    // Thumbnails.
    let artwork: Vec<Image> = renderer
        .thumbnail
        .as_ref()
        .and_then(|t| t.renderer.as_ref())
        .map(|r| {
            r.thumbnail
                .thumbnails
                .iter()
                .map(|t| Image {
                    url: t.url.clone(),
                    width: t.width.map(|w| w as u32),
                    height: t.height.map(|h| h as u32),
                })
                .collect()
        })
        .unwrap_or_default();

    Some(Track {
        id,
        title,
        artists,
        album,
        duration_ms,
        artwork,
    })
}

/// Parse a standard YouTube video renderer into a Track.
fn parse_video_renderer(renderer: &VideoRenderer) -> Option<Track> {
    let video_id = renderer.video_id.as_ref()?;
    let id = format!("yt:{}", video_id);

    let title = renderer
        .title
        .as_ref()
        .map(|t| t.flatten())
        .unwrap_or_default();

    let artist = renderer
        .short_byline
        .as_ref()
        .map(|t| t.flatten())
        .unwrap_or_default();

    let artists = if artist.is_empty() {
        vec![]
    } else {
        vec![artist]
    };

    let duration_ms = renderer
        .length_seconds
        .as_ref()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|s| s * 1000)
        .or_else(|| {
            renderer
                .length_text
                .as_ref()
                .and_then(|t| super::api::parse_duration_to_ms(&t.flatten()))
        });

    let artwork: Vec<Image> = renderer
        .thumbnail
        .as_ref()
        .map(|t| {
            t.thumbnails
                .iter()
                .map(|t| Image {
                    url: t.url.clone(),
                    width: t.width.map(|w| w as u32),
                    height: t.height.map(|h| h as u32),
                })
                .collect()
        })
        .unwrap_or_default();

    Some(Track {
        id,
        title,
        artists,
        album: None,
        duration_ms,
        artwork,
    })
}
