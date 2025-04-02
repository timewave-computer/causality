// Domain capabilities
//
// This module provides domain-specific capability types and functionality
// that integrate with the core capability system.

use std::any::Any;
use std::collections::{HashSet, HashMap};
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

// Fix imports to use the correct types
use crate::capability::{ResourceId, ContentAddressingError, ContentRef, CapabilityGrants, Capability, ResourceGuard, ResourceRegistry, CapabilityError, IdentityId};
use crate::capability::utils;
use causality_types::{ContentHash, ContentId};
use std::marker::PhantomData;

// Domain-specific capability that can be supported by domain adapters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainCapabilityType {
    // Transaction capabilities
    SendTransaction,
    SignTransaction,
    BatchTransactions,
    
    // Smart contract capabilities
    DeployContract,
    ExecuteContract,
    QueryContract,
    
    // State capabilities
    ReadState,
    WriteState,
    
    // Cryptographic capabilities
    VerifySignature,
    GenerateProof,
    VerifyProof,
    
    // ZK capabilities
    ZkProve,
    ZkVerify,
    
    // Consensus capabilities
    Stake,
    Validate,
    Vote,
    
    // Governance capabilities
    ProposeUpgrade,
    VoteOnProposal,
    
    // Cross-domain capabilities
    BridgeAssets,
    VerifyBridgeTransaction,
    
    // Custom capability (with name)
    Custom(String)
}

impl DomainCapabilityType {
    /// Convert a domain capability to a string
    pub fn to_string(&self) -> String {
        match self {
            DomainCapabilityType::SendTransaction => "send_transaction".to_string(),
            DomainCapabilityType::SignTransaction => "sign_transaction".to_string(),
            DomainCapabilityType::BatchTransactions => "batch_transactions".to_string(),
            DomainCapabilityType::DeployContract => "deploy_contract".to_string(),
            DomainCapabilityType::ExecuteContract => "execute_contract".to_string(),
            DomainCapabilityType::QueryContract => "query_contract".to_string(),
            DomainCapabilityType::ReadState => "read_state".to_string(),
            DomainCapabilityType::WriteState => "write_state".to_string(),
            DomainCapabilityType::VerifySignature => "verify_signature".to_string(),
            DomainCapabilityType::GenerateProof => "generate_proof".to_string(),
            DomainCapabilityType::VerifyProof => "verify_proof".to_string(),
            DomainCapabilityType::ZkProve => "zk_prove".to_string(),
            DomainCapabilityType::ZkVerify => "zk_verify".to_string(),
            DomainCapabilityType::Stake => "stake".to_string(),
            DomainCapabilityType::Validate => "validate".to_string(),
            DomainCapabilityType::Vote => "vote".to_string(),
            DomainCapabilityType::ProposeUpgrade => "propose_upgrade".to_string(),
            DomainCapabilityType::VoteOnProposal => "vote_on_proposal".to_string(),
            DomainCapabilityType::BridgeAssets => "bridge_assets".to_string(),
            DomainCapabilityType::VerifyBridgeTransaction => "verify_bridge_transaction".to_string(),
            DomainCapabilityType::Custom(name) => format!("custom_{}", name),
        }
    }
    
    /// Create a capability from a domain capability type
    pub fn create_capability(&self, grants: CapabilityGrants, owner: IdentityId) -> DomainCapability {
        let id = self.create_resource_id();
        
        DomainCapability {
            capability_type: self.clone(),
            grants,
            id,
            origin: Some(owner),
            content_hash: None,
        }
    }
    
    /// Create a resource ID for a domain capability
    fn create_resource_id(&self) -> ResourceId {
        let capability_str = self.to_string();
        let id_str = format!("domain_{}", capability_str);
        ResourceId::new(utils::hash_string(&id_str))
    }
}

/// A domain-specific capability
#[derive(Debug, Clone)]
pub struct DomainCapability {
    /// The domain capability type
    pub capability_type: DomainCapabilityType,
    
    /// The capability grants
    pub grants: CapabilityGrants,
    
    /// The identifier for the capability
    pub id: ResourceId,
    
    /// The origin identity that created the capability
    pub origin: Option<IdentityId>,
    
    /// The content hash if content-addressed
    pub content_hash: Option<ContentHash>,
}

impl DomainCapability {
    /// Convert to a generic capability
    pub fn to_capability<T: Send + Sync + 'static>(&self) -> Capability<T> {
        Capability {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            _phantom: PhantomData,
        }
    }
    
    /// Create a new domain capability
    pub fn new(
        capability_type: DomainCapabilityType,
        grants: CapabilityGrants,
        owner: IdentityId
    ) -> Self {
        capability_type.create_capability(grants, owner)
    }
    
    /// Create a content-addressed version of this capability
    pub fn to_content_addressed(&self, content_hash: ContentHash) -> Self {
        let mut capability = self.clone();
        capability.content_hash = Some(content_hash);
        capability
    }
    
    /// Get the content hash (if content-addressed)
    pub fn content_hash(&self) -> Option<&ContentHash> {
        self.content_hash.as_ref()
    }
    
    /// Check if this capability is content-addressed
    pub fn is_content_addressed(&self) -> bool {
        self.content_hash.is_some()
    }
    
    /// Check if this capability applies to a specific content reference
    pub fn applies_to<T: ?Sized>(
        &self,
        content_ref: &ContentRef<T>,
    ) -> bool {
        if let Some(content_hash) = &self.content_hash {
            &content_ref.hash == content_hash
        } else {
            false
        }
    }
}

/// Errors specific to domain capabilities
#[derive(Debug, Error)]
pub enum DomainCapabilityError {
    #[error("Invalid capability type: {0}")]
    InvalidCapabilityType(String),
    
    #[error("Missing required grants")]
    MissingGrants,
    
    #[error("Underlying capability error")]
    CapabilityError(Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Content addressing error: {0}")]
    ContentAddressingError(String),
}

/// A registry for domain capabilities
#[derive(Debug)]
pub struct DomainCapabilityRegistry {
    /// The underlying resource registry
    registry: ResourceRegistry,
    
    /// Default domain capabilities by domain type
    domain_types: HashMap<String, HashSet<DomainCapabilityType>>,
}

impl DomainCapabilityRegistry {
    /// Create a new domain capability registry
    pub fn new() -> Self {
        Self {
            registry: ResourceRegistry::new(),
            domain_types: HashMap::new(),
        }
    }
    
    /// Register a domain type with its capabilities
    pub fn register_domain_type(&mut self, domain_type: &str, capabilities: HashSet<DomainCapabilityType>) {
        self.domain_types.insert(domain_type.to_string(), capabilities);
    }
    
    /// Get the capabilities for a domain type
    pub fn get_domain_capabilities(&self, domain_type: &str) -> Option<&HashSet<DomainCapabilityType>> {
        self.domain_types.get(domain_type)
    }
    
    /// Register a resource and create a domain capability for it
    pub fn register<T: Send + Sync + 'static + serde::Serialize>(
        &mut self,
        resource: T,
        owner: IdentityId,
        capability_type: DomainCapabilityType,
    ) -> Result<DomainCapability, CapabilityError> {
        // Register the resource with the underlying registry
        let capability = self.registry.register(resource, owner.clone())?;
        
        // Create a domain capability
        let domain_capability = DomainCapability {
            capability_type,
            grants: capability.grants.clone(),
            id: capability.id.clone(),
            origin: capability.origin.clone(),
            content_hash: None,
        };
        
        Ok(domain_capability)
    }
    
    /// Access a resource using a domain capability
    pub fn access<T: Send + Sync + Clone + 'static>(
        &self,
        capability: &DomainCapability,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        // Convert to a generic capability
        let generic_capability = capability.to_capability::<T>();
        
        // Access using the underlying registry
        self.registry.access(&generic_capability)
    }
    
    /// Access a resource by content reference
    pub fn access_by_content<T: Send + Sync + Clone + 'static>(
        &self,
        content_ref: &ContentRef<T>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        self.registry.access_by_content(content_ref)
    }
    
    /// Transfer a capability from one identity to another
    pub fn transfer_capability(
        &mut self,
        capability: &DomainCapability,
        from: &IdentityId,
        to: &IdentityId,
    ) -> Result<(), CapabilityError> {
        // Convert to a generic capability for transfer
        let generic_capability = capability.to_capability::<Box<dyn Any + Send + Sync>>();
        
        // Transfer using the underlying registry
        self.registry.transfer_capability(&generic_capability, from, to)
    }
}

/// Helper functions for domain capabilities
pub mod helpers {
    use super::*;
    
    /// Create a domain capability registry
    pub fn create_domain_registry() -> DomainCapabilityRegistry {
        DomainCapabilityRegistry::new()
    }
    
    /// Create a domain capability registry with default capabilities
    pub fn create_domain_registry_with_defaults() -> DomainCapabilityRegistry {
        let mut registry = DomainCapabilityRegistry::new();
        
        // EVM domain capabilities
        let mut evm_capabilities = HashSet::new();
        evm_capabilities.insert(DomainCapabilityType::SendTransaction);
        evm_capabilities.insert(DomainCapabilityType::DeployContract);
        evm_capabilities.insert(DomainCapabilityType::ExecuteContract);
        evm_capabilities.insert(DomainCapabilityType::ReadState);
        registry.register_domain_type("evm", evm_capabilities);
        
        // CosmWasm domain capabilities
        let mut cosmwasm_capabilities = HashSet::new();
        cosmwasm_capabilities.insert(DomainCapabilityType::SendTransaction);
        cosmwasm_capabilities.insert(DomainCapabilityType::DeployContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::ExecuteContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::QueryContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::ReadState);
        registry.register_domain_type("cosmwasm", cosmwasm_capabilities);
        
        // Solana domain capabilities
        let mut solana_capabilities = HashSet::new();
        solana_capabilities.insert(DomainCapabilityType::SendTransaction);
        solana_capabilities.insert(DomainCapabilityType::DeployContract);
        solana_capabilities.insert(DomainCapabilityType::ExecuteContract);
        solana_capabilities.insert(DomainCapabilityType::ReadState);
        registry.register_domain_type("solana", solana_capabilities);
        
        registry
    }
    
    /// Create a read state capability
    pub fn create_read_state_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::ReadState,
            CapabilityGrants::read_only(),
            owner,
        )
    }
    
    /// Create a transaction capability
    pub fn create_transaction_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::SendTransaction,
            CapabilityGrants::full(),
            owner,
        )
    }
    
    /// Create a contract execution capability
    pub fn create_contract_execution_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::ExecuteContract,
            CapabilityGrants::new(true, true, false), // read+write
            owner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestIdentityId(String);
    
    impl TestIdentityId {
        fn new(id: impl Into<String>) -> Self {
            Self(id.into())
        }
    }
    
    impl From<TestIdentityId> for IdentityId {
        fn from(id: TestIdentityId) -> Self {
            id.0
        }
    }
    
    fn create_test_identity() -> IdentityId {
        TestIdentityId::new("test_identity").into()
    }
    
    #[test]
    fn test_domain_capability_types() {
        // Test string conversion
        assert_eq!(DomainCapabilityType::ReadState.to_string(), "read_state");
        assert_eq!(DomainCapabilityType::ExecuteContract.to_string(), "execute_contract");
        assert_eq!(DomainCapabilityType::Custom("my_custom".to_string()).to_string(), "custom_my_custom");
        
        // Test resource ID creation
        let id = DomainCapabilityType::ReadState.create_resource_id();
        assert!(id.name.is_none());
    }
    
    #[test]
    fn test_domain_capability_registry() {
        let mut registry = DomainCapabilityRegistry::new();
        let identity = create_test_identity();
        
        // Create a test value
        let value = "test value".to_string();
        
        // Test registration
        let capability = registry.register(
            value,
            identity.clone(),
            DomainCapabilityType::ReadState,
        ).unwrap();
        
        assert_eq!(capability.capability_type, DomainCapabilityType::ReadState);
        assert!(capability.grants.allows_read());
        
        // TODO: More tests for access, transfer, etc. once implementation is complete
    }
    
    #[test]
    fn test_domain_capability_helpers() {
        let registry = helpers::create_domain_registry_with_defaults();
        let identity = create_test_identity();
        
        // Check default domain types
        assert!(registry.get_domain_capabilities("evm").is_some());
        assert!(registry.get_domain_capabilities("cosmwasm").is_some());
        assert!(registry.get_domain_capabilities("solana").is_some());
        
        // Check helper functions for creating capabilities
        let read_cap = helpers::create_read_state_capability(identity.clone());
        assert_eq!(read_cap.capability_type, DomainCapabilityType::ReadState);
        assert!(read_cap.grants.allows_read());
        assert!(!read_cap.grants.allows_write());
        
        let tx_cap = helpers::create_transaction_capability(identity.clone());
        assert_eq!(tx_cap.capability_type, DomainCapabilityType::SendTransaction);
        assert!(tx_cap.grants.allows_read());
        assert!(tx_cap.grants.allows_write());
        assert!(tx_cap.grants.allows_delegation());
    }
} 