// utils.rs
use colored::*;
use std::fmt::Display;
#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}
#[allow(dead_code)]
pub fn print_score<T: Display>(label: &str, score: T) {
    println!("  {}: {}", label.bright_white(), score);
}
#[allow(dead_code)]
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let minutes = (seconds / 60.0) as u64;
        let secs = seconds % 60.0;
        format!("{}m {:.2}s", minutes, secs)
    }
}
