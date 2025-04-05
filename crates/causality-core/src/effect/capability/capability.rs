use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::any::Any;
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::effect::types::EffectId;
use crate::resource::ResourceId;
use crate::effect::context::Capability as ContextCapability;
use crate::effect::EffectContext;

/// A capability represents a permission to perform a specific operation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capability {
    /// Name of the capability
    name: String,
}

impl Capability {
    /// Creates a new capability with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    
    /// Returns the name of this capability
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Constraints on a capability
#[derive(Clone, Debug)]
pub struct CapabilityConstraints {
    /// Time-based constraints (e.g., expiration)
    time_constraints: Option<TimeConstraints>,
    /// Scope-based constraints (limiting to specific resources)
    scope_constraints: Option<ScopeConstraints>,
}

impl CapabilityConstraints {
    /// Creates a new empty set of constraints
    pub fn new() -> Self {
        Self {
            time_constraints: None,
            scope_constraints: None,
        }
    }
    
    /// Adds time constraints to this set
    pub fn with_time_constraints(mut self, constraints: TimeConstraints) -> Self {
        self.time_constraints = Some(constraints);
        self
    }
    
    /// Adds scope constraints to this set
    pub fn with_scope_constraints(mut self, constraints: ScopeConstraints) -> Self {
        self.scope_constraints = Some(constraints);
        self
    }
    
    /// Returns any time constraints
    pub fn time_constraints(&self) -> Option<&TimeConstraints> {
        self.time_constraints.as_ref()
    }
    
    /// Returns any scope constraints
    pub fn scope_constraints(&self) -> Option<&ScopeConstraints> {
        self.scope_constraints.as_ref()
    }
}

/// Time-based constraints on a capability
#[derive(Clone, Debug)]
pub struct TimeConstraints {
    /// When this capability expires (if any)
    expires_at: Option<u64>,
    /// How many times this capability can be used (if any)
    max_uses: Option<usize>,
}

impl TimeConstraints {
    /// Creates a new set of time constraints
    pub fn new() -> Self {
        Self {
            expires_at: None,
            max_uses: None,
        }
    }
    
    /// Sets an expiration time for this capability
    pub fn with_expiration(mut self, time: u64) -> Self {
        self.expires_at = Some(time);
        self
    }
    
    /// Sets a usage limit for this capability
    pub fn with_max_uses(mut self, uses: usize) -> Self {
        self.max_uses = Some(uses);
        self
    }
    
    /// Returns the expiration time (if any)
    pub fn expires_at(&self) -> Option<u64> {
        self.expires_at
    }
    
    /// Returns the usage limit (if any)
    pub fn max_uses(&self) -> Option<usize> {
        self.max_uses
    }
}

/// Scope-based constraints on a capability
#[derive(Clone, Debug)]
pub struct ScopeConstraints {
    /// Resource types this capability applies to
    resource_types: Option<Vec<String>>,
    /// Specific resource IDs this capability applies to
    resource_ids: Option<Vec<String>>,
    /// Additional scope parameters
    parameters: HashMap<String, String>,
}

impl ScopeConstraints {
    /// Creates a new set of scope constraints
    pub fn new() -> Self {
        Self {
            resource_types: None,
            resource_ids: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Limits this capability to specific resource types
    pub fn with_resource_types(mut self, types: Vec<String>) -> Self {
        self.resource_types = Some(types);
        self
    }
    
    /// Limits this capability to specific resource IDs
    pub fn with_resource_ids(mut self, ids: Vec<String>) -> Self {
        self.resource_ids = Some(ids);
        self
    }
    
    /// Adds a parameter to these constraints
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Returns the resource types (if any)
    pub fn resource_types(&self) -> Option<&Vec<String>> {
        self.resource_types.as_ref()
    }
    
    /// Returns the resource IDs (if any)
    pub fn resource_ids(&self) -> Option<&Vec<String>> {
        self.resource_ids.as_ref()
    }
    
    /// Returns the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

/// A default implementation of the effect context based on capabilities
#[derive(Debug)]
pub struct CapabilityContext {
    /// The capabilities this context has
    capabilities: Vec<Capability>,
    /// Data available in this context
    data: HashMap<String, String>,
    /// Metadata for this context
    metadata: HashMap<String, String>,
}

impl CapabilityContext {
    /// Creates a new capability context
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Adds a capability to this context
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Adds data to this context
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Adds metadata to this context
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Returns all capabilities in this context
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
}

// Helper function to get a placeholder EffectId
fn placeholder_effect_id() -> &'static EffectId {
    static PLACEHOLDER: OnceLock<EffectId> = OnceLock::new();
    PLACEHOLDER.get_or_init(|| EffectId::from_string("placeholder-id"))
}

impl crate::effect::EffectContext for CapabilityContext {
    fn effect_id(&self) -> &EffectId {
        // If we don't have a proper effect ID, use the placeholder
        placeholder_effect_id()
    }
    
    fn capabilities(&self) -> &[ContextCapability] {
        // This is a temporary solution - in a real implementation we would use 
        // the same capability type throughout the codebase
        static EMPTY_CAPABILITIES: Vec<ContextCapability> = Vec::new();
        &EMPTY_CAPABILITIES
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        // If we don't have resources, return an empty set
        // In a real implementation, this should be set properly when the context is created
        static EMPTY_RESOURCES: OnceLock<HashSet<ResourceId>> = OnceLock::new();
        EMPTY_RESOURCES.get_or_init(|| HashSet::new())
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        None // No parent context in this implementation
    }
    
    fn has_capability(&self, capability: &crate::effect::context::Capability) -> bool {
        // Convert from context::Capability to our local Capability
        // In a real implementation, capability models should be unified
        let name = capability.to_string();
        self.capabilities.iter().any(|cap| cap.name() == name)
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        // Create a new context with the same capabilities but a new effect ID
        let mut new_metadata = self.metadata.clone();
        new_metadata.insert("effect_id".to_string(), effect_id.as_str().to_string());
        
        Box::new(CapabilityContext {
            capabilities: self.capabilities.clone(),
            data: self.data.clone(),
            metadata: new_metadata,
        })
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<crate::effect::context::Capability>) -> Box<dyn EffectContext> {
        // Convert from context::Capability to our local Capability
        let new_capabilities = capabilities.iter()
            .map(|cap| Capability::new(&cap.to_string()))
            .collect::<Vec<_>>();
        
        let mut all_caps = self.capabilities.clone();
        all_caps.extend(new_capabilities);
        
        Box::new(CapabilityContext {
            capabilities: all_caps,
            data: self.data.clone(),
            metadata: self.metadata.clone(),
        })
    }
    
    fn with_additional_resources(&self, _resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        // In this simplified model, we don't track resources, just return a clone
        self.clone_context()
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_metadata = self.metadata.clone();
        new_metadata.extend(metadata);
        
        Box::new(CapabilityContext {
            capabilities: self.capabilities.clone(),
            data: self.data.clone(),
            metadata: new_metadata,
        })
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(CapabilityContext {
            capabilities: self.capabilities.clone(),
            data: self.data.clone(),
            metadata: self.metadata.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 