// Time-based snapshot data structures
// Original file: src/time/time_map_snapshot.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use std::fmt;

use crate::Result;
use crate::crypto_primitives::{ContentAddressed, HashOutput, HashError};

/// A snapshot of a time map at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeMapSnapshot {
    /// Timestamp of this snapshot
    pub timestamp: u64,
    
    /// Domain timestamps at this point in time
    pub domain_timestamps: HashMap<String, u64>,
    
    /// Causality edges known at this point in time
    pub causality_edges: Vec<(String, String)>,
    
    /// Hash of the time map at this point
    pub hash: String,
}

impl TimeMapSnapshot {
    /// Create a new time map snapshot
    pub fn new(
        timestamp: u64,
        domain_timestamps: HashMap<String, u64>,
        causality_edges: Vec<(String, String)>,
    ) -> Self {
        // In a real implementation, we'd compute a proper hash here
        let hash = format!("snapshot_{}", timestamp);
        
        Self {
            timestamp,
            domain_timestamps,
            causality_edges,
            hash,
        }
    }
    
    /// Create an empty snapshot with just a timestamp
    pub fn with_timestamp(timestamp: u64) -> Self {
        Self {
            timestamp,
            domain_timestamps: HashMap::new(),
            causality_edges: Vec::new(),
            hash: format!("snapshot_{}", timestamp),
        }
    }
    
    /// Get the timestamp of this snapshot
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Get the hash of this snapshot
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// Effect that changes time-related state
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum TimeEffect {
    /// Update to causal ordering
    CausalUpdate {
        /// Operations being causally ordered
        operations: Vec<String>,
        /// Causal ordering constraints
        ordering: Vec<(String, String)>, // (before, after)
    },
    
    /// Clock time attestation
    ClockAttestation {
        /// Domain providing the clock time
        domain_id: String,
        /// Actual timestamp value
        timestamp: u64,
        /// Source of the attestation
        source: AttestationSource,
        /// Confidence level (0.0-1.0)
        confidence: f64,
    },
    
    /// Time map update
    TimeMapUpdate {
        /// New domain positions
        positions: HashMap<String, u64>,
        /// Proof of domain positions
        proofs: HashMap<String, String>,
    },
}

/// The source of a time attestation
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AttestationSource {
    /// Network Time Protocol
    NTP,
    /// External time source
    External(String),
    /// Consensus-based time
    Consensus(String),
    /// User-provided time
    User,
    /// Custom time source
    Custom(String),
    /// System clock source
    SystemClock,
    /// Blockchain source
    Blockchain {
        /// Chain ID or name
        chain_id: String,
        /// Block number
        block_number: Option<u64>,
    },
    /// Operator source
    Operator {
        /// Operator identifier
        operator_id: String,
        /// Signature
        signature: String,
    },
    /// Committee source
    Committee {
        /// Committee identifier
        committee_id: String,
        /// Signatures
        signatures: Vec<String>,
    },
    /// Oracle source
    Oracle {
        /// Oracle identifier
        oracle_id: String,
        /// Oracle data
        data: String,
    },
}

impl borsh::BorshSerialize for AttestationSource {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            AttestationSource::NTP => {
                borsh::BorshSerialize::serialize(&0u8, writer)?;
            }
            AttestationSource::External(s) => {
                borsh::BorshSerialize::serialize(&1u8, writer)?;
                borsh::BorshSerialize::serialize(s, writer)?;
            }
            AttestationSource::Consensus(s) => {
                borsh::BorshSerialize::serialize(&2u8, writer)?;
                borsh::BorshSerialize::serialize(s, writer)?;
            }
            AttestationSource::User => {
                borsh::BorshSerialize::serialize(&3u8, writer)?;
            }
            AttestationSource::Custom(s) => {
                borsh::BorshSerialize::serialize(&4u8, writer)?;
                borsh::BorshSerialize::serialize(s, writer)?;
            }
            AttestationSource::SystemClock => {
                borsh::BorshSerialize::serialize(&5u8, writer)?;
            }
            AttestationSource::Blockchain { chain_id, block_number } => {
                borsh::BorshSerialize::serialize(&6u8, writer)?;
                borsh::BorshSerialize::serialize(chain_id, writer)?;
                borsh::BorshSerialize::serialize(block_number, writer)?;
            }
            AttestationSource::Operator { operator_id, signature } => {
                borsh::BorshSerialize::serialize(&7u8, writer)?;
                borsh::BorshSerialize::serialize(operator_id, writer)?;
                borsh::BorshSerialize::serialize(signature, writer)?;
            }
            AttestationSource::Committee { committee_id, signatures } => {
                borsh::BorshSerialize::serialize(&8u8, writer)?;
                borsh::BorshSerialize::serialize(committee_id, writer)?;
                borsh::BorshSerialize::serialize(signatures, writer)?;
            }
            AttestationSource::Oracle { oracle_id, data } => {
                borsh::BorshSerialize::serialize(&9u8, writer)?;
                borsh::BorshSerialize::serialize(oracle_id, writer)?;
                borsh::BorshSerialize::serialize(data, writer)?;
            }
        }
        Ok(())
    }
}

