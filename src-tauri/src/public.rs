use std::sync::{Arc, Mutex};

use native_db::Database;
use tauri::State;

use crate::{storage::list_recent_track, types::TrackView};

#[tauri::command]
pub fn get_recent_tracks(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<TrackView>, String> {
    let db_clone = Arc::clone(&db);
    let db_use = db_clone.lock().unwrap();
    let track_db_list = list_recent_track(&db_use).map_err(|e| e.to_string())?;
    let track_view_list: Vec<TrackView> = track_db_list
        .iter()
        .map(|d| TrackView {
            id: d.id.clone(),
            title: d.title.clone(),
            artist: d.artist.clone(),
            cover_url: d.cover_url.clone(),
            duration: d.duration.clone(),
        })
        .collect();
    Ok(track_view_list)
}
