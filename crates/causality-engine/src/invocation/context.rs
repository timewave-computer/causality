// Invocation context for effect patterns
// Original file: src/invocation/context.rs

use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use causality_error::{Error, Result, EngineError};
use causality_types::{ContentId, DomainId, TraceId};
use causality_core::time::TimeMap;

// Import EngineResult and other error types
use causality_error::{EngineResult, CausalityError, Result as CausalityResult};

// Import ExecutionEvent from execution context
use crate::execution::context::ExecutionEvent;

/// Context trait for invocation contexts
pub trait InvocationContextTrait: Clone + fmt::Debug + Send + Sync + 'static {
    /// Get the context type
    fn context_type(&self) -> &str;
    
    /// Get the domain ID for this context (if any)
    fn domain_id(&self) -> Option<&DomainId>;
    
    /// Get a unique ID for this context
    fn context_id(&self) -> &str;
    
    /// Get the parent context ID (if any)
    fn parent_id(&self) -> Option<&str>;
    
    /// Clone this context with a new ID
    fn with_id(&self, id: &str) -> Self;
    
    /// Check if context has a capability
    fn has_capability(&self, capability: &str) -> bool;
    
    /// Get the security policy
    fn security_policy(&self) -> Arc<dyn SecurityPolicy>;
}

/// Security policy for controlling access to capabilities
pub trait SecurityPolicy: Send + Sync + 'static {
    /// Check if the given capability is allowed
    fn is_allowed(&self, capability: &str) -> bool;
    
    /// Get all allowed capabilities
    fn allowed_capabilities(&self) -> Vec<String>;
}

/// Basic implementation of an execution context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicContext {
    /// Unique ID for this context
    pub id: String,
    
    /// Parent context ID, if any
    pub parent_id: Option<String>,
    
    /// Domain ID, if any
    pub domain_id: Option<DomainId>,
    
    /// Working directory for local filesystem operations
    pub working_dir: Option<PathBuf>,
    
    /// Allowed capabilities
    pub capabilities: HashMap<String, bool>,
}

impl InvocationContextTrait for BasicContext {
    fn context_type(&self) -> &str {
        "basic"
    }
    
    fn domain_id(&self) -> Option<&DomainId> {
        self.domain_id.as_ref()
    }
    
    fn context_id(&self) -> &str {
        &self.id
    }
    
    fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_ref().map(|s| s.as_str())
    }
    
    fn with_id(&self, id: &str) -> Self {
        let mut new_context = self.clone();
        new_context.id = id.to_string();
        new_context
    }
    
    fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.get(capability).copied().unwrap_or(false)
    }
    
    fn security_policy(&self) -> Arc<dyn SecurityPolicy> {
        Arc::new(ContextSecurityPolicy::new(self.clone()))
    }
}

impl BasicContext {
    /// Create a new basic context
    pub fn new(id: &str) -> Self {
        BasicContext {
            id: id.to_string(),
            parent_id: None,
            domain_id: None,
            working_dir: None,
            capabilities: HashMap::new(),
        }
    }
    
    /// Set the parent context ID
    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }
    
    /// Set the domain ID
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Set the working directory
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.insert(capability.to_string(), true);
        self
    }
    
    /// Remove a capability
    pub fn without_capability(mut self, capability: &str) -> Self {
        self.capabilities.insert(capability.to_string(), false);
        self
    }
}

/// Security policy based on context capabilities
#[derive(Clone)]
pub struct ContextSecurityPolicy {
    context: BasicContext,
}

impl ContextSecurityPolicy {
    /// Create a new security policy from a context
    pub fn new(context: BasicContext) -> Self {
        ContextSecurityPolicy {
            context,
        }
    }
}

impl SecurityPolicy for ContextSecurityPolicy {
    fn is_allowed(&self, capability: &str) -> bool {
        self.context.has_capability(capability)
    }
    
    fn allowed_capabilities(&self) -> Vec<String> {
        self.context.capabilities.iter()
            .filter_map(|(cap, allowed)| if *allowed { Some(cap.clone()) } else { None })
            .collect()
    }
}

/// Physical context for interacting with the real world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicalContext {
    /// Base context
    pub base: BasicContext,
    
    /// Time map snapshot
    pub time_map: TimeMap,
}

impl InvocationContextTrait for PhysicalContext {
    fn context_type(&self) -> &str {
        "physical"
    }
    
    fn domain_id(&self) -> Option<&DomainId> {
        self.base.domain_id()
    }
    
    fn context_id(&self) -> &str {
        self.base.context_id()
    }
    
    fn parent_id(&self) -> Option<&str> {
        self.base.parent_id()
    }
    
