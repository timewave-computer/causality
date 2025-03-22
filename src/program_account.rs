// Program Account Module
//
// This module defines the user-facing program account layer that serves
// as the touchpoint for interacting with the system.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::resource::{
    RegisterId, RegisterContents, Register, ResourceId,
    ResourceAllocator, ResourceRequest, ResourceGrant,
};
use crate::types::{Address, TraceId};

/// A capability that grants permission to perform operations on a program account
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgramAccountCapability {
    /// Unique ID for this capability
    pub id: String,
    /// The account this capability applies to
    pub account_id: String,
    /// The specific action this capability allows
    pub action: String,
    /// Any limitations on this capability
    pub restrictions: Option<HashMap<String, String>>,
    /// Expiration time for this capability (if any)
    pub expires_at: Option<u64>,
}

/// Represents a resource owned by a program account
#[derive(Debug, Clone)]
pub struct ProgramAccountResource {
    /// Unique ID for this resource
    pub id: ResourceId,
    /// The register ID for this resource (if applicable)
    pub register_id: Option<RegisterId>,
    /// The type of resource (token, NFT, data, etc.)
    pub resource_type: String,
    /// The domain this resource belongs to
    pub domain_id: Option<DomainId>,
    /// Metadata for this resource
    pub metadata: HashMap<String, String>,
}

/// Represents an available effect that can be applied to a program account
#[derive(Debug, Clone)]
pub struct AvailableEffect {
    /// Unique ID for this effect
    pub id: String,
    /// The name of the effect
    pub name: String,
    /// Description of what this effect does
    pub description: String,
    /// The domain this effect operates on
    pub domain_id: Option<DomainId>,
    /// Parameters required for this effect
    pub parameters: Vec<EffectParameter>,
    /// Whether this effect requires authorization
    pub requires_authorization: bool,
}

/// Parameter for an effect
#[derive(Debug, Clone)]
pub struct EffectParameter {
    /// The name of the parameter
    pub name: String,
    /// The type of the parameter
    pub parameter_type: String,
    /// Description of the parameter
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value for this parameter (if any)
    pub default_value: Option<String>,
}

/// Result of executing an effect
#[derive(Debug, Clone)]
pub struct EffectResult {
    /// Unique ID for this effect execution
    pub id: String,
    /// Status of the effect execution
    pub status: EffectStatus,
    /// Transaction ID (if applicable)
    pub transaction_id: Option<String>,
    /// New resources created by this effect
    pub new_resources: Vec<ProgramAccountResource>,
    /// Resources modified by this effect
    pub modified_resources: Vec<ProgramAccountResource>,
    /// Resources consumed by this effect
    pub consumed_resources: Vec<ResourceId>,
    /// Output values from the effect
    pub outputs: HashMap<String, String>,
    /// Error message (if applicable)
    pub error: Option<String>,
}

/// Status of an effect execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectStatus {
    /// Effect has been submitted but not yet processed
    Pending,
    /// Effect is currently being processed
    Processing,
    /// Effect has completed successfully
    Completed,
    /// Effect failed
    Failed,
    /// Effect was reverted
    Reverted,
}

/// Core trait for program accounts
pub trait ProgramAccount {
    /// Get the unique ID for this account
    fn id(&self) -> &str;
    
    /// Get the owner of this account
    fn owner(&self) -> &Address;
    
    /// Get the name of this account
    fn name(&self) -> &str;
    
    /// Get the type of this account
    fn account_type(&self) -> &str;
    
    /// Get the domains this account has access to
    fn domains(&self) -> &HashSet<DomainId>;
    
    /// Get the resources owned by this account
    fn resources(&self) -> Vec<ProgramAccountResource>;
    
    /// Get a specific resource by ID
    fn get_resource(&self, resource_id: &ResourceId) -> Result<Option<ProgramAccountResource>>;
    
    /// Get the available effects for this account
    fn available_effects(&self) -> Vec<AvailableEffect>;
    
    /// Get a specific effect by ID
    fn get_effect(&self, effect_id: &str) -> Result<Option<AvailableEffect>>;
    
