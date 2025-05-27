//! Time and Causality Types
//!
//! This module provides types for representing time, causality relationships,
//! and logical clocks in the Causality framework. It supports both wall clock
//! time and logical time for ordering events across distributed domains.

use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::primitive::ids::DomainId;

//-----------------------------------------------------------------------------
// Constant
//-----------------------------------------------------------------------------

/// Maximum number of domains that can be tracked in a compact vector clock
pub const MAX_CLOCK_DOMAINS: usize = 8;

//-----------------------------------------------------------------------------
// Causal Relationship
//-----------------------------------------------------------------------------

/// Causal relationships between events in a distributed system
///
/// These relationships form the foundation of the happens-before relation
/// in the Causality framework, allowing for deterministic ordering of events
/// across distributed domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Causality {
    /// First event happens before second event
    Before,

    /// First event happens after second event
    After,

    /// Events are concurrent (neither happens before the other)
    Concurrent,

    /// Events are equal (same logical time)
    Equal,
}

//-----------------------------------------------------------------------------
// Clock Type
//-----------------------------------------------------------------------------

/// Clock time source mode determining time source behavior
///
/// This enables switching between real system time and mock time
/// for deterministic testing in a controlled environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockMode {
    /// Use real system time
    Real,

    /// Use mock time for deterministic testing
    Mock,
}

/// Clock implementation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockType {
    /// Lamport clock (simple scalar)
    Lamport,

    /// Vector clock (vector of counters)
    Vector,

    /// Matrix clock (matrix of counters)
    Matrix,
}

/// A wall-clock timestamp based on system time (milliseconds since epoch)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct WallClock(pub u64);

/// A comprehensive timestamp combining logical clock, wall clock, and domain ID
///
/// This timestamp provides the foundation for causal ordering in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timestamp {
    /// Domain ID where this timestamp originated
    pub domain_id: DomainId,

    /// Logical clock value (can be Lamport or part of Vector clock)
    pub logical: u64,

    /// Wall clock time (ms since epoch, for human-readable time)
    pub wall: WallClock,
}

/// Mock clock state for deterministic testing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockState {
    /// Current mocked wall clock time
    pub current_time: u64,

    /// Step increment for advance operations
    pub step_increment: u64,
}

/// Clock configuration options
pub struct ClockConfig {
    /// Mode (real or mock)
    pub mode: ClockMode,

    /// Implementation type
    pub clock_type: ClockType,

    /// Domain ID for the clock
    pub domain_id: DomainId,
}

//-----------------------------------------------------------------------------
// Causal Relation Trait
//-----------------------------------------------------------------------------

/// Trait for causal relation between events
pub trait AsCausalRelation {
    /// Check if this event happens before another event
    fn happens_before(&self, other: &Self) -> bool;

    /// Check if this event happens after another event
    fn happens_after(&self, other: &Self) -> bool;

    /// Check if this event is concurrent with another event
    fn concurrent_with(&self, other: &Self) -> bool;
}

//-----------------------------------------------------------------------------
// Clock Trait
//-----------------------------------------------------------------------------

/// Trait for clock implementations that track causal dependencies
pub trait AsClock {
    /// Get the current timestamp
    fn now(&self) -> Timestamp;

    /// Increment the clock and return a new timestamp
    fn tick(&self) -> Timestamp;

    /// Update the clock based on a received timestamp
    fn update(&self, other: &Timestamp) -> Timestamp;

    /// Get the domain ID associated with this clock
    fn domain_id(&self) -> &DomainId;

    /// Set the clock to a specific timestamp for deterministic testing
    fn set_time(&self, timestamp: Timestamp) -> Result<(), anyhow::Error>;

    /// Advance the clock by a specific amount for deterministic testing
    fn advance(&self, duration_ms: u64) -> Result<(), anyhow::Error>;

    /// Reset the clock to use real time
    fn reset(&self) -> Result<(), anyhow::Error>;

    /// Check if this clock is currently using mock time
    fn is_mocked(&self) -> bool;
}

//-----------------------------------------------------------------------------
// Generator Trait
//-----------------------------------------------------------------------------

/// Trait for types that can generate timestamps
pub trait AsTimestampGenerator {
    /// Generate a new timestamp
    fn generate(&self) -> Timestamp;

    /// Generate a timestamp from milliseconds since epoch
    fn from_millis(millis: u64) -> Timestamp;
}

//-----------------------------------------------------------------------------
// Time Strategy Trait
//-----------------------------------------------------------------------------

/// Trait for counter strategies used in causal clocks
pub trait AsCounterStrategy {
    /// Get the counter value for a domain
    fn get_counter(&self, domain_id: &DomainId) -> u32;

    /// Increment the counter for a domain
    fn increment(&self, domain_id: &DomainId) -> u32;

    /// Update the counter based on a received counter value
    fn update(&self, domain_id: &DomainId, counter: u32);

    /// Compare two timestamps and determine their causal relationship
    fn compare(&self, a: &Timestamp, b: &Timestamp) -> Causality;
}

//-----------------------------------------------------------------------------
// Timestamp Implementation
//-----------------------------------------------------------------------------

// Implement AsCausalRelation for Timestamp
impl AsCausalRelation for Timestamp {
    fn happens_before(&self, other: &Self) -> bool {
        if self.domain_id == other.domain_id {
            self.logical < other.logical
        } else {
            // For timestamps from different domains, we compare the wall clock times
            self.wall.0 < other.wall.0
        }
    }

    fn happens_after(&self, other: &Self) -> bool {
        other.happens_before(self)
    }

    fn concurrent_with(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl crate::serialization::Encode for Timestamp {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.logical.to_le_bytes());
        bytes.extend_from_slice(&self.wall.0.to_le_bytes());
        bytes
    }
}

impl crate::serialization::Decode for Timestamp {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
        if bytes.len() < 48 { // 32 bytes for DomainId + 8 bytes for logical + 8 bytes for wall
            return Err(crate::serialization::DecodeError { 
                message: "Insufficient bytes for Timestamp".to_string() 
            });
        }
        
        let mut offset = 0;
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32; // DomainId is always 32 bytes
        
        let logical = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]
        ]);
        offset += 8;
        
        let wall_time = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]
        ]);
        
        Ok(Timestamp {
            domain_id,
            logical,
            wall: WallClock(wall_time),
        })
    }
}

impl crate::serialization::SimpleSerialize for Timestamp {}

impl Timestamp {
    /// Create a new timestamp with the current wall clock time
    pub fn now() -> Self {
        let wall_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
            
        Self {
            domain_id: DomainId::new([0u8; 32]),
            logical: 0,
            wall: WallClock(wall_time),
        }
    }
    
    /// Create a timestamp for a specific domain
    pub fn for_domain(domain_id: DomainId) -> Self {
        let mut ts = Self::now();
        ts.domain_id = domain_id;
        ts
    }

    /// Get timestamp as milliseconds since epoch
    pub fn as_millis(&self) -> u64 {
        self.wall.0
    }
}
