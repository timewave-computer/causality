use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::content_addressed::ContentAddressed;
use crate::verification::error::{VerificationError, VerificationResult};
use crate::verification::metrics::{VerificationMetric, VerificationMetricsCollector};
use crate::verification::trust::TrustBoundary;

/// Registry for verification operations
pub struct VerificationRegistry {
    /// The metrics collector
    metrics_collector: Arc<dyn VerificationMetricsCollector>,
    /// Success counter
    success_count: RwLock<u64>,
    /// Failure counter
    failure_count: RwLock<u64>,
}

impl VerificationRegistry {
    /// Creates a new verification registry
    pub fn new(metrics_collector: Arc<dyn VerificationMetricsCollector>) -> Self {
        Self {
            metrics_collector,
            success_count: RwLock::new(0),
            failure_count: RwLock::new(0),
        }
    }

    /// Gets a shared instance of the verification registry
    pub fn instance() -> Arc<Self> {
        // In a real implementation, this would use a proper singleton pattern
        // For simplicity, we create a new instance every time
        Arc::new(Self::new(Arc::new(
            crate::verification::metrics::DefaultVerificationMetricsCollector::new(),
        )))
    }

    /// Verifies a content-addressed object
    pub fn verify<T: ContentAddressed>(
        &self,
        object: &T,
        trust_boundary: TrustBoundary,
    ) -> Result<VerificationResult, VerificationError> {
        let start_time = Instant::now();

        // Compute the content hash
        let content_hash = object.content_hash().map_err(|e| VerificationError::HashError(e.to_string()))?;

        // Verify the hash
        let is_valid = object.verify_hash(&content_hash).map_err(|e| VerificationError::HashError(e.to_string()))?;

        // Record the result
        let duration = start_time.elapsed();
        let result = if is_valid {
            self.record_success();
            self.record_verification(
                content_hash.to_string(),
                &trust_boundary.to_string(),
                VerificationMetric::ContentVerification {
                    duration,
                    result: true,
                },
            );
            VerificationResult::verified()
        } else {
            self.record_failure();
            self.record_verification(
                content_hash.to_string(),
                &trust_boundary.to_string(),
                VerificationMetric::ContentVerification {
                    duration,
                    result: false,
                },
            );
            VerificationResult::failed(format!(
                "Hash verification failed for {}",
                content_hash
            ))
        };

        Ok(result)
    }

    /// Records a verification metric
    pub fn record_verification(&self, object_id: String, boundary: &str, metric: VerificationMetric) {
        self.metrics_collector.record_metric(&object_id, boundary, metric);
    }

    /// Records a verification success
    fn record_success(&self) {
        let mut count = self.success_count.write().unwrap();
        *count += 1;
    }

    /// Records a verification failure
    fn record_failure(&self) {
        let mut count = self.failure_count.write().unwrap();
        *count += 1;
    }

    /// Gets the success count
    pub fn success_count(&self) -> u64 {
        *self.success_count.read().unwrap()
    }

    /// Gets the failure count
    pub fn failure_count(&self) -> u64 {
        *self.failure_count.read().unwrap()
    }

    /// Gets the metrics collector
    pub fn metrics_collector(&self) -> &Arc<dyn VerificationMetricsCollector> {
        &self.metrics_collector
    }
} 