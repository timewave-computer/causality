// Utility account implementation
// Original file: src/program_account/utility_account.rs

// Utility Program Account Implementation
//
// This module provides a specialized implementation of the ProgramAccount trait
// for data storage and common functionality.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use borsh::{BorshSerialize, BorshDeserialize};

use crate::domain::DomainId;
use causality_types::{Error, Result};
use crate::resource::{RegisterId, RegisterContents, Register, ContentId};
use causality_types::{Address, TraceId};
use crate::program_account::{
    ProgramAccount, UtilityProgramAccount, ProgramAccountCapability, ProgramAccountResource,
    AvailableEffect, EffectResult, EffectStatus, TransactionRecord, TransactionStatus
};
use causality_resource::BaseAccount;
use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// A specialized implementation of the ProgramAccount trait for utility operations
pub struct UtilityAccount {
    /// The base account implementation
    base: BaseAccount,
    
    /// Stored data in this account
    data: RwLock<HashMap<String, StoredData>>,
}

/// Represents data stored in a utility account
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct StoredData {
    /// The key for this data
    pub key: String,
    
    /// The resource ID for this data
    pub resource_id: ContentId,
    
    /// The actual data bytes
    pub data: Vec<u8>,
    
    /// Metadata for this data
    pub metadata: HashMap<String, String>,
    
    /// The domain this data belongs to (if any)
    pub domain_id: Option<DomainId>,
    
    /// When this data was created
    pub created_at: u64,
    
    /// When this data was last updated
    pub updated_at: u64,
}

