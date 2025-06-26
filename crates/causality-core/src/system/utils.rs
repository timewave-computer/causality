//! Utility functions and types for the Causality system
//!
//! This module provides common utility functions and wrapper types
//! used throughout the system.

use serde::{Serialize, Deserialize};
use std::time::{Duration, UNIX_EPOCH};

/// Get the current time in milliseconds since Unix epoch
pub fn get_current_time_ms() -> u64 {
    crate::system::deterministic_system_time()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// SSZ-compatible wrapper for Duration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SszDuration {
    /// Duration in milliseconds 
    pub millis: u64,
}

impl SszDuration {
    /// Create from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self { millis }
    }
    
    /// Convert to std::time::Duration
    pub fn to_duration(&self) -> Duration {
        Duration::from_millis(self.millis)
    }
    
    /// Get milliseconds value
    pub fn as_millis(&self) -> u64 {
        self.millis
    }
}

impl From<Duration> for SszDuration {
    fn from(duration: Duration) -> Self {
        Self {
            millis: duration.as_millis() as u64,
        }
    }
}

impl From<SszDuration> for Duration {
    fn from(duration: SszDuration) -> Self {
        Duration::from_millis(duration.millis)
    }
}

impl ssz::Encode for SszDuration {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        8
    }

    fn ssz_bytes_len(&self) -> usize {
        8
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.millis.to_le_bytes());
    }
}

impl ssz::Decode for SszDuration {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        8
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() != 8 {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 8,
            });
        }
        
        let mut millis_bytes = [0u8; 8];
        millis_bytes.copy_from_slice(bytes);
        let millis = u64::from_le_bytes(millis_bytes);
        
        Ok(Self { millis })
    }
} 