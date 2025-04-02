// Common invocation patterns
// Original file: src/invocation/patterns.rs

// Invocation patterns module
//
// This module defines patterns for effect invocation, including direct,
// callback-based, continuation-based, promise-based, and streaming invocation.
// All patterns use content addressing for tracking and identification.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fmt::Debug;
use std::string::String;

use causality_error::{Result, Error};
use causality_types::ContentId;
use causality_core::time::TimeMap;

use super::context::InvocationContext;
use super::context::propagation::ContextPropagator;

use super::registry::{
    EffectRegistry, 
    EffectHandler, 
    HandlerInput, 
    HandlerOutput,
};
use causality_types::crypto_primitives::ContentHash;

// Common interface for all invocation patterns
#[async_trait]
pub trait InvocationPatternTrait: Send + Sync {
    /// Get a unique content identifier for this invocation pattern
    fn get_content_id(&self) -> ContentId;
    
    /// Execute the invocation with the given registry and propagator
    async fn execute(
        &self, 
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput>;
    
    /// Get a description of this invocation pattern
    fn get_description(&self) -> String;
    
    /// Get metadata about this invocation pattern
    fn get_metadata(&self) -> serde_json::Value;
}

/// Represents different invocation patterns for operation handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvocationPatternEnum {
    /// Direct synchronous invocation
    Direct(DirectInvocation),
    /// Callback-based invocation
    Callback(CallbackInvocation),
    /// Continuation-based invocation
    Continuation(ContinuationInvocation),
    /// Promise-based invocation
    Promise(PromiseInvocation),
    /// Streaming invocation pattern
    Streaming(StreamingInvocation),
    /// Batch processing invocation
    Batch(BatchInvocation),
}

//----------------------------------------------------------
// Direct Invocation Pattern
//----------------------------------------------------------

/// Direct synchronous invocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectInvocation {
    /// Target service ID
    pub target_service: String,
    /// Operation to invoke
    pub operation: String,
}

impl DirectInvocation {
    /// Create a new direct invocation
    pub fn new(
        target_service: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        let target_service = target_service.into();
        let operation = operation.into();
        
        DirectInvocation {
            target_service,
            operation,
        }
    }
}

#[async_trait]
impl InvocationPatternTrait for DirectInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.target_service.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        // Get the handler
        let handler = registry.get_handler(&self.target_service)?
            .ok_or_else(|| Error::NotFound(format!("Handler not found: {}", self.target_service)))?;
        
        // Create a time map
        let time_map = TimeMap::new();
        
        // Create an invocation context
        let context = propagator.create_context(
            None,
            None,
            time_map,
        )?;
        
        // Create the handler input
        let input = HandlerInput {
            action: self.operation.clone(),
            params: serde_json::json!({}),
            context_id: context.id().to_string(),
        };
        
        // Handle the invocation
        let result = handler.handle(input).await;
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Direct invocation of {} / {}", self.target_service, self.operation)
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "direct",
            "target_service": self.target_service,
            "operation": self.operation,
        })
    }
}

/// Type for future returned from streaming invocation
pub type InvocationStreamFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;

/// Streaming invocation future
pub struct StreamingInvocationFuture {
    /// The future that completes when the stream is done
    future: InvocationStreamFuture,
}

impl StreamingInvocationFuture {
    /// Create a new streaming invocation future
    pub fn new(future: InvocationStreamFuture) -> Self {
        Self { future }
    }
    
    /// Wait for the stream to complete
    pub async fn await_completion(self) -> Result<()> {
        self.future.await
    }
}

//----------------------------------------------------------
// Callback-based Invocation Pattern
//----------------------------------------------------------

/// Callback-based invocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackInvocation {
    /// Target service ID
    pub target_service: String,
    /// Operation to invoke
    pub operation: String,
    /// Callback endpoint
    pub callback_url: String,
}

impl CallbackInvocation {
    /// Create a new callback invocation
    pub fn new(
        target_service: impl Into<String>,
        operation: impl Into<String>,
        callback_url: impl Into<String>,
    ) -> Self {
        let target_service = target_service.into();
        let operation = operation.into();
        let callback_url = callback_url.into();
        
        CallbackInvocation {
            target_service,
            operation,
            callback_url,
        }
    }
}

