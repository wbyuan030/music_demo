use std::sync::Arc;

use tauri::State;

use crate::{extractor::model::PlaybackManifest, extractor::youtube, playback::BackendRuntime};

/// Get a playable audio manifest for a YouTube video ID or URL.
#[tauri::command]
pub async fn get_youtube_manifest(
    video_id: String,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<PlaybackManifest, String> {
    let vid = youtube::extract_video_id(&video_id).unwrap_or(video_id);
    youtube::player::get_manifest(&runtime.context, &vid)
        .await
        .map_err(|e| e.to_string())
}
