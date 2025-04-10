// Effect node implementation for the Temporal Effect Graph
// This file defines the EffectNode struct, which represents an effect operation node
// in the TEG.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use causality_types::{ContentHash, ContentAddressed, ContentHashError};

use crate::{EffectId, ResourceId, CapabilityId, FactId, DomainId};

/// Enumeration of parameter value types for effect operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Array(Vec<ParameterValue>),
    Object(HashMap<String, ParameterValue>),
    Null,
}

/// Represents an effect operation node in the Temporal Effect Graph
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EffectNode {
    /// Unique identifier for this effect
    pub id: EffectId,
    
    /// The type of effect operation
    pub effect_type: String,
    
    /// Parameters for the effect operation
    pub parameters: HashMap<String, ParameterValue>,
    
    /// Capabilities required to execute this effect
    pub required_capabilities: Vec<CapabilityId>,
    
    /// Resources accessed by this effect
    pub resources_accessed: Vec<ResourceId>,
    
    /// Temporal fact dependencies
    pub fact_dependencies: Vec<FactId>,
    
    /// Domain in which this effect executes
    pub domain_id: DomainId,
    
    /// Content hash for this effect node
    pub content_hash: ContentHash,
}

impl EffectNode {
    /// Create a new effect node builder
    pub fn builder() -> EffectNodeBuilder {
        EffectNodeBuilder::new()
    }
    
    /// Check if this effect requires a specific capability
    pub fn requires_capability(&self, capability_id: &CapabilityId) -> bool {
        self.required_capabilities.contains(capability_id)
    }
    
    /// Check if this effect accesses a specific resource
    pub fn accesses_resource(&self, resource_id: &ResourceId) -> bool {
        self.resources_accessed.contains(resource_id)
    }
    
    /// Get a parameter value by name
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterValue> {
        self.parameters.get(name)
    }
    
    /// Get the effect identifier
    pub fn effect_id(&self) -> &EffectId {
        &self.id
    }
    
    /// Get the effect type
    pub fn effect_type(&self) -> &str {
        &self.effect_type
    }
    
    /// Get the effect parameters
    pub fn parameters(&self) -> &HashMap<String, ParameterValue> {
        &self.parameters
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Check if the effect is public
    pub fn is_public(&self) -> bool {
        // Implementation depends on your definition of "public"
        // For example, you might have a parameter or metadata field that indicates this
        self.get_parameter("public").map_or(false, |v| {
            match v {
                ParameterValue::Boolean(b) => *b,
                _ => false,
            }
        })
    }
    
    /// Get the operation type, if available
    pub fn operation_type(&self) -> Option<&str> {
        self.get_parameter("operation_type").and_then(|v| {
            match v {
                ParameterValue::String(s) => Some(s.as_str()),
                _ => None,
            }
        })
    }
    
    /// Get the effect name
    pub fn name(&self) -> &str {
        self.get_parameter("name").and_then(|v| {
            match v {
                ParameterValue::String(s) => Some(s.as_str()),
                _ => None,
            }
        }).unwrap_or(&self.effect_type)
    }
    
    /// Get the return type, if available
    pub fn return_type(&self) -> Option<&str> {
        self.get_parameter("return_type").and_then(|v| {
            match v {
                ParameterValue::String(s) => Some(s.as_str()),
                _ => None,
            }
        })
    }
    
    /// Get metadata for the effect
    pub fn metadata(&self) -> &HashMap<String, ParameterValue> {
        // In this simple implementation, we're using parameters as metadata
        // In a more complex implementation, you might have a separate metadata field
        &self.parameters
    }
}

impl ContentAddressed for EffectNode {
    fn content_hash(&self) -> Result<ContentHash, ContentHashError> {
        // For now, we'll return the precalculated hash
        // In a full implementation, we would compute the hash here
        Ok(self.content_hash.clone())
    }
    
    fn verify(&self) -> Result<bool, ContentHashError> {
        let computed_hash = self.content_hash()?;
        Ok(computed_hash == self.content_hash)
    }
}

/// Builder for creating Effect Nodes
pub struct EffectNodeBuilder {
    id: Option<EffectId>,
    effect_type: Option<String>,
    parameters: HashMap<String, ParameterValue>,
    required_capabilities: Vec<CapabilityId>,
    resources_accessed: Vec<ResourceId>,
    fact_dependencies: Vec<FactId>,
    domain_id: Option<DomainId>,
}

impl EffectNodeBuilder {
    /// Create a new effect node builder
    pub fn new() -> Self {
        Self {
            id: None,
            effect_type: None,
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: None,
        }
    }
    
    /// Set the effect ID
    pub fn id(mut self, id: impl Into<EffectId>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set the effect type
    pub fn effect_type(mut self, effect_type: impl Into<String>) -> Self {
        self.effect_type = Some(effect_type.into());
        self
    }
    
    /// Add a parameter
    pub fn parameter(mut self, name: impl Into<String>, value: ParameterValue) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }
    
    /// Add a string parameter
    pub fn string_parameter(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameter(name, ParameterValue::String(value.into()))
    }
    
    /// Add an integer parameter
    pub fn int_parameter(self, name: impl Into<String>, value: i64) -> Self {
        self.parameter(name, ParameterValue::Integer(value))
    }
    
    /// Add a capability requirement
    pub fn requires_capability(mut self, capability_id: impl Into<CapabilityId>) -> Self {
        self.required_capabilities.push(capability_id.into());
        self
    }
    
    /// Add a resource access
    pub fn accesses_resource(mut self, resource_id: impl Into<ResourceId>) -> Self {
        self.resources_accessed.push(resource_id.into());
        self
    }
    
    /// Add a fact dependency
    pub fn depends_on_fact(mut self, fact_id: impl Into<FactId>) -> Self {
        self.fact_dependencies.push(fact_id.into());
        self
    }
    
    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<DomainId>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }
    
    /// Build the effect node
    pub fn build(self) -> Result<EffectNode, String> {
        let id = self.id.ok_or_else(|| "Effect ID is required".to_string())?;
        let effect_type = self.effect_type.ok_or_else(|| "Effect type is required".to_string())?;
        let domain_id = self.domain_id.ok_or_else(|| "Domain ID is required".to_string())?;
        
        // In a real implementation, we would compute the content hash here
        let content_hash = ContentHash::default(); // Placeholder
        
        Ok(EffectNode {
            id,
            effect_type,
            parameters: self.parameters,
            required_capabilities: self.required_capabilities,
            resources_accessed: self.resources_accessed,
            fact_dependencies: self.fact_dependencies,
            domain_id,
            content_hash,
        })
    }
}
