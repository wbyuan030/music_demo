use tokio::sync::broadcast::Sender;

use crate::music_handler::handler::MusicState;

#[tauri::command]
pub async fn handle_event(
    sender: tauri::State<'_, Sender<MusicState>>,
    event: String,
) -> Result<(), String> {
    let event: serde_json::Value =
        serde_json::from_str(&event).map_err(|e| format!("JSON解析错误:{}", e))?;
    let action = event["action"]
        .as_str()
        .ok_or_else(|| "Unknown action".to_string())?;
    match action {
        "play" => {
            let id = event["id"]
                .as_str()
                .ok_or_else(|| "play action missing id".to_string())?;
            sender
                .send(MusicState::Play(id.to_owned()))
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        "recovery" => {
            sender
                .send(MusicState::Recovery)
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        "pause" => {
            sender
                .send(MusicState::Pause)
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        "volume" => {
            let volume = event["volume"]
                .as_f64()
                .ok_or_else(|| "volume action missing volume".to_string())?;
            sender
                .send(MusicState::Volume(volume as f32))
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        "quit" => {
            sender
                .send(MusicState::Quit)
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        "seek" => {
            let time = event["time"]
                .as_f64()
                .ok_or_else(|| "seek action missing time".to_string())?;
            sender
                .send(MusicState::Seek(time as f32))
                .map_err(|_| "playback event channel closed".to_string())?;
        }
        _ => return Err(format!("Unknown action: {}", action)),
    }
    Ok(())
}
