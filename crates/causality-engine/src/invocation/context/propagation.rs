use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use causality_error::{Error, Result};
use causality_types::{ContentId, TraceId};
use causality_core::time::TimeMap;

use crate::execution::context::ExecutionContext;
use crate::invocation::context::{InvocationContext, InvocationContextTrait, BasicContext};

/// Represents context storage for invocation contexts
pub trait ContextStorage: Send + Sync {
    /// Store a context
    fn store_context(&self, context_id: String, data: HashMap<String, String>) -> Result<()>;
    
    /// Retrieve a context
    fn retrieve_context(&self, context_id: &str) -> Result<Option<HashMap<String, String>>>;
    
    /// Delete a context
    fn delete_context(&self, context_id: &str) -> Result<()>;
    
    /// List all contexts
    fn list_contexts(&self) -> Result<Vec<String>>;
}

/// In-memory implementation of ContextStorage
pub struct MemoryContextStorage {
    contexts: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl MemoryContextStorage {
    /// Create a new memory context storage
    pub fn new() -> Self {
        MemoryContextStorage {
            contexts: RwLock::new(HashMap::new()),
        }
    }
}

impl ContextStorage for MemoryContextStorage {
    fn store_context(&self, context_id: String, data: HashMap<String, String>) -> Result<()> {
        let mut contexts = self.contexts.write().map_err(|_| {
            Error::InternalError("Failed to acquire write lock on contexts".to_string())
        })?;
        contexts.insert(context_id, data);
        Ok(())
    }
    
    fn retrieve_context(&self, context_id: &str) -> Result<Option<HashMap<String, String>>> {
        let contexts = self.contexts.read().map_err(|_| {
            Error::InternalError("Failed to acquire read lock on contexts".to_string())
        })?;
        Ok(contexts.get(context_id).cloned())
    }
    
    fn delete_context(&self, context_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().map_err(|_| {
            Error::InternalError("Failed to acquire write lock on contexts".to_string())
        })?;
        contexts.remove(context_id);
        Ok(())
    }
    
    fn list_contexts(&self) -> Result<Vec<String>> {
        let contexts = self.contexts.read().map_err(|_| {
            Error::InternalError("Failed to acquire read lock on contexts".to_string())
        })?;
        Ok(contexts.keys().cloned().collect())
    }
}

/// Context propagator manages execution contexts across invocations
pub struct ContextPropagator {
    storage: Arc<dyn ContextStorage>,
}

impl ContextPropagator {
    /// Create a new context propagator
    pub fn new(storage: Arc<dyn ContextStorage>) -> Self {
        ContextPropagator {
            storage,
        }
    }
    
    /// Create a new context with the given trace ID
    pub fn create_context(&self, trace_id: Option<TraceId>, parent_id: Option<String>, time_map: TimeMap) -> Result<InvocationContext> {
        // Generate a unique ID for the context
        let id = format!("ctx:{}", uuid::Uuid::new_v4());
        
        // Create a new invocation context
        let context = InvocationContext::new(id, trace_id, parent_id, time_map);
        
        // Store context data in storage
        let context_data = HashMap::new();
        self.storage.store_context(context.id().to_string(), context_data)?;
        
        Ok(context)
    }
    
    /// Get a context by ID
    pub fn get_context(&self, context_id: &str) -> Result<Option<InvocationContext>> {
        // Try to retrieve the context data
        let context_data = self.storage.retrieve_context(context_id)?;
        
        if let Some(data) = context_data {
            // Reconstruct the context from data
            // This is a simplified version - in a real implementation we'd deserialize the full context
            let time_map = TimeMap::new(); // Would be loaded from data
            let trace_id = None; // Would be loaded from data
            let parent_id = data.get("parent_id").cloned();
            
            let mut context = InvocationContext::new(
                context_id.to_string(),
                trace_id,
                parent_id,
                time_map
            );
            
            // Load additional data
            for (key, value) in data {
                if key != "parent_id" {
                    context.set(key, value);
                }
            }
            
            Ok(Some(context))
        } else {
            Ok(None)
        }
    }
    
    /// Delete a context
    pub fn delete_context(&self, context_id: &str) -> Result<()> {
        self.storage.delete_context(context_id)
    }
    
    /// Create a new execution context for an invocation
    pub fn create_execution_context(&self, invocation_context: &InvocationContext) -> Result<Arc<dyn InvocationContextTrait>> {
        // Create a basic context with the same ID
        let basic_context = BasicContext::new(invocation_context.id())
            .with_capability("basic.execution");
        
        if let Some(domain_id) = invocation_context.get("domain_id") {
            // If there's a domain ID in the context data, use it
            Ok(Arc::new(basic_context.with_domain(domain_id.to_string())))
        } else {
            // Otherwise just return the basic context
            Ok(Arc::new(basic_context))
        }
    }
} 