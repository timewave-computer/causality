// Effect node implementation for the Temporal Effect Graph
// This file defines the EffectNode struct, which represents an effect operation node
// in the TEG.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use causality_types::{ContentHash, ContentAddressed, HashError, test_content_hash};
use serde_json;

use crate::{EffectId, ResourceId, CapabilityId, FactId, DomainId};
use crate::graph::edge::Edge;

/// Enumeration of parameter value types for effect operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ParameterValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
    /// Float value
    Float(f64),
    /// Array of values
    Array(Vec<ParameterValue>),
    /// Object (key-value map)
    Object(HashMap<String, ParameterValue>),
    /// Null value
    Null,
    /// Binary data
    Bytes(Vec<u8>),
}

// Implement Eq manually, using PartialEq for most variants except Float
impl Eq for ParameterValue {
    fn assert_receiver_is_total_eq(&self) {}
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
    
    /// Metadata for this effect
    pub metadata: HashMap<String, ParameterValue>,
    
    /// Content hash for this effect node
    pub content_hash: ContentHash,
}

impl EffectNode {
    /// Create a new effect node builder
    pub fn builder() -> EffectNodeBuilder {
        EffectNodeBuilder::new()
    }
    
    /// Create a new effect node
    pub fn new(
        id: EffectId, 
        effect_type: String, 
        domain_id: DomainId
    ) -> Self {
        // Create a basic effect node with default values
        Self {
            id,
            effect_type,
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id,
            metadata: HashMap::new(),
            content_hash: ContentHash::new("blake3", vec![0; 32]), // Placeholder
        }
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
        &self.metadata
    }
    
    /// Check if this effect is a constant value
    pub fn is_constant(&self) -> bool {
        // Effect is constant if it has the "constant" parameter set to true
        // or if it has the "constant_value" parameter defined
        self.get_parameter("constant").map_or(false, |v| {
            match v {
                ParameterValue::Boolean(b) => *b,
                _ => false,
            }
        }) || self.get_parameter("constant_value").is_some()
    }
    
    /// Get the constant value of this effect, if it is a constant
    pub fn constant_value(&self) -> Option<&ParameterValue> {
        // Return the constant_value parameter if it exists
        self.get_parameter("constant_value")
    }
    
    // TODO: Implement properly
    pub fn has_side_effects(&self) -> bool {
        // Check metadata or operation type, e.g., if it's "io"
        self.operation_type() == Some("io") // Simple check for now
    }
    
    // TODO: Implement properly
    pub fn is_resource_operation(&self) -> bool {
        !self.resources_accessed.is_empty()
    }
    
    // TODO: Implement properly - needs access to graph edges
    // This likely needs to be a method on TemporalEffectGraph taking EffectId
    // Or EffectNode needs to store relevant edge info.
    // Returning empty for now to satisfy compiler.
    pub fn resource_edges(&self) -> Vec<(ResourceId, Edge)> { // Placeholder return type
        Vec::new()
    }
    
    /// Set whether this effect should be treated as pure (no side effects)
    pub fn is_pure(&self) -> bool {
        // A pure effect has no side effects and is deterministic
        !self.has_side_effects() && 
        self.get_parameter("pure").map_or(false, |v| {
            match v {
                ParameterValue::Boolean(b) => *b,
                _ => false,
            }
        })
    }
    
    /// Set a constant value for this effect
    pub fn set_constant_value(&mut self, value: String) {
        let param_value = ParameterValue::String(value);
        self.parameters.insert("constant_value".to_string(), param_value);
        self.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
    }
    
    /// Set parameters for this effect
    pub fn set_parameters(&mut self, params: HashMap<String, ParameterValue>) {
        self.parameters = params;
    }
    
    /// Set metadata for this effect
    pub fn set_metadata(&mut self, metadata: HashMap<String, ParameterValue>) {
        self.metadata = metadata;
    }
}

impl ContentAddressed for EffectNode {
    fn content_hash(&self) -> Result<causality_types::HashOutput, HashError> {
        // We need to create a copy without the content_hash field to avoid circular hashing
        let mut effect_for_hash = self.clone();
        // Reset the content hash to a default/empty value to avoid it affecting the hash
        effect_for_hash.content_hash = ContentHash::new("blake3", vec![0; 32]);
        
        // Serialize the effect to JSON bytes
        let serialized = serde_json::to_vec(&effect_for_hash)
            .map_err(|e| HashError::SerializationError(e.to_string()))?;
        
        // Calculate the hash of the serialized data
        let hash_output = causality_types::content_addressing::content_hash_from_bytes(&serialized);
        Ok(hash_output)
    }
    
    fn verify(&self, expected_hash: &causality_types::HashOutput) -> Result<bool, HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        serde_json::to_vec(self).map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> where Self: Sized {
        serde_json::from_slice(bytes).map_err(|e| HashError::SerializationError(e.to_string()))
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
    metadata: HashMap<String, ParameterValue>,
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
            metadata: HashMap::new(),
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
    
    /// Add metadata to the effect
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<ParameterValue>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the effect node
    pub fn build(self) -> Result<EffectNode, String> {
        let id = self.id.ok_or_else(|| "Effect ID is required".to_string())?;
        let effect_type = self.effect_type.ok_or_else(|| "Effect type is required".to_string())?;
        let domain_id = self.domain_id.ok_or_else(|| "Domain ID is required".to_string())?;
        
        // Use test_content_hash instead of creating a new ContentHash
        let content_hash = test_content_hash();
        
        Ok(EffectNode {
            id,
            effect_type,
            parameters: self.parameters,
            required_capabilities: self.required_capabilities,
            resources_accessed: self.resources_accessed,
            fact_dependencies: self.fact_dependencies,
            domain_id,
            metadata: self.metadata,
            content_hash,
        })
    }
}