#[async_trait]
impl InvocationPatternTrait for CallbackInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.target_service.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        let result = self.execute(registry, propagator).await?;
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Callback invocation of {} / {}", self.target_service, self.operation)
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "callback",
            "target_service": self.target_service,
            "operation": self.operation,
            "callback_url": self.callback_url,
        })
    }
}

//----------------------------------------------------------
// Continuation-based Invocation Pattern
//----------------------------------------------------------

/// Continuation-based invocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationInvocation {
    /// Target service ID
    pub target_service: String,
    /// Operation to invoke
    pub operation: String,
    /// Continuation ID for resuming execution
    pub continuation_id: String,
}

impl ContinuationInvocation {
    /// Create a new continuation invocation
    pub fn new(
        target_service: impl Into<String>,
        operation: impl Into<String>,
        continuation_id: impl Into<String>,
    ) -> Self {
        let target_service = target_service.into();
        let operation = operation.into();
        let continuation_id = continuation_id.into();
        
        ContinuationInvocation {
            target_service,
            operation,
            continuation_id,
        }
    }
}

#[async_trait]
impl InvocationPatternTrait for ContinuationInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.target_service.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        let result = self.execute(registry, propagator).await?;
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Continuation invocation of {} / {}", self.target_service, self.operation)
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "continuation",
            "target_service": self.target_service,
            "operation": self.operation,
            "continuation_id": self.continuation_id,
        })
    }
}

//----------------------------------------------------------
// Promise-based Invocation Pattern
//----------------------------------------------------------

/// Promise-based invocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromiseInvocation {
    /// Target service ID
    pub target_service: String,
    /// Operation to invoke
    pub operation: String,
    /// Time until promise expiration
    pub timeout_ms: u64,
}

impl PromiseInvocation {
    /// Create a new promise invocation
    pub fn new(
        target_service: impl Into<String>,
        operation: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        let target_service = target_service.into();
        let operation = operation.into();
        
        PromiseInvocation {
            target_service,
            operation,
            timeout_ms,
        }
    }
}

#[async_trait]
impl InvocationPatternTrait for PromiseInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.target_service.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        let result = self.execute(registry, propagator).await?;
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Promise invocation of {} / {}", self.target_service, self.operation)
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "promise",
            "target_service": self.target_service,
            "operation": self.operation,
            "timeout_ms": self.timeout_ms,
        })
    }
}

//----------------------------------------------------------
// Streaming Invocation Pattern
//----------------------------------------------------------

/// Streaming invocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingInvocation {
    /// Target service ID
    pub target_service: String,
    /// Operation to invoke
    pub operation: String,
    /// Stream ID
    pub stream_id: String,
}

impl StreamingInvocation {
    /// Create a new streaming invocation
    pub fn new(
        target_service: impl Into<String>,
        operation: impl Into<String>,
        stream_id: impl Into<String>,
    ) -> Self {
        let target_service = target_service.into();
        let operation = operation.into();
        let stream_id = stream_id.into();
        
        StreamingInvocation {
            target_service,
            operation,
            stream_id,
        }
    }
}

#[async_trait]
impl InvocationPatternTrait for StreamingInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.target_service.as_bytes())
    }
    
    /// Execute the invocation and return a stream of results
    pub fn execute_streaming(
        self,
        registry: Arc<EffectRegistry>,
        propagator: Arc<ContextPropagator>,
    ) -> (mpsc::Receiver<Result<HandlerOutput>>, InvocationStreamFuture) {
        // Create a channel for streaming results
        let (tx, rx) = mpsc::channel(self.channel_capacity);
        
        // Create a future that performs the invocation and sends the result to the channel
        let future = Box::pin(async move {
            let result = self.invocation.execute(&registry, &propagator).await;
            
            // Try to send the result to the channel, but don't fail if the receiver is gone
            let _ = tx.send(result.clone()).await;
            
            result
        });
        
        (rx, future)
    }
}

#[async_trait]
impl InvocationPatternTrait for StreamingInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.invocation.content_hash.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        self.invocation.execute(registry, propagator).await
    }
    
    fn get_description(&self) -> String {
        format!("Streaming invocation of {}", self.invocation.get_description())
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        let mut metadata = self.invocation.get_metadata();
        
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("pattern_type".to_string(), serde_json::Value::String("streaming".to_string()));
            map.insert("channel_capacity".to_string(), serde_json::Value::Number(serde_json::Number::from(self.channel_capacity)));
        }
        
        metadata
    }
}

