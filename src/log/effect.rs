use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use rand;

use crate::error::{Error, Result};
use crate::log::{LogEntry, EffectEntry, LogStorage};
use crate::log::{EntryType, EntryData};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::effect::EffectType;
use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Data for content-addressed log entry IDs
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct LogContentData {
    /// Timestamp for the entry
    timestamp: i64,
    /// Trace ID for the entry
    trace_id: Option<String>,
    /// Type of entry
    entry_type: String,
    /// Random nonce for uniqueness
    nonce: [u8; 8],
}

impl ContentAddressed for LogContentData {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Manages effect logging
pub struct EffectLogger {
    /// The underlying storage
    storage: Arc<Mutex<dyn LogStorage + Send>>,
    /// The domain ID for this logger
    domain_id: DomainId,
}

/// Metadata for an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectMetadata {
    /// The time when the effect was initiated
    pub initiated_at: DateTime<Utc>,
    /// The time when the effect was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// The initiator of the effect
    pub initiator: String,
    /// Whether the effect was successful
    pub success: bool,
    /// Error message if the effect failed
    pub error: Option<String>,
    /// Duration of the effect in milliseconds
    pub duration_ms: Option<u64>,
    /// Whether the effect is reversible
    pub reversible: bool,
    /// Whether the effect has been reversed
    pub reversed: bool,
    /// Custom metadata
    pub custom: Option<Vec<u8>>,
}

impl Default for EffectMetadata {
    fn default() -> Self {
        EffectMetadata {
            initiated_at: Utc::now(),
            completed_at: None,
            initiator: "unknown".to_string(),
            success: true,
            error: None,
            duration_ms: None,
            reversible: false,
            reversed: false,
            custom: None,
        }
    }
}

impl EffectMetadata {
    /// Create new effect metadata
    pub fn new(initiator: &str) -> Self {
        EffectMetadata {
            initiated_at: Utc::now(),
            completed_at: None,
            initiator: initiator.to_string(),
            success: true,
            error: None,
            duration_ms: None,
            reversible: false,
            reversed: false,
            custom: None,
        }
    }
    
    /// Set the completion time
    pub fn completed(mut self) -> Self {
        let now = Utc::now();
        self.completed_at = Some(now);
        let duration = now.signed_duration_since(self.initiated_at);
        self.duration_ms = Some(duration.num_milliseconds() as u64);
        self
    }
    
    /// Mark the effect as failed
    pub fn failed(mut self, error: &str) -> Self {
        self.success = false;
        self.error = Some(error.to_string());
        self.completed()
    }
    
    /// Set the reversibility status
    pub fn with_reversibility(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }
    
    /// Mark the effect as reversed
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
    
    /// Add custom metadata
    pub fn with_custom<T: Serialize>(mut self, custom: &T) -> Result<Self> {
        self.custom = Some(bincode::serialize(custom)
            .map_err(|e| Error::SerializationError(e.to_string()))?);
        Ok(self)
    }
}

impl EffectLogger {
    /// Create a new effect logger
    pub fn new(
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        domain_id: DomainId,
    ) -> Self {
        EffectLogger {
            storage,
            domain_id,
        }
    }
    
    /// Log an effect with the given type, resource ID, and data
    pub fn log_effect<T: Serialize>(
        &self,
        trace_id: TraceId,
        effect_type: EffectType,
        resource_id: ContentId,
        data: &T,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let serialized_data = bincode::serialize(data)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
            
        let _metadata = metadata.unwrap_or_default();
        
        // Create parameters map with serialized data
        let mut parameters = HashMap::new();
        parameters.insert("data".to_string(), serde_json::to_value(&serialized_data)
            .map_err(|e| Error::SerializationError(e.to_string()))?);
        
        // Create the effect entry
        let effect_entry = EffectEntry::new(
            effect_type,
            vec![resource_id],
            vec![self.domain_id.clone()],
            None, // code_hash
            parameters,
            None, // result
            true, // success
            None, // error
        );
        
        // Create content data for ID generation
        let content_data = LogContentData {
            timestamp: Utc::now().timestamp(),
            trace_id: Some(trace_id.to_string()),
            entry_type: "effect".to_string(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate a content-derived ID
        let entry_id = format!("log:{}", content_data.content_id());
        
        let log_entry = LogEntry {
            id: entry_id,
            timestamp: Utc::now(),
            entry_type: EntryType::Effect,
            data: EntryData::Effect(effect_entry),
            trace_id: Some(trace_id.to_string()),
            parent_id: None,
            metadata: HashMap::new(),
        };
        
        let storage = self.storage.lock()
            .map_err(|_| Error::LockError("Failed to lock storage".to_string()))?;
            
        storage.append(log_entry)?;
        Ok(())
    }
    
    /// Log a state change effect
    pub fn log_state_change<T: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        old_state: &T,
        new_state: &T,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let state_change = StateChangeData {
            old_state: bincode::serialize(old_state)
                .map_err(|e| Error::SerializationError(e.to_string()))?,
            new_state: bincode::serialize(new_state)
                .map_err(|e| Error::SerializationError(e.to_string()))?,
        };
        
        self.log_effect(
            trace_id,
            EffectType::Update,
            resource_id,
            &state_change,
            metadata,
        )
    }
    
    /// Log a resource creation effect
    pub fn log_resource_creation<T: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        initial_state: &T,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        self.log_effect(
            trace_id,
            EffectType::Create,
            resource_id,
            initial_state,
            metadata,
        )
    }
    
