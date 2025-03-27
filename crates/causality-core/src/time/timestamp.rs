// Timestamp implementation
//
// This module provides the core timestamp type for representing
// points in time across the Causality system.

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::ops::{Add, Sub};

use super::duration::Duration;

/// A timestamp representing a point in time
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Timestamp {
    /// The number of nanoseconds since the epoch
    nanos: u64,
}

impl Timestamp {
    /// Create a new timestamp from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
    
    /// Create a new timestamp from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros * 1_000,
        }
    }
    
    /// Create a new timestamp from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            nanos: millis * 1_000_000,
        }
    }
    
    /// Create a new timestamp from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            nanos: secs * 1_000_000_000,
        }
    }
    
    /// Get the number of nanoseconds since the epoch
    pub fn as_nanos(&self) -> u64 {
        self.nanos
    }
    
    /// Get the number of microseconds since the epoch
    pub fn as_micros(&self) -> u64 {
        self.nanos / 1_000
    }
    
    /// Get the number of milliseconds since the epoch
    pub fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }
    
    /// Get the number of seconds since the epoch
    pub fn as_secs(&self) -> u64 {
        self.nanos / 1_000_000_000
    }
    
    /// Get the timestamp corresponding to zero (epoch)
    pub const fn zero() -> Self {
        Self { nanos: 0 }
    }
    
    /// Get the maximum possible timestamp
    pub const fn max() -> Self {
        Self { nanos: u64::MAX }
    }
    
    /// Check if this timestamp is zero
    pub fn is_zero(&self) -> bool {
        self.nanos == 0
    }
    
    /// Check if this timestamp is the maximum value
    pub fn is_max(&self) -> bool {
        self.nanos == u64::MAX
    }
    
    /// Add a duration to this timestamp, saturating at the maximum value
    pub fn saturating_add(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_add(duration.as_nanos()),
        }
    }
    
    /// Subtract a duration from this timestamp, saturating at zero
    pub fn saturating_sub(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_sub(duration.as_nanos()),
        }
    }
    
    /// Check if this timestamp is between two other timestamps (inclusive)
    pub fn is_between(&self, start: Timestamp, end: Timestamp) -> bool {
        *self >= start && *self <= end
    }
    
    /// Create a time range from this timestamp to another
    pub fn range_to(&self, end: Timestamp) -> TimeRange {
        TimeRange::new(*self, end)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format as seconds.nanoseconds
        let secs = self.as_secs();
        let nanos = self.nanos % 1_000_000_000;
        
        if nanos == 0 {
            write!(f, "{}s", secs)
        } else if nanos % 1_000_000 == 0 {
            write!(f, "{}.{:03}s", secs, nanos / 1_000_000)
        } else if nanos % 1_000 == 0 {
            write!(f, "{}.{:06}s", secs, nanos / 1_000)
        } else {
            write!(f, "{}.{:09}s", secs, nanos)
        }
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanos.cmp(&other.nanos)
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;
    
    fn add(self, duration: Duration) -> Self::Output {
        Self {
            nanos: self.nanos.checked_add(duration.as_nanos()).unwrap_or(u64::MAX),
        }
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Timestamp;
    
    fn sub(self, duration: Duration) -> Self::Output {
        Self {
            nanos: self.nanos.checked_sub(duration.as_nanos()).unwrap_or(0),
        }
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Duration;
    
    fn sub(self, other: Timestamp) -> Self::Output {
        if self >= other {
            Duration::from_nanos(self.nanos - other.nanos)
        } else {
            Duration::zero() // Cannot have negative durations, so return zero
        }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::zero()
    }
}

/// A range of time between two timestamps
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TimeRange {
    /// The start of the range (inclusive)
    start: Timestamp,
    
    /// The end of the range (inclusive)
    end: Timestamp,
}

impl TimeRange {
    /// Create a new time range from start to end (inclusive)
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        // Ensure start <= end
        if start > end {
            Self { start: end, end: start }
        } else {
            Self { start, end }
        }
    }
    
    /// Create a time range spanning from time for the specified duration
    pub fn from_duration(time: Timestamp, duration: Duration) -> Self {
        let end = time + duration;
        Self::new(time, end)
    }
    
    /// Create a time range centered around a timestamp with the specified duration
    pub fn around(time: Timestamp, duration: Duration) -> Self {
        let half_duration = Duration::from_nanos(duration.as_nanos() / 2);
        let start = time - half_duration;
        let end = time + half_duration;
        Self::new(start, end)
    }
    
    /// Get the start of the range
    pub fn start(&self) -> Timestamp {
        self.start
    }
    
    /// Get the end of the range
    pub fn end(&self) -> Timestamp {
        self.end
    }
    
    /// Get the duration of the range
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
    
    /// Check if the range is empty (zero duration)
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// Check if the range contains a timestamp
    pub fn contains(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
    
    /// Check if this range overlaps with another range
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }
    
    /// Get the intersection of this range with another range
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        
        let start = std::cmp::max(self.start, other.start);
        let end = std::cmp::min(self.end, other.end);
        
        Some(Self::new(start, end))
    }
    
    /// Get the union of this range with another range
    /// Only works if the ranges overlap or are adjacent
    pub fn union(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) && !self.is_adjacent(other) {
            return None;
        }
        
        let start = std::cmp::min(self.start, other.start);
        let end = std::cmp::max(self.end, other.end);
        
        Some(Self::new(start, end))
    }
    
    /// Check if this range is adjacent to another range
    /// (i.e., they share an endpoint)
    pub fn is_adjacent(&self, other: &Self) -> bool {
        self.end == other.start || self.start == other.end
    }
    
    /// Extend this range by a duration on both ends
    pub fn extend(&self, duration: Duration) -> Self {
        Self::new(
            self.start - duration,
            self.end + duration,
        )
    }
    
    /// Create an iterator over timestamps in this range with the given step
    pub fn iter(&self, step: Duration) -> TimeRangeIterator {
        TimeRangeIterator {
            current: self.start,
            end: self.end,
            step: step.as_nanos(),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} to {}]", self.start, self.end)
    }
}

