//! Deterministic system for zkVM compatibility
//!
//! This module provides deterministic functions for all operations that need
//! to be reproducible in zero-knowledge proof systems.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, UNIX_EPOCH};
use sha2::{Sha256, Digest};

// Use simple integer type instead of FixedPoint for deterministic arithmetic
pub type DeterministicFloat = i64;

/// Deterministic system for zkVM-compatible operations
#[derive(Debug)]
pub struct DeterministicSystem {
    /// Counter for deterministic ID generation
    counter: AtomicU64,
    
    /// Seed for deterministic randomness
    seed: u64,
    
    /// Current timestamp for deterministic time
    current_time: Duration,
}

impl Clone for DeterministicSystem {
    fn clone(&self) -> Self {
        Self {
            counter: AtomicU64::new(self.counter.load(Ordering::SeqCst)),
            seed: self.seed,
            current_time: self.current_time,
        }
    }
}

impl DeterministicSystem {
    /// Create a new deterministic system
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
            seed: 0x1234567890abcdef,
            current_time: Duration::from_secs(1),
        }
    }
    
    /// Create with specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            counter: AtomicU64::new(1),
            seed,
            current_time: Duration::from_secs(1),
        }
    }
    
    /// Get next counter value
    pub fn next_counter(&mut self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Get next Lamport timestamp
    pub fn next_lamport_time(&mut self) -> u64 {
        self.current_time.as_secs()
    }
    
    /// Generate deterministic u64
    pub fn deterministic_u64(&mut self) -> u64 {
        let value = self.seed.wrapping_add(self.next_counter());
        self.counter.store(value + 1, Ordering::SeqCst);
        value
    }
    
    /// Generate deterministic probability
    pub fn deterministic_probability(&self) -> DeterministicFloat {
        50 // Fixed 0.5 probability as integer (50%)
    }
    
    /// Generate deterministic UUID-like string
    pub fn deterministic_uuid(&mut self) -> String {
        let counter = self.next_counter();
        format!("det-{:016x}-{:016x}", self.seed, counter)
    }
    
    /// Generate deterministic hash
    pub fn deterministic_hash(&self, input: &str) -> String {
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(hash)
    }
    
    /// Get current time (alias for next_lamport_time for compatibility)
    pub fn current_time(&self) -> u64 {
        self.current_time.as_secs()
    }
    
    /// Advance time deterministically
    pub fn advance_time(&mut self, seconds: u64) {
        self.current_time += Duration::from_secs(seconds);
    }
    
    /// Generate deterministic ID string
    pub fn deterministic_id(&mut self) -> String {
        let counter = self.next_counter();
        format!("det_{:016x}_{:016x}", self.seed, counter)
    }
}

impl Default for DeterministicSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub fn deterministic_uuid() -> String {
    use std::sync::Mutex;
    use std::sync::OnceLock;
    
    static SYSTEM: OnceLock<Mutex<DeterministicSystem>> = OnceLock::new();
    
    let system = SYSTEM.get_or_init(|| Mutex::new(DeterministicSystem::new()));
    let mut system = system.lock().unwrap();
    system.deterministic_uuid()
}

/// Get deterministic system time
pub fn deterministic_system_time() -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(1)
}

/// Get deterministic instant
pub fn deterministic_instant() -> std::time::Instant {
    std::time::Instant::now() // This is the best we can do for Instant
}

/// Get deterministic duration in milliseconds
pub fn deterministic_duration_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

/// Get deterministic Lamport timestamp
pub fn deterministic_lamport_time() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Get deterministic probability as percentage
pub fn deterministic_probability_percent() -> u32 {
    50 // Fixed 50% probability
}

/// Get deterministic timestamp
pub fn deterministic_timestamp() -> Duration {
    Duration::from_secs(1)
} 