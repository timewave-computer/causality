// Context propagation for invocations
// Original file: src/invocation/context/propagation.rs

// Context propagation module for invocation contexts
//
// This module provides functionality for propagating invocation contexts
// between programs, allowing execution state to flow through invocations.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use chrono::Utc;
use rand;
use std::fmt;
use serde::{Serialize, Deserialize};

use causality_error::{Error, Result, EngineError};
use causality_types::TraceId;
use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};
use causality_domain::map::TimeMap;
use super::InvocationContext;

/// Invocation context data for ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct InvocationData {
    /// Timestamp when created
    timestamp: i64,
    /// Trace ID
    trace_id: String,
    /// Parent ID if any
    parent_id: Option<String>,
    /// Random nonce
    nonce: [u8; 8],
}

impl ContentAddressed for InvocationData {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Storage for active invocation contexts
#[derive(Debug)]
pub struct ContextStorage {
    /// Active contexts indexed by invocation ID
    contexts: RwLock<HashMap<String, Arc<RwLock<InvocationContext>>>>,
}

impl ContextStorage {
    /// Create a new context storage
    pub fn new() -> Self {
        ContextStorage {
            contexts: RwLock::new(HashMap::new()),
        }
    }
    
    /// Store a context
    pub fn store(&self, context: InvocationContext) -> Result<Arc<RwLock<InvocationContext>>> {
        let context_id = context.invocation_id.clone();
        let arc_context = Arc::new(RwLock::new(context));
        
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on contexts".to_string()))?;
        
        contexts.insert(context_id, arc_context.clone());
        
        Ok(arc_context)
    }
    
    /// Retrieve a context by ID
    pub fn get(&self, invocation_id: &str) -> Result<Option<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on contexts".to_string()))?;
        
        let context = contexts.get(invocation_id).cloned();
        
        Ok(context)
    }
    
    /// Remove a context
    pub fn remove(&self, invocation_id: &str) -> Result<Option<Arc<RwLock<InvocationContext>>>> {
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on contexts".to_string()))?;
        
        let context = contexts.remove(invocation_id);
        
        Ok(context)
    }
    
    /// Get all contexts for a trace
    pub fn get_by_trace(&self, trace_id: &TraceId) -> Result<Vec<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on contexts".to_string()))?;
        
        let trace_contexts = contexts.values()
            .filter(|ctx| {
                if let Ok(guard) = ctx.read() {
                    guard.trace_id == *trace_id
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        Ok(trace_contexts)
    }
    
    /// Count active contexts
    pub fn count(&self) -> Result<usize> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on contexts".to_string()))?;
        
        Ok(contexts.len())
    }
    
    /// Clear all contexts
    pub fn clear(&self) -> Result<()> {
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on contexts".to_string()))?;
        
        contexts.clear();
        
        Ok(())
    }
}

/// Context propagator for transferring context between programs
#[derive(Debug)]
pub struct ContextPropagator {
    /// Storage for active contexts
    storage: Arc<ContextStorage>,
}

impl ContextPropagator {
    /// Create a new context propagator with the given storage
    pub fn new(storage: Arc<ContextStorage>) -> Self {
        ContextPropagator {
            storage,
        }
    }
    
