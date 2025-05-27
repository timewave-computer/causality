// Invocation context for effect patterns
// Original file: src/invocation/context.rs

use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use causality_types::{ContentId, DomainId, TraceId};
use causality_core::time::TimeMap;
use causality_error::EngineError;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

// Import EngineResult and other error types
use causality_error::{EngineResult, CausalityError};

// Import ExecutionEvent from execution context

// Ensure correct import path for ResourceId

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

/// State of an invocation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationState {
    /// The invocation has been created but not started
    Created,
    /// The invocation is running
    Running,
    /// The invocation has completed successfully
    Completed,
    /// The invocation has failed
    Failed,
    /// The invocation has been canceled
    Canceled,
    /// The invocation is waiting for a resource
    Waiting,
    /// The invocation is waiting for a fact
    WaitingForFact,
}

/// Invocation context
#[derive(Clone)]
pub struct InvocationContext {
    /// Invocation ID
    id: String,
    /// Trace ID
    trace_id: Option<TraceId>,
    /// Parent invocation ID
    parent_id: Option<String>,
    /// Execution context ID
    execution_context_id: Option<String>,
    /// Time map for tracking progress
    time_map: TimeMap,
    /// Context data
    data: HashMap<String, serde_json::Value>,
    /// Invocation state
    state: InvocationState,
    /// Child invocation IDs
    children: Vec<String>,
    /// Observed facts
    observed_facts: HashMap<String, serde_json::Value>,
    /// Metadata
    metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Start timestamp
    started_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    completed_at: Option<DateTime<Utc>>,
}

impl InvocationContext {
    /// Create a new invocation context
    pub fn new(
        id: String,
        trace_id: Option<TraceId>,
        parent_id: Option<String>,
        time_map: TimeMap,
    ) -> Self {
        InvocationContext {
            id,
            trace_id,
            parent_id,
            execution_context_id: None,
            time_map,
            data: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            state: InvocationState::Created,
            children: Vec::new(),
            observed_facts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get the invocation ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> Option<&TraceId> {
        self.trace_id.as_ref()
    }

    /// Get the parent ID
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_ref().map(|id| id.as_str())
    }

    /// Get the execution context ID
    pub fn execution_context_id(&self) -> Option<&str> {
        self.execution_context_id.as_ref().map(|id| id.as_str())
    }

    /// Set the execution context ID
    pub fn set_execution_context_id(&mut self, id: String) {
        self.execution_context_id = Some(id);
    }

    /// Get a value from the context
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|value| {
            serde_json::from_value(value.clone()).ok()
        })
    }

    /// Set a value in the context
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> EngineResult<()> {
        let value = serde_json::to_value(value).map_err(|e| 
            EngineError::SerializationFailed(format!("Failed to serialize value: {}", e)))?;
        self.data.insert(key.to_string(), value);
        Ok(())
    }

    /// Create a child context
    pub fn create_child(&self, id: String) -> Self {
        let trace_id = self.trace_id.clone();
        let parent_id = Some(self.id.clone());
        let time_map = self.time_map.clone();
        
        InvocationContext::new(id, trace_id, parent_id, time_map)
    }

    /// Check if the invocation is in an active state
    pub fn is_active(&self) -> bool {
        matches!(self.state, InvocationState::Running | InvocationState::Waiting | InvocationState::WaitingForFact)
    }

    /// Check if the invocation is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self.state, InvocationState::Completed | InvocationState::Failed | InvocationState::Canceled)
    }

    /// Start the invocation
    pub fn start(&mut self) -> EngineResult<()> {
        if matches!(self.state, InvocationState::Created) {
            self.state = InvocationState::Running;
            self.started_at = Some(Utc::now());
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot start invocation in state {:?}", self.state)))
        }
    }

    /// Mark the invocation as complete
    pub fn complete(&mut self) -> EngineResult<()> {
        if self.is_active() {
            self.state = InvocationState::Completed;
            self.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot complete invocation in state {:?}", self.state)))
        }
    }

    /// Mark the invocation as failed
    pub fn fail(&mut self, reason: &str) -> EngineResult<()> {
        if self.is_active() {
            self.state = InvocationState::Failed;
            self.completed_at = Some(Utc::now());
            self.metadata.insert("failure_reason".to_string(), 
                serde_json::Value::String(reason.to_string()));
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot fail invocation in state {:?}", self.state)))
        }
    }

    /// Mark the invocation as waiting for a resource
    pub fn wait_for_resource(&mut self, _resource_id: ContentId) -> EngineResult<()> {
        if self.is_active() {
            self.state = InvocationState::Waiting;
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot wait in state {:?}", self.state)))
        }
    }

    /// Mark the invocation as waiting for a fact
    pub fn wait_for_fact(&mut self, _fact_key: &str) -> EngineResult<()> {
        if self.is_active() {
            self.state = InvocationState::WaitingForFact;
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot wait for fact in state {:?}", self.state)))
        }
    }

    /// Resume a waiting invocation
    pub fn resume(&mut self) -> EngineResult<()> {
        if matches!(self.state, InvocationState::Waiting | InvocationState::WaitingForFact) {
            self.state = InvocationState::Running;
            Ok(())
        } else {
            Err(EngineError::ContextError(format!("Cannot resume invocation in state {:?}", self.state)))
        }
    }

    /// Add a child invocation ID
    pub fn add_child(&mut self, child_id: &str) -> EngineResult<()> {
        self.children.push(child_id.to_string());
        Ok(())
    }

    /// Get the invocation state
    pub fn state(&self) -> &InvocationState {
        &self.state
    }

    /// Get the time map
    pub fn time_map(&self) -> &TimeMap {
        &self.time_map
    }

    /// Get the observed facts
    pub fn observed_facts(&self) -> &HashMap<String, serde_json::Value> {
        &self.observed_facts
    }

    /// Add a fact to the context
    pub fn add_fact(&mut self, key: &str, value: serde_json::Value) {
        self.observed_facts.insert(key.to_string(), value);
    }

    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }

    /// Add metadata to the context
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Get the children invocation IDs
    pub fn children(&self) -> &[String] {
        &self.children
    }
}

impl fmt::Debug for InvocationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvocationContext")
            .field("id", &self.id)
            .field("trace_id", &self.trace_id)
            .field("parent_id", &self.parent_id)
            .field("state", &self.state)
            .field("execution_context_id", &self.execution_context_id)
            .field("children", &self.children)
            .field("observed_facts", &self.observed_facts.len())
            .field("metadata", &self.metadata.len())
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
        let ctx = PhysicalContext::new("test-context", DomainId::from("domain1"))
            .with_capability("fs.read")
            .with_capability("network.connect");
            
        assert!(ctx.has_capability("fs.read"));
        assert!(ctx.has_capability("network.connect"));
        assert!(!ctx.has_capability("fs.write"));
    }
} 