    /// Log a resource deletion effect
    pub fn log_resource_deletion<T: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        final_state: &T,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        self.log_effect(
            trace_id,
            EffectType::Delete,
            resource_id,
            final_state,
            metadata,
        )
    }
    
    /// Log a lock acquisition effect
    pub fn log_lock_acquisition(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        lock_type: &str,
        lock_timeout_ms: Option<u64>,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let lock_data = LockData {
            lock_type: lock_type.to_string(),
            lock_timeout_ms,
        };
        
        self.log_effect(
            trace_id,
            EffectType::LockAcquisition,
            resource_id,
            &lock_data,
            metadata,
        )
    }
    
    /// Log a lock release effect
    pub fn log_lock_release(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        lock_type: &str,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let lock_data = LockData {
            lock_type: lock_type.to_string(),
            lock_timeout_ms: None,
        };
        
        self.log_effect(
            trace_id,
            EffectType::LockRelease,
            resource_id,
            &lock_data,
            metadata,
        )
    }
    
    /// Log a computation effect
    pub fn log_computation<T: Serialize, U: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        input: &T,
        output: &U,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let computation_data = ComputationData {
            input: bincode::serialize(input)
                .map_err(|e| Error::SerializationError(e.to_string()))?,
            output: bincode::serialize(output)
                .map_err(|e| Error::SerializationError(e.to_string()))?,
        };
        
        self.log_effect(
            trace_id,
            EffectType::Computation,
            resource_id,
            &computation_data,
            metadata,
        )
    }
    
    /// Log a resource transfer effect
    pub fn log_resource_transfer(
        &self,
        trace_id: TraceId,
        resource_id: ContentId,
        from_owner: &str,
        to_owner: &str,
        metadata: Option<EffectMetadata>,
    ) -> Result<()> {
        let transfer_data = TransferData {
            from_owner: from_owner.to_string(),
            to_owner: to_owner.to_string(),
        };
        
        self.log_effect(
            trace_id,
            EffectType::Transfer,
            resource_id,
            &transfer_data,
            metadata,
        )
    }
}

/// Data for a state change effect
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateChangeData {
    /// The old state
    pub old_state: Vec<u8>,
    /// The new state
    pub new_state: Vec<u8>,
}

/// Data for a lock acquisition/release effect
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LockData {
    /// The lock type
    pub lock_type: String,
    /// The lock timeout in milliseconds
    pub lock_timeout_ms: Option<u64>,
}

/// Data for a computation effect
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComputationData {
    /// The input data
    pub input: Vec<u8>,
    /// The output data
    pub output: Vec<u8>,
}

/// Data for a resource transfer effect
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransferData {
    /// The previous owner
    pub from_owner: String,
    /// The new owner
    pub to_owner: String,
}

/// Utility for monitoring effects on resources
pub struct EffectMonitor {
    /// The logger
    logger: EffectLogger,
    /// The active effects
    active_effects: Mutex<Vec<ActiveEffect>>,
}

/// An active effect being monitored
#[derive(Debug, Clone)]
struct ActiveEffect {
    /// The trace ID
    pub trace_id: TraceId,
    /// The effect type
    pub effect_type: EffectType,
    /// The resource ID
    pub resource_id: ContentId,
    /// The metadata
    pub metadata: EffectMetadata,
    /// The serialized data
    pub data: Vec<u8>,
}

impl EffectMonitor {
    /// Create a new effect monitor
    pub fn new(
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        domain_id: DomainId,
    ) -> Self {
        EffectMonitor {
            logger: EffectLogger::new(storage, domain_id),
            active_effects: Mutex::new(Vec::new()),
        }
    }
    
