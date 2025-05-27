//! Nullifier utility functions for the Causality framework
//!
//! This module provides utility functions for Nullifiers, implementing
//! functionality that maintains a clean separation between type definitions
//! in causality-types and implementations in causality-core.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use sha2::{Digest, Sha256};

use causality_types::{
    core::id::NullifierId,
    resource::Nullifier,
    serialization::Encode,
};

//-----------------------------------------------------------------------------
// Nullifier Utility Functions
//-----------------------------------------------------------------------------

/// Get the ID of a Nullifier
pub fn compute_nullifier_id(nullifier: &Nullifier) -> NullifierId {
    NullifierId::new(compute_nullifier_hash(nullifier))
}

/// Compute the hash of a Nullifier
pub fn compute_nullifier_hash(nullifier: &Nullifier) -> [u8; 32] {
    let bytes = nullifier.as_ssz_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}
