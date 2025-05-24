//! Utility functions for the dashboard
//!
//! This module provides utility functions for the dashboard.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Read a JSON file
pub fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let value = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {}", path.display()))?;
    Ok(value)
}

/// Write a JSON file
pub fn write_json_file<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;

    let file =
        File::create(path).with_context(|| format!("Failed to create file: {}", path.display()))?;
    let writer = io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value)
        .with_context(|| format!("Failed to write JSON to file: {}", path.display()))?;
    Ok(())
}

/// Read a CSV file
pub fn read_csv_file<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut records = Vec::new();

    for result in csv_reader.deserialize() {
        let record: T = result
            .with_context(|| format!("Failed to parse CSV record from file: {}", path.display()))?;
        records.push(record);
    }

    Ok(records)
}

/// Write a CSV file
pub fn write_csv_file<T>(path: &Path, records: &[T]) -> Result<()>
where
    T: Serialize,
{
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;

    let file =
        File::create(path).with_context(|| format!("Failed to create file: {}", path.display()))?;
    let writer = io::BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    for record in records {
        csv_writer
            .serialize(record)
            .with_context(|| format!("Failed to write CSV record to file: {}", path.display()))?;
    }

    csv_writer
        .flush()
        .with_context(|| format!("Failed to flush CSV writer for file: {}", path.display()))?;
    Ok(())
}

/// Find the latest file with a given prefix and extension
pub fn find_latest_file(dir: &Path, prefix: &str, extension: &str) -> Option<std::path::PathBuf> {
    if !dir.exists() {
        return None;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    let mut latest_file = None;
    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with(prefix) && file_name_str.ends_with(extension) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest_file = Some(entry.path());
                        }
                    }
                }
            }
        }
    }

    latest_file
}

/// Format a timestamp as a human-readable string
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a duration as a human-readable string
pub fn format_duration(duration: &std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a number with commas
pub fn format_number(number: u64) -> String {
    let mut result = String::new();
    let number_str = number.to_string();
    let mut count = 0;

    for (i, c) in number_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

/// Format a percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format a file size
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.2} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

/// Get a color for a status level
pub fn status_level_color(level: &crate::data::StatusLevel) -> &'static str {
    use crate::data::StatusLevel;

    match level {
        StatusLevel::Critical => "#d9534f",
        StatusLevel::Error => "#f0ad4e",
        StatusLevel::Warning => "#f0ad4e",
        StatusLevel::Info => "#5bc0de",
        StatusLevel::Success => "#5cb85c",
    }
}

/// Get a color for a value in a range
pub fn value_color(value: f64, min: f64, max: f64, invert: bool) -> String {
    let normalized = (value - min) / (max - min);
    let normalized = normalized.max(0.0).min(1.0);

    let normalized = if invert { 1.0 - normalized } else { normalized };

    let r = (255.0 * (1.0 - normalized)).round() as u8;
    let g = (255.0 * normalized).round() as u8;
    let b = 0;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Generate a random color
pub fn random_color() -> String {
    let r = rand::random::<u8>();
    let g = rand::random::<u8>();
    let b = rand::random::<u8>();

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Generate a set of distinct colors
pub fn generate_distinct_colors(count: usize) -> Vec<String> {
    let mut colors = Vec::with_capacity(count);

    for i in 0..count {
        let hue = (i as f64 / count as f64) * 360.0;
        let saturation = 0.7;
        let lightness = 0.5;

        colors.push(hsl_to_hex(hue, saturation, lightness));
    }

    colors
}

/// Convert HSL to hex color
fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0).round() as u8;
    let g = ((g + m) * 255.0).round() as u8;
    let b = ((b + m) * 255.0).round() as u8;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
