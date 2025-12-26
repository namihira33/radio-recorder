use crate::radiko::AuthToken;
use crate::nhk;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug)]
pub struct RecordingError(pub String);

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RecordingError {}

pub fn get_recordings_dir() -> PathBuf {
    let dir = std::env::var("RECORDINGS_DIR")
        .unwrap_or_else(|_| "recordings".to_string());
    PathBuf::from(dir)
}

pub async fn record_radiko(
    token: &AuthToken,
    station_id: &str,
    station_name: &str,
    duration_minutes: u32,
    output_name: &str,
) -> Result<PathBuf, RecordingError> {
    let stream_url = crate::radiko::get_stream_url(token, station_id);

    let recordings_dir = get_recordings_dir();
    let station_dir = recordings_dir.join(station_id);
    std::fs::create_dir_all(&station_dir)
        .map_err(|e| RecordingError(format!("Failed to create directory: {}", e)))?;

    let output_path = station_dir.join(format!("{}.mp3", output_name));
    let duration_seconds = duration_minutes * 60;

    tracing::info!(
        "Starting radiko recording: {} for {} minutes",
        station_name,
        duration_minutes
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-headers", &format!("X-Radiko-AuthToken: {}", token.auth_token),
            "-i", &stream_url,
            "-t", &duration_seconds.to_string(),
            "-acodec", "libmp3lame",
            "-ab", "192k",
            "-ar", "44100",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .map_err(|e| RecordingError(format!("Failed to start ffmpeg: {}", e)))?;

    if !status.success() {
        return Err(RecordingError("ffmpeg recording failed".to_string()));
    }

    tracing::info!("Recording completed: {}", output_path.display());

    Ok(output_path)
}

pub async fn record_nhk(
    station_id: &str,
    station_name: &str,
    duration_minutes: u32,
    output_name: &str,
) -> Result<PathBuf, RecordingError> {
    let stream_url = nhk::get_stream_url(station_id);

    if stream_url.is_empty() {
        return Err(RecordingError("Unknown NHK station".to_string()));
    }

    let recordings_dir = get_recordings_dir();
    let station_dir = recordings_dir.join(station_id);
    std::fs::create_dir_all(&station_dir)
        .map_err(|e| RecordingError(format!("Failed to create directory: {}", e)))?;

    let output_path = station_dir.join(format!("{}.mp3", output_name));
    let duration_seconds = duration_minutes * 60;

    tracing::info!(
        "Starting NHK recording: {} for {} minutes",
        station_name,
        duration_minutes
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &stream_url,
            "-t", &duration_seconds.to_string(),
            "-acodec", "libmp3lame",
            "-ab", "192k",
            "-ar", "44100",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .map_err(|e| RecordingError(format!("Failed to start ffmpeg: {}", e)))?;

    if !status.success() {
        return Err(RecordingError("ffmpeg recording failed".to_string()));
    }

    tracing::info!("Recording completed: {}", output_path.display());

    Ok(output_path)
}

/// Start recording in background (non-blocking)
pub fn spawn_recording(
    service_type: String,
    station_id: String,
    station_name: String,
    duration_minutes: u32,
    output_name: String,
    token: Option<AuthToken>,
    db: std::sync::Arc<crate::db::Database>,
) {
    tokio::spawn(async move {
        let result = if service_type == "nhk" {
            record_nhk(&station_id, &station_name, duration_minutes, &output_name).await
        } else {
            match token {
                Some(t) => record_radiko(&t, &station_id, &station_name, duration_minutes, &output_name).await,
                None => Err(RecordingError("Radiko authentication required".to_string())),
            }
        };

        match result {
            Ok(file_path) => {
                let recording = crate::db::Recording {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: output_name.clone(),
                    station_id: station_id.clone(),
                    station_name: station_name.clone(),
                    service_type,
                    duration: duration_minutes as i32,
                    file_path: file_path.to_string_lossy().to_string(),
                    recorded_at: chrono::Utc::now().to_rfc3339(),
                };

                if let Err(e) = db.add_recording(&recording) {
                    tracing::error!("Failed to save recording metadata: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Recording failed: {}", e);
            }
        }
    });
}
