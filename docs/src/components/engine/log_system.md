<!-- The log system -->
<!-- Original file: docs/src/log_system.md -->

# Causality Unified Log System

## Overview

The Causality Unified Log System provides a comprehensive approach to recording, storing, and replaying all system activities. This document outlines the log formats, storage mechanisms, and usage patterns for developers integrating with the Causality ecosystem.

## Core Concepts

### Log Entry Types

The system defines three primary types of log entries:

1. **Effect Entries**: Record state changes and side effects in the system
2. **Fact Entries**: Document observed truths or assertions about the system state
3. **Event Entries**: Capture significant occurrences that may not directly change state

### Log Structure

Each log entry contains:

- **Entry Type**: Identifies the entry as an Effect, Fact, or Event
- **Timestamp**: Records when the entry was logged
- **Trace ID**: Groups related entries in a causal chain
- **Domain**: Categorizes the entry by functional area
- **Resource ID**: Identifies the affected resource (if applicable)
- **Data**: Contains the serialized payload specific to the entry type
- **Metadata**: Stores additional contextual information

## Storage Format

### Log Segments

Logs are stored in segments to facilitate:
- Efficient rotation and archival
- Parallel processing and replay
- Resource-specific filtering

Segment files use a binary format with:
- Header containing version, creation timestamp, and entry count
- Sequence of serialized log entries
- Index for rapid entry retrieval by timestamp or trace ID

### Serialization Format

Log entries are serialized using bincode with the following schema:

```rust
// Entry Header (common to all types)
struct LogEntryHeader {
    entry_type: EntryType,      // u8: 0=Event, 1=Effect, 2=Fact
    timestamp: u64,             // Milliseconds since epoch
    trace_id: [u8; 32],         // 32-byte trace identifier
    domain: [u8; 16],           // 16-byte domain identifier
    resource_id: Option<u64>,   // Resource identifier if applicable
    data_length: u32,           // Length of the entry-specific data
}

// Followed by entry-specific data and metadata
```

## Working with the Log System

### Logging Events

```rust
use causality::log::{LogStorage, EventEntry};

fn log_event(storage: &mut impl LogStorage, trace_id: TraceId, event_type: &str, data: &[u8]) {
    let entry = EventEntry::new(trace_id, event_type, data);
    storage.append_entry(&entry).expect("Failed to log event");
}
```

### Logging Effects

```rust
use causality::log::{LogStorage, EffectEntry};
use causality::effect::EffectType;

fn log_effect(
    storage: &mut impl LogStorage, 
    trace_id: TraceId, 
    effect_type: EffectType, 
    resource_id: u64, 
    data: &[u8]
) {
    let entry = EffectEntry::new(trace_id, effect_type, resource_id, data);
    storage.append_entry(&entry).expect("Failed to log effect");
}
```

### Logging Facts

```rust
use causality::log::{LogStorage, FactEntry};

fn log_fact(
    storage: &mut impl LogStorage, 
    trace_id: TraceId, 
    fact_type: &str, 
    resource_id: Option<u64>, 
    data: &[u8]
) {
    let entry = FactEntry::new(trace_id, fact_type, resource_id, data);
    storage.append_entry(&entry).expect("Failed to log fact");
}
```

### Reading Log Entries

```rust
use causality::log::{LogStorage, LogEntry};

fn read_logs(storage: &impl LogStorage) {
    let entries = storage.read_entries(0, 100)
        .expect("Failed to read entries");
    
    for entry in entries {
        match entry.entry_type() {
            EntryType::Event => println!("Event: {}", entry.as_event().event_type()),
            EntryType::Effect => println!("Effect on resource: {}", entry.as_effect().resource_id()),
            EntryType::Fact => println!("Fact: {}", entry.as_fact().fact_type()),
        }
    }
}
```

### Log Replay

