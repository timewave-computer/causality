// Context propagation for invocations
// Original file: src/invocation/context/propagation.rs

// Context propagation module for invocation contexts
//
// This module provides functionality for propagating invocation contexts
// between programs, allowing execution state to flow through invocations.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use chrono::Utc;
use rand;

use causality_error::{EngineError, EngineResult};
use causality_types::{TraceId, ContentId, ContentAddressed};
use causality_types::crypto_primitives::{HashOutput, HashAlgorithm};
use causality_core::time::TimeMap;
use super::InvocationContext;

/// Invocation context data for ID generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationData {
    /// Timestamp when the invocation was created
    pub timestamp: i64,
    /// Trace ID in string format
    pub trace_id: String,
    /// Optional parent ID
    pub parent_id: Option<String>,
    /// Random nonce for uniqueness
    pub nonce: [u8; 8],
}

impl ContentAddressed for InvocationData {
    fn content_hash(&self) -> std::result::Result<HashOutput, causality_types::HashError> {
        let mut hasher = blake3::Hasher::new();
        let serialized = serde_json::to_vec(self).map_err(|e| 
            causality_types::HashError::SerializationError(e.to_string())
        )?;
        hasher.update(&serialized);
        let hash_bytes = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(hash_bytes.as_bytes());
        Ok(HashOutput::new(output, HashAlgorithm::Blake3))
    }
    
    fn verify(&self, expected_hash: &HashOutput) -> std::result::Result<bool, causality_types::HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    fn to_bytes(&self) -> std::result::Result<Vec<u8>, causality_types::HashError> {
        serde_json::to_vec(self).map_err(|e| 
            causality_types::HashError::SerializationError(e.to_string())
        )
    }
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, causality_types::HashError> {
        serde_json::from_slice(bytes).map_err(|e| 
            causality_types::HashError::SerializationError(e.to_string())
        )
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
    pub fn store(&self, context: InvocationContext) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        let context_id = context.id().to_string();
        let arc_context = Arc::new(RwLock::new(context));
        
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on contexts".to_string()))?;
        
        contexts.insert(context_id, arc_context.clone());
        
        Ok(arc_context)
    }
    
    /// Retrieve a context by ID
    pub fn get(&self, invocation_id: &str) -> EngineResult<Option<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on contexts".to_string()))?;
        
        let context = contexts.get(invocation_id).cloned();
        
