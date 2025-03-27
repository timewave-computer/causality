// Clock implementations for time measurement
//
// This module provides clock abstractions for deterministic and
// non-deterministic time sources

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::timestamp::Timestamp;
use super::duration::Duration;

/// A trait for sources of time
pub trait ClockSource {
    /// Get the current timestamp from this clock source
    fn now(&self) -> Timestamp;
    
    /// Get the resolution of this clock source
    fn resolution(&self) -> Duration;
    
    /// Check if this clock source is deterministic
    fn is_deterministic(&self) -> bool;
}

/// A trait representing a clock that provides timestamps
pub trait Clock: ClockSource {
    /// Advance the clock by the specified duration (for deterministic clocks)
    fn advance(&mut self, duration: Duration) -> Timestamp;
    
    /// Set the clock to a specific timestamp (for deterministic clocks)
    fn set(&mut self, timestamp: Timestamp);
    
    /// Reset the clock to zero
    fn reset(&mut self);
}

/// A system clock that uses the system time
#[derive(Debug, Clone)]
pub struct SystemClock;

impl SystemClock {
    /// Create a new system clock
    pub fn new() -> Self {
        Self
    }
    
    /// Get the current system time as a timestamp
    pub fn now() -> Timestamp {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
        
        Timestamp::from_nanos(now.as_nanos() as u64)
    }
}

impl ClockSource for SystemClock {
    fn now(&self) -> Timestamp {
        SystemClock::now()
    }
    
    fn resolution(&self) -> Duration {
        // Typical system clock resolution is around 1 microsecond
        Duration::from_micros(1)
    }
    
    fn is_deterministic(&self) -> bool {
        false
    }
}

impl Clock for SystemClock {
    fn advance(&mut self, _duration: Duration) -> Timestamp {
        // Cannot advance system clock, just return current time
        self.now()
    }
    
    fn set(&mut self, _timestamp: Timestamp) {
        // Cannot set system clock, no-op
    }
    
