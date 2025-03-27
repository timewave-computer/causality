// Time watching functionality
//
// This module provides abstractions for watching and monitoring time-related events.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::clock::{Clock, ClockSource};
use super::duration::Duration;
use super::timestamp::Timestamp;

/// A trait for watching time and monitoring deadlines
pub trait TimeWatcher {
    /// Check if a deadline has been reached
    fn is_deadline_reached(&self, deadline: Timestamp) -> bool;
    
    /// Check if a duration has elapsed since a start time
    fn has_elapsed(&self, start: Timestamp, duration: Duration) -> bool;
    
    /// Wait until a deadline is reached
    fn wait_until_deadline(&self, deadline: Timestamp);
    
    /// Wait for a duration from now
    fn wait_for(&self, duration: Duration);
    
    /// Wait for a duration from a start time
    fn wait_from(&self, start: Timestamp, duration: Duration);
    
    /// Get the current time according to this watcher
    fn now(&self) -> Timestamp;
    
    /// Calculate a deadline from now plus a duration
    fn deadline_from_now(&self, duration: Duration) -> Timestamp;
}

/// A time watch that monitors deadlines and durations
#[derive(Debug, Clone)]
pub struct TimeWatch<C: ClockSource> {
    /// The clock that provides time information
    clock: C,
}

impl<C: ClockSource> TimeWatch<C> {
    /// Create a new time watch with the specified clock
    pub fn new(clock: C) -> Self {
        Self { clock }
    }
    
    /// Get a reference to the inner clock
    pub fn clock(&self) -> &C {
        &self.clock
    }
}

impl<C: ClockSource> TimeWatcher for TimeWatch<C> {
    fn is_deadline_reached(&self, deadline: Timestamp) -> bool {
        self.clock.now() >= deadline
    }
    
    fn has_elapsed(&self, start: Timestamp, duration: Duration) -> bool {
        let current = self.clock.now();
        current >= start + duration
    }
    
