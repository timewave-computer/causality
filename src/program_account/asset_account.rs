// Asset Program Account Implementation
//
// This module provides a specialized implementation of the ProgramAccount trait
// for handling token and NFT assets.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use thiserror::Error;
use borsh::{BorshSerialize, BorshDeserialize};
use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::resource::{RegisterId, RegisterContents, Register, ContentId};
use crate::types::{Address, TraceId};
use crate::program_account::{
    ProgramAccount, AssetProgramAccount, ProgramAccountCapability, ProgramAccountResource,
    AvailableEffect, EffectResult, EffectStatus, TransactionRecord, TransactionStatus
};
use crate::program_account::base_account::BaseAccount;

/// Asset types supported by the AssetAccount
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// Fungible token
    Token,
    /// Non-fungible token
    NFT,
    /// Semi-fungible token (limited edition)
    SFT,
}

impl AssetType {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "token" => Some(AssetType::Token),
            "nft" => Some(AssetType::NFT),
            "sft" => Some(AssetType::SFT),
            _ => None,
        }
    }
    
    /// Convert to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            AssetType::Token => "token",
            AssetType::NFT => "nft",
            AssetType::SFT => "sft",
        }
    }
}

/// A specialized implementation of the ProgramAccount trait for assets
pub struct AssetAccount {
    /// The base account implementation
    base: BaseAccount,
    
    /// Asset collections in this account
    assets: RwLock<HashMap<String, AssetCollection>>,
}

/// Asset collection in an asset account
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct AssetCollection {
    /// Collection ID
    pub id: String,
    
    /// Collection name
    pub name: String,
    
    /// Asset type for this collection
    pub asset_type: AssetType,
    
    /// Domain this collection belongs to
    pub domain_id: DomainId,
    
    /// Metadata for this collection
    pub metadata: HashMap<String, String>,
    
    /// Assets in this collection
    pub assets: Vec<ContentId>,
}

impl ContentAddressed for AssetCollection {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
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

/// Data for transfer operation IDs
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct TransferData {
    from: String,
    to: String,
    asset_id: String,
    amount: u64,
    timestamp: u64,
}

impl ContentAddressed for TransferData {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
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

/// Data for creating asset resource IDs
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct AssetData {
    asset_type: String,
    metadata: HashMap<String, String>,
    owner: String,
    amount: u64,
    timestamp: u64,
}

impl ContentAddressed for AssetData {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
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

impl AssetAccount {
    /// Create a new asset account
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
                "asset".to_string(),
                initial_domains,
            ),
            assets: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new asset collection
    pub fn create_collection(
        &self,
        name: String,
        asset_type: AssetType,
        domain_id: DomainId,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        // Ensure the account has access to the domain
        if !self.base.domains().contains(&domain_id) {
            return Err(Error::PermissionDenied(format!(
                "Account does not have access to domain: {}", domain_id
            )));
        }
        
        // Create the collection with preliminary data
        let mut collection = AssetCollection {
            id: String::new(),  // Temporary ID will be replaced
            name,
            asset_type,
            domain_id,
            metadata,
            assets: Vec::new(),
        };
        
        // Create a content-derived ID
        let content_id = collection.content_id();
        let collection_id = format!("col-{}", content_id);
        
        // Update the ID
        collection.id = collection_id.clone();
        
        // Store the collection
        let mut assets = self.assets.write().map_err(|_| Error::LockError)?;
        assets.insert(collection_id.clone(), collection);
        
        Ok(collection_id)
    }
    
    /// Get a collection by ID
    pub fn get_collection(&self, collection_id: &str) -> Result<Option<AssetCollection>> {
        let assets = self.assets.read().map_err(|_| Error::LockError)?;
        Ok(assets.get(collection_id).cloned())
    }
    
    /// Get all collections
    pub fn get_collections(&self) -> Result<Vec<AssetCollection>> {
        let assets = self.assets.read().map_err(|_| Error::LockError)?;
        Ok(assets.values().cloned().collect())
    }
    
    /// Get collections by asset type
    pub fn get_collections_by_type(&self, asset_type: AssetType) -> Result<Vec<AssetCollection>> {
        let assets = self.assets.read().map_err(|_| Error::LockError)?;
        Ok(assets.values()
            .filter(|c| c.asset_type == asset_type)
            .cloned()
            .collect())
    }
    
