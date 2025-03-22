// Asset Program Account Implementation
//
// This module provides a specialized implementation of the ProgramAccount trait
// for handling token and NFT assets.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::resource::{RegisterId, RegisterContents, Register, ResourceId};
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

/// A collection of similar assets
#[derive(Debug, Clone)]
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
    pub assets: Vec<ResourceId>,
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
        
        // Create a new collection ID
        let collection_id = format!("col-{}", Uuid::new_v4());
        
        // Create the collection
        let collection = AssetCollection {
            id: collection_id.clone(),
            name,
            asset_type,
            domain_id,
            metadata,
            assets: Vec::new(),
        };
        
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
    pub fn add_asset_to_collection(&self, collection_id: &str, resource_id: ResourceId) -> Result<()> {
        let mut assets = self.assets.write().map_err(|_| Error::LockError)?;
        
        let collection = assets.get_mut(collection_id)
            .ok_or_else(|| Error::NotFound(format!("Collection not found: {}", collection_id)))?;
        
        collection.assets.push(resource_id);
        
        Ok(())
    }
    
    /// Remove an asset from a collection
    pub fn remove_asset_from_collection(&self, collection_id: &str, resource_id: &ResourceId) -> Result<bool> {
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
    
    fn get_resource(&self, resource_id: &ResourceId) -> Result<Option<ProgramAccountResource>> {
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
        // In a real implementation, this would:
        // 1. Look up the asset in the account
        // 2. Verify the account has enough balance
        // 3. Execute a transfer operation through the domain adapter
        // 4. Update the local state
        // 5. Return the result
        
        // For now, we just create a simulated result
        let result = EffectResult {
            id: format!("transfer-{}", Uuid::new_v4()),
            status: EffectStatus::Completed,
            transaction_id: trace_id.map(|id| id.to_string()),
            new_resources: Vec::new(),
            modified_resources: Vec::new(),
            consumed_resources: Vec::new(),
            outputs: HashMap::new(),
            error: None,
        };
        
        // Add a transaction record
        let record = TransactionRecord {
            id: result.id.clone(),
            transaction_type: "asset_transfer".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: TransactionStatus::Confirmed,
            resources: vec![ResourceId::from_str(asset_id)],
            effects: Vec::new(),
            domains: Vec::new(),
            metadata: HashMap::from([
                ("recipient".to_string(), recipient.to_string()),
                ("amount".to_string(), amount.to_string()),
            ]),
        };
        
        self.base.add_transaction_record(record)?;
        
        Ok(result)
    }
    
    fn create_asset(
        &self,
        asset_type: &str,
        metadata: HashMap<String, String>,
        amount: Option<u64>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Parse the asset type
        let parsed_type = AssetType::from_str(asset_type)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid asset type: {}", asset_type)))?;
        
        // Get the collection ID from metadata
        let collection_id = metadata.get("collection_id")
            .ok_or_else(|| Error::InvalidArgument("Missing collection_id in metadata".to_string()))?;
        
        // Get the collection
        let collection = self.get_collection(collection_id)?
            .ok_or_else(|| Error::NotFound(format!("Collection not found: {}", collection_id)))?;
        
        // Verify the asset type matches the collection
        if parsed_type != collection.asset_type {
            return Err(Error::InvalidArgument(format!(
                "Asset type {} does not match collection type {}",
                asset_type, collection.asset_type.to_str()
            )));
        }
        
        // Create a new resource ID
        let resource_id = ResourceId::from_str(&format!("asset-{}", Uuid::new_v4()));
        
        // In a real implementation, this would:
        // 1. Create a register for the asset
        // 2. Set up the appropriate metadata
        // 3. Update the collection
        
        // Add the asset to the collection
        self.add_asset_to_collection(collection_id, resource_id.clone())?;
        
        // Create a ProgramAccountResource for the new asset
        let resource = ProgramAccountResource {
            id: resource_id.clone(),
            register_id: None, // In a real implementation, this would be set
            resource_type: asset_type.to_string(),
            domain_id: Some(collection.domain_id.clone()),
            metadata: metadata.clone(),
        };
        
        // Register the resource with the base account
        self.base.register_resource(resource.clone())?;
        
        // If this is a fungible token, set the balance
        if parsed_type == AssetType::Token && amount.is_some() {
            self.base.set_balance(&resource_id.to_string(), amount.unwrap())?;
        }
        
        // Create a result
        let result = EffectResult {
            id: format!("create-asset-{}", Uuid::new_v4()),
            status: EffectStatus::Completed,
            transaction_id: trace_id.map(|id| id.to_string()),
            new_resources: vec![resource],
            modified_resources: Vec::new(),
            consumed_resources: Vec::new(),
            outputs: HashMap::from([
                ("resource_id".to_string(), resource_id.to_string()),
            ]),
            error: None,
        };
        
        // Add a transaction record
        let record = TransactionRecord {
            id: result.id.clone(),
            transaction_type: "asset_creation".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: TransactionStatus::Confirmed,
            resources: vec![resource_id],
            effects: Vec::new(),
            domains: vec![collection.domain_id.clone()],
            metadata,
        };
        
        self.base.add_transaction_record(record)?;
        
        Ok(result)
    }
    
    fn get_asset_details(&self, asset_id: &str) -> Result<HashMap<String, String>> {
        // Get the resource
        let resource = self.get_resource(&ResourceId::from_str(asset_id))?
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