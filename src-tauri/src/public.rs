use std::{collections::HashSet, sync::Arc};

use tauri::State;

use crate::{
    global::get_db,
    music_handler::{status::PlaybackStateView, MusicHandler},
    playback::{BackendRuntime, SourceKind},
    storage::{
        get_track_by_id, list_liked_track, list_recent_track, toggle_liked_by_id,
        upsert_track_entry,
    },
    types::TrackView,
};

/// 播放状态快照查询：前端挂载/重连对账用。
/// 后端是唯一状态源，事件流可能因监听器注册时机丢失事件，快照兜底。
#[tauri::command]
pub fn get_playback_state(handler: State<'_, MusicHandler>) -> Result<PlaybackStateView, String> {
    let position = handler.telemetry.get_pos();
    Ok(handler.status.snapshot(position.as_secs_f32()))
}

/// 统一多来源搜索。
///
/// `source` 可选过滤：来源名（如 `"youtube"`），缺省或 `"all"` 搜索全部已注册来源；
/// 未知来源名报错。单来源失败只记日志，不阻塞其它来源；全部失败才返回错误。
#[tauri::command]
pub async fn search_music(
    keyword: String,
    source: Option<String>,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<Vec<TrackView>, String> {
    if keyword.trim().is_empty() {
        return Ok(Vec::new());
    }

    // 分发完全由注册表驱动：新增来源只注册，不修改这里。
    let requested = source.as_deref().unwrap_or("all");
    let sources: Vec<SourceKind> = runtime
        .search
        .sources()
        .iter()
        .copied()
        .filter(|kind| requested == "all" || kind.as_str() == requested)
        .collect();
    if sources.is_empty() {
        return Err(format!("unknown search source: {}", requested));
    }

    let mut views = Vec::new();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();

    for kind in &sources {
        match runtime
            .search
            .search(*kind, &keyword, &runtime.context)
            .await
        {
            Ok(entries) => {
                for entry in entries {
                    if seen.insert(entry.view.id.clone()) {
                        views.push(entry.view.clone());
                        runtime.catalog.insert(entry.view.id.clone(), entry).await;
                    }
                }
            }
            Err(error) => {
                log::warn!(
                    target: "playback_trace",
                    "stage=search source={} status=error error={}",
                    kind.as_str(),
                    error
                );
                errors.push(error.to_string());
            }
        }
    }

    if views.is_empty() && !errors.is_empty() {
        return Err(errors.join("; "));
    }
    Ok(views)
}

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
    log::info!("list_liked_tracks command received");
    let db = get_db();
    let track_db_list = match list_liked_track(db) {
        Ok(tracks) => tracks,
        Err(error) => {
            log::error!("list_liked_tracks failed: {}", error);
            return Err(error.to_string());
        }
    };
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
    log::info!(
        "list_liked_tracks command completed count={}",
        track_view_list.len()
    );
    Ok(track_view_list)
}

#[tauri::command]
pub fn report_frontend_log(
    level: String,
    source: String,
    message: String,
    stack: Option<String>,
    command: Option<String>,
) {
    let stack = stack.as_deref().unwrap_or("-");
    let command = command.as_deref().unwrap_or("-");
    match level.as_str() {
        "error" => log::error!(
            target: "frontend",
            "source={} command={} message={} stack={}",
            source,
            command,
            message,
            stack
        ),
        "warn" => log::warn!(
            target: "frontend",
            "source={} command={} message={} stack={}",
            source,
            command,
            message,
            stack
        ),
        "debug" => log::debug!(
            target: "frontend",
            "source={} command={} message={} stack={}",
            source,
            command,
            message,
            stack
        ),
        _ => log::info!(
            target: "frontend",
            "source={} command={} message={} stack={}",
            source,
            command,
            message,
            stack
        ),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn liked_tracks_command_name_is_stable() {
        assert_eq!(
            __tauri_command_name_list_liked_tracks!(),
            "list_liked_tracks"
        );
    }

    #[test]
    fn frontend_log_command_name_is_stable() {
        assert_eq!(
            __tauri_command_name_report_frontend_log!(),
            "report_frontend_log"
        );
    }

    #[test]
    fn search_music_command_name_is_stable() {
        assert_eq!(__tauri_command_name_search_music!(), "search_music");
    }
}

#[tauri::command]
pub async fn toggle_liked_track(
    id: String,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<(), String> {
    let db = get_db();
    if get_track_by_id(db, id.clone())
        .map_err(|error| error.to_string())?
        .is_none()
    {
        let entry = runtime
            .catalog
            .get(&id)
            .await
            .ok_or_else(|| "track not found".to_string())?;
        upsert_track_entry(db, &entry, None).map_err(|error| error.to_string())?;
    }
    toggle_liked_by_id(db, id).map_err(|error| error.to_string())
}