```rust
use causality::log::{ReplayEngine, ReplayCallback, ReplayFilter};

struct MyCallback;

impl ReplayCallback for MyCallback {
    fn on_effect(&mut self, effect: &EffectEntry) {
        println!("Replaying effect on resource: {}", effect.resource_id());
        // Perform the actual effect
    }
    
    fn on_fact(&mut self, fact: &FactEntry) {
        println!("Observed fact: {}", fact.fact_type());
        // Update state based on fact
    }
}

fn replay_logs(storage: &impl LogStorage) {
    let callback = MyCallback;
    let filter = ReplayFilter::new()
        .with_resource_id(123)  // Only entries for this resource
        .with_domain("billing");  // Only entries from this domain
    
    let mut engine = ReplayEngine::new(storage, callback);
    let result = engine.run_with_filter(&filter);
    
    println!("Replay completed with {} entries processed", result.processed_count);
}
```

## Storage Implementation Options

### Memory Storage

Ideal for testing and temporary sessions:

```rust
use causality::log::MemoryLogStorage;

let storage = MemoryLogStorage::new();
// Use storage for logging...
```

### File Storage

Persistent storage with configurable retention:

```rust
use causality::log::FileLogStorage;

let storage = FileLogStorage::new("/path/to/logs", "app-logs")
    .with_segment_size(1024 * 1024)  // 1MB segments
    .with_retention_days(30);        // Keep logs for 30 days
```

## Advanced Usage

### Time Map Integration

The log system integrates with Causality's time map for temporal queries through the `time_map` module:

```rust
use causality::log::LogTimeMapIntegration;
use causality::domain::map::map::TimeMap;

fn query_logs_by_time(
    time_map: &TimeMap, 
    storage: &impl LogStorage, 
    start_time: u64, 
    end_time: u64
) {
    let entries = LogTimeMapIntegration::query_time_range(
        time_map, 
        storage, 
        start_time, 
        end_time
    );
    
    for entry in entries {
        // Process entries within the time range
    }
}
```

The `LogTimeMapIntegration` implementation provides feature-gated functionality:

```rust
// When the "domain" feature is enabled, a full implementation is provided
#[cfg(feature = "domain")]
let integration = LogTimeMapIntegration::new(Arc::new(Mutex::new(storage)));

// For all feature configurations, static methods are available
// Attach time map information to a log entry
LogTimeMapIntegration::attach_time_map(&mut entry, &time_map)?;

// Verify a log entry's time map hash
if !LogTimeMapIntegration::verify_time_map(&entry, &time_map)? {
    // Handle time map inconsistency
}
```

### Causal Consistency Verification

Verify that logged effects maintain causal consistency:

```rust
use causality::log::ConsistencyVerifier;

fn verify_consistency(storage: &impl LogStorage) -> bool {
    let verifier = ConsistencyVerifier::new();
    verifier.verify(storage)
}
```

## Best Practices

1. **Trace ID Management**: Always propagate trace IDs across system boundaries to maintain causal relationships.

2. **Domain Organization**: Organize logs by functional domains to improve filtering and readability.

3. **Resource Tagging**: Consistently tag log entries with relevant resource IDs to enable resource-specific replay.

4. **Payload Size**: Keep log entry payloads small and focused for efficiency; reference larger data externally.

5. **Regular Rotation**: Configure appropriate segment rotation to balance performance and storage needs.

6. **Structured Logging**: Use structured formats within the log data payload for easier parsing and analysis.

7. **Replay Isolation**: When replaying logs, ensure side effects are properly isolated or mocked to prevent unintended consequences.

## Troubleshooting

Common issues and their solutions:

1. **Missing Entries**: Check segment boundaries and ensure proper timestamp ordering.

2. **Replay Inconsistencies**: Verify that all resources referenced in effects exist and are in expected states.

3. **Performance Issues**: Consider increasing segment size or implementing batched writes for high-volume logging.

4. **Storage Growth**: Monitor log retention policies and implement compression for archived segments.

## Conclusion

The Unified Log System serves as the foundation for auditing, debugging, and state reconstruction in the Causality ecosystem. By following the patterns and practices outlined in this document, developers can leverage the full power of deterministic replay and consistent state management. 