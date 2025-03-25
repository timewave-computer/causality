// Fact type definitions
// Original file: src/log/fact_types.rs

// Fact Types for Causality
//
// This module defines all the fact types in the Causality system
// as specified in ADR 007: Fact Management.

use serde::{Serialize, Deserialize};
use std::fmt;
use causality_types::{BlockHeight};
use crate::resource::register::{RegisterId, RegisterState};

/// FactType enum representing all types of facts in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactType {
    /// Facts about token or native currency balances
    BalanceFact,
    
    /// Facts about transactions on external domains
    TransactionFact,
    
    /// Facts provided by external oracles
    OracleFact,
    
    /// Facts about blocks on external chains
    BlockFact,
    
    /// Facts about time observations
    TimeFact,
    
    /// Facts about register state or operations
    RegisterFact(RegisterFact),
    
    /// Facts about ZK proof verification results
    ZKProofFact(ZKProofFact),
    
    /// Custom fact types
    Custom(String),
}

impl fmt::Display for FactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactType::BalanceFact => write!(f, "BalanceFact"),
            FactType::TransactionFact => write!(f, "TransactionFact"),
            FactType::OracleFact => write!(f, "OracleFact"),
            FactType::BlockFact => write!(f, "BlockFact"),
            FactType::TimeFact => write!(f, "TimeFact"),
            FactType::RegisterFact(reg_fact) => write!(f, "RegisterFact({})", reg_fact),
            FactType::ZKProofFact(zk_fact) => write!(f, "ZKProofFact({})", zk_fact),
            FactType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// RegisterFact enum representing register-related facts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegisterFact {
    /// Fact about register creation
    RegisterCreation {
        /// ID of the created register
        register_id: ContentId,
        /// Initial data of the register
        initial_data: Vec<u8>,
        /// Owner of the register
        owner: String,
        /// Domain of the register
        domain: String,
    },
    
    /// Fact about register update
    RegisterUpdate {
        /// ID of the updated register
        register_id: ContentId,
        /// New data of the register
        new_data: Vec<u8>,
        /// Previous version of the register data (hash)
        previous_version: String,
    },
    
    /// Fact about register transfer across domains
    RegisterTransfer {
        /// ID of the transferred register
        register_id: ContentId,
        /// Source domain
        source_domain: String,
        /// Target domain
        target_domain: String,
    },
    
    /// Fact about register merge
    RegisterMerge {
        /// IDs of the source registers
        source_registers: Vec<RegisterId>,
        /// ID of the resulting register
        result_register: ContentId,
    },
    
    /// Fact about register split
    RegisterSplit {
        /// ID of the source register
        source_register: ContentId,
        /// IDs of the resulting registers
        result_registers: Vec<RegisterId>,
    },

    /// Fact about register consumption (one-time use)
    RegisterConsumption {
        /// ID of the consumed register
        register_id: ContentId,
        /// Transaction ID that consumed this register
        transaction_id: String,
        /// Nullifier created by consumption
        nullifier: String,
        /// IDs of any successor registers created
        successors: Vec<RegisterId>,
        /// Block height when consumption occurred
        block_height: BlockHeight,
    },

    /// Fact about register state change
    RegisterStateChange {
        /// ID of the register
        register_id: ContentId,
        /// Previous state
        previous_state: RegisterState,
        /// New state
        new_state: RegisterState,
        /// Reason for state change
        reason: String,
    },

    /// Fact about register ownership transfer
    RegisterOwnershipTransfer {
        /// ID of the register
        register_id: ContentId,
        /// Previous owner
        previous_owner: String,
        /// New owner
        new_owner: String,
    },

    /// Fact about register locking
    RegisterLock {
        /// ID of the register
        register_id: ContentId,
        /// Reason for locking
        reason: String,
    },

    /// Fact about register unlocking
    RegisterUnlock {
        /// ID of the register
        register_id: ContentId,
        /// Reason for unlocking
        reason: String,
    },

    /// Fact about register epoch transition
    RegisterEpochTransition {
        /// ID of the register
        register_id: ContentId,
        /// Previous epoch
        previous_epoch: u64,
        /// New epoch
        new_epoch: u64,
    },

    /// Fact about register summarization
    RegisterSummarization {
        /// IDs of the summarized registers
        summarized_registers: Vec<RegisterId>,
        /// ID of the summary register
        summary_register_id: ContentId,
        /// Type of summarization performed
        summary_type: String,
        /// Epoch for this summary
        epoch: u64,
    },

    /// Fact about register archival
    RegisterArchival {
        /// ID of the register
        register_id: ContentId,
        /// Archive ID where the register is stored
        archive_id: String,
        /// Hash of the archived data for validation
        data_hash: String,
    },

    /// Fact about register authorization
    RegisterAuthorization {
        /// ID of the register
        register_id: ContentId,
        /// Type of authorization used
        authorization_type: String,
        /// ID of the authorization (e.g., signature, proof hash)
        authorization_id: String,
        /// Whether authorization succeeded
        success: bool,
    },

    /// Fact about register nullifier creation
    RegisterNullifierCreation {
        /// ID of the register
        register_id: ContentId,
        /// The nullifier value
        nullifier: String,
        /// Block height when nullifier was created
        block_height: BlockHeight,
    },
}

