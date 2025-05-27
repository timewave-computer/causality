// Effect Storage
//
// This module provides storage functionality for effect execution records,
// outcomes, and related data using content addressing.

use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use causality_types::content_addressing::storage::ContentAddressedStorage;
use causality_types::content_addressing::storage::error::ContentAddressedStorageError;
use causality_types::ContentId;

use crate::serialization::{Serializer, SerializationError, to_bytes, from_bytes};
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
/// Represents the data to be stored.
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
/// Represents the data to be stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectOutcomeRecord {
    /// Effect completed successfully with result data
    Success(HashMap<String, String>),
    
    /// Effect failed with error
    Error(String),
}

// Note: The From<EffectOutcome> implementation is closely tied to how Success data is stored.
// This might need refinement or move closer to where EffectOutcome is actually constructed
// and stored if the storage format changes.
impl From<EffectOutcome> for EffectOutcomeRecord {
    fn from(outcome: EffectOutcome) -> Self {
        match outcome {
            EffectOutcome::Success(data) => {
                let mut result_map = HashMap::new();
                // TODO: Implement robust serialization of the Box<dyn Any> data.
                // This current implementation is a placeholder and likely incorrect.
                // It assumes the data is always HashMap<String, String>.
                if let Some(map) = data.downcast_ref::<HashMap<String, String>>() {
                    result_map = map.clone();
                } else {
                    // Provide a default representation if downcast fails
                    result_map.insert("result_type".to_string(), format!("{:?}", data.type_id()));
                    result_map.insert("result_placeholder".to_string(), "Success (Opaque Data)".to_string());
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

/// Interface for storing and retrieving effects and their execution records.
#[async_trait]
pub trait EffectStorage: Send + Sync + Debug {
    /// Store an effect representation.
    /// Note: This stores the *definition* or *instance* of an effect, not its execution.
    /// The specific serialization format is up to the implementation.
    async fn store_effect(&self, effect: Arc<dyn Effect>) -> EffectStorageResult<ContentId>;
    
    /// Retrieve an effect representation by its ContentId.
    /// The implementation needs to deserialize the stored data back into an `Effect` trait object.
    /// This likely requires type information or a registry.
    async fn get_effect(&self, id: &ContentId) -> EffectStorageResult<Arc<dyn Effect>>;
    
    /// Check if an effect representation exists by its ContentId.
    async fn has_effect(&self, id: &ContentId) -> EffectStorageResult<bool>;
    
    /// Store an execution record for an effect.
    /// The record links an effect instance (by ID) to its outcome and metadata.
    async fn store_execution_record(&self, record: EffectExecutionRecord) -> EffectStorageResult<()>;
    
    /// Get all execution records associated with a specific effect ID.
    async fn get_execution_records(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<EffectExecutionRecord>>;
    
    // --- Optional Query Methods --- 
    // Implementations may choose to support these for querying stored data.
    
    // /// Find effects by type.
    // async fn find_effects_by_type(&self, effect_type: &EffectTypeId) -> EffectStorageResult<Vec<ContentId>>;
    
    // /// Find effects by domain.
    // async fn find_effects_by_domain(&self, domain: &str) -> EffectStorageResult<Vec<ContentId>>;
    
    // /// Find effects with dependencies on the given effect ID.
    // async fn find_dependent_effects(&self, effect_id: &EffectId) -> EffectStorageResult<Vec<ContentId>>;
}

// --- Implementation Details Removed ---
// Concrete structs like ContentAddressedEffectStorage, InMemoryEffectStorage,
// EffectStorageConfig, create_effect_storage function, and tests
// have been removed from this file. They belong in specific storage crates
// (e.g., causality-storage-mem, causality-storage-rocksdb) or potentially
// causality-runtime for default implementations. 