// Time domain types
//
// This module defines types for working with domains in the time module.

/// Domain identifier type
pub type DomainId = String;

/// Domain position in time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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