    fn wait_until_deadline(&self, deadline: Timestamp) {
        // If the clock is deterministic, we can't wait
        if self.clock.is_deterministic() {
            return;
        }
        
        // Poll until the deadline is reached
        while !self.is_deadline_reached(deadline) {
            // For non-deterministic clocks like system clock, sleep to avoid CPU spin
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    fn wait_for(&self, duration: Duration) {
        let deadline = self.deadline_from_now(duration);
        self.wait_until_deadline(deadline);
    }
    
    fn wait_from(&self, start: Timestamp, duration: Duration) {
        let deadline = start + duration;
        self.wait_until_deadline(deadline);
    }
    
    fn now(&self) -> Timestamp {
        self.clock.now()
    }
    
    fn deadline_from_now(&self, duration: Duration) -> Timestamp {
        self.clock.now() + duration
    }
}

/// A utility for watching deadlines and timeouts
#[derive(Debug)]
pub struct DeadlineWatcher {
    /// The inner time watcher
    watcher: Arc<dyn TimeWatcher + Send + Sync>,
    
    /// The deadline
    deadline: AtomicU64,
    
    /// Whether the deadline has been reached
    reached: AtomicBool,
}

impl DeadlineWatcher {
    /// Create a new deadline watcher with the specified watcher and deadline
    pub fn new(watcher: Arc<dyn TimeWatcher + Send + Sync>, deadline: Timestamp) -> Self {
        Self {
            watcher,
            deadline: AtomicU64::new(deadline.as_nanos()),
            reached: AtomicBool::new(false),
        }
    }
    
    /// Create a new deadline watcher that will expire after the specified duration
    pub fn with_duration(watcher: Arc<dyn TimeWatcher + Send + Sync>, duration: Duration) -> Self {
        let deadline = watcher.deadline_from_now(duration);
        Self::new(watcher, deadline)
    }
    
    /// Get the current deadline
    pub fn deadline(&self) -> Timestamp {
        Timestamp::from_nanos(self.deadline.load(Ordering::SeqCst))
    }
    
    /// Set a new deadline
    pub fn set_deadline(&self, deadline: Timestamp) {
        self.deadline.store(deadline.as_nanos(), Ordering::SeqCst);
        self.reached.store(false, Ordering::SeqCst);
    }
    
    /// Check if the deadline has been reached
    pub fn is_reached(&self) -> bool {
        // If already marked as reached, return true quickly
        if self.reached.load(Ordering::SeqCst) {
            return true;
        }
        
        // Check if the deadline has been reached
        let deadline = self.deadline();
        let is_reached = self.watcher.is_deadline_reached(deadline);
        
        // If reached, set the flag for future checks
        if is_reached {
            self.reached.store(true, Ordering::SeqCst);
        }
        
        is_reached
    }
    
    /// Extend the deadline by the specified duration
    pub fn extend(&self, duration: Duration) {
        let current_deadline = self.deadline();
        let new_deadline = current_deadline + duration;
        self.set_deadline(new_deadline);
    }
    
    /// Wait until the deadline is reached
    pub fn wait(&self) {
        if !self.is_reached() {
            self.watcher.wait_until_deadline(self.deadline());
            self.reached.store(true, Ordering::SeqCst);
        }
    }
    
    /// Time remaining until the deadline
    pub fn remaining(&self) -> Duration {
        let now = self.watcher.now();
        let deadline = self.deadline();
        
        if now >= deadline {
            Duration::zero()
        } else {
            deadline - now
        }
    }
}

/// A periodic timer that fires at regular intervals
#[derive(Debug)]
pub struct PeriodicTimer {
    /// The inner time watcher
    watcher: Arc<dyn TimeWatcher + Send + Sync>,
    
    /// The interval duration
    interval: Duration,
    
    /// The next time the timer will fire
    next_time: AtomicU64,
}

impl PeriodicTimer {
    /// Create a new periodic timer with the specified watcher and interval
    pub fn new(watcher: Arc<dyn TimeWatcher + Send + Sync>, interval: Duration) -> Self {
        let now = watcher.now();
        let next_time = now + interval;
        
        Self {
            watcher,
            interval,
            next_time: AtomicU64::new(next_time.as_nanos()),
        }
    }
    
    /// Get the interval duration
    pub fn interval(&self) -> Duration {
        self.interval
    }
    
    /// Set a new interval duration
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }
    
    /// Check if the timer has fired and update the next firing time
    pub fn check(&self) -> bool {
        let now = self.watcher.now();
        let next_time = Timestamp::from_nanos(self.next_time.load(Ordering::SeqCst));
        
        if now >= next_time {
            // Update the next time to fire
            let mut new_next_time = next_time;
            
            // Skip any missed intervals and set to the next future interval
            while new_next_time <= now {
                new_next_time = new_next_time + self.interval;
            }
            
            self.next_time.store(new_next_time.as_nanos(), Ordering::SeqCst);
            true
        } else {
            false
        }
    }
    
    /// Reset the timer to fire after one interval from now
    pub fn reset(&self) {
        let now = self.watcher.now();
        let next_time = now + self.interval;
        self.next_time.store(next_time.as_nanos(), Ordering::SeqCst);
    }
    
    /// Time remaining until the next firing
    pub fn remaining(&self) -> Duration {
        let now = self.watcher.now();
        let next_time = Timestamp::from_nanos(self.next_time.load(Ordering::SeqCst));
        
        if now >= next_time {
            Duration::zero()
        } else {
            next_time - now
        }
    }
    
    /// Wait until the timer fires
    pub fn wait(&self) -> bool {
        let next_time = Timestamp::from_nanos(self.next_time.load(Ordering::SeqCst));
        self.watcher.wait_until_deadline(next_time);
        self.check()
    }
}

/// Helper functions to create time watches
pub mod helpers {
    use super::*;
    use super::super::clock::{SystemClock, ManualClock};
    
    /// Create a new time watch using the system clock
    pub fn system_watch() -> TimeWatch<SystemClock> {
        TimeWatch::new(SystemClock::new())
    }
    
