//! Effect context module
//!
//! This module defines the execution context for effects, including capabilities,
//! resources, and metadata.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use std::any::Any;
use serde::{Serialize, Deserialize};

use super::types::{EffectId, Right, ExecutionBoundary};
use causality_types::ContentId;
use super::registry::EffectExecutor;
use crate::resource::ResourceId;

/// A capability represents permission to access a resource in a specific way
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// The resource ID the capability applies to
    pub resource_id: ContentId,
    /// The type of access permission
    pub right: Right,
}

impl Capability {
    /// Create a new capability
    pub fn new(resource_id: ContentId, right: Right) -> Self {
        Self { resource_id, right }
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.resource_id, self.right)
    }
}

impl std::str::FromStr for Capability {
    type Err = CapabilityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(CapabilityError::InvalidFormat(format!("Invalid capability format: {}", s)));
        }
        
        let resource_id = parts[0].parse::<ContentId>()
            .map_err(|_| CapabilityError::InvalidFormat(format!("Invalid ContentId in capability: {}", parts[0])))?;
        
        let right = parts[1].parse::<Right>()
            .map_err(|_| CapabilityError::InvalidFormat(format!("Invalid Right in capability: {}", parts[1])))?;
        
        Ok(Self::new(resource_id, right))
    }
}

/// A set of capability grants
pub type CapabilityGrants = HashSet<Capability>;

/// Error type for capability operations
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Missing required capability: {0:?}")]
    MissingCapability(Capability),
    
    #[error("Invalid capability format: {0}")]
    InvalidFormat(String),
    
    #[error("Capability validation failed: {0}")]
    ValidationFailed(String),
}

/// Error type for effect context operations
#[derive(Debug, Error)]
pub enum EffectContextError {
    #[error("Missing capability: {0:?}")]
    MissingCapability(Capability),
    
    #[error("Missing resource: {0}")]
    MissingResource(ResourceId),
    
    #[error("Invalid resource: {0}")]
    InvalidResource(String),
    
    #[error("Resource access denied: {0}")]
    ResourceAccessDenied(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Context error: {0}")]
    ContextError(String),
}

/// Result type for effect context operations
pub type EffectContextResult<T> = Result<T, EffectContextError>;

/// Trait representing the execution context for effects.
pub trait EffectContext: Send + Sync + Debug {
    /// Get the effect ID
    fn effect_id(&self) -> &EffectId;
    
    /// Get the capabilities available in this context
    fn capabilities(&self) -> &[Capability];
    
    /// Get metadata for this context
    fn metadata(&self) -> &HashMap<String, String>;
    
    /// Get resources available in this context
    fn resources(&self) -> &HashSet<ResourceId>;
    
    /// Get the parent context
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>>;
    
    /// Check if this context has a capability
    fn has_capability(&self, capability: &Capability) -> bool;
    
    /// Get the associated registry for this context, if any
    fn get_registry(&self) -> Option<Arc<dyn EffectExecutor>> {
        None
    }
    
    /// Derive a new context with a different effect ID
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext>;
    
    /// Add capabilities to this context
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext>;
    
    /// Add resources to this context
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext>;
    
    /// Add metadata to this context
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext>;
    
    /// Clone this context
    fn clone_context(&self) -> Box<dyn EffectContext>;
    
    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Basic implementation of effect context
#[derive(Debug)]
pub struct BasicEffectContext {
    /// Effect ID
    effect_id: EffectId,
    
    /// Capabilities
    capabilities: Vec<Capability>,
    
    /// Metadata
    metadata: HashMap<String, String>,
    
    /// Resources
    resources: HashSet<ResourceId>,
    
    /// Parent context
    parent_context: Option<Arc<dyn EffectContext>>,
}

impl BasicEffectContext {
    /// Create a new basic effect context
    pub fn new(effect_id: EffectId) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            resources: HashSet::new(),
            parent_context: None,
        }
    }
    
    /// Create a new context with a parent
    pub fn with_parent(effect_id: EffectId, parent: Arc<dyn EffectContext>) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            resources: HashSet::new(),
            parent_context: Some(parent),
        }
    }
    
    /// Set capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }
    
    /// Add a capability
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }
    
    /// Add multiple capabilities
    pub fn add_capabilities(&mut self, capabilities: Vec<Capability>) {
        self.capabilities.extend(capabilities);
    }
    
    /// Add a metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Add multiple metadata entries
    pub fn add_metadata_entries(&mut self, entries: HashMap<String, String>) {
        self.metadata.extend(entries);
    }
    
    /// Set domain ID
    pub fn set_domain_id(&mut self, domain_id: String) {
        self.metadata.insert("domain_id".to_string(), domain_id);
    }
    
    /// Add a resource
    pub fn add_resource(&mut self, resource_id: ResourceId) {
        self.resources.insert(resource_id);
    }
    
    /// Add multiple resources
    pub fn add_resources(&mut self, resources: HashSet<ResourceId>) {
        self.resources.extend(resources);
    }
    
    /// Set the execution boundary
    pub fn set_execution_boundary(&mut self, boundary: ExecutionBoundary) {
        self.metadata.insert(
            "execution_boundary".to_string(),
            boundary.to_string(),
        );
    }
    
    /// Get the execution boundary
    pub fn execution_boundary(&self) -> ExecutionBoundary {
        self.metadata
            .get("execution_boundary")
            .map(|s| match s.as_str() {
                "local" => ExecutionBoundary::Local,
                "none" => ExecutionBoundary::None,
                s if s.starts_with("domain:") => {
                    ExecutionBoundary::Domain(s[7..].to_string())
                }
                s if s.starts_with("custom:") => {
                    ExecutionBoundary::Custom(s[7..].to_string())
                }
                _ => ExecutionBoundary::None,
            })
            .unwrap_or_default()
    }
}

