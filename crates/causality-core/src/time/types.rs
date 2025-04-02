// Time domain types
//
// This module defines types for working with domains in the time module.

use std::fmt;
use serde::{Serialize, Deserialize};

/// Domain identifier type
pub type DomainId = String;

/// Domain position in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPosition {
    /// The timestamp in the domain
    pub timestamp: u64,
    
    /// The position index (for domains with the same timestamp)
    pub index: u32,
}

impl DomainPosition {
    /// Create a new domain position
    pub fn new(timestamp: u64, index: u32) -> Self {
        Self { timestamp, index }
    }
    
    /// Create a new domain position with a timestamp
    pub fn with_timestamp(timestamp: u64) -> Self {
        Self { timestamp, index: 0 }
    }
    
    /// Get the timestamp
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Get the index
    pub fn get_index(&self) -> u32 {
        self.index
    }
    
    /// Check if this position is before another position
    pub fn is_before(&self, other: &Self) -> bool {
        self.timestamp < other.timestamp || (self.timestamp == other.timestamp && self.index < other.index)
    }
    
    /// Check if this position is after another position
    pub fn is_after(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp || (self.timestamp == other.timestamp && self.index > other.index)
    }
}

/// Domain attestation source type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainAttestationSource {
    /// System clock time source
    System,
    
    /// Network time protocol source
    NTP,
    
    /// External time source
    External(String),
    
    /// Consensus time source
    Consensus(String),
    
    /// User-provided time
    User,
    
    /// Custom time source
    Custom(String),
}

impl fmt::Display for DomainAttestationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainAttestationSource::System => write!(f, "System"),
            DomainAttestationSource::NTP => write!(f, "NTP"),
            DomainAttestationSource::External(src) => write!(f, "External({})", src),
            DomainAttestationSource::Consensus(src) => write!(f, "Consensus({})", src),
            DomainAttestationSource::User => write!(f, "User"),
            DomainAttestationSource::Custom(name) => write!(f, "Custom({})", name),
        }
    }
} 