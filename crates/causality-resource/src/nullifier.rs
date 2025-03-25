// Resource nullifier implementation
// Original file: src/resource/nullifier.rs

// Register Nullifier System
//
// This module implements the register nullifier system for one-time use registers
// as described in ADR-006: ZK-Based Register System for Domain Adapters.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use causality_types::{Error, Result};
use crate::resource::register::{ContentId, RegisterState, BlockHeight};
use crate::util::hash::{hash_to_hex, Hash256};

/// Represents a nullifier for a consumed register
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterNullifier {
    /// Register ID that this nullifier is for
    pub register_id: ContentId,
    
    /// The nullifier value
    pub nullifier: Hash256,
    
    /// Block height when the nullifier was created
    pub block_height: BlockHeight,
    
    /// Transaction ID that created the nullifier
    pub transaction_id: String,
    
    /// Timestamp when the nullifier was created
    pub created_at: u64,
}

impl RegisterNullifier {
    /// Create a new register nullifier
    pub fn new(
        register_id: ContentId,
        transaction_id: String,
        block_height: BlockHeight,
    ) -> Self {
        // Generate the nullifier value as a hash of the register ID and transaction ID
        let nullifier_input = format!("{}:{}", register_id, transaction_id);
        let nullifier_value = hash_to_hex(nullifier_input.as_bytes());
        let nullifier = Hash256::from_hex(&nullifier_value).unwrap();
        
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            register_id,
            nullifier,
            block_height,
            transaction_id,
            created_at,
        }
    }
    
    /// Get the hex string representation of the nullifier
    pub fn nullifier_hex(&self) -> String {
        self.nullifier.to_hex()
    }
}

/// Status of a nullifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullifierStatus {
    /// The nullifier is valid
    Valid,
    
    /// The nullifier has been used
    Used,
    
    /// The nullifier is pending verification
    PendingVerification,
    
    /// The nullifier has expired
    Expired,
    
    /// The nullifier is invalid
    Invalid,
}

impl fmt::Display for NullifierStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "Valid"),
            Self::Used => write!(f, "Used"),
            Self::PendingVerification => write!(f, "PendingVerification"),
            Self::Expired => write!(f, "Expired"),
            Self::Invalid => write!(f, "Invalid"),
        }
    }
}

/// Registry of register nullifiers
pub struct NullifierRegistry {
    /// Nullifiers by their hash value
    nullifiers: HashMap<Hash256, RegisterNullifier>,
    
    /// Nullifiers by register ID
    register_nullifiers: HashMap<ContentId, Hash256>,
    
    /// Nullifier status by hash value
    nullifier_status: HashMap<Hash256, NullifierStatus>,
    
    /// Current block height
    current_block_height: BlockHeight,
    
    /// Maximum number of blocks a pending nullifier can exist
    pending_nullifier_timeout: u64,
}

impl NullifierRegistry {
    /// Create a new nullifier registry
    pub fn new(current_block_height: BlockHeight, pending_nullifier_timeout: u64) -> Self {
        Self {
            nullifiers: HashMap::new(),
            register_nullifiers: HashMap::new(),
            nullifier_status: HashMap::new(),
            current_block_height,
            pending_nullifier_timeout,
        }
    }
    
    /// Update the current block height
    pub fn update_block_height(&mut self, block_height: BlockHeight) {
        self.current_block_height = block_height;
        
        // Expire any pending nullifiers that have been pending too long
        let timeout_block = self.current_block_height.saturating_sub(self.pending_nullifier_timeout);
        let pending_nullifiers: Vec<(Hash256, BlockHeight)> = self.nullifiers.iter()
            .filter(|(hash, _)| 
                matches!(self.nullifier_status.get(hash), Some(NullifierStatus::PendingVerification)))
            .map(|(hash, n)| (*hash, n.block_height))
            .collect();
            
        for (hash, block_height) in pending_nullifiers {
            if block_height < timeout_block {
                self.nullifier_status.insert(hash, NullifierStatus::Expired);
            }
        }
    }
    
