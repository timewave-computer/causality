// Clock Module
//
// Provides clock implementations for timekeeping within the causality system.
// All clocks implement the Clock trait which provides methods for retrieving
// the current time, both regular and monotonic.

use crate::time::error::TimeError;
use crate::time::Timestamp;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The Clock trait provides methods for retrieving the current time.
/// All clock implementations must be Send + Sync for thread safety.
pub trait Clock: Send + Sync {
    /// Returns the current time as a Timestamp.
    fn now(&self) -> Result<Timestamp, TimeError>;
    
    /// Returns a monotonically increasing timestamp.
    /// Each call to this method must return a timestamp
    /// that is greater than or equal to the previous call.
    fn monotonic_now(&self) -> Result<Timestamp, TimeError>;
}

/// A clock that always returns the same timestamp.
/// Useful for testing and deterministic simulations.
#[derive(Debug)]
pub struct FixedClock {
    time: Timestamp,
}

impl FixedClock {
    /// Creates a new FixedClock with the specified timestamp.
    pub fn new(time: Timestamp) -> Self {
        Self { time }
    }

    /// Get the fixed timestamp of this clock
    pub fn timestamp(&self) -> Timestamp {
        self.time
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<Timestamp, TimeError> {
        Ok(self.time)
    }
    
    fn monotonic_now(&self) -> Result<Timestamp, TimeError> {
        Ok(self.time)
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::new(Timestamp::from_secs(0))
    }
}

/// A clock that uses a monotonic counter to ensure 
/// monotonically increasing timestamps.
#[derive(Debug)]
pub struct MonotonicClock {
    counter: AtomicU64,
}

impl MonotonicClock {
    /// Creates a new MonotonicClock with an initial timestamp.
    pub fn new(start_time: Timestamp) -> Self {
        Self {
            counter: AtomicU64::new(start_time.as_nanos()),
        }
    }

    /// Create a new monotonic clock starting from zero
    pub fn default() -> Self {
        Self::new(Timestamp::from_secs(0))
    }
}

impl Clock for MonotonicClock {
    fn now(&self) -> Result<Timestamp, TimeError> {
        let nanos = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(Timestamp::from_nanos(nanos))
    }
    
    fn monotonic_now(&self) -> Result<Timestamp, TimeError> {
        self.now()
    }
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new(Timestamp::from_secs(0))
    }
}

/// A thread-local clock that uses atomic counters for thread safety.
#[derive(Debug)]
pub struct ThreadLocalClock {
    counter: AtomicU64,
    start_time: Timestamp,
}

impl ThreadLocalClock {
    /// Create a new thread-local clock with the given start time.
    pub fn new(start_time: Timestamp) -> Self {
        Self {
            counter: AtomicU64::new(0),
            start_time,
        }
    }
}

impl Clock for ThreadLocalClock {
    fn now(&self) -> Result<Timestamp, TimeError> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let nanos = duration.as_nanos() as u64;
                Ok(Timestamp::from_nanos(nanos))
            }
            Err(_) => Err(TimeError::ClockError("System time is before UNIX_EPOCH".to_string())),
        }
    }
    
    fn monotonic_now(&self) -> Result<Timestamp, TimeError> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let nanos = self.start_time.as_nanos().checked_add(count)
            .ok_or_else(|| TimeError::OutOfBounds("Timestamp overflow".to_string()))?;
        
        Ok(Timestamp::from_nanos(nanos))
    }
}

impl Default for ThreadLocalClock {
    fn default() -> Self {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Self::new(Timestamp::from_nanos(duration.as_nanos() as u64)),
            Err(_) => Self::new(Timestamp::from_secs(0)),
        }
    }
}

/// Returns a default clock implementation that uses the system time.
pub fn default_clock() -> PhysicalClock {
    PhysicalClock::new()
}

/// Creates a fixed clock with the specified timestamp.
pub fn fixed_clock(timestamp: Timestamp) -> FixedClock {
    FixedClock::new(timestamp)
}

/// Creates a monotonic clock with the specified initial timestamp.
pub fn monotonic_clock(start_time: Timestamp) -> MonotonicClock {
    MonotonicClock::new(start_time)
}

/// Creates a thread-local clock with the current system time.
pub fn thread_local_clock() -> ThreadLocalClock {
    ThreadLocalClock::default()
}

/// Standard implementation of the Clock trait
#[derive(Debug)]
pub struct PhysicalClock {
    last_monotonic: AtomicU64,
}

impl PhysicalClock {
    /// Create a new physical clock
    pub fn new() -> Self {
        Self {
            last_monotonic: AtomicU64::new(0),
        }
    }
}

impl Clock for PhysicalClock {
    fn now(&self) -> Result<Timestamp, TimeError> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let seconds = duration.as_secs();
                let nanos = duration.subsec_nanos() as u64;
                
                // Combine seconds and nanos into a single timestamp
                let total_nanos = seconds
                    .checked_mul(1_000_000_000)
                    .and_then(|s_nanos| s_nanos.checked_add(nanos))
                    .ok_or_else(|| TimeError::OutOfBounds("Timestamp overflow".to_string()))?;
                
                Ok(Timestamp::from_nanos(total_nanos))
            }
            Err(_) => Err(TimeError::ClockError("System time is before UNIX_EPOCH".to_string())),
        }
    }

    fn monotonic_now(&self) -> Result<Timestamp, TimeError> {
        // Get the current time
        let current_time = self.now()?;
        let current_nanos = current_time.as_nanos();
        
        // Get the last monotonic time
        let last = self.last_monotonic.load(Ordering::SeqCst);
        
        // Update the monotonic time if the current time is greater
        let next = if current_nanos > last {
            current_nanos
        } else {
            last + 1
        };
        
        self.last_monotonic.store(next, Ordering::SeqCst);
        Ok(Timestamp::from_nanos(next))
    }
}

impl Default for PhysicalClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_fixed_clock() {
        let timestamp = Timestamp::from_secs(1600000000);
        let clock = FixedClock::new(timestamp);
        
        assert_eq!(clock.now().unwrap(), timestamp);
        assert_eq!(clock.monotonic_now().unwrap(), timestamp);
    }
    
    #[test]
    fn test_monotonic_clock() {
        let initial = Timestamp::from_secs(1600000000);
        let clock = MonotonicClock::new(initial);
        
        let t1 = clock.monotonic_now().unwrap();
        let t2 = clock.monotonic_now().unwrap();
        
        assert!(t2 >= t1, "Monotonic clock should return increasing timestamps");
    }
    
    #[test]
    fn test_thread_local_clock() -> Result<(), TimeError> {
        let clock = ThreadLocalClock::new(Timestamp::from_secs(0));
        
        let t1 = clock.now()?;
        sleep(Duration::from_millis(10));
        let t2 = clock.now()?;
        
        assert!(t2 > t1, "Thread-local clock should advance with time");
        
        let m1 = clock.monotonic_now()?;
        let m2 = clock.monotonic_now()?;
        
        assert!(m2 > m1, "Monotonic clock should return increasing timestamps");
        
        Ok(())
    }

    #[test]
    fn test_physical_clock() -> Result<(), TimeError> {
        let clock = PhysicalClock::new();
        
        let t1 = clock.now()?;
        let t2 = clock.now()?;
        
        // Physical times might be the same if called in quick succession
        assert!(t2 >= t1);
        
        let m1 = clock.monotonic_now()?;
        let m2 = clock.monotonic_now()?;
        
        // Monotonic times should be strictly increasing
        assert!(m2 > m1);
        
        Ok(())
    }
} 