//! Performance Metrics Collection and Analysis
//!
//! This module provides interfaces and implementations for collecting,
//! analyzing, and visualizing performance metrics in a ZK-compatible manner.

use async_trait::async_trait;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use causality_types::core::{
    AsErrorContext, ContextualError, ErrorCategory, ErrorMetadata,
};

//-----------------------------------------------------------------------------
// Performance Metrics and Analysis
//-----------------------------------------------------------------------------

/// Maximum number of data points per metric
pub const MAX_DATA_POINTS: usize = 1000;

/// Maximum number of metrics to track
pub const MAX_METRICS: usize = 100;

/// Metric type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricType {
    /// Counter (monotonically increasing value)
    Counter,

    /// Gauge (value that can increase or decrease)
    Gauge,

    /// Histogram (distribution of values)
    Histogram,

    /// Timer (duration measurements)
    Timer,
}

/// A single data point for a metric
#[derive(Debug, Clone)]
pub struct MetricDataPoint {
    /// Timestamp (nanoseconds since epoch)
    pub timestamp: u64,

    /// Value (as serializable bytes)
    pub value: [u8; 8],

    /// Associated metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
}

/// A complete metric with data points
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Data points
    pub data_points: Vec<MetricDataPoint>,

    /// Creation timestamp
    pub created_at: u64,

    /// Last updated timestamp
    pub updated_at: u64,

    /// Description
    pub description: String,

    /// Unit of measurement
    pub unit: String,

    /// Tags
    pub tags: Vec<String>,
}

/// Metric ID
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct MetricId([u8; 16]);

impl Default for MetricId {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricId {
    /// Create a new random metric ID
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        Self(bytes)
    }

    /// Convert to hexadecimal string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Try to parse from hexadecimal string
    pub fn from_hex(hex_str: &str) -> Result<Self, ContextualError> {
        let bytes = hex::decode(hex_str).map_err(|_| {
            ContextualError::new(
                "Invalid hex string for metric ID",
                ErrorMetadata::new(ErrorCategory::Validation),
            )
        })?;

        if bytes.len() != 16 {
            return Err(ContextualError::new(
                "Metric ID must be 16 bytes",
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        let mut id = [0u8; 16];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }
}

/// Statistics for a metric
#[derive(Debug, Clone)]
pub struct MetricStats {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Minimum value (as serializable bytes)
    pub min: [u8; 8],

    /// Maximum value (as serializable bytes)
    pub max: [u8; 8],

    /// Average value (as serializable bytes)
    pub avg: [u8; 8],

    /// Median value (as serializable bytes)
    pub median: [u8; 8],

    /// 90th percentile (as serializable bytes)
    pub p90: [u8; 8],

    /// 95th percentile (as serializable bytes)
    pub p95: [u8; 8],

    /// 99th percentile (as serializable bytes)
    pub p99: [u8; 8],

    /// Standard deviation (as serializable bytes)
    pub std_dev: [u8; 8],

    /// Sample count
    pub sample_count: u64,

    /// Time range start (nanoseconds since epoch)
    pub range_start: u64,

    /// Time range end (nanoseconds since epoch)
    pub range_end: u64,
}

/// Interface for collecting and analyzing metrics
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Register a new metric
    async fn register_metric(
        &self,
        name: &str,
        metric_type: MetricType,
        description: &str,
        unit: &str,
        tags: Vec<String>,
    ) -> Result<MetricId, ContextualError>;

    /// Record a data point for a metric
    async fn record_data_point(
        &self,
        metric_id: &MetricId,
        value: f64,
        metadata: HashMap<String, String>,
    ) -> Result<(), ContextualError>;

    /// Get a metric by ID
    async fn get_metric(
        &self,
        metric_id: &MetricId,
    ) -> Result<Metric, ContextualError>;

    /// Find a metric by name
    async fn find_metric_by_name(
        &self,
        name: &str,
    ) -> Result<(MetricId, Metric), ContextualError>;

    /// List all metrics
    async fn list_metrics(
        &self,
    ) -> Result<Vec<(MetricId, String, MetricType)>, ContextualError>;

    /// Get statistics for a metric
    async fn get_metric_stats(
        &self,
        metric_id: &MetricId,
    ) -> Result<MetricStats, ContextualError>;

    /// Delete a metric
    async fn delete_metric(
        &self,
        metric_id: &MetricId,
    ) -> Result<(), ContextualError>;
}

/// Interface for analyzing metrics
#[async_trait]
pub trait MetricsAnalyzer: Send + Sync {
    /// Analyze a metric over a time range
    async fn analyze_metric(
        &self,
        metric_id: &MetricId,
        start_time: u64,
        end_time: u64,
    ) -> Result<MetricAnalysis, ContextualError>;

    /// Find correlations between metrics
    async fn find_correlations(
        &self,
        metric_ids: &[MetricId],
    ) -> Result<Vec<MetricCorrelation>, ContextualError>;

    /// Detect anomalies in a metric
    async fn detect_anomalies(
        &self,
        metric_id: &MetricId,
    ) -> Result<Vec<MetricAnomaly>, ContextualError>;

    /// Forecast future values for a metric
    async fn forecast_metric(
        &self,
        metric_id: &MetricId,
        forecast_periods: usize,
    ) -> Result<MetricForecast, ContextualError>;
}

/// Result of metric analysis
#[derive(Debug, Clone)]
pub struct MetricAnalysis {
    /// Metric ID
    pub metric_id: MetricId,

    /// Metric statistics
    pub stats: MetricStats,

    /// Trend analysis
    pub trend: MetricTrend,

    /// Detected anomalies
    pub anomalies: Vec<MetricAnomaly>,

    /// Seasonality analysis
    pub seasonality: Option<MetricSeasonality>,
}

/// Metric trend analysis
#[derive(Debug, Clone)]
pub struct MetricTrend {
    /// Trend direction
    pub direction: TrendDirection,

    /// Trend strength (0-100)
    pub strength: u8,

    /// Slope (rate of change per second, as serializable bytes)
    pub slope: [u8; 8],

    /// Volatility (standard deviation of rate of change, as serializable bytes)
    pub volatility: [u8; 8],
}

/// Trend direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrendDirection {
    /// Increasing trend
    Increasing,

    /// Decreasing trend
    Decreasing,

    /// Flat/no trend
    Flat,

    /// Volatile (no clear direction)
    Volatile,
}

/// A detected anomaly in a metric
#[derive(Debug, Clone)]
pub struct MetricAnomaly {
    /// Anomaly type
    pub anomaly_type: AnomalyType,

