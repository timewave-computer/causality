//! Simulated clock for deterministic time management in tests

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Simulated timestamp for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct SimulatedTimestamp(u64);

impl SimulatedTimestamp {
    /// Create a new simulated timestamp from seconds since epoch
    pub fn from_secs(secs: u64) -> Self {
        Self(secs)
    }
    
    /// Create a new simulated timestamp from nanoseconds (for compatibility)
    pub fn new(nanos: u64) -> Self {
        Self(nanos / 1_000_000_000) // Convert nanoseconds to seconds
    }
    
    /// Get the timestamp as seconds since epoch
    pub fn as_secs(&self) -> u64 {
        self.0
    }
    
    /// Get the timestamp value (for ID generation)
    pub fn timestamp(&self) -> u64 {
        self.0
    }
    
    /// Add duration to timestamp
    pub fn add_duration(&self, duration: Duration) -> Self {
        Self(self.0 + duration.as_secs())
    }
    
    /// Get duration between timestamps
    pub fn duration_since(&self, earlier: SimulatedTimestamp) -> Duration {
        Duration::from_secs(self.0.saturating_sub(earlier.0))
    }
}

/// Simulated clock for controlled time progression in tests
#[derive(Debug, Clone)]
pub struct SimulatedClock {
    current_time: Arc<Mutex<SimulatedTimestamp>>,
    time_scale: f64, // Speed multiplier for time progression
}

impl SimulatedClock {
    /// Create a new simulated clock starting at the given timestamp
    pub fn new(start_time: SimulatedTimestamp) -> Self {
        Self {
            current_time: Arc::new(Mutex::new(start_time)),
            time_scale: 1.0,
        }
    }
    
    /// Create a simulated clock starting at the current system time
    pub fn from_system_time() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Self::new(SimulatedTimestamp::from_secs(now))
    }
    
    /// Get the current simulated time
    pub fn now(&self) -> SimulatedTimestamp {
        *self.current_time.lock().unwrap()
    }
    
    /// Advance the simulated time by the given duration
    pub fn advance(&self, duration: Duration) {
        let mut current = self.current_time.lock().unwrap();
        *current = current.add_duration(duration);
    }
    
    /// Set the time scale (1.0 = normal speed, 2.0 = 2x speed, etc.)
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale;
    }
    
    /// Sleep for the given duration in simulated time
    pub async fn sleep(&self, duration: Duration) {
        if self.time_scale > 0.0 {
            let scaled_duration = Duration::from_secs_f64(duration.as_secs_f64() / self.time_scale);
            tokio::time::sleep(scaled_duration).await;
        }
        self.advance(duration);
    }
    
    /// Check if a timeout has occurred
    pub fn is_timeout(&self, start_time: SimulatedTimestamp, timeout: Duration) -> bool {
        let current = self.now();
        current.duration_since(start_time) >= timeout
    }
    
    /// Wait until the specified time
    pub async fn wait_until(&self, target_time: SimulatedTimestamp) {
        let current = self.now();
        if target_time > current {
            let duration = target_time.duration_since(current);
            self.sleep(duration).await;
        }
    }
}

impl Default for SimulatedClock {
    fn default() -> Self {
        Self::from_system_time()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulated_timestamp() {
        let ts1 = SimulatedTimestamp::from_secs(100);
        let ts2 = ts1.add_duration(Duration::from_secs(50));
        
        assert_eq!(ts2.as_secs(), 150);
        assert_eq!(ts2.duration_since(ts1), Duration::from_secs(50));
    }
    
    #[test]
    fn test_simulated_clock() {
        let clock = SimulatedClock::new(SimulatedTimestamp::from_secs(1000));
        
        assert_eq!(clock.now().as_secs(), 1000);
        
        clock.advance(Duration::from_secs(100));
        assert_eq!(clock.now().as_secs(), 1100);
    }
    
    #[test]
    fn test_timeout_detection() {
        let clock = SimulatedClock::new(SimulatedTimestamp::from_secs(1000));
        let start = clock.now();
        
        assert!(!clock.is_timeout(start, Duration::from_secs(100)));
        
        clock.advance(Duration::from_secs(150));
        assert!(clock.is_timeout(start, Duration::from_secs(100)));
    }
} 