//! Tests for integration between LogSegmentManager and ReplayEngine

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use tempfile::tempdir;

use crate::error::Result;
use crate::log::entry::{EntryData, EntryType, EventEntry, EventSeverity, LogEntry};
use crate::log::replay::{
    NoopReplayCallback, ReplayEngine, ReplayOptions, ReplayStatus,
};
use crate::log::segment_manager::{LogSegmentManager, RotationCriteria, SegmentManagerOptions};
use crate::log::storage::{StorageConfig, LogStorage};
use crate::log::storage::memory_storage::MemoryLogStorage;

#[test]
fn test_replay_with_segment_manager() -> Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let base_dir = temp_dir.path().to_path_buf();
    let index_dir = base_dir.join("index");
    
    // Create segment manager options
    let options = SegmentManagerOptions {
        base_dir,
        max_active_segments: 5,
        compress_inactive: false,
        rotation_criteria: vec![
            RotationCriteria::EntryCount(10), // Small value for testing
        ],
        segment_name_pattern: "segment_{timestamp}".to_string(),
        auto_flush: true,
        index_dir: Some(index_dir),
    };
    
    // Create storage config
    let storage_config = StorageConfig {
        verify_hashes: false,
        ..Default::default()
    };
    
    // Create a segment manager
    let segment_manager = Arc::new(LogSegmentManager::new(options, storage_config.clone())?);
    
    // Create test entries spanning multiple time ranges
    create_test_entries(segment_manager.clone())?;
    
    // Create a storage reference for the replay engine
    let storage = Arc::new(MemoryLogStorage::new_with_config(storage_config));
    
    // Create replay options with a time range filter
    let now = Utc::now();
    let start_time = (now - Duration::hours(1)).timestamp() as u64;
    let end_time = now.timestamp() as u64;
    
    let replay_options = ReplayOptions {
        start_time: Some(start_time),
        end_time: Some(end_time),
        ..Default::default()
    };
    
    // Create a replay engine with the segment manager
    let engine = ReplayEngine::with_segment_manager(
        storage,
        replay_options,
        Arc::new(NoopReplayCallback),
        segment_manager.clone(),
    );
    
    // Run the replay
    let result = engine.run()?;
    
    // Verify the result
    assert_eq!(result.status, ReplayStatus::Complete);
    assert!(result.processed_entries > 0);
    assert!(result.error.is_none());
    
    // The state should contain events from the specified time range
    if let Some(state) = result.state {
        // Verify that we have events in the state
        let events = state.events();
        assert!(!events.is_empty());
        
        // All events should be within the time range
        for event in events {
            assert!(event.timestamp >= start_time);
            assert!(event.timestamp <= end_time);
        }
    } else {
        panic!("Replay state is missing");
    }
    
    Ok(())
}

/// Helper function to create test entries across multiple segments
fn create_test_entries(segment_manager: Arc<LogSegmentManager>) -> Result<()> {
    let now = Utc::now();
    
    // Create entries spanning different time ranges
    let time_ranges = vec![
        (now - Duration::hours(3), now - Duration::hours(2)), // 3-2 hours ago
        (now - Duration::hours(2), now - Duration::hours(1)), // 2-1 hours ago
        (now - Duration::hours(1), now),                      // Last hour
    ];
    
    let mut entry_count = 0;
    
    for (range_start, range_end) in time_ranges {
        // Create 15 entries for each time range (should span multiple segments)
        for i in 0..15 {
            // Calculate timestamp within the range
            let progress = i as f32 / 15.0;
            let time_diff = range_end.timestamp() - range_start.timestamp();
            let timestamp = range_start.timestamp() + (time_diff as f32 * progress) as i64;
            
            // Create a log entry
            let entry = LogEntry {
                id: format!("entry_{}", entry_count),
                entry_type: EntryType::Event,
                timestamp: timestamp as u64,
                data: EntryData::Event(EventEntry {
                    event_name: format!("test_event_{}", i),
                    severity: EventSeverity::Info,
                    component: "test_component".to_string(),
                    details: serde_json::json!({
                        "time_range": format!("{}-{}", range_start, range_end),
                        "index": i
                    }),
                    resources: None,
                    domains: None,
                }),
                trace_id: Some(format!("trace_{}", i % 3)),
                parent_id: None,
                metadata: HashMap::new(),
                entry_hash: None,
            };
            
            // Append to the segment manager
            segment_manager.append(entry)?;
            entry_count += 1;
            
            // Small delay to ensure unique timestamps
            std::thread::sleep(StdDuration::from_millis(5));
        }
    }
    
    // Flush to ensure all data is written
    segment_manager.flush()?;
    
    Ok(())
}

#[test]
fn test_replay_filtered_segments() -> Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create segment manager with time-based rotation
    let options = SegmentManagerOptions {
        base_dir,
        max_active_segments: 3,
        compress_inactive: false,
        rotation_criteria: vec![
            RotationCriteria::TimeInterval(Duration::minutes(30)),
        ],
        segment_name_pattern: "segment_{timestamp}".to_string(),
        auto_flush: true,
        index_dir: None,
    };
    
    // Create storage config
    let storage_config = StorageConfig::default();
    
    // Create a segment manager
    let segment_manager = Arc::new(LogSegmentManager::new(options, storage_config.clone())?);
    
    // Create entries with specific domains and resources
    let domains = vec!["domain1", "domain2", "domain3"];
    let resources = vec!["resource1", "resource2", "resource3"];
    
    // Add test entries
    for i in 0..30 {
        // Select domain and resource
        let domain_idx = i % domains.len();
        let resource_idx = (i / 10) % resources.len();
        
        let entry = LogEntry {
            id: format!("entry_{}", i),
            entry_type: EntryType::Event,
            timestamp: (Utc::now() - Duration::minutes((30 - i) as i64)).timestamp() as u64,
            data: EntryData::Event(EventEntry {
                event_name: format!("test_event_{}", i),
                severity: EventSeverity::Info,
                component: "test_component".to_string(),
                details: serde_json::json!({"index": i}),
                resources: Some(vec![resources[resource_idx].to_string()]),
                domains: Some(vec![domains[domain_idx].to_string()]),
            }),
            trace_id: Some("trace_id".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        segment_manager.append(entry)?;
        
        // Small delay to ensure unique timestamps
        std::thread::sleep(StdDuration::from_millis(5));
    }
    
    // Flush to ensure all data is written
    segment_manager.flush()?;
    
    // Create a storage reference
    let storage = Arc::new(MemoryLogStorage::new());
    
    // Create replay options to filter by domain and resource
    let replay_options = ReplayOptions {
        domains: Some(vec!["domain1".to_string()]),
        resources: Some(vec!["resource2".to_string()]),
        ..Default::default()
    };
    
    // Create a replay engine
    let engine = ReplayEngine::with_segment_manager(
        storage,
        replay_options,
        Arc::new(NoopReplayCallback),
        segment_manager.clone(),
    );
    
    // Run the replay
    let result = engine.run()?;
    
    // Verify the result
    assert_eq!(result.status, ReplayStatus::Complete);
    assert!(result.error.is_none());
    
    // Check that we only got entries for domain1 and resource2
    if let Some(state) = result.state {
        for event in state.events() {
            if let EntryData::Event(e) = &event.data {
                // Event should be from domain1
                if let Some(domains) = &e.domains {
                    assert!(domains.contains(&"domain1".to_string()));
                }
                
                // Event should be from resource2
                if let Some(resources) = &e.resources {
                    assert!(resources.contains(&"resource2".to_string()));
                }
            }
        }
    } else {
        panic!("Replay state is missing");
    }
    
    Ok(())
} 