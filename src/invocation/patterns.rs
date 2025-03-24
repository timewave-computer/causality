// Invocation patterns module
//
// This module defines patterns for effect invocation, including direct,
// callback-based, continuation-based, promise-based, and streaming invocation.
// All patterns use content addressing for tracking and identification.

use std::sync::{Arc, RwLock};
use std::pin::Pin;
use std::future::Future;
use async_trait::async_trait;
use tokio::sync::{oneshot, mpsc};
use serde::{Serialize, Deserialize};
use blake3::Hasher;
use hex;

use crate::error::{Error, Result};
use crate::types::{TraceId, ContentId, ContentHash};
use crate::domain::map::map::TimeMap;
use crate::invocation::context::{
    InvocationContext,
    propagation::{ContextPropagator, ContextStorage},
};
use crate::invocation::registry::{
    EffectRegistry, 
    EffectHandler, 
    HandlerInput, 
    HandlerOutput,
};

/// Common interface for all invocation patterns
#[async_trait]
pub trait InvocationPattern: Send + Sync {
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

//----------------------------------------------------------
// Direct Invocation Pattern
//----------------------------------------------------------

/// Direct invocation - simple synchronous request-response pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectInvocation {
    /// Handler ID to invoke
    pub handler_id: String,
    /// Action to perform
    pub action: String,
    /// Input parameters
    pub params: serde_json::Value,
    /// Trace ID for tracking related invocations
    pub trace_id: Option<TraceId>,
    /// Parent invocation ID if this is a child invocation
    pub parent_id: Option<String>,
    /// Content hash of the inputs (for content addressing)
    pub content_hash: ContentHash,
}

impl DirectInvocation {
    /// Create a new direct invocation
    pub fn new(
        handler_id: impl Into<String>,
        action: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        let handler_id = handler_id.into();
        let action = action.into();
        
        // Generate a content hash for this invocation
        let mut hasher = Hasher::new();
        hasher.update(handler_id.as_bytes());
        hasher.update(action.as_bytes());
        hasher.update(params.to_string().as_bytes());
        let hash = hasher.finalize();
        let content_hash = ContentHash(hex::encode(hash.as_bytes()));
        
        DirectInvocation {
            handler_id,
            action,
            params,
            trace_id: None,
            parent_id: None,
            content_hash,
        }
    }
    
    /// Set the trace ID for this invocation
    pub fn with_trace(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Set the parent ID for this invocation
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

#[async_trait]
impl InvocationPattern for DirectInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("direct-invocation:{}", self.content_hash.0))
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        // Get the handler
        let handler = registry.get_handler(&self.handler_id)?
            .ok_or_else(|| Error::NotFound(format!("Handler not found: {}", self.handler_id)))?;
        
        // Create a time map
        let time_map = TimeMap::new();
        
        // Create an invocation context
        let context = propagator.create_context(
            self.trace_id.clone(),
            self.parent_id.clone(),
            time_map,
        )?;
        
        // Get the invocation ID
        let invocation_id = {
            let guard = context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on context".to_string()))?;
            
            guard.invocation_id.clone()
        };
        
        // Start the context
        propagator.start_context(&invocation_id)?;
        
        // Create the handler input
        let input = HandlerInput {
            action: self.action.clone(),
            params: self.params.clone(),
            context: context.clone(),
        };
        
        // Handle the invocation
        let result = handler.handle(input).await;
        
        // Complete or fail the context based on the result
        match &result {
            Ok(_) => propagator.complete_context(&invocation_id)?,
            Err(e) => propagator.fail_context(&invocation_id, &e.to_string())?,
        }
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Direct invocation of {} / {}", self.handler_id, self.action)
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern_type": "direct",
            "handler_id": self.handler_id,
            "action": self.action,
            "content_hash": self.content_hash.0,
        })
    }
}

//----------------------------------------------------------
// Callback-based Invocation Pattern
//----------------------------------------------------------

/// Type for invocation callbacks
pub type InvocationCallback = Box<dyn FnOnce(Result<HandlerOutput>) + Send + 'static>;

