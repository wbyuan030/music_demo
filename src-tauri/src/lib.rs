use std::sync::Arc;

use tauri::Manager;

mod global;
use crate::extractor::context::ExtractorContext;
use crate::global::{get_track_state, init_db, init_track_state};
use crate::music_handler::handle_event;
use crate::music_handler::MusicHandler;
use crate::playback::BackendRuntime;

use crate::extractor::bilibili::get_bilibili_manifest;
use crate::extractor::youtube::commands::get_youtube_manifest;
use crate::music_fetch::wx::parse_track_from_wx;
use crate::music_fetch::url::parse_track_from_url;
use crate::public::{
    add_playlist_track, clear_cache, create_playlist, delete_playlist, get_cache_info,
    get_playback_state, list_liked_tracks, list_playlists, list_recent_tracks,
    remove_playlist_track, reorder_playlist_track, rename_playlist, report_frontend_log,
    search_music, toggle_liked_track,
};

mod audio_quality;
pub mod extractor;
mod music_fetch;
mod music_handler;
pub mod playback;
mod public;
mod storage;
mod types;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_db().expect("database initialization failed");
    init_track_state().expect("track catalog initialization failed");

    let catalog = get_track_state().expect("track catalog is not initialized");
    let context =
        Arc::new(ExtractorContext::new().expect("extractor context initialization failed"));
    let runtime = Arc::new(BackendRuntime::new(
        context,
        catalog,
        std::env::temp_dir().join("music_cache"),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .manage(runtime.clone())
        .plugin(tauri_plugin_http::init())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let handler = MusicHandler::new(app.app_handle().clone(), runtime.playback.clone());
            app.manage(handler.event_sender.clone());
            app.manage(handler);
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_bilibili_manifest,
            get_youtube_manifest,
            handle_event,
            search_music,
            parse_track_from_wx,
            parse_track_from_url,
            list_recent_tracks,
            list_liked_tracks,
            get_playback_state,
            report_frontend_log,
            toggle_liked_track,
            get_cache_info,
            list_playlists,
            create_playlist,
            rename_playlist,
            delete_playlist,
            add_playlist_track,
            remove_playlist_track,
            reorder_playlist_track,
            clear_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
