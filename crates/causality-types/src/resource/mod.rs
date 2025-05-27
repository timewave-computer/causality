//! Resource model and related types
//!
//! Resource definitions, conversions, state management, nullifiers,
//! resource flows, and resource nodes.

pub mod types;
pub mod conversion;
pub mod state;
pub mod nullifier;
pub mod flow;
pub mod node;

// Re-exports for convenience
pub use types::*;
pub use conversion::*;
pub use state::*;
pub use nullifier::*;
pub use flow::*;
pub use node::*;

// Re-export unified core types - all resource functionality is now in causality_types::effect
pub use crate::effect::{Effect, Intent, Handler, Transaction};

// Keep resource tests for regression testing
#[cfg(test)]
pub mod resource_tests;

use crate::serialization::{SimpleSerialize, Encode, Decode, DecodeError};

/// Type of resources in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    /// Normal data resource
    Data,
    /// Effect resource
    Effect,
    /// Handler resource
    Handler,
    /// Intent resource
    Intent,
    /// Transaction resource
    Transaction,
}

impl Encode for ResourceType {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            ResourceType::Data => vec![0u8],
            ResourceType::Effect => vec![1u8],
            ResourceType::Handler => vec![2u8],
            ResourceType::Intent => vec![3u8],
            ResourceType::Transaction => vec![4u8],
        }
    }
}

impl Decode for ResourceType {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 1 {
            return Err(DecodeError {
                message: format!("ResourceType requires exactly 1 byte, got {}", bytes.len()),
            });
        }
        
        match bytes[0] {
            0 => Ok(ResourceType::Data),
            1 => Ok(ResourceType::Effect),
            2 => Ok(ResourceType::Handler),
            3 => Ok(ResourceType::Intent),
            4 => Ok(ResourceType::Transaction),
            _ => Err(DecodeError {
                message: format!("Invalid ResourceType variant: {}", bytes[0]),
            }),
        }
    }
}

impl SimpleSerialize for ResourceType {}