    /// Add an asset to a collection
    pub fn add_asset_to_collection(&self, collection_id: &str, resource_id: ContentId) -> Result<()> {
        let mut assets = self.assets.write().map_err(|_| Error::LockError)?;
        
        let collection = assets.get_mut(collection_id)
            .ok_or_else(|| Error::NotFound(format!("Collection not found: {}", collection_id)))?;
        
        collection.assets.push(resource_id);
        
        Ok(())
    }
    
    /// Remove an asset from a collection
    pub fn remove_asset_from_collection(&self, collection_id: &str, resource_id: &ContentId) -> Result<bool> {
        let mut assets = self.assets.write().map_err(|_| Error::LockError)?;
        
        let collection = assets.get_mut(collection_id)
            .ok_or_else(|| Error::NotFound(format!("Collection not found: {}", collection_id)))?;
        
        let pos = collection.assets.iter().position(|id| id == resource_id);
        
        if let Some(index) = pos {
            collection.assets.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ProgramAccount for AssetAccount {
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

impl AssetProgramAccount for AssetAccount {
    fn transfer_asset(
        &self,
        asset_id: &str,
        recipient: &str,
        amount: u64,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Check ownership
        let asset_details = self.get_asset_details(asset_id)?;
        
        // Ensure the asset exists
        if asset_details.is_empty() {
            return Err(Error::NotFound(format!("Asset not found: {}", asset_id)));
        }
        
        // Ensure the asset is available
        let current_balance = self.get_balance(asset_id)?;
        if current_balance < amount {
            return Err(Error::InvalidInput(format!(
                "Insufficient balance: {} < {}", current_balance, amount
            )));
        }
        
        // Create transfer data
        let transfer_data = TransferData {
            from: self.id().to_string(),
            to: recipient.to_string(),
            asset_id: asset_id.to_string(),
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Generate content-derived ID
        let transfer_id = format!("transfer-{}", transfer_data.content_id());
        
        // Perform the transfer
        // This is a simplified implementation - in a real system this would involve
        // creating a transaction and sending it to the appropriate domain
        
        // Create a record of the transfer
        let record = TransactionRecord {
            id: transfer_id.clone(),
            transaction_type: "asset_transfer".to_string(),
            asset_id: asset_id.to_string(),
            amount: amount,
            from: Some(self.id().to_string()),
            to: Some(recipient.to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: TransactionStatus::Confirmed,
            trace_id: trace_id.map(|t| t.to_string()),
            metadata: HashMap::new(),
        };
        
        // Add the record to the transaction history
        self.base.add_transaction_record(record)?;
        
        // Return a successful result
        Ok(EffectResult {
            id: transfer_id,
            status: EffectStatus::Success,
            result: Some(format!("Transferred {} of asset {} to {}", amount, asset_id, recipient)),
            metadata: HashMap::new(),
        })
    }
    
    fn create_asset(
        &self,
        asset_type: &str,
        metadata: HashMap<String, String>,
        amount: Option<u64>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Validate asset type
        let asset_enum = match asset_type {
            "token" => AssetType::Token,
            "nft" => AssetType::NFT,
            "sft" => AssetType::SFT,
            _ => return Err(Error::InvalidInput(format!("Invalid asset type: {}", asset_type))),
        };

        // For NFTs, amount must be 1
        let amount = if asset_enum == AssetType::NFT {
            1
        } else {
            amount.unwrap_or(1)
        };

        // Create asset data for content addressing
        let asset_data = AssetData {
            asset_type: asset_type.to_string(),
            metadata: metadata.clone(),
            owner: self.id().to_string(),
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Generate content-derived resource ID
        let resource_id = ContentId::from_str(&format!("asset-{}", asset_data.content_id()))
            .map_err(|_| Error::ValidationError("Failed to create resource ID".to_string()))?;
        
        // Create asset creation ID
        let creation_id = format!("create-asset-{}", asset_data.content_id());

        // In a real implementation, this would:
        // 1. Register the asset in the domain
        // 2. Create a resource for the asset
        // 3. Add the asset to the appropriate collection
        
        // For now, we just create the resource and return it
        let resource = ProgramAccountResource {
            id: resource_id.to_string(),
            resource_type: format!("asset:{}", asset_type),
            name: metadata.get("name").cloned().unwrap_or_else(|| "Unnamed Asset".to_string()),
            domain: self.base.domains().iter().next().cloned().unwrap_or_default(),
            metadata: metadata.clone(),
        };
        
        // Add the resource to the account
        self.base.add_resource(resource.clone())?;
        
        // Create a record of the asset creation
        let record = TransactionRecord {
            id: creation_id.clone(),
            transaction_type: "asset_creation".to_string(),
            asset_id: resource_id.to_string(),
            amount,
            from: None,
            to: Some(self.id().to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: TransactionStatus::Confirmed,
            trace_id: trace_id.map(|t| t.to_string()),
            metadata: metadata.clone(),
        };
        
        // Add the record to the transaction history
        self.base.add_transaction_record(record)?;
        
        // Return a successful result
        Ok(EffectResult {
            id: creation_id,
            status: EffectStatus::Success,
            result: Some(format!("Created asset {} of type {}", resource_id, asset_type)),
            metadata: metadata,
        })
    }
    
    fn get_asset_details(&self, asset_id: &str) -> Result<HashMap<String, String>> {
        // Get the resource
        let resource = self.get_resource(&ContentId::from_str(asset_id))?
            .ok_or_else(|| Error::NotFound(format!("Asset not found: {}", asset_id)))?;
        
        // Return the metadata
        Ok(resource.metadata)
    }
    
    fn list_assets_by_type(&self, asset_type: &str) -> Result<Vec<ProgramAccountResource>> {
        // Parse the asset type
        let parsed_type = AssetType::from_str(asset_type)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid asset type: {}", asset_type)))?;
        
        // Get all resources
        let resources = self.resources();
        
        // Filter resources by type
        Ok(resources.into_iter()
            .filter(|r| r.resource_type == asset_type)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_account_creation() {
        let account = AssetAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Asset Account".to_string(),
            None,
        );
        
        assert_eq!(account.id(), "acc-1");
        assert_eq!(account.owner().to_string(), "owner-1");
        assert_eq!(account.name(), "Asset Account");
        assert_eq!(account.account_type(), "asset");
    }
    
    #[test]
    fn test_collection_creation() {
        let mut domains = HashSet::new();
        domains.insert(DomainId::new("domain-1"));
        
        let account = AssetAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Asset Account".to_string(),
            Some(domains),
        );
        
        let metadata = HashMap::new();
        let collection_id = account.create_collection(
            "Test Collection".to_string(),
            AssetType::NFT,
            DomainId::new("domain-1"),
            metadata,
        ).unwrap();
        
        let collection = account.get_collection(&collection_id).unwrap().unwrap();
        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.asset_type, AssetType::NFT);
        assert_eq!(collection.domain_id, DomainId::new("domain-1"));
        assert!(collection.assets.is_empty());
    }
    
    #[test]
    fn test_asset_creation() {
        let mut domains = HashSet::new();
        domains.insert(DomainId::new("domain-1"));
        
        let account = AssetAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Asset Account".to_string(),
            Some(domains),
        );
        
        let collection_id = account.create_collection(
            "Test Collection".to_string(),
            AssetType::NFT,
            DomainId::new("domain-1"),
            HashMap::new(),
        ).unwrap();
        
        let mut metadata = HashMap::new();
        metadata.insert("collection_id".to_string(), collection_id.clone());
        metadata.insert("name".to_string(), "Test NFT".to_string());
        
        let result = account.create_asset(
            "nft",
            metadata,
            None,
            None,
        ).unwrap();
        
        assert_eq!(result.status, EffectStatus::Completed);
        assert_eq!(result.new_resources.len(), 1);
        
        let resource_id = result.outputs.get("resource_id").unwrap();
        
        // Verify the asset was added to the collection
        let collection = account.get_collection(&collection_id).unwrap().unwrap();
        assert_eq!(collection.assets.len(), 1);
        assert_eq!(collection.assets[0].to_string(), *resource_id);
    }
} 