    /// Register a new nullifier
    pub fn register_nullifier(
        &mut self,
        register_id: ContentId,
        transaction_id: String,
    ) -> Result<RegisterNullifier> {
        // Check if there's already a nullifier for this register ID
        if self.register_nullifiers.contains_key(&register_id) {
            return Err(Error::AlreadyExists(
                format!("Nullifier already exists for register {}", register_id)
            ));
        }
        
        // Create the nullifier
        let nullifier = RegisterNullifier::new(
            register_id.clone(),
            transaction_id,
            self.current_block_height,
        );
        
        // Store the nullifier
        self.nullifiers.insert(nullifier.nullifier.clone(), nullifier.clone());
        self.register_nullifiers.insert(register_id, nullifier.nullifier.clone());
        self.nullifier_status.insert(nullifier.nullifier.clone(), NullifierStatus::PendingVerification);
        
        Ok(nullifier)
    }
    
    /// Verify a nullifier (mark it as valid)
    pub fn verify_nullifier(&mut self, nullifier_hash: &Hash256) -> Result<()> {
        match self.nullifier_status.get(nullifier_hash) {
            Some(NullifierStatus::PendingVerification) => {
                self.nullifier_status.insert(nullifier_hash.clone(), NullifierStatus::Valid);
                Ok(())
            }
            Some(NullifierStatus::Valid) => Ok(()), // Already valid
            Some(status) => Err(Error::InvalidState(
                format!("Cannot verify nullifier with status {:?}", status)
            )),
            None => Err(Error::NotFound(
                format!("Nullifier {} not found", nullifier_hash.to_hex())
            )),
        }
    }
    
    /// Mark a nullifier as used
    pub fn use_nullifier(&mut self, nullifier_hash: &Hash256) -> Result<()> {
        match self.nullifier_status.get(nullifier_hash) {
            Some(NullifierStatus::Valid) => {
                self.nullifier_status.insert(nullifier_hash.clone(), NullifierStatus::Used);
                Ok(())
            }
            Some(NullifierStatus::Used) => Err(Error::AlreadyUsed(
                format!("Nullifier {} has already been used", nullifier_hash.to_hex())
            )),
            Some(status) => Err(Error::InvalidState(
                format!("Cannot use nullifier with status {:?}", status)
            )),
            None => Err(Error::NotFound(
                format!("Nullifier {} not found", nullifier_hash.to_hex())
            )),
        }
    }
    
    /// Get the status of a nullifier
    pub fn get_nullifier_status(&self, nullifier_hash: &Hash256) -> Option<NullifierStatus> {
        self.nullifier_status.get(nullifier_hash).copied()
    }
    
    /// Get the nullifier for a register
    pub fn get_nullifier_for_register(&self, register_id: &ContentId) -> Option<&RegisterNullifier> {
        self.register_nullifiers.get(register_id)
            .and_then(|hash| self.nullifiers.get(hash))
    }
    
    /// Check if a register has a nullifier
    pub fn has_nullifier(&self, register_id: &ContentId) -> bool {
        self.register_nullifiers.contains_key(register_id)
    }
    
    /// Count the number of nullifiers by status
    pub fn count_by_status(&self) -> HashMap<NullifierStatus, usize> {
        let mut counts = HashMap::new();
        
        for status in self.nullifier_status.values() {
            *counts.entry(*status).or_insert(0) += 1;
        }
        
        counts
    }
}

/// Thread-safe wrapper for the NullifierRegistry
pub struct SharedNullifierRegistry {
    inner: Arc<Mutex<NullifierRegistry>>,
}