impl fmt::Display for RegisterFact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterFact::RegisterCreation { register_id, owner, .. } => {
                write!(f, "RegisterCreation({}, owner={})", register_id, owner)
            }
            RegisterFact::RegisterUpdate { register_id, .. } => {
                write!(f, "RegisterUpdate({})", register_id)
            }
            RegisterFact::RegisterTransfer { register_id, source_domain, target_domain } => {
                write!(f, "RegisterTransfer({}, {} -> {})", register_id, source_domain, target_domain)
            }
            RegisterFact::RegisterMerge { result_register, .. } => {
                write!(f, "RegisterMerge(-> {})", result_register)
            }
            RegisterFact::RegisterSplit { source_register, .. } => {
                write!(f, "RegisterSplit({} ->)", source_register)
            }
            RegisterFact::RegisterConsumption { register_id, .. } => {
                write!(f, "RegisterConsumption({})", register_id)
            }
            RegisterFact::RegisterStateChange { register_id, previous_state, new_state, .. } => {
                write!(f, "RegisterStateChange({}, {:?} -> {:?})", register_id, previous_state, new_state)
            }
            RegisterFact::RegisterOwnershipTransfer { register_id, previous_owner, new_owner } => {
                write!(f, "RegisterOwnershipTransfer({}, {} -> {})", register_id, previous_owner, new_owner)
            }
            RegisterFact::RegisterLock { register_id, .. } => {
                write!(f, "RegisterLock({})", register_id)
            }
            RegisterFact::RegisterUnlock { register_id, .. } => {
                write!(f, "RegisterUnlock({})", register_id)
            }
            RegisterFact::RegisterEpochTransition { register_id, previous_epoch, new_epoch } => {
                write!(f, "RegisterEpochTransition({}, {} -> {})", register_id, previous_epoch, new_epoch)
            }
            RegisterFact::RegisterSummarization { summary_register_id, epoch, .. } => {
                write!(f, "RegisterSummarization({}, epoch={})", summary_register_id, epoch)
            }
            RegisterFact::RegisterArchival { register_id, archive_id, .. } => {
                write!(f, "RegisterArchival({}, archive={})", register_id, archive_id)
            }
            RegisterFact::RegisterAuthorization { register_id, authorization_type, success, .. } => {
                write!(f, "RegisterAuthorization({}, type={}, success={})", register_id, authorization_type, success)
            }
            RegisterFact::RegisterNullifierCreation { register_id, nullifier, .. } => {
                write!(f, "RegisterNullifierCreation({}, nullifier={})", register_id, nullifier)
            }
        }
    }
}

/// ZKProofFact enum representing ZK proof-related facts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZKProofFact {
    /// Fact about a single proof verification
    ProofVerification {
        /// ID of the verification key
        verification_key_id: String,
        /// Hash of the verified proof
        proof_hash: String,
        /// Public inputs to the verification
        public_inputs: Vec<String>,
        /// Whether verification succeeded
        success: bool,
    },
    
    /// Fact about batch verification of proofs
    BatchVerification {
        /// IDs of the verification keys
        verification_key_ids: Vec<String>,
        /// Hashes of the verified proofs
        proof_hashes: Vec<String>,
        /// Combined public inputs
        public_inputs: Vec<String>,
        /// Whether batch verification succeeded
        success: bool,
    },
    
    /// Fact about circuit execution
    CircuitExecution {
        /// ID of the circuit
        circuit_id: String,
        /// Private inputs
        private_inputs_hash: String,
        /// Public inputs
        public_inputs: Vec<String>,
        /// Hash of the generated proof
        generated_proof_hash: String,
    },
    
    /// Fact about proof composition
    ProofComposition {
        /// IDs of the source proofs
        source_proof_hashes: Vec<String>,
        /// ID of the resulting proof
        result_proof_hash: String,
        /// ID of the composition circuit
        composition_circuit_id: String,
    },
}

impl fmt::Display for ZKProofFact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZKProofFact::ProofVerification { verification_key_id, success, .. } => {
                write!(f, "ProofVerification({}, success={})", verification_key_id, success)
            }
            ZKProofFact::BatchVerification { success, .. } => {
                write!(f, "BatchVerification(success={})", success)
            }
            ZKProofFact::CircuitExecution { circuit_id, .. } => {
                write!(f, "CircuitExecution({})", circuit_id)
            }
            ZKProofFact::ProofComposition { result_proof_hash, .. } => {
                write!(f, "ProofComposition(->{})", result_proof_hash)
            }
        }
    }
} 