    /// Start monitoring an effect
    pub fn start_effect<T: Serialize>(
        &self,
        trace_id: TraceId,
        effect_type: EffectType,
        resource_id: ContentId,
        data: &T,
        initiator: &str,
    ) -> Result<u64> {
        let serialized_data = bincode::serialize(data)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
            
        let metadata = EffectMetadata::new(initiator);
        
        let effect = ActiveEffect {
            trace_id: trace_id.clone(),
            effect_type: effect_type.clone(),
            resource_id,
            metadata: metadata.clone(),
            data: serialized_data.clone(),
        };
        
        let effect_id = {
            let mut active_effects = self.active_effects.lock()
                .map_err(|_| Error::LockError("Failed to lock active effects".to_string()))?;
                
            let effect_id = active_effects.len() as u64;
            active_effects.push(effect);
            effect_id
        };
        
        // Log the start of the effect
        self.logger.log_effect(
            trace_id,
            effect_type,
            resource_id,
            &serialized_data,
            Some(metadata),
        )?;
        
        Ok(effect_id)
    }
    
    /// Complete an effect
    pub fn complete_effect(&self, effect_id: u64) -> Result<()> {
        let (effect, updated_metadata) = {
            let mut active_effects = self.active_effects.lock()
                .map_err(|_| Error::LockError("Failed to lock active effects".to_string()))?;
                
            if effect_id >= active_effects.len() as u64 {
                return Err(Error::InvalidArgument(format!("Invalid effect ID: {}", effect_id)));
            }
            
            let effect = active_effects[effect_id as usize].clone();
            let updated_metadata = effect.metadata.clone().completed();
            
            // Update the metadata
            active_effects[effect_id as usize].metadata = updated_metadata.clone();
            
            (effect, updated_metadata)
        };
        
        // Log the completion of the effect
        self.logger.log_effect(
            effect.trace_id,
            effect.effect_type,
            effect.resource_id,
            &effect.data,
            Some(updated_metadata),
        )
    }
    
    /// Mark an effect as failed
    pub fn fail_effect(&self, effect_id: u64, error: &str) -> Result<()> {
        let (effect, updated_metadata) = {
            let mut active_effects = self.active_effects.lock()
                .map_err(|_| Error::LockError("Failed to lock active effects".to_string()))?;
                
            if effect_id >= active_effects.len() as u64 {
                return Err(Error::InvalidArgument(format!("Invalid effect ID: {}", effect_id)));
            }
            
            let effect = active_effects[effect_id as usize].clone();
            let updated_metadata = effect.metadata.clone().failed(error);
            
            // Update the metadata
            active_effects[effect_id as usize].metadata = updated_metadata.clone();
            
            (effect, updated_metadata)
        };
        
        // Log the failure of the effect
        self.logger.log_effect(
            effect.trace_id,
            effect.effect_type,
            effect.resource_id,
            &effect.data,
            Some(updated_metadata),
        )
    }
    
