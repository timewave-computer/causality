//! Utility System
//!
//! Utility types and functions for the causality-types crate.
//! This module contains trait interfaces and utility types.
//! These traits define the contract between different components of the system.

// use crate::primitive::ids::AsId; // Removed unused import
use crate::serialization::{Decode, Encode, SimpleSerialize, DecodeError};
// use serde::{Deserialize, Serialize}; // Removed unused import
use std::fmt::Debug;
use std::time::Duration;

//-----------------------------------------------------------------------------
// Module Exports
//-----------------------------------------------------------------------------

pub mod registry;
// pub mod context; // Removed as context traits moved to spi
pub mod transform;
pub mod time_utils;

pub use registry::AsRegistry;
// pub use context::AsExecutionContext; // Removed as AsExecutionContext moved to spi and re-exported from lib.rs
pub use crate::core::time::{
    AsCausalRelation, AsClock, AsCounterStrategy, AsTimestampGenerator,
};
pub use transform::{TransformFn, constant_transform, identity_transform};
pub use time_utils::get_current_time_ms;

//-----------------------------------------------------------------------------
// AsIdentifiable Trait
//-----------------------------------------------------------------------------

/// Trait for types that can be identified by an ID
pub trait AsIdentifiable {
    /// The ID type for this type
    type Id;

    /// Returns the ID for this instance
    fn id(&self) -> Self::Id;
    
    /// Compute the hash of this instance
    fn compute_hash(&self) -> [u8; 32] {
        // Default implementation - should be overridden by implementors
        [0u8; 32]
    }
}

/// Trait for types that can be resolved from their IDs
pub trait AsResolvable<T> {
    /// The ID type for resolving T
    type Id;

    /// Attempts to resolve an ID to an instance of T
    fn resolve(&self, id: &Self::Id) -> Option<T>;
}

//-----------------------------------------------------------------------------
// SSZ Duration Type
//-----------------------------------------------------------------------------

/// A wrapper around `std::time::Duration` to allow SSZ (de)serialization.
/// Serializes as (seconds: u64, nanos: u32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SszDuration {
    pub secs: u64,
    pub nanos: u32,
}

impl Encode for SszDuration {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.secs.to_le_bytes());
        bytes.extend_from_slice(&self.nanos.to_le_bytes());
        bytes
    }
}

impl Decode for SszDuration {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 12 { // 8 bytes for u64, 4 bytes for u32
            return Err(DecodeError { message: format!("SszDuration bytes length must be 12, got {}", bytes.len()) });
        }
        let mut secs_bytes = [0u8; 8];
        secs_bytes.copy_from_slice(&bytes[0..8]);
        let secs = u64::from_le_bytes(secs_bytes);

        let mut nanos_bytes = [0u8; 4];
        nanos_bytes.copy_from_slice(&bytes[8..12]);
        let nanos = u32::from_le_bytes(nanos_bytes);

        Ok(SszDuration { secs, nanos })
    }
}

impl SimpleSerialize for SszDuration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDuration(pub Duration);

//-----------------------------------------------------------------------------
// SSZ Duration Implementation
//-----------------------------------------------------------------------------

// Implement From traits for easy conversion
impl From<Duration> for SszDuration {
    fn from(duration: Duration) -> Self {
        SszDuration {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

impl From<SszDuration> for Duration {
    fn from(ssz_duration: SszDuration) -> Self {
        Duration::new(ssz_duration.secs, ssz_duration.nanos)
    }
}

// AsIdentifiable implementation is now in core::id module to avoid circular dependencies

// Re-export all utility types and traits
pub use crate::core::traits::*;
// Remove old resource imports that no longer exist
// pub use crate::resource::AsResourceProjectable;
