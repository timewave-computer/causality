// Domain System for Causality
//
// This module provides interfaces and implementations for interacting with 
// external chains and managing observed state.

use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{BlockHash, BlockHeight, Timestamp};
use crate::log::fact_types::FactType;

// Primary modules in the domain system
pub mod map;
pub mod fact;

// Re-exports from domain modules
pub use map::{DomainMap, DomainMapEntry, MapQueryResult};

// Re-exports from fact module
pub use fact::{
    VerificationResult,
    MerkleProofVerifier, SignatureVerifier, ConsensusVerifier,
    VerifierRegistry
};

// Time module re-exports (unified model)
pub use crate::time::{
    TimeMap, TimeMapEntry, TimePoint, TimeRange, TimeWindow,
    TimeSyncManager, TimeSyncConfig, SyncResult, SyncStatus
};

/// Transaction data to be submitted to a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Domain ID
    pub domain_id: DomainId,
    /// Transaction type
    pub tx_type: String,
    /// Transaction data
    pub data: Vec<u8>,
    /// Transaction metadata
    pub metadata: HashMap<String, String>,
}

/// A transaction ID used to identify and track transactions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub String);

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub tx_id: TransactionId,
    /// Domain ID
    pub domain_id: DomainId,
    /// Block height
    pub block_height: Option<BlockHeight>,
    /// Block hash
    pub block_hash: Option<BlockHash>,
    /// Timestamp
    pub timestamp: Option<Timestamp>,
    /// Status (success, pending, failed)
    pub status: TransactionStatus,
    /// Error message if failed
    pub error: Option<String>,
    /// Gas used
    pub gas_used: Option<u64>,
    /// Additional receipt metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction has been included in a block and succeeded
    Success,
    /// Transaction is pending inclusion
    Pending,
    /// Transaction has been included but failed
    Failed,
    /// Transaction has been dropped from the mempool
    Dropped,
    /// Transaction status is unknown
    Unknown,
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Success => write!(f, "success"),
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Dropped => write!(f, "dropped"),
            TransactionStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DomainId(String);

impl DomainId {
    /// Create a new domain ID
    pub fn new(id: &str) -> Self {
        DomainId(id.to_string())
    }
    
    /// Get the domain ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Check if this is a valid domain ID
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.len() <= 64
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DomainId {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self> {
        let id = DomainId::new(s);
        
        if !id.is_valid() {
            return Err(Error::InvalidArgument("Invalid domain ID".to_string()));
        }
        
        Ok(id)
    }
}

impl From<String> for DomainId {
    fn from(s: String) -> Self {
        DomainId(s)
    }
}

impl From<&str> for DomainId {
    fn from(s: &str) -> Self {
        DomainId(s.to_string())
    }
}

/// Domain capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCapabilities {
    /// Whether this domain supports fact verification
    pub supports_verification: bool,
    /// Whether this domain provides guarantees
    pub provides_guarantees: bool,
    /// Whether this domain can be queried directly
    pub queryable: bool,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: Option<u32>,
    /// Supported query types
    pub supported_queries: Vec<String>,
}

impl Default for DomainCapabilities {
    fn default() -> Self {
        DomainCapabilities {
            supports_verification: false,
            provides_guarantees: false,
            queryable: false,
            max_concurrent_requests: None,
            supported_queries: Vec::new(),
        }
    }
}

/// Domain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    /// Domain identifier
    pub id: DomainId,
    /// Domain name
    pub name: String,
    /// Domain description
    pub description: Option<String>,
    /// Domain version
    pub version: String,
    /// Domain capabilities
    pub capabilities: DomainCapabilities,
}

impl DomainInfo {
    /// Create a new domain info
    pub fn new(id: DomainId, name: &str, version: &str) -> Self {
        DomainInfo {
            id,
            name: name.to_string(),
            description: None,
            version: version.to_string(),
            capabilities: DomainCapabilities::default(),
        }
    }
    
    /// Set the domain description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set the domain capabilities
    pub fn with_capabilities(mut self, capabilities: DomainCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Domain status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainStatus {
    /// Domain is online and operational
    Online,
    /// Domain is offline
    Offline,
    /// Domain is in maintenance mode
    Maintenance,
    /// Domain has an error
    Error(String),
}

impl fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainStatus::Online => write!(f, "online"),
            DomainStatus::Offline => write!(f, "offline"),
            DomainStatus::Maintenance => write!(f, "maintenance"),
            DomainStatus::Error(err) => write!(f, "error: {}", err),
        }
    }
}

/// Domain type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainType {
    /// Ethereum Virtual Machine
    EVM,
    /// Solana
    Solana,
    /// Cosmos
    Cosmos,
    /// CosmWasm with ZK proofs
    CosmWasmZK,
    /// Layer-2 Solution
    Layer2,
    /// Optimistic rollup
    OptimisticRollup,
    /// Zero-knowledge rollup
    ZkRollup,
    /// Cross-chain
    CrossChain,
    /// Custom domain
    Custom(u32),
}