impl borsh::BorshDeserialize for AttestationSource {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let variant_idx = u8::deserialize_reader(reader)?;
        match variant_idx {
            0 => Ok(AttestationSource::NTP),
            1 => {
                let s = String::deserialize_reader(reader)?;
                Ok(AttestationSource::External(s))
            }
            2 => {
                let s = String::deserialize_reader(reader)?;
                Ok(AttestationSource::Consensus(s))
            }
            3 => Ok(AttestationSource::User),
            4 => {
                let s = String::deserialize_reader(reader)?;
                Ok(AttestationSource::Custom(s))
            }
            5 => Ok(AttestationSource::SystemClock),
            6 => {
                let chain_id = String::deserialize_reader(reader)?;
                let block_number = Option::<u64>::deserialize_reader(reader)?;
                Ok(AttestationSource::Blockchain { chain_id, block_number })
            }
            7 => {
                let operator_id = String::deserialize_reader(reader)?;
                let signature = String::deserialize_reader(reader)?;
                Ok(AttestationSource::Operator { operator_id, signature })
            }
            8 => {
                let committee_id = String::deserialize_reader(reader)?;
                let signatures = Vec::<String>::deserialize_reader(reader)?;
                Ok(AttestationSource::Committee { committee_id, signatures })
            }
            9 => {
                let oracle_id = String::deserialize_reader(reader)?;
                let data = String::deserialize_reader(reader)?;
                Ok(AttestationSource::Oracle { oracle_id, data })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown AttestationSource variant: {}", variant_idx),
            )),
        }
    }
}

impl fmt::Display for AttestationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttestationSource::NTP => write!(f, "NTP"),
            AttestationSource::External(src) => write!(f, "External({})", src),
            AttestationSource::Consensus(src) => write!(f, "Consensus({})", src),
            AttestationSource::User => write!(f, "User"),
            AttestationSource::Custom(name) => write!(f, "Custom({})", name),
            AttestationSource::SystemClock => write!(f, "SystemClock"),
            AttestationSource::Blockchain { chain_id, block_number } => {
                write!(f, "Blockchain({}: {})", chain_id, 
                    block_number.map_or_else(|| "unknown".to_string(), |b| b.to_string()))
            },
            AttestationSource::Operator { operator_id, signature } => {
                write!(f, "Operator({}): {}", operator_id, signature)
            },
            AttestationSource::Committee { committee_id, signatures } => {
                write!(f, "Committee({}): {} signatures", committee_id, signatures.len())
            },
            AttestationSource::Oracle { oracle_id, data } => {
                write!(f, "Oracle({}): {}", oracle_id, data)
            },
        }
    }
}

/// Domain position in time
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainPosition {
    /// Domain identifier
    pub domain_id: String,
    /// Timestamp value
    pub timestamp: u64,
    /// Optional block number
    pub block: Option<u64>,
    /// Optional transaction index
    pub tx_index: Option<u64>,
}

impl DomainPosition {
    /// Create a new domain position
    pub fn new(domain_id: &str, timestamp: u64) -> Self {
        Self {
            domain_id: domain_id.to_string(),
            timestamp,
            block: None,
            tx_index: None,
        }
    }
    
    /// Set the block number
    pub fn with_block(mut self, block: u64) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Set the transaction index
    pub fn with_tx_index(mut self, tx_index: u64) -> Self {
        self.tx_index = Some(tx_index);
        self
    }
}

impl fmt::Display for DomainPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.domain_id, self.timestamp)?;
        if let Some(block) = self.block {
            write!(f, " (block: {})", block)?;
        }
        if let Some(tx_index) = self.tx_index {
            write!(f, " (tx: {})", tx_index)?;
        }
        Ok(())
    }
}

/// Result of a time effect operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeEffectResult {
    /// Result of a causal update
    CausalUpdate {
        /// Content hash of the updated causal graph
        graph_hash: String,
        /// Operations that were affected
        affected_operations: Vec<String>,
    },
    
    /// Result of a clock attestation
    ClockUpdate {
        /// Domain ID
        domain_id: String,
        /// Updated timestamp
        timestamp: u64,
        /// Confidence level
        confidence: f64,
    },
    
    /// Result of a time map update
    TimeMapUpdate {
        /// Content hash of the updated time map
        map_hash: String,
        /// Domains that were updated
        domains_updated: Vec<String>,
    },
}

// Implement ContentAddressed for TimeEffect
impl ContentAddressed for TimeEffect {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        crate::content_addressing::canonical_content_hash(self)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        // Use canonical binary serialization
        crate::content_addressing::canonical::to_canonical_binary(self)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        // Use canonical binary deserialization
        crate::content_addressing::canonical::from_canonical_binary(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
} 