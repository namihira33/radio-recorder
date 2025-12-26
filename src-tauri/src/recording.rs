use crate::radiko::{self, AuthToken};
use crate::nhk;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::process::Command;

lazy_static::lazy_static! {
    static ref ACTIVE_RECORDINGS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub struct RecordingError(String);

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RecordingError {}

pub async fn start_recording(
    token: &AuthToken,
    station_id: &str,
    duration_minutes: u32,
    output_name: &str,
) -> Result<String, RecordingError> {
    let stream_url = radiko::get_stream_url(token, station_id);

    // Create output directory
    let library_path = crate::library::get_library_path()
        .map_err(|e| RecordingError(e.to_string()))?;

    let station_dir = library_path.join(station_id);
    std::fs::create_dir_all(&station_dir)
        .map_err(|e| RecordingError(format!("Failed to create directory: {}", e)))?;

    let output_path = station_dir.join(format!("{}.mp3", output_name));
    let recording_id = uuid::Uuid::new_v4().to_string();

    // Start ffmpeg process
    let duration_seconds = duration_minutes * 60;

    let child = Command::new("ffmpeg")
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
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| RecordingError(format!("Failed to start ffmpeg: {}", e)))?;

    // Store the recording process ID
    if let Some(pid) = child.id() {
        let mut recordings = ACTIVE_RECORDINGS.lock().unwrap();
        recordings.insert(recording_id.clone(), pid);
    }

    // Save recording metadata
    save_recording_metadata(
        &recording_id,
        output_name,
        station_id,
        duration_minutes,
        &output_path,
    ).await?;

    Ok(recording_id)
}

pub async fn stop_recording(recording_id: &str) -> Result<(), RecordingError> {
    let pid = {
        let mut recordings = ACTIVE_RECORDINGS.lock().unwrap();
        recordings.remove(recording_id)
    };

    if let Some(pid) = pid {
        // Kill the process using system command
        std::process::Command::new("kill")
            .arg(pid.to_string())
            .output()
            .map_err(|e| RecordingError(format!("Failed to stop recording: {}", e)))?;
    }

    Ok(())
}

async fn save_recording_metadata(
    id: &str,
    title: &str,
    station: &str,
    duration: u32,
    file_path: &PathBuf,
) -> Result<(), RecordingError> {
    let metadata = crate::Recording {
        id: id.to_string(),
        title: title.to_string(),
        station: station.to_string(),
        recorded_at: chrono::Utc::now().to_rfc3339(),
        duration,
        file_path: file_path.to_string_lossy().to_string(),
        tags: vec![],
    };

    let library_path = crate::library::get_library_path()
        .map_err(|e| RecordingError(e.to_string()))?;

    let metadata_path = library_path.join("metadata.json");

    // Read existing metadata or create new
    let mut recordings: Vec<crate::Recording> = if metadata_path.exists() {
        let content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| RecordingError(format!("Failed to read metadata: {}", e)))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    recordings.push(metadata);

    let json = serde_json::to_string_pretty(&recordings)
        .map_err(|e| RecordingError(format!("Failed to serialize metadata: {}", e)))?;

    std::fs::write(&metadata_path, json)
        .map_err(|e| RecordingError(format!("Failed to write metadata: {}", e)))?;

    Ok(())
}

pub fn is_recording(recording_id: &str) -> bool {
    let recordings = ACTIVE_RECORDINGS.lock().unwrap();
    recordings.contains_key(recording_id)
}

// NHK recording (no authentication required)
pub async fn start_nhk_recording(
    station_id: &str,
    duration_minutes: u32,
    output_name: &str,
) -> Result<String, RecordingError> {
    let stream_url = nhk::get_stream_url(station_id, "tokyo");

    if stream_url.is_empty() {
        return Err(RecordingError("Unknown NHK station".to_string()));
    }

    // Create output directory
    let library_path = crate::library::get_library_path()
        .map_err(|e| RecordingError(e.to_string()))?;

    let station_name = match station_id {
        "r1" => "NHK-R1",
        "r2" => "NHK-R2",
        "fm" => "NHK-FM",
        _ => station_id,
    };

    let station_dir = library_path.join(station_name);
    std::fs::create_dir_all(&station_dir)
        .map_err(|e| RecordingError(format!("Failed to create directory: {}", e)))?;

    let output_path = station_dir.join(format!("{}.mp3", output_name));
    let recording_id = uuid::Uuid::new_v4().to_string();

    // Start ffmpeg process
    let duration_seconds = duration_minutes * 60;

    let child = Command::new("ffmpeg")
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
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| RecordingError(format!("Failed to start ffmpeg: {}", e)))?;

    // Store the recording process ID
    if let Some(pid) = child.id() {
        let mut recordings = ACTIVE_RECORDINGS.lock().unwrap();
        recordings.insert(recording_id.clone(), pid);
    }

    // Save recording metadata
    save_recording_metadata(
        &recording_id,
        output_name,
        station_name,
        duration_minutes,
        &output_path,
    ).await?;

    Ok(recording_id)
}