impl ContentAddressed for StoredData {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
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
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl UtilityAccount {
    /// Create a new utility account
    pub fn new(
        id: String,
        owner: Address,
        name: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Self {
        Self {
            base: BaseAccount::new(
                id,
                owner,
                name,
                "utility".to_string(),
                initial_domains,
            ),
            data: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get stored data by key
    pub fn get_data_by_key(&self, key: &str) -> Result<Option<StoredData>> {
        let data = self.data.read().map_err(|_| Error::LockError)?;
        Ok(data.get(key).cloned())
    }
    
    /// Get all stored data
    pub fn get_all_data(&self) -> Result<Vec<StoredData>> {
        let data = self.data.read().map_err(|_| Error::LockError)?;
        Ok(data.values().cloned().collect())
    }
    
    /// Get all data keys
    pub fn get_data_keys(&self) -> Result<Vec<String>> {
        let data = self.data.read().map_err(|_| Error::LockError)?;
        Ok(data.keys().cloned().collect())
    }
    
    /// Update stored data
    pub fn update_data(
        &self,
        key: &str,
        new_data: &[u8],
        metadata_updates: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let mut data_map = self.data.write().map_err(|_| Error::LockError)?;
        
        let stored_data = data_map.get_mut(key)
            .ok_or_else(|| Error::NotFound(format!("Data not found with key: {}", key)))?;
        
        // Update the data
        stored_data.data = new_data.to_vec();
        
        // Update metadata if provided
        if let Some(updates) = metadata_updates {
            for (key, value) in updates {
                stored_data.metadata.insert(key, value);
            }
        }
        
        // Update the timestamp
        stored_data.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Ok(())
    }
}

impl ProgramAccount for UtilityAccount {
    fn id(&self) -> &str {
        self.base.id()
    }
    
    fn owner(&self) -> &Address {
        self.base.owner()
    }
    
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn account_type(&self) -> &str {
        self.base.account_type()
    }
    
    fn domains(&self) -> &HashSet<DomainId> {
        self.base.domains()
    }
    
    fn resources(&self) -> Vec<ProgramAccountResource> {
        self.base.resources()
    }
    
    fn get_resource(&self, resource_id: &ContentId) -> Result<Option<ProgramAccountResource>> {
        self.base.get_resource(resource_id)
    }
    
    fn available_effects(&self) -> Vec<AvailableEffect> {
        self.base.available_effects()
    }
    
    fn get_effect(&self, effect_id: &str) -> Result<Option<AvailableEffect>> {
        self.base.get_effect(effect_id)
    }
    
    fn execute_effect(
        &self,
        effect_id: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        self.base.execute_effect(effect_id, parameters, trace_id)
    }
    
    fn capabilities(&self) -> Vec<ProgramAccountCapability> {
        self.base.capabilities()
    }
    
    fn has_capability(&self, action: &str) -> bool {
        self.base.has_capability(action)
    }
    
    fn grant_capability(&mut self, capability: ProgramAccountCapability) -> Result<()> {
        self.base.grant_capability(capability)
    }
    
    fn revoke_capability(&mut self, capability_id: &str) -> Result<()> {
        self.base.revoke_capability(capability_id)
    }
    
    fn get_balance(&self, asset_id: &str) -> Result<u64> {
        self.base.get_balance(asset_id)
    }
    
    fn get_all_balances(&self) -> Result<HashMap<String, u64>> {
        self.base.get_all_balances()
    }
    
    fn transaction_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<TransactionRecord>> {
        self.base.transaction_history(limit, offset)
    }
}

impl UtilityProgramAccount for UtilityAccount {
    fn store_data(
        &self,
        key: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
        trace_id: Option<&TraceId>,
    ) -> Result<ContentId> {
        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create metadata if not provided
        let meta = metadata.unwrap_or_else(HashMap::new);
        
        // Create the stored data
        let mut stored_data = StoredData {
            key: key.to_string(),
            resource_id: ContentId::default(), // Temporary placeholder
            data: data.to_vec(),
            metadata: meta.clone(),
            domain_id: None, // In a real implementation, this would be set based on context
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        // Derive a content-based resource ID
        let content_id = stored_data.content_id();
        let resource_id = ContentId::from_str(&format!("data-{}", content_id));
        stored_data.resource_id = resource_id.clone();
        
        // Store the data
        {
            let mut data_map = self.data.write().map_err(|_| Error::LockError)?;
            
            // If the key already exists, return an error
            if data_map.contains_key(key) {
                return Err(Error::AlreadyExists(format!("Data already exists with key: {}", key)));
            }
            
            data_map.insert(key.to_string(), stored_data);
        }
        
        // Create a ProgramAccountResource for the stored data
        let resource = ProgramAccountResource {
            id: resource_id.clone(),
            register_id: None, // In a real implementation, this would be set
            resource_type: "data".to_string(),
            domain_id: None,
            metadata: meta,
        };
        
        // Register the resource with the base account
        self.base.register_resource(resource)?;
        
        // Create transaction data for content derivation
        let transaction_data = format!("store-data-{}-{}-{}", key, timestamp, resource_id);
        
        // Hash the transaction data to derive a content ID
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let transaction_content_id = ContentId::from(hasher.hash(transaction_data.as_bytes()));
        
        // Add a transaction record
        let record = TransactionRecord {
            id: format!("store-data-{}", transaction_content_id),
            transaction_type: "data_storage".to_string(),
            timestamp,
            status: TransactionStatus::Confirmed,
            resources: vec![resource_id.clone()],
            effects: Vec::new(),
            domains: Vec::new(),
            metadata: HashMap::from([
                ("key".to_string(), key.to_string()),
                ("size".to_string(), data.len().to_string()),
            ]),
        };
        
        self.base.add_transaction_record(record)?;
        
        Ok(resource_id)
    }
    
    fn retrieve_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let stored_data = self.get_data_by_key(key)?;
        
        match stored_data {
            Some(data) => Ok(Some(data.data)),
            None => Ok(None),
        }
    }
    
    fn delete_data(&self, key: &str, trace_id: Option<&TraceId>) -> Result<()> {
        // Get current data to find resource ID
        let resource_id = {
            let data_map = self.data.read().map_err(|_| Error::LockError)?;
            
            let stored_data = data_map.get(key)
                .ok_or_else(|| Error::NotFound(format!("Data not found with key: {}", key)))?;
            
            stored_data.resource_id.clone()
        };
        
        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Remove the data
        {
            let mut data_map = self.data.write().map_err(|_| Error::LockError)?;
            
            if !data_map.contains_key(key) {
                return Err(Error::NotFound(format!("Data not found with key: {}", key)));
            }
            
            data_map.remove(key);
        }
        
        // In a real implementation, this would:
        // 1. Remove the register for the data
        // 2. Clean up any references or indices
        
        // Create transaction data for content derivation
        let transaction_data = format!("delete-data-{}-{}-{}", key, timestamp, resource_id);
        
        // Hash the transaction data to derive a content ID
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let transaction_content_id = ContentId::from(hasher.hash(transaction_data.as_bytes()));
        
        // Add a transaction record
        let record = TransactionRecord {
            id: format!("delete-data-{}", transaction_content_id),
            transaction_type: "data_deletion".to_string(),
            timestamp,
            status: TransactionStatus::Confirmed,
            resources: vec![resource_id],
            effects: Vec::new(),
            domains: Vec::new(),
            metadata: HashMap::from([
                ("key".to_string(), key.to_string()),
            ]),
        };
        
        self.base.add_transaction_record(record)?;
        
        Ok(())
    }
    
    fn list_data_keys(&self) -> Result<Vec<String>> {
        self.get_data_keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utility_account_creation() {
        let account = UtilityAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Utility Account".to_string(),
            None,
        );
        
        assert_eq!(account.id(), "acc-1");
        assert_eq!(account.owner().to_string(), "owner-1");
        assert_eq!(account.name(), "Utility Account");
        assert_eq!(account.account_type(), "utility");
    }
    
    #[test]
    fn test_data_storage_and_retrieval() {
        let account = UtilityAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Utility Account".to_string(),
            None,
        );
        
        let data = b"test data".to_vec();
        let mut metadata = HashMap::new();
        metadata.insert("content-type".to_string(), "text/plain".to_string());
        
        let resource_id = account.store_data(
            "test-key",
            &data,
            Some(metadata),
            None,
        ).unwrap();
        
        // Verify the resource was created
        let resource = account.get_resource(&resource_id).unwrap().unwrap();
        assert_eq!(resource.resource_type, "data");
        
        // Retrieve the data
        let retrieved = account.retrieve_data("test-key").unwrap().unwrap();
        assert_eq!(retrieved, data);
        
        // List keys
        let keys = account.list_data_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test-key");
        
        // Get data by key
        let stored_data = account.get_data_by_key("test-key").unwrap().unwrap();
        assert_eq!(stored_data.key, "test-key");
        assert_eq!(stored_data.resource_id, resource_id);
        assert_eq!(stored_data.data, data);
        assert_eq!(stored_data.metadata.get("content-type").unwrap(), "text/plain");
    }
    
    #[test]
    fn test_data_update() {
        let account = UtilityAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Utility Account".to_string(),
            None,
        );
        
        let data = b"test data".to_vec();
        account.store_data("test-key", &data, None, None).unwrap();
        
        let new_data = b"updated data".to_vec();
        let mut metadata_updates = HashMap::new();
        metadata_updates.insert("version".to_string(), "2".to_string());
        
        account.update_data("test-key", &new_data, Some(metadata_updates)).unwrap();
        
        // Verify the data was updated
        let retrieved = account.retrieve_data("test-key").unwrap().unwrap();
        assert_eq!(retrieved, new_data);
        
        // Verify metadata was updated
        let stored_data = account.get_data_by_key("test-key").unwrap().unwrap();
        assert_eq!(stored_data.metadata.get("version").unwrap(), "2");
    }
    
    #[test]
    fn test_data_deletion() {
        let account = UtilityAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Utility Account".to_string(),
            None,
        );
        
        let data = b"test data".to_vec();
        account.store_data("test-key", &data, None, None).unwrap();
        
        account.delete_data("test-key", None).unwrap();
        
        // Verify the data was deleted
        let retrieved = account.retrieve_data("test-key").unwrap();
        assert!(retrieved.is_none());
        
        // Verify the key is gone from the list
        let keys = account.list_data_keys().unwrap();
        assert!(keys.is_empty());
    }
} 
