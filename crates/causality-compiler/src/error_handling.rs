use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
/// Enhanced error handling for Causality compiler operations
///
/// This module provides comprehensive error handling and recovery mechanisms
/// for the Causality compiler, including storage errors, network failures,
/// and validation issues.
use std::fmt;

/// Comprehensive error type for Causality compiler operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausalityError {
    /// Storage-related errors
    Storage {
        message: String,
        details: Option<String>,
        recoverable: bool,
    },

    /// Network-related errors
    Network {
        message: String,
        endpoint: Option<String>,
        status_code: Option<u16>,
        retry_count: u32,
    },

    /// Compilation errors
    Compilation {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        source_context: Option<String>,
    },

    /// Serialization/deserialization errors
    Serialization {
        message: String,
        format: String,
        data_type: Option<String>,
    },

    /// Validation errors
    Validation {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Configuration errors
    Configuration {
        message: String,
        config_key: Option<String>,
        config_file: Option<String>,
    },

    /// Resource exhaustion errors
    ResourceExhaustion {
        message: String,
        resource_type: String,
        current_usage: Option<u64>,
        limit: Option<u64>,
    },

    /// Permission/authorization errors
    Permission {
        message: String,
        required_permission: Option<String>,
        current_role: Option<String>,
    },

    /// Timeout errors
    Timeout {
        message: String,
        operation: String,
        duration_ms: u64,
        timeout_ms: u64,
    },

    /// Generic errors with structured context
    Generic {
        message: String,
        error_code: Option<String>,
        context: std::collections::HashMap<String, String>,
    },
}

impl fmt::Display for CausalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausalityError::Storage {
                message,
                details,
                recoverable,
            } => {
                write!(f, "Storage error: {}", message)?;
                if let Some(details) = details {
                    write!(f, " ({})", details)?;
                }
                if *recoverable {
                    write!(f, " [recoverable]")?;
                }
                Ok(())
            }
            CausalityError::Network {
                message,
                endpoint,
                status_code,
                retry_count,
            } => {
                write!(f, "Network error: {}", message)?;
                if let Some(endpoint) = endpoint {
                    write!(f, " (endpoint: {})", endpoint)?;
                }
                if let Some(code) = status_code {
                    write!(f, " (status: {})", code)?;
                }
                if *retry_count > 0 {
                    write!(f, " (retries: {})", retry_count)?;
                }
                Ok(())
            }
            CausalityError::Compilation {
                message,
                line,
                column,
                source_context,
            } => {
                write!(f, "Compilation error: {}", message)?;
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, " at line {}, column {}", line, column)?;
                }
                if let Some(context) = source_context {
                    write!(f, "\nSource context: {}", context)?;
                }
                Ok(())
            }
            CausalityError::Serialization {
                message,
                format,
                data_type,
            } => {
                write!(f, "Serialization error ({}): {}", format, message)?;
                if let Some(data_type) = data_type {
                    write!(f, " for type {}", data_type)?;
                }
                Ok(())
            }
            CausalityError::Validation {
                message,
                field,
                expected,
                actual,
            } => {
                write!(f, "Validation error: {}", message)?;
                if let Some(field) = field {
                    write!(f, " (field: {})", field)?;
                }
                if let (Some(expected), Some(actual)) = (expected, actual) {
                    write!(f, " (expected: {}, actual: {})", expected, actual)?;
                }
                Ok(())
            }
            CausalityError::Configuration {
                message,
                config_key,
                config_file,
            } => {
                write!(f, "Configuration error: {}", message)?;
                if let Some(key) = config_key {
                    write!(f, " (key: {})", key)?;
                }
                if let Some(file) = config_file {
                    write!(f, " (file: {})", file)?;
                }
                Ok(())
            }
            CausalityError::ResourceExhaustion {
                message,
                resource_type,
                current_usage,
                limit,
            } => {
                write!(f, "Resource exhaustion ({}): {}", resource_type, message)?;
                if let (Some(usage), Some(limit)) = (current_usage, limit) {
                    write!(f, " ({}/{} used)", usage, limit)?;
                }
                Ok(())
            }
            CausalityError::Permission {
                message,
                required_permission,
                current_role,
            } => {
                write!(f, "Permission error: {}", message)?;
                if let Some(permission) = required_permission {
                    write!(f, " (required: {})", permission)?;
                }
                if let Some(role) = current_role {
                    write!(f, " (current role: {})", role)?;
                }
                Ok(())
            }
            CausalityError::Timeout {
                message,
                operation,
                duration_ms,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Timeout error in {}: {} ({}ms > {}ms)",
                    operation, message, duration_ms, timeout_ms
                )
            }
            CausalityError::Generic {
                message,
                error_code,
                context,
            } => {
                write!(f, "Error: {}", message)?;
                if let Some(code) = error_code {
                    write!(f, " (code: {})", code)?;
                }
                if !context.is_empty() {
                    write!(f, " (context: {:?})", context)?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for CausalityError {}

/// Error context for better debugging and tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub component: String,
    pub trace_id: Option<String>,
    pub user_id: Option<String>,
    pub additional_data: std::collections::BTreeMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            operation: operation.to_string(),
            component: component.to_string(),
            trace_id: None,
            user_id: None,
            additional_data: std::collections::BTreeMap::new(),
        }
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.additional_data
            .insert(key.to_string(), value.to_string());
        self
    }
}

