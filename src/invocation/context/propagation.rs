// Context propagation module for invocation contexts
//
// This module provides functionality for propagating invocation contexts
// between programs, allowing execution state to flow through invocations.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::TraceId;
use crate::domain::map::map::TimeMap;
use super::InvocationContext;

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
            Error::InternalError("Failed to acquire write lock on contexts".to_string()))?;
        
        contexts.insert(context_id, arc_context.clone());
        
        Ok(arc_context)
    }
    
    /// Retrieve a context by ID
    pub fn get(&self, invocation_id: &str) -> Result<Option<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            Error::InternalError("Failed to acquire read lock on contexts".to_string()))?;
        
        let context = contexts.get(invocation_id).cloned();
        
        Ok(context)
    }
    
    /// Remove a context
    pub fn remove(&self, invocation_id: &str) -> Result<Option<Arc<RwLock<InvocationContext>>>> {
        let mut contexts = self.contexts.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on contexts".to_string()))?;
        
        let context = contexts.remove(invocation_id);
        
        Ok(context)
    }
    
    /// Get all contexts for a trace
    pub fn get_by_trace(&self, trace_id: &TraceId) -> Result<Vec<Arc<RwLock<InvocationContext>>>> {
        let contexts = self.contexts.read().map_err(|_| 
            Error::InternalError("Failed to acquire read lock on contexts".to_string()))?;
        
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
            Error::InternalError("Failed to acquire read lock on contexts".to_string()))?;
        
        Ok(contexts.len())
    }
    
    /// Clear all contexts
    pub fn clear(&self) -> Result<()> {
        let mut contexts = self.contexts.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on contexts".to_string()))?;
        
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
        // Generate a new invocation ID
        let invocation_id = Uuid::new_v4().to_string();
        
        // Use provided trace ID or create a new one
        let trace_id = trace_id.unwrap_or_else(TraceId::new);
        
        // Create the context
        let mut context = InvocationContext::new(
            invocation_id,
            trace_id,
            time_map,
        );
        
        // Set parent ID if provided
        if let Some(parent) = parent_id {
            context.parent_id = Some(parent);
            
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
        
        // Generate a new invocation ID
        let invocation_id = Uuid::new_v4().to_string();
        
        // Create the child context
        let child_context = {
            let parent_guard = parent_ctx.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on parent context".to_string()))?;
            
            parent_guard.create_child(invocation_id)
        };
        
        // Add this as a child to the parent
        {
            let mut parent_guard = parent_ctx.write().map_err(|_| 
                Error::InternalError("Failed to acquire write lock on parent context".to_string()))?;
            
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
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.start()
    }
    
    /// Complete a context
    pub fn complete_context(&self, invocation_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.complete()
    }
    
    /// Fail a context with a reason
    pub fn fail_context(&self, invocation_id: &str, reason: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.fail(reason)
    }
    
    /// Wait for a resource
    pub fn wait_for_resource(&self, invocation_id: &str, resource_id: crate::types::ResourceId) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.wait_for_resource(resource_id)
    }
    
    /// Wait for a fact
    pub fn wait_for_fact(&self, invocation_id: &str, fact_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.wait_for_fact(fact_id)
    }
    
    /// Resume a context
    pub fn resume_context(&self, invocation_id: &str) -> Result<()> {
        let context = self.storage.get(invocation_id)?
            .ok_or_else(|| Error::NotFound(format!("Context not found: {}", invocation_id)))?;
        
        let mut ctx_guard = context.write().map_err(|_| 
            Error::InternalError("Failed to acquire write lock on context".to_string()))?;
        
        ctx_guard.resume()
    }
    
    /// Get all active contexts in a trace
    pub fn get_trace_contexts(&self, trace_id: &TraceId) -> Result<Vec<Arc<RwLock<InvocationContext>>>> {
        self.storage.get_by_trace(trace_id)
    }
    
    /// Check if all contexts in a trace are final
    pub fn is_trace_complete(&self, trace_id: &TraceId) -> Result<bool> {
        let contexts = self.storage.get_by_trace(trace_id)?;
        
        if contexts.is_empty() {
            return Ok(false);
        }
        
        for ctx in &contexts {
            let guard = ctx.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on context".to_string()))?;
            
            if !guard.is_final() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Thread-local context for the current invocation
thread_local! {
    static CURRENT_CONTEXT: Mutex<Option<String>> = Mutex::new(None);
}

/// Set the current invocation context for this thread
pub fn set_current_context(invocation_id: Option<String>) -> Result<()> {
    CURRENT_CONTEXT.with(|current| {
        let mut current = current.lock().map_err(|_| 
            Error::InternalError("Failed to acquire lock on thread-local context".to_string()))?;
        
        *current = invocation_id;
        
        Ok(())
    })
}

/// Get the current invocation context for this thread
pub fn get_current_context() -> Result<Option<String>> {
    CURRENT_CONTEXT.with(|current| {
        let current = current.lock().map_err(|_| 
            Error::InternalError("Failed to acquire lock on thread-local context".to_string()))?;
        
        Ok(current.clone())
    })
}

/// Run a function with a specific invocation context
pub fn with_context<F, R>(
    propagator: &ContextPropagator,
    invocation_id: &str,
    f: F
) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    // Save the previous context
    let previous_context = get_current_context()?;
    
    // Set the new context
    set_current_context(Some(invocation_id.to_string()))?;
    
    // Run the function
    let result = f();
    
    // Restore the previous context
    set_current_context(previous_context)?;
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ResourceId, DomainId};
    
    fn create_time_map() -> TimeMap {
        TimeMap::new()
    }
    
    #[test]
    fn test_context_storage() -> Result<()> {
        let storage = ContextStorage::new();
        let time_map = create_time_map();
        let trace_id = TraceId::new();
        let invocation_id = "test_invocation".to_string();
        
        // Create a context
        let context = InvocationContext::new(
            invocation_id.clone(),
            trace_id.clone(),
            time_map,
        );
        
        // Store the context
        let arc_context = storage.store(context)?;
        
        // Count should be 1
        assert_eq!(storage.count()?, 1);
        
        // Retrieve by ID
        let retrieved = storage.get(&invocation_id)?;
        assert!(retrieved.is_some());
        
        // Retrieve by trace
        let trace_contexts = storage.get_by_trace(&trace_id)?;
        assert_eq!(trace_contexts.len(), 1);
        
        // Remove the context
        let removed = storage.remove(&invocation_id)?;
        assert!(removed.is_some());
        
        // Count should be 0
        assert_eq!(storage.count()?, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_context_propagator() -> Result<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        let time_map = create_time_map();
        let trace_id = TraceId::new();
        
        // Create a root context
        let root_context = propagator.create_context(
            Some(trace_id.clone()),
            None,
            time_map.clone(),
        )?;
        
        let root_id = {
            let root_guard = root_context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on root context".to_string()))?;
            
            root_guard.invocation_id.clone()
        };
        
        // Start the root context
        propagator.start_context(&root_id)?;
        
        // Create a child context
        let child_context = propagator.create_child_context(&root_id)?;
        
        let child_id = {
            let child_guard = child_context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on child context".to_string()))?;
            
            child_guard.invocation_id.clone()
        };
        
        // Check parent-child relationship
        {
            let child_guard = child_context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on child context".to_string()))?;
            
            assert_eq!(child_guard.parent_id, Some(root_id.clone()));
            assert_eq!(child_guard.trace_id, trace_id);
        }
        
        {
            let root_guard = root_context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on root context".to_string()))?;
            
            assert!(root_guard.children.contains(&child_id));
        }
        
        // Start and complete the child
        propagator.start_context(&child_id)?;
        propagator.complete_context(&child_id)?;
        
        // Fail the root context
        propagator.fail_context(&root_id, "Test failure")?;
        
        // Check if trace is complete
        assert!(propagator.is_trace_complete(&trace_id)?);
        
        Ok(())
    }
    
    #[test]
    fn test_thread_local_context() -> Result<()> {
        // No context initially
        assert_eq!(get_current_context()?, None);
        
        // Set a context
        set_current_context(Some("test_context".to_string()))?;
        assert_eq!(get_current_context()?, Some("test_context".to_string()));
        
        // Clear the context
        set_current_context(None)?;
        assert_eq!(get_current_context()?, None);
        
        Ok(())
    }
    
    #[test]
    fn test_with_context() -> Result<()> {
        let storage = Arc::new(ContextStorage::new());
        let propagator = ContextPropagator::new(storage.clone());
        let time_map = create_time_map();
        
        // Create a context
        let context = propagator.create_context(
            None,
            None,
            time_map.clone(),
        )?;
        
        let invocation_id = {
            let guard = context.read().map_err(|_| 
                Error::InternalError("Failed to acquire read lock on context".to_string()))?;
            
            guard.invocation_id.clone()
        };
        
        // Run with the context
        let result = with_context(&propagator, &invocation_id, || {
            // Get the current context
            let current = get_current_context()?;
            
            // Should match the invocation ID
            assert_eq!(current, Some(invocation_id.clone()));
            
            Ok("success")
        })?;
        
        assert_eq!(result, "success");
        
        // After the function, there should be no current context
        assert_eq!(get_current_context()?, None);
        
        Ok(())
    }
} 