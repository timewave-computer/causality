// Duration implementation
//
// This module provides the core duration type for representing
// time spans across the Causality system.

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// A duration representing a span of time
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Duration {
    /// The number of nanoseconds
    nanos: u64,
}

impl Duration {
    /// Create a new duration from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
    
    /// Create a new duration from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros * 1_000,
        }
    }
    
    /// Create a new duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            nanos: millis * 1_000_000,
        }
    }
    
    /// Create a new duration from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            nanos: secs * 1_000_000_000,
        }
    }
    
    /// Create a new duration from minutes
    pub fn from_minutes(minutes: u64) -> Self {
        Self {
            nanos: minutes * 60 * 1_000_000_000,
        }
    }
    
    /// Create a new duration from hours
    pub fn from_hours(hours: u64) -> Self {
        Self {
            nanos: hours * 60 * 60 * 1_000_000_000,
        }
    }
    
    /// Get the number of nanoseconds
    pub fn as_nanos(&self) -> u64 {
        self.nanos
    }
    
    /// Get the number of microseconds
    pub fn as_micros(&self) -> u64 {
        self.nanos / 1_000
    }
    
    /// Get the number of milliseconds
    pub fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }
    
    /// Get the number of seconds
    pub fn as_secs(&self) -> u64 {
        self.nanos / 1_000_000_000
    }
    
    /// Get the number of minutes
    pub fn as_minutes(&self) -> u64 {
        self.nanos / (60 * 1_000_000_000)
    }
    
    /// Get the number of hours
    pub fn as_hours(&self) -> u64 {
        self.nanos / (60 * 60 * 1_000_000_000)
    }
    
    /// Get the number of seconds as a floating-point value
    pub fn as_secs_f64(&self) -> f64 {
        self.nanos as f64 / 1_000_000_000.0
    }
    
    /// Get the zero duration
    pub const fn zero() -> Self {
        Self { nanos: 0 }
    }
    
    /// Get the maximum possible duration
    pub const fn max() -> Self {
        Self { nanos: u64::MAX }
    }
    
    /// Check if this duration is zero
    pub fn is_zero(&self) -> bool {
        self.nanos == 0
    }
    
    /// Check if this duration is the maximum value
    pub fn is_max(&self) -> bool {
        self.nanos == u64::MAX
    }
    
    /// Add another duration to this one, saturating at the maximum value
    pub fn saturating_add(&self, other: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_add(other.nanos),
        }
    }
    
    /// Subtract another duration from this one, saturating at zero
    pub fn saturating_sub(&self, other: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_sub(other.nanos),
        }
    }
    
    /// Multiply this duration by a scalar, saturating at the maximum value
    pub fn saturating_mul(&self, scalar: u64) -> Self {
        Self {
            nanos: self.nanos.saturating_mul(scalar),
        }
    }
    
    /// Check if this duration is within a certain range
    pub fn is_within(&self, min: Duration, max: Duration) -> bool {
        *self >= min && *self <= max
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format in the most appropriate unit
        if self.nanos == 0 {
            write!(f, "0s")
        } else if self.nanos < 1_000 {
            write!(f, "{}ns", self.nanos)
        } else if self.nanos < 1_000_000 {
            write!(f, "{:.2}μs", self.nanos as f64 / 1_000.0)
        } else if self.nanos < 1_000_000_000 {
            write!(f, "{:.2}ms", self.nanos as f64 / 1_000_000.0)
        } else if self.nanos < 60 * 1_000_000_000 {
            write!(f, "{:.2}s", self.nanos as f64 / 1_000_000_000.0)
        } else if self.nanos < 60 * 60 * 1_000_000_000 {
            write!(f, "{:.2}m", self.nanos as f64 / (60.0 * 1_000_000_000.0))
        } else {
            write!(f, "{:.2}h", self.nanos as f64 / (60.0 * 60.0 * 1_000_000_000.0))
        }
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanos.cmp(&other.nanos)
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Duration {
    type Output = Duration;
    
    fn add(self, other: Self) -> Self::Output {
        Self {
            nanos: self.nanos.checked_add(other.nanos).unwrap_or(u64::MAX),
        }
    }
}

impl Sub for Duration {
    type Output = Duration;
    
    fn sub(self, other: Self) -> Self::Output {
        Self {
            nanos: self.nanos.checked_sub(other.nanos).unwrap_or(0),
        }
    }
}

impl Mul<u64> for Duration {
    type Output = Duration;
    
    fn mul(self, scalar: u64) -> Self::Output {
        Self {
            nanos: self.nanos.checked_mul(scalar).unwrap_or(u64::MAX),
        }
    }
}

impl Div<u64> for Duration {
    type Output = Duration;
    
    fn div(self, scalar: u64) -> Self::Output {
        if scalar == 0 {
            return Self::max();
        }
        
        Self {
            nanos: self.nanos / scalar,
        }
    }
}

impl Div<Duration> for Duration {
    type Output = u64;
    
    fn div(self, other: Duration) -> Self::Output {
        if other.nanos == 0 {
            return u64::MAX;
        }
        
        self.nanos / other.nanos
    }
}

impl From<std::time::Duration> for Duration {
    fn from(duration: std::time::Duration) -> Self {
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos() as u64;
        
        let total_nanos = secs
            .checked_mul(1_000_000_000)
            .and_then(|secs_as_nanos| secs_as_nanos.checked_add(nanos))
            .unwrap_or(u64::MAX);
        
        Self { nanos: total_nanos }
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs();
        let nanos = (duration.nanos % 1_000_000_000) as u32;
        
        std::time::Duration::new(secs, nanos)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duration_creation() {
        assert_eq!(Duration::from_nanos(1).as_nanos(), 1);
        assert_eq!(Duration::from_micros(1).as_nanos(), 1_000);
        assert_eq!(Duration::from_millis(1).as_nanos(), 1_000_000);
        assert_eq!(Duration::from_secs(1).as_nanos(), 1_000_000_000);
        assert_eq!(Duration::from_minutes(1).as_nanos(), 60 * 1_000_000_000);
        assert_eq!(Duration::from_hours(1).as_nanos(), 60 * 60 * 1_000_000_000);
    }
    
    #[test]
    fn test_duration_conversion() {
        let duration = Duration::from_nanos(1_234_567_890);
        assert_eq!(duration.as_nanos(), 1_234_567_890);
        assert_eq!(duration.as_micros(), 1_234_567);
        assert_eq!(duration.as_millis(), 1_234);
        assert_eq!(duration.as_secs(), 1);
        assert_eq!(duration.as_minutes(), 0);
        assert_eq!(duration.as_hours(), 0);
        
        assert!((duration.as_secs_f64() - 1.23456789).abs() < 1e-9);
    }
    
    #[test]
    fn test_duration_constants() {
        assert_eq!(Duration::zero().as_nanos(), 0);
        assert_eq!(Duration::max().as_nanos(), u64::MAX);
        
        assert!(Duration::zero().is_zero());
        assert!(!Duration::from_secs(1).is_zero());
        
        assert!(Duration::max().is_max());
        assert!(!Duration::from_secs(1).is_max());
    }
    
    #[test]
    fn test_duration_operations() {
        let d1 = Duration::from_secs(10);
        let d2 = Duration::from_secs(5);
        
        assert_eq!((d1 + d2).as_secs(), 15);
        assert_eq!((d1 - d2).as_secs(), 5);
        
        // Subtraction saturates at zero
        assert_eq!((d2 - d1).as_nanos(), 0);
        
        // Multiplication
        assert_eq!((d1 * 2).as_secs(), 20);
        
        // Division
        assert_eq!((d1 / 2).as_secs(), 5);
        
        // Division by duration
        assert_eq!(d1 / d2, 2);
    }
    
    #[test]
    fn test_duration_comparison() {
        let d1 = Duration::from_secs(10);
        let d2 = Duration::from_secs(20);
        
        assert!(d1 < d2);
        assert!(d2 > d1);
        assert!(d1 <= d1);
        assert!(d1 >= d1);
        assert_eq!(d1, d1);
        assert_ne!(d1, d2);
    }
    
    #[test]
    fn test_duration_is_within() {
        let d = Duration::from_secs(10);
        let min = Duration::from_secs(5);
        let max = Duration::from_secs(15);
        
        assert!(d.is_within(min, max));
        assert!(d.is_within(d, d));
        assert!(min.is_within(min, max));
        assert!(max.is_within(min, max));
        
        let too_small = Duration::from_secs(4);
        let too_large = Duration::from_secs(16);
        
        assert!(!too_small.is_within(min, max));
        assert!(!too_large.is_within(min, max));
    }
    
    #[test]
    fn test_std_duration_conversion() {
        let causality_duration = Duration::from_secs(5);
        let std_duration = std::time::Duration::from(causality_duration);
        
        assert_eq!(std_duration.as_secs(), 5);
        assert_eq!(std_duration.subsec_nanos(), 0);
        
        let causality_duration2 = Duration::from(std_duration);
        assert_eq!(causality_duration, causality_duration2);
        
        let std_duration2 = std::time::Duration::new(10, 500_000_000);
        let causality_duration3 = Duration::from(std_duration2);
        
        assert_eq!(causality_duration3.as_secs(), 10);
        assert_eq!(causality_duration3.nanos % 1_000_000_000, 500_000_000);
    }
} 