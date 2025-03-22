// Effect types for Causality Effect System
//
// This module defines the different types of effects that can be
// performed in the Causality system.

use std::fmt;
use serde::{Serialize, Deserialize};

/// The type of effect being performed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    /// Create a new resource
    Create,
    /// Read an existing resource
    Read,
    /// Update an existing resource
    Update,
    /// Delete an existing resource
    Delete,
    /// Deposit tokens into an account
    Deposit,
    /// Withdraw tokens from an account
    Withdraw,
    /// Observe an account's balance
    Observe,
    /// Transfer tokens between accounts
    Transfer,
    /// Execute code internal to the system
    Execute,
    /// Call an external service or API
    Call,
    /// Wait for a specific time
    Wait,
    /// Acquiring a lock on a resource
    LockAcquisition,
    /// Releasing a lock on a resource
    LockRelease,
    /// Computational effect (pure computation)
    Computation,
    /// Compile a ZK program from source code
    CompileZkProgram,
    /// Generate a witness for a ZK program
    GenerateZkWitness,
    /// Generate a ZK proof from a witness
    GenerateZkProof,
    /// Verify a ZK proof
    VerifyZkProof,
    /// A custom effect type
    Custom(String),
}

impl fmt::Display for EffectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectType::Create => write!(f, "Create"),
            EffectType::Read => write!(f, "Read"),
            EffectType::Update => write!(f, "Update"),
            EffectType::Delete => write!(f, "Delete"),
            EffectType::Deposit => write!(f, "Deposit"),
            EffectType::Withdraw => write!(f, "Withdraw"),
            EffectType::Observe => write!(f, "Observe"),
            EffectType::Transfer => write!(f, "Transfer"),
            EffectType::Execute => write!(f, "Execute"),
            EffectType::Call => write!(f, "Call"),
            EffectType::Wait => write!(f, "Wait"),
            EffectType::LockAcquisition => write!(f, "LockAcquisition"),
            EffectType::LockRelease => write!(f, "LockRelease"),
            EffectType::Computation => write!(f, "Computation"),
            EffectType::CompileZkProgram => write!(f, "CompileZkProgram"),
            EffectType::GenerateZkWitness => write!(f, "GenerateZkWitness"),
            EffectType::GenerateZkProof => write!(f, "GenerateZkProof"),
            EffectType::VerifyZkProof => write!(f, "VerifyZkProof"),
            EffectType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
} 