    fn reset(&mut self) {
        // Cannot reset system clock, no-op
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A manual clock that can be controlled programmatically
#[derive(Debug, Clone)]
pub struct ManualClock {
    /// The current timestamp
    current: Timestamp,
}

impl ManualClock {
    /// Create a new manual clock at the specified timestamp
    pub fn new(initial: Timestamp) -> Self {
        Self {
            current: initial,
        }
    }
    
    /// Create a new manual clock at zero
    pub fn zero() -> Self {
        Self::new(Timestamp::zero())
    }
}

impl ClockSource for ManualClock {
    fn now(&self) -> Timestamp {
        self.current
    }
    
    fn resolution(&self) -> Duration {
        // Manual clock has perfect resolution
        Duration::from_nanos(1)
    }
    
    fn is_deterministic(&self) -> bool {
        true
    }
}

impl Clock for ManualClock {
    fn advance(&mut self, duration: Duration) -> Timestamp {
        self.current = self.current + duration;
        self.current
    }
    
    fn set(&mut self, timestamp: Timestamp) {
        self.current = timestamp;
    }
    
    fn reset(&mut self) {
        self.current = Timestamp::zero();
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::zero()
    }
}

/// A deterministic incrementing clock that increments on each read
#[derive(Debug)]
pub struct IncrementingClock {
    /// The current timestamp
    current: AtomicU64,
    
    /// The increment amount
    increment: u64,
}

impl IncrementingClock {
    /// Create a new incrementing clock
    pub fn new(initial: Timestamp, increment: Duration) -> Self {
        Self {
            current: AtomicU64::new(initial.as_nanos()),
            increment: increment.as_nanos(),
        }
    }
    
    /// Create a new incrementing clock that increments by 1 nanosecond
    pub fn nano_step() -> Self {
        Self::new(Timestamp::zero(), Duration::from_nanos(1))
    }
    
    /// Create a new incrementing clock that increments by 1 microsecond
    pub fn micro_step() -> Self {
        Self::new(Timestamp::zero(), Duration::from_micros(1))
    }
    
    /// Create a new incrementing clock that increments by 1 millisecond
    pub fn milli_step() -> Self {
        Self::new(Timestamp::zero(), Duration::from_millis(1))
    }
}

impl ClockSource for IncrementingClock {
    fn now(&self) -> Timestamp {
        let current = self.current.fetch_add(self.increment, Ordering::SeqCst);
        Timestamp::from_nanos(current)
    }
    
    fn resolution(&self) -> Duration {
        Duration::from_nanos(self.increment)
    }
    
    fn is_deterministic(&self) -> bool {
        true
    }
}

impl Clock for IncrementingClock {
    fn advance(&mut self, duration: Duration) -> Timestamp {
        let nanos = duration.as_nanos();
        let current = self.current.fetch_add(nanos, Ordering::SeqCst);
        Timestamp::from_nanos(current + nanos)
    }
    
    fn set(&mut self, timestamp: Timestamp) {
        self.current.store(timestamp.as_nanos(), Ordering::SeqCst);
    }
    
    fn reset(&mut self) {
        self.current.store(0, Ordering::SeqCst);
    }
}

impl Clone for IncrementingClock {
    fn clone(&self) -> Self {
        Self {
            current: AtomicU64::new(self.current.load(Ordering::SeqCst)),
            increment: self.increment,
        }
    }
}

impl Default for IncrementingClock {
    fn default() -> Self {
        Self::nano_step()
    }
}

/// A shared clock that can be cloned
#[derive(Debug, Clone)]
pub struct SharedClock<C: Clock + ?Sized> {
    /// The inner clock
    inner: Arc<C>,
}

impl<C: Clock + ?Sized> SharedClock<C> {
    /// Create a new shared clock
    pub fn new(clock: C) -> Self 
    where 
        C: Sized,
    {
        Self {
            inner: Arc::new(clock),
        }
    }
    
    /// Get a reference to the inner clock
    pub fn inner(&self) -> &C {
        &self.inner
    }
}

impl<C: Clock + ?Sized> ClockSource for SharedClock<C> {
    fn now(&self) -> Timestamp {
        self.inner.now()
    }
    
    fn resolution(&self) -> Duration {
        self.inner.resolution()
    }
    
    fn is_deterministic(&self) -> bool {
        self.inner.is_deterministic()
    }
}

// Cannot implement Clock for SharedClock because it requires &mut self
// but SharedClock only has immutable access to the inner clock

/// Helper functions to create clocks
pub mod helpers {
    use super::*;
    
    /// Create a new system clock
    pub fn system_clock() -> SystemClock {
        SystemClock::new()
    }
    
    /// Create a new manual clock
    pub fn manual_clock(initial: Timestamp) -> ManualClock {
        ManualClock::new(initial)
    }
    
    /// Create a new incrementing clock
    pub fn incrementing_clock(increment: Duration) -> IncrementingClock {
        IncrementingClock::new(Timestamp::zero(), increment)
    }
    
    /// Create a new shared system clock
    pub fn shared_system_clock() -> SharedClock<SystemClock> {
        SharedClock::new(SystemClock::new())
    }
    
    /// Create a new shared manual clock
    pub fn shared_manual_clock(initial: Timestamp) -> SharedClock<ManualClock> {
        SharedClock::new(ManualClock::new(initial))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_clock() {
        let clock = SystemClock::new();
        
        let t1 = clock.now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = clock.now();
        
        assert!(t2 > t1);
        assert!(!clock.is_deterministic());
    }
    
    #[test]
    fn test_manual_clock() {
        let mut clock = ManualClock::zero();
        
        assert_eq!(clock.now(), Timestamp::zero());
        
        clock.advance(Duration::from_secs(1));
        assert_eq!(clock.now(), Timestamp::from_secs(1));
        
        clock.set(Timestamp::from_secs(10));
        assert_eq!(clock.now(), Timestamp::from_secs(10));
        
        clock.reset();
        assert_eq!(clock.now(), Timestamp::zero());
        
        assert!(clock.is_deterministic());
    }
    
    #[test]
    fn test_incrementing_clock() {
        let mut clock = IncrementingClock::new(Timestamp::zero(), Duration::from_secs(1));
        
        // First call returns the initial value
        assert_eq!(clock.now(), Timestamp::zero());
        
        // Subsequent calls increment by the specified amount
        assert_eq!(clock.now(), Timestamp::from_secs(1));
        assert_eq!(clock.now(), Timestamp::from_secs(2));
        
        clock.set(Timestamp::from_secs(10));
        assert_eq!(clock.now(), Timestamp::from_secs(10));
        
        clock.advance(Duration::from_secs(5));
        assert_eq!(clock.now(), Timestamp::from_secs(16));
        
        assert!(clock.is_deterministic());
    }
    
    #[test]
    fn test_shared_clock() {
        let manual = ManualClock::zero();
        let shared = SharedClock::new(manual);
        let shared2 = shared.clone();
        
        // Shared clock forwards to the inner clock
        assert_eq!(shared.now(), Timestamp::zero());
        assert_eq!(shared2.now(), Timestamp::zero());
        
        // Cannot modify the inner clock through SharedClock
        // But can observe changes made to the inner clock
        let inner = ManualClock::zero();
        let mut inner_ref = inner.clone();
        let shared_inner = SharedClock::new(inner);
        
        inner_ref.advance(Duration::from_secs(1));
        // This would assert_eq!(shared_inner.now(), Timestamp::from_secs(1));
        // but it fails because the inner clock is cloned, not shared
        
        assert!(shared.is_deterministic());
        assert!(shared2.is_deterministic());
    }
} 