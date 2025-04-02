// Verification and validation framework
//
// This module provides abstractions for verification and validation of various
// aspects of the Causality system, including signatures, proofs, and constraints.

// Core submodules
pub mod signatures;
pub mod constraints;
pub mod proofs;

// Re-export key types
pub use signatures::{Signature, Signer, Verifier};
pub use constraints::{Constraint, ConstraintSet, ConstraintVerifier};
pub use proofs::{Proof, Prover, ProofVerifier};

use std::marker::PhantomData;

/// A trait for entities that can be verified
pub trait Verifiable {
    /// The error type returned when verification fails
    type Error;
    
    /// Verify that this entity is valid
    fn verify(&self) -> Result<(), Self::Error>;
    
    /// Check if this entity is valid without returning details
    fn is_valid(&self) -> bool {
        self.verify().is_ok()
    }
}

/// A trait for verifying entities
pub trait Verify<T> {
    /// The error type returned when verification fails
    type Error;
    
    /// Verify the given entity
    fn verify(&self, entity: &T) -> Result<(), Self::Error>;
    
    /// Check if the given entity is valid without returning details
    fn is_valid(&self, entity: &T) -> bool {
        self.verify(entity).is_ok()
    }
}

/// A verification context that provides necessary information for verification
pub trait VerificationContext {
    /// The type of entities this context can verify
    type Entity;
    
    /// The error type returned when verification fails
    type Error;
    
    /// Verify the given entity in this context
    fn verify(&self, entity: &Self::Entity) -> Result<(), Self::Error>;
    
    /// Check if the given entity is valid in this context
    fn is_valid(&self, entity: &Self::Entity) -> bool {
        self.verify(entity).is_ok()
    }
}

/// Helper functions for verification
pub mod helpers {
    use super::*;
    
    /// Verify a collection of verifiable entities
    pub fn verify_all<T, E, I>(entities: I) -> Result<(), E>
    where
        T: Verifiable<Error = E>,
        I: IntoIterator<Item = T>,
    {
        for entity in entities {
            entity.verify()?;
        }
        Ok(())
    }
    
    /// Verify a collection of entities using a verifier
    pub fn verify_all_with<T, V, E, I>(verifier: &V, entities: I) -> Result<(), E>
    where
        V: Verify<T, Error = E>,
        I: IntoIterator<Item = T>,
    {
        for entity in entities {
            verifier.verify(&entity)?;
        }
        Ok(())
    }
}

// We're removing the ComposedVerifier struct as it doesn't align properly with
// the Verifier trait in signatures.rs. This can be implemented properly in a future
// update after fixing the core module dependencies. 