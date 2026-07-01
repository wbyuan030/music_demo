use crate::{
    global::get_db,
    storage::{get_track_by_id, list_liked_track, list_recent_track, toggle_liked_by_id},
    types::TrackView,
};

#[tauri::command]
pub fn list_recent_tracks() -> Result<Vec<TrackView>, String> {
    let db = get_db();
    let track_db_list = list_recent_track(db).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn list_liked_tracks() -> Result<Vec<TrackView>, String> {
    let db = get_db();
    let track_db_list = list_liked_track(&db).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn get_track(id: String) -> Result<TrackView, String> {
    let db = get_db();
    let track_db = get_track_by_id(db, id).map_err(|e| e.to_string())?;
    match track_db {
        Some(d) => Ok(TrackView {
            id: d.id.clone(),
            title: d.title.clone(),
            artist: d.artist.clone(),
            cover_url: d.cover_url.clone(),
            duration: d.duration.clone(),
        }),
        None => return Err("track not found".to_string()),
    }
}

#[tauri::command]
pub fn toggle_liked_track(id: String) -> Result<(), String> {
    let db = get_db();
    toggle_liked_by_id(db, id).map_err(|e| e.to_string())?;
    Ok(())
}