    fn with_id(&self, id: &str) -> Self {
        let mut new_context = self.clone();
        new_context.base = self.base.with_id(id);
        new_context
    }
    
    fn has_capability(&self, capability: &str) -> bool {
        self.base.has_capability(capability)
    }
    
    fn security_policy(&self) -> Arc<dyn SecurityPolicy> {
        self.base.security_policy()
    }
}

impl PhysicalContext {
    /// Create a new physical context
    pub fn new(id: &str, domain_id: DomainId) -> Self {
        let base = BasicContext::new(id)
            .with_domain(domain_id);
        
        let time_map = TimeMap::new();
        
        PhysicalContext {
            base,
            time_map,
        }
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> EngineResult<&DomainId> {
        self.base.domain_id.as_ref().ok_or_else(|| 
            EngineError::InvalidArgument("Domain ID required for physical context".to_string())
        )
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.base = self.base.with_capability(capability);
        self
    }
    
    /// Remove a capability
    pub fn without_capability(mut self, capability: &str) -> Self {
        self.base = self.base.without_capability(capability);
        self
    }
    
    /// Set the parent context ID
    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.base = self.base.with_parent(parent_id);
        self
    }
    
    /// Set the time map
    pub fn with_time_map(mut self, time_map: TimeMap) -> Self {
        self.time_map = time_map;
        self
    }
}

/// Invocation state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvocationState {
    /// Invocation has been created but not started
    Created,
    /// Invocation is currently running
    Running,
    /// Invocation has completed successfully
    Completed,
    /// Invocation has failed
    Failed(String),
    /// Invocation has been canceled
    Canceled,
    /// Invocation is waiting for a resource
    Waiting(ContentId),
    /// Invocation is waiting for an external fact
    WaitingForFact(String),
}

/// Context for an invocation
#[derive(Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    /// Unique ID for this context
    id: String,
    
    /// Trace ID for tracking related invocations
    trace_id: Option<TraceId>,
    
    /// Parent context ID
    parent_id: Option<String>,
    
    /// Context ID for execution
    execution_context_id: Option<String>,
    
    /// Time map for tracking operation timing
    time_map: TimeMap,
    
    /// Additional context data
    data: HashMap<String, String>,
}

impl InvocationContext {
    /// Create a new invocation context
    pub fn new(
        id: impl Into<String>,
        trace_id: Option<TraceId>,
        parent_id: Option<String>,
        time_map: TimeMap,
    ) -> Self {
        InvocationContext {
            id: id.into(),
            trace_id,
            parent_id,
            execution_context_id: None,
            time_map,
            data: HashMap::new(),
        }
    }
    
    /// Get the context ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the trace ID
    pub fn trace_id(&self) -> Option<&TraceId> {
        self.trace_id.as_ref()
    }
    
    /// Get the parent context ID
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_ref().map(|s| s.as_str())
    }
    
    /// Get the execution context ID
    pub fn execution_context_id(&self) -> Option<&str> {
        self.execution_context_id.as_ref().map(|s| s.as_str())
    }
    
    /// Set the execution context ID
    pub fn set_execution_context_id(&mut self, id: String) {
        self.execution_context_id = Some(id);
    }
    
    /// Get the time map
    pub fn time_map(&self) -> &TimeMap {
        &self.time_map
    }
    
    /// Get a value from the context data
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
    
    /// Set a value in the context data
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }
    
    /// Get all context data
    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }
    
    /// Create a child context
    pub fn create_child(&self, id: impl Into<String>) -> Self {
        InvocationContext {
            id: id.into(),
            trace_id: self.trace_id.clone(),
            parent_id: Some(self.id.clone()),
            execution_context_id: None,
            time_map: self.time_map.clone(),
            data: HashMap::new(),
        }
    }
}

impl fmt::Debug for InvocationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvocationContext")
            .field("id", &self.id)
            .field("trace_id", &self.trace_id)
            .field("parent_id", &self.parent_id)
            .field("execution_context_id", &self.execution_context_id)
            .field("data_size", &self.data.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_context() {
        let ctx = BasicContext::new("test-context")
            .with_capability("fs.read")
            .with_capability("network.connect");
            
        assert!(ctx.has_capability("fs.read"));
        assert!(ctx.has_capability("network.connect"));
        assert!(!ctx.has_capability("fs.write"));
    }
    
    #[test]
    fn test_physical_context() {
        let ctx = PhysicalContext::new("test-context", "domain1".to_string())
            .with_capability("fs.read")
            .with_capability("network.connect");
            
        assert!(ctx.has_capability("fs.read"));
        assert!(ctx.has_capability("network.connect"));
        assert!(!ctx.has_capability("fs.write"));
    }
} 
