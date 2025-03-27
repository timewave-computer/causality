// Content verification system
//
// This module provides utilities for verifying content-addressed objects
// and tracking verification metrics.

// Re-export all contents through the module
pub use self::verification::*;

mod verification {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use std::collections::HashMap;
    use parking_lot::RwLock;
    use thiserror::Error;
    
    use crate::{ContentAddressed, ContentId, HashOutput};
    
    /// Error types for verification operations
    #[derive(Error, Debug)]
    pub enum VerificationError {
        /// Hash mismatch during verification
        #[error("Hash mismatch: expected {expected}, got {actual}")]
        HashMismatch {
            expected: String,
            actual: String,
        },
        
        /// Verification failed with a specific reason
        #[error("Verification failed: {0}")]
        VerificationFailed(String),
        
        /// Content hash computation failed
        #[error("Hash computation failed: {0}")]
        HashError(String),
        
        /// Content not found
        #[error("Content not found: {0}")]
        NotFound(String),
        
        /// Invalid content format
        #[error("Invalid content format: {0}")]
        InvalidFormat(String),
    }
    
    /// Result of a verification operation
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum VerificationResult {
        /// Verification succeeded
        Verified,
        
        /// Verification failed with a reason
        Failed {
            /// Reason for failure
            reason: String,
        },
    }
    
    impl VerificationResult {
        /// Check if the verification succeeded
        pub fn is_verified(&self) -> bool {
            matches!(self, VerificationResult::Verified)
        }
        
        /// Get the failure reason if verification failed
        pub fn failure_reason(&self) -> Option<&str> {
            match self {
                VerificationResult::Failed { reason } => Some(reason),
                _ => None,
            }
        }
    }
    
    /// A trust boundary where verification should occur
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum TrustBoundary {
        /// Content crossing storage boundaries
        Storage,
        
        /// Content crossing domain boundaries
        Domain,
        
        /// Content retrieved from external sources
        External,
        
        /// Content used for critical operations
        Critical,
        
        /// Content used for identity verification
        Identity,
        
        /// Content used for capability verification
        Capability,
        
        /// Custom trust boundary
        Custom(String),
    }
    