impl DomainType {
    /// Get the domain type as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainType::EVM => "evm",
            DomainType::Solana => "solana",
            DomainType::Cosmos => "cosmos",
            DomainType::CosmWasmZK => "cosmwasm_zk",
            DomainType::Layer2 => "layer2",
            DomainType::OptimisticRollup => "optimistic_rollup",
            DomainType::ZkRollup => "zk_rollup",
            DomainType::CrossChain => "cross_chain",
            DomainType::Custom(_) => "custom",
        }
    }
    
    /// Create a DomainType from a domain ID
    pub fn from_domain_id(domain_id: &DomainId) -> Self {
        // Extract domain type from domain ID format
        // Expected format: <type>:<chain_id>:<network>
        let parts: Vec<&str> = domain_id.as_str().split(':').collect();
        
        if parts.len() >= 1 {
            match parts[0].to_lowercase().as_str() {
                "evm" => DomainType::EVM,
                "solana" => DomainType::Solana,
                "cosmos" => DomainType::Cosmos,
                "cosmwasm_zk" => DomainType::CosmWasmZK,
                "layer2" => DomainType::Layer2,
                "optimistic" => DomainType::OptimisticRollup,
                "zk_rollup" => DomainType::ZkRollup,
                "cross" => DomainType::CrossChain,
                _ => {
                    // Try to parse a custom type ID
                    if let Ok(type_id) = parts[0].parse::<u32>() {
                        DomainType::Custom(type_id)
                    } else {
                        DomainType::Custom(0)
                    }
                }
            }
        } else {
            DomainType::Custom(0)
        }
    }
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainType::EVM => write!(f, "EVM"),
            DomainType::Solana => write!(f, "Solana"),
            DomainType::Cosmos => write!(f, "Cosmos"),
            DomainType::CosmWasmZK => write!(f, "CosmWasmZK"),
            DomainType::Layer2 => write!(f, "Layer2"),
            DomainType::OptimisticRollup => write!(f, "OptimisticRollup"),
            DomainType::ZkRollup => write!(f, "ZkRollup"),
            DomainType::CrossChain => write!(f, "CrossChain"),
            DomainType::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_id() {
        let id = DomainId::new("evm:1:mainnet");
        assert_eq!(id.as_str(), "evm:1:mainnet");
        assert!(id.is_valid());
        
        let id_str = "cosmos:cosmoshub-4:mainnet";
        let id: DomainId = id_str.parse().unwrap();
        assert_eq!(id.as_str(), "cosmos:cosmoshub-4:mainnet");
        
        let empty_id = DomainId::new("");
        assert!(!empty_id.is_valid());
    }
    
    #[test]
    fn test_domain_capabilities() {
        let capabilities = DomainCapabilities {
            supports_verification: true,
            provides_guarantees: true,
            queryable: true,
            max_concurrent_requests: Some(10),
            supported_queries: vec!["balance".to_string(), "transaction".to_string()],
        };
        
        assert!(capabilities.supports_verification);
        assert!(capabilities.provides_guarantees);
        assert!(capabilities.queryable);
        assert_eq!(capabilities.max_concurrent_requests, Some(10));
        assert_eq!(capabilities.supported_queries.len(), 2);
    }
    
    #[test]
    fn test_domain_info() {
        let info = DomainInfo::new(
            DomainId::new("evm:1:mainnet"),
            "Ethereum Mainnet",
            "1.0.0"
        )
        .with_description("Ethereum Mainnet Network")
        .with_capabilities(DomainCapabilities {
            supports_verification: true,
            provides_guarantees: true,
            queryable: true,
            max_concurrent_requests: Some(10),
            supported_queries: vec!["balance".to_string()],
        });
        
        assert_eq!(info.id.as_str(), "evm:1:mainnet");
        assert_eq!(info.name, "Ethereum Mainnet");
        assert_eq!(info.description, Some("Ethereum Mainnet Network".to_string()));
        assert!(info.capabilities.supports_verification);
    }
    
    #[test]
    fn test_domain_status() {
        let status = DomainStatus::Online;
        assert_eq!(status.to_string(), "online");
        
        let error_status = DomainStatus::Error("Connection timeout".to_string());
        assert_eq!(error_status.to_string(), "error: Connection timeout");
    }
    
    #[test]
    fn test_domain_type() {
        let domain_id = DomainId::new("evm:1:mainnet");
        let domain_type = DomainType::from_domain_id(&domain_id);
        assert_eq!(domain_type, DomainType::EVM);
        
        let cosmos_id = DomainId::new("cosmos:cosmoshub-4:mainnet");
        let cosmos_type = DomainType::from_domain_id(&cosmos_id);
        assert_eq!(cosmos_type, DomainType::Cosmos);
        
        assert_eq!(DomainType::EVM.as_str(), "evm");
        assert_eq!(DomainType::Custom(42).as_str(), "custom");
    }
} 