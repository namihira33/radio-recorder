use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{radiko, nhk, recording, db, AppState};

// Request/Response types
#[derive(Deserialize)]
pub struct AuthRequest {
    email: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    success: bool,
    area_id: String,
    is_premium: bool,
}

#[derive(Deserialize)]
pub struct RecordingRequest {
    service_type: String,  // "nhk" or "radiko"
    station_id: String,
    station_name: String,
    title: String,
    duration: i32,  // minutes
}

#[derive(Deserialize)]
pub struct ScheduleRequest {
    station_id: String,
    station_name: String,
    service_type: String,
    title: String,
    start_time: String,
    duration: i32,
    repeat: Option<String>,
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

// Auth endpoints
pub async fn authenticate_radiko(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ApiError>)> {
    let email = req.email.unwrap_or_default();
    let password = req.password.unwrap_or_default();

    match radiko::authenticate(&email, &password).await {
        Ok(token) => {
            let response = AuthResponse {
                success: true,
                area_id: token.area_id.clone(),
                is_premium: token.is_premium,
            };

            let mut t = state.radiko_token.write().await;
            *t = Some(token);

            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

// Station endpoints
pub async fn get_nhk_stations() -> Json<Vec<nhk::Station>> {
    Json(nhk::get_stations())
}

pub async fn get_radiko_stations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<radiko::Station>>, (StatusCode, Json<ApiError>)> {
    let token = state.radiko_token.read().await;

    match &*token {
        Some(t) => match radiko::get_stations(t).await {
            Ok(stations) => Ok(Json(stations)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: e.to_string() }),
            )),
        },
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError { error: "Not authenticated".to_string() }),
        )),
    }
}

// Program endpoints
pub async fn get_nhk_programs(
    Path((station_id, date)): Path<(String, String)>,
) -> Result<Json<Vec<nhk::Program>>, (StatusCode, Json<ApiError>)> {
    match nhk::get_programs(&station_id, &date).await {
        Ok(programs) => Ok(Json(programs)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

pub async fn get_radiko_programs(
    State(state): State<Arc<AppState>>,
    Path((station_id, date)): Path<(String, String)>,
) -> Result<Json<Vec<radiko::Program>>, (StatusCode, Json<ApiError>)> {
    let token = state.radiko_token.read().await;

    match &*token {
        Some(t) => match radiko::get_programs(t, &station_id, &date).await {
            Ok(programs) => Ok(Json(programs)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: e.to_string() }),
            )),
        },
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError { error: "Not authenticated".to_string() }),
        )),
    }
}

// Recording endpoints
pub async fn get_recordings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<db::Recording>>, (StatusCode, Json<ApiError>)> {
    match state.db.get_recordings() {
        Ok(recordings) => Ok(Json(recordings)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

pub async fn start_recording(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let token = if req.service_type == "radiko" {
        let t = state.radiko_token.read().await;
        match &*t {
            Some(token) => Some(token.clone()),
            None => return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError { error: "Radiko authentication required".to_string() }),
            )),
        }
    } else {
        None
    };

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let safe_title = req.title.replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
    let output_name = format!("{}_{}", timestamp, safe_title);

    recording::spawn_recording(
        req.service_type,
        req.station_id,
        req.station_name,
        req.duration as u32,
        output_name,
        token,
        Arc::new(state.db.clone()),
    );

    Ok(Json(serde_json::json!({ "status": "recording_started" })))
}

pub async fn delete_recording(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    match state.db.delete_recording(&id) {
        Ok(Some(file_path)) => {
            // Delete the file
            if let Err(e) = std::fs::remove_file(&file_path) {
                tracing::warn!("Failed to delete file {}: {}", file_path, e);
            }
            Ok(Json(serde_json::json!({ "status": "deleted" })))
        }
        Ok(None) => Ok(Json(serde_json::json!({ "status": "deleted" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

// Schedule endpoints
pub async fn get_schedules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<db::ScheduledRecording>>, (StatusCode, Json<ApiError>)> {
    match state.db.get_schedules() {
        Ok(schedules) => Ok(Json(schedules)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScheduleRequest>,
) -> Result<Json<db::ScheduledRecording>, (StatusCode, Json<ApiError>)> {
    let schedule = db::ScheduledRecording {
        id: uuid::Uuid::new_v4().to_string(),
        station_id: req.station_id,
        station_name: req.station_name,
        service_type: req.service_type,
        title: req.title,
        start_time: req.start_time,
        duration: req.duration,
        enabled: true,
        repeat: req.repeat,
    };

    match state.db.add_schedule(&schedule) {
        Ok(_) => Ok(Json(schedule)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}

pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    match state.db.delete_schedule(&id) {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "deleted" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: e.to_string() }),
        )),
    }
}
