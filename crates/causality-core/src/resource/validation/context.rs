// Validation context module
// This file defines the context for resource validation operations.

use std::collections::HashMap;
use std::sync::Arc;

use crate::resource_types::{
    ResourceId, ResourceTypeId,
};
use crate::resource::ResourceState;
use crate::resource::validation::ResourcePermission;
use crate::resource::ResourceSchema;
use crate::capability::Capability;
use causality_types::domain::DomainId;
use causality_types::ContentHash;

/// Phase of validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationPhase {
    /// Pre-execution validation (before any state changes)
    PreExecution,
    
    /// Mid-execution validation (during state changes)
    MidExecution,
    
    /// Post-execution validation (after state changes)
    PostExecution,
    
    /// Verification validation (verifying external data)
    Verification,
    
    /// Schema validation only
    SchemaOnly,
    
    /// State transition validation only
    StateOnly,
    
    /// Permission validation only
    PermissionOnly,
}

/// Options for validation operations
#[derive(Debug, Clone)]
pub struct ValidationOptions {
    /// Enable structure validation
    pub validate_structure: bool,
    
    /// Enable state validation
    pub validate_state: bool,
    
    /// Enable schema validation
    pub validate_schema: bool,
    
    /// Enable permission validation
    pub validate_permissions: bool,
    
    /// Enable relationship validation
    pub validate_relationships: bool,
    
    /// Enable custom validation rules
    pub validate_custom_rules: bool,
    
    /// Maximum validation depth for nested resources
    pub max_validation_depth: usize,
    
    /// Whether to cache validation results
    pub enable_caching: bool,
    
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            validate_structure: true,
            validate_state: true,
            validate_schema: true,
            validate_permissions: true,
            validate_relationships: true,
            validate_custom_rules: true,
            max_validation_depth: 5,
            enable_caching: true,
            options: HashMap::new(),
        }
    }
}

/// Context for validation operations
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Resource being validated
    pub resource_id: Option<ResourceId>,
    
    /// Resource type
    pub resource_type: Option<ResourceTypeId>,
    
    /// Current resource state
    pub current_state: Option<ResourceState>,
    
    /// Target resource state (for transitions)
    pub target_state: Option<ResourceState>,
    
    /// Resource schema
    pub schema: Option<ResourceSchema>,
    
    /// Capabilities for validation
    pub capabilities: Option<CapabilitySet>,
    
    /// Domain ID
    pub domain_id: Option<DomainId>,
    
    /// Effect context (if available)
    pub effect_context: Option<Box<dyn EffectContext>>,
    
    /// Time context
    pub time_context: Option<TimeContext>,
    
    /// Validation phase
    pub phase: ValidationPhase,
    
    /// Validation options
    pub options: ValidationOptions,
    
    /// Additional context data
    pub context_data: HashMap<String, Vec<u8>>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self {
            resource_id: None,
            resource_type: None,
            current_state: None,
            target_state: None,
            schema: None,
            capabilities: None,
            domain_id: None,
            effect_context: None,
            time_context: None,
            phase: ValidationPhase::PreExecution,
            options: ValidationOptions::default(),
            context_data: HashMap::new(),
        }
    }
    
    /// Set the resource ID
    pub fn with_resource_id(mut self, resource_id: ResourceId) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    /// Set the resource type
    pub fn with_resource_type(mut self, resource_type: ResourceTypeId) -> Self {
        self.resource_type = Some(resource_type);
        self
    }
    
    /// Set the current resource state
    pub fn with_current_state(mut self, state: ResourceState) -> Self {
        self.current_state = Some(state);
        self
    }
    
    /// Set the target resource state
    pub fn with_target_state(mut self, state: ResourceState) -> Self {
        self.target_state = Some(state);
        self
    }
    
    /// Set the resource schema
    pub fn with_schema(mut self, schema: ResourceSchema) -> Self {
        self.schema = Some(schema);
        self
    }
    
    /// Set the capabilities
    pub fn with_capabilities(mut self, capabilities: CapabilitySet) -> Self {
        self.capabilities = Some(capabilities);
        self
    }
    
    /// Set the domain ID
    pub fn with_domain_id(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Set the effect context
    pub fn with_effect_context(mut self, context: Box<dyn EffectContext>) -> Self {
        self.effect_context = Some(context);
        self
    }
    
    /// Set the time context
    pub fn with_time_context(mut self, context: TimeContext) -> Self {
        self.time_context = Some(context);
        self
    }
    
    /// Set the validation phase
    pub fn with_phase(mut self, phase: ValidationPhase) -> Self {
        self.phase = phase;
        self
    }
    
    /// Set the validation options
    pub fn with_options(mut self, options: ValidationOptions) -> Self {
        self.options = options;
        self
    }
    
    /// Add additional context data
    pub fn with_context_data(mut self, key: impl Into<String>, value: Vec<u8>) -> Self {
        self.context_data.insert(key.into(), value);
        self
    }
    
    /// Add string context data
    pub fn with_string_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context_data.insert(key.into(), value.into().into_bytes());
        self
    }
    
    /// Get string context data
    pub fn get_string_context(&self, key: &str) -> Option<String> {
        self.context_data.get(key)
            .and_then(|data| String::from_utf8(data.clone()).ok())
    }
}

