// Effect module for content-addressed operations
//
// This module defines the Effect type and related functionality
// for handling content-addressed operations.

use borsh::{BorshSerialize, BorshDeserialize};
use std::collections::HashMap;
use crate::crypto::{
    ContentAddressed, ContentId, HashOutput, HashFactory, HashError
};

/// Type of effect
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum EffectType {
    /// Create a resource
    CreateResource,
    /// Update a resource
    UpdateResource,
    /// Delete a resource
    DeleteResource,
    /// Custom effect type
    Custom(String),
}

impl std::fmt::Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateResource => write!(f, "CreateResource"),
            Self::UpdateResource => write!(f, "UpdateResource"),
            Self::DeleteResource => write!(f, "DeleteResource"),
            Self::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// An effect that can be executed
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Effect {
    /// Type of effect
    effect_type: EffectType,
    /// Parameters for the effect
    parameters: HashMap<String, Vec<u8>>,
    /// Optional resource identifier this effect operates on
    resource_id: Option<ContentId>,
}

impl Effect {
    /// Create a new effect
    pub fn new(effect_type: EffectType) -> Self {
        Self {
            effect_type,
            parameters: HashMap::new(),
            resource_id: None,
        }
    }
    
    /// Create a new effect with a resource ID
    pub fn with_resource(effect_type: EffectType, resource_id: ContentId) -> Self {
        Self {
            effect_type,
            parameters: HashMap::new(),
            resource_id: Some(resource_id),
        }
    }
    
    /// Get the effect type
    pub fn effect_type(&self) -> &EffectType {
        &self.effect_type
    }
    
    /// Set a parameter
    pub fn set_parameter(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.parameters.insert(key.into(), value.into());
    }
    
    /// Get a parameter
    pub fn get_parameter(&self, key: &str) -> Option<&[u8]> {
        self.parameters.get(key).map(|v| v.as_slice())
    }
    
    /// Set the resource ID
    pub fn set_resource_id(&mut self, resource_id: ContentId) {
        self.resource_id = Some(resource_id);
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> Option<&ContentId> {
        self.resource_id.as_ref()
    }
}

impl ContentAddressed for Effect {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher from the registry
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization of the effect
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Result of effect execution
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct EffectOutcome {
    /// Whether the effect executed successfully
    success: bool,
    /// Optional result data
    result: Option<Vec<u8>>,
    /// Optional error message if the effect failed
    error: Option<String>,
}

impl EffectOutcome {
    /// Create a successful outcome
    pub fn success() -> Self {
        Self {
            success: true,
            result: None,
            error: None,
        }
    }
    
    /// Create a successful outcome with result data
    pub fn with_result(result: Vec<u8>) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create a failed outcome with an error message
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error.into()),
        }
    }
    
    /// Check if the outcome was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Get the result data
    pub fn result(&self) -> Option<&[u8]> {
        self.result.as_ref().map(|v| v.as_slice())
    }
    
    /// Get the error message
    pub fn error(&self) -> Option<&str> {
        self.error.as_ref().map(|s| s.as_str())
    }
}

