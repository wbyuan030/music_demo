mod data;
mod search;
mod utils;
use anyhow::Result as InnerResult;
use futures::future::join_all;
use tauri::State;
use uuid::Uuid;

use crate::{
    music::TrackStore,
    music_fetch::bilibili::data::{search::get_media_source, search::Daum, MediaItem},
    types::{Track, TrackView},
};

fn parse_duration(duration: &str) -> f32 {
    let time_arr: Vec<f32> = duration
        .split(':')
        .map(|s| s.parse::<f32>().unwrap())
        .collect();
    let mut result: f32 = 0.0;
    for t in time_arr.iter() {
        result = result * 60.0;
        result += t;
    }
    result
}

#[tauri::command]
pub async fn search_music(
    state: State<'_, TrackStore>,
    keyword: &str,
) -> Result<Vec<TrackView>, String> {
    let _track_list = _search_music(keyword).await;
    let track_list = match _track_list {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{:?}", e);
            return Err(e.to_string());
        }
    };
    let track_view_list: Vec<TrackView> = track_list
        .into_iter()
        .map(|t| {
            let id = Uuid::new_v4().to_string();
            let track_view = TrackView::new(
                t.title.clone(),
                t.artist.clone(),
                t.cover_url.clone(),
                t.duration.clone(),
                id.clone(),
            );
            state.tracks.lock().unwrap().insert(id, t);
            track_view
        })
        .collect();
    Ok(track_view_list)
}

async fn _search_music(keyword: &str) -> InnerResult<Vec<Track>> {
    let raw_search_result = search::search_global(keyword).await?;
    let music_parsed: Vec<Daum> = raw_search_result
        .get("data")
        .expect("Missing data")
        .get("result")
        .expect("Missing result")
        .as_array()
        .expect("Result not array")
        .iter()
        .filter(|item| item["result_type"] == "video") // 1. 找到视频区
        .flat_map(|item| {
            item.get("data")
                .and_then(|d| d.as_array())
                .into_iter()
                .flatten()
        })
        .filter_map(|v| {
            // 3. 关键：将 Value 转换成你的结构体 DataResult
            // 注意：v 是 &Value，from_value 需要 Value，所以通常需要 clone
            match serde_json::from_value::<Daum>(v.clone()) {
                Ok(d) => Some(d),
                Err(e) => {
                    eprintln!("Error parsing json:{:?}", e);
                    None
                }
            }
        })
        .collect();

    let tasks = music_parsed.into_iter().map(|candidate| async move {
        let item = MediaItem::new(None, Some(candidate.bvid), Some(candidate.aid.to_string()));
        match get_media_source(item, data::MediaQuality::Standard).await {
            Ok(src) => Some(Track::new(
                candidate.title,
                candidate.author,
                candidate.pic,
                parse_duration(candidate.duration.as_str()),
                src,
            )),
            Err(e) => {
                eprintln!("Skipping Video {}:{:?}", candidate.title, e);
                None
            }
        }
    });
    let tracks: Vec<Track> = join_all(tasks).await.into_iter().flatten().collect();
    Ok(tracks)
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_search_bilibili() {
        let test_cases = vec!["红色高跟鞋", "红莲的弓矢", "起风了", "虎二", "1"];
        for data in test_cases {
            let res = super::_search_music(data).await;
            assert!(
                res.is_ok(),
                "关键词`{}` 搜索失败 错误:{}",
                data,
                res.unwrap_err().to_string()
            );
            println!("{:?}", res.unwrap())
        }
    }
}
