use tokio::sync::broadcast::Sender;

use crate::music_handler::handler::MusicState;

#[tauri::command]
pub async fn handle_event(
    sender: tauri::State<'_, Sender<MusicState>>,
    event: String,
) -> Result<(), String> {
    let event: serde_json::Value =
        serde_json::from_str(&event).map_err(|e| format!("JSON解析错误:{}", e))?;
    if let Some(act) = event["action"].as_str() {
        match act {
            "play" => {
                event["id"]
                    .as_str()
                    .map(|id| sender.send(MusicState::Play(id.to_owned())));
                Ok(())
            }
            "recovery" => {
                let _ = Some(sender.send(MusicState::Recovery));
                Ok(())
            }
            "pause" => {
                let _ = Some(sender.send(MusicState::Pause));
                Ok(())
            }
            "volume" => {
                let _ = event["volume"]
                    .as_f64()
                    .map(|vol| sender.send(MusicState::Volume(vol as f32)));
                Ok(())
            }
            "quit" => {
                let _ = Some(sender.send(MusicState::Quit));
                Ok(())
            }
            "seek" => {
                let _ = event["time"]
                    .as_f64()
                    .map(|t| sender.send(MusicState::Seek(t as f32)));
                Ok(())
            }
            _ => Ok(()),
        }
    } else {
        Err(format!("Unknown action"))
    }
}
