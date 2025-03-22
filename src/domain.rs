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

// Sub-modules (Legacy - will be removed)
mod selection;
mod registry;

// New refactored modules
pub mod map;
pub mod fact;

// Re-exports from legacy modules
pub use registry::DomainRegistry;
pub use selection::{DomainSelector, SelectionCriteria};

// Re-exports from time module
pub use time::map::{TimeMap, TimeMapEntry};
pub use time::types::{TimePoint, TimeRange, TimeWindow};
pub use time::sync::{TimeSyncManager, TimeSyncConfig, SyncResult, SyncStatus};

// Re-exports from fact module
pub use fact::{
    VerificationResult,
    MerkleProofVerifier, SignatureVerifier, ConsensusVerifier,
    VerifierRegistry
};

// Re-export DomainId from types (legacy)
pub use crate::types::DomainId as LegacyDomainId;

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
            DomainStatus::Error(e) => write!(f, "error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_id() {
        let id = DomainId::new("test-domain");
        assert_eq!(id.as_str(), "test-domain");
        assert!(id.is_valid());
        
        let empty_id = DomainId::new("");
        assert!(!empty_id.is_valid());
        
        let from_string: DomainId = "another-domain".into();
        assert_eq!(from_string.as_str(), "another-domain");
        
        let parsed = "parsed-domain".parse::<DomainId>();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().as_str(), "parsed-domain");
        
        let invalid_parse = "".parse::<DomainId>();
        assert!(invalid_parse.is_err());
    }
    
    #[test]
    fn test_domain_capabilities() {
        let default_caps = DomainCapabilities::default();
        assert!(!default_caps.supports_verification);
        assert!(!default_caps.provides_guarantees);
        assert!(!default_caps.queryable);
        assert!(default_caps.max_concurrent_requests.is_none());
        assert!(default_caps.supported_queries.is_empty());
        
        let custom_caps = DomainCapabilities {
            supports_verification: true,
            provides_guarantees: true,
            queryable: true,
            max_concurrent_requests: Some(10),
            supported_queries: vec!["query1".to_string(), "query2".to_string()],
        };
        
        assert!(custom_caps.supports_verification);
        assert!(custom_caps.provides_guarantees);
        assert!(custom_caps.queryable);
        assert_eq!(custom_caps.max_concurrent_requests, Some(10));
        assert_eq!(custom_caps.supported_queries.len(), 2);
    }
    
    #[test]
    fn test_domain_info() {
        let id = DomainId::new("test-domain");
        let info = DomainInfo::new(id.clone(), "Test Domain", "1.0.0")
            .with_description("A test domain")
            .with_capabilities(DomainCapabilities {
                supports_verification: true,
                queryable: true,
                ..Default::default()
            });
        
        assert_eq!(info.id, id);
        assert_eq!(info.name, "Test Domain");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, Some("A test domain".to_string()));
        assert!(info.capabilities.supports_verification);
        assert!(info.capabilities.queryable);
    }
    
    #[test]
    fn test_domain_status() {
        let online = DomainStatus::Online;
        let error = DomainStatus::Error("Connection failed".to_string());
        
        assert_eq!(online.to_string(), "online");
        assert_eq!(error.to_string(), "error: Connection failed");
    }
} 