    /// Create a new time watch using a manual clock
    pub fn manual_watch(initial: Timestamp) -> TimeWatch<ManualClock> {
        TimeWatch::new(ManualClock::new(initial))
    }
    
    /// Create a deadline watcher using the system clock
    pub fn deadline_watcher(deadline: Timestamp) -> DeadlineWatcher {
        let watch = Arc::new(system_watch());
        DeadlineWatcher::new(watch, deadline)
    }
    
    /// Create a deadline watcher using a duration from now
    pub fn timeout(duration: Duration) -> DeadlineWatcher {
        let watch = Arc::new(system_watch());
        DeadlineWatcher::with_duration(watch, duration)
    }
    
    /// Create a periodic timer using the system clock
    pub fn periodic_timer(interval: Duration) -> PeriodicTimer {
        let watch = Arc::new(system_watch());
        PeriodicTimer::new(watch, interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::clock::ManualClock;
    
    #[test]
    fn test_time_watch() {
        let mut clock = ManualClock::zero();
        let watch = TimeWatch::new(clock.clone());
        
        assert_eq!(watch.now(), Timestamp::zero());
        
        // Test deadline calculation
        let deadline = watch.deadline_from_now(Duration::from_secs(5));
        assert_eq!(deadline, Timestamp::from_secs(5));
        
        // Test deadline checking
        assert!(!watch.is_deadline_reached(deadline));
        
        // Advance the clock to the deadline
        clock.advance(Duration::from_secs(5));
        assert!(watch.is_deadline_reached(deadline));
        
        // Test duration elapsed checking
        let start = Timestamp::from_secs(5);
        assert!(!watch.has_elapsed(start, Duration::from_secs(5)));
        
        clock.advance(Duration::from_secs(5));
        assert!(watch.has_elapsed(start, Duration::from_secs(5)));
    }
    
    #[test]
    fn test_deadline_watcher() {
        let mut clock = ManualClock::zero();
        let watch = Arc::new(TimeWatch::new(clock.clone()));
        
        let deadline = Timestamp::from_secs(10);
        let watcher = DeadlineWatcher::new(watch.clone(), deadline);
        
        assert_eq!(watcher.deadline(), deadline);
        assert!(!watcher.is_reached());
        
        // Advance to just before the deadline
        clock.advance(Duration::from_secs(9));
        assert!(!watcher.is_reached());
        
        // Advance to the deadline
        clock.advance(Duration::from_secs(1));
        assert!(watcher.is_reached());
        
        // Test extending the deadline
        watcher.extend(Duration::from_secs(5));
        assert_eq!(watcher.deadline(), Timestamp::from_secs(15));
        assert!(!watcher.is_reached());
        
        // Test remaining time
        assert_eq!(watcher.remaining(), Duration::from_secs(5));
    }
    
    #[test]
    fn test_periodic_timer() {
        let mut clock = ManualClock::zero();
        let watch = Arc::new(TimeWatch::new(clock.clone()));
        
        let interval = Duration::from_secs(5);
        let timer = PeriodicTimer::new(watch.clone(), interval);
        
        // Should not fire immediately
        assert!(!timer.check());
        
        // Advance just before the first interval
        clock.advance(Duration::from_secs(4));
        assert!(!timer.check());
        
        // Advance to the first interval
        clock.advance(Duration::from_secs(1));
        assert!(timer.check());
        
        // Should not fire again immediately
        assert!(!timer.check());
        
        // Next firing should be at t=10
        assert_eq!(timer.remaining(), Duration::from_secs(5));
        
        // Test resetting the timer
        timer.reset();
        // Next firing should now be at t=14
        assert_eq!(timer.remaining(), Duration::from_secs(5));
        
        // Advance to t=10, should not fire
        clock.advance(Duration::from_secs(5));
        assert!(!timer.check());
        
        // Advance to t=14, should fire
        clock.advance(Duration::from_secs(4));
        assert!(timer.check());
    }
} 