    /// Get active effects for a resource
    pub fn get_active_effects_for_resource(&self, resource_id: ContentId) -> Result<Vec<(u64, EffectType)>> {
        let active_effects = self.active_effects.lock()
            .map_err(|_| Error::LockError("Failed to lock active effects".to_string()))?;
            
        let effects: Vec<(u64, EffectType)> = active_effects.iter()
            .enumerate()
            .filter(|(_, effect)| effect.resource_id == resource_id)
            .map(|(id, effect)| (id as u64, effect.effect_type.clone()))
            .collect();
            
        Ok(effects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::MemoryLogStorage;
    
    fn create_test_logger() -> EffectLogger {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let domain_id = vec![1, 2, 3, 4];
        EffectLogger::new(storage, domain_id)
    }
    
    #[test]
    fn test_log_basic_effect() {
        let logger = create_test_logger();
        let trace_id = vec![5, 6, 7, 8];
        let data = "test data";
        
        let result = logger.log_effect(
            trace_id.clone(),
            EffectType::Update,
            123,
            &data,
            None,
        );
        
        assert!(result.is_ok());
        
        // Verify the effect was stored
        let storage = logger.storage.lock().unwrap();
        assert_eq!(storage.entry_count(), 1);
        
        if let Some(LogEntry::Effect(effect)) = storage.read_entry(0).unwrap() {
            assert_eq!(effect.effect_type(), EffectType::Update);
            assert_eq!(effect.resource_id(), Some(123));
            assert_eq!(effect.trace_id(), &trace_id);
        } else {
            panic!("Expected an effect entry");
        }
    }
    
    #[test]
    fn test_log_with_metadata() {
        let logger = create_test_logger();
        let trace_id = vec![5, 6, 7, 8];
        let data = "test data";
        
        let metadata = EffectMetadata::new("test_initiator")
            .with_reversibility(true)
            .completed();
            
        let result = logger.log_effect(
            trace_id.clone(),
            EffectType::Update,
            123,
            &data,
            Some(metadata.clone()),
        );
        
        assert!(result.is_ok());
        
        // Verify the metadata was stored correctly
        let storage = logger.storage.lock().unwrap();
        
        if let Some(LogEntry::Effect(effect)) = storage.read_entry(0).unwrap() {
            let stored_metadata: EffectMetadata = bincode::deserialize(effect.metadata()).unwrap();
            assert_eq!(stored_metadata.initiator, "test_initiator");
            assert_eq!(stored_metadata.reversible, true);
            assert!(stored_metadata.completed_at.is_some());
            assert!(stored_metadata.duration_ms.is_some());
        } else {
            panic!("Expected an effect entry");
        }
    }
    
    #[test]
    fn test_specialized_effect_types() {
        let logger = create_test_logger();
        let trace_id = vec![5, 6, 7, 8];
        
        // State change effect
        let old_state = "old state";
        let new_state = "new state";
        let result = logger.log_state_change(
            trace_id.clone(),
            123,
            &old_state,
            &new_state,
            None,
        );
        assert!(result.is_ok());
        
        // Resource creation effect
        let initial_state = "initial state";
        let result = logger.log_resource_creation(
            trace_id.clone(),
            456,
            &initial_state,
            None,
        );
        assert!(result.is_ok());
        
        // Lock acquisition effect
        let result = logger.log_lock_acquisition(
            trace_id.clone(),
            789,
            "exclusive",
            Some(1000),
            None,
        );
        assert!(result.is_ok());
        
        // Resource transfer effect
        let result = logger.log_resource_transfer(
            trace_id.clone(),
            123,
            "owner_a",
            "owner_b",
            None,
        );
        assert!(result.is_ok());
        
        // Verify all effects were stored
        let storage = logger.storage.lock().unwrap();
        assert_eq!(storage.entry_count(), 4);
    }
    
    #[test]
    fn test_effect_monitor() {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let domain_id = vec![1, 2, 3, 4];
        let monitor = EffectMonitor::new(storage.clone(), domain_id);
        
        let trace_id = vec![5, 6, 7, 8];
        let data = "test data";
        
        // Start effect
        let effect_id = monitor.start_effect(
            trace_id.clone(),
            EffectType::Update,
            123,
            &data,
            "test_monitor",
        ).unwrap();
        
        // Complete effect
        let result = monitor.complete_effect(effect_id);
        assert!(result.is_ok());
        
        // Start another effect and fail it
        let effect_id2 = monitor.start_effect(
            trace_id.clone(),
            EffectType::LockAcquisition,
            456,
            &"lock data",
            "test_monitor",
        ).unwrap();
        
        let result = monitor.fail_effect(effect_id2, "Lock acquisition failed");
        assert!(result.is_ok());
        
        // Check active effects for a resource
        let active_effects = monitor.get_active_effects_for_resource(123).unwrap();
        assert_eq!(active_effects.len(), 1);
        
        // Verify the effects were stored
        let storage_lock = storage.lock().unwrap();
        assert_eq!(storage_lock.entry_count(), 4); // 2 starts + 2 completions
        
        // Verify the metadata of the completed effect
        if let Some(LogEntry::Effect(effect)) = storage_lock.read_entry(1).unwrap() {
            let metadata: EffectMetadata = bincode::deserialize(effect.metadata()).unwrap();
            assert!(metadata.completed_at.is_some());
            assert!(metadata.duration_ms.is_some());
            assert_eq!(metadata.success, true);
        } else {
            panic!("Expected an effect entry");
        }
        
        // Verify the metadata of the failed effect
        if let Some(LogEntry::Effect(effect)) = storage_lock.read_entry(3).unwrap() {
            let metadata: EffectMetadata = bincode::deserialize(effect.metadata()).unwrap();
            assert!(metadata.completed_at.is_some());
            assert!(metadata.duration_ms.is_some());
            assert_eq!(metadata.success, false);
            assert_eq!(metadata.error, Some("Lock acquisition failed".to_string()));
        } else {
            panic!("Expected an effect entry");
        }
    }
} 
