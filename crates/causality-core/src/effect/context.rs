// Effect Context
//
// This module provides the context for executing effects, including
// capabilities, resources, and metadata.

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug};
use std::sync::Arc;

use thiserror::Error;
use serde::{Serialize, Deserialize};

use causality_types::ContentId;
use crate::resource_types::ResourceId;
use crate::capability::{Capability, CapabilityGrants, CapabilityError, ResourceId as CapResourceId};
use super::types::{EffectId, EffectTypeId, ExecutionBoundary};

/// Type for capability references
pub type CapabilityReference = Arc<Capability<dyn std::any::Any + Send + Sync>>;

/// Errors that can occur during effect context operations
#[derive(Error, Debug)]
pub enum EffectContextError {
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    #[error("Invalid resource: {0}")]
    InvalidResource(String),
    
    #[error("Missing parent context")]
    MissingParentContext,
    
    #[error("Capability verification error: {0}")]
    CapabilityVerificationError(#[from] CapabilityError),
    
    #[error("Context serialization error: {0}")]
    SerializationError(String),
    
    #[error("Context deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Context access error: {0}")]
    AccessError(String),
    
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
}

/// Result type for effect context operations
pub type EffectContextResult<T> = Result<T, EffectContextError>;

/// Effect context trait
pub trait EffectContext: Debug + Send + Sync {
    /// Get the effect ID
    fn effect_id(&self) -> &EffectId;
    
    /// Get the capabilities
    fn capabilities(&self) -> &[CapabilityReference];
    
    /// Get metadata
    fn metadata(&self) -> &HashMap<String, String>;
    
    /// Get the domain ID if available
    fn domain_id(&self) -> Option<&str> {
        self.metadata().get("domain_id").map(|s| s.as_str())
    }
    
    /// Get resources
    fn resources(&self) -> &HashSet<ResourceId>;
    
    /// Get parent context if available
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>>;
    
    /// Check if a capability is present
    fn has_capability(&self, capability_id: &ContentId) -> bool {
        self.capabilities().iter().any(|cap| {
            cap.id.hash.to_content_id().map(|id| &id == capability_id).unwrap_or(false)
        })
    }
    
    /// Check if resource capability is present with required grants
    fn verify_resource_capability(&self, resource_id: &ResourceId, required_grants: &CapabilityGrants) -> EffectContextResult<bool> {
        // First check this context's capabilities
        for capability in self.capabilities() {
            if capability.id.hash.to_content_id().map(|id| id == resource_id.content_hash().to_content_id().unwrap()).unwrap_or(false) {
                // Check if the capability grants the required access
                if capability.grants.includes(required_grants) {
                    return Ok(true);
                }
            }
        }
        
        // Then check parent context if available
        if let Some(parent) = self.parent_context() {
            return parent.verify_resource_capability(resource_id, required_grants);
        }
        
        Ok(false)
    }
    
    /// Check if context has read access to resource
    fn can_read_resource(&self, resource_id: &ResourceId) -> EffectContextResult<bool> {
        let read_grants = CapabilityGrants {
            read: true,
            write: false,
            delegate: false,
        };
        
        self.verify_resource_capability(resource_id, &read_grants)
    }
    
    /// Check if context has write access to resource
    fn can_write_resource(&self, resource_id: &ResourceId) -> EffectContextResult<bool> {
        let write_grants = CapabilityGrants {
            read: false,
            write: true,
            delegate: false,
        };
        
        self.verify_resource_capability(resource_id, &write_grants)
    }
    
    /// Check if context has delegation rights for resource
    fn can_delegate_resource(&self, resource_id: &ResourceId) -> EffectContextResult<bool> {
        let delegate_grants = CapabilityGrants {
            read: false,
            write: false,
            delegate: true,
        };
        
        self.verify_resource_capability(resource_id, &delegate_grants)
    }
    
    /// Get metadata value
    fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata().get(key).map(|s| s.as_str())
    }
    
    /// Check if a resource is accessible
    fn has_resource(&self, resource_id: &ResourceId) -> bool {
        self.resources().contains(resource_id)
    }
    
    /// Create a derived context for a new effect
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext>;
    
    /// Create a derived context with additional capabilities
    fn with_additional_capabilities(&self, capabilities: Vec<CapabilityReference>) -> Box<dyn EffectContext>;
    
    /// Create a derived context with additional resources
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext>;
    
    /// Create a derived context with additional metadata
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext>;
    
    /// Serialize the context to bytes
    fn to_bytes(&self) -> EffectContextResult<Vec<u8>>;
    
    /// Create a context from bytes
    fn from_bytes(bytes: &[u8]) -> EffectContextResult<Box<dyn EffectContext>> where Self: Sized;
}

/// Basic effect context implementation
#[derive(Debug, Clone)]
pub struct BasicEffectContext {
    /// Effect ID
    effect_id: EffectId,
    
    /// Capabilities
    capabilities: Vec<CapabilityReference>,
    
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
    
