// Duration implementation
//
// This module provides the core duration type for representing
// time spans across the Causality system.

use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;
use chrono::{Duration as ChronoDuration, TimeDelta as ChronoTimeDelta}; // Use ChronoDuration
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum DurationConversionError {
    #[error("Duration out of range for std::time::Duration: {0}")]
    OutOfRange(String),
}

/// Represents a duration or time difference, wrapping chrono::Duration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(pub ChronoDuration);

impl TimeDelta {
    /// Create a new time delta from a standard chrono Duration
    pub fn new(duration: ChronoDuration) -> Self {
        Self(duration)
    }

    /// Get the underlying chrono duration
    pub fn as_duration(&self) -> ChronoDuration {
        self.0
    }

    /// Get the number of nanoseconds in the duration
    pub fn as_nanos(&self) -> u64 {
        self.0.num_nanoseconds().unwrap_or(0) as u64
    }
    
    /// Get the number of seconds in the duration
    pub fn as_secs(&self) -> i64 {
        self.0.num_seconds()
    }

    /// Create from nanoseconds
    pub fn from_nanos(nanos: i64) -> Self {
        Self(ChronoDuration::nanoseconds(nanos))
    }

    /// Create from microseconds
    pub fn from_micros(micros: i64) -> Self {
        Self(ChronoDuration::microseconds(micros))
    }

    /// Create from milliseconds
    pub fn from_millis(millis: i64) -> Self {
        Self(ChronoDuration::milliseconds(millis))
    }

    /// Create from seconds
    pub fn from_secs(secs: i64) -> Self {
        Self(ChronoDuration::seconds(secs))
    }

    // Helper for max value
    pub fn max_value() -> Self {
        Self(ChronoDuration::MAX)
    }

    // Helper for zero value
    pub fn zero() -> Self {
        Self(ChronoDuration::zero())
    }

    // Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

// --- Operator Implementations for TimeDelta ---

impl Add for TimeDelta {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0) // Delegate to chrono::Duration's Add
    }
}

impl Sub for TimeDelta {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0) // Delegate to chrono::Duration's Sub
    }
}

impl Mul<i32> for TimeDelta { // Use i32 as chrono::Duration does
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self(self.0 * rhs) // Delegate to chrono::Duration's Mul<i32>
    }
}

impl Div<i32> for TimeDelta { // Use i32 as chrono::Duration does
    type Output = Self;

    fn div(self, rhs: i32) -> Self {
        Self(self.0 / rhs) // Delegate to chrono::Duration's Div<i32>
    }
}

// Division by another TimeDelta results in a ratio (f64)
impl Div<TimeDelta> for TimeDelta {
    type Output = f64;

    fn div(self, rhs: TimeDelta) -> f64 {
        if rhs.is_zero() {
            // Handle division by zero, perhaps return NaN or infinity, or panic
            // Chrono duration division returns None for division by zero
            // Returning f64::NAN seems reasonable here.
            return f64::NAN;
        }
        // Perform division on nanoseconds for precision
        (self.0.num_nanoseconds().unwrap_or(0) as f64) /
        (rhs.0.num_nanoseconds().unwrap_or(0) as f64)
    }
}


// --- Conversion Implementations ---

impl TryFrom<std::time::Duration> for TimeDelta {
    type Error = DurationConversionError;

    fn try_from(duration: std::time::Duration) -> Result<Self, Self::Error> {
        ChronoDuration::from_std(duration)
            .map(TimeDelta::new)
            .map_err(|e| DurationConversionError::OutOfRange(e.to_string()))
    }
}

impl TryFrom<TimeDelta> for std::time::Duration {
    type Error = DurationConversionError;

    fn try_from(time_delta: TimeDelta) -> Result<Self, Self::Error> {
        time_delta.0.to_std()
            .map_err(|e| DurationConversionError::OutOfRange(e.to_string()))
    }
}


// --- Default Implementation ---

impl Default for TimeDelta {
    fn default() -> Self {
        Self(ChronoDuration::zero())
    }
}

// --- Display Implementation ---
impl fmt::Display for TimeDelta {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         // Simple display using chrono's Debug format for now
         // Can be customized later
         write!(f, "{:?}", self.0)
     }
}


// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto; // Import TryInto

    #[test]
    fn test_timedelta_creation() {
        assert_eq!(TimeDelta::from_nanos(1).0, ChronoDuration::nanoseconds(1));
        assert_eq!(TimeDelta::from_micros(1).0, ChronoDuration::microseconds(1));
        assert_eq!(TimeDelta::from_millis(1).0, ChronoDuration::milliseconds(1));
        assert_eq!(TimeDelta::from_secs(1).0, ChronoDuration::seconds(1));
        assert_eq!(TimeDelta::new(ChronoDuration::minutes(1)).0, ChronoDuration::minutes(1));
        assert_eq!(TimeDelta::new(ChronoDuration::hours(1)).0, ChronoDuration::hours(1));
    }

    #[test]
    fn test_timedelta_conversion() {
        let duration = TimeDelta::from_nanos(1_234_567_890);
        assert_eq!(duration.0.num_nanoseconds(), Some(1_234_567_890));
        assert_eq!(duration.0.num_microseconds(), Some(1_234_567));
        assert_eq!(duration.0.num_milliseconds(), 1_234);
        assert_eq!(duration.0.num_seconds(), 1);
        assert_eq!(duration.0.num_minutes(), 0);
        assert_eq!(duration.0.num_hours(), 0);

        let secs_f64 = duration.0.num_nanoseconds().unwrap_or(0) as f64 / 1_000_000_000.0;
        assert!((secs_f64 - 1.23456789).abs() < 1e-9);
    }

    #[test]
    fn test_timedelta_constants() {
        assert_eq!(TimeDelta::zero().0, ChronoDuration::zero());
        assert_eq!(TimeDelta::default().0, ChronoDuration::zero());
        assert_eq!(TimeDelta::max_value().0, ChronoDuration::MAX);

        assert!(TimeDelta::zero().is_zero());
        assert!(!TimeDelta::from_secs(1).is_zero());

        assert!(TimeDelta::max_value().0 == ChronoDuration::MAX);
        assert!(!(TimeDelta::from_secs(1).0 == ChronoDuration::MAX));
    }

    #[test]
    fn test_timedelta_operations() {
        let d1 = TimeDelta::from_secs(10);
        let d2 = TimeDelta::from_secs(5);

        assert_eq!((d1 + d2), TimeDelta::from_secs(15));
        assert_eq!((d1 - d2), TimeDelta::from_secs(5));

        // Subtraction result depends on chrono's behavior (likely wraps or panics on underflow in debug)
        // Let's test subtraction that doesn't underflow
        assert_eq!((d2 - TimeDelta::from_secs(2)), TimeDelta::from_secs(3));

        // Multiplication
        assert_eq!((d1 * 2), TimeDelta::from_secs(20));

        // Division by scalar
        assert_eq!((d1 / 2), TimeDelta::from_secs(5));

        // Division by TimeDelta
        assert!(((d1 / d2) - 2.0).abs() < f64::EPSILON);
        assert!(((d2 / d1) - 0.5).abs() < f64::EPSILON);
        assert!((d1 / TimeDelta::zero()).is_nan()); // Check division by zero
    }

    #[test]
    fn test_timedelta_comparison() {
        let d1 = TimeDelta::from_secs(10);
        let d2 = TimeDelta::from_secs(20);

        assert!(d1 < d2);
        assert!(d2 > d1);
        assert!(d1 <= d1);
        assert!(d1 >= d1);
        assert_eq!(d1, d1);
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_duration_is_within() { // Renamed test, logic uses comparisons now
        let d = TimeDelta::from_secs(10);
        let min = TimeDelta::from_secs(5);
        let max = TimeDelta::from_secs(15);

        assert!(d >= min && d <= max);
        assert!(d >= d && d <= d);
        assert!(min >= min && min <= max);
        assert!(max >= min && max <= max);

        let too_small = TimeDelta::from_secs(4);
        let too_large = TimeDelta::from_secs(16);

        assert!(!(too_small >= min && too_small <= max));
        assert!(!(too_large >= min && too_large <= max));
    }

    #[test]
    fn test_std_duration_conversion() {
        let causality_duration = TimeDelta::from_secs(5);
        let std_duration: std::time::Duration = causality_duration.try_into().expect("Conversion failed");

        assert_eq!(std_duration.as_secs(), 5);
        assert_eq!(std_duration.subsec_nanos(), 0);

        let causality_duration2: TimeDelta = std_duration.try_into().expect("Conversion failed");
        assert_eq!(causality_duration, causality_duration2);

        let std_duration2 = std::time::Duration::new(10, 500_000_000);
        let causality_duration3: TimeDelta = std_duration2.try_into().expect("Conversion failed");

        assert_eq!(causality_duration3.0.num_seconds(), 10);
        assert_eq!(causality_duration3.0.num_milliseconds(), 10_500); // Check millis directly
    }
} 