impl ContentAddressed for EffectOutcome {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher from the registry
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization of the outcome
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// A registry for content-addressed effects
pub struct EffectRegistry {
    /// Effects indexed by their content ID
    effects: HashMap<ContentId, Effect>,
}

impl EffectRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }
    
    /// Register an effect
    pub fn register(&mut self, effect: Effect) -> ContentId {
        let content_id = effect.content_id();
        self.effects.insert(content_id.clone(), effect);
        content_id
    }
    
    /// Get an effect by its content ID
    pub fn get(&self, content_id: &ContentId) -> Option<&Effect> {
        self.effects.get(content_id)
    }
    
    /// Remove an effect by its content ID
    pub fn remove(&mut self, content_id: &ContentId) -> Option<Effect> {
        self.effects.remove(content_id)
    }
    
    /// Check if the registry contains an effect
    pub fn contains(&self, content_id: &ContentId) -> bool {
        self.effects.contains_key(content_id)
    }
    
    /// Get the number of effects in the registry
    pub fn len(&self) -> usize {
        self.effects.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
    
    /// Get an iterator over all effects
    pub fn iter(&self) -> impl Iterator<Item = (&ContentId, &Effect)> {
        self.effects.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_content_addressing() {
        // Create an effect
        let mut effect = Effect::new(EffectType::CreateResource);
        effect.set_parameter("name", b"test resource".to_vec());
        effect.set_parameter("data", b"test data".to_vec());
        
        // Get the content hash
        let hash = effect.content_hash();
        
        // Create an identical effect
        let mut effect2 = Effect::new(EffectType::CreateResource);
        effect2.set_parameter("name", b"test resource".to_vec());
        effect2.set_parameter("data", b"test data".to_vec());
        
        // The content hashes should be identical
        assert_eq!(hash, effect2.content_hash());
        
        // Create a different effect
        let mut effect3 = Effect::new(EffectType::CreateResource);
        effect3.set_parameter("name", b"different resource".to_vec());
        
        // The content hash should be different
        assert_ne!(hash, effect3.content_hash());
        
        // Test serialization and deserialization
        let bytes = effect.to_bytes();
        let deserialized = Effect::from_bytes(&bytes).unwrap();
        assert_eq!(effect, deserialized);
        
        // Test verification
        assert!(effect.verify());
        assert!(effect2.verify());
        assert!(effect3.verify());
    }
    
    #[test]
    fn test_effect_content_id() {
        // Create an effect
        let mut effect = Effect::new(EffectType::UpdateResource);
        effect.set_parameter("data", b"new data".to_vec());
        
        // Get the content ID
        let content_id = effect.content_id();
        
        // The content ID should be derived from the content hash
        assert_eq!(content_id.hash(), &effect.content_hash());
        
        // Test string representation
        let id_str = content_id.to_string();
        assert!(id_str.starts_with("cid:"));
        
        // Test parsing
        let parsed_id = ContentId::parse(&id_str).unwrap();
        assert_eq!(content_id, parsed_id);
    }
    
    #[test]
    fn test_effect_outcome_content_addressing() {
        // Create a successful outcome
        let outcome = EffectOutcome::with_result(b"result data".to_vec());
        
        // Get the content hash
        let hash = outcome.content_hash();
        
        // Create an identical outcome
        let outcome2 = EffectOutcome::with_result(b"result data".to_vec());
        
        // The content hashes should be identical
        assert_eq!(hash, outcome2.content_hash());
        
        // Create a different outcome
        let outcome3 = EffectOutcome::failure("An error occurred");
        
        // The content hash should be different
        assert_ne!(hash, outcome3.content_hash());
        
        // Test serialization and deserialization
        let bytes = outcome.to_bytes();
        let deserialized = EffectOutcome::from_bytes(&bytes).unwrap();
        assert_eq!(outcome, deserialized);
        
        // Test verification
        assert!(outcome.verify());
        assert!(outcome2.verify());
        assert!(outcome3.verify());
    }
    
    #[test]
    fn test_effect_registry() {
        // Create a registry
        let mut registry = EffectRegistry::new();
        
        // Create some effects
        let mut effect1 = Effect::new(EffectType::CreateResource);
        effect1.set_parameter("name", b"effect1".to_vec());
        
        let mut effect2 = Effect::new(EffectType::UpdateResource);
        effect2.set_parameter("data", b"new data".to_vec());
        
        // Register the effects
        let id1 = registry.register(effect1.clone());
        let id2 = registry.register(effect2.clone());
        
        // Check that the registry contains the effects
        assert!(registry.contains(&id1));
        assert!(registry.contains(&id2));
        
        // Get the effects
        let retrieved1 = registry.get(&id1).unwrap();
        let retrieved2 = registry.get(&id2).unwrap();
        
        // Check that the retrieved effects match the originals
        assert_eq!(retrieved1, &effect1);
        assert_eq!(retrieved2, &effect2);
        
        // Check that the registry has the correct number of effects
        assert_eq!(registry.len(), 2);
        
        // Remove an effect
        let removed = registry.remove(&id1).unwrap();
        assert_eq!(removed, effect1);
        
        // Check that the registry no longer contains the removed effect
        assert!(!registry.contains(&id1));
        assert_eq!(registry.len(), 1);
    }
} 