    /// Execute an effect on this account
    fn execute_effect(
        &self,
        effect_id: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult>;
    
    /// Get the capabilities for this account
    fn capabilities(&self) -> Vec<ProgramAccountCapability>;
    
    /// Check if this account has a specific capability
    fn has_capability(&self, action: &str) -> bool;
    
    /// Grant a capability to this account
    fn grant_capability(&mut self, capability: ProgramAccountCapability) -> Result<()>;
    
    /// Revoke a capability from this account
    fn revoke_capability(&mut self, capability_id: &str) -> Result<()>;
    
    /// Get the current balance of a specific asset
    fn get_balance(&self, asset_id: &str) -> Result<u64>;
    
    /// Get all balances for this account
    fn get_all_balances(&self) -> Result<HashMap<String, u64>>;
    
    /// Get the transaction history for this account
    fn transaction_history(&self, limit: Option<usize>, offset: Option<usize>) 
        -> Result<Vec<TransactionRecord>>;
}

/// Represents a transaction record for a program account
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    /// Unique ID for this transaction
    pub id: String,
    /// The type of transaction
    pub transaction_type: String,
    /// When this transaction occurred
    pub timestamp: u64,
    /// Status of this transaction
    pub status: TransactionStatus,
    /// Resources involved in this transaction
    pub resources: Vec<ResourceId>,
    /// Effects executed in this transaction
    pub effects: Vec<String>,
    /// The domains involved in this transaction
    pub domains: Vec<DomainId>,
    /// Additional metadata for this transaction
    pub metadata: HashMap<String, String>,
}

/// Status of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction has been submitted but not yet processed
    Pending,
    /// Transaction is being processed
    Processing,
    /// Transaction has been confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction was rejected
    Rejected,
}