/// Callback-based invocation - invokes a handler and calls back when complete
#[derive(Debug)]
pub struct CallbackInvocation {
    /// The direct invocation to perform
    pub invocation: DirectInvocation,
    /// The callback function to call with the result
    pub callback: Option<InvocationCallback>,
}

impl CallbackInvocation {
    /// Create a new callback invocation
    pub fn new(
        invocation: DirectInvocation,
        callback: impl FnOnce(Result<HandlerOutput>) + Send + 'static,
    ) -> Self {
        CallbackInvocation {
            invocation,
            callback: Some(Box::new(callback)),
        }
    }
}

#[async_trait]
impl InvocationPattern for CallbackInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("callback-invocation:{}", self.invocation.content_hash.0))
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        let result = self.invocation.execute(registry, propagator).await;
        
        // Execute the callback with the result if present
        if let Some(callback) = &self.callback {
            let callback = std::mem::replace(&mut self.callback.clone().unwrap(), Box::new(|_| {}));
            callback(result.clone());
        }
        
        result
    }
    
    fn get_description(&self) -> String {
        format!("Callback invocation of {}", self.invocation.get_description())
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        let mut metadata = self.invocation.get_metadata();
        
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("pattern_type".to_string(), serde_json::Value::String("callback".to_string()));
            map.insert("has_callback".to_string(), serde_json::Value::Bool(self.callback.is_some()));
        }
        
        metadata
    }
}

//----------------------------------------------------------
// Continuation-based Invocation Pattern
//----------------------------------------------------------

/// Continuation function type for processing invocation results
pub type InvocationContinuation<T> = Box<dyn FnOnce(Result<HandlerOutput>) -> Result<T> + Send + 'static>;

/// Continuation-based invocation - chains multiple invocations together
#[derive(Debug)]
pub struct ContinuationInvocation<T: Send + 'static> {
    /// The direct invocation to perform
    pub invocation: DirectInvocation,
    /// The continuation function to process the result
    pub continuation: Option<InvocationContinuation<T>>,
    /// Content hash for the continuation (derived from invocation + continuation type)
    pub content_hash: ContentHash,
}

impl<T: Send + 'static> ContinuationInvocation<T> {
    /// Create a new continuation invocation
    pub fn new(
        invocation: DirectInvocation,
        continuation: impl FnOnce(Result<HandlerOutput>) -> Result<T> + Send + 'static,
    ) -> Self {
        // Add the type name to the content hash for the continuation
        let type_name = std::any::type_name::<T>();
        
        let mut hasher = Hasher::new();
        hasher.update(invocation.content_hash.0.as_bytes());
        hasher.update(type_name.as_bytes());
        let hash = hasher.finalize();
        let content_hash = ContentHash(hex::encode(hash.as_bytes()));
        
        Self {
            invocation,
            continuation: Some(Box::new(continuation)),
            content_hash,
        }
    }
    
    /// Execute the invocation and apply the continuation
    pub async fn execute_with_continuation(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<T> {
        let result = self.invocation.execute(registry, propagator).await;
        
        // Apply the continuation to the result if present
        if let Some(continuation) = &self.continuation {
            let continuation = std::mem::replace(&mut self.continuation.clone().unwrap(), Box::new(|_| panic!("Continuation called twice")));
            continuation(result)
        } else {
            Err(Error::InternalError("No continuation available".to_string()))
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> InvocationPattern for ContinuationInvocation<T> {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("continuation-invocation:{}", self.content_hash.0))
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        // For the InvocationPattern trait, we can't return T directly,
        // so we'll just execute the invocation part and return that.
        self.invocation.execute(registry, propagator).await
    }
    
    fn get_description(&self) -> String {
        format!("Continuation invocation of {}", self.invocation.get_description())
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        let mut metadata = self.invocation.get_metadata();
        
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("pattern_type".to_string(), serde_json::Value::String("continuation".to_string()));
            map.insert("continuation_type".to_string(), serde_json::Value::String(std::any::type_name::<T>().to_string()));
            map.insert("content_hash".to_string(), serde_json::Value::String(self.content_hash.0.clone()));
        }
        
        metadata
    }
}

//----------------------------------------------------------
// Promise-based Invocation Pattern
//----------------------------------------------------------