/// Enhanced error with context for better observability
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: CausalityError,
    pub context: ErrorContext,
    pub chain: Vec<CausalityError>,
}

impl ContextualError {
    pub fn new(error: CausalityError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            chain: Vec::new(),
        }
    }

    pub fn with_cause(mut self, cause: CausalityError) -> Self {
        self.chain.push(cause);
        self
    }

    pub fn add_context_data(&mut self, key: &str, value: &str) {
        self.context
            .additional_data
            .insert(key.to_string(), value.to_string());
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}] {}",
            self.context.component, self.context.operation, self.error
        )?;

        if !self.chain.is_empty() {
            write!(f, " (caused by: ")?;
            for (i, cause) in self.chain.iter().enumerate() {
                if i > 0 {
                    write!(f, " -> ")?;
                }
                write!(f, "{}", cause)?;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl std::error::Error for ContextualError {}

/// Conversion from common error types
impl From<serde_json::Error> for CausalityError {
    fn from(error: serde_json::Error) -> Self {
        CausalityError::Serialization {
            message: error.to_string(),
            format: "json".to_string(),
            data_type: None,
        }
    }
}

impl From<reqwest::Error> for CausalityError {
    fn from(error: reqwest::Error) -> Self {
        CausalityError::Network {
            message: error.to_string(),
            endpoint: error.url().map(|u| u.to_string()),
            status_code: error.status().map(|s| s.as_u16()),
            retry_count: 0,
        }
    }
}

impl From<tokio::time::error::Elapsed> for CausalityError {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        CausalityError::Timeout {
            message: error.to_string(),
            operation: "unknown".to_string(),
            duration_ms: 0,
            timeout_ms: 0,
        }
    }
}

/// Result type alias for convenience
pub type CausalityResult<T> = Result<T, CausalityError>;
pub type ContextualResult<T> = Result<T, ContextualError>;

/// Error handling utilities
pub struct ErrorHandler {
    component: String,
    enable_logging: bool,
    enable_metrics: bool,
}

impl ErrorHandler {
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
            enable_logging: true,
            enable_metrics: true,
        }
    }

    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// Handle an error with full context and observability
    pub fn handle_error(
        &self,
        error: CausalityError,
        operation: &str,
    ) -> ContextualError {
        let context = ErrorContext::new(operation, &self.component);
        let contextual_error = ContextualError::new(error, context);

        if self.enable_logging {
            self.log_error(&contextual_error);
        }

        if self.enable_metrics {
            self.record_error_metric(&contextual_error);
        }

        contextual_error
    }

    /// Handle an error with additional context data
    pub fn handle_error_with_context(
        &self,
        error: CausalityError,
        operation: &str,
        context_data: std::collections::BTreeMap<String, String>,
    ) -> ContextualError {
        let mut context = ErrorContext::new(operation, &self.component);
        context.additional_data = context_data;

        let contextual_error = ContextualError::new(error, context);

        if self.enable_logging {
            self.log_error(&contextual_error);
        }

        if self.enable_metrics {
            self.record_error_metric(&contextual_error);
        }

        contextual_error
    }

    /// Log error with structured logging
    fn log_error(&self, error: &ContextualError) {
        log::error!(
            target: &self.component,
            "Error in {}: {} | Context: {:?}",
            error.context.operation,
            error.error,
            error.context.additional_data
        );

        // Log error chain if present
        for (i, cause) in error.chain.iter().enumerate() {
            log::error!(
                target: &self.component,
                "  Cause {}: {}",
                i + 1,
                cause
            );
        }
    }

    /// Record error metrics (placeholder for actual metrics implementation)
    fn record_error_metric(&self, error: &ContextualError) {
        // In a real implementation, this would integrate with metrics systems like Prometheus
        log::debug!(
            target: "metrics",
            "error_count{{component=\"{}\",operation=\"{}\",error_type=\"{}\"}} 1",
            self.component,
            error.context.operation,
            error.error.error_type()
        );
    }
}

