// Tests for the time module
use std::thread::sleep;
use std::time::Duration as StdDuration;

use super::{TimeDelta, Timer, Duration, Timestamp};
use super::clock::{Clock, ManualClock, SystemClock};
use super::map::TimeMap;

#[test]
fn test_timestamp_operations() {
    let ts1 = Timestamp::from_nanos(1000);
    let ts2 = Timestamp::from_nanos(2000);
    
    // Addition with Duration
    let sum = ts1 + Duration::from_nanos(2000);
    assert_eq!(sum, Timestamp::from_nanos(3000));
    
    // Subtraction returns Duration
    let diff = ts2 - ts1;
    assert_eq!(diff, Duration::from_nanos(1000));
    
    assert!(ts2 > ts1);
    assert!(ts1 < ts2);
}

#[test]
fn test_duration_operations() {
    let d1 = Duration::from_nanos(1000);
    let d2 = Duration::from_micros(2);
    
    assert_eq!(d2, Duration::from_nanos(2000));
    assert_eq!(d1 + d2, Duration::from_nanos(3000));
    assert_eq!(d2 - d1, Duration::from_nanos(1000));
    assert!(d2 > d1);
    assert!(d1 < d2);
    
    assert_eq!(d1 * 2, Duration::from_nanos(2000));
    assert_eq!(d2 / 2, Duration::from_nanos(1000));
}

#[test]
fn test_system_timer() {
    let timer = Timer::new();
    sleep(StdDuration::from_millis(10));
    let elapsed = timer.elapsed();
    
    // Sleep should have caused at least some time to elapse
    assert!(elapsed > Duration::from_nanos(0));
    
    // Test that start_time returns the correct timestamp
    assert!(timer.start_time() <= SystemClock::now());
}

#[test]
fn test_manual_clock() {
    let mut clock = ManualClock::new(Timestamp::from_nanos(1000));
    assert_eq!(clock.now(), Timestamp::from_nanos(1000));
    
    // Test advancing the clock
    clock.advance(Duration::from_nanos(500));
    assert_eq!(clock.now(), Timestamp::from_nanos(1500));
}

#[test]
fn test_time_map() {
    let mut map = TimeMap::new();
    
    // Add some domains
    map.update_position("domain1", 1000);
    map.update_position("domain2", 2000);
    
    // Check retrieval
    let pos1 = map.get_position("domain1").expect("Domain should exist");
    let pos2 = map.get_position("domain2").expect("Domain should exist");
    
    assert_eq!(pos1.get_timestamp(), 1000);
    assert_eq!(pos2.get_timestamp(), 2000);
    
    // Update positions
    map.update_position("domain1", 1500);
    let pos1_updated = map.get_position("domain1").expect("Domain should exist");
    assert_eq!(pos1_updated.get_timestamp(), 1500);
    
    // Create a snapshot
    let snapshot = map.snapshot();
    
    // Test snapshot validity
    assert!(snapshot.is_valid_at(&map));
    
    // Make snapshot invalid
    map.update_position("domain1", 900); // This breaks causality
    assert!(!snapshot.is_valid_at(&map));
}

#[test]
fn test_time_delta() {
    let delta = TimeDelta::from_nanos(1000);
    let delta2 = TimeDelta::from_micros(1);
    
    assert_eq!(delta.as_duration(), Duration::from_nanos(1000));
    assert_eq!(delta2.as_duration(), Duration::from_nanos(1000));
    
    let delta3 = TimeDelta::new(Duration::from_millis(1));
    assert_eq!(delta3.as_duration(), Duration::from_nanos(1_000_000));
}

#[test]
fn test_helper_functions() {
    // Test now() helper
    let now = super::now();
    assert!(now > Timestamp::from_nanos(0));
    
    // Test deadline() helper
    let deadline = super::deadline(Duration::from_millis(100));
    assert!(deadline > now);
    
    // Test create_time_map() helper
    let map = super::create_time_map();
    assert_eq!(map.get_position("test"), None);
    
    // Test is_snapshot_valid_at() helper
    let mut map = TimeMap::new();
    map.update_position("domain1", 1000);
    let snapshot = map.snapshot();
    assert!(super::is_snapshot_valid_at(&snapshot, &map));
} 