//----------------------------------------------------------
// Batch Invocation Pattern
//----------------------------------------------------------

/// Batch invocation - executes multiple invocations as a batch
#[derive(Debug)]
pub struct BatchInvocation {
    /// The invocations to perform
    pub invocations: Vec<DirectInvocation>,
    /// Whether to execute the invocations in parallel or sequentially
    pub parallel: bool,
    /// Content hash for the batch
    pub content_hash: ContentHash,
}

impl BatchInvocation {
    /// Create a new batch invocation
    pub fn new(invocations: Vec<DirectInvocation>, parallel: bool) -> Self {
        // Generate a content hash for this batch
        let mut hasher = Sha256::new();
        for inv in &invocations {
            hasher.update(inv.content_hash.to_hex().as_bytes());
        }
        hasher.update(&[parallel as u8]);
        let hash_bytes = hasher.finalize().to_vec();
        let content_hash = ContentHash::new("sha256", hash_bytes);
        
        BatchInvocation {
            invocations,
            parallel,
            content_hash,
        }
    }
    
    /// Execute the invocations in the batch and return all results
    pub async fn execute_batch(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<Vec<Result<HandlerOutput>>> {
        let mut results = Vec::with_capacity(self.invocations.len());
        
        if self.parallel {
            // Execute in parallel
            let futures = self.invocations.iter()
                .map(|invocation| invocation.execute(registry, propagator));
            
            // Collect results as they complete
            for result in futures::future::join_all(futures).await {
                results.push(result);
            }
        } else {
            // Execute sequentially
            for invocation in &self.invocations {
                let result = invocation.execute(registry, propagator).await;
                results.push(result);
            }
        }
        
        Ok(results)
    }
}

#[async_trait]
impl InvocationPatternTrait for BatchInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::from_bytes(self.content_hash.as_bytes())
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        // For the trait implementation, we'll execute all invocations and return 
        // a composite result with all outputs
        let results = self.execute_batch(registry, propagator).await?;
        
        // Create a composite result
        let outputs: Vec<_> = results.into_iter()
            .map(|r| match r {
                Ok(output) => output.data,
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            })
            .collect();
        
