// Effect Storage System
//
// This module provides the storage interfaces and implementations for
// persisting effects and tracking their execution history. All storage 
// follows content addressing principles to maintain immutability and 
// integrity of the effect system.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use crate::storage::{ContentAddressedStorage, ContentAddressedStorageError};
use crate::serialization::{SerializationError, to_bytes, from_bytes};

use super::{Effect, EffectId, EffectOutcome, EffectResult, EffectTypeId, EffectError};

/// Errors that can occur during effect storage operations
#[derive(Error, Debug)]
pub enum EffectStorageError {
    #[error("Effect not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Effect validation error: {0}")]
    ValidationError(String),
    
    #[error("Effect already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for effect storage operations
pub type EffectStorageResult<T> = Result<T, EffectStorageError>;

/// Effect execution record containing metadata about an executed effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectExecutionRecord {
    /// ID of the executed effect
    pub effect_id: EffectId,
    
    /// Type of the effect
    pub effect_type: EffectTypeId,
    
    /// When the effect was executed
    pub executed_at: u64,
    
    /// Outcome of the execution
    pub outcome: EffectOutcomeRecord,
    
    /// Dependencies of this effect (other effects/facts it relied on)
    pub dependencies: Vec<EffectId>,
    
    /// Domain in which the effect was executed (if applicable)
    pub domain: Option<String>,
    
    /// Metadata about the execution
    pub metadata: HashMap<String, String>,
}

/// Serializable version of the effect outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectOutcomeRecord {
    /// Effect completed successfully with result data
    Success(HashMap<String, String>),
    
    /// Effect failed with error
    Error(String),
}

impl From<EffectOutcome> for EffectOutcomeRecord {
    fn from(outcome: EffectOutcome) -> Self {
        match outcome {
            EffectOutcome::Success(data) => {
                let mut result_map = HashMap::new();
                
                // Convert the Box<dyn Any> data to a string map
                // In a real implementation, this would use proper serialization
                // of the inner data type using reflection or type information
                
                // For simplicity, assuming data is a HashMap<String, String> for now
                if let Some(map) = data.downcast_ref::<HashMap<String, String>>() {
                    result_map = map.clone();
                } else {
                    result_map.insert("result".to_string(), "Success".to_string());
                }
                
                EffectOutcomeRecord::Success(result_map)
            },
            EffectOutcome::Error(err) => {
                // Convert the error to a string representation
                EffectOutcomeRecord::Error(format!("{}", err))
            }
        }
    }
}

/// Interface for storing and retrieving effects
#[async_trait]
pub trait EffectStorage: Send + Sync + Debug {
    /// Store an effect
    async fn store_effect(&self, effect: Box<dyn Effect>) -> EffectStorageResult<EffectId>;
    
    /// Retrieve an effect by ID
    async fn get_effect(&self, effect_id: &EffectId) -> EffectStorageResult<Box<dyn Effect>>;
    
    /// Check if an effect exists
    async fn has_effect(&self, effect_id: &EffectId) -> EffectStorageResult<bool>;
    
    /// Store an execution record for an effect
    async fn store_execution_record(&self, record: EffectExecutionRecord) -> EffectStorageResult<()>;
    
    /// Get execution records for an effect
    async fn get_execution_records(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectExecutionRecord>>;
    
    /// Find effects by type
    async fn find_effects_by_type(&self, effect_type: &EffectTypeId) -> EffectStorageResult<Vec<EffectId>>;
    
    /// Find effects by domain
    async fn find_effects_by_domain(&self, domain: &str) -> EffectStorageResult<Vec<EffectId>>;
    
    /// Find effects with dependencies on the given effect
    async fn find_dependent_effects(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectId>>;
}

/// Implementation of effect storage using content-addressed storage
#[derive(Debug)]
pub struct ContentAddressedEffectStorage {
    /// Underlying content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Index of effects by type
    type_index: HashMap<EffectTypeId, HashSet<EffectId>>,
    
    /// Index of effects by domain
    domain_index: HashMap<String, HashSet<EffectId>>,
    
    /// Index of effect dependencies
    dependency_index: HashMap<EffectId, HashSet<EffectId>>,
    
    /// Execution records by effect ID
    execution_records: HashMap<EffectId, Vec<EffectExecutionRecord>>,
}

impl ContentAddressedEffectStorage {
    /// Create a new content-addressed effect storage
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            storage,
            type_index: HashMap::new(),
            domain_index: HashMap::new(),
            dependency_index: HashMap::new(),
            execution_records: HashMap::new(),
        }
    }
    
