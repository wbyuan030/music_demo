use native_db::Builder;

use crate::music::handle_event;
use crate::music::Music;

use crate::music::TrackStore;
use crate::music_fetch::bilibili::search_music;
use crate::music_fetch::wx::parse_track_from_wx;
use crate::public::get_recent_tracks;
use crate::storage::TRACK_MODEL;
use crate::types::Track;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
mod music;
mod music_fetch;
mod public;
mod storage;
mod types;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let track_state = TrackStore {
        tracks: Arc::new(Mutex::new(HashMap::<String, Track>::new())),
    };
    let tracks_for_music = Arc::clone(&track_state.tracks);
    let mut db = Arc::new(Mutex::new(
        Builder::new().create(&TRACK_MODEL, "./local.db").unwrap(),
    ));
    let db_for_music = Arc::clone(&db);

    let music = Music::new(tracks_for_music, db_for_music);
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .manage(track_state)
        .manage(db)
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            handle_event,
            parse_track_from_wx,
            search_music,
            get_recent_tracks
        ])
        // share sender and sink with the frontend
        .manage(music.event_sender)
        .manage(music.sink)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
