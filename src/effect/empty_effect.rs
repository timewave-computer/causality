use std::collections::HashMap;
use std::sync::Arc;
use std::any::Any;
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};

use crate::effect::{Effect, EffectContext, EffectOutcome, EffectResult, EffectId, ExecutionBoundary};
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// A simple effect that does nothing, used as a placeholder
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct SimpleEmptyEffect {
    id: EffectId,
    description: String,
}

impl ContentAddressed for SimpleEmptyEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization
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

impl SimpleEmptyEffect {
    pub fn new() -> Self {
        let description = "Empty effect that performs no action".to_string();
        let mut effect = Self {
            id: EffectId::new_unique(),
            description,
        };
        
        effect
    }
    
    pub fn with_description(description: String) -> Self {
        Self {
            id: EffectId::new_unique(),
            description,
        }
    }
}

#[async_trait]
impl Effect for SimpleEmptyEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Internal
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        Ok(EffectOutcome {
            id: self.id.as_str().to_string(),
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: context.execution_id.clone(),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }
    
    async fn validate(&self, _context: &EffectContext) -> EffectResult<()> {
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 