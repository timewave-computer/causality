// Causality Crypto
//
// This module provides cryptographic primitives for the Causality system.

// Core crypto modules - these don't depend on causality-types
pub mod hash;

// These modules need fixes as well
// pub mod signature;
// pub mod zk;

// Advanced functionality - these modules depend on causality-types
// Temporarily commented out until causality-types compiles
// pub mod content_store;
// pub mod smt_content_store; 
// pub mod deferred;
// pub mod traits;

// Basic re-exports from the hash module
pub use hash::{HashAlgorithm, HashOutput, HashError, ContentHash};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_output() {
        let bytes = [1u8; 32];
        let hash = HashOutput::new(bytes, HashAlgorithm::Blake3);
        assert_eq!(hash.algorithm(), HashAlgorithm::Blake3);
    }
} 