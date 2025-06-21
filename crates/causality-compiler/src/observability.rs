//! Observability Module
//!
//! This module provides comprehensive logging, metrics, and monitoring capabilities
//! for real API calls and system operations.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Metrics collector for system operations
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    counters: Arc<Mutex<BTreeMap<String, u64>>>,
    gauges: Arc<Mutex<BTreeMap<String, f64>>>,
    histograms: Arc<Mutex<BTreeMap<String, Vec<f64>>>>,
    component: String,
}

impl MetricsCollector {
    pub fn new(component: &str) -> Self {
        Self {
            counters: Arc::new(Mutex::new(BTreeMap::new())),
            gauges: Arc::new(Mutex::new(BTreeMap::new())),
            histograms: Arc::new(Mutex::new(BTreeMap::new())),
            component: component.to_string(),
        }
    }
    
    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str, labels: Option<BTreeMap<String, String>>) {
        let metric_name = self.format_metric_name(name, labels);
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(metric_name).or_insert(0) += 1;
    }
    
    /// Set a gauge metric
    pub fn set_gauge(&self, name: &str, value: f64, labels: Option<BTreeMap<String, String>>) {
        let metric_name = self.format_metric_name(name, labels);
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(metric_name, value);
    }
    
    /// Record a histogram value (for timing, sizes, etc.)
    pub fn record_histogram(&self, name: &str, value: f64, labels: Option<BTreeMap<String, String>>) {
        let metric_name = self.format_metric_name(name, labels);
        let mut histograms = self.histograms.lock().unwrap();
        histograms.entry(metric_name).or_insert_with(Vec::new).push(value);
    }
    
    /// Record operation duration
    pub fn record_duration(&self, name: &str, duration: Duration, labels: Option<BTreeMap<String, String>>) {
        self.record_histogram(name, duration.as_secs_f64(), labels);
    }
    
    /// Get current metrics snapshot
    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.lock().unwrap().clone();
        let gauges = self.gauges.lock().unwrap().clone();
        let histograms = self.histograms.lock().unwrap().clone();
        
        MetricsSnapshot {
            component: self.component.clone(),
            timestamp: chrono::Utc::now(),
            counters,
            gauges,
            histograms: histograms.into_iter().map(|(k, v)| {
                let histogram_stats = HistogramStats::from_values(&v);
                (k, histogram_stats)
            }).collect(),
        }
    }
    
    fn format_metric_name(&self, name: &str, labels: Option<BTreeMap<String, String>>) -> String {
        let mut metric_name = format!("{}_{}", self.component, name);
        
        if let Some(labels) = labels {
            let mut label_parts: Vec<String> = labels.iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            label_parts.sort();
            
            if !label_parts.is_empty() {
                metric_name.push_str(&format!("{{{}}}", label_parts.join(",")));
            }
        }
        
        metric_name
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub counters: BTreeMap<String, u64>,
    pub gauges: BTreeMap<String, f64>,
    pub histograms: BTreeMap<String, HistogramStats>,
}

