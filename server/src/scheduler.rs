use std::sync::Arc;
use chrono::{DateTime, Utc, Duration, Datelike, Weekday};
use crate::{recording, AppState};

pub async fn start_scheduler(state: Arc<AppState>) {
    tracing::info!("Starting recording scheduler");

    loop {
        // Check every 30 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        if let Err(e) = check_schedules(&state).await {
            tracing::error!("Scheduler error: {}", e);
        }
    }
}

async fn check_schedules(state: &Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let schedules = state.db.get_enabled_schedules()?;
    let now = Utc::now();

    for schedule in schedules {
        let start_time: DateTime<Utc> = schedule.start_time.parse()?;

        // Check if it's time to start (within 1 minute window)
        let diff = start_time.signed_duration_since(now);
        if diff >= Duration::seconds(-30) && diff <= Duration::seconds(30) {
            tracing::info!("Starting scheduled recording: {}", schedule.title);

            let token = if schedule.service_type == "radiko" {
                let t = state.radiko_token.read().await;
                t.clone()
            } else {
                None
            };

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let safe_title = schedule.title.replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
            let output_name = format!("{}_{}", timestamp, safe_title);

            // Clone db for the spawned task
            let db = Arc::new(state.db.clone());

            recording::spawn_recording(
                schedule.service_type.clone(),
                schedule.station_id.clone(),
                schedule.station_name.clone(),
                schedule.duration as u32,
                output_name,
                token,
                db,
            );

            // Handle repeat scheduling
            match schedule.repeat.as_deref() {
                Some("daily") => {
                    let next_time = start_time + Duration::days(1);
                    state.db.update_schedule_time(&schedule.id, &next_time.to_rfc3339())?;
                }
                Some("weekly") => {
                    let next_time = start_time + Duration::weeks(1);
                    state.db.update_schedule_time(&schedule.id, &next_time.to_rfc3339())?;
                }
                Some("weekdays") => {
                    let mut next_time = start_time + Duration::days(1);
                    // Skip weekends
                    while matches!(next_time.weekday(), Weekday::Sat | Weekday::Sun) {
                        next_time = next_time + Duration::days(1);
                    }
                    state.db.update_schedule_time(&schedule.id, &next_time.to_rfc3339())?;
                }
                _ => {
                    // One-time recording, delete the schedule
                    state.db.delete_schedule(&schedule.id)?;
                }
            }
        }
    }

    Ok(())
}