impl SharedNullifierRegistry {
    /// Create a new shared nullifier registry
    pub fn new(current_block_height: BlockHeight, pending_nullifier_timeout: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(NullifierRegistry::new(
                current_block_height,
                pending_nullifier_timeout,
            ))),
        }
    }
    
    /// Update the current block height
    pub fn update_block_height(&self, block_height: BlockHeight) -> Result<()> {
        let mut registry = self.inner.lock().map_err(|_| Error::LockError)?;
        registry.update_block_height(block_height);
        Ok(())
    }
    
    /// Register a new nullifier
    pub fn register_nullifier(
        &self,
        register_id: ContentId,
        transaction_id: String,
    ) -> Result<RegisterNullifier> {
        let mut registry = self.inner.lock().map_err(|_| Error::LockError)?;
        registry.register_nullifier(register_id, transaction_id)
    }
    
    /// Verify a nullifier (mark it as valid)
    pub fn verify_nullifier(&self, nullifier_hash: &Hash256) -> Result<()> {
        let mut registry = self.inner.lock().map_err(|_| Error::LockError)?;
        registry.verify_nullifier(nullifier_hash)
    }
    
    /// Mark a nullifier as used
    pub fn use_nullifier(&self, nullifier_hash: &Hash256) -> Result<()> {
        let mut registry = self.inner.lock().map_err(|_| Error::LockError)?;
        registry.use_nullifier(nullifier_hash)
    }
    
    /// Get the status of a nullifier
    pub fn get_nullifier_status(&self, nullifier_hash: &Hash256) -> Result<Option<NullifierStatus>> {
        let registry = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(registry.get_nullifier_status(nullifier_hash))
    }
    
    /// Get the nullifier for a register
    pub fn get_nullifier_for_register(&self, register_id: &ContentId) -> Result<Option<RegisterNullifier>> {
        let registry = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(registry.get_nullifier_for_register(register_id).cloned())
    }
    
    /// Check if a register has a nullifier
    pub fn has_nullifier(&self, register_id: &ContentId) -> Result<bool> {
        let registry = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(registry.has_nullifier(register_id))
    }
    
    /// Count the number of nullifiers by status
    pub fn count_by_status(&self) -> Result<HashMap<NullifierStatus, usize>> {
        let registry = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(registry.count_by_status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_nullifier() {
        let register_id = ContentId::new_unique();
        let transaction_id = "test-tx-id".to_string();
        let nullifier = RegisterNullifier::new(
            register_id.clone(),
            transaction_id.clone(),
            100,
        );
        
        assert_eq!(nullifier.register_id, register_id);
        assert_eq!(nullifier.transaction_id, transaction_id);
        assert_eq!(nullifier.block_height, 100);
    }
    
    #[test]
    fn test_nullifier_registry() {
        let mut registry = NullifierRegistry::new(100, 10);
        let register_id = ContentId::new_unique();
        let transaction_id = "test-tx-id".to_string();
        
        // Register a nullifier
        let nullifier = registry.register_nullifier(
            register_id.clone(),
            transaction_id.clone(),
        ).unwrap();
        
        // Check that the nullifier exists
        assert!(registry.has_nullifier(&register_id));
        
        // Check that the nullifier is pending verification
        assert_eq!(
            registry.get_nullifier_status(&nullifier.nullifier),
            Some(NullifierStatus::PendingVerification)
        );
        
        // Verify the nullifier
        registry.verify_nullifier(&nullifier.nullifier).unwrap();
        
        // Check that the nullifier is valid
        assert_eq!(
            registry.get_nullifier_status(&nullifier.nullifier),
            Some(NullifierStatus::Valid)
        );
        
        // Use the nullifier
        registry.use_nullifier(&nullifier.nullifier).unwrap();
        
        // Check that the nullifier is used
        assert_eq!(
            registry.get_nullifier_status(&nullifier.nullifier),
            Some(NullifierStatus::Used)
        );
        
        // Trying to use the nullifier again should fail
        assert!(registry.use_nullifier(&nullifier.nullifier).is_err());
    }
    
    #[test]
    fn test_nullifier_expiration() {
        let mut registry = NullifierRegistry::new(100, 10);
        let register_id = ContentId::new_unique();
        let transaction_id = "test-tx-id".to_string();
        
        // Register a nullifier
        let nullifier = registry.register_nullifier(
            register_id.clone(),
            transaction_id.clone(),
        ).unwrap();
        
        // Check that the nullifier is pending verification
        assert_eq!(
            registry.get_nullifier_status(&nullifier.nullifier),
            Some(NullifierStatus::PendingVerification)
        );
        
        // Update the block height to expire the nullifier
        registry.update_block_height(120);
        
        // Check that the nullifier is expired
        assert_eq!(
            registry.get_nullifier_status(&nullifier.nullifier),
            Some(NullifierStatus::Expired)
        );
    }
} 
