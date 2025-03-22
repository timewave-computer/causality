// Invocation module
//
// This module provides the invocation system for Causality, which is responsible
// for managing the execution of effects across domains and programs.
// All invocations are content-addressed for traceability and verification.

pub mod context;
pub mod registry;
pub mod patterns;

use std::sync::Arc;

use crate::error::Result;
use crate::types::{ContentId, ContentHash, TraceId};
use crate::domain::map::map::TimeMap;

use self::context::propagation::{ContextPropagator, ContextStorage};
use self::registry::{EffectRegistry, HandlerOutput, HandlerRegistration, HandlerInput};
use self::patterns::{
    InvocationPattern,
    DirectInvocation,
    CallbackInvocation,
    ContinuationInvocation,
    PromiseInvocation,
    StreamingInvocation,
    BatchInvocation,
};

// Re-export core types
pub use self::context::InvocationContext;

/// Invocation system for executing effects
#[derive(Debug)]
pub struct InvocationSystem {
    /// Registry of available effect handlers
    registry: Arc<EffectRegistry>,
    /// Context propagator for managing invocation contexts
    propagator: Arc<ContextPropagator>,
}

impl InvocationSystem {
    /// Create a new invocation system
    pub fn new() -> Self {
        let registry = Arc::new(EffectRegistry::new());
        let storage = Arc::new(ContextStorage::new());
        let propagator = Arc::new(ContextPropagator::new(storage));
        
        InvocationSystem {
            registry,
            propagator,
        }
    }
    
    /// Create a new invocation system with the given registry and propagator
    pub fn with_components(
        registry: Arc<EffectRegistry>,
        propagator: Arc<ContextPropagator>,
    ) -> Self {
        InvocationSystem {
            registry,
            propagator,
        }
    }
    
    /// Get a reference to the effect registry
    pub fn registry(&self) -> &Arc<EffectRegistry> {
        &self.registry
    }
    
    /// Get a reference to the context propagator
    pub fn propagator(&self) -> &Arc<ContextPropagator> {
        &self.propagator
    }
    
    /// Execute an invocation pattern
    pub async fn execute<P: InvocationPattern>(&self, pattern: &P) -> Result<HandlerOutput> {
        pattern.execute(&self.registry, &self.propagator).await
    }
    
    /// Create and execute a direct invocation
    pub async fn invoke(
        &self,
        handler_id: impl Into<String>,
        action: impl Into<String>,
        params: serde_json::Value,
    ) -> Result<HandlerOutput> {
        let invocation = DirectInvocation::new(handler_id, action, params);
        self.execute(&invocation).await
    }
    
    /// Create and execute a direct invocation with a trace ID
    pub async fn invoke_with_trace(
        &self,
        handler_id: impl Into<String>,
        action: impl Into<String>,
        params: serde_json::Value,
        trace_id: TraceId,
    ) -> Result<HandlerOutput> {
        let invocation = DirectInvocation::new(handler_id, action, params)
            .with_trace(trace_id);
        self.execute(&invocation).await
    }
    
    /// Create and execute a batch of invocations
    pub async fn invoke_batch(
        &self,
        invocations: Vec<DirectInvocation>,
        parallel: bool,
    ) -> Result<Vec<Result<HandlerOutput>>> {
        let batch = BatchInvocation::new(invocations, parallel);
        batch.execute_batch(&self.registry, &self.propagator).await
    }
    