        Ok(HandlerOutput::new(serde_json::json!({
            "batch_results": outputs,
            "total_invocations": self.invocations.len(),
            "parallel": self.parallel,
        })))
    }
    
    fn get_description(&self) -> String {
        format!("Batch invocation of {} effects ({})", 
            self.invocations.len(),
            if self.parallel { "parallel" } else { "sequential" }
        )
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "batch",
            "invocation_count": self.invocations.len(),
            "parallel": self.parallel,
            "content_hash": self.content_hash.to_hex(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::domain::DomainId;
    
    struct TestHandler {
        registration: HandlerRegistration,
    }
    
    impl TestHandler {
        fn new(id: &str) -> Self {
            let domain = DomainId::new("test-domain");
            
            TestHandler {
                registration: HandlerRegistration::new(
                    id,
                    format!("Test Handler {}", id),
                    "Test handler for unit tests",
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
            // Simple echo handler that returns the input params as output
            Ok(HandlerOutput::new(input.params))
        }
    }
    
    fn setup_test_environment() -> (Arc<EffectRegistry>, Arc<ContextPropagator>) {
        let registry = Arc::new(EffectRegistry::new());
        let storage = Arc::new(ContextStorage::new());
        let propagator = Arc::new(ContextPropagator::new(storage));
        
        // Register a test handler
        let handler = Arc::new(TestHandler::new("test_handler"));
        registry.register_handler(handler).unwrap();
        
        (registry, propagator)
    }
    
    #[tokio::test]
    async fn test_direct_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create a direct invocation
        let invocation = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({
                "value": 42,
                "message": "hello",
            }),
        );
        
        // Execute the invocation
        let result = invocation.execute(&registry, &propagator).await?;
        
        // Check the result
        assert_eq!(result.data["value"], 42);
        assert_eq!(result.data["message"], "hello");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_callback_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create a callback channel to verify the callback is called
        let (tx, rx) = oneshot::channel();
        
        // Create a direct invocation
        let direct = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Create a callback invocation
        let invocation = CallbackInvocation::new(
            direct,
            move |result| {
                tx.send(result).unwrap();
                Ok(())
            },
        );
        
        // Execute the invocation
        let result = invocation.execute(&registry, &propagator).await?;
        
        // Check the direct result
        assert_eq!(result.data["value"], 42);
        
        // Check the callback result
        let callback_result = rx.await.unwrap()?;
        assert_eq!(callback_result.data["value"], 42);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_continuation_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create a direct invocation
        let direct = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Create a continuation invocation
        let invocation = ContinuationInvocation::<i32>::new(
            direct,
            |result| {
                let result = result?;
                let value = result.data["value"].as_i64().unwrap() as i32;
                Ok(value * 2) // Double the value
            },
        );
        
        // Execute the invocation
        let result = invocation.execute(&registry, &propagator).await?;
        
        // Check the direct result
        assert_eq!(result.data["value"], 42);
        
        // Execute with continuation
        let transformed = invocation.execute_with_continuation(&registry, &propagator).await?;
        
        // Check the transformed result
        assert_eq!(transformed, 84); // 42 * 2
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_promise_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create a direct invocation
        let direct = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Create a promise invocation
        let invocation = PromiseInvocation::new(direct);
        
        // Execute the invocation asynchronously
        let future = invocation.execute_async(registry.clone(), propagator.clone());
        
        // Await the future
        let result = future.await?;
        
        // Check the result
        assert_eq!(result.data["value"], 42);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_streaming_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create a direct invocation
        let direct = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Create a streaming invocation
        let invocation = StreamingInvocation::new(direct, 10);
        
        // Execute the invocation with streaming
        let (mut rx, future) = invocation.execute_streaming(registry.clone(), propagator.clone());
        
        // Check that we can get the result from the stream
        let stream_result = rx.recv().await.unwrap()?;
        assert_eq!(stream_result.data["value"], 42);
        
        // Also check the future result
        let future_result = future.await?;
        assert_eq!(future_result.data["value"], 42);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_batch_invocation() -> Result<()> {
        let (registry, propagator) = setup_test_environment();
        
        // Create multiple direct invocations
        let invocations = vec![
            DirectInvocation::new(
                "test_handler",
                "test_action",
                serde_json::json!({ "value": 1 }),
            ),
            DirectInvocation::new(
                "test_handler",
                "test_action",
                serde_json::json!({ "value": 2 }),
            ),
            DirectInvocation::new(
                "test_handler",
                "test_action",
                serde_json::json!({ "value": 3 }),
            ),
        ];
        
        // Test sequential batch
        let sequential_batch = BatchInvocation::new(invocations.clone(), false);
        let sequential_results = sequential_batch.execute_batch(&registry, &propagator).await?;
        
        // Check sequential results
        assert_eq!(sequential_results.len(), 3);
        assert_eq!(sequential_results[0].as_ref().unwrap().data["value"], 1);
        assert_eq!(sequential_results[1].as_ref().unwrap().data["value"], 2);
        assert_eq!(sequential_results[2].as_ref().unwrap().data["value"], 3);
        
        // Test parallel batch
        let parallel_batch = BatchInvocation::new(invocations, true);
        let parallel_results = parallel_batch.execute_batch(&registry, &propagator).await?;
        
        // Check parallel results (order may vary)
        assert_eq!(parallel_results.len(), 3);
        let values: Vec<i64> = parallel_results.iter()
            .map(|r| r.as_ref().unwrap().data["value"].as_i64().unwrap())
            .collect();
        
        // Sort and check values (doesn't matter what order they came in)
        let mut sorted_values = values.clone();
        sorted_values.sort();
        assert_eq!(sorted_values, vec![1, 2, 3]);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_content_addressing() -> Result<()> {
        // Create identical invocations and check that they have the same content ID
        let invocation1 = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        let invocation2 = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 42 }),
        );
        
        // Content hashes should be the same
        assert_eq!(invocation1.content_hash.to_hex(), invocation2.content_hash.to_hex());
        
        // Content IDs should be the same
        assert_eq!(invocation1.get_content_id(), invocation2.get_content_id());
        
        // Create a different invocation and check that it has a different content ID
        let invocation3 = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 43 }), // Different value
        );
        
        // Content hash should be different
        assert_ne!(invocation1.content_hash.to_hex(), invocation3.content_hash.to_hex());
        
        // Content ID should be different
        assert_ne!(invocation1.get_content_id(), invocation3.get_content_id());
        
        Ok(())
    }
} 