impl EffectContext for BasicEffectContext {
    fn effect_id(&self) -> &EffectId {
        &self.effect_id
    }
    
    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        &self.resources
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        self.parent_context.as_ref()
    }
    
    fn has_capability(&self, capability: &Capability) -> bool {
        // Check if this context has the capability
        if self.capabilities.contains(capability) {
            return true;
        }
        
        // Check parent context if available
        if let Some(parent) = &self.parent_context {
            return parent.has_capability(capability);
        }
        
        false
    }
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        Box::new(BasicEffectContext {
            effect_id,
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
            parent_context: self.parent_context.clone(),
        })
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
        let mut new_context = BasicEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
            parent_context: self.parent_context.clone(),
        };
        
        for capability in capabilities {
            new_context.add_capability(capability);
        }
        
        Box::new(new_context)
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut new_context = BasicEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
            parent_context: self.parent_context.clone(),
        };
        
        for resource in resources {
            new_context.add_resource(resource);
        }
        
        Box::new(new_context)
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_context = BasicEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
            parent_context: self.parent_context.clone(),
        };
        
        for (key, value) in metadata {
            new_context.add_metadata(key, value);
        }
        
        Box::new(new_context)
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(BasicEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
            parent_context: self.parent_context.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn get_registry(&self) -> Option<Arc<dyn EffectExecutor>> {
        None
    }
}

/// Builder for effect contexts
pub struct EffectContextBuilder {
    effect_id: EffectId,
    capabilities: Vec<Capability>,
    metadata: HashMap<String, String>,
    resources: HashSet<ResourceId>,
    parent_context: Option<Arc<dyn EffectContext>>,
}

impl EffectContextBuilder {
    /// Create a new builder
    pub fn new(effect_id: EffectId) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            resources: HashSet::new(),
            parent_context: None,
        }
    }
    
    /// Set the parent context
    pub fn with_parent(mut self, parent: Arc<dyn EffectContext>) -> Self {
        self.parent_context = Some(parent);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
    
    /// Add a metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Add multiple metadata entries
    pub fn with_metadata_entries(mut self, entries: HashMap<String, String>) -> Self {
        self.metadata.extend(entries);
        self
    }
    
    /// Set domain ID
    pub fn with_domain_id(mut self, domain_id: String) -> Self {
        self.metadata.insert("domain_id".to_string(), domain_id);
        self
    }
    
    /// Add a resource
    pub fn with_resource(mut self, resource_id: ResourceId) -> Self {
        self.resources.insert(resource_id);
        self
    }
    
    /// Add multiple resources
    pub fn with_resources(mut self, resources: HashSet<ResourceId>) -> Self {
        self.resources.extend(resources);
        self
    }
    
    /// Set the execution boundary
    pub fn with_execution_boundary(mut self, boundary: ExecutionBoundary) -> Self {
        self.metadata.insert(
            "execution_boundary".to_string(),
            boundary.to_string(),
        );
        self
    }
    
    /// Build the context
    pub fn build(self) -> BasicEffectContext {
        BasicEffectContext {
            effect_id: self.effect_id,
            capabilities: self.capabilities,
            metadata: self.metadata,
            resources: self.resources,
            parent_context: self.parent_context,
        }
    }
}

/// Boxed effect context implementation
/// This is a simple wrapper around a Box<dyn EffectContext> that implements the EffectContext trait
#[derive(Debug)]
pub struct BoxedEffectContext {
    /// The wrapped context
    inner: Box<dyn EffectContext>,
}

impl Clone for BoxedEffectContext {
    fn clone(&self) -> Self {
        // Use the clone_context method to create a cloned context
        Self {
            inner: self.inner.clone_context(),
        }
    }
}

impl BoxedEffectContext {
    /// Create a new boxed context from an existing context
    pub fn new(context: Box<dyn EffectContext>) -> Self {
        Self { inner: context }
    }

    /// Create a new boxed context from an existing context reference
    pub fn from_context(context: &dyn EffectContext) -> Self {
        Self { inner: context.clone_context() }
    }

    /// Get a reference to the inner context
    pub fn inner(&self) -> &dyn EffectContext {
        self.inner.as_ref()
    }

    /// Convert to inner box
    pub fn into_inner(self) -> Box<dyn EffectContext> {
        self.inner
    }
}

impl EffectContext for BoxedEffectContext {
    fn effect_id(&self) -> &EffectId {
        self.inner.effect_id()
    }
    
    fn capabilities(&self) -> &[Capability] {
        self.inner.capabilities()
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        self.inner.metadata()
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        self.inner.resources()
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        self.inner.parent_context()
    }
    
    fn has_capability(&self, capability: &Capability) -> bool {
        self.inner.has_capability(capability)
    }
    
    fn get_registry(&self) -> Option<Arc<dyn EffectExecutor>> {
        self.inner.get_registry()
    }
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        self.inner.derive_context(effect_id)
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
        self.inner.with_additional_capabilities(capabilities)
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        self.inner.with_additional_resources(resources)
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        self.inner.with_additional_metadata(metadata)
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(Self {
            inner: self.inner.clone_context(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 