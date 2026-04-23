use chrono::TimeDelta;

use crate::subripfile::{Subtitle, SubtitleError};

/// Returns a formatted timestamp from milliseconds
pub fn timestamp_from_ms(duration: f64) -> String {
    let total_milliseconds = duration.trunc() as i64;
    let milliseconds = total_milliseconds.rem_euclid(1000);
    let total_seconds = total_milliseconds.div_euclid(1000);
    let seconds = total_seconds.rem_euclid(60);
    let total_minutes = total_seconds.div_euclid(60);
    let minutes = total_minutes.rem_euclid(60);
    let hours = total_minutes.div_euclid(60);

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours, minutes, seconds, milliseconds
    )
}

/// Returns a formatted timestamp from seconds
pub fn timestamp_from_seconds(duration: f64) -> String {
    timestamp_from_ms(duration * 1000.0)
}

/// Returns milliseconds from a timestamp
pub fn ms_from_timestamp(timestamp: &str) -> Result<i64, SubtitleError> {
    // Replace various separators with colons
    let normalized = timestamp.replace("T:", "").replace([';', '.', ','], ":");

    let parts: Vec<&str> = normalized.split(':').collect();
    if parts.len() != 4 {
        return Err(SubtitleError::Parse(format!(
            "Invalid timestamp format: {}",
            timestamp
        )));
    }

    let hours: i64 = parts[0]
        .parse()
        .map_err(|_| SubtitleError::Parse(format!("Invalid hours in timestamp: {}", timestamp)))?;
    let minutes: i64 = parts[1].parse().map_err(|_| {
        SubtitleError::Parse(format!("Invalid minutes in timestamp: {}", timestamp))
    })?;
    let seconds: i64 = parts[2].parse().map_err(|_| {
        SubtitleError::Parse(format!("Invalid seconds in timestamp: {}", timestamp))
    })?;
    let milliseconds: i64 = parts[3].parse().map_err(|_| {
        SubtitleError::Parse(format!("Invalid milliseconds in timestamp: {}", timestamp))
    })?;

    Ok(hours * 3600000 + minutes * 60000 + seconds * 1000 + milliseconds)
}

/// Returns TimeDelta from a timestamp
pub fn timedelta_from_timestamp(timestamp: &str) -> Result<TimeDelta, SubtitleError> {
    let ms = ms_from_timestamp(timestamp)?;
    Ok(TimeDelta::milliseconds(ms))
}

/// Returns TimeDelta from milliseconds
pub fn timedelta_from_ms(duration: f64) -> TimeDelta {
    TimeDelta::milliseconds(duration as i64)
}

/// Returns duration of a subtitle line
pub fn line_duration(line: &Subtitle) -> TimeDelta {
    (line.end - line.start).abs()
}