/// Statistics for histogram metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl HistogramStats {
    fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                count: 0,
                sum: 0.0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        Self {
            count,
            sum,
            min: sorted_values[0],
            max: sorted_values[count - 1],
            mean,
            p50: percentile(&sorted_values, 0.5),
            p95: percentile(&sorted_values, 0.95),
            p99: percentile(&sorted_values, 0.99),
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (p * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

/// Operation timer for measuring durations
pub struct OperationTimer {
    start_time: Instant,
    operation_name: String,
    metrics_collector: MetricsCollector,
    labels: Option<BTreeMap<String, String>>,
}

impl OperationTimer {
    pub fn new(
        operation_name: &str,
        metrics_collector: MetricsCollector,
        labels: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
            metrics_collector,
            labels,
        }
    }
    
    pub fn finish(self) -> Duration {
        let duration = self.start_time.elapsed();
        
        // Record timing metrics
        self.metrics_collector.record_duration(
            &format!("{}_duration", self.operation_name),
            duration,
            self.labels.clone(),
        );
        
        // Record operation count
        self.metrics_collector.increment_counter(
            &format!("{}_total", self.operation_name),
            self.labels,
        );
        
        duration
    }
    
    pub fn finish_with_result<T>(self, result: &Result<T, crate::error_handling::CausalityError>) -> Duration {
        let duration = self.start_time.elapsed();
        
        // Record timing metrics
        self.metrics_collector.record_duration(
            &format!("{}_duration", self.operation_name),
            duration,
            self.labels.clone(),
        );
        
        // Record operation count with result status
        let mut labels = self.labels.clone().unwrap_or_default();
        labels.insert("status".to_string(), if result.is_ok() { "success" } else { "error" }.to_string());
        
        self.metrics_collector.increment_counter(
            &format!("{}_total", self.operation_name),
            Some(labels),
        );
        
        // Record error count if failed
        if result.is_err() {
            self.metrics_collector.increment_counter(
                &format!("{}_errors", self.operation_name),
                self.labels.clone(),
            );
        }
        
        duration
    }
}

/// Structured logger for system operations
#[derive(Clone)]
pub struct StructuredLogger {
    component: String,
    enable_console: bool,
    enable_json: bool,
}

impl StructuredLogger {
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
            enable_console: true,
            enable_json: false,
        }
    }
    
    pub fn with_console(mut self, enable: bool) -> Self {
        self.enable_console = enable;
        self
    }
    
    pub fn with_json(mut self, enable: bool) -> Self {
        self.enable_json = enable;
        self
    }
    
    /// Log an operation start
    pub fn log_operation_start(&self, operation: &str, context: BTreeMap<String, String>) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            component: self.component.clone(),
            operation: operation.to_string(),
            message: format!("Starting operation: {}", operation),
            context,
            duration_ms: None,
            error: None,
        };
        
        self.emit_log(&log_entry);
    }
    
    /// Log an operation completion
    pub fn log_operation_complete(&self, operation: &str, duration: Duration, context: BTreeMap<String, String>) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            component: self.component.clone(),
            operation: operation.to_string(),
            message: format!("Completed operation: {} in {}ms", operation, duration.as_millis()),
            context,
            duration_ms: Some(duration.as_millis() as u64),
            error: None,
        };
        
        self.emit_log(&log_entry);
    }
    
    /// Log an operation error
    pub fn log_operation_error(
        &self,
        operation: &str,
        error: &crate::error_handling::CausalityError,
        context: BTreeMap<String, String>,
    ) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Error,
            component: self.component.clone(),
            operation: operation.to_string(),
            message: format!("Operation failed: {}", operation),
            context,
            duration_ms: None,
            error: Some(error.to_string()),
        };
        
        self.emit_log(&log_entry);
    }
    
    /// Log API call details
    pub fn log_api_call(
        &self,
        endpoint: &str,
        method: &str,
        status_code: Option<u16>,
        duration: Duration,
        error: Option<&str>,
    ) {
        let mut context = BTreeMap::new();
        context.insert("endpoint".to_string(), endpoint.to_string());
        context.insert("method".to_string(), method.to_string());
        
        if let Some(code) = status_code {
            context.insert("status_code".to_string(), code.to_string());
        }
        
        let level = if error.is_some() || status_code.map_or(false, |c| c >= 400) {
            LogLevel::Error
        } else {
            LogLevel::Info
        };
        
        let message = if let Some(err) = error {
            format!("API call to {} failed: {}", endpoint, err)
        } else {
            format!("API call to {} completed in {}ms", endpoint, duration.as_millis())
        };
        
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            component: self.component.clone(),
            operation: "api_call".to_string(),
            message,
            context,
            duration_ms: Some(duration.as_millis() as u64),
            error: error.map(|e| e.to_string()),
        };
        
        self.emit_log(&log_entry);
    }
    
    fn emit_log(&self, entry: &LogEntry) {
        if self.enable_console {
            self.emit_console_log(entry);
        }
        
        if self.enable_json {
            self.emit_json_log(entry);
        }
    }
    
    fn emit_console_log(&self, entry: &LogEntry) {
        let level_str = match entry.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        
        let context_str = if entry.context.is_empty() {
            String::new()
        } else {
            format!(" | {}", 
                entry.context.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        
        let duration_str = entry.duration_ms
            .map(|d| format!(" | {}ms", d))
            .unwrap_or_default();
        
        println!(
            "{} [{}:{}] {}{}{}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            entry.component,
            level_str,
            entry.message,
            context_str,
            duration_str
        );
        
        if let Some(error) = &entry.error {
            println!("  Error: {}", error);
        }
    }
    
    fn emit_json_log(&self, entry: &LogEntry) {
        if let Ok(json) = serde_json::to_string(entry) {
            println!("{}", json);
        }
    }
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    level: LogLevel,
    component: String,
    operation: String,
    message: String,
    context: BTreeMap<String, String>,
    duration_ms: Option<u64>,
    error: Option<String>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// System monitor that tracks overall system health and performance
pub struct SystemMonitor {
    metrics_collector: MetricsCollector,
    logger: StructuredLogger,
    monitoring_interval: Duration,
}

impl SystemMonitor {
    pub fn new(component: &str) -> Self {
        Self {
            metrics_collector: MetricsCollector::new(component),
            logger: StructuredLogger::new(component),
            monitoring_interval: Duration::from_secs(60),
        }
    }
    
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.monitoring_interval = interval;
        self
    }
    
    /// Start background monitoring
    pub async fn start_monitoring(&self) {
        let mut interval_timer = interval(self.monitoring_interval);
        let metrics_collector = self.metrics_collector.clone();
        let logger = self.logger.clone();
        
        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;
                
                // Collect system metrics
                let snapshot = metrics_collector.get_metrics_snapshot();
                
                // Log system status
                let mut context = BTreeMap::new();
                context.insert("counters_count".to_string(), snapshot.counters.len().to_string());
                context.insert("gauges_count".to_string(), snapshot.gauges.len().to_string());
                context.insert("histograms_count".to_string(), snapshot.histograms.len().to_string());
                
                logger.log_operation_complete(
                    "system_monitoring",
                    Duration::from_millis(0), // Monitoring doesn't have a meaningful duration
                    context,
                );
                
                // Log key metrics
                for (name, value) in &snapshot.counters {
                    if *value > 0 {
                        log::info!("Counter {}: {}", name, value);
                    }
                }
                
                for (name, value) in &snapshot.gauges {
                    log::info!("Gauge {}: {:.2}", name, value);
                }
                
                for (name, stats) in &snapshot.histograms {
                    if stats.count > 0 {
                        log::info!(
                            "Histogram {}: count={}, mean={:.2}ms, p95={:.2}ms",
                            name, stats.count, stats.mean * 1000.0, stats.p95 * 1000.0
                        );
                    }
                }
            }
        });
    }
    
    /// Get metrics collector for recording custom metrics
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics_collector
    }
    
    /// Get logger for structured logging
    pub fn logger(&self) -> &StructuredLogger {
        &self.logger
    }
    
    /// Create an operation timer
    pub fn start_operation_timer(
        &self,
        operation_name: &str,
        labels: Option<BTreeMap<String, String>>,
    ) -> OperationTimer {
        OperationTimer::new(operation_name, self.metrics_collector.clone(), labels)
    }
}

