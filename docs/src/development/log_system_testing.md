<!-- Testing the log system -->
<!-- Original file: docs/src/log_system_testing.md -->

# Comprehensive Testing Framework for Causality Log System

This document outlines the testing approach for the Causality Unified Log System, including the various types of tests implemented to ensure robustness and reliability.

## Testing Approach

The Causality log system employs a multi-layered testing strategy:

1. **Unit Tests**: Focused tests for individual components
2. **Integration Tests**: Tests that verify interactions between multiple components
3. **Property-Based Tests**: Tests that verify system invariants with randomized inputs
4. **Fuzz Tests**: Tests designed to find edge cases and unexpected behaviors
5. **Simulation Tests**: Time-based and scenario-based tests

## Core Testing Components

### Unit Tests

Unit tests validate individual components such as:

- `LogEntry` and its serialization/deserialization
- `MemoryLogStorage` and `FileLogStorage` implementations
- `LogSegment` and segment management
- Entry hash generation and verification
- Filtering and query mechanisms

Example unit test:

```rust
#[test]
fn test_memory_log_storage() -> Result<()> {
    let storage = MemoryLogStorage::new();
    
    // Add some test entries
    for i in 0..10 {
        let entry = create_test_entry(i);
        storage.append(entry)?;
    }
    
    // Check entry count
    assert_eq!(storage.entry_count()?, 10);
    
    // Read entries
    let entries = storage.read(0, 5)?;
    assert_eq!(entries.len(), 5);
    
    Ok(())
}
```

### Integration Tests

Integration tests verify that components work correctly together:

- Storage and replay engine integration
- Segment manager and storage integration
- Content addressing across the entire storage pipeline
- Time map integration with the log system

Example integration test:

```rust
#[test]
fn test_replay_with_segment_manager() -> Result<()> {
    // Create a segment manager
    let segment_manager = create_test_segment_manager();
    
    // Create entries in multiple segments
    create_test_entries(segment_manager.clone())?;
    
    // Create a replay engine with the segment manager
    let engine = ReplayEngine::with_segment_manager(
        storage,
        replay_options,
        callback,
        segment_manager.clone(),
    );
    
    // Run the replay
    let result = engine.run()?;
    
    // Verify the result
    assert_eq!(result.status, ReplayStatus::Complete);
    
    Ok(())
}
```

### Property-Based Tests

Property-based tests use generated random inputs to verify system invariants:

- Storage operations maintain data integrity
- Reading with different offsets and limits works correctly
- Hash verification correctly validates entries
- Storage remains consistent under a wide range of inputs

Example property test:

```rust
proptest! {
    #[test]
    fn test_append_read_preserves_data(entries in vec(log_entry_strategy(), 1..100)) -> Result<()> {
        let storage = MemoryLogStorage::new();
        
        // Append all entries
        for entry in &entries {
            storage.append(entry.clone())?;
        }
        
        // Check entry count
        prop_assert_eq!(storage.entry_count()?, entries.len());
        
        // Read all entries and verify they match
        let read_entries = storage.read(0, entries.len())?;
        for (original, read) in entries.iter().zip(read_entries.iter()) {
            prop_assert_eq!(original.id, read.id);
            // ...more assertions
        }
        
        Ok(())
    }
}
```

### Fuzz Tests

Fuzz tests focus on finding edge cases and unexpected behaviors:

- Boundary conditions for read/write operations
- Hash tamper detection
- Log replay with mixed entry types
- Segment rotation under various conditions

Example fuzz test:

```rust
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
    
    // Generate entries, sometimes tampering with data
    for i in 0..50 {
        let mut entry = generate_random_entry(&mut rng, i);
        
        // Sometimes tamper with the data
        if entry.entry_hash.is_some() && rng.gen_bool(0.2) {
            // Tamper with entry data
            // ...
            
            // Should fail to append tampered entry
            let result = storage.append(entry.clone());
            assert!(result.is_err());
        }
        
        // For valid entries, append them
        if entry.entry_hash.is_none() {
            storage.append(entry.clone())?;
        }
    }
    
    // Verify all stored entries have valid hashes
    let stored_entries = storage.read(0, storage.entry_count()?)?;
    for entry in &stored_entries {
        assert!(verify_entry_hash(entry)?);
    }
    
    Ok(())
}
```

### Simulation Tests

Simulation tests create complex scenarios to validate system behavior:

- Time-based segment rotation
- Log replay across multiple segments
- Resource and domain filtering
- High-volume operations

Example simulation test:

```rust
#[test]
fn test_segment_manager_rotation() -> Result<()> {
    // Create segment manager with time-based rotation
    let options = SegmentManagerOptions {
        rotation_criteria: vec![
            RotationCriteria::TimeInterval(Duration::minutes(5)),
            RotationCriteria::EntryCount(100),
        ],
        // ...other options
    };
    
    let segment_manager = LogSegmentManager::new(options, config)?;
    
    // Add entries with simulated time passing
    for i in 0..500 {
        let entry = create_test_entry(i);
        segment_manager.append(entry)?;
        
        // Simulate time passing for every 50 entries
        if i % 50 == 0 {
            advance_time(Duration::minutes(10));
        }
    }
    
    // Verify segments were rotated correctly
    let segments = segment_manager.list_segments()?;
    assert!(segments.len() > 1);
    
    Ok(())
}
```

## Testing Invariants

The comprehensive testing framework verifies the following system invariants:

1. **Data Integrity**: Entries are stored and retrieved without corruption
2. **Content Addressing**: Hashes correctly verify entry integrity
3. **Segmentation**: Entries span segments correctly and can be queried
4. **Rotation**: Segments rotate based on configured criteria
5. **Query Performance**: Queries return correct results efficiently
6. **Replay Fidelity**: Replaying logs reconstructs correct state

## Test Coverage

The test suite is designed to maintain a high level of coverage across the log system:

- **Line Coverage**: > 85% of code lines
- **Branch Coverage**: > 80% of conditional branches
- **Function Coverage**: > 90% of functions

## Continuous Testing

The log system tests are integrated into the CI/CD pipeline:

1. Fast unit tests run on every commit
2. Integration tests run on pull requests
3. Property and fuzz tests run on main branch updates
4. Long-running simulation tests run nightly

## Conclusion

The comprehensive testing framework ensures the Causality log system is robust, reliable, and behaves correctly under a wide range of conditions. The combination of unit tests, integration tests, property-based tests, fuzz tests, and simulation tests provides confidence in the system's correctness and resilience. 