    /// Add to type index
    fn index_by_type(&mut self, effect_id: &EffectId, effect_type: &EffectTypeId) {
        let effects = self.type_index
            .entry(effect_type.clone())
            .or_insert_with(HashSet::new);
        effects.insert(effect_id.clone());
    }
    
    /// Add to domain index
    fn index_by_domain(&mut self, effect_id: &EffectId, domain: &str) {
        let effects = self.domain_index
            .entry(domain.to_string())
            .or_insert_with(HashSet::new);
        effects.insert(effect_id.clone());
    }
    
    /// Add to dependency index
    fn index_dependencies(&mut self, effect_id: &EffectId, dependencies: &[EffectId]) {
        for dep in dependencies {
            let dependents = self.dependency_index
                .entry(dep.clone())
                .or_insert_with(HashSet::new);
            dependents.insert(effect_id.clone());
        }
    }
    
    /// Store execution record
    fn store_execution_record_internal(&mut self, record: EffectExecutionRecord) {
        let records = self.execution_records
            .entry(record.effect_id.clone())
            .or_insert_with(Vec::new);
        records.push(record);
    }
}

#[async_trait]
impl EffectStorage for ContentAddressedEffectStorage {
    async fn store_effect(&self, effect: Box<dyn Effect>) -> EffectStorageResult<EffectId> {
        let effect_id = effect.id().clone();
        let effect_type = effect.type_id();
        
        // Check if already exists
        if self.has_effect(&effect_id).await? {
            return Err(EffectStorageError::AlreadyExists(
                format!("Effect already exists: {}", effect_id)
            ));
        }
        
        // Serialize the effect
        let effect_bytes = to_bytes(&effect)
            .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
        
        // Store in content-addressed storage
        let content_id = self.storage.store(&effect_bytes)
            .await
            .map_err(|e| EffectStorageError::StorageError(e.to_string()))?;
        
        // Index by type (would require locking in a thread-safe implementation)
        // In a real implementation, these indexes would be persisted
        // Either in a database or using another storage mechanism
        
        Ok(effect_id)
    }
    
    async fn get_effect(&self, effect_id: &EffectId) -> EffectStorageResult<Box<dyn Effect>> {
        // Convert effect ID to content ID
        // This would typically involve a lookup table or derivation function
        
        // Retrieve from content-addressed storage
        let effect_bytes = self.storage.get(effect_id.as_content_id())
            .await
            .map_err(|e| match e {
                ContentAddressedStorageError::NotFound(_) => 
                    EffectStorageError::NotFound(format!("Effect not found: {}", effect_id)),
                _ => EffectStorageError::StorageError(e.to_string()),
            })?;
        
        // Deserialize the effect
        // In a real implementation, this would use type information to deserialize to the correct
        // concrete effect type
        
        let effect: Box<dyn Effect> = from_bytes(&effect_bytes)
            .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
        
        Ok(effect)
    }
    
    async fn has_effect(&self, effect_id: &EffectId) -> EffectStorageResult<bool> {
        // Check in content-addressed storage
        let exists = self.storage.exists(effect_id.as_content_id())
            .await
            .map_err(|e| EffectStorageError::StorageError(e.to_string()))?;
        
        Ok(exists)
    }
    
