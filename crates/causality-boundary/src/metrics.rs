// Boundary metrics collection
// Original file: src/boundary/metrics.rs

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use lazy_static::lazy_static;

/// Statistics for boundary crossings
#[derive(Debug, Default, Clone)]
pub struct CrossingStats {
    /// Total number of boundary crossings
    pub count: usize,
    /// Total time spent in crossings
    pub total_time: Duration,
    /// Average time per crossing
    pub average_time: Duration,
    /// Maximum time for a single crossing
    pub max_time: Duration,
    /// Minimum time for a single crossing
    pub min_time: Duration,
    /// Number of successful crossings
    pub success_count: usize,
    /// Number of failed crossings
    pub error_count: usize,
}

lazy_static! {
    /// Global counter for all boundary crossings
    static ref TOTAL_CROSSINGS: AtomicUsize = AtomicUsize::new(0);
    
    /// Counters for each type of boundary crossing
    static ref CROSSING_COUNTS: RwLock<HashMap<String, AtomicUsize>> = RwLock::new(HashMap::new());
    
    /// Performance metrics for each type of boundary crossing
    static ref CROSSING_TIMES: RwLock<HashMap<String, Vec<Duration>>> = RwLock::new(HashMap::new());
    
    /// Error counts for each type of boundary crossing
    static ref CROSSING_ERRORS: RwLock<HashMap<String, AtomicUsize>> = RwLock::new(HashMap::new());
}

/// Record a boundary crossing
pub fn record_boundary_crossing(crossing_type: &str) {
    TOTAL_CROSSINGS.fetch_add(1, Ordering::SeqCst);
    
    // Update the count for this specific crossing type
    let mut counts = CROSSING_COUNTS.write().unwrap();
    let count = counts.entry(crossing_type.to_string())
        .or_insert_with(|| AtomicUsize::new(0));
    count.fetch_add(1, Ordering::SeqCst);
}

/// Start timing a boundary crossing
pub fn start_boundary_crossing_timer(crossing_type: &str) -> Instant {
    Instant::now()
}

/// Record the completion of a boundary crossing with timing
pub fn complete_boundary_crossing(crossing_type: &str, start_time: Instant, success: bool) {
    let duration = start_time.elapsed();
    
    // Record the time taken
    let mut times = CROSSING_TIMES.write().unwrap();
    let durations = times.entry(crossing_type.to_string())
        .or_insert_with(Vec::new);
    durations.push(duration);
    
    // Record error if the crossing failed
    if !success {
        let mut errors = CROSSING_ERRORS.write().unwrap();
        let error_count = errors.entry(crossing_type.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        error_count.fetch_add(1, Ordering::SeqCst);
    }
}

/// Get statistics for all boundary crossings
pub fn get_total_crossing_stats() -> usize {
    TOTAL_CROSSINGS.load(Ordering::SeqCst)
}

/// Get statistics for a specific type of boundary crossing
pub fn get_crossing_stats(crossing_type: &str) -> CrossingStats {
    let mut stats = CrossingStats::default();
    
    // Get the count
    if let Ok(counts) = CROSSING_COUNTS.read() {
        if let Some(count) = counts.get(crossing_type) {
            stats.count = count.load(Ordering::SeqCst);
        }
    }
    
    // Get error count
    if let Ok(errors) = CROSSING_ERRORS.read() {
        if let Some(error_count) = errors.get(crossing_type) {
            stats.error_count = error_count.load(Ordering::SeqCst);
            stats.success_count = stats.count - stats.error_count;
        }
    }
    
    // Calculate timing statistics
    if let Ok(times) = CROSSING_TIMES.read() {
        if let Some(durations) = times.get(crossing_type) {
            if !durations.is_empty() {
                stats.total_time = durations.iter().sum();
                stats.average_time = stats.total_time / durations.len() as u32;
                stats.max_time = *durations.iter().max().unwrap_or(&Duration::from_secs(0));
                stats.min_time = *durations.iter().min().unwrap_or(&Duration::from_secs(0));
            }
        }
    }
    
    stats
}

/// Reset all metrics
pub fn reset_metrics() {
    TOTAL_CROSSINGS.store(0, Ordering::SeqCst);
    
    if let Ok(mut counts) = CROSSING_COUNTS.write() {
        counts.clear();
    }
    
    if let Ok(mut times) = CROSSING_TIMES.write() {
        times.clear();
    }
    
    if let Ok(mut errors) = CROSSING_ERRORS.write() {
        errors.clear();
    }
}

/// Export metrics as JSON
pub fn export_metrics_json() -> String {
    let total = get_total_crossing_stats();
    let mut crossing_stats = HashMap::new();
    
    if let Ok(counts) = CROSSING_COUNTS.read() {
        for crossing_type in counts.keys() {
            crossing_stats.insert(crossing_type, get_crossing_stats(crossing_type));
        }
    }
    
    let metrics = serde_json::json!({
        "total_crossings": total,
        "crossing_types": crossing_stats.iter().map(|(k, v)| {
            (k.clone(), serde_json::json!({
                "count": v.count,
                "success_count": v.success_count,
                "error_count": v.error_count,
                "total_time_ms": v.total_time.as_millis(),
                "avg_time_ms": v.average_time.as_millis(),
                "max_time_ms": v.max_time.as_millis(),
                "min_time_ms": v.min_time.as_millis(),
            }))
        }).collect::<HashMap<_, _>>(),
    });
    
    serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "{}".to_string())
} 