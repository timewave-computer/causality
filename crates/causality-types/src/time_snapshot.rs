// Time-based snapshot data structures
// Original file: src/time/time_map_snapshot.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

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

/// Source of a time attestation
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum AttestationSource {
    /// Blockchain timestamp (trust depends on consensus model)
    Blockchain {
        /// Block height
        height: u64,
        /// Block hash
        block_hash: String,
    },
    
    /// User attestation (low trust)
    User {
        /// User ID
        user_id: String,
        /// Signature
        signature: String,
    },
    
    /// Operator attestation (medium trust)
    Operator {
        /// Operator ID
        operator_id: String,
        /// Signature
        signature: String,
    },
    
    /// Committee attestation (higher trust)
    Committee {
        /// Committee ID
        committee_id: String,
        /// Threshold signature
        threshold_signature: String,
    },
    
    /// External oracle (trust depends on oracle reputation)
    Oracle {
        /// Oracle ID
        oracle_id: String,
        /// Signature
        signature: String,
    },
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