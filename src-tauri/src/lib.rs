mod radiko;
mod nhk;
mod recording;
mod library;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
pub struct AppState {
    auth_token: Mutex<Option<radiko::AuthToken>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub area_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Program {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub station_id: String,
    pub performer: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub station: String,
    pub recorded_at: String,
    pub duration: u32,
    pub file_path: String,
    pub tags: Vec<String>,
}

#[tauri::command]
async fn authenticate(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    match radiko::authenticate(&email, &password).await {
        Ok(token) => {
            let mut auth = state.auth_token.lock().unwrap();
            *auth = Some(token);
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_stations(state: State<'_, AppState>) -> Result<Vec<Station>, String> {
    let token = {
        let auth = state.auth_token.lock().unwrap();
        match &*auth {
            Some(t) => t.clone(),
            None => return Err("認証が必要です".to_string()),
        }
    };
    radiko::get_stations(&token).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_programs(
    station_id: String,
    date: String,
    state: State<'_, AppState>,
) -> Result<Vec<Program>, String> {
    let token = {
        let auth = state.auth_token.lock().unwrap();
        match &*auth {
            Some(t) => t.clone(),
            None => return Err("認証が必要です".to_string()),
        }
    };
    radiko::get_programs(&token, &station_id, &date)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_recording(
    station_id: String,
    duration_minutes: u32,
    output_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let token = {
        let auth = state.auth_token.lock().unwrap();
        match &*auth {
            Some(t) => t.clone(),
            None => return Err("認証が必要です".to_string()),
        }
    };
    recording::start_recording(&token, &station_id, duration_minutes, &output_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_recording(recording_id: String) -> Result<(), String> {
    recording::stop_recording(&recording_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_library() -> Result<Vec<Recording>, String> {
    library::get_recordings().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_recording(id: String) -> Result<(), String> {
    library::delete_recording(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_library_path() -> Result<String, String> {
    library::get_library_path()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

// NHK commands (no authentication required)
#[tauri::command]
fn get_nhk_stations() -> Vec<Station> {
    nhk::get_stations()
}

#[tauri::command]
async fn get_nhk_programs(station_id: String, date: String) -> Result<Vec<Program>, String> {
    nhk::get_programs(&station_id, &date)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_nhk_stream_url(station_id: String) -> String {
    nhk::get_stream_url(&station_id, "tokyo")
}

#[tauri::command]
async fn start_nhk_recording(
    station_id: String,
    duration_minutes: u32,
    output_name: String,
) -> Result<String, String> {
    recording::start_nhk_recording(&station_id, duration_minutes, &output_name)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            authenticate,
            get_stations,
            get_programs,
            start_recording,
            stop_recording,
            get_library,
            delete_recording,
            get_library_path,
            get_nhk_stations,
            get_nhk_programs,
            get_nhk_stream_url,
            start_nhk_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
