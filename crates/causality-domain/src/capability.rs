// Domain capability system
// Original file: src/domain/capability.rs

// Domain Capabilities System
//
// This module implements a capability system specifically for domain adapters,
// integrating with the general capability system.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::domain::{DomainId, DomainType, DomainAdapter};
use causality_types::{Error, Result};
use crate::resource::{
    CapabilityId, ContentId, Right,
    capability_system::{
        RigorousCapability, CapabilityConstraint, CapabilitySystem,
        CapabilityStatus, AuthenticationFactor
    }
};
use causality_types::Address;

/// Standard domain capabilities that can be supported by domain adapters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCapability {
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

impl DomainCapability {
    /// Convert a domain capability to a string
    pub fn to_string(&self) -> String {
        match self {
            DomainCapability::SendTransaction => "send_transaction".to_string(),
            DomainCapability::SignTransaction => "sign_transaction".to_string(),
            DomainCapability::BatchTransactions => "batch_transactions".to_string(),
            DomainCapability::DeployContract => "deploy_contract".to_string(),
            DomainCapability::ExecuteContract => "execute_contract".to_string(),
            DomainCapability::QueryContract => "query_contract".to_string(),
            DomainCapability::ReadState => "read_state".to_string(),
            DomainCapability::WriteState => "write_state".to_string(),
            DomainCapability::VerifySignature => "verify_signature".to_string(),
            DomainCapability::GenerateProof => "generate_proof".to_string(),
            DomainCapability::VerifyProof => "verify_proof".to_string(),
            DomainCapability::ZkProve => "zk_prove".to_string(),
            DomainCapability::ZkVerify => "zk_verify".to_string(),
            DomainCapability::Stake => "stake".to_string(),
            DomainCapability::Validate => "validate".to_string(),
            DomainCapability::Vote => "vote".to_string(),
            DomainCapability::ProposeUpgrade => "propose_upgrade".to_string(),
            DomainCapability::VoteOnProposal => "vote_on_proposal".to_string(),
            DomainCapability::BridgeAssets => "bridge_assets".to_string(),
            DomainCapability::VerifyBridgeTransaction => "verify_bridge_transaction".to_string(),
            DomainCapability::Custom(name) => format!("custom_{}", name),
        }
    }
    
    /// Convert a string to a domain capability
    pub fn from_string(s: &str) -> Option<DomainCapability> {
        match s {
            "send_transaction" => Some(DomainCapability::SendTransaction),
            "sign_transaction" => Some(DomainCapability::SignTransaction),
            "batch_transactions" => Some(DomainCapability::BatchTransactions),
            "deploy_contract" => Some(DomainCapability::DeployContract),
            "execute_contract" => Some(DomainCapability::ExecuteContract),
            "query_contract" => Some(DomainCapability::QueryContract),
            "read_state" => Some(DomainCapability::ReadState),
            "write_state" => Some(DomainCapability::WriteState),
            "verify_signature" => Some(DomainCapability::VerifySignature),
            "generate_proof" => Some(DomainCapability::GenerateProof),
            "verify_proof" => Some(DomainCapability::VerifyProof),
            "zk_prove" => Some(DomainCapability::ZkProve),
            "zk_verify" => Some(DomainCapability::ZkVerify),
            "stake" => Some(DomainCapability::Stake),
            "validate" => Some(DomainCapability::Validate),
            "vote" => Some(DomainCapability::Vote),
            "propose_upgrade" => Some(DomainCapability::ProposeUpgrade),
            "vote_on_proposal" => Some(DomainCapability::VoteOnProposal),
            "bridge_assets" => Some(DomainCapability::BridgeAssets),
            "verify_bridge_transaction" => Some(DomainCapability::VerifyBridgeTransaction),
            s if s.starts_with("custom_") => Some(DomainCapability::Custom(s[7..].to_string())),
            _ => None,
        }
    }
    
