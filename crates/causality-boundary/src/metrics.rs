// Boundary metrics collection
// Original file: src/boundary/metrics.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    // Total number of boundary crossings
    static ref TOTAL_CROSSINGS: RwLock<u64> = RwLock::new(0);
    
    // Count of crossings by type
    static ref CROSSING_COUNTS: RwLock<HashMap<String, u64>> = RwLock::new(HashMap::new());
    
    // Average crossing time by type
    static ref CROSSING_TIMES: RwLock<HashMap<String, Duration>> = RwLock::new(HashMap::new());
    
    // Crossing errors
    static ref CROSSING_ERRORS: RwLock<HashMap<String, u64>> = RwLock::new(HashMap::new());
}

/// Record a boundary crossing
pub fn record_boundary_crossing(crossing_type: &str) {
    // Increment total crossings
    {
        let mut total = TOTAL_CROSSINGS.write().unwrap();
        *total += 1;
    }
    
    // Increment count for specific crossing type
    {
        let mut counts = CROSSING_COUNTS.write().unwrap();
        let count = counts.entry(crossing_type.to_string()).or_insert(0);
        *count += 1;
    }
}

/// Start timing a boundary crossing
pub fn start_boundary_crossing_timer(crossing_type: &str) -> Instant {
    Instant::now()
}

/// Record a crossing error
pub fn record_boundary_crossing_error(crossing_type: &str, error: &str) {
    let mut errors = CROSSING_ERRORS.write().unwrap();
    let count = errors.entry(format!("{}:{}", crossing_type, error)).or_insert(0);
    *count += 1;
}

/// Complete a boundary crossing timing
pub fn complete_boundary_crossing(crossing_type: &str, start: Instant, success: bool) {
    // Calculate duration
    let duration = start.elapsed();
    
    // Record timing
    {
        let mut times = CROSSING_TIMES.write().unwrap();
        let avg_time = times.entry(crossing_type.to_string()).or_insert(Duration::from_secs(0));
        
        // Simple moving average (this could be improved)
        if *avg_time == Duration::from_secs(0) {
            *avg_time = duration;
        } else {
            *avg_time = (*avg_time + duration) / 2;
        }
    }
    
    // Record error if not successful
    if !success {
        record_boundary_crossing_error(crossing_type, "generic_failure");
    }
}

/// Get the total number of crossings
pub fn get_total_crossings() -> u64 {
    let total = TOTAL_CROSSINGS.read().unwrap();
    *total
}

/// Get crossing counts by type
pub fn get_crossing_counts() -> HashMap<String, u64> {
    let counts = CROSSING_COUNTS.read().unwrap();
    counts.clone()
}

/// Get crossing errors
pub fn get_crossing_errors() -> HashMap<String, u64> {
    if let Ok(errors) = CROSSING_ERRORS.read() {
        errors.clone()
    } else {
        HashMap::new()
    }
}

/// Get average crossing times
pub fn get_crossing_times() -> HashMap<String, Duration> {
    if let Ok(times) = CROSSING_TIMES.read() {
        times.clone()
    } else {
        HashMap::new()
    }
}

/// Reset all metrics
pub fn reset_metrics() {
    // Reset total crossings
    if let Ok(mut total) = TOTAL_CROSSINGS.write() {
        *total = 0;
    }
    
    // Reset crossing counts
    if let Ok(mut counts) = CROSSING_COUNTS.write() {
        counts.clear();
    }
    
    // Reset crossing times
    if let Ok(mut times) = CROSSING_TIMES.write() {
        times.clear();
    }
    
    // Reset crossing errors
    if let Ok(mut errors) = CROSSING_ERRORS.write() {
        errors.clear();
    }
}

/// Export metrics as JSON
pub fn export_metrics_json() -> String {
    let counts = get_crossing_counts();
    
    let times_map: HashMap<String, u64> = get_crossing_times()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_millis() as u64))
        .collect();
    
    let json = serde_json::json!({
        "total_crossings": get_total_crossings(),
        "counts_by_type": counts,
        "avg_time_ms_by_type": times_map,
        "errors": get_crossing_errors(),
    });
    
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
} 