    /// Create a continuation invocation and execute it with a continuation
    pub async fn invoke_with_continuation<T, F>(
        &self,
        invocation: DirectInvocation,
        continuation: F,
    ) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Result<HandlerOutput>) -> Result<T> + Send + 'static,
    {
        let continuation_invocation = ContinuationInvocation::new(invocation, continuation);
        continuation_invocation.execute_with_continuation(&self.registry, &self.propagator).await
    }
    
    /// Get a content ID from an invocation
    pub fn get_content_id<P: InvocationPattern>(&self, pattern: &P) -> ContentId {
        pattern.get_content_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainId;
    use crate::invocation::registry::{EffectHandler, HandlerInput, HandlerRegistration};
    use async_trait::async_trait;
    
    struct TestHandler {
        registration: HandlerRegistration,
    }
    
    impl TestHandler {
        fn new(id: &str) -> Self {
            let domain = DomainId::new("test-domain");
            
            TestHandler {
                registration: HandlerRegistration::new(
                    id,
                    "Test Handler",
                    "A test handler for unit tests",
                    domain,
                ),
            }
        }
    }
    
    #[async_trait]
    impl EffectHandler for TestHandler {
        fn get_registration(&self) -> HandlerRegistration {
            self.registration.clone()
        }
        
        async fn handle(&self, input: HandlerInput) -> Result<HandlerOutput> {
            // Just echo back the input
            Ok(HandlerOutput::new(input.params))
        }
    }
    
    #[tokio::test]
    async fn test_invocation_system() -> Result<()> {
        // Create a new invocation system
        let system = InvocationSystem::new();
        
        // Register a test handler
        let handler = Arc::new(TestHandler::new("test-handler"));
        system.registry().register_handler(handler)?;
        
        // Invoke the handler
        let result = system.invoke(
            "test-handler",
            "test-action",
            serde_json::json!({ "test": "value" }),
        ).await?;
        
        // Check the result
        assert_eq!(result.data["test"], "value");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_continuation() -> Result<()> {
        // Create a new invocation system
        let system = InvocationSystem::new();
        
        // Register a test handler
        let handler = Arc::new(TestHandler::new("test-handler"));
        system.registry().register_handler(handler)?;
        
        // Create a direct invocation
        let invocation = DirectInvocation::new(
            "test-handler",
            "test-action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Invoke with a continuation
        let result = system.invoke_with_continuation(
            invocation,
            |result| {
                let result = result?;
                let value = result.data["value"].as_i64().unwrap() as i32;
                Ok(value * 2) // Double the value
            },
        ).await?;
        
        // Check the result
        assert_eq!(result, 84); // 42 * 2
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_batch_invocation() -> Result<()> {
        // Create a new invocation system
        let system = InvocationSystem::new();
        
        // Register a test handler
        let handler = Arc::new(TestHandler::new("test-handler"));
        system.registry().register_handler(handler)?;
        
        // Create a batch of invocations
        let invocations = vec![
            DirectInvocation::new(
                "test-handler",
                "action-1",
                serde_json::json!({ "value": 1 }),
            ),
            DirectInvocation::new(
                "test-handler",
                "action-2",
                serde_json::json!({ "value": 2 }),
            ),
            DirectInvocation::new(
                "test-handler",
                "action-3",
                serde_json::json!({ "value": 3 }),
            ),
        ];
        
        // Invoke the batch sequentially
        let results = system.invoke_batch(invocations.clone(), false).await?;
        
        // Check the results
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap().data["value"], 1);
        assert_eq!(results[1].as_ref().unwrap().data["value"], 2);
        assert_eq!(results[2].as_ref().unwrap().data["value"], 3);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_content_addressing() -> Result<()> {
        // Create a new invocation system
        let system = InvocationSystem::new();
        
        // Create two identical invocations
        let invocation1 = DirectInvocation::new(
            "test-handler",
            "test-action",
            serde_json::json!({ "value": 42 }),
        );
        
        let invocation2 = DirectInvocation::new(
            "test-handler",
            "test-action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Check that they have the same content ID
        assert_eq!(
            system.get_content_id(&invocation1),
            system.get_content_id(&invocation2)
        );
        
        // Create a different invocation
        let invocation3 = DirectInvocation::new(
            "test-handler",
            "test-action",
            serde_json::json!({ "value": 43 }), // Different value
        );
        
        // Check that it has a different content ID
        assert_ne!(
            system.get_content_id(&invocation1),
            system.get_content_id(&invocation3)
        );
        
        Ok(())
    }
} 