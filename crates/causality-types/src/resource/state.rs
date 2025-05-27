//! State management types and utilities for tracking and managing resource
//! state in the Causality framework.

use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};

/// Represents the state of a resource at a specific point in time.
/// This is used for state management and tracking resource changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ResourceState {
    /// Resource is available for use
    #[default]
    Available,
    /// Resource has been consumed/spent
    Consumed,
    /// Resource is temporarily locked
    Locked,
}

impl Encode for ResourceState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            ResourceState::Available => vec![0u8],
            ResourceState::Consumed => vec![1u8],
            ResourceState::Locked => vec![2u8],
        }
    }
}

impl Decode for ResourceState {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::new("Empty bytes for ResourceState"));
        }
        
        match bytes[0] {
            0 => Ok(ResourceState::Available),
            1 => Ok(ResourceState::Consumed),
            2 => Ok(ResourceState::Locked),
            _ => Err(DecodeError::new("Invalid ResourceState variant")),
        }
    }
}

impl SimpleSerialize for ResourceState {}


// TODO: Consider if other state-related types are needed here in the future.
