use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub station_id: String,
    pub station_name: String,
    pub service_type: String, // "nhk" or "radiko"
    pub duration: i32,
    pub file_path: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledRecording {
    pub id: String,
    pub station_id: String,
    pub station_name: String,
    pub service_type: String,
    pub title: String,
    pub start_time: String,  // ISO 8601
    pub duration: i32,       // minutes
    pub enabled: bool,
    pub repeat: Option<String>, // "daily", "weekly", "weekdays", null for one-time
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recordings (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                station_id TEXT NOT NULL,
                station_name TEXT NOT NULL,
                service_type TEXT NOT NULL,
                duration INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                recorded_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                station_id TEXT NOT NULL,
                station_name TEXT NOT NULL,
                service_type TEXT NOT NULL,
                title TEXT NOT NULL,
                start_time TEXT NOT NULL,
                duration INTEGER NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                repeat TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // Recordings
    pub fn add_recording(&self, recording: &Recording) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO recordings (id, title, station_id, station_name, service_type, duration, file_path, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                recording.id,
                recording.title,
                recording.station_id,
                recording.station_name,
                recording.service_type,
                recording.duration,
                recording.file_path,
                recording.recorded_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_recordings(&self) -> Result<Vec<Recording>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, station_id, station_name, service_type, duration, file_path, recorded_at
             FROM recordings ORDER BY recorded_at DESC"
        )?;

        let recordings = stmt.query_map([], |row| {
            Ok(Recording {
                id: row.get(0)?,
                title: row.get(1)?,
                station_id: row.get(2)?,
                station_name: row.get(3)?,
                service_type: row.get(4)?,
                duration: row.get(5)?,
                file_path: row.get(6)?,
                recorded_at: row.get(7)?,
            })
        })?;

        recordings.collect()
    }

    pub fn delete_recording(&self, id: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();

        // Get file path first
        let file_path: Option<String> = conn.query_row(
            "SELECT file_path FROM recordings WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ).ok();

        conn.execute("DELETE FROM recordings WHERE id = ?1", params![id])?;

        Ok(file_path)
    }

    // Schedules
    pub fn add_schedule(&self, schedule: &ScheduledRecording) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO schedules (id, station_id, station_name, service_type, title, start_time, duration, enabled, repeat)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                schedule.id,
                schedule.station_id,
                schedule.station_name,
                schedule.service_type,
                schedule.title,
                schedule.start_time,
                schedule.duration,
                schedule.enabled,
                schedule.repeat,
            ],
        )?;
        Ok(())
    }

    pub fn get_schedules(&self) -> Result<Vec<ScheduledRecording>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, station_id, station_name, service_type, title, start_time, duration, enabled, repeat
             FROM schedules ORDER BY start_time"
        )?;

        let schedules = stmt.query_map([], |row| {
            Ok(ScheduledRecording {
                id: row.get(0)?,
                station_id: row.get(1)?,
                station_name: row.get(2)?,
                service_type: row.get(3)?,
                title: row.get(4)?,
                start_time: row.get(5)?,
                duration: row.get(6)?,
                enabled: row.get(7)?,
                repeat: row.get(8)?,
            })
        })?;

        schedules.collect()
    }

    pub fn get_enabled_schedules(&self) -> Result<Vec<ScheduledRecording>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, station_id, station_name, service_type, title, start_time, duration, enabled, repeat
             FROM schedules WHERE enabled = 1 ORDER BY start_time"
        )?;

        let schedules = stmt.query_map([], |row| {
            Ok(ScheduledRecording {
                id: row.get(0)?,
                station_id: row.get(1)?,
                station_name: row.get(2)?,
                service_type: row.get(3)?,
                title: row.get(4)?,
                start_time: row.get(5)?,
                duration: row.get(6)?,
                enabled: row.get(7)?,
                repeat: row.get(8)?,
            })
        })?;

        schedules.collect()
    }

    pub fn delete_schedule(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_schedule_time(&self, id: &str, new_time: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE schedules SET start_time = ?1 WHERE id = ?2",
            params![new_time, id],
        )?;
        Ok(())
    }

    // Settings
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).optional()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
