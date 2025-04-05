use causality_core::time::{TimeDelta, TimeMap, ManualClock, Clock, ClockSource};
use std::time::{Duration, Instant};

fn main() {
    println!("Testing time module...");
    
    // Test ManualClock
    let mut clock = ManualClock::new(100);
    println!("Initial clock time: {}", clock.now());
    assert_eq!(clock.now(), 100);
    
    clock.advance(50);
    println!("Advanced clock time: {}", clock.now());
    assert_eq!(clock.now(), 150);
    
    // Test TimeMap
    let mut time_map = TimeMap::new();
    let pos1 = time_map.update_position("test1", 100);
    println!("Position 1: {:?}", pos1);
    
    let pos2 = time_map.update_position("test2", 150);
    println!("Position 2: {:?}", pos2);
    
    let snapshot = time_map.snapshot();
    println!("TimeMap snapshot: {:?}", snapshot);
    
    // Test causality
    let causes = time_map.get_causes("test2");
    println!("Causes of test2: {:?}", causes);
    
    println!("All tests passed!");
}

fn test_timestamp_operations() {
    println!("Testing timestamp operations...");
    let now = Instant::now();
    let later = now + Duration::from_secs(1);
    
    assert!(later > now);
    assert!(now < later);
    
    let diff = later - now;
    assert_eq!(diff.as_secs(), 1);
    
    println!("✓ Timestamp operations test passed");
}

fn test_duration_operations() {
    println!("Testing duration operations...");
    let duration1 = Duration::from_secs(1);
    let duration2 = Duration::from_secs(2);
    
    let sum = duration1 + duration2;
    assert_eq!(sum.as_secs(), 3);
    
    let diff = duration2 - duration1;
    assert_eq!(diff.as_secs(), 1);
    
    let doubled = duration1 * 2;
    assert_eq!(doubled.as_secs(), 2);
    
    let halved = duration2 / 2;
    assert_eq!(halved.as_secs(), 1);
    
    println!("✓ Duration operations test passed");
}

fn test_system_timer() {
    println!("Testing system timer...");
    let start = Instant::now();
    // Simulate some work
    for _ in 0..1000000 {
        let _ = 1 + 1;
    }
    let elapsed = start.elapsed();
    
    // Just ensure some time has passed
    assert!(elapsed.as_nanos() > 0);
    
    println!("✓ System timer test passed");
}

fn test_manual_clock() {
    println!("Testing manual clock...");
    let mut clock = ManualClock::new(10);
    
    // Test initial time
    assert_eq!(clock.now(), 10);
    
    // Test advancing time
    clock.advance(5);
    assert_eq!(clock.now(), 15);
    
    println!("✓ Manual clock test passed");
}

fn test_time_map() {
    println!("Testing time map...");
    let mut map = TimeMap::new();
    
    // Test updating positions
    map.update_position("A", 10);
    map.update_position("B", 5);
    
    // Test getting positions
    assert_eq!(map.get_position("A"), Some(10));
    assert_eq!(map.get_position("B"), Some(5));
    assert_eq!(map.get_position("C"), None);
    
    // Test snapshot validity
    let snapshot = map.snapshot();
    assert!(map.validate_snapshot(&snapshot));
    
    // Test causality relationship
    assert!(map.is_causally_after("A", "B"));
    assert!(!map.is_causally_after("B", "A"));
    
    println!("✓ Time map test passed");
}

fn test_time_delta() {
    println!("Testing time delta conversion...");
    let delta = TimeDelta::from_nanos(1000);
    let duration = Duration::from_nanos(1000);
    
    // Test conversion to duration
    let converted = delta.to_duration();
    assert_eq!(converted, duration);
    
    // Test creation from duration
    let delta_from_duration = TimeDelta::from_duration(duration);
    assert_eq!(delta, delta_from_duration);
    
    println!("✓ Time delta conversion test passed");
}

fn test_helper_functions() {
    println!("Testing helper functions...");
    
    // Create a time map with some positions
    let mut map = TimeMap::new();
    map.update_position("A", 10);
    map.update_position("B", 20);
    
    // Test that positions are correctly stored
    assert_eq!(map.get_position("A"), Some(10));
    assert_eq!(map.get_position("B"), Some(20));
    
    // Test that non-existent positions return None
    assert_eq!(map.get_position("C"), None);
    
    println!("✓ Helper functions test passed");
} 