/// Extension trait for CausalityError to get error type for metrics
impl CausalityError {
    pub fn error_type(&self) -> &'static str {
        match self {
            CausalityError::Storage { .. } => "storage",
            CausalityError::Network { .. } => "network",
            CausalityError::Compilation { .. } => "compilation",
            CausalityError::Serialization { .. } => "serialization",
            CausalityError::Validation { .. } => "validation",
            CausalityError::Configuration { .. } => "configuration",
            CausalityError::ResourceExhaustion { .. } => "resource_exhaustion",
            CausalityError::Permission { .. } => "permission",
            CausalityError::Timeout { .. } => "timeout",
            CausalityError::Generic { .. } => "generic",
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            CausalityError::Storage { recoverable, .. } => *recoverable,
            CausalityError::Network { .. } | CausalityError::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Check if error should trigger circuit breaker
    pub fn should_break_circuit(&self) -> bool {
        matches!(
            self,
            CausalityError::Storage { .. }
                | CausalityError::Network { .. }
                | CausalityError::Configuration { .. }
                | CausalityError::ResourceExhaustion { .. }
                | CausalityError::Permission { .. }
        )
    }

    /// Check if error should trigger an alert
    pub fn should_alert(&self) -> bool {
        matches!(
            self,
            CausalityError::Storage { .. }
                | CausalityError::Network { .. }
                | CausalityError::Configuration { .. }
                | CausalityError::ResourceExhaustion { .. }
                | CausalityError::Permission { .. }
        )
    }
}

/// Retry logic with exponential backoff
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a fallible operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    error_handler: &ErrorHandler,
    operation_name: &str,
) -> ContextualResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = CausalityResult<T>>,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error.clone());

                if !error.is_retryable() || attempt == config.max_attempts {
                    break;
                }

                log::warn!(
                    "Attempt {}/{} failed for {}: {}. Retrying in {}ms",
                    attempt,
                    config.max_attempts,
                    operation_name,
                    error,
                    delay
                );

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay = std::cmp::min(
                    (delay as f64 * config.backoff_multiplier) as u64,
                    config.max_delay_ms,
                );
            }
        }
    }

    let final_error = last_error.unwrap_or_else(|| CausalityError::Generic {
        message: "Retry operation failed without error".to_string(),
        error_code: None,
        context: std::collections::HashMap::new(),
    });

    Err(error_handler.handle_error(final_error, operation_name))
}

/// Health check utilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: std::collections::BTreeMap<String, String>,
    pub dependencies: Vec<DependencyHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub is_healthy: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