/// A service for managing program accounts
pub trait ProgramAccountRegistry {
    /// Create a new program account
    fn create_account(
        &self,
        owner: Address,
        name: String,
        account_type: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Result<Box<dyn ProgramAccount>>;
    
    /// Get an account by ID
    fn get_account(&self, account_id: &str) -> Result<Option<Box<dyn ProgramAccount>>>;
    
    /// Get all accounts owned by an address
    fn get_accounts_for_owner(&self, owner: &Address) -> Result<Vec<Box<dyn ProgramAccount>>>;
    
    /// Register an available effect type
    fn register_effect(
        &self,
        effect: AvailableEffect,
    ) -> Result<()>;
    
    /// Get available effect types for a domain
    fn get_effects_for_domain(&self, domain_id: &DomainId) -> Result<Vec<AvailableEffect>>;
    
    /// Register a new domain for accounts to use
    fn register_domain(&self, domain_id: DomainId) -> Result<()>;
    
    /// Get all registered domains
    fn get_domains(&self) -> Result<HashSet<DomainId>>;
}

/// An asset-specific interface for program accounts dealing with tokens/NFTs
pub trait AssetProgramAccount: ProgramAccount {
    /// Transfer an asset to another account
    fn transfer_asset(
        &self,
        asset_id: &str,
        recipient: &str,
        amount: u64,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult>;
    
    /// Create a new asset
    fn create_asset(
        &self,
        asset_type: &str,
        metadata: HashMap<String, String>,
        amount: Option<u64>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult>;
    
    /// Get details about a specific asset
    fn get_asset_details(&self, asset_id: &str) -> Result<HashMap<String, String>>;
    
    /// List all assets of a specific type
    fn list_assets_by_type(&self, asset_type: &str) -> Result<Vec<ProgramAccountResource>>;
}

/// A utility interface for common program account functionality
pub trait UtilityProgramAccount: ProgramAccount {
    /// Store arbitrary data in the account
    fn store_data(
        &self,
        key: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
        trace_id: Option<&TraceId>,
    ) -> Result<ResourceId>;
    
    /// Retrieve data from the account
    fn retrieve_data(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete data from the account
    fn delete_data(&self, key: &str, trace_id: Option<&TraceId>) -> Result<()>;
    
    /// List all stored data keys
    fn list_data_keys(&self) -> Result<Vec<String>>;
}

/// An interface for cross-domain operations
pub trait DomainBridgeProgramAccount: ProgramAccount {
    /// Transfer a resource to another domain
    fn transfer_to_domain(
        &self,
        resource_id: &ResourceId,
        target_domain: &DomainId,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult>;
    
    /// Import a resource from another domain
    fn import_from_domain(
        &self,
        source_domain: &DomainId,
        resource_reference: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult>;
    
    /// Get pending cross-domain transfers
    fn pending_transfers(&self) -> Result<Vec<CrossDomainTransfer>>;
    
    /// Get history of cross-domain transfers
    fn transfer_history(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<CrossDomainTransfer>>;
}

/// Represents a cross-domain transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransfer {
    /// Unique ID for this transfer
    pub id: String,
    /// The resource being transferred
    pub resource_id: ResourceId,
    /// The source domain
    pub source_domain: DomainId,
    /// The target domain
    pub target_domain: DomainId,
    /// Status of this transfer
    pub status: TransferStatus,
    /// When this transfer was initiated
    pub initiated_at: u64,
    /// When this transfer was completed (if applicable)
    pub completed_at: Option<u64>,
    /// Error message (if applicable)
    pub error: Option<String>,
    /// The proof of the transfer (if applicable)
    pub proof: Option<String>,
}

/// Status of a cross-domain transfer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferStatus {
    /// Transfer has been initiated
    Initiated,
    /// Resource has been locked in source domain
    SourceLocked,
    /// Proof has been generated
    ProofGenerated,
    /// Proof has been submitted to target domain
    ProofSubmitted,
    /// Resource has been created in target domain
    TargetCreated,
    /// Transfer has completed successfully
    Completed,
    /// Transfer failed
    Failed,
    /// Transfer was reverted
    Reverted,
}

/// Module for UI representation of program accounts
pub mod ui {
    use super::*;
    use serde::{Serialize, Deserialize};
    
    /// UI representation of a program account
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProgramAccountView {
        /// Unique ID for this account
        pub id: String,
        /// The owner of this account
        pub owner: String,
        /// The name of this account
        pub name: String,
        /// The type of this account
        pub account_type: String,
        /// The domains this account has access to
        pub domains: Vec<String>,
        /// The resources owned by this account
        pub resources: Vec<ResourceView>,
        /// Available effects for this account
        pub available_effects: Vec<EffectView>,
    }
    
    /// UI representation of a resource
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResourceView {
        /// Unique ID for this resource
        pub id: String,
        /// The type of resource
        pub resource_type: String,
        /// The domain this resource belongs to
        pub domain: Option<String>,
        /// Display name for this resource
        pub name: String,
        /// Short description of this resource
        pub description: Option<String>,
        /// Thumbnail or icon for this resource
        pub icon_url: Option<String>,
        /// Additional metadata for this resource
        pub metadata: HashMap<String, String>,
    }
    
    /// UI representation of an available effect
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EffectView {
        /// Unique ID for this effect
        pub id: String,
        /// The name of the effect
        pub name: String,
        /// Description of what this effect does
        pub description: String,
        /// The domain this effect operates on
        pub domain: Option<String>,
        /// Parameters required for this effect
        pub parameters: Vec<ParameterView>,
        /// Whether this effect requires authorization
        pub requires_authorization: bool,
    }
    
    /// UI representation of an effect parameter
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ParameterView {
        /// The name of the parameter
        pub name: String,
        /// The type of the parameter
        pub parameter_type: String,
        /// Description of the parameter
        pub description: String,
        /// Whether this parameter is required
        pub required: bool,
        /// Default value for this parameter (if any)
        pub default_value: Option<String>,
    }
    
    /// View transformer for program accounts
    pub trait ViewTransformer {
        /// Transform a program account into its UI representation
        fn transform_account(&self, account: &dyn ProgramAccount) -> Result<ProgramAccountView>;
        
        /// Transform a resource into its UI representation
        fn transform_resource(&self, resource: &ProgramAccountResource) -> Result<ResourceView>;
        
        /// Transform an effect into its UI representation
        fn transform_effect(&self, effect: &AvailableEffect) -> Result<EffectView>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_account_capability() {
        let capability = ProgramAccountCapability {
            id: "cap-1".to_string(),
            account_id: "acc-1".to_string(),
            action: "transfer".to_string(),
            restrictions: None,
            expires_at: None,
        };
        
        assert_eq!(capability.id, "cap-1");
        assert_eq!(capability.account_id, "acc-1");
        assert_eq!(capability.action, "transfer");
    }
    
    #[test]
    fn test_program_account_resource() {
        let resource = ProgramAccountResource {
            id: ResourceId::from_str("res-1"),
            register_id: Some(RegisterId::from_str("reg-1")),
            resource_type: "token".to_string(),
            domain_id: Some(DomainId::new("domain-1")),
            metadata: HashMap::new(),
        };
        
        assert_eq!(resource.id.to_string(), "res-1");
        assert_eq!(resource.register_id.unwrap().to_string(), "reg-1");
        assert_eq!(resource.resource_type, "token");
    }
} 