    /// Get capability based on domain type
    pub fn capabilities_for_domain_type(domain_type: &DomainType) -> HashSet<DomainCapability> {
        let mut capabilities = HashSet::new();
        
        // Common capabilities for all domains
        capabilities.insert(DomainCapability::SendTransaction);
        capabilities.insert(DomainCapability::ReadState);
        
        // Add domain-specific capabilities
        match domain_type {
            DomainType::EVM => {
                capabilities.insert(DomainCapability::DeployContract);
                capabilities.insert(DomainCapability::ExecuteContract);
                capabilities.insert(DomainCapability::BatchTransactions);
                capabilities.insert(DomainCapability::WriteState);
                capabilities.insert(DomainCapability::VerifySignature);
            },
            DomainType::CosmWasm => {
                capabilities.insert(DomainCapability::DeployContract);
                capabilities.insert(DomainCapability::ExecuteContract);
                capabilities.insert(DomainCapability::QueryContract);
                capabilities.insert(DomainCapability::WriteState);
                capabilities.insert(DomainCapability::VerifySignature);
                capabilities.insert(DomainCapability::Stake);
                capabilities.insert(DomainCapability::Validate);
                capabilities.insert(DomainCapability::Vote);
                capabilities.insert(DomainCapability::ProposeUpgrade);
                capabilities.insert(DomainCapability::VoteOnProposal);
            },
            DomainType::SOL => {
                capabilities.insert(DomainCapability::DeployContract);
                capabilities.insert(DomainCapability::ExecuteContract);
                capabilities.insert(DomainCapability::WriteState);
                capabilities.insert(DomainCapability::VerifySignature);
                capabilities.insert(DomainCapability::Stake);
                capabilities.insert(DomainCapability::Validate);
            },
            DomainType::TEL => {
                capabilities.insert(DomainCapability::ExecuteContract);
                capabilities.insert(DomainCapability::QueryContract);
                capabilities.insert(DomainCapability::ZkProve);
                capabilities.insert(DomainCapability::ZkVerify);
            },
            DomainType::Unknown => {
                // Only basic capabilities for unknown domain types
            },
        }
        
        capabilities
    }
}

/// Domain capability manager that integrates with the resource capability system
pub struct DomainCapabilityManager {
    /// Reference to the general capability system
    capability_system: Arc<dyn CapabilitySystem>,
    
    /// Default domain capabilities by domain type
    default_capabilities: HashMap<DomainType, HashSet<DomainCapability>>,
    
    /// Cache of domain capabilities by domain ID
    domain_capabilities: HashMap<DomainId, HashSet<DomainCapability>>,
}

impl DomainCapabilityManager {
    /// Create a new domain capability manager
    pub fn new(capability_system: Arc<dyn CapabilitySystem>) -> Self {
        let mut default_capabilities = HashMap::new();
        
        // Initialize default capabilities for each domain type
        default_capabilities.insert(DomainType::EVM, DomainCapability::capabilities_for_domain_type(&DomainType::EVM));
        default_capabilities.insert(DomainType::CosmWasm, DomainCapability::capabilities_for_domain_type(&DomainType::CosmWasm));
        default_capabilities.insert(DomainType::SOL, DomainCapability::capabilities_for_domain_type(&DomainType::SOL));
        default_capabilities.insert(DomainType::TEL, DomainCapability::capabilities_for_domain_type(&DomainType::TEL));
        default_capabilities.insert(DomainType::Unknown, DomainCapability::capabilities_for_domain_type(&DomainType::Unknown));
        
        Self {
            capability_system,
            default_capabilities,
            domain_capabilities: HashMap::new(),
        }
    }
    
    /// Register domain capabilities for a specific domain
    pub fn register_domain_capabilities(
        &mut self,
        domain_id: &DomainId,
        capabilities: HashSet<DomainCapability>,
    ) {
        self.domain_capabilities.insert(domain_id.clone(), capabilities);
    }
    
    /// Get registered capabilities for a domain
    pub fn get_domain_capabilities(&self, domain_id: &DomainId) -> Option<&HashSet<DomainCapability>> {
        self.domain_capabilities.get(domain_id)
    }
    
    /// Check if a domain has a specific capability
    pub fn domain_has_capability(&self, domain_id: &DomainId, capability: &DomainCapability) -> bool {
        if let Some(capabilities) = self.domain_capabilities.get(domain_id) {
            capabilities.contains(capability)
        } else {
            false
        }
    }
    