    impl std::fmt::Display for TrustBoundary {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TrustBoundary::Storage => write!(f, "storage"),
                TrustBoundary::Domain => write!(f, "domain"),
                TrustBoundary::External => write!(f, "external"),
                TrustBoundary::Critical => write!(f, "critical"),
                TrustBoundary::Identity => write!(f, "identity"),
                TrustBoundary::Capability => write!(f, "capability"),
                TrustBoundary::Custom(name) => write!(f, "custom:{}", name),
            }
        }
    }
    
    /// Verification metrics for content addressing
    #[derive(Debug, Clone)]
    pub struct VerificationMetrics {
        /// Total number of verification attempts
        pub total_verifications: u64,
        
        /// Number of successful verifications
        pub successful_verifications: u64,
        
        /// Number of failed verifications
        pub failed_verifications: u64,
        
        /// Average verification time in milliseconds
        pub avg_verification_time_ms: f64,
        
        /// Verification failures by boundary
        pub failures_by_boundary: HashMap<TrustBoundary, u64>,
    }
    
    /// Interface for collecting verification metrics
    pub trait VerificationMetricsCollector: Send + Sync {
        /// Record a verification attempt
        fn record_verification(
            &self,
            boundary: &TrustBoundary,
            result: &VerificationResult,
            duration: Duration,
        );
        
        /// Get the current metrics
        fn get_metrics(&self) -> VerificationMetrics;
        
        /// Reset the metrics
        fn reset(&self);
    }
    
    /// Default implementation of verification metrics collector
    #[derive(Debug)]
    pub struct DefaultVerificationMetricsCollector {
        /// Total verification attempts
        total_verifications: AtomicU64,
        
        /// Successful verification attempts
        successful_verifications: AtomicU64,
        
        /// Failed verification attempts
        failed_verifications: AtomicU64,
        
        /// Total verification time in milliseconds
        total_verification_time_ms: AtomicU64,
        
        /// Failures by boundary
        failures_by_boundary: RwLock<HashMap<TrustBoundary, u64>>,
    }
    
    impl DefaultVerificationMetricsCollector {
        /// Create a new metrics collector
        pub fn new() -> Self {
            Self {
                total_verifications: AtomicU64::new(0),
                successful_verifications: AtomicU64::new(0),
                failed_verifications: AtomicU64::new(0),
                total_verification_time_ms: AtomicU64::new(0),
                failures_by_boundary: RwLock::new(HashMap::new()),
            }
        }
    }
    
    impl Default for DefaultVerificationMetricsCollector {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl VerificationMetricsCollector for DefaultVerificationMetricsCollector {
        fn record_verification(
            &self,
            boundary: &TrustBoundary,
            result: &VerificationResult,
            duration: Duration,
        ) {
            // Increment total verifications
            self.total_verifications.fetch_add(1, Ordering::Relaxed);
            
            // Add verification time
            let duration_ms = duration.as_millis() as u64;
            self.total_verification_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
            
            // Record result
            match result {
                VerificationResult::Verified => {
                    self.successful_verifications.fetch_add(1, Ordering::Relaxed);
                }
                VerificationResult::Failed { .. } => {
                    self.failed_verifications.fetch_add(1, Ordering::Relaxed);
                    
                    // Record failure by boundary
                    let mut failures = self.failures_by_boundary.write();
                    *failures.entry(boundary.clone()).or_insert(0) += 1;
                }
            }
        }
        
        fn get_metrics(&self) -> VerificationMetrics {
            let total = self.total_verifications.load(Ordering::Relaxed);
            let successful = self.successful_verifications.load(Ordering::Relaxed);
            let failed = self.failed_verifications.load(Ordering::Relaxed);
            let total_time_ms = self.total_verification_time_ms.load(Ordering::Relaxed);
            
            let avg_time = if total > 0 {
                total_time_ms as f64 / total as f64
            } else {
                0.0
            };
            
            VerificationMetrics {
                total_verifications: total,
                successful_verifications: successful,
                failed_verifications: failed,
                avg_verification_time_ms: avg_time,
                failures_by_boundary: self.failures_by_boundary.read().clone(),
            }
        }
        
        fn reset(&self) {
            self.total_verifications.store(0, Ordering::Relaxed);
            self.successful_verifications.store(0, Ordering::Relaxed);
            self.failed_verifications.store(0, Ordering::Relaxed);
            self.total_verification_time_ms.store(0, Ordering::Relaxed);
            self.failures_by_boundary.write().clear();
        }
    }
    
    /// Global singleton for verification metrics
    pub struct VerificationRegistry {
        /// Metrics collector
        metrics_collector: Arc<dyn VerificationMetricsCollector>,
    }
    
    impl VerificationRegistry {
        /// Create a new verification registry
        pub fn new(metrics_collector: Arc<dyn VerificationMetricsCollector>) -> Self {
            Self { metrics_collector }
        }
        
        /// Get the default registry instance
        pub fn instance() -> Arc<Self> {
            // This would normally use lazy_static, but we're keeping it simple
            static mut INSTANCE: Option<Arc<VerificationRegistry>> = None;
            static INIT: std::sync::Once = std::sync::Once::new();
            
            unsafe {
                INIT.call_once(|| {
                    let collector = Arc::new(DefaultVerificationMetricsCollector::new());
                    INSTANCE = Some(Arc::new(VerificationRegistry::new(collector)));
                });
                
                INSTANCE.clone().unwrap()
            }
        }
        
        /// Verify a content-addressed object
        pub fn verify<T: ContentAddressed>(
            &self,
            object: &T,
            boundary: TrustBoundary,
        ) -> Result<VerificationResult, VerificationError> {
            let start_time = Instant::now();
            
            // Attempt to compute the content hash
            let hash_result = object.content_hash().map_err(|e| {
                VerificationError::HashError(e.to_string())
            });
            
            // Process the result
            let result = match hash_result {
                Ok(hash) => {
                    // Check if the hash is computed correctly
                    match object.verify(&hash) {
                        Ok(true) => VerificationResult::Verified,
                        Ok(false) => VerificationResult::Failed {
                            reason: "Hash verification failed".to_string(),
                        },
                        Err(e) => VerificationResult::Failed {
                            reason: format!("Verification error: {}", e),
                        },
                    }
                }
                Err(e) => VerificationResult::Failed {
                    reason: format!("Hash computation failed: {}", e),
                },
            };
            
            // Record metrics
            let duration = start_time.elapsed();
            self.metrics_collector.record_verification(&boundary, &result, duration);
            
            Ok(result)
        }
        
        /// Verify content hash against expected hash
        pub fn verify_hash<T: ContentAddressed>(
            &self,
            object: &T,
            expected_hash: &HashOutput,
            boundary: TrustBoundary,
        ) -> Result<VerificationResult, VerificationError> {
            let start_time = Instant::now();
            
            // Compute the content hash
            let actual_hash = object.content_hash().map_err(|e| {
                VerificationError::HashError(e.to_string())
            })?;
            
            // Compare hashes
            let result = if &actual_hash == expected_hash {
                VerificationResult::Verified
            } else {
                VerificationResult::Failed {
                    reason: format!(
                        "Hash mismatch: expected {}, got {}",
                        expected_hash, actual_hash
                    ),
                }
            };
            
            // Record metrics
            let duration = start_time.elapsed();
            self.metrics_collector.record_verification(&boundary, &result, duration);
            
            Ok(result)
        }
        
        /// Get the current verification metrics
        pub fn get_metrics(&self) -> VerificationMetrics {
            self.metrics_collector.get_metrics()
        }
        
        /// Reset the metrics
        pub fn reset_metrics(&self) {
            self.metrics_collector.reset();
        }
    }
    
    /// Explicit verification point for trust boundaries
    pub struct VerificationPoint<T: ContentAddressed> {
        /// The object to verify
        object: T,
        
        /// The trust boundary
        boundary: TrustBoundary,
        
        /// The verification registry
        registry: Arc<VerificationRegistry>,
    }
    
    impl<T: ContentAddressed> VerificationPoint<T> {
        /// Create a new verification point
        pub fn new(object: T, boundary: TrustBoundary) -> Self {
            Self {
                object,
                boundary,
                registry: VerificationRegistry::instance(),
            }
        }
        
        /// Create a new verification point with a custom registry
        pub fn with_registry(
            object: T,
            boundary: TrustBoundary,
            registry: Arc<VerificationRegistry>,
        ) -> Self {
            Self {
                object,
                boundary,
                registry,
            }
        }
        
        /// Verify the object
        pub fn verify(&self) -> Result<VerificationResult, VerificationError> {
            self.registry.verify(&self.object, self.boundary.clone())
        }
        
        /// Verify against an expected hash
        pub fn verify_hash(&self, expected_hash: &HashOutput) -> Result<VerificationResult, VerificationError> {
            self.registry.verify_hash(&self.object, expected_hash, self.boundary.clone())
        }
        
        /// Get the object after verification
        pub fn into_verified(self) -> Result<T, VerificationError> {
            match self.verify()? {
                VerificationResult::Verified => Ok(self.object),
                VerificationResult::Failed { reason } => {
                    Err(VerificationError::VerificationFailed(reason))
                }
            }
        }
        
        /// Get a reference to the object
        pub fn get(&self) -> &T {
            &self.object
        }
    }
    
    /// Helper trait for creating verification points
    pub trait Verifiable: ContentAddressed + Sized {
        /// Create a verification point at a specific trust boundary
        fn at_boundary(self, boundary: TrustBoundary) -> VerificationPoint<Self> {
            VerificationPoint::new(self, boundary)
        }
        
        /// Create a verification point at the storage boundary
        fn at_storage_boundary(self) -> VerificationPoint<Self> {
            VerificationPoint::new(self, TrustBoundary::Storage)
        }
        
        /// Create a verification point at the domain boundary
        fn at_domain_boundary(self) -> VerificationPoint<Self> {
            VerificationPoint::new(self, TrustBoundary::Domain)
        }
        
        /// Create a verification point for external content
        fn from_external(self) -> VerificationPoint<Self> {
            VerificationPoint::new(self, TrustBoundary::External)
        }
        
        /// Create a verification point for critical operations
        fn for_critical_operation(self) -> VerificationPoint<Self> {
            VerificationPoint::new(self, TrustBoundary::Critical)
        }
    }
    
    impl<T: ContentAddressed> Verifiable for T {}
    
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::HashError;
        
        // Test implementation of ContentAddressed
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct TestObject {
            id: u64,
            data: Vec<u8>,
            should_fail: bool,
        }
        
        impl ContentAddressed for TestObject {
            fn content_hash(&self) -> Result<HashOutput, HashError> {
                if self.should_fail {
                    return Err(HashError::ComputationError("Intentional failure".to_string()));
                }
                
                // Simple mock hash
                let mut hash_data = Vec::with_capacity(8 + self.data.len());
                hash_data.extend_from_slice(&self.id.to_le_bytes());
                hash_data.extend_from_slice(&self.data);
                
                Ok(HashOutput::new("TEST".to_string(), hash_data))
            }
            
            fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
                let mut bytes = Vec::with_capacity(8 + self.data.len());
                bytes.extend_from_slice(&self.id.to_le_bytes());
                bytes.extend_from_slice(&self.data);
                Ok(bytes)
            }
            
            fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
                if bytes.len() < 8 {
                    return Err(HashError::DeserializationError("Not enough data".to_string()));
                }
                
                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&bytes[0..8]);
                let id = u64::from_le_bytes(id_bytes);
                
                let data = bytes[8..].to_vec();
                
                Ok(Self {
                    id,
                    data,
                    should_fail: false,
                })
            }
        }
        
        #[test]
        fn test_verification_success() {
            let obj = TestObject {
                id: 1,
                data: vec![1, 2, 3, 4, 5],
                should_fail: false,
            };
            
            let registry = VerificationRegistry::instance();
            let result = registry.verify(&obj, TrustBoundary::Storage).unwrap();
            
            assert!(result.is_verified());
        }
        
        #[test]
        fn test_verification_failure() {
            let obj = TestObject {
                id: 1,
                data: vec![1, 2, 3, 4, 5],
                should_fail: true,
            };
            
            let registry = VerificationRegistry::instance();
            let result = registry.verify(&obj, TrustBoundary::Storage).unwrap();
            
            assert!(!result.is_verified());
            assert!(result.failure_reason().is_some());
        }
        
        #[test]
        fn test_verification_metrics() {
            let collector = Arc::new(DefaultVerificationMetricsCollector::new());
            let registry = Arc::new(VerificationRegistry::new(collector.clone()));
            
            // Successful verification
            let success_obj = TestObject {
                id: 1,
                data: vec![1, 2, 3, 4, 5],
                should_fail: false,
            };
            
            let _ = registry.verify(&success_obj, TrustBoundary::Storage);
            
            // Failed verification
            let fail_obj = TestObject {
                id: 2,
                data: vec![1, 2, 3, 4, 5],
                should_fail: true,
            };
            
            let _ = registry.verify(&fail_obj, TrustBoundary::External);
            
            // Get metrics
            let metrics = registry.get_metrics();
            
            assert_eq!(metrics.total_verifications, 2);
            assert_eq!(metrics.successful_verifications, 1);
            assert_eq!(metrics.failed_verifications, 1);
            assert!(metrics.avg_verification_time_ms >= 0.0);
            assert_eq!(metrics.failures_by_boundary.len(), 1);
            assert_eq!(*metrics.failures_by_boundary.get(&TrustBoundary::External).unwrap(), 1);
        }
        
        #[test]
        fn test_verification_point() {
            let obj = TestObject {
                id: 1,
                data: vec![1, 2, 3, 4, 5],
                should_fail: false,
            };
            
            let verification_point = obj.at_storage_boundary();
            let result = verification_point.verify().unwrap();
            
            assert!(result.is_verified());
            
            // Test into_verified
            let obj = TestObject {
                id: 2,
                data: vec![5, 4, 3, 2, 1],
                should_fail: false,
            };
            
            let verification_point = obj.at_domain_boundary();
            let verified_obj = verification_point.into_verified().unwrap();
            
            assert_eq!(verified_obj.id, 2);
        }
    }
} 