        Ok(context)
    }
    
    /// Remove a context
    pub fn remove(&self, invocation_id: &str) -> EngineResult<Option<Arc<RwLock<InvocationContext>>>> {
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on contexts".to_string()))?;
        
        let context = contexts.remove(invocation_id);
        
        Ok(context)
    }
    
    /// Get all contexts for a trace
    pub fn get_by_trace(&self, trace_id: &TraceId) -> EngineResult<Vec<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on contexts".to_string()))?;
        
        let trace_contexts = contexts.values()
            .filter(|ctx| {
                if let Ok(guard) = ctx.read() {
                    if let Some(ctx_trace_id) = guard.trace_id() {
                        return ctx_trace_id == trace_id;
                    }
                }
                false
            })
            .cloned()
            .collect();
        
        Ok(trace_contexts)
    }
    
    /// Count active contexts
    pub fn count(&self) -> EngineResult<usize> {
        let contexts = self.contexts.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on contexts".to_string()))?;
        
        Ok(contexts.len())
    }
    
    /// Clear all contexts
    pub fn clear(&self) -> EngineResult<()> {
        let mut contexts = self.contexts.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on contexts".to_string()))?;
        
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
    
    /// Create a context with specified trace ID and parent
    pub fn create_context(
        &self,
        trace_id: Option<TraceId>,
        parent_id: Option<String>,
        time_map: TimeMap,
    ) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        // Create invocation data for generating content ID
        let invocation_data = InvocationData {
            timestamp: Utc::now().timestamp(),
            trace_id: trace_id.as_ref().map(|t| t.as_str().to_string()).unwrap_or_default(),
            parent_id: parent_id.clone(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate a deterministic invocation ID
        let content_id = invocation_data.content_id().map_err(|e| 
            EngineError::InternalError(format!("Failed to generate content ID: {}", e))
        )?;
        
        let invocation_id = format!("invocation:{}", content_id);
        
        // Create the context
        let context = InvocationContext::new(
            invocation_id,
            trace_id,
            parent_id.clone(),
            time_map,
        );
        
        // If there's a parent ID, also register this as a child
        if let Some(parent) = parent_id.as_ref() {
            // Also add this as a child to the parent
            if let Some(parent_ctx) = self.storage.get(parent)? {
                let mut parent_guard = parent_ctx.write().map_err(|_| 
                    EngineError::SyncError("Failed to acquire write lock on parent context".to_string()))?;
                parent_guard.add_child(context.id())?;
            }
        }
        
        // Store the context
        self.storage.store(context)
    }
    
    /// Create a child context
    pub fn create_child_context(
        &self,
        parent_id: &str,
    ) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        let parent_ctx = self.storage.get(parent_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Parent context not found: {}", parent_id)))?;
        
        let parent_guard = parent_ctx.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on parent context".to_string()))?;
        
        let trace_id = parent_guard.trace_id().cloned();
            
        let invocation_data = InvocationData {
            timestamp: Utc::now().timestamp(),
            trace_id: trace_id.as_ref().map(|t| t.as_str().to_string()).unwrap_or_default(),
            parent_id: Some(parent_id.to_string()),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        let content_id = invocation_data.content_id().map_err(|e| 
            EngineError::InternalError(format!("Failed to generate content ID: {}", e))
        )?;
        
        let invocation_id = format!("invocation:{}", content_id);
        
        let child_context = parent_guard.create_child(invocation_id);
        
        {
            let mut parent_guard = parent_ctx.write().map_err(|_| 
                EngineError::SyncError("Failed to acquire write lock on parent context".to_string()))?;
            
            parent_guard.add_child(child_context.id())?;
        }
        
        self.storage.store(child_context)
    }
    
    /// Create a child context with a custom ID (for testing)
    pub fn create_child_context_with_id(
        &self,
        parent_id: &str,
        child_id: String,
    ) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        let parent_ctx = self.storage.get(parent_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Parent context not found: {}", parent_id)))?;
        
        let parent_guard = parent_ctx.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on parent context".to_string()))?;
        
        let trace_id = parent_guard.trace_id().cloned();
        let child_context = parent_guard.create_child(child_id);
        
        {
            let mut parent_guard = parent_ctx.write().map_err(|_| 
                EngineError::SyncError("Failed to acquire write lock on parent context".to_string()))?;
            
            parent_guard.add_child(child_context.id())?;
        }
        
        self.storage.store(child_context)
    }
    
    /// Start an invocation
    pub fn start_context(&self, invocation_id: &str) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.start()?;
        
        Ok(())
    }
    
    /// Complete an invocation
    pub fn complete_context(&self, invocation_id: &str) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.complete()?;
        
        Ok(())
    }
    
    /// Fail an invocation
    pub fn fail_context(&self, invocation_id: &str, reason: &str) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.fail(reason)?;
        
        Ok(())
    }
    
    /// Wait for a resource
    pub fn wait_for_resource(&self, invocation_id: &str, resource_id: ContentId) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.wait_for_resource(resource_id)?;
        
        Ok(())
    }
    
    /// Wait for a fact
    pub fn wait_for_fact(&self, invocation_id: &str, fact_key: &str) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.wait_for_fact(fact_key)?;
        
        Ok(())
    }
    
    /// Resume a context that is waiting
    pub fn resume_context(&self, invocation_id: &str) -> EngineResult<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut guard = context.write().map_err(|_| 
            EngineError::SyncError("Failed to acquire write lock on context".to_string()))?;
        
        guard.resume()?;
        
        Ok(())
    }
    
    /// Get all contexts for a trace
    pub fn get_trace_contexts(&self, trace_id: &TraceId) -> EngineResult<Vec<Arc<RwLock<InvocationContext>>>> {
        self.storage.get_by_trace(trace_id)
    }
    
    /// Check if a trace is completely done (all invocations are in a final state)
    pub fn is_trace_complete(&self, trace_id: &TraceId) -> EngineResult<bool> {
        let contexts = self.storage.get_by_trace(trace_id)?;
        
        if contexts.is_empty() {
            return Ok(true);
        }
        
        for ctx in &contexts {
            let guard = ctx.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock on context".to_string()))?;
            
            if !guard.is_final() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Export a context for transfer to another process
    pub fn export_context(&self, invocation_id: &str) -> EngineResult<SerializedContext> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on context".to_string()))?;
        
        let time_map_bytes = bincode::serialize(guard.time_map())
            .map_err(|e| EngineError::SerializationFailed(format!("Failed to serialize time map: {}", e)))?;
        
        let facts_json = serde_json::to_string(guard.observed_facts())
            .map_err(|e| EngineError::SerializationFailed(format!("Failed to serialize facts: {}", e)))?;
        
        let metadata = serde_json::to_string(guard.metadata())
            .map_err(|e| EngineError::SerializationFailed(format!("Failed to serialize metadata: {}", e)))?;
        
        Ok(SerializedContext {
            invocation_id: guard.id().to_string(),
            trace_id: guard.trace_id().map(|t| t.as_str().to_string()).unwrap_or_default(),
            parent_id: guard.parent_id().map(|s| s.to_string()),
            state: format!("{:?}", guard.state()),
            time_map_bytes,
            created_at: Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            facts_json,
            metadata,
        })
    }
    
    /// Import a context from another process
    pub fn import_context(&self, serialized: SerializedContext) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        let time_map = bincode::deserialize(&serialized.time_map_bytes)
            .map_err(|e| EngineError::DeserializationFailed(format!("Failed to deserialize time map: {}", e)))?;
        
        let mut context = InvocationContext::new(
            serialized.invocation_id,
            Some(TraceId::from_str(&serialized.trace_id)),
            serialized.parent_id,
            time_map,
        );
        
        let facts: HashMap<String, serde_json::Value> = serde_json::from_str(&serialized.facts_json)
            .map_err(|e| EngineError::DeserializationFailed(format!("Failed to deserialize facts: {}", e)))?;
        
        for (key, value) in facts {
            context.add_fact(&key, value);
        }
        
        let metadata: HashMap<String, serde_json::Value> = serde_json::from_str(&serialized.metadata)
            .map_err(|e| EngineError::DeserializationFailed(format!("Failed to deserialize metadata: {}", e)))?;
        
        for (key, value) in metadata {
            context.add_metadata(&key, value);
        }
        
        self.storage.store(context)
    }

    /// Execute an action with a context from another process
    pub fn with_imported_context<F, R>(&self, serialized: SerializedContext, f: F) -> EngineResult<R>
    where
        F: FnOnce(&Arc<RwLock<InvocationContext>>) -> EngineResult<R>,
    {
        let context = self.import_context(serialized)?;
        let result = f(&context);
        
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
pub fn set_current_context(invocation_id: Option<String>) -> EngineResult<()> {
    CURRENT_CONTEXT.with(|ctx| {
        let mut current = ctx.lock().map_err(|_| 
            EngineError::SyncError("Failed to acquire lock on current context".to_string()))?;
        
        *current = invocation_id;
        
        Ok(())
    })
}

/// Get the current context for this thread
pub fn get_current_context() -> EngineResult<Option<String>> {
    CURRENT_CONTEXT.with(|ctx| {
        let current = ctx.lock().map_err(|_| 
            EngineError::SyncError("Failed to acquire lock on current context".to_string()))?;
        
        Ok(current.clone())
    })
}

/// Execute a function within the scope of a context
pub fn with_context<F, R>(
    propagator: &ContextPropagator,
    invocation_id: &str,
    f: F
) -> EngineResult<R>
where
    F: FnOnce() -> EngineResult<R>,
{
    let previous = get_current_context()?;
    
    set_current_context(Some(invocation_id.to_string()))?;
    
    let maybe_start = {
        let context = propagator.storage.get(invocation_id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock on context".to_string()))?;
        
        !guard.is_active()
    };
    
    if maybe_start {
        propagator.start_context(invocation_id)?;
    }
    
    let result = f();
    
    if maybe_start && result.is_ok() {
        let _ = propagator.complete_context(invocation_id);
    } else if maybe_start && result.is_err() {
        if let Err(err) = &result {
            let _ = propagator.fail_context(invocation_id, &format!("{}", err));
        }
    }
    
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
    pub async fn send_context(&self, invocation_id: &str, endpoint_index: usize) -> EngineResult<()> {
        let endpoint = self.endpoints.get(endpoint_index)
            .ok_or_else(|| EngineError::InvalidArgument(format!("Invalid endpoint index: {}", endpoint_index)))?;
        
        let serialized = self.propagator.export_context(invocation_id)?;
        
        let json = serde_json::to_string(&serialized)
            .map_err(|e| EngineError::SerializationFailed(format!("Failed to serialize context: {}", e)))?;
        
        let client = reqwest::Client::new();
        let mut request = client.post(endpoint)
            .header("Content-Type", "application/json");
        
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.body(json).send().await
            .map_err(|e| EngineError::Other(format!("Failed to send context: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(EngineError::Other(format!(
                "Failed to send context: HTTP {}", response.status()
            )));
        }
        
        Ok(())
    }
    
    /// Receive a context from a remote endpoint
    pub async fn receive_context(&self, endpoint_index: usize) -> EngineResult<Arc<RwLock<InvocationContext>>> {
        let endpoint = self.endpoints.get(endpoint_index)
            .ok_or_else(|| EngineError::InvalidArgument(format!("Invalid endpoint index: {}", endpoint_index)))?;
        
        let client = reqwest::Client::new();
        let mut request = client.get(endpoint);
        
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await
            .map_err(|e| EngineError::Other(format!("Failed to receive context: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(EngineError::Other(format!(
                "Failed to receive context: HTTP {}", response.status()
            )));
        }
        
        let serialized: SerializedContext = response.json().await
            .map_err(|e| EngineError::DeserializationFailed(format!("Failed to parse response: {}", e)))?;
        
        self.propagator.import_context(serialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_time_map() -> TimeMap {
        TimeMap::new()
    }
    
    #[test]
    fn test_context_storage() -> EngineResult<()> {
        let storage = ContextStorage::new();
        
        let trace_id = TraceId::from_str("test-trace-1");
        let time_map = create_time_map();
        
        let context = InvocationContext::new(
            "test-invocation-1".to_string(),
            Some(trace_id.clone()),
            None,
            time_map,
        );
        
        let ctx_ref = storage.store(context)?;
        
        {
            let guard = ctx_ref.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock".to_string()))?;
            
            assert_eq!(guard.id(), "test-invocation-1");
            assert_eq!(guard.trace_id(), Some(&trace_id));
        }
        
        let found = storage.get("test-invocation-1")?;
        assert!(found.is_some());
        
        let not_found = storage.get("nonexistent")?;
        assert!(not_found.is_none());
        
        let by_trace = storage.get_by_trace(&trace_id)?;
        assert_eq!(by_trace.len(), 1);
        
        assert_eq!(storage.count()?, 1);
        
        storage.clear()?;
        assert_eq!(storage.count()?, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_context_propagator() -> EngineResult<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        
        let trace_id = TraceId::from_str("test-trace-1");
        let time_map = create_time_map();
        
        let ctx = propagator.create_context(
            Some(trace_id.clone()),
            None,
            time_map,
        )?;
        
        let id = {
            let guard = ctx.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock".to_string()))?;
            guard.id().to_string()
        };
        
        propagator.start_context(&id)?;
        
        let child_id = "child-context";
        let child_ctx = propagator.create_child_context_with_id(&id, child_id.to_string())?;
        
        propagator.start_context(&child_id)?;
        
        let resource_id = ContentId::new("test-resource-id");
        propagator.wait_for_resource(&child_id, resource_id.clone())?;
        
        {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock".to_string()))?;
            assert!(matches!(guard.state(), InvocationState::Waiting));
        }
        
        propagator.resume_context(&child_id)?;
        
        {
            let guard = child_ctx.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock".to_string()))?;
            assert!(matches!(guard.state(), InvocationState::Running));
        }
        
        propagator.complete_context(&child_id)?;
        
        let by_trace = propagator.get_trace_contexts(&trace_id)?;
        assert_eq!(by_trace.len(), 2);
        
        assert!(propagator.is_trace_complete(&trace_id)?);
        
        Ok(())
    }
    
    #[test]
    fn test_thread_local_context() -> EngineResult<()> {
        set_current_context(Some("test-thread-context".to_string()))?;
        
        let current = get_current_context()?;
        assert_eq!(current, Some("test-thread-context".to_string()));
        
        set_current_context(None)?;
        
        let current = get_current_context()?;
        assert_eq!(current, None);
        
        Ok(())
    }
    
    #[test]
    fn test_with_context() -> EngineResult<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        
        let trace_id = TraceId::from_str("test-trace-3");
        let time_map = create_time_map();
        
        let ctx = propagator.create_context(
            Some(trace_id.clone()),
            None,
            time_map,
        )?;
        
        let id = {
            let guard = ctx.read().map_err(|_| 
                EngineError::SyncError("Failed to acquire read lock".to_string()))?;
            guard.id().to_string()
        };
        
        let result = with_context(&propagator, &id, || {
            let current = get_current_context()?;
            
            assert_eq!(current, Some(id.clone()));
            
            Ok::<_, EngineError>(42)
        })?;
        
        assert_eq!(result, 42);
        
        let context = propagator.storage.get(&id)?
            .ok_or_else(|| EngineError::NotFound(format!("Context not found: {}", id)))?;
        
        let guard = context.read().map_err(|_| 
            EngineError::SyncError("Failed to acquire read lock".to_string()))?;
        
        assert_eq!(guard.id(), id);
        
        Ok(())
    }
} 
