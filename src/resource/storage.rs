//! Storage strategies and effects for the resource system
//!
//! This module defines the different storage strategies that can be used
//! for storing resources, as well as the storage effects that implement
//! the underlying storage operations.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::resource::{ResourceId, RegisterId, ResourceRegister};
use crate::domain::DomainType;
use crate::effect::{Effect, EffectContext, EffectOutcome, EffectId, EffectExecution};
use crate::error::{Error, Result};

/// Storage strategy for resources
///
/// Defines where and how a resource should be stored,
/// which can vary based on the domain and specific requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageStrategy {
    /// Store the resource directly on-chain
    OnChain,
    
    /// Store only a commitment to the resource on-chain
    Commitment,
    
    /// Store the resource in a nullifier-based system
    Nullifier,
    
    /// Store the resource in multiple locations for hybrid access patterns
    Hybrid,
}

impl StorageStrategy {
    /// Get all available storage strategies
    pub fn all() -> Vec<StorageStrategy> {
        vec![
            StorageStrategy::OnChain,
            StorageStrategy::Commitment,
            StorageStrategy::Nullifier,
            StorageStrategy::Hybrid,
        ]
    }
    
    /// Convert a string to a storage strategy
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "onchain" => Some(StorageStrategy::OnChain),
            "commitment" => Some(StorageStrategy::Commitment),
            "nullifier" => Some(StorageStrategy::Nullifier),
            "hybrid" => Some(StorageStrategy::Hybrid),
            _ => None,
        }
    }
    
    /// Convert a storage strategy to a string
    pub fn to_str(&self) -> &'static str {
        match self {
            StorageStrategy::OnChain => "onchain",
            StorageStrategy::Commitment => "commitment",
            StorageStrategy::Nullifier => "nullifier",
            StorageStrategy::Hybrid => "hybrid",
        }
    }
}

/// Storage effect for resource storage operations
///
/// This effect provides an interface for storing resources
/// using different storage strategies.
#[derive(Debug, Clone)]
pub struct StorageEffect {
    /// ID of the effect
    pub id: EffectId,
    
    /// Register to operate on
    pub register_id: RegisterId,
    
    /// Storage strategy to use
    pub strategy: StorageStrategy,
    
    /// Operation to perform
    pub operation: StorageOperation,
    
    /// Domain type for the storage
    pub domain_type: DomainType,
    
    /// Additional parameters for the storage operation
    pub params: HashMap<String, String>,
}

/// Storage operation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageOperation {
    /// Store a resource
    Store(ResourceRegister),
    
    /// Read a resource
    Read,
    
    /// Update a resource
    Update(ResourceRegister),
    
    /// Delete a resource
    Delete,
}

impl Effect for StorageEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn domain_type(&self) -> &DomainType {
        &self.domain_type
    }
    
    fn description(&self) -> String {
        match &self.operation {
            StorageOperation::Store(_) => format!(
                "Store resource {} with strategy {}",
                self.register_id,
                self.strategy.to_str(),
            ),
            StorageOperation::Read => format!(
                "Read resource {} with strategy {}",
                self.register_id,
                self.strategy.to_str(),
            ),
            StorageOperation::Update(_) => format!(
                "Update resource {} with strategy {}",
                self.register_id,
                self.strategy.to_str(),
            ),
            StorageOperation::Delete => format!(
                "Delete resource {} with strategy {}",
                self.register_id,
                self.strategy.to_str(),
            ),
        }
    }
    
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome> {
        // In a real implementation, this would be delegated to the domain adapter
        // For testing, we'll return a mock success outcome
        let outcome = match &self.operation {
            StorageOperation::Store(register) => {
                let mut data = HashMap::new();
                data.insert("register_id".to_string(), register.id.clone());
                data.insert("resource_id".to_string(), register.resource_id.clone());
                data.insert("status".to_string(), "stored".to_string());
                
                EffectOutcome {
                    id: self.id.clone(),
                    success: true,
                    data,
                    error: None,
                }
            },
            StorageOperation::Read => {
                let mut data = HashMap::new();
                data.insert("register_id".to_string(), self.register_id.clone());
                data.insert("status".to_string(), "read".to_string());
                
                // In a real implementation, this would contain the actual register data
                
                EffectOutcome {
                    id: self.id.clone(),
                    success: true,
                    data,
                    error: None,
                }
            },
            StorageOperation::Update(register) => {
                let mut data = HashMap::new();
                data.insert("register_id".to_string(), register.id.clone());
                data.insert("resource_id".to_string(), register.resource_id.clone());
                data.insert("status".to_string(), "updated".to_string());
                
                EffectOutcome {
                    id: self.id.clone(),
                    success: true,
                    data,
                    error: None,
                }
            },
            StorageOperation::Delete => {
                let mut data = HashMap::new();
                data.insert("register_id".to_string(), self.register_id.clone());
                data.insert("status".to_string(), "deleted".to_string());
                
                EffectOutcome {
                    id: self.id.clone(),
                    success: true,
                    data,
                    error: None,
                }
            },
        };
        
        Ok(outcome)
    }
    
    fn execute_async(&self, context: &EffectContext) -> EffectExecution {
        // For simplicity, we'll just execute synchronously here
        match self.execute(context) {
            Ok(outcome) => EffectExecution::Complete(outcome),
            Err(err) => EffectExecution::Error(err),
        }
    }
    
    fn dependencies(&self) -> Vec<EffectId> {
        // Storage effects typically don't have dependencies
        vec![]
    }
}

impl StorageEffect {
    /// Create a new storage effect for storing a resource
    pub fn new_store(
        register: ResourceRegister,
        strategy: StorageStrategy,
        domain_type: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id: register.id.clone(),
            strategy,
            operation: StorageOperation::Store(register),
            domain_type,
            params: HashMap::new(),
        }
    }
    
    /// Create a new storage effect for reading a resource
    pub fn new_read(
        register_id: RegisterId,
        strategy: StorageStrategy,
        domain_type: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id,
            strategy,
            operation: StorageOperation::Read,
            domain_type,
            params: HashMap::new(),
        }
    }
    
    /// Create a new storage effect for updating a resource
    pub fn new_update(
        register: ResourceRegister,
        strategy: StorageStrategy,
        domain_type: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id: register.id.clone(),
            strategy,
            operation: StorageOperation::Update(register),
            domain_type,
            params: HashMap::new(),
        }
    }
    
    /// Create a new storage effect for deleting a resource
    pub fn new_delete(
        register_id: RegisterId,
        strategy: StorageStrategy,
        domain_type: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id,
            strategy,
            operation: StorageOperation::Delete,
            domain_type,
            params: HashMap::new(),
        }
    }
    
    /// Add a parameter to the storage effect
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }
} 