/// Builder for validation context
pub struct ValidationContextBuilder {
    context: ValidationContext,
}

impl ValidationContextBuilder {
    /// Create a new validation context builder
    pub fn new() -> Self {
        Self {
            context: ValidationContext::new(),
        }
    }
    
    /// Set the resource ID
    pub fn resource_id(mut self, resource_id: ResourceId) -> Self {
        self.context.resource_id = Some(resource_id);
        self
    }
    
    /// Set the resource type
    pub fn resource_type(mut self, resource_type: ResourceTypeId) -> Self {
        self.context.resource_type = Some(resource_type);
        self
    }
    
    /// Set the current resource state
    pub fn current_state(mut self, state: ResourceState) -> Self {
        self.context.current_state = Some(state);
        self
    }
    
    /// Set the target resource state
    pub fn target_state(mut self, state: ResourceState) -> Self {
        self.context.target_state = Some(state);
        self
    }
    
    /// Set the resource schema
    pub fn schema(mut self, schema: ResourceSchema) -> Self {
        self.context.schema = Some(schema);
        self
    }
    
    /// Set the capabilities
    pub fn capabilities(mut self, capabilities: CapabilitySet) -> Self {
        self.context.capabilities = Some(capabilities);
        self
    }
    
    /// Set the domain ID
    pub fn domain_id(mut self, domain_id: DomainId) -> Self {
        self.context.domain_id = Some(domain_id);
        self
    }
    
    /// Set the effect context
    pub fn effect_context(mut self, context: Box<dyn EffectContext>) -> Self {
        self.context.effect_context = Some(context);
        self
    }
    
    /// Set the time context
    pub fn time_context(mut self, context: TimeContext) -> Self {
        self.context.time_context = Some(context);
        self
    }
    
    /// Set the validation phase
    pub fn phase(mut self, phase: ValidationPhase) -> Self {
        self.context.phase = phase;
        self
    }
    
    /// Set the validation options
    pub fn options(mut self, options: ValidationOptions) -> Self {
        self.context.options = options;
        self
    }
    
    /// Add additional context data
    pub fn context_data(mut self, key: impl Into<String>, value: Vec<u8>) -> Self {
        self.context.context_data.insert(key.into(), value);
        self
    }
    
    /// Add string context data
    pub fn string_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.context_data.insert(key.into(), value.into().into_bytes());
        self
    }
    
    /// Build the validation context
    pub fn build(self) -> ValidationContext {
        self.context
    }
} 