    async fn store_execution_record(&self, record: EffectExecutionRecord) -> EffectStorageResult<()> {
        // Validate that the effect exists
        if !self.has_effect(&record.effect_id).await? {
            return Err(EffectStorageError::NotFound(
                format!("Cannot store execution record for non-existent effect: {}", record.effect_id)
            ));
        }
        
        // Serialize the record
        let record_bytes = to_bytes(&record)
            .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
        
        // Store in content-addressed storage
        // Would use a prefix or namespace to distinguish from effects
        let record_key = format!("record:{}:{}", record.effect_id, record.executed_at);
        
        self.storage.store_with_key(&record_key, &record_bytes)
            .await
            .map_err(|e| EffectStorageError::StorageError(e.to_string()))?;
        
        // In a real implementation, this would update indexes
        
        Ok(())
    }
    
    async fn get_execution_records(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectExecutionRecord>> {
        // In a real implementation, this would query the storage using a prefix search
        // or retrieve from an index
        
        // For this simplified example:
        let record_prefix = format!("record:{}:", effect_id);
        
        // Retrieve all records with the matching prefix
        let record_keys = self.storage.find_keys_with_prefix(&record_prefix)
            .await
            .map_err(|e| EffectStorageError::StorageError(e.to_string()))?;
        
        let mut records = Vec::new();
        
        for key in record_keys {
            let record_bytes = self.storage.get_by_key(&key)
                .await
                .map_err(|e| EffectStorageError::StorageError(e.to_string()))?;
            
            let record: EffectExecutionRecord = from_bytes(&record_bytes)
                .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
            
            records.push(record);
        }
        
        // Sort by execution time
        records.sort_by_key(|r| r.executed_at);
        
        Ok(records)
    }
    
    async fn find_effects_by_type(&self, effect_type: &EffectTypeId) -> EffectStorageResult<Vec<EffectId>> {
        // In a real implementation, this would query an index or database
        
        // For this simplified example:
        if let Some(effects) = self.type_index.get(effect_type) {
            Ok(effects.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn find_effects_by_domain(&self, domain: &str) -> EffectStorageResult<Vec<EffectId>> {
        // In a real implementation, this would query an index or database
        
        // For this simplified example:
        if let Some(effects) = self.domain_index.get(domain) {
            Ok(effects.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn find_dependent_effects(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectId>> {
        // In a real implementation, this would query a dependency index or database
        
        // For this simplified example:
        if let Some(dependents) = self.dependency_index.get(effect_id) {
            Ok(dependents.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
}

/// In-memory implementation of effect storage for testing
#[derive(Debug)]
pub struct InMemoryEffectStorage {
    effects: HashMap<EffectId, Vec<u8>>,
    execution_records: HashMap<EffectId, Vec<EffectExecutionRecord>>,
    type_index: HashMap<EffectTypeId, HashSet<EffectId>>,
    domain_index: HashMap<String, HashSet<EffectId>>,
    dependency_index: HashMap<EffectId, HashSet<EffectId>>,
}

impl InMemoryEffectStorage {
    /// Create a new in-memory effect storage
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            execution_records: HashMap::new(),
            type_index: HashMap::new(),
            domain_index: HashMap::new(),
            dependency_index: HashMap::new(),
        }
    }
    
    /// Add to type index
    fn index_by_type(&mut self, effect_id: &EffectId, effect_type: &EffectTypeId) {
        let effects = self.type_index
            .entry(effect_type.clone())
            .or_insert_with(HashSet::new);
        effects.insert(effect_id.clone());
    }
    
    /// Add to domain index
    fn index_by_domain(&mut self, effect_id: &EffectId, domain: &str) {
        let effects = self.domain_index
            .entry(domain.to_string())
            .or_insert_with(HashSet::new);
        effects.insert(effect_id.clone());
    }
    
    /// Add to dependency index
    fn index_dependencies(&mut self, effect_id: &EffectId, dependencies: &[EffectId]) {
        for dep in dependencies {
            let dependents = self.dependency_index
                .entry(dep.clone())
                .or_insert_with(HashSet::new);
            dependents.insert(effect_id.clone());
        }
    }
}

#[async_trait]
impl EffectStorage for InMemoryEffectStorage {
    async fn store_effect(&self, effect: Box<dyn Effect>) -> EffectStorageResult<EffectId> {
        let effect_id = effect.id().clone();
        let effect_type = effect.type_id();
        
        // Serialize the effect
        let effect_bytes = to_bytes(&effect)
            .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
        
        // Store in memory
        let mut effects = self.effects.clone();
        effects.insert(effect_id.clone(), effect_bytes);
        
        // Index by type
        let mut type_index = self.type_index.clone();
        let effects = type_index
            .entry(effect_type.clone())
            .or_insert_with(HashSet::new);
        effects.insert(effect_id.clone());
        
        // Index dependencies
        let dependencies = effect.dependencies();
        let mut dependency_index = self.dependency_index.clone();
        for dep in &dependencies {
            let dependents = dependency_index
                .entry(dep.clone())
                .or_insert_with(HashSet::new);
            dependents.insert(effect_id.clone());
        }
        
        Ok(effect_id)
    }
    
    async fn get_effect(&self, effect_id: &EffectId) -> EffectStorageResult<Box<dyn Effect>> {
        if let Some(effect_bytes) = self.effects.get(effect_id) {
            let effect: Box<dyn Effect> = from_bytes(effect_bytes)
                .map_err(|e| EffectStorageError::SerializationError(e.to_string()))?;
            
            Ok(effect)
        } else {
            Err(EffectStorageError::NotFound(format!("Effect not found: {}", effect_id)))
        }
    }
    
    async fn has_effect(&self, effect_id: &EffectId) -> EffectStorageResult<bool> {
        Ok(self.effects.contains_key(effect_id))
    }
    
    async fn store_execution_record(&self, record: EffectExecutionRecord) -> EffectStorageResult<()> {
        // Check if effect exists
        if !self.effects.contains_key(&record.effect_id) {
            return Err(EffectStorageError::NotFound(
                format!("Cannot store execution record for non-existent effect: {}", record.effect_id)
            ));
        }
        
        // Store the record
        let mut execution_records = self.execution_records.clone();
        let records = execution_records
            .entry(record.effect_id.clone())
            .or_insert_with(Vec::new);
        records.push(record.clone());
        
        // Update domain index if applicable
        if let Some(domain) = &record.domain {
            let mut domain_index = self.domain_index.clone();
            let effects = domain_index
                .entry(domain.clone())
                .or_insert_with(HashSet::new);
            effects.insert(record.effect_id.clone());
        }
        
        Ok(())
    }
    
    async fn get_execution_records(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectExecutionRecord>> {
        if let Some(records) = self.execution_records.get(effect_id) {
            Ok(records.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn find_effects_by_type(&self, effect_type: &EffectTypeId) -> EffectStorageResult<Vec<EffectId>> {
        if let Some(effects) = self.type_index.get(effect_type) {
            Ok(effects.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn find_effects_by_domain(&self, domain: &str) -> EffectStorageResult<Vec<EffectId>> {
        if let Some(effects) = self.domain_index.get(domain) {
            Ok(effects.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn find_dependent_effects(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectId>> {
        if let Some(dependents) = self.dependency_index.get(effect_id) {
            Ok(dependents.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
}

/// Feature flags for effect storage
#[derive(Debug, Clone)]
pub struct EffectStorageConfig {
    /// Whether to verify effect content hashes on retrieval
    pub verify_content_hash: bool,
    
    /// Whether to track execution history
    pub track_execution_history: bool,
    
    /// Whether to maintain indexes for querying
    pub maintain_indexes: bool,
    
    /// Whether to use batching for better performance
    pub use_batching: bool,
}

impl Default for EffectStorageConfig {
    fn default() -> Self {
        Self {
            verify_content_hash: true,
            track_execution_history: true,
            maintain_indexes: true,
            use_batching: false,
        }
    }
}

/// Create a configured effect storage
pub fn create_effect_storage(
    storage: Arc<dyn ContentAddressedStorage>,
    config: EffectStorageConfig,
) -> Arc<dyn EffectStorage> {
    // In a real implementation, this would create a storage implementation
    // with the requested features enabled/disabled
    
    Arc::new(ContentAddressedEffectStorage::new(storage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryContentAddressedStorage;
    use std::any::Any;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEffect {
        id: EffectId,
        name: String,
        dependencies: Vec<EffectId>,
    }
    
    #[async_trait]
    impl Effect for TestEffect {
        fn id(&self) -> &EffectId {
            &self.id
        }
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new(&format!("test.effect.{}", self.name))
        }
        
        fn boundary(&self) -> super::super::ExecutionBoundary {
            super::super::ExecutionBoundary::Inside
        }
        
        fn name(&self) -> String {
            format!("TestEffect({})", self.name)
        }
        
        fn is_valid(&self) -> bool {
            true
        }
        
        fn dependencies(&self) -> Vec<EffectId> {
            self.dependencies.clone()
        }
        
        fn modifications(&self) -> Vec<String> {
            vec![]
        }
        
        fn clone_effect(&self) -> Box<dyn Effect> {
            Box::new(self.clone())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[tokio::test]
    async fn test_store_and_retrieve_effect() {
        // Create an in-memory effect storage for testing
        let storage = InMemoryEffectStorage::new();
        
        // Create a test effect
        let effect_id = EffectId::new_unique();
        let effect = TestEffect {
            id: effect_id.clone(),
            name: "test".to_string(),
            dependencies: vec![],
        };
        
        // Store the effect
        let stored_id = storage.store_effect(Box::new(effect)).await.unwrap();
        assert_eq!(stored_id, effect_id);
        
        // Check if effect exists
        let exists = storage.has_effect(&effect_id).await.unwrap();
        assert!(exists);
        
        // Retrieve the effect
        let retrieved_effect = storage.get_effect(&effect_id).await.unwrap();
        assert_eq!(retrieved_effect.id(), &effect_id);
        assert_eq!(retrieved_effect.name(), "TestEffect(test)");
    }
    
    #[tokio::test]
    async fn test_execution_record() {
        // Create an in-memory effect storage for testing
        let storage = InMemoryEffectStorage::new();
        
        // Create a test effect
        let effect_id = EffectId::new_unique();
        let effect = TestEffect {
            id: effect_id.clone(),
            name: "test".to_string(),
            dependencies: vec![],
        };
        
        // Store the effect
        storage.store_effect(Box::new(effect)).await.unwrap();
        
        // Create an execution record
        let record = EffectExecutionRecord {
            effect_id: effect_id.clone(),
            effect_type: EffectTypeId::new("test.effect.test"),
            executed_at: 12345,
            outcome: EffectOutcomeRecord::Success(HashMap::new()),
            dependencies: vec![],
            domain: Some("test_domain".to_string()),
            metadata: HashMap::new(),
        };
        
        // Store the execution record
        storage.store_execution_record(record.clone()).await.unwrap();
        
        // Retrieve execution records
        let records = storage.get_execution_records(&effect_id).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].executed_at, 12345);
        
        // Find effects by domain
        let domain_effects = storage.find_effects_by_domain("test_domain").await.unwrap();
        assert_eq!(domain_effects.len(), 1);
        assert_eq!(domain_effects[0], effect_id);
    }
    
    #[tokio::test]
    async fn test_effect_dependencies() {
        // Create an in-memory effect storage for testing
        let storage = InMemoryEffectStorage::new();
        
        // Create dependency effect
        let dep_effect_id = EffectId::new_unique();
        let dep_effect = TestEffect {
            id: dep_effect_id.clone(),
            name: "dependency".to_string(),
            dependencies: vec![],
        };
        
        // Store dependency effect
        storage.store_effect(Box::new(dep_effect)).await.unwrap();
        
        // Create dependent effect
        let effect_id = EffectId::new_unique();
        let effect = TestEffect {
            id: effect_id.clone(),
            name: "dependent".to_string(),
            dependencies: vec![dep_effect_id.clone()],
        };
        
        // Store dependent effect
        storage.store_effect(Box::new(effect)).await.unwrap();
        
        // Find dependent effects
        let dependents = storage.find_dependent_effects(&dep_effect_id).await.unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], effect_id);
    }
} 