/// An iterator over timestamps in a time range
#[derive(Debug, Clone)]
pub struct TimeRangeIterator {
    /// The current timestamp
    current: Timestamp,
    
    /// The end timestamp
    end: Timestamp,
    
    /// The step size in nanoseconds
    step: u64,
}

impl Iterator for TimeRangeIterator {
    type Item = Timestamp;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }
        
        let result = self.current;
        
        // Update for next iteration
        let next_nanos = self.current.as_nanos().saturating_add(self.step);
        self.current = Timestamp::from_nanos(next_nanos);
        
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timestamp_creation() {
        assert_eq!(Timestamp::from_nanos(1).as_nanos(), 1);
        assert_eq!(Timestamp::from_micros(1).as_nanos(), 1_000);
        assert_eq!(Timestamp::from_millis(1).as_nanos(), 1_000_000);
        assert_eq!(Timestamp::from_secs(1).as_nanos(), 1_000_000_000);
    }
    
    #[test]
    fn test_timestamp_conversion() {
        let ts = Timestamp::from_nanos(1_234_567_890);
        assert_eq!(ts.as_nanos(), 1_234_567_890);
        assert_eq!(ts.as_micros(), 1_234_567);
        assert_eq!(ts.as_millis(), 1_234);
        assert_eq!(ts.as_secs(), 1);
    }
    
    #[test]
    fn test_timestamp_constants() {
        assert_eq!(Timestamp::zero().as_nanos(), 0);
        assert_eq!(Timestamp::max().as_nanos(), u64::MAX);
        
        assert!(Timestamp::zero().is_zero());
        assert!(!Timestamp::from_secs(1).is_zero());
        
        assert!(Timestamp::max().is_max());
        assert!(!Timestamp::from_secs(1).is_max());
    }
    
    #[test]
    fn test_timestamp_operations() {
        let ts = Timestamp::from_secs(10);
        let duration = Duration::from_secs(5);
        
        assert_eq!((ts + duration).as_secs(), 15);
        assert_eq!((ts - duration).as_secs(), 5);
        
        // Test timestamp subtraction
        let ts1 = Timestamp::from_secs(10);
        let ts2 = Timestamp::from_secs(5);
        assert_eq!((ts1 - ts2).as_secs(), 5);
        
        // Subtracting a larger timestamp gives zero duration
        assert_eq!((ts2 - ts1).as_nanos(), 0);
    }
    
    #[test]
    fn test_timestamp_comparison() {
        let ts1 = Timestamp::from_secs(10);
        let ts2 = Timestamp::from_secs(20);
        
        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
        assert!(ts1 <= ts1);
        assert!(ts1 >= ts1);
        assert_eq!(ts1, ts1);
        assert_ne!(ts1, ts2);
    }
    
    #[test]
    fn test_time_range() {
        let start = Timestamp::from_secs(10);
        let end = Timestamp::from_secs(20);
        let range = TimeRange::new(start, end);
        
        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
        assert_eq!(range.duration().as_secs(), 10);
        
        // If start > end, they should be swapped
        let range2 = TimeRange::new(end, start);
        assert_eq!(range, range2);
    }
    
    #[test]
    fn test_time_range_from_duration() {
        let time = Timestamp::from_secs(10);
        let duration = Duration::from_secs(5);
        let range = TimeRange::from_duration(time, duration);
        
        assert_eq!(range.start(), time);
        assert_eq!(range.end(), Timestamp::from_secs(15));
        assert_eq!(range.duration(), duration);
    }
    
    #[test]
    fn test_time_range_around() {
        let time = Timestamp::from_secs(10);
        let duration = Duration::from_secs(6);
        let range = TimeRange::around(time, duration);
        
        // 10 - 3 = 7, 10 + 3 = 13
        assert_eq!(range.start(), Timestamp::from_secs(7));
        assert_eq!(range.end(), Timestamp::from_secs(13));
        assert_eq!(range.duration().as_secs(), 6);
    }
    
    #[test]
    fn test_time_range_contains() {
        let range = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        assert!(range.contains(Timestamp::from_secs(10)));
        assert!(range.contains(Timestamp::from_secs(15)));
        assert!(range.contains(Timestamp::from_secs(20)));
        
        assert!(!range.contains(Timestamp::from_secs(5)));
        assert!(!range.contains(Timestamp::from_secs(25)));
    }
    
    #[test]
    fn test_time_range_overlaps() {
        let range1 = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        let range2 = TimeRange::new(
            Timestamp::from_secs(15),
            Timestamp::from_secs(25),
        );
        
        let range3 = TimeRange::new(
            Timestamp::from_secs(5),
            Timestamp::from_secs(15),
        );
        
        let range4 = TimeRange::new(
            Timestamp::from_secs(0),
            Timestamp::from_secs(5),
        );
        
        let range5 = TimeRange::new(
            Timestamp::from_secs(25),
            Timestamp::from_secs(30),
        );
        
        assert!(range1.overlaps(&range1)); // Self-overlap
        assert!(range1.overlaps(&range2)); // Partial overlap
        assert!(range1.overlaps(&range3)); // Partial overlap
        assert!(!range1.overlaps(&range4)); // No overlap
        assert!(!range1.overlaps(&range5)); // No overlap
    }
    
    #[test]
    fn test_time_range_intersection() {
        let range1 = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        let range2 = TimeRange::new(
            Timestamp::from_secs(15),
            Timestamp::from_secs(25),
        );
        
        let intersection = range1.intersection(&range2).unwrap();
        assert_eq!(intersection.start(), Timestamp::from_secs(15));
        assert_eq!(intersection.end(), Timestamp::from_secs(20));
        
        // No intersection
        let range3 = TimeRange::new(
            Timestamp::from_secs(0),
            Timestamp::from_secs(5),
        );
        
        assert!(range1.intersection(&range3).is_none());
    }
    
    #[test]
    fn test_time_range_union() {
        let range1 = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        let range2 = TimeRange::new(
            Timestamp::from_secs(15),
            Timestamp::from_secs(25),
        );
        
        let union = range1.union(&range2).unwrap();
        assert_eq!(union.start(), Timestamp::from_secs(10));
        assert_eq!(union.end(), Timestamp::from_secs(25));
        
        // Adjacent ranges can be unioned
        let range3 = TimeRange::new(
            Timestamp::from_secs(20),
            Timestamp::from_secs(30),
        );
        
        let union = range1.union(&range3).unwrap();
        assert_eq!(union.start(), Timestamp::from_secs(10));
        assert_eq!(union.end(), Timestamp::from_secs(30));
        
        // Non-overlapping and non-adjacent ranges cannot be unioned
        let range4 = TimeRange::new(
            Timestamp::from_secs(0),
            Timestamp::from_secs(5),
        );
        
        assert!(range1.union(&range4).is_none());
    }
    
    #[test]
    fn test_time_range_is_adjacent() {
        let range1 = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        let range2 = TimeRange::new(
            Timestamp::from_secs(20),
            Timestamp::from_secs(30),
        );
        
        let range3 = TimeRange::new(
            Timestamp::from_secs(0),
            Timestamp::from_secs(10),
        );
        
        let range4 = TimeRange::new(
            Timestamp::from_secs(0),
            Timestamp::from_secs(5),
        );
        
        assert!(range1.is_adjacent(&range2));
        assert!(range1.is_adjacent(&range3));
        assert!(!range1.is_adjacent(&range4));
    }
    
    #[test]
    fn test_time_range_extend() {
        let range = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(20),
        );
        
        let extended = range.extend(Duration::from_secs(5));
        assert_eq!(extended.start(), Timestamp::from_secs(5));
        assert_eq!(extended.end(), Timestamp::from_secs(25));
    }
    
    #[test]
    fn test_time_range_iterator() {
        let range = TimeRange::new(
            Timestamp::from_secs(10),
            Timestamp::from_secs(15),
        );
        
        let timestamps: Vec<_> = range.iter(Duration::from_secs(2)).collect();
        
        assert_eq!(timestamps.len(), 3);
        assert_eq!(timestamps[0], Timestamp::from_secs(10));
        assert_eq!(timestamps[1], Timestamp::from_secs(12));
        assert_eq!(timestamps[2], Timestamp::from_secs(14));
    }
} 