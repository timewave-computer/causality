// Time management module
//
// This module provides abstractions for managing time, timestamps, and durations.
// It includes core abstractions for time management and utilities for working with time.

pub mod clock;
pub mod duration;
pub mod map;
pub mod timestamp;
pub mod watch;
pub mod effect;
pub mod service;
pub mod types;
pub mod provider;
pub mod effect_handler;
pub mod factory;
pub mod facade;
pub mod adapter;

#[cfg(test)]
mod tests;

// Re-export important types to simplify imports
pub use map::TimeMap;
pub use map::TimeMapSnapshot;
pub use timestamp::Timestamp;
pub use duration::Duration;
pub use watch::TimeObserver;
pub use effect::{
    TimeEffect, TimeEffectHandler, BasicTimeEffectHandler, 
    TimeEffectType, TimeEffectFactory,
    AdvanceCausalTimeEffect, SetClockTimeEffect, RegisterAttestationEffect,
    TimeAttestation, AttestationSource,
};
pub use service::TimeService;
pub use types::{DomainId, DomainPosition};
pub use provider::{TimeProvider, TimeProviderFactory};
pub use effect_handler::{TimeEffectHandlerImpl, AttestationStore, InMemoryAttestationStore};
pub use factory::{
    configure_time_effect_system, configure_time_effect_system_with_components,
    create_simulation_time_system, create_in_memory_time_system,
};
pub use facade::{
    TimeFacade, CausalUpdateBuilder, ClockAttestationBuilder, TimeMapUpdateBuilder,
};
pub use adapter::{TimeSystemAdapter, TimeSystemAdapterFactory};

// Time constants
/// The default timescale (in picoseconds)
pub const DEFAULT_TIMESCALE: u64 = 1;

/// The default interval for ticks (in picoseconds)
pub const DEFAULT_TICK_INTERVAL: u64 = 1_000_000_000; // 1ms

/// The default number of ticks
pub const DEFAULT_TICK_COUNT: u64 = 100;

/// A constant representing the zero timestamp
pub const ZERO_TIMESTAMP: Timestamp = Timestamp::zero();

/// A constant representing the max timestamp
pub const MAX_TIMESTAMP: Timestamp = Timestamp::max();

/// A constant representing the zero duration
pub const ZERO_DURATION: Duration = Duration::zero();

/// A constant representing the max duration
pub const MAX_DURATION: Duration = Duration::max();

/// Helper function to get the current time from the system clock
pub fn now() -> Timestamp {
    SystemClock::now()
}

/// Helper function to sleep for the specified duration
pub fn sleep(duration: Duration) {
    std::thread::sleep(std::time::Duration::from_nanos(duration.as_nanos()));
}

/// Helper function to create a timer that measures elapsed time
pub fn timer() -> Timer {
    Timer::new()
}

/// Helper function to create a deadline with the specified duration from now
pub fn deadline(duration: Duration) -> Timestamp {
    SystemClock::now() + duration
}

/// A timer for measuring elapsed time
#[derive(Debug, Clone)]
pub struct Timer {
    start: Timestamp,
}

impl Timer {
    /// Create a new timer starting from now
    pub fn new() -> Self {
        Self {
            start: SystemClock::now(),
        }
    }
    
    /// Reset the timer to start from now
    pub fn reset(&mut self) {
        self.start = SystemClock::now();
    }
    
    /// Get the elapsed time since the timer started
    pub fn elapsed(&self) -> Duration {
        SystemClock::now() - self.start
    }
    
    /// Check if the timer has exceeded the specified duration
    pub fn has_elapsed(&self, duration: Duration) -> bool {
        self.elapsed() >= duration
    }
    
    /// Get the start timestamp of the timer
    pub fn start_time(&self) -> Timestamp {
        self.start
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

// Time management and representation abstractions
//
// This module provides time management functionality, including logical and
// physical time representations, time maps for tracking causal relationships
// between events across domains, and utilities for working with time.

// Public submodules
pub mod temporal;
pub mod physical;
pub mod time_map_snapshot;

// Re-exports for convenience
pub use temporal::{LogicalTimestamp, VectorClock, TimeProvider as LegacyTimeProvider};
pub use physical::{PhysicalTimestamp, TimestampFormat, TimeUtils};
pub use time_map_snapshot::TimeMapSnapshot as LegacyTimeMapSnapshot;

/// Create a new time map with the current timestamp
pub fn create_time_map() -> map::TimeMap {
    let timestamp = LogicalTimestamp::new(1, 0); // Default node 0, first event
    map::TimeMap::new()
}

/// Check if a snapshot is valid at a reference point
pub fn is_snapshot_valid_at(
    snapshot: &map::TimeMapSnapshot,
    reference: &map::TimeMapSnapshot,
) -> bool {
    // A snapshot is valid if it does not claim to observe any domain position
    // that the reference has not seen yet
    for (domain_id, position) in &snapshot.positions {
        if let Some(ref_position) = reference.positions.get(domain_id) {
            if position.is_after(ref_position) {
                return false;
            }
        } else {
            // Reference hasn't observed this domain at all
            return false;
        }
    }
    
    true
}

// Re-export SystemClock for internal use
use clock::SystemClock;

/// Trait for components that need to be notified of time events
pub trait TimeObserver {
    /// Handle a time event
    fn on_time_event(&mut self, event: TimeEvent) -> crate::error::Result<()>;
}

/// Types of time events that observers can handle
pub enum TimeEvent {
    /// A new time map snapshot is available
    NewSnapshot(map::TimeMapSnapshot),
    /// A request to synchronize time information
    SyncRequest,
    /// An inconsistency was detected in the time system
    InconsistencyDetected(String),
} 