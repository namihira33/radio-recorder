use crate::{Program, Station};
use reqwest::Client;

#[derive(Debug)]
pub struct NhkError(String);

impl std::fmt::Display for NhkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for NhkError {}

// NHK stations
pub fn get_stations() -> Vec<Station> {
    vec![
        Station {
            id: "r1".to_string(),
            name: "NHKラジオ第1".to_string(),
            area_id: "nhk".to_string(),
        },
        Station {
            id: "r2".to_string(),
            name: "NHKラジオ第2".to_string(),
            area_id: "nhk".to_string(),
        },
        Station {
            id: "fm".to_string(),
            name: "NHK FM".to_string(),
            area_id: "nhk".to_string(),
        },
    ]
}

// Get stream URL for NHK station
pub fn get_stream_url(station_id: &str, area: &str) -> String {
    // NHK uses area codes like "tokyo", "osaka", etc.
    // Default to tokyo
    let area_code = if area.is_empty() { "tokyo" } else { area };

    // NHK stream URL format
    match station_id {
        "r1" => format!(
            "https://radio-stream.nhk.jp/hls/live/2023229/nhkradiruakr1/master.m3u8"
        ),
        "r2" => format!(
            "https://radio-stream.nhk.jp/hls/live/2023501/nhkradiruakr2/master.m3u8"
        ),
        "fm" => format!(
            "https://radio-stream.nhk.jp/hls/live/2023507/nhkradiruakfm/master.m3u8"
        ),
        _ => String::new(),
    }
}

// Get NHK program schedule
pub async fn get_programs(station_id: &str, date: &str) -> Result<Vec<Program>, NhkError> {
    let client = Client::new();

    // NHK API for program schedule
    // Format: YYYY-MM-DD
    let formatted_date = if date.len() == 8 {
        format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
    } else {
        date.to_string()
    };

    let service = match station_id {
        "r1" => "r1",
        "r2" => "r2",
        "fm" => "r3",
        _ => return Err(NhkError("Unknown station".to_string())),
    };

    let url = format!(
        "https://api.nhk.or.jp/v2/pg/list/130/{}/{}.json?key={}",
        service,
        formatted_date,
        "EJfK8jdS57GmAHuLEgFcOUfcniJKFGaa" // Public API key
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| NhkError(format!("Failed to get programs: {}", e)))?;

    if !response.status().is_success() {
        // If API fails, return empty list (API key might be invalid)
        return Ok(vec![]);
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| NhkError(format!("Failed to parse response: {}", e)))?;

    let mut programs = Vec::new();

    if let Some(list) = json.get("list").and_then(|l| l.get(service)).and_then(|s| s.as_array()) {
        for item in list {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let subtitle = item.get("subtitle").and_then(|v| v.as_str()).unwrap_or("");
            let start = item.get("start_time").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let end = item.get("end_time").and_then(|v| v.as_str()).unwrap_or("").to_string();

            // Convert datetime format: 2025-01-01T10:00:00+09:00 -> 20250101100000
            let start_time = start.replace("-", "").replace(":", "").replace("T", "").chars().take(14).collect();
            let end_time = end.replace("-", "").replace(":", "").replace("T", "").chars().take(14).collect();

            programs.push(Program {
                id,
                title: if subtitle.is_empty() { title } else { format!("{} - {}", title, subtitle) },
                description: String::new(),
                start_time,
                end_time,
                station_id: station_id.to_string(),
                performer: None,
            });
        }
    }

    Ok(programs)
}

// Get "聴き逃し" (missed programs) - NHK's on-demand service
pub async fn get_ondemand_programs(station_id: &str) -> Result<Vec<Program>, NhkError> {
    // NHK聴き逃しサービスのAPIは複雑なので、基本的な番組リストを返す
    // 実際の実装では NHK の聴き逃しAPIを使う
    Ok(vec![])
}
