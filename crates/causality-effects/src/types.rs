// Effect type definitions
// Original file: src/effect/types.rs

// Effect types for Causality Effect System
//
// This module defines the different types of effects that can be
// performed in the Causality system.

use std::fmt;
use serde::{Serialize, Deserialize};
use causality_crypto::ContentId;

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

/// Type of resource change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceChangeType {
    /// Resource was created
    Created,
    /// Resource was updated
    Updated,
    /// Resource was deleted
    Deleted,
    /// Resource was marked as consumed
    Consumed,
    /// Resource ownership was transferred
    Transferred,
    /// Resource was merged with another
    Merged,
    /// Resource was split into multiple resources
    Split,
    /// Resource was locked for exclusive access
    Locked,
    /// Resource was unlocked after exclusive access
    Unlocked,
    /// Resource was published to a public registry
    Published,
    /// Resource was unpublished from a public registry
    Unpublished,
    /// Resource was archived (still stored but inactive)
    Archived,
    /// Resource was restored from archive
    Restored,
    /// Resource was versioned (new version created)
    Versioned,
    /// Resource was attached to another resource
    Attached,
    /// Resource was detached from another resource
    Detached,
    /// Resource was tagged with metadata
    Tagged,
    /// Resource was committed to persistent storage
    Committed,
    /// Resource state was rolled back
    RolledBack,
    /// Resource content was hashed for verification
    ContentHashed,
    /// Resource was computed from other resources
    Computed,
}

impl fmt::Display for ResourceChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceChangeType::Created => write!(f, "Created"),
            ResourceChangeType::Updated => write!(f, "Updated"),
            ResourceChangeType::Deleted => write!(f, "Deleted"),
            ResourceChangeType::Consumed => write!(f, "Consumed"),
            ResourceChangeType::Transferred => write!(f, "Transferred"),
            ResourceChangeType::Merged => write!(f, "Merged"),
            ResourceChangeType::Split => write!(f, "Split"),
            ResourceChangeType::Locked => write!(f, "Locked"),
            ResourceChangeType::Unlocked => write!(f, "Unlocked"),
            ResourceChangeType::Published => write!(f, "Published"),
            ResourceChangeType::Unpublished => write!(f, "Unpublished"),
            ResourceChangeType::Archived => write!(f, "Archived"),
            ResourceChangeType::Restored => write!(f, "Restored"),
            ResourceChangeType::Versioned => write!(f, "Versioned"),
            ResourceChangeType::Attached => write!(f, "Attached"),
            ResourceChangeType::Detached => write!(f, "Detached"),
            ResourceChangeType::Tagged => write!(f, "Tagged"),
            ResourceChangeType::Committed => write!(f, "Committed"),
            ResourceChangeType::RolledBack => write!(f, "RolledBack"),
            ResourceChangeType::ContentHashed => write!(f, "ContentHashed"),
            ResourceChangeType::Computed => write!(f, "Computed"),
        }
    }
}

/// Represents a change to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    /// ID of the resource that was changed
    pub resource_id: ContentId,
    /// Type of change that occurred
    pub change_type: ResourceChangeType,
    /// Hash of the previous state (if any)
    pub previous_state_hash: Option<String>,
    /// Hash of the new state
    pub new_state_hash: String,
} 
