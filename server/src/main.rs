mod api;
mod radiko;
mod nhk;
mod recording;
mod db;
mod scheduler;

use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use tracing_subscriber;

pub struct AppState {
    pub db: db::Database,
    pub radiko_token: RwLock<Option<radiko::AuthToken>>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let db = db::Database::new("radio_recorder.db")
        .expect("Failed to initialize database");

    let state = Arc::new(AppState {
        db,
        radiko_token: RwLock::new(None),
    });

    // Start scheduler
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        scheduler::start_scheduler(scheduler_state).await;
    });

    // Build router
    let app = Router::new()
        // Auth
        .route("/api/auth/radiko", post(api::authenticate_radiko))
        // Stations
        .route("/api/stations/nhk", get(api::get_nhk_stations))
        .route("/api/stations/radiko", get(api::get_radiko_stations))
        // Programs
        .route("/api/programs/nhk/:station_id/:date", get(api::get_nhk_programs))
        .route("/api/programs/radiko/:station_id/:date", get(api::get_radiko_programs))
        // Recording
        .route("/api/recordings", get(api::get_recordings))
        .route("/api/recordings", post(api::start_recording))
        .route("/api/recordings/:id", delete(api::delete_recording))
        // Scheduled recordings
        .route("/api/schedules", get(api::get_schedules))
        .route("/api/schedules", post(api::create_schedule))
        .route("/api/schedules/:id", delete(api::delete_schedule))
        // Static files (recordings)
        .nest_service("/files", ServeDir::new("recordings"))
        // Web UI
        .fallback_service(ServeDir::new("static"))
        // CORS
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