    /// Timestamp when the anomaly occurred
    pub timestamp: u64,

    /// Expected value (as serializable bytes)
    pub expected_value: [u8; 8],

    /// Actual value (as serializable bytes)
    pub actual_value: [u8; 8],

    /// Severity (0-100)
    pub severity: u8,

    /// Description
    pub description: String,
}

/// Type of anomaly
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// Spike (sudden increase)
    Spike,

    /// Dip (sudden decrease)
    Dip,

    /// Level shift (sustained change)
    LevelShift,

    /// Trend change
    TrendChange,

    /// Missing data
    MissingData,

    /// Unusual variance
    UnusualVariance,
}

/// Seasonality analysis for a metric
#[derive(Debug, Clone)]
pub struct MetricSeasonality {
    /// Period length (nanoseconds)
    pub period: u64,

    /// Seasonality strength (0-100)
    pub strength: u8,

    /// Number of detected cycles
    pub cycle_count: u64,
}

/// Correlation between two metrics
#[derive(Debug, Clone)]
pub struct MetricCorrelation {
    /// First metric ID
    pub metric_id1: MetricId,

    /// Second metric ID
    pub metric_id2: MetricId,

    /// Correlation coefficient (-1.0 to 1.0, as serializable bytes)
    pub correlation: [u8; 8],

    /// Lag (nanoseconds)
    pub lag: i64,

    /// Strength description
    pub strength_description: String,

    /// Potential causality indicator (0-100)
    pub causality_indicator: u8,
}

/// Forecast for future metric values
#[derive(Debug, Clone)]
pub struct MetricForecast {
    /// Metric ID
    pub metric_id: MetricId,

    /// Forecasted values
    pub values: Vec<ForecastPoint>,

    /// Model used for forecasting
    pub model: String,

    /// Forecast accuracy (0-100)
    pub accuracy: u8,

    /// Confidence level (0-100)
    pub confidence: u8,
}

/// A single forecasted point
#[derive(Debug, Clone)]
pub struct ForecastPoint {
    /// Timestamp (nanoseconds since epoch)
    pub timestamp: u64,

    /// Forecasted value
    pub value: f64,

    /// Lower bound of confidence interval
    pub lower_bound: f64,