/// Future type for promise-based invocations
pub type InvocationFuture = Pin<Box<dyn Future<Output = Result<HandlerOutput>> + Send>>;

/// Promise-based invocation - returns a future that resolves with the result
#[derive(Debug)]
pub struct PromiseInvocation {
    /// The direct invocation to perform
    pub invocation: DirectInvocation,
}

impl PromiseInvocation {
    /// Create a new promise invocation
    pub fn new(invocation: DirectInvocation) -> Self {
        PromiseInvocation {
            invocation,
        }
    }
    
    /// Execute the invocation and return a future that resolves with the result
    pub fn execute_async(
        self,
        registry: Arc<EffectRegistry>,
        propagator: Arc<ContextPropagator>,
    ) -> InvocationFuture {
        // Create a future that performs the invocation
        Box::pin(async move {
            self.invocation.execute(&registry, &propagator).await
        })
    }
}

#[async_trait]
impl InvocationPattern for PromiseInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("promise-invocation:{}", self.invocation.content_hash.0))
    }
    
    async fn execute(
        &self,
        registry: &EffectRegistry,
        propagator: &ContextPropagator,
    ) -> Result<HandlerOutput> {
        self.invocation.execute(registry, propagator).await
    }
    
    fn get_description(&self) -> String {
        format!("Promise invocation of {}", self.invocation.get_description())
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        let mut metadata = self.invocation.get_metadata();
        
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("pattern_type".to_string(), serde_json::Value::String("promise".to_string()));
        }
        
        metadata
    }
}

//----------------------------------------------------------
// Streaming Invocation Pattern
//----------------------------------------------------------

/// Streaming invocation - streams results as they become available
#[derive(Debug)]
pub struct StreamingInvocation {
    /// The direct invocation to perform
    pub invocation: DirectInvocation,
    /// Channel capacity for the stream
    pub channel_capacity: usize,
}

impl StreamingInvocation {
    /// Create a new streaming invocation
    pub fn new(invocation: DirectInvocation, channel_capacity: usize) -> Self {
        StreamingInvocation {
            invocation,
            channel_capacity,
        }
    }
    
    /// Execute the invocation and return a stream of results
    pub fn execute_streaming(
        self,
        registry: Arc<EffectRegistry>,
        propagator: Arc<ContextPropagator>,
    ) -> (mpsc::Receiver<Result<HandlerOutput>>, InvocationFuture) {
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
impl InvocationPattern for StreamingInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("streaming-invocation:{}", self.invocation.content_hash.0))
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
        // Generate a content hash for the batch
        let mut hasher = Hasher::new();
        
        for invocation in &invocations {
            hasher.update(invocation.content_hash.0.as_bytes());
        }
        hasher.update(if parallel { b"parallel" } else { b"sequential" });
        let hash = hasher.finalize();
        let content_hash = ContentHash(hex::encode(hash.as_bytes()));
        
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
impl InvocationPattern for BatchInvocation {
    fn get_content_id(&self) -> ContentId {
        ContentId::new(format!("batch-invocation:{}", self.content_hash.0))
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
            "content_hash": self.content_hash.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::invocation::{AccessLevel, HandlerRegistration};
    use crate::domain::DomainId;
    use crate::crypto::hash::ContentId;
    
    struct TestHandler {
        registration: HandlerRegistration,
    }
    
    impl TestHandler {
        fn new(id: &str, domain: DomainId) -> Self {
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
        let domain = DomainId::new();
        let handler = Arc::new(TestHandler::new("test_handler", domain));
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
        assert_eq!(invocation1.content_hash.0, invocation2.content_hash.0);
        
        // Content IDs should be the same
        assert_eq!(invocation1.get_content_id(), invocation2.get_content_id());
        
        // Create a different invocation and check that it has a different content ID
        let invocation3 = DirectInvocation::new(
            "test_handler",
            "test_action",
            serde_json::json!({ "value": 43 }), // Different value
        );
        
        // Content hash should be different
        assert_ne!(invocation1.content_hash.0, invocation3.content_hash.0);
        
        // Content ID should be different
        assert_ne!(invocation1.get_content_id(), invocation3.get_content_id());
        
        Ok(())
    }
} 
