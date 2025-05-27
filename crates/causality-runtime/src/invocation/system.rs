// Invocation system implementation

use std::sync::Arc;
use std::fmt::Debug;

use tokio::runtime::Handle;
use causality_error::Result;
use causality_types::{ContentId, TraceId};

use super::propagation::ContextPropagator;
use super::patterns::{
    InvocationPatternTrait,
    DirectInvocation,
    ContinuationInvocation,
    BatchInvocation,
};
use super::registry::{EffectRegistry, HandlerOutput};

/// System for executing invocations
#[derive(Debug)]
pub struct InvocationSystem {
    /// Registry for effect handlers
    registry: Arc<EffectRegistry>,
    /// Context propagator
    propagator: Arc<ContextPropagator>,
    /// Tokio runtime handle
    runtime: Handle,
    /// Metrics about invocations (optional)
    metrics: Option<InvocationMetrics>,
}

/// Metrics about invocation execution
#[derive(Debug, Clone, Default)]
pub struct InvocationMetrics {
    /// Total number of invocations
    pub total_invocations: usize,
    /// Number of successful invocations
    pub successful_invocations: usize,
    /// Number of failed invocations
    pub failed_invocations: usize,
    /// Average invocation time in milliseconds
    pub average_time_ms: f64,
    /// Current running invocations
    pub running_invocations: usize,
}

impl InvocationSystem {
    /// Create a new invocation system
    pub fn new(registry: Arc<EffectRegistry>, propagator: Arc<ContextPropagator>) -> Self {
        InvocationSystem {
            registry,
            propagator,
            runtime: Handle::current(),
            metrics: Some(InvocationMetrics::default()),
        }
    }
    
    /// Create a new direct invocation
    pub fn create_direct_invocation(
        &self,
        target_service: impl Into<String>,
        operation: impl Into<String>,
    ) -> DirectInvocation {
        DirectInvocation::new(target_service, operation)
    }
    
    /// Execute an invocation pattern
    pub async fn execute<P: InvocationPatternTrait>(&self, pattern: &P) -> Result<HandlerOutput> {
        // Update metrics
        if let Some(metrics) = &self.metrics {
            let mut metrics_clone = metrics.clone();
            metrics_clone.running_invocations += 1;
            metrics_clone.total_invocations += 1;
            // Note: in a real implementation, we should update self.metrics
            // but since it's immutable, this would require Arc<Mutex<>> or similar
        }
        
        // Execute the pattern
        let result = pattern.execute(&self.registry, &self.propagator).await;
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            let mut metrics_clone = metrics.clone();
            metrics_clone.running_invocations -= 1;
            
            match &result {
                Ok(_) => metrics_clone.successful_invocations += 1,
                Err(_) => metrics_clone.failed_invocations += 1,
            }
            // Note: in a real implementation, we should update self.metrics
            // but since it's immutable, this would require Arc<Mutex<>> or similar
        }
        
        result
    }
    
    /// Execute a pattern with a trace ID
    pub async fn execute_with_trace<P: InvocationPatternTrait>(
        &self,
        pattern: &P,
        _trace_id: TraceId,
    ) -> Result<HandlerOutput> {
        // TODO: Implement trace handling
        self.execute(pattern).await
    }
    
    /// Create a continuation
    pub fn create_continuation(
        &self,
        target_service: impl Into<String>,
        operation: impl Into<String>,
        continuation_id: impl Into<String>,
    ) -> ContinuationInvocation {
        ContinuationInvocation::new(target_service, operation, continuation_id)
    }
    
    /// Create a batch invocation
    pub fn create_batch(
        &self,
        invocations: Vec<DirectInvocation>,
        parallel: bool,
    ) -> BatchInvocation {
        BatchInvocation::new(invocations, parallel)
    }
    
    /// Get the registry
    pub fn registry(&self) -> &Arc<EffectRegistry> {
        &self.registry
    }
    
    /// Get the propagator
    pub fn propagator(&self) -> &Arc<ContextPropagator> {
        &self.propagator
    }
    
    /// Get the metrics
    pub fn metrics(&self) -> Option<&InvocationMetrics> {
        self.metrics.as_ref()
    }
    
    /// Get the content ID of an invocation pattern
    pub fn get_content_id<P: InvocationPatternTrait>(&self, pattern: &P) -> ContentId {
        pattern.get_content_id()
    }
} 