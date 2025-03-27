use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

/// Metrics for verification operations
#[derive(Debug, Clone)]
pub enum VerificationMetric {
    /// Basic content verification
    ContentVerification {
        /// The time it took to verify
        duration: Duration,
        /// The result of verification
        result: bool,
    },
    
    /// Cross-domain verification
    CrossDomainVerification {
        /// The source domain
        source_domain: String,
        /// The target domain
        target_domain: String,
        /// The proof type used
        proof_type: String,
        /// The result of verification
        result: bool,
    },
    
    /// Batch verification
    BatchVerification {
        /// The number of objects verified
        count: usize,
        /// The time it took to verify
        duration: Duration,
        /// The number of successful verifications
        success_count: usize,
    },
    
    /// Custom verification metric
    Custom {
        /// The metric name
        name: String,
        /// The metric value
        value: String,
    },
}

impl Display for VerificationMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationMetric::ContentVerification { duration, result } => {
                write!(
                    f,
                    "ContentVerification(duration={}ms, result={})",
                    duration.as_millis(),
                    result
                )
            }
            VerificationMetric::CrossDomainVerification {
                source_domain,
                target_domain,
                proof_type,
                result,
            } => {
                write!(
                    f,
                    "CrossDomainVerification(source={}, target={}, proof={}, result={})",
                    source_domain, target_domain, proof_type, result
                )
            }
            VerificationMetric::BatchVerification {
                count,
                duration,
                success_count,
            } => {
                write!(
                    f,
                    "BatchVerification(count={}, duration={}ms, success={})",
                    count,
                    duration.as_millis(),
                    success_count
                )
            }
            VerificationMetric::Custom { name, value } => {
                write!(f, "Custom({}={})", name, value)
            }
        }
    }
}

/// Interface for collecting verification metrics
pub trait VerificationMetricsCollector: Send + Sync {
    /// Record a verification metric
    fn record_metric(&self, object_id: &str, boundary: &str, metric: VerificationMetric);
}

/// Default implementation of verification metrics collector
pub struct DefaultVerificationMetricsCollector;

impl VerificationMetricsCollector for DefaultVerificationMetricsCollector {
    fn record_metric(&self, object_id: &str, boundary: &str, metric: VerificationMetric) {
        // In a real implementation, this would log metrics to a monitoring system
        tracing::debug!(
            object_id = %object_id,
            boundary = %boundary,
            metric = %metric,
            "Verification metric recorded"
        );
    }
}

impl DefaultVerificationMetricsCollector {
    /// Create a new default verification metrics collector
    pub fn new() -> Self {
        Self
    }

    /// Create a new default verification metrics collector wrapped in an Arc
    pub fn new_arc() -> Arc<dyn VerificationMetricsCollector> {
        Arc::new(Self::new())
    }
} 