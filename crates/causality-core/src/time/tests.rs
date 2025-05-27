//! Tests for time-related functionality

use super::{Clock, FixedClock, MonotonicClock, ThreadLocalClock, Timestamp, TimeDelta};
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_timestamp_creation() {
    let ts1 = Timestamp::from_secs(1600000000);
    let ts2 = Timestamp::from_millis(1600000000 * 1000);
    let ts3 = Timestamp::from_micros(1600000000 * 1000 * 1000);
    let ts4 = Timestamp::from_nanos(1600000000 * 1000 * 1000 * 1000);
    
    assert_eq!(ts1, ts2);
    assert_eq!(ts2, ts3);
    assert_eq!(ts3, ts4);
    assert_eq!(ts1.as_secs(), 1600000000);
    assert_eq!(ts2.as_millis(), 1600000000 * 1000);
    assert_eq!(ts3.as_micros(), 1600000000 * 1000 * 1000);
    assert_eq!(ts4.as_nanos(), 1600000000 * 1000 * 1000 * 1000);
}

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
fn test_thread_local_clock() -> Result<(), super::error::TimeError> {
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
fn test_timestamp_next() {
    let ts = Timestamp::from_secs(1600000000);
    let ts_next = ts.next().unwrap();
    
    assert!(ts_next > ts);
    assert_eq!(ts_next.as_nanos(), ts.as_nanos() + 1);
}

#[test]
fn test_timestamp_add_std_duration() {
    let ts = Timestamp::from_secs(1600000000);
    let duration = Duration::from_secs(60);
    
    let ts_plus = ts.add_std_duration(duration).unwrap();
    
    assert_eq!(ts_plus.as_secs(), ts.as_secs() + 60);
}
