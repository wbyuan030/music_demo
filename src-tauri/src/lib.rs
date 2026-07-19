use tauri::Manager;
mod global;
use crate::global::get_track_state;
use crate::global::init_db;
use crate::global::init_track_state;
use crate::music_handler::handle_event;
use crate::music_handler::MusicHandler;

use crate::extractor::bilibili::search_bilibili_video;
use crate::extractor::bilibili::get_bilibili_manifest;
use crate::extractor::youtube::commands::{get_youtube_manifest, search_youtube_music};
use crate::music_fetch::bilibili::search_music;
use crate::music_fetch::wx::parse_track_from_wx;
use crate::public::{list_liked_tracks, list_recent_tracks, toggle_liked_track};

pub mod extractor;
mod audio_quality;
mod music_fetch;
mod music_handler;
mod public;
mod storage;
mod types;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_db().unwrap();
    init_track_state().unwrap();
    let track_state = get_track_state().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .manage(track_state)
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let handler = MusicHandler::new(app.app_handle().clone());
            app.manage(handler.event_sender.clone());
            app.manage(handler);
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            search_bilibili_video,
            get_bilibili_manifest,
            search_youtube_music,
            get_youtube_manifest,
            handle_event,
            search_music,
            parse_track_from_wx,
            list_recent_tracks,
            list_liked_tracks,
            toggle_liked_track,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
