//! Coprocessor Monitoring Interface
//!
//! This module provides interfaces and implementations for monitoring ZK coprocessors,
//! tracking health status, and collecting performance metrics. The monitoring system
//! uses caching to reduce overhead while maintaining up-to-date information.

//-----------------------------------------------------------------------------
// Imports and Dependencie
//-----------------------------------------------------------------------------

use async_trait::async_trait;
// Serialization imports removed as we don't use manual SSZ implementations here
// Removed unused import: std::sync::Arc

use super::types::CoprocessorId;
use crate::gateway::ApiError;

//-----------------------------------------------------------------------------
// Monitoring Type
//-----------------------------------------------------------------------------

/// Health status for a coprocessor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// The coprocessor is healthy and processing requests normally
    Healthy,

    /// The coprocessor is operating but with degraded performance
    Degraded,

    /// The coprocessor is unhealthy and not processing requests
    Unhealthy,

    /// The status of the coprocessor is unknown
    Unknown,
}

/// Performance metrics for a coprocessor
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,

    /// Average proof generation time in milliseconds
    pub avg_proof_time_ms: u64,

    /// Success rate (0-100)
    pub success_rate: u8,

    /// Number of pending requests
    pub pending_requests: u32,

    /// Number of in-progress requests
    pub in_progress_requests: u32,

    /// CPU utilization percentage (0-100)
    pub cpu_utilization: u8,

    /// Memory utilization percentage (0-100)
    pub memory_utilization: u8,
}

//-----------------------------------------------------------------------------
// Monitoring Interface
//-----------------------------------------------------------------------------

/// Monitoring interface for coprocessors
#[async_trait]
pub trait CoprocessorMonitor: Send + Sync {
    /// Get the health status of a coprocessor
    async fn get_health_status(
        &self,
        coprocessor_id: &CoprocessorId,
    ) -> Result<HealthStatus, ApiError>;

    /// Get performance metrics for a coprocessor
    async fn get_performance_metrics(
        &self,
        coprocessor_id: &CoprocessorId,
    ) -> Result<PerformanceMetrics, ApiError>;

    /// Check if a coprocessor is available for new requests
    async fn is_available(
        &self,
        coprocessor_id: &CoprocessorId,
    ) -> Result<bool, ApiError>;

    /// Get the estimated wait time for a new request in milliseconds
    async fn get_estimated_wait_time(
        &self,
        coprocessor_id: &CoprocessorId,
    ) -> Result<u64, ApiError>;
}

//-----------------------------------------------------------------------------
// Monitoring Implementation
//-----------------------------------------------------------------------------

/// Basic implementation of a coprocessor monitor
pub struct BasicCoprocessorMonitor;

impl Default for BasicCoprocessorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicCoprocessorMonitor {
    /// Create a new coprocessor monitor with default cache TTL (30 seconds)
    pub fn new() -> Self {
        Self
    }

    /// Create a new coprocessor monitor with a custom cache TTL
    pub fn with_cache_ttl(_ttl_seconds: u64) -> Self {
        Self
    }
}

#[async_trait]
impl CoprocessorMonitor for BasicCoprocessorMonitor {
    async fn get_health_status(
        &self,
        _coprocessor_id: &CoprocessorId,
    ) -> Result<HealthStatus, ApiError> {
        // For a basic monitor, we can assume the coprocessor is always healthy
        // or return Unknown if no information is available.
        Ok(HealthStatus::Unknown)
    }

    async fn get_performance_metrics(
        &self,
        _coprocessor_id: &CoprocessorId,
    ) -> Result<PerformanceMetrics, ApiError> {
        // Return default/dummy metrics for the basic monitor
        Ok(PerformanceMetrics {
            avg_response_time_ms: 0,
            avg_proof_time_ms: 0,
            success_rate: 100,
            pending_requests: 0,
            in_progress_requests: 0,
            cpu_utilization: 0,
            memory_utilization: 0,
        })
    }

    async fn is_available(
        &self,
        _coprocessor_id: &CoprocessorId,
    ) -> Result<bool, ApiError> {
        // Assume always available for the basic monitor
        Ok(true)
    }

    async fn get_estimated_wait_time(
        &self,
        _coprocessor_id: &CoprocessorId,
    ) -> Result<u64, ApiError> {
        // Assume no wait time for the basic monitor
        Ok(0)
    }
}