impl HealthStatus {
    pub fn new(component: &str) -> Self {
        Self {
            is_healthy: true,
            component: component.to_string(),
            timestamp: chrono::Utc::now(),
            details: std::collections::BTreeMap::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, dependency: DependencyHealth) {
        if !dependency.is_healthy {
            self.is_healthy = false;
        }
        self.dependencies.push(dependency);
    }

    pub fn add_detail(&mut self, key: &str, value: &str) {
        self.details.insert(key.to_string(), value.to_string());
    }
}

/// Health checker for external dependencies
pub struct HealthChecker {
    component: String,
    timeout: chrono::Duration,
}

impl HealthChecker {
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
            timeout: chrono::Duration::try_seconds(5).unwrap(),
        }
    }

    pub fn with_timeout(mut self, timeout: chrono::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check health of an HTTP endpoint
    pub async fn check_http_endpoint(
        &self,
        name: &str,
        url: &str,
    ) -> DependencyHealth {
        let start_time = std::time::Instant::now();

        match tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout.num_milliseconds() as u64),
            reqwest::get(url),
        )
        .await
        {
            Ok(Ok(response)) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                DependencyHealth {
                    name: name.to_string(),
                    is_healthy: response.status().is_success(),
                    response_time_ms: Some(response_time),
                    error: if response.status().is_success() {
                        None
                    } else {
                        Some(format!("HTTP {}", response.status()))
                    },
                }
            }
            Ok(Err(e)) => DependencyHealth {
                name: name.to_string(),
                is_healthy: false,
                response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                error: Some(e.to_string()),
            },
            Err(_) => DependencyHealth {
                name: name.to_string(),
                is_healthy: false,
                response_time_ms: Some(self.timeout.num_milliseconds() as u64),
                error: Some("Timeout".to_string()),
            },
        }
    }

    /// Perform comprehensive health check
    pub async fn comprehensive_health_check(&self) -> HealthStatus {
        let mut health_status = HealthStatus::new(&self.component);

        // Check Almanac
        let almanac_health = self
            .check_http_endpoint("almanac", "http://localhost:8080/health")
            .await;
        health_status.add_dependency(almanac_health);

        // Check Valence
        let valence_health = self
            .check_http_endpoint("valence", "http://localhost:9090/health")
            .await;
        health_status.add_dependency(valence_health);

        // Add system details
        health_status.add_detail("version", env!("CARGO_PKG_VERSION"));
        health_status.add_detail("build_time", "unknown");

        health_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CausalityError::Storage {
            message: "Test storage error".to_string(),
            details: None,
            recoverable: false,
        };

        assert_eq!(error.error_type(), "storage");
        assert!(!error.is_retryable());
        assert!(error.should_alert());
    }

    #[test]
    fn test_contextual_error() {
        let error = CausalityError::Network {
            message: "Connection failed".to_string(),
            endpoint: Some("http://localhost:8080".to_string()),
            status_code: Some(500),
            retry_count: 1,
        };

        let context = ErrorContext::new("test_operation", "test_component");
        let contextual_error = ContextualError::new(error, context);

        assert_eq!(contextual_error.context.operation, "test_operation");
        assert_eq!(contextual_error.context.component, "test_component");
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let error_handler = ErrorHandler::new("test");

        // Use Arc<Mutex<>> to share mutable state across closure calls
        let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = retry_with_backoff(
            move || {
                let attempt_count = attempt_count_clone.clone();
                async move {
                    let mut count = attempt_count.lock().unwrap();
                    *count += 1;
                    let current_attempt = *count;
                    drop(count); // Release the lock

                    if current_attempt < 2 {
                        Err(CausalityError::Network {
                            message: "Temporary failure".to_string(),
                            endpoint: None,
                            status_code: None,
                            retry_count: current_attempt,
                        })
                    } else {
                        Ok("Success")
                    }
                }
            },
            config,
            &error_handler,
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(*attempt_count.lock().unwrap(), 2);
    }
}