    /// Register domain adapter capabilities based on the adapter's capabilities
    pub fn register_domain_adapter(&mut self, adapter: &dyn DomainAdapter) -> Result<()> {
        let domain_id = adapter.domain_id();
        let domain_info = adapter.domain_info().now_or_never()
            .unwrap_or(Err(Error::UnsupportedOperation("Failed to get domain info".to_string())))??;
        
        // Get capabilities from adapter
        let adapter_capabilities: HashSet<DomainCapability> = adapter.capabilities()
            .iter()
            .filter_map(|cap_str| DomainCapability::from_string(cap_str))
            .collect();
        
        // Get default capabilities for this domain type
        let default_caps = self.default_capabilities
            .get(&domain_info.domain_type)
            .cloned()
            .unwrap_or_default();
        
        // Merge adapter capabilities with defaults
        let mut capabilities = default_caps.clone();
        capabilities.extend(adapter_capabilities);
        
        // Register the merged capabilities
        self.register_domain_capabilities(domain_id, capabilities);
        
        Ok(())
    }
    
    /// Create a capability for using a domain
    pub async fn create_domain_capability(
        &self,
        domain_id: &DomainId,
        resource_id: &ContentId,
        owner: &Address,
        issuer: &Address,
        capabilities: &[DomainCapability],
        delegatable: bool,
    ) -> Result<CapabilityId> {
        // Convert domain capabilities to capability constraints
        let operations: Vec<String> = capabilities
            .iter()
            .map(|cap| cap.to_string())
            .collect();
            
        let domains_constraint = CapabilityConstraint::Domains(vec![domain_id.to_string()]);
        let operations_constraint = CapabilityConstraint::Operations(operations);
        
        // Create the capability constraints
        let constraints = vec![domains_constraint, operations_constraint];
        
        // Create rights for the capability
        let mut rights = HashSet::new();
        rights.insert(Right::Execute);
        
        // Create the capability
        let capability = RigorousCapability {
            id: CapabilityId::new_random(),
            resource_id: resource_id.clone(),
            rights,
            delegated_from: None,
            issuer: issuer.clone(),
            owner: owner.clone(),
            expires_at: None, // No expiration
            revocation_id: Some(format!("domain_capability_{}", domain_id)),
            delegatable,
            constraints,
            proof: None, // No proof required for system-created capabilities
        };
        
        // Create the capability in the capability system
        self.capability_system.create_capability(capability).await
    }
    
    /// Check if an address has capability to perform a domain operation
    pub async fn check_domain_operation_capability(
        &self,
        address: &Address,
        domain_id: &DomainId,
        operation: &str,
        resource_id: &ContentId,
    ) -> Result<bool> {
        // Get all capabilities owned by the address
        let capabilities = self.capability_system.get_capabilities_for_owner(address).await?;
        
        // Filter capabilities that apply to the specific resource
        let relevant_capabilities: Vec<&RigorousCapability> = capabilities.iter()
            .filter(|cap| &cap.resource_id == resource_id)
            .collect();
            
        // Check if any capability grants access to this domain and operation
        for cap in relevant_capabilities {
            // Check if capability has the Execute right
            if !cap.rights.contains(&Right::Execute) {
                continue;
            }
            
            // Check domain constraint
            let domain_allowed = cap.constraints.iter().any(|constraint| {
                if let CapabilityConstraint::Domains(domains) = constraint {
                    domains.contains(&domain_id.to_string())
                } else {
                    false
                }
            });
            
            if !domain_allowed {
                continue;
            }
            
            // Check operation constraint
            let operation_allowed = cap.constraints.iter().any(|constraint| {
                if let CapabilityConstraint::Operations(operations) = constraint {
                    operations.contains(&operation.to_string())
                } else {
                    true // No operations constraint means all operations allowed
                }
            });
            
            if operation_allowed {
                // Validate the capability
                let status = self.capability_system.validate_capability(&cap.id).await?;
                if matches!(status, CapabilityStatus::Valid) {
                    return Ok(true);
                }
            }
        }
        
        // No valid capability found
        Ok(false)
    }
}

/// Trait extension for DomainAdapter to provide capability methods
pub trait DomainCapabilityExtension {
    /// Check if this domain adapter has a specific capability
    fn has_domain_capability(&self, capability: &DomainCapability) -> bool;
    
    /// Get all domain capabilities
    fn domain_capabilities(&self) -> HashSet<DomainCapability>;
}