/// Macro for timing operations with automatic metrics collection
#[macro_export]
macro_rules! timed_operation {
    ($monitor:expr, $operation:expr, $code:block) => {{
        let timer = $monitor.start_operation_timer($operation, None);
        let result = $code;
        timer.finish_with_result(&result);
        result
    }};
    
    ($monitor:expr, $operation:expr, $labels:expr, $code:block) => {{
        let timer = $monitor.start_operation_timer($operation, Some($labels));
        let result = $code;
        timer.finish_with_result(&result);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new("test");
        
        // Test counter
        collector.increment_counter("requests", None);
        collector.increment_counter("requests", None);
        
        // Test gauge
        collector.set_gauge("memory_usage", 75.5, None);
        
        // Test histogram
        collector.record_histogram("response_time", 0.1, None);
        collector.record_histogram("response_time", 0.2, None);
        collector.record_histogram("response_time", 0.15, None);
        
        let snapshot = collector.get_metrics_snapshot();
        
        assert_eq!(snapshot.counters.get("test_requests"), Some(&2));
        assert_eq!(snapshot.gauges.get("test_memory_usage"), Some(&75.5));
        
        let response_time_stats = snapshot.histograms.get("test_response_time").unwrap();
        assert_eq!(response_time_stats.count, 3);
        assert_eq!(response_time_stats.min, 0.1);
        assert_eq!(response_time_stats.max, 0.2);
    }
    
    #[test]
    fn test_operation_timer() {
        let collector = MetricsCollector::new("test");
        let timer = OperationTimer::new("test_op", collector.clone(), None);
        
        thread::sleep(Duration::from_millis(10));
        let duration = timer.finish();
        
        assert!(duration >= Duration::from_millis(10));
        
        let snapshot = collector.get_metrics_snapshot();
        assert_eq!(snapshot.counters.get("test_test_op_total"), Some(&1));
    }
    
    #[test]
    fn test_structured_logger() {
        let logger = StructuredLogger::new("test");
        
        let mut context = BTreeMap::new();
        context.insert("user_id".to_string(), "123".to_string());
        
        // These should not panic
        logger.log_operation_start("test_operation", context.clone());
        logger.log_operation_complete("test_operation", Duration::from_millis(100), context);
    }
} 