    /// Upper bound of confidence interval
    pub upper_bound: f64,
}

/// In-memory implementation of metrics collector
pub struct InMemoryMetricsCollector {
    /// Map of metric ID to metric
    metrics: tokio::sync::Mutex<HashMap<[u8; 16], Metric>>,

    /// Map of metric name to metric ID
    metrics_by_name: tokio::sync::Mutex<HashMap<String, [u8; 16]>>,

    /// Error context
    error_context: Arc<dyn AsErrorContext>,
}

impl InMemoryMetricsCollector {
    /// Create a new in-memory metrics collector
    pub fn new(error_context: Arc<dyn AsErrorContext>) -> Self {
        Self {
            metrics: tokio::sync::Mutex::new(HashMap::new()),
            metrics_by_name: tokio::sync::Mutex::new(HashMap::new()),
            error_context,
        }
    }

    /// Get the current timestamp in nanoseconds
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Helper function to convert [u8; 8] to f64
    fn bytes_to_f64(bytes: [u8; 8]) -> f64 {
        f64::from_le_bytes(bytes)
    }

    /// Helper function to convert f64 to [u8; 8]
    fn f64_to_bytes(value: f64) -> [u8; 8] {
        value.to_le_bytes()
    }

    /// Calculate statistics for a given metric
    fn calculate_stats(&self, metric: &Metric) -> MetricStats {
        let mut values: Vec<f64> =
            metric.data_points.iter().map(|dp| Self::bytes_to_f64(dp.value)).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = values.len();
        if count == 0 {
            return MetricStats {
                name: metric.name.clone(),
                metric_type: metric.metric_type.clone(),
                min: Self::f64_to_bytes(0.0),
                max: Self::f64_to_bytes(0.0),
                avg: Self::f64_to_bytes(0.0),
                median: Self::f64_to_bytes(0.0),
                p90: Self::f64_to_bytes(0.0),
                p95: Self::f64_to_bytes(0.0),
                p99: Self::f64_to_bytes(0.0),
                std_dev: Self::f64_to_bytes(0.0),
                sample_count: 0,
                range_start: metric.created_at,
                range_end: metric.updated_at,
            };
        }

        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        let median = if count % 2 == 0 {
            (values[count / 2 - 1] + values[count / 2]) / 2.0
        } else {
            values[count / 2]
        };

        let p90 = values[(count as f64 * 0.9).floor() as usize];
        let p95 = values[(count as f64 * 0.95).floor() as usize];
        let p99 = values[(count as f64 * 0.99).floor() as usize];

        let variance: f64 =
            values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        MetricStats {
            name: metric.name.clone(),
            metric_type: metric.metric_type.clone(),
            min: Self::f64_to_bytes(*values.first().unwrap_or(&0.0)),
            max: Self::f64_to_bytes(*values.last().unwrap_or(&0.0)),
            avg: Self::f64_to_bytes(avg),
            median: Self::f64_to_bytes(median),
            p90: Self::f64_to_bytes(p90),
            p95: Self::f64_to_bytes(p95),
            p99: Self::f64_to_bytes(p99),
            std_dev: Self::f64_to_bytes(std_dev),
            sample_count: count as u64,
            range_start: metric.data_points.first().map_or(0, |dp| dp.timestamp),
            range_end: metric.data_points.last().map_or(0, |dp| dp.timestamp),
        }
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn register_metric(
        &self,
        name: &str,
        metric_type: MetricType,
        description: &str,
        unit: &str,
        tags: Vec<String>,
    ) -> Result<MetricId, ContextualError> {
        let mut metrics = self.metrics.lock().await;
        let mut metrics_by_name = self.metrics_by_name.lock().await;

        // Check if we've reached the maximum number of metrics
        if metrics.len() >= MAX_METRICS {
            return Err(self.error_context.create_error(
                format!("Maximum number of metrics ({}) reached", MAX_METRICS),
                ErrorMetadata::new(ErrorCategory::Resource),
            ));
        }

        // Check if metric already exists
        if metrics_by_name.contains_key(name) {
            return Err(self.error_context.create_error(
                format!("Metric with name '{}' already exists", name),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        // Create the metric
        let metric_id = MetricId::new();
        let now = self.current_timestamp();

        let metric = Metric {
            name: name.to_string(),
            metric_type,
            data_points: Vec::with_capacity(MAX_DATA_POINTS),
            created_at: now,
            updated_at: now,
            description: description.to_string(),
            unit: unit.to_string(),
            tags,
        };

        // Store the metric
        metrics.insert(metric_id.0, metric);
        metrics_by_name.insert(name.to_string(), metric_id.0);

        Ok(metric_id)
    }

    async fn record_data_point(
        &self,
        metric_id: &MetricId,
        value: f64,
        metadata: HashMap<String, String>,
    ) -> Result<(), ContextualError> {
        let mut metrics = self.metrics.lock().await;

        let metric = metrics.get_mut(&metric_id.0).ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric not found: {}", hex::encode(metric_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        // Create the data point
        let data_point = MetricDataPoint {
            timestamp: self.current_timestamp(),
            value: Self::f64_to_bytes(value),
            metadata,
        };

        // Add the data point
        metric.data_points.push(data_point);

        // Ensure we don't exceed the maximum number of data points
        while metric.data_points.len() > MAX_DATA_POINTS {
            metric.data_points.remove(0);
        }

        // Update the timestamp
        metric.updated_at = self.current_timestamp();

        Ok(())
    }

    async fn get_metric(
        &self,
        metric_id: &MetricId,
    ) -> Result<Metric, ContextualError> {
        let metrics = self.metrics.lock().await;

        metrics.get(&metric_id.0).cloned().ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric not found: {}", hex::encode(metric_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })
    }

    async fn find_metric_by_name(
        &self,
        name: &str,
    ) -> Result<(MetricId, Metric), ContextualError> {
        let metrics_by_name = self.metrics_by_name.lock().await;
        let metrics = self.metrics.lock().await;

        let id = metrics_by_name.get(name).ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric with name '{}' not found", name),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        let metric = metrics.get(id).ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric with ID '{}' not found", hex::encode(id)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        Ok((MetricId(*id), metric.clone()))
    }

    async fn list_metrics(
        &self,
    ) -> Result<Vec<(MetricId, String, MetricType)>, ContextualError> {
        let metrics = self.metrics.lock().await;

        let result = metrics
            .iter()
            .map(|(id, metric)| {
                (
                    MetricId(*id),
                    metric.name.clone(),
                    metric.metric_type.clone(),
                )
            })
            .collect();

        Ok(result)
    }

    async fn get_metric_stats(
        &self,
        metric_id: &MetricId,
    ) -> Result<MetricStats, ContextualError> {
        let metrics = self.metrics.lock().await;

        let metric = metrics.get(&metric_id.0).ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric not found: {}", hex::encode(metric_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        Ok(self.calculate_stats(metric))
    }

    async fn delete_metric(
        &self,
        metric_id: &MetricId,
    ) -> Result<(), ContextualError> {
        let mut metrics = self.metrics.lock().await;
        let mut metrics_by_name = self.metrics_by_name.lock().await;

        let metric = metrics.remove(&metric_id.0).ok_or_else(|| {
            self.error_context.create_error(
                format!("Metric not found: {}", hex::encode(metric_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        // Remove from metrics_by_name
        metrics_by_name.remove(&metric.name);

        Ok(())
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// MetricType
impl Encode for MetricType {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            MetricType::Counter => vec![0u8],
            MetricType::Gauge => vec![1u8],
            MetricType::Histogram => vec![2u8],
            MetricType::Timer => vec![3u8],
        }
    }
}

impl Decode for MetricType {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for MetricType".to_string() });
        }
        match bytes[0] {
            0 => Ok(MetricType::Counter),
            1 => Ok(MetricType::Gauge),
            2 => Ok(MetricType::Histogram),
            3 => Ok(MetricType::Timer),
            other => Err(DecodeError { message: format!("Invalid MetricType variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for MetricType {}

// MetricId
impl Encode for MetricId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for MetricId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 16 {
            return Err(DecodeError { message: format!("MetricId requires exactly 16 bytes, got {}", bytes.len()) });
        }
        let mut array = [0u8; 16];
        array.copy_from_slice(bytes);
        Ok(MetricId(array))
    }
}

impl SimpleSerialize for MetricId {}

// TrendDirection
impl Encode for TrendDirection {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            TrendDirection::Increasing => vec![0u8],
            TrendDirection::Decreasing => vec![1u8],
            TrendDirection::Flat => vec![2u8],
            TrendDirection::Volatile => vec![3u8],
        }
    }
}

impl Decode for TrendDirection {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for TrendDirection".to_string() });
        }
        match bytes[0] {
            0 => Ok(TrendDirection::Increasing),
            1 => Ok(TrendDirection::Decreasing),
            2 => Ok(TrendDirection::Flat),
            3 => Ok(TrendDirection::Volatile),
            other => Err(DecodeError { message: format!("Invalid TrendDirection variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for TrendDirection {}

// AnomalyType
impl Encode for AnomalyType {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            AnomalyType::Spike => vec![0u8],
            AnomalyType::Dip => vec![1u8],
            AnomalyType::LevelShift => vec![2u8],
            AnomalyType::TrendChange => vec![3u8],
            AnomalyType::MissingData => vec![4u8],
            AnomalyType::UnusualVariance => vec![5u8],
        }
    }
}

impl Decode for AnomalyType {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for AnomalyType".to_string() });
        }
        match bytes[0] {
            0 => Ok(AnomalyType::Spike),
            1 => Ok(AnomalyType::Dip),
            2 => Ok(AnomalyType::LevelShift),
            3 => Ok(AnomalyType::TrendChange),
            4 => Ok(AnomalyType::MissingData),
            5 => Ok(AnomalyType::UnusualVariance),
            other => Err(DecodeError { message: format!("Invalid AnomalyType variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for AnomalyType {}

// Complex types with proper field serialization
impl Encode for MetricDataPoint {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.timestamp.as_ssz_bytes());
        bytes.extend(self.value.as_ssz_bytes());
        bytes.extend(self.metadata.as_ssz_bytes());
        bytes
    }
}

impl Decode for MetricDataPoint {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
        offset += 8;
        
        let value = <[u8; 8]>::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode value: {}", e) })?;
        offset += 8;
        
        let metadata = HashMap::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode metadata: {}", e) })?;
        
        Ok(MetricDataPoint {
            timestamp,
            value,
            metadata,
        })
    }
}

impl SimpleSerialize for MetricDataPoint {}

impl Encode for Metric {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.name.as_ssz_bytes());
        bytes.extend(self.metric_type.as_ssz_bytes());
        
        // Serialize data points length and data
        bytes.extend((self.data_points.len() as u64).as_ssz_bytes());
        for point in &self.data_points {
            bytes.extend(point.as_ssz_bytes());
        }
        
        bytes.extend(self.created_at.as_ssz_bytes());
        bytes.extend(self.updated_at.as_ssz_bytes());
        bytes.extend(self.description.as_ssz_bytes());
        bytes.extend(self.unit.as_ssz_bytes());
        
        // Serialize tags length and data
        bytes.extend((self.tags.len() as u64).as_ssz_bytes());
        for tag in &self.tags {
            bytes.extend(tag.as_ssz_bytes());
        }
        
        bytes
    }
}

impl Decode for Metric {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode name
        let name = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode name: {}", e) })?;
        let name_size = name.as_ssz_bytes().len();
        offset += name_size;
        
        // Decode metric type
        let metric_type = MetricType::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode metric_type: {}", e) })?;
        offset += 1; // MetricType is 1 byte
        
        // Decode data points
        let data_points_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data_points length: {}", e) })? as usize;
        offset += 8;
        
        let mut data_points = Vec::with_capacity(data_points_len);
        for _ in 0..data_points_len {
            let point = MetricDataPoint::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode data point: {}", e) })?;
            let point_size = point.as_ssz_bytes().len();
            offset += point_size;
            data_points.push(point);
        }
        
        // Decode remaining fields
        let created_at = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode created_at: {}", e) })?;
        offset += 8;
        
        let updated_at = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode updated_at: {}", e) })?;
        offset += 8;
        
        let description = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode description: {}", e) })?;
        let description_size = description.as_ssz_bytes().len();
        offset += description_size;
        
        let unit = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode unit: {}", e) })?;
        let unit_size = unit.as_ssz_bytes().len();
        offset += unit_size;
        
        // Decode tags
        let tags_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode tags length: {}", e) })? as usize;
        offset += 8;
        
        let mut tags = Vec::with_capacity(tags_len);
        for _ in 0..tags_len {
            let tag = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode tag: {}", e) })?;
            let tag_size = tag.as_ssz_bytes().len();
            offset += tag_size;
            tags.push(tag);
        }
        
        Ok(Metric {
            name,
            metric_type,
            data_points,
            created_at,
            updated_at,
            description,
            unit,
            tags,
        })
    }
}

impl SimpleSerialize for Metric {}
