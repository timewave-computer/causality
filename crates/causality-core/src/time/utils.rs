// Placeholder for time utility functions

use chrono::Duration;

pub fn parse_duration(s: &str) -> Result<Duration, String> {
    // Basic parsing, enhance as needed
    if let Some(secs_str) = s.strip_suffix('s') {
        secs_str.parse::<i64>().map(Duration::seconds).map_err(|e| e.to_string())
    } else if let Some(ms_str) = s.strip_suffix("ms") {
        ms_str.parse::<i64>().map(Duration::milliseconds).map_err(|e| e.to_string())
    } else {
        Err(format!("Invalid duration format: {}", s))
    }
}

pub fn format_duration(d: Duration) -> String {
    // Basic formatting, enhance as needed
    if d.num_milliseconds() % 1000 == 0 {
        format!("{}s", d.num_seconds())
    } else {
        format!("{}ms", d.num_milliseconds())
    }
} 