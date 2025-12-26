use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub area_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub station_id: String,
    pub performer: Option<String>,
}

#[derive(Debug)]
pub struct NhkError(pub String);

impl std::fmt::Display for NhkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for NhkError {}

pub fn get_stations() -> Vec<Station> {
    vec![
        Station {
            id: "r1".to_string(),
            name: "NHKラジオ第1".to_string(),
            area_id: "130".to_string(),
        },
        Station {
            id: "r2".to_string(),
            name: "NHKラジオ第2".to_string(),
            area_id: "130".to_string(),
        },
        Station {
            id: "fm".to_string(),
            name: "NHK FM".to_string(),
            area_id: "130".to_string(),
        },
    ]
}

pub fn get_stream_url(station_id: &str) -> String {
    match station_id {
        "r1" => "https://radio-stream.nhk.jp/hls/live/2023229/nhkradiruakr1/master.m3u8".to_string(),
        "r2" => "https://radio-stream.nhk.jp/hls/live/2023501/nhkradiruakr2/master.m3u8".to_string(),
        "fm" => "https://radio-stream.nhk.jp/hls/live/2023507/nhkradiruakfm/master.m3u8".to_string(),
        _ => String::new(),
    }
}

pub async fn get_programs(station_id: &str, date: &str) -> Result<Vec<Program>, NhkError> {
    if let Ok(programs) = get_programs_from_api(station_id, date).await {
        if !programs.is_empty() {
            return Ok(programs);
        }
    }

    Ok(get_fallback_programs(station_id, date))
}

async fn get_programs_from_api(station_id: &str, date: &str) -> Result<Vec<Program>, NhkError> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| NhkError(format!("Failed to create client: {}", e)))?;

    let formatted_date = normalize_date(date);
    let service = match station_id {
        "r1" => "r1",
        "r2" => "r2",
        "fm" => "r3",
        _ => return Err(NhkError("Unknown station".to_string())),
    };

    let url = format!(
        "https://api.nhk.or.jp/v2/pg/list/130/{}/{}.json?key=EJfK8jdS57GmAHuLEgFcOUfcniJKFGaa",
        service, formatted_date
    );

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Ok(vec![]),
    };

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    let json: serde_json::Value = match response.json().await {
        Ok(j) => j,
        Err(_) => return Ok(vec![]),
    };

    let mut programs = Vec::new();

    if let Some(list) = json.get("list").and_then(|l| l.get(service)).and_then(|s| s.as_array()) {
        for item in list {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let subtitle = item.get("subtitle").and_then(|v| v.as_str()).unwrap_or("");
            let start = item.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
            let end = item.get("end_time").and_then(|v| v.as_str()).unwrap_or("");

            if title.is_empty() || start.is_empty() {
                continue;
            }

            programs.push(Program {
                id: if id.is_empty() { format!("{}_{}", station_id, start) } else { id },
                title: if subtitle.is_empty() { title } else { format!("{} - {}", title, subtitle) },
                description: String::new(),
                start_time: normalize_time(start),
                end_time: normalize_time(end),
                station_id: station_id.to_string(),
                performer: item.get("act").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from),
            });
        }
    }

    Ok(programs)
}

fn normalize_date(date: &str) -> String {
    if date.len() == 8 && !date.contains('-') {
        format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
    } else {
        date.to_string()
    }
}

fn normalize_time(time_str: &str) -> String {
    if time_str.is_empty() {
        return String::new();
    }

    time_str
        .replace("-", "")
        .replace(":", "")
        .replace("T", "")
        .replace("+09:00", "")
        .replace("+0900", "")
        .replace("Z", "")
        .chars()
        .take(14)
        .collect()
}

fn get_fallback_programs(station_id: &str, date: &str) -> Vec<Program> {
    let station_name = match station_id {
        "r1" => "NHKラジオ第1",
        "r2" => "NHKラジオ第2",
        "fm" => "NHK FM",
        _ => "NHK",
    };

    let date_prefix = if date.len() >= 8 {
        date.replace("-", "").chars().take(8).collect::<String>()
    } else {
        chrono::Local::now().format("%Y%m%d").to_string()
    };

    let hours = [
        ("05", "07", "早朝ニュース・天気"),
        ("07", "09", "朝の情報番組"),
        ("09", "12", "午前の放送"),
        ("12", "13", "昼のニュース"),
        ("13", "17", "午後の放送"),
        ("17", "19", "夕方のニュース"),
        ("19", "21", "夜の情報番組"),
        ("21", "23", "夜の放送"),
        ("23", "24", "深夜放送"),
    ];

    hours
        .iter()
        .enumerate()
        .map(|(i, (start_h, end_h, desc))| Program {
            id: format!("{}_{}_{}", station_id, date_prefix, i),
            title: format!("{} - {}", station_name, desc),
            description: String::new(),
            start_time: format!("{}{}0000", date_prefix, start_h),
            end_time: format!("{}{}0000", date_prefix, end_h),
            station_id: station_id.to_string(),
            performer: None,
        })
        .collect()
}
