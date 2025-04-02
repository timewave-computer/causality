// Domain system for isolated execution environments
// Original file: src/domain/mod.rs

// Domain Module
//
// This module defines interfaces, types, and utilities for working with blockchain domains.

pub mod adapter;
pub mod capability;
pub mod selection;
pub mod error;
pub mod fact;
pub mod resource;
mod resource_impl;

// Re-export key types
pub use adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry, FactQuery};
pub use selection::{
    DomainId, DomainInfo,
    DomainSelectionStrategy, PreferredDomainStrategy, 
    LatencyBasedStrategy, CostBasedStrategy, CompositeStrategy,
    SelectionCriteria, SelectionResult
};
pub use error::{DomainError, AdapterError, Result};
pub use capability::{DomainCapability, DomainCapabilityManager, CapabilityExtension};

// Re-export from fact module
pub use fact::{FactObserver, FactVerifier, FactVerification};
pub use fact::types::FactType;

use serde::{Serialize, Deserialize};
use std::fmt;
use std::collections::HashMap;

/// Domain type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainType {
    /// EVM-based domain (Ethereum, etc.)
    EVM,
    /// CosmWasm-based domain (Cosmos, etc.)
    CosmWasm,
    /// Solana domain
    SOL,
    /// TEL domain
    TEL,
    /// Unknown domain type
    Unknown,
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainType::EVM => write!(f, "evm"),
            DomainType::CosmWasm => write!(f, "cosmwasm"),
            DomainType::SOL => write!(f, "sol"),
            DomainType::TEL => write!(f, "tel"),
            DomainType::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<String> for DomainType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "evm" => DomainType::EVM,
            "eth" => DomainType::EVM,
            "ethereum" => DomainType::EVM,
            "cosmwasm" => DomainType::CosmWasm,
            "cosmos" => DomainType::CosmWasm,
            "sol" => DomainType::SOL,
            "solana" => DomainType::SOL,
            "tel" => DomainType::TEL,
            _ => DomainType::Unknown,
        }
    }
}

impl From<&str> for DomainType {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

/// Domain status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainStatus {
    /// Domain is active and operating normally
    Active,
    /// Domain is inactive and not accepting transactions
    Inactive,
    /// Domain is in maintenance mode
    Maintenance,
    /// Domain is in an error state
    Error,
    /// Domain is being initialized
    Initializing,
    /// Domain status is unknown
    Unknown,
}

impl fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainStatus::Active => write!(f, "active"),
            DomainStatus::Inactive => write!(f, "inactive"),
            DomainStatus::Maintenance => write!(f, "maintenance"),
            DomainStatus::Error => write!(f, "error"),
            DomainStatus::Initializing => write!(f, "initializing"),
            DomainStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Block height in a domain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHeight(pub u64);

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Block hash in a domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHash(pub [u8; 32]);

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a timestamp from the current system time
    pub fn now() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(now)
    }

    /// Get the timestamp as a u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction ID in a domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub String);

impl TransactionId {
    /// Create a transaction ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the transaction ID as a string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub transaction_id: TransactionId,
    /// Block height where the transaction was included
    pub block_height: BlockHeight,
    /// Block hash where the transaction was included
    pub block_hash: BlockHash,
    /// Transaction execution status
    pub status: TransactionStatus,
    /// Gas used by the transaction (if applicable)
    pub gas_used: Option<u64>,
    /// Fee paid for the transaction
    pub fee_paid: Option<u64>,
    /// Logs produced by the transaction
    pub logs: Vec<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction was successful
    Success,
    /// Transaction failed
    Failure(String),
    /// Transaction is pending
    Pending,
    /// Transaction timed out
    Timeout,
    /// Transaction was rejected
    Rejected(String),
    /// Transaction status is unknown
    Unknown,
}

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction data
    pub data: Vec<u8>,
    /// Transaction type
    pub transaction_type: String,
    /// Domain ID for this transaction
    pub domain_id: DomainId,
    /// Transaction metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Metadata about a fact observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactObservationMeta {
    /// Time when the fact was observed
    pub observed_at: Timestamp,
    /// Block height at which the fact was observed
    pub block_height: Option<BlockHeight>,
    /// How reliable the fact is (0.0 to 1.0)
    pub reliability: f64,
    /// Source of the fact
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
} 