    /// Create a new context and store it
    pub fn create_context(
        &self,
        trace_id: Option<TraceId>,
        parent_id: Option<String>,
        time_map: TimeMap,
    ) -> Result<Arc<RwLock<InvocationContext>>> {
        // Use provided trace ID or create a new one
        let trace_id = trace_id.unwrap_or_else(TraceId::new);
        
        // Create invocation data for generating content ID
        let invocation_data = InvocationData {
            timestamp: Utc::now().timestamp(),
            trace_id: trace_id.to_string(),
            parent_id: parent_id.clone(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate a content-derived invocation ID
        let invocation_id = format!("invocation:{}", invocation_data.content_id());
        
        // Create the context
        let mut context = InvocationContext::new(
            invocation_id,
            trace_id,
            time_map,
        );
        
        // Set parent ID if provided
        if let Some(parent) = parent_id {
            context.parent_id = Some(parent.clone());
            
            // Also add this as a child to the parent
            if let Some(parent_ctx) = self.storage.get(&parent)? {
                if let Ok(mut parent_guard) = parent_ctx.write() {
                    parent_guard.add_child(&context.invocation_id)?;
                }
            }
        }
        
        // Store the context
        self.storage.store(context)
    }
    
    /// Create a child context from a parent
    pub fn create_child_context(
        &self,
        parent_id: &str,
    ) -> Result<Arc<RwLock<InvocationContext>>> {
        // Retrieve the parent context
        let parent_ctx = self.storage.get(parent_id)?
            .ok_or_else(|| Error::NotFound(format!("Parent context not found: {}", parent_id)))?;
        
        // Create invocation data for generating content ID
        let parent_guard = parent_ctx.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on parent context".to_string()))?;
        
        let invocation_data = InvocationData {
            timestamp: Utc::now().timestamp(),
            trace_id: parent_guard.trace_id.to_string(),
            parent_id: Some(parent_id.to_string()),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate a content-derived invocation ID
        let invocation_id = format!("invocation:{}", invocation_data.content_id());
        
        // Create the child context
        let child_context = parent_guard.create_child(invocation_id);
        
        // Add this as a child to the parent
        {
            let mut parent_guard = parent_ctx.write().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire write lock on parent context".to_string()))?;
            
            parent_guard.add_child(&child_context.invocation_id)?;
        }
        
        // Store the child context
        self.storage.store(child_context)
    }
    
    /// Start a context
    pub fn start_context(&self, invocation_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.start()
    }
    
    /// Complete a context
    pub fn complete_context(&self, invocation_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.complete()
    }
    
    /// Fail a context with a reason
    pub fn fail_context(&self, invocation_id: &str, reason: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.fail(reason)
    }
    
    /// Wait for a resource
    pub fn wait_for_resource(&self, invocation_id: &str, resource_id: ContentId) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.wait_for_resource(resource_id)
    }
    
    /// Wait for a fact
    pub fn wait_for_fact(&self, invocation_id: &str, fact_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.wait_for_fact(fact_id)
    }
    
    /// Resume a context
    pub fn resume_context(&self, invocation_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.resume()
    }
    
    /// Get all contexts for a trace
    pub fn get_trace_contexts(&self, trace_id: &TraceId) -> Result<Vec<Arc<RwLock<InvocationContext>>>> {
        self.storage.get_by_trace(trace_id)
    }
    
    /// Check if a trace is completely done (all invocations are in a final state)
    pub fn is_trace_complete(&self, trace_id: &TraceId) -> Result<bool> {
        // Get all contexts for this trace
        let contexts = self.storage.get_by_trace(trace_id)?;
        
        // If there are no contexts, consider it complete
        if contexts.is_empty() {
            return Ok(true);
        }
        
        // Check if all contexts are in a final state
        for ctx in &contexts {
            let guard = ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock on context".to_string()))?;
            
            if !guard.is_final() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Export a context for transfer to another process
    pub fn export_context(&self, invocation_id: &str) -> Result<SerializedContext> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on context".to_string()))?;
        
        // Serialize the time map
        let time_map_bytes = bincode::serialize(&guard.time_map)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize time map: {}", e)))?;
        
        // Serialize facts to JSON
        let facts_json = serde_json::to_string(&guard.observed_facts)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize facts: {}", e)))?;
        
        // Serialize metadata
        let metadata = serde_json::to_string(&guard.metadata)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize metadata: {}", e)))?;
        
        Ok(SerializedContext {
            invocation_id: guard.invocation_id.clone(),
            trace_id: guard.trace_id.to_string(),
            parent_id: guard.parent_id.clone(),
            state: format!("{:?}", guard.state),
            time_map_bytes,
            created_at: guard.created_at.timestamp(),
            started_at: guard.started_at.map(|t| t.timestamp()),
            completed_at: guard.completed_at.map(|t| t.timestamp()),
            facts_json,
            metadata,
        })
    }
    
    /// Import a context from another process
    pub fn import_context(&self, serialized: SerializedContext) -> Result<Arc<RwLock<InvocationContext>>> {
        // Deserialize the time map
        let time_map = bincode::deserialize(&serialized.time_map_bytes)
            .map_err(|e| Error::DeserializationError(format!("Failed to deserialize time map: {}", e)))?;
        
        // Create a new context
        let mut context = InvocationContext::new(
            serialized.invocation_id,
            TraceId::from_string(&serialized.trace_id)
                .map_err(|_| Error::DeserializationError("Invalid trace ID".to_string()))?,
            time_map,
        );
        
        // Set parent ID
        context.parent_id = serialized.parent_id;
        
        // Set timestamps
        context.created_at = chrono::DateTime::from_timestamp(serialized.created_at, 0)
            .ok_or_else(|| Error::DeserializationError("Invalid created_at timestamp".to_string()))?;
        
        if let Some(started_at) = serialized.started_at {
            context.started_at = chrono::DateTime::from_timestamp(started_at, 0);
        }
        
        if let Some(completed_at) = serialized.completed_at {
            context.completed_at = chrono::DateTime::from_timestamp(completed_at, 0);
        }
        
        // Deserialize facts
        let facts = serde_json::from_str(&serialized.facts_json)
            .map_err(|e| Error::DeserializationError(format!("Failed to deserialize facts: {}", e)))?;
        context.observed_facts = facts;
        
        // Deserialize metadata
        let metadata = serde_json::from_str(&serialized.metadata)
            .map_err(|e| Error::DeserializationError(format!("Failed to deserialize metadata: {}", e)))?;
        context.metadata = metadata;
        
        // Store the imported context
        self.storage.store(context)
    }

    /// Execute an action with a context from another process
    pub fn with_imported_context<F, R>(&self, serialized: SerializedContext, f: F) -> Result<R>
    where
        F: FnOnce(&Arc<RwLock<InvocationContext>>) -> Result<R>,
    {
        let context = self.import_context(serialized)?;
        let result = f(&context);
        
        // Export the context back after execution if needed
        // (This would be used to send the updated context back to the original process)
        
        result
    }
    
    /// Clone the storage
    pub fn clone_storage(&self) -> Arc<ContextStorage> {
        self.storage.clone()
    }
}

// Thread-local storage of the current context ID
thread_local! {
    static CURRENT_CONTEXT: Mutex<Option<String>> = Mutex::new(None);
}

/// Set the current context for this thread
pub fn set_current_context(invocation_id: Option<String>) -> Result<()> {
    CURRENT_CONTEXT.with(|ctx| {
        let mut current = ctx.lock().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire lock on current context".to_string()))?;
        
        *current = invocation_id;
        
        Ok(())
    })
}

/// Get the current context for this thread
pub fn get_current_context() -> Result<Option<String>> {
    CURRENT_CONTEXT.with(|ctx| {
        let current = ctx.lock().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire lock on current context".to_string()))?;
        
        Ok(current.clone())
    })
}

/// Execute a function within the scope of a context
pub fn with_context<F, R>(
    propagator: &ContextPropagator,
    invocation_id: &str,
    f: F
) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    // Save the previous context
    let previous = get_current_context()?;
    
    // Set the new context
    set_current_context(Some(invocation_id.to_string()))?;
    
    // Start the context if it's not already running
    let maybe_start = {
        let context = propagator.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on context".to_string()))?;
        
        !guard.is_active()
    };
    
    if maybe_start {
        propagator.start_context(invocation_id)?;
    }
    
    // Execute the function
    let result = f();
    
    // Complete the context if we started it and it was successful
    if maybe_start && result.is_ok() {
        let _ = propagator.complete_context(invocation_id);
    } else if maybe_start && result.is_err() {
        if let Err(err) = &result {
            let _ = propagator.fail_context(invocation_id, &format!("{}", err));
        }
    }
    
    // Restore the previous context
    set_current_context(previous)?;
    
    result
}

/// Context serialization format for cross-process propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedContext {
    /// The invocation ID
    pub invocation_id: String,
    /// The trace ID
    pub trace_id: String,
    /// Parent invocation ID if any
    pub parent_id: Option<String>,
    /// Current state of the invocation
    pub state: String,
    /// Time map serialized to bytes
    pub time_map_bytes: Vec<u8>,
    /// Creation timestamp
    pub created_at: i64,
    /// Start timestamp if started
    pub started_at: Option<i64>,
    /// Completion timestamp if completed
    pub completed_at: Option<i64>,
    /// Observed facts serialized to JSON
    pub facts_json: String,
    /// Metadata serialized to JSON
    pub metadata: String,
}

/// Remote context connector for cross-process propagation
pub struct RemoteContextConnector {
    /// Local propagator
    propagator: ContextPropagator,
    /// Remote endpoints to connect to
    endpoints: Vec<String>,
    /// Authentication token for remote endpoints
    auth_token: Option<String>,
}

impl RemoteContextConnector {
    /// Create a new remote context connector
    pub fn new(propagator: ContextPropagator, endpoints: Vec<String>) -> Self {
        RemoteContextConnector {
            propagator,
            endpoints,
            auth_token: None,
        }
    }
    
    /// Set the authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }
    
    /// Send a context to a remote endpoint
    pub async fn send_context(&self, invocation_id: &str, endpoint_index: usize) -> Result<()> {
        // Get the endpoint
        let endpoint = self.endpoints.get(endpoint_index)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid endpoint index: {}", endpoint_index)))?;
        
        // Export the context
        let serialized = self.propagator.export_context(invocation_id)?;
        
        // Convert to JSON
        let json = serde_json::to_string(&serialized)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize context: {}", e)))?;
        
        // Create HTTP client
        let client = reqwest::Client::new();
        let mut request = client.post(endpoint)
            .header("Content-Type", "application/json");
        
        // Add auth token if available
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        // Send the request
        let response = request.body(json).send().await
            .map_err(|e| Error::NetworkError(format!("Failed to send context: {}", e)))?;
        
        // Check the response
        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "Failed to send context: HTTP {}", response.status()
            )));
        }
        
        Ok(())
    }
    
    /// Receive a context from a remote endpoint
    pub async fn receive_context(&self, endpoint_index: usize) -> Result<Arc<RwLock<InvocationContext>>> {
        // Get the endpoint
        let endpoint = self.endpoints.get(endpoint_index)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid endpoint index: {}", endpoint_index)))?;
        
        // Create HTTP client
        let client = reqwest::Client::new();
        let mut request = client.get(endpoint);
        
        // Add auth token if available
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        // Send the request
        let response = request.send().await
            .map_err(|e| Error::NetworkError(format!("Failed to receive context: {}", e)))?;
        
        // Check the response
        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "Failed to receive context: HTTP {}", response.status()
            )));
        }
        
        // Parse the response
        let serialized: SerializedContext = response.json().await
            .map_err(|e| Error::DeserializationError(format!("Failed to parse response: {}", e)))?;
        
        // Import the context
        self.propagator.import_context(serialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    fn create_time_map() -> TimeMap {
        TimeMap::new()
    }
    
    #[test]
    fn test_context_storage() -> Result<()> {
        let storage = ContextStorage::new();
        
        // Create a test context
        let trace_id = TraceId::new();
        let time_map = create_time_map();
        let context = InvocationContext::new(
            "test-invocation-1".to_string(),
            trace_id.clone(),
            time_map,
        );
        
        // Store it
        let stored = storage.store(context)?;
        assert_eq!(storage.count()?, 1);
        
        // Retrieve it
        let retrieved = storage.get("test-invocation-1")?;
        assert!(retrieved.is_some());
        
        // Get by trace
        let trace_contexts = storage.get_by_trace(&trace_id)?;
        assert_eq!(trace_contexts.len(), 1);
        
        // Remove it
        let removed = storage.remove("test-invocation-1")?;
        assert!(removed.is_some());
        assert_eq!(storage.count()?, 0);
        
        // Clear should work on empty storage
        storage.clear()?;
        assert_eq!(storage.count()?, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_context_propagator() -> Result<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        
        // Create a new root context
        let trace_id = TraceId::new();
        let time_map = create_time_map();
        let root_ctx = propagator.create_context(
            Some(trace_id.clone()),
            None,
            time_map.clone(),
        )?;
        
        let root_id = {
            let guard = root_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            guard.invocation_id.clone()
        };
        
        // Start the context
        propagator.start_context(&root_id)?;
        
        {
            let guard = root_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            assert!(guard.is_active());
        }
        
        // Create a child context
        let child_ctx = propagator.create_child_context(&root_id)?;
        
        let child_id = {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            guard.invocation_id.clone()
        };
        
        // Start the child
        propagator.start_context(&child_id)?;
        
        // Make the child wait for a resource
        let resource_id = ContentId::new();
        propagator.wait_for_resource(&child_id, resource_id.clone())?;
        
        {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            match guard.state {
                super::InvocationState::Waiting(id) => assert_eq!(id, resource_id),
                _ => panic!("Expected Waiting state"),
            }
        }
        
        // Resume the child
        propagator.resume_context(&child_id)?;
        
        {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            assert_eq!(guard.state, super::InvocationState::Running);
        }
        
        // Complete the child
        propagator.complete_context(&child_id)?;
        
        {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            assert!(guard.is_final());
        }
        
        // Fail the root context
        propagator.fail_context(&root_id, "Test failure")?;
        
        {
            let guard = root_ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            assert!(guard.is_final());
        }
        
        // Check trace completion
        assert!(propagator.is_trace_complete(&trace_id)?);
        
        Ok(())
    }
    
    #[test]
    fn test_thread_local_context() -> Result<()> {
        // Set the current context
        set_current_context(Some("test-thread-context".to_string()))?;
        
        // Get it back
        let current = get_current_context()?;
        assert_eq!(current, Some("test-thread-context".to_string()));
        
        // Clear it
        set_current_context(None)?;
        
        // Should be None now
        let current = get_current_context()?;
        assert_eq!(current, None);
        
        Ok(())
    }
    
    #[test]
    fn test_with_context() -> Result<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        
        // Create a context
        let time_map = create_time_map();
        let ctx = propagator.create_context(None, None, time_map)?;
        
        let id = {
            let guard = ctx.read().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
            guard.invocation_id.clone()
        };
        
        // Run a function with this context
        let result = with_context(&propagator, &id, || {
            // Get the current context
            let current = get_current_context()?;
            assert_eq!(current, Some(id.clone()));
            
            // Return a value
            Ok::<_, Error>(42)
        })?;
        
        assert_eq!(result, 42);
        
        // Check the context state
        let context = propagator.storage.get(&id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock".to_string()))?;
        
        assert!(guard.is_final());
        
        Ok(())
    }
} 