    /// Create a context with a parent
    pub fn with_parent(effect_id: EffectId, parent: Arc<dyn EffectContext>) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            resources: HashSet::new(),
            parent_context: Some(parent),
        }
    }
    
    /// Create a context with capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityReference>) -> Self {
        self.capabilities = capabilities;
        self
    }
    
    /// Add a capability
    pub fn add_capability(&mut self, capability: CapabilityReference) {
        self.capabilities.push(capability);
    }
    
    /// Add multiple capabilities
    pub fn add_capabilities(&mut self, capabilities: Vec<CapabilityReference>) {
        self.capabilities.extend(capabilities);
    }
    
    /// Add metadata
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
    
    /// Set execution boundary
    pub fn set_execution_boundary(&mut self, boundary: ExecutionBoundary) {
        self.metadata.insert("execution_boundary".to_string(), format!("{:?}", boundary));
    }
    
    /// Get the execution boundary
    pub fn execution_boundary(&self) -> ExecutionBoundary {
        self.metadata.get("execution_boundary")
            .and_then(|s| match s.as_str() {
                "Inside" => Some(ExecutionBoundary::Inside),
                "Outside" => Some(ExecutionBoundary::Outside),
                "Boundary" => Some(ExecutionBoundary::Boundary),
                "Any" => Some(ExecutionBoundary::Any),
                _ => None,
            })
            .unwrap_or(ExecutionBoundary::Any)
    }
    
    /// Apply parent context capabilities and resources
    pub fn apply_parent_context(&mut self) -> EffectContextResult<()> {
        if let Some(parent) = &self.parent_context {
            // Copy capabilities from parent
            for capability in parent.capabilities() {
                self.capabilities.push(capability.clone());
            }
            
            // Copy resources from parent
            for resource in parent.resources() {
                self.resources.insert(resource.clone());
            }
            
            // Merge metadata, but don't override existing values
            for (key, value) in parent.metadata() {
                if !self.metadata.contains_key(key) {
                    self.metadata.insert(key.clone(), value.clone());
                }
            }
        }
        
        Ok(())
    }
}

impl EffectContext for BasicEffectContext {
    fn effect_id(&self) -> &EffectId {
        &self.effect_id
    }
    
    fn capabilities(&self) -> &[CapabilityReference] {
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
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        let mut context = BasicEffectContext::with_parent(effect_id, Arc::new(self.clone()));
        
        // Apply basic inheritance from parent
        let _ = context.apply_parent_context();
        
        Box::new(context)
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<CapabilityReference>) -> Box<dyn EffectContext> {
        let mut context = self.clone();
        context.add_capabilities(capabilities);
        Box::new(context)
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut context = self.clone();
        context.add_resources(resources);
        Box::new(context)
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut context = self.clone();
        context.add_metadata_entries(metadata);
        Box::new(context)
    }
    
    fn to_bytes(&self) -> EffectContextResult<Vec<u8>> {
        // Simplified implementation - in a real system, use proper serialization
        let mut bytes = Vec::new();
        
        // Serialize effect ID
        bytes.extend_from_slice(&self.effect_id.to_bytes().map_err(|e| 
            EffectContextError::SerializationError(format!("Failed to serialize effect ID: {}", e)))?);
        
        // For other fields, use proper serialization in a real implementation
        
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> EffectContextResult<Box<dyn EffectContext>> {
        // Simplified implementation - in a real system, use proper deserialization
        Err(EffectContextError::SerializationError(
            "Context deserialization not implemented".to_string()
        ))
    }
}

/// Builder for effect context
pub struct EffectContextBuilder {
    effect_id: EffectId,
    capabilities: Vec<CapabilityReference>,
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
    
    /// Set parent context
    pub fn with_parent(mut self, parent: Arc<dyn EffectContext>) -> Self {
        self.parent_context = Some(parent);
        self
    }
    
    /// Add capability
    pub fn with_capability(mut self, capability: CapabilityReference) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityReference>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
    
    /// Add metadata
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
    
    /// Add resource
    pub fn with_resource(mut self, resource_id: ResourceId) -> Self {
        self.resources.insert(resource_id);
        self
    }
    
    /// Add resources
    pub fn with_resources(mut self, resources: HashSet<ResourceId>) -> Self {
        self.resources.extend(resources);
        self
    }
    
    /// Set execution boundary
    pub fn with_execution_boundary(mut self, boundary: ExecutionBoundary) -> Self {
        self.metadata.insert("execution_boundary".to_string(), format!("{:?}", boundary));
        self
    }
    
    /// Build the context
    pub fn build(self) -> BasicEffectContext {
        let mut context = if let Some(parent) = self.parent_context {
            BasicEffectContext::with_parent(self.effect_id, parent)
        } else {
            BasicEffectContext::new(self.effect_id)
        };
        
        context.add_capabilities(self.capabilities);
        context.add_metadata_entries(self.metadata);
        context.add_resources(self.resources);
        
        context
    }
} 