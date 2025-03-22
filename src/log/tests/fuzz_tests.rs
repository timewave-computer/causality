//! Fuzz testing for the Causality log system
//!
//! This module contains fuzz tests that help identify edge cases and potential issues.
//! These tests use randomized inputs and boundary values to stress test the log system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::distributions::{Alphanumeric, Standard, Uniform};

use crate::error::Result;
use crate::log::entry::{
    LogEntry, EntryType, EntryData, EventEntry, EventSeverity, 
    EffectEntry, FactEntry, generate_entry_hash, verify_entry_hash
};
use crate::log::storage::{LogStorage, MemoryLogStorage, StorageConfig, FileLogStorage};
use crate::log::replay::{ReplayEngine, ReplayOptions, ReplayStatus, StatsCallback};
use crate::log::segment_manager::{LogSegmentManager, RotationCriteria, SegmentManagerOptions};
use crate::types::{ResourceId, DomainId};
use crate::effect::types::EffectType;

/// Generate a random string of the given length
fn random_string(rng: &mut StdRng, len: usize) -> String {
    (0..len)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

/// Generate a random timestamp (between 2020 and 2030)
fn random_timestamp(rng: &mut StdRng) -> u64 {
    rng.gen_range(1577836800u64..1893456000u64)
}

/// Generate a random log entry
fn generate_random_entry(rng: &mut StdRng, entry_id: usize) -> LogEntry {
    let entry_type = match rng.gen_range(0..3) {
        0 => EntryType::Event,
        1 => EntryType::Effect,
        _ => EntryType::Fact,
    };
    
    let data = match entry_type {
        EntryType::Event => {
            EntryData::Event(EventEntry {
                event_name: format!("event_{}", random_string(rng, 8)),
                severity: match rng.gen_range(0..5) {
                    0 => EventSeverity::Debug,
                    1 => EventSeverity::Info,
                    2 => EventSeverity::Warning,
                    3 => EventSeverity::Error,
                    _ => EventSeverity::Critical,
                },
                component: random_string(rng, 5),
                details: serde_json::json!({
                    "value": random_string(rng, 10),
                    "counter": rng.gen::<u32>(),
                }),
                resources: if rng.gen_bool(0.7) {
                    Some((0..rng.gen_range(1..5))
                        .map(|i| ResourceId::new(format!("res_{}", i)))
                        .collect())
                } else {
                    None
                },
                domains: if rng.gen_bool(0.5) {
                    Some((0..rng.gen_range(1..3))
                        .map(|i| DomainId::new(format!("dom_{}", i)))
                        .collect())
                } else {
                    None
                },
            })
        },
        EntryType::Effect => {
            EntryData::Effect(EffectEntry {
                effect_type: EffectType::Create,
                resources: (0..rng.gen_range(1..5))
                    .map(|i| ResourceId::new(format!("res_{}", i)))
                    .collect(),
                domains: (0..rng.gen_range(1..3))
                    .map(|i| DomainId::new(format!("dom_{}", i)))
                    .collect(),
                code_hash: if rng.gen_bool(0.5) {
                    Some(random_string(rng, 32))
                } else {
                    None
                },
                parameters: {
                    let mut params = HashMap::new();
                    for _ in 0..rng.gen_range(0..5) {
                        params.insert(random_string(rng, 5), random_string(rng, 10));
                    }
                    params
                },
                result: if rng.gen_bool(0.7) {
                    Some(serde_json::json!({
                        "status": if rng.gen_bool(0.9) { "success" } else { "failure" },
                        "data": {
                            "value": random_string(rng, 8),
                            "counter": rng.gen::<u32>(),
                        }
                    }))
                } else {
                    None
                },
                success: rng.gen_bool(0.9),
                error: if rng.gen_bool(0.1) {
                    Some(format!("Error: {}", random_string(rng, 15)))
                } else {
                    None
                },
            })
        },
        EntryType::Fact => {
            EntryData::Fact(FactEntry {
                domain: DomainId::new(format!("dom_{}", rng.gen_range(0..3))),
                block_height: rng.gen_range(0..10000),
                block_hash: if rng.gen_bool(0.9) {
                    Some(random_string(rng, 32))
                } else {
                    None
                },
                observed_at: rng.gen_range(0..100000),
                fact_type: format!("fact_type_{}", rng.gen_range(0..5)),
                resources: (0..rng.gen_range(1..5))
                    .map(|i| ResourceId::new(format!("res_{}", i)))
                    .collect(),
                data: serde_json::json!({
                    "value": random_string(rng, 12),
                    "counter": rng.gen::<u32>(),
                    "nested": {
                        "array": [1, 2, 3, 4, 5],
                        "object": {
                            "key": "value"
                        }
                    }
                }),
                verified: rng.gen_bool(0.8),
            })
        },
    };
    
    LogEntry {
        id: format!("entry_{}", entry_id),
        timestamp: random_timestamp(rng),
        entry_type,
        data,
        trace_id: if rng.gen_bool(0.8) {
            Some(format!("trace_{}", rng.gen_range(0..5)))
        } else {
            None
        },
        parent_id: if rng.gen_bool(0.3) {
            Some(format!("entry_{}", rng.gen_range(0..entry_id)))
        } else {
            None
        },
        metadata: {
            let mut metadata = HashMap::new();
            for _ in 0..rng.gen_range(0..3) {
                metadata.insert(random_string(rng, 5), random_string(rng, 8));
            }
            metadata
        },
        entry_hash: None,
    }
}

/// Fuzz test for basic log storage operations with boundary conditions
#[test]
fn test_fuzz_storage_operations() -> Result<()> {
    // Use a fixed seed for reproducibility
    let seed = 12345;
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Create storage
    let storage = MemoryLogStorage::new();
    
    // Generate and append random entries
    let entry_count = 200;
    let entries: Vec<LogEntry> = (0..entry_count)
        .map(|i| generate_random_entry(&mut rng, i))
        .collect();
    
    for entry in &entries {
        storage.append(entry.clone())?;
    }
    
    // Verify entry count
    assert_eq!(storage.entry_count()?, entry_count);
    
    // Test boundary conditions for read operations
    
    // 1. Reading with zero limit
    let zero_limit = storage.read(0, 0)?;
    assert_eq!(zero_limit.len(), 0);
    
    // 2. Reading with large offset
    let large_offset = storage.read(entry_count + 100, 10)?;
    assert_eq!(large_offset.len(), 0);
    
    // 3. Reading with offset at boundary
    let boundary_offset = storage.read(entry_count - 1, 10)?;
    assert_eq!(boundary_offset.len(), 1);
    
    // 4. Reading with large limit
    let large_limit = storage.read(0, entry_count + 100)?;
    assert_eq!(large_limit.len(), entry_count);
    
    // 5. Reading with various combinations of offset and limit
    for _ in 0..20 {
        let offset = rng.gen_range(0..entry_count);
        let limit = rng.gen_range(1..entry_count);
        
        let expected_count = std::cmp::min(limit, entry_count - offset);
        let read_entries = storage.read(offset, limit)?;
        
        assert_eq!(read_entries.len(), expected_count);
        
        // Verify entries match
        for i in 0..read_entries.len() {
            assert_eq!(entries[offset + i].id, read_entries[i].id);
        }
    }
    
    Ok(())
}

/// Fuzz test for hash verification under various conditions
#[test]
fn test_fuzz_hash_verification() -> Result<()> {
    let seed = 67890;
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Create storage with hash verification
    let storage = MemoryLogStorage::new_with_config(
        StorageConfig {
            verify_hashes: true,
            enforce_hash_verification: true,
            ..Default::default()
        }
    );
    
    // Generate and append random entries
    let entry_count = 50;
    let mut entries = Vec::with_capacity(entry_count);
    
    for i in 0..entry_count {
        let mut entry = generate_random_entry(&mut rng, i);
        
        // For some entries, pre-compute the hash
        if rng.gen_bool(0.5) {
            generate_entry_hash(&mut entry)?;
        }
        
        // For some entries with pre-computed hashes, tamper with the data
        if entry.entry_hash.is_some() && rng.gen_bool(0.2) {
            let hash = entry.entry_hash.clone();
            
            // Tamper with the entry data
            match &mut entry.data {
                EntryData::Event(e) => e.event_name = format!("tampered_{}", e.event_name),
                EntryData::Effect(e) => e.success = !e.success,
                EntryData::Fact(f) => f.verified = !f.verified,
            }
            
            // Try to append with invalid hash - should fail
            let result = storage.append(entry.clone());
            assert!(result.is_err());
            
            // Fix the hash for subsequent operations
            entry.entry_hash = None;
        }
        
        // For valid entries, append them
        if entry.entry_hash.is_none() {
            storage.append(entry.clone())?;
            entries.push(entry);
        }
    }
    
    // Verify all stored entries have valid hashes
    let stored_entries = storage.read(0, entries.len())?;
    for entry in &stored_entries {
        assert!(entry.entry_hash.is_some());
        assert!(verify_entry_hash(entry)?);
    }
    
    Ok(())
}

/// Fuzz test for log replay with mixed entry types and error conditions
#[test]
fn test_fuzz_replay_engine() -> Result<()> {
    let seed = 13579;
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Create storage
    let storage = Arc::new(MemoryLogStorage::new());
    
    // Generate random entries with different trace IDs and resources
    let entry_count = 100;
    let mut entries = Vec::with_capacity(entry_count);
    
    // Track trace IDs and resources for filtering
    let mut trace_ids = Vec::new();
    let mut resources = Vec::new();
    
    for i in 0..entry_count {
        let mut entry = generate_random_entry(&mut rng, i);
        
        // Assign to specific trace IDs for filtering tests
        let trace_id = format!("trace_{}", rng.gen_range(0..5));
        entry.trace_id = Some(trace_id.clone());
        
        if !trace_ids.contains(&trace_id) {
            trace_ids.push(trace_id);
        }
        
        // Extract resources for filtering
        match &entry.data {
            EntryData::Event(e) => {
                if let Some(res) = &e.resources {
                    for r in res {
                        if !resources.contains(r) {
                            resources.push(r.clone());
                        }
                    }
                }
            },
            EntryData::Effect(e) => {
                for r in &e.resources {
                    if !resources.contains(r) {
                        resources.push(r.clone());
                    }
                }
            },
            EntryData::Fact(f) => {
                for r in &f.resources {
                    if !resources.contains(r) {
                        resources.push(r.clone());
                    }
                }
            },
        }
        
        storage.append(entry.clone())?;
        entries.push(entry);
    }
    
    // Test replay with trace filtering
    if !trace_ids.is_empty() {
        let trace_id = trace_ids[rng.gen_range(0..trace_ids.len())].clone();
        
        let options = ReplayOptions {
            trace_id: Some(trace_id.clone()),
            ..Default::default()
        };
        
        let callback = Arc::new(StatsCallback::new());
        let engine = ReplayEngine::new(
            storage.clone(),
            options,
            callback.clone(),
        );
        
        let result = engine.run()?;
        assert_eq!(result.status, ReplayStatus::Complete);
        
        let filtered_count = entries.iter()
            .filter(|e| e.trace_id.as_deref() == Some(&trace_id))
            .count();
        
        assert_eq!(callback.entries_processed(), filtered_count);
    }
    
    // Test replay with resource filtering
    if !resources.is_empty() {
        let resource = resources[rng.gen_range(0..resources.len())].clone();
        
        let options = ReplayOptions {
            resources: Some(vec![resource.clone()]),
            ..Default::default()
        };
        
        let callback = Arc::new(StatsCallback::new());
        let engine = ReplayEngine::new(
            storage.clone(),
            options,
            callback.clone(),
        );
        
        let result = engine.run()?;
        assert_eq!(result.status, ReplayStatus::Complete);
        
        // Count should match filtered entries
        let filtered_count = entries.iter()
            .filter(|e| {
                match &e.data {
                    EntryData::Event(event) => {
                        if let Some(resources) = &event.resources {
                            resources.contains(&resource)
                        } else {
                            false
                        }
                    },
                    EntryData::Effect(effect) => effect.resources.contains(&resource),
                    EntryData::Fact(fact) => fact.resources.contains(&resource),
                }
            })
            .count();
        
        assert_eq!(callback.entries_processed(), filtered_count);
    }
    
    Ok(())
}

/// Fuzz test for segment manager with various rotation criteria
#[test]
fn test_fuzz_segment_manager() -> Result<()> {
    use tempfile::tempdir;
    use std::time::Duration;
    
    let seed = 24680;
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create segment manager with random rotation criteria
    let options = SegmentManagerOptions {
        base_dir,
        max_active_segments: rng.gen_range(2..10),
        compress_inactive: rng.gen_bool(0.5),
        rotation_criteria: vec![
            RotationCriteria::EntryCount(rng.gen_range(5..20)),
            RotationCriteria::Size(rng.gen_range(1000..10000)),
            RotationCriteria::TimeInterval(chrono::Duration::seconds(rng.gen_range(1..10))),
        ],
        segment_name_pattern: "test_segment_{timestamp}".to_string(),
        auto_flush: rng.gen_bool(0.7),
        index_dir: None,
    };
    
    // Create a segment manager
    let segment_manager = Arc::new(LogSegmentManager::new(
        options,
        StorageConfig::default(),
    )?);
    
    // Generate and append random entries
    let entry_count = 100;
    
    for i in 0..entry_count {
        let entry = generate_random_entry(&mut rng, i);
        segment_manager.append(entry)?;
        
        // Random small delay to test time-based rotation
        if rng.gen_bool(0.1) {
            std::thread::sleep(Duration::from_millis(rng.gen_range(10..100)));
        }
    }
    
    // Check the number of segments created
    let segments = segment_manager.list_segments()?;
    assert!(!segments.is_empty());
    
    // Total entries across all segments should match what we added
    let mut total_entries = 0;
    for segment_id in &segments {
        if let Some(segment) = segment_manager.get_segment(segment_id)? {
            total_entries += segment.lock().unwrap().entry_count()?;
        }
    }
    
    assert_eq!(total_entries, entry_count);
    
    // Test reading entries across segments
    let all_entries = segment_manager.read_all_entries()?;
    assert_eq!(all_entries.len(), entry_count);
    
    Ok(())
} 