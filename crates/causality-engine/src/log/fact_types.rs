// Fact type definitions for the Causality Engine
// Original file: src/log/fact_types.rs

// This module defines the data structures for facts in the Causality log

use std::fmt;
use serde::{Serialize, Deserialize};
use causality_types::{DomainId, Timestamp, ContentId};
use causality_error::Result;

/// Type of fact in a log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactType {
    /// A fact about a blockchain event
    BlockchainEvent,
    /// A fact from a domain adapter
    DomainAdapterFact,
    /// A fact about a register operation (resource)
    RegisterOperation,
    /// A fact about the state of a register (resource)
    RegisterState,
    /// A fact derived from a cross-domain operation
    CrossDomainOperation,
    /// A fact about a proof
    Proof,
    /// A fact about verification of an item
    Verification,
    /// A custom fact type
    Custom(String),
}

impl fmt::Display for FactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactType::BlockchainEvent => write!(f, "blockchain_event"),
            FactType::DomainAdapterFact => write!(f, "domain_adapter_fact"),
            FactType::RegisterOperation => write!(f, "register_operation"),
            FactType::RegisterState => write!(f, "register_state"),
            FactType::CrossDomainOperation => write!(f, "cross_domain_operation"),
            FactType::Proof => write!(f, "proof"),
            FactType::Verification => write!(f, "verification"),
            FactType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Operation type for a register
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegisterOperationType {
    /// Create operation
    Create,
    /// Read operation
    Read,
    /// Update operation
    Update,
    /// Delete operation
    Delete,
}

/// A fact about a register operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterOperationFact {
    /// Register ID
    pub register_id: ContentId,
    /// Operation type
    pub operation_type: RegisterOperationType,
    /// Operation data
    pub data: Option<Vec<u8>>,
}

/// A fact about the state of a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterStateFact {
    /// Register ID
    pub register_id: ContentId,
    /// State data
    pub state: Vec<u8>,
    /// Version of the state
    pub version: u64,
}

/// A fact about a blockchain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEventFact {
    /// Register ID
    pub register_id: ContentId,
    /// Block number
    pub block_number: u64,
    /// Transaction hash
    pub transaction_hash: String,
    /// Event data
    pub event_data: Vec<u8>,
}

/// A fact about cross-domain operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainOperationFact {
    /// Source domain
    pub source_domain: DomainId,
    /// Target domain
    pub target_domain: DomainId,
    /// Result register
    pub result_register: ContentId,
    /// Operation data
    pub operation_data: Vec<u8>,
}

/// A fact with a proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofFact {
    /// Source register
    pub source_register: ContentId,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Proof type
    pub proof_type: String,
}

/// A fact about verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationFact {
    /// Register ID
    pub register_id: ContentId,
    /// Verification result
    pub verified: bool,
    /// Verification metadata
    pub metadata: serde_json::Value,
}

/// A fact from a domain adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAdapterFact {
    /// Register ID
    pub register_id: ContentId,
    /// Adapter name
    pub adapter_name: String,
    /// Fact data
    pub fact_data: Vec<u8>,
}

/// A register state change fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterFact {
    /// Register ID
    pub register_id: ContentId,
    /// State data
    pub state: Vec<u8>,
    /// State version
    pub version: u64,
    /// Previous state hash (if any)
    pub previous_hash: Option<String>,
}

/// A ZK proof fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProofFact {
    /// Register ID
    pub register_id: ContentId,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Circuit ID
    pub circuit_id: String,
    /// Public inputs
    pub public_inputs: Vec<String>,
}

/// A time map fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMapFact {
    /// Domain ID
    pub domain_id: DomainId,
    /// Origin timestamp
    pub origin_timestamp: Timestamp,
    /// Target timestamp
    pub target_timestamp: Timestamp,
    /// Summary register ID
    pub summary_register_id: ContentId,
}

/// A cross-chain relay fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayFact {
    /// Register ID
    pub register_id: ContentId,
    /// Source domain
    pub source_domain: DomainId,
    /// Target domain
    pub target_domain: DomainId,
    /// Relay data
    pub relay_data: Vec<u8>,
}

/// A consensus fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusFact {
    /// Register ID
    pub register_id: ContentId,
    /// Consensus type
    pub consensus_type: String,
    /// Participants
    pub participants: Vec<String>,
    /// Consensus data
    pub consensus_data: Vec<u8>,
}

/// A protocol fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFact {
    /// Register ID
    pub register_id: ContentId,
    /// Protocol name
    pub protocol_name: String,
    /// Protocol version
    pub protocol_version: String,
    /// Protocol data
    pub protocol_data: Vec<u8>,
}

/// A resource capability fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilityFact {
    /// Register ID
    pub register_id: ContentId,
    /// Capability type
    pub capability_type: String,
    /// Grantee
    pub grantee: String,
    /// Capability data
    pub capability_data: Vec<u8>,
} 
