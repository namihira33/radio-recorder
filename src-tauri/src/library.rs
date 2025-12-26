use crate::Recording;
use std::path::PathBuf;

#[derive(Debug)]
pub struct LibraryError(String);

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LibraryError {}

pub fn get_library_path() -> Result<PathBuf, LibraryError> {
    let home = dirs::home_dir()
        .ok_or_else(|| LibraryError("Could not find home directory".to_string()))?;

    let library_path = home.join("RadioLibrary");

    // Create directory if it doesn't exist
    if !library_path.exists() {
        std::fs::create_dir_all(&library_path)
            .map_err(|e| LibraryError(format!("Failed to create library directory: {}", e)))?;
    }

    Ok(library_path)
}

pub async fn get_recordings() -> Result<Vec<Recording>, LibraryError> {
    let library_path = get_library_path()?;
    let metadata_path = library_path.join("metadata.json");

    if !metadata_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| LibraryError(format!("Failed to read metadata: {}", e)))?;

    let recordings: Vec<Recording> = serde_json::from_str(&content)
        .map_err(|e| LibraryError(format!("Failed to parse metadata: {}", e)))?;

    // Filter out recordings whose files no longer exist
    let valid_recordings: Vec<Recording> = recordings
        .into_iter()
        .filter(|r| PathBuf::from(&r.file_path).exists())
        .collect();

    Ok(valid_recordings)
}

pub async fn delete_recording(id: &str) -> Result<(), LibraryError> {
    let library_path = get_library_path()?;
    let metadata_path = library_path.join("metadata.json");

    if !metadata_path.exists() {
        return Err(LibraryError("Metadata file not found".to_string()));
    }

    let content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| LibraryError(format!("Failed to read metadata: {}", e)))?;

    let mut recordings: Vec<Recording> = serde_json::from_str(&content)
        .map_err(|e| LibraryError(format!("Failed to parse metadata: {}", e)))?;

    // Find and remove the recording
    if let Some(pos) = recordings.iter().position(|r| r.id == id) {
        let recording = recordings.remove(pos);

        // Delete the file
        let file_path = PathBuf::from(&recording.file_path);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| LibraryError(format!("Failed to delete file: {}", e)))?;
        }

        // Save updated metadata
        let json = serde_json::to_string_pretty(&recordings)
            .map_err(|e| LibraryError(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&metadata_path, json)
            .map_err(|e| LibraryError(format!("Failed to write metadata: {}", e)))?;
    }

    Ok(())
}

pub async fn add_tag(id: &str, tag: &str) -> Result<(), LibraryError> {
    let library_path = get_library_path()?;
    let metadata_path = library_path.join("metadata.json");

    let content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| LibraryError(format!("Failed to read metadata: {}", e)))?;

    let mut recordings: Vec<Recording> = serde_json::from_str(&content)
        .map_err(|e| LibraryError(format!("Failed to parse metadata: {}", e)))?;

    if let Some(recording) = recordings.iter_mut().find(|r| r.id == id) {
        if !recording.tags.contains(&tag.to_string()) {
            recording.tags.push(tag.to_string());
        }
    }

    let json = serde_json::to_string_pretty(&recordings)
        .map_err(|e| LibraryError(format!("Failed to serialize metadata: {}", e)))?;

    std::fs::write(&metadata_path, json)
        .map_err(|e| LibraryError(format!("Failed to write metadata: {}", e)))?;

    Ok(())
}

pub async fn remove_tag(id: &str, tag: &str) -> Result<(), LibraryError> {
    let library_path = get_library_path()?;
    let metadata_path = library_path.join("metadata.json");

    let content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| LibraryError(format!("Failed to read metadata: {}", e)))?;

    let mut recordings: Vec<Recording> = serde_json::from_str(&content)
        .map_err(|e| LibraryError(format!("Failed to parse metadata: {}", e)))?;

    if let Some(recording) = recordings.iter_mut().find(|r| r.id == id) {
        recording.tags.retain(|t| t != tag);
    }

    let json = serde_json::to_string_pretty(&recordings)
        .map_err(|e| LibraryError(format!("Failed to serialize metadata: {}", e)))?;

    std::fs::write(&metadata_path, json)
        .map_err(|e| LibraryError(format!("Failed to write metadata: {}", e)))?;

    Ok(())
}