impl<T: DomainAdapter> DomainCapabilityExtension for T {
    fn has_domain_capability(&self, capability: &DomainCapability) -> bool {
        let capability_str = capability.to_string();
        self.has_capability(&capability_str)
    }
    
    fn domain_capabilities(&self) -> HashSet<DomainCapability> {
        self.capabilities()
            .iter()
            .filter_map(|cap_str| DomainCapability::from_string(cap_str))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainInfo, BlockHeight, BlockHash, Timestamp, TimeMapEntry, 
                        FactQuery, FactResult, Transaction, TransactionId, TransactionReceipt};
    use std::fmt;
    
    // Mock implementation of DomainAdapter for testing
    #[derive(Debug)]
    struct MockDomainAdapter {
        domain_id: DomainId,
        domain_type: DomainType,
        capabilities: Vec<String>,
    }
    
    impl MockDomainAdapter {
        fn new(domain_id: &str, domain_type: DomainType, capabilities: Vec<String>) -> Self {
            Self {
                domain_id: DomainId::new(domain_id),
                domain_type,
                capabilities,
            }
        }
    }
    
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                domain_id: self.domain_id.clone(),
                name: format!("Test Domain {}", self.domain_id),
                domain_type: self.domain_type.clone(),
                status: crate::domain::DomainStatus::Active,
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight(100))
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            Ok(BlockHash([0; 32]))
        }
        
        async fn current_time(&self) -> Result<Timestamp> {
            Ok(Timestamp::now())
        }
        
        async fn time_map_entry(&self, _height: BlockHeight) -> Result<TimeMapEntry> {
            Ok(TimeMapEntry {
                domain_id: self.domain_id.clone(),
                height: BlockHeight(100),
                time: Timestamp::now(),
                hash: BlockHash([0; 32]),
            })
        }
        
        async fn observe_fact(&self, _query: &FactQuery) -> FactResult {
            unimplemented!()
        }
        
        async fn submit_transaction(&self, _tx: Transaction) -> Result<TransactionId> {
            Ok(TransactionId::from_str("test_tx_id"))
        }
        
        async fn transaction_receipt(&self, _tx_id: &TransactionId) -> Result<TransactionReceipt> {
            unimplemented!()
        }
        
        async fn transaction_confirmed(&self, _tx_id: &TransactionId) -> Result<bool> {
            Ok(true)
        }
        
        async fn wait_for_confirmation(
            &self,
            _tx_id: &TransactionId,
            _max_wait_ms: Option<u64>,
        ) -> Result<TransactionReceipt> {
            unimplemented!()
        }
        
        fn capabilities(&self) -> Vec<String> {
            self.capabilities.clone()
        }
    }
    
    // Mock implementation of CapabilitySystem for testing
    struct MockCapabilitySystem {
        capabilities: HashMap<CapabilityId, RigorousCapability>,
    }
    
    impl MockCapabilitySystem {
        fn new() -> Self {
            Self {
                capabilities: HashMap::new(),
            }
        }
    }
    
    #[async_trait]
    impl CapabilitySystem for MockCapabilitySystem {
        async fn create_capability(&self, capability: RigorousCapability) -> Result<CapabilityId> {
            Ok(capability.id.clone())
        }
        
        async fn get_capability(&self, id: &CapabilityId) -> Result<RigorousCapability> {
            self.capabilities.get(id)
                .cloned()
                .ok_or_else(|| Error::ResourceNotFound(format!("Capability not found: {}", id)))
        }
        
        async fn validate_capability(&self, _id: &CapabilityId) -> Result<CapabilityStatus> {
            Ok(CapabilityStatus::Valid)
        }
        
        async fn check_capability_rights(&self, _id: &CapabilityId, _rights: &[Right]) -> Result<bool> {
            Ok(true)
        }
        
        async fn delegate_capability(
            &self,
            _from_id: &CapabilityId,
            _to_address: &Address,
            _rights: &[Right],
            _constraints: Vec<CapabilityConstraint>,
            _delegatable: bool,
        ) -> Result<CapabilityId> {
            unimplemented!()
        }
        
        async fn revoke_capability(&self, _id: &CapabilityId) -> Result<()> {
            Ok(())
        }
        
        async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>> {
            Ok(self.capabilities
                .values()
                .filter(|cap| &cap.resource_id == resource_id)
                .cloned()
                .collect())
        }
        
        async fn get_capabilities_for_owner(&self, owner: &Address) -> Result<Vec<RigorousCapability>> {
            Ok(self.capabilities
                .values()
                .filter(|cap| &cap.owner == owner)
                .cloned()
                .collect())
        }
        
        async fn can_perform_operation(
            &self,
            _id: &CapabilityId,
            _operation: &str,
            _parameters: &HashMap<String, serde_json::Value>,
        ) -> Result<bool> {
            Ok(true)
        }
        
        async fn consume_capability_use(&self, _id: &CapabilityId) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_domain_capability_extension() {
        let adapter = MockDomainAdapter::new(
            "test-domain",
            DomainType::CosmWasm,
            vec![
                "send_transaction".to_string(),
                "execute_contract".to_string(),
                "query_contract".to_string(),
            ],
        );
        
        // Test capability check
        assert!(adapter.has_domain_capability(&DomainCapability::SendTransaction));
        assert!(adapter.has_domain_capability(&DomainCapability::ExecuteContract));
        assert!(adapter.has_domain_capability(&DomainCapability::QueryContract));
        assert!(!adapter.has_domain_capability(&DomainCapability::DeployContract));
        
        // Test getting all capabilities
        let caps = adapter.domain_capabilities();
        assert_eq!(caps.len(), 3);
        assert!(caps.contains(&DomainCapability::SendTransaction));
        assert!(caps.contains(&DomainCapability::ExecuteContract));
        assert!(caps.contains(&DomainCapability::QueryContract));
    }
    
    #[test]
    fn test_capabilities_for_domain_type() {
        // Test EVM capabilities
        let evm_caps = DomainCapability::capabilities_for_domain_type(&DomainType::EVM);
        assert!(evm_caps.contains(&DomainCapability::SendTransaction));
        assert!(evm_caps.contains(&DomainCapability::DeployContract));
        assert!(evm_caps.contains(&DomainCapability::ExecuteContract));
        assert!(evm_caps.contains(&DomainCapability::WriteState));
        assert!(!evm_caps.contains(&DomainCapability::Vote)); // EVM typically doesn't have this
        
        // Test CosmWasm capabilities
        let cosmwasm_caps = DomainCapability::capabilities_for_domain_type(&DomainType::CosmWasm);
        assert!(cosmwasm_caps.contains(&DomainCapability::SendTransaction));
        assert!(cosmwasm_caps.contains(&DomainCapability::ExecuteContract));
        assert!(cosmwasm_caps.contains(&DomainCapability::QueryContract));
        assert!(cosmwasm_caps.contains(&DomainCapability::Vote)); // CosmWasm has governance
        assert!(cosmwasm_caps.contains(&DomainCapability::Stake)); // CosmWasm has staking
    }
    
    #[tokio::test]
    async fn test_domain_capability_manager() {
        let mock_capability_system = Arc::new(MockCapabilitySystem::new());
        let mut manager = DomainCapabilityManager::new(mock_capability_system);
        
        // Create a test domain adapter
        let adapter = MockDomainAdapter::new(
            "test-domain",
            DomainType::CosmWasm,
            vec![
                "send_transaction".to_string(),
                "execute_contract".to_string(),
                "custom_specialized_operation".to_string(),
            ],
        );
        
        // Register the adapter
        manager.register_domain_adapter(&adapter).unwrap();
        
        // Check registered capabilities
        let caps = manager.get_domain_capabilities(&adapter.domain_id()).unwrap();
        
        // Should have default CosmWasm capabilities plus custom ones
        assert!(caps.contains(&DomainCapability::SendTransaction));
        assert!(caps.contains(&DomainCapability::ExecuteContract));
        assert!(caps.contains(&DomainCapability::QueryContract)); // From default CosmWasm
        assert!(caps.contains(&DomainCapability::Custom("specialized_operation".to_string()))); // Custom
        
        // Check capability check
        assert!(manager.domain_has_capability(&adapter.domain_id(), &DomainCapability::SendTransaction));
        assert!(manager.domain_has_capability(&adapter.domain_id(), &DomainCapability::QueryContract));
        assert!(!manager.domain_has_capability(&adapter.domain_id(), &DomainCapability::ZkProve)); // Not in CosmWasm
    }
} 
