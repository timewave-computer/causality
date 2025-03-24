# Time Map and Unified Log Integration

This document describes the integration between the Map of Time and the Unified Log System in Causality.

## Overview

The Map of Time is a causal graph that tracks the observed state of external domains over time. It serves as a "global clock" for the system, ensuring that all actors have a consistent view of external state. The Unified Log, on the other hand, is an append-only record of all events, facts, and effects that occur in the system.

The integration between these two systems ensures:

1. **Temporal Consistency**: All effects are anchored to a specific state of external domains (the Time Map).
2. **Causal Verification**: Effects can verify that they were applied in a consistent causal order.
3. **Deterministic Replay**: The system can be deterministically replayed using the unified log and time map together.

## Core Components

### LogTimeMapIntegration

The `LogTimeMapIntegration` struct (located in the `time_map.rs` module) provides methods for attaching time map information to log entries and querying logs based on time-related criteria:

```rust
pub struct LogTimeMapIntegration;

impl LogTimeMapIntegration {
    // Add time map information to a log entry
    pub fn attach_time_map(entry: &mut LogEntry, time_map: &TimeMap) -> Result<()>;
    
    // Calculate a hash of the time map
    fn calculate_time_map_hash(time_map: &TimeMap) -> Result<String>;
    
    // Query log entries by time map time range
    pub fn query_time_range(
        time_map: &TimeMap,
        storage: &dyn LogStorage,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<Vec<LogEntry>>;
    
    // Get the time map hash from a log entry
    pub fn get_time_map_hash(entry: &LogEntry) -> Option<&str>;
    
    // Verify that a log entry's time map hash matches a given time map
    pub fn verify_time_map(entry: &LogEntry, time_map: &TimeMap) -> Result<bool>;
}
```

The implementation also provides feature-gated versions with different capabilities:

```rust
// When the "domain" feature is enabled, a full implementation is provided
#[cfg(feature = "domain")]
pub struct LogTimeMapIntegration {
    storage: Arc<Mutex<dyn LogStorage + Send>>,
    time_map: TimeMap,
    indexed_up_to: u64,
}

// When the "domain" feature is not enabled, a minimal stub implementation is provided
#[cfg(not(feature = "domain"))]
pub struct LogTimeMapIntegration {}
```

### Time Map Integration in ReplayEngine

The `ReplayEngine` has been enhanced to support time map integration:

```rust
pub struct ReplayEngine {
    // ... other fields ...
    time_map: Option<TimeMap>,
}

impl ReplayEngine {
    // Create a new replay engine with a time map for temporal verification
    pub fn with_time_map(
        storage: Arc<dyn LogStorage>,
        options: ReplayOptions,
        callback: Arc<dyn ReplayCallback>,
        time_map: TimeMap,
    ) -> Self;
    
    // Run the replay with a time map time range filter
    pub fn run_with_time_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<ReplayResult>;
}
```

## How It Works

### 1. Time Map Attachment

When an effect is proposed, the current Time Map is attached to the log entry:

```rust
let mut entry = LogEntry::new_effect(...);
LogTimeMapIntegration::attach_time_map(&mut entry, &current_time_map)?;
```

This attachment includes:
- A hash of the time map to ensure integrity
- The list of observed domains
- The time map version
- The time map creation timestamp

### 2. Time Map Verification

During replay or effect application, the system can verify that the time map is consistent:

```rust
if !LogTimeMapIntegration::verify_time_map(&entry, &time_map)? {
    return Err(Error::Other("Time map verification failed".to_string()));
}
```

This ensures that effects are only applied in a consistent causal order.

### 3. Time-Based Queries

The system can query log entries based on time criteria:

```rust
let entries = LogTimeMapIntegration::query_time_range(
    &time_map,
    &storage,
    start_time,
    end_time
)?;
```

This allows for efficient retrieval of log entries that occurred during a specific time window.

## Implementation Details

### Time Map Hash Generation

To ensure the integrity of the time map, a deterministic hash is generated:

1. Domain entries are sorted by domain ID for consistent ordering
2. A string representation is created with version, timestamp, and domain entries
3. The hash is calculated using Blake3

### Integration in Fact and Effect Processing

Facts update the time map by providing new information about domain state. Effects verify their consistency against the time map.

#### Facts

When a fact is observed, it updates the time map with the latest domain information:

```rust
for domain_id in domains {
    let time_map_entry = TimeMapEntry::new(
        domain_id.clone(),
        fact.height.clone(),
        fact.hash.clone(),
        fact.timestamp.clone()
    );
    time_map.update_domain(time_map_entry);
}
```

#### Effects

When an effect is applied, it verifies its consistency with the time map:

```rust
let time_map_hash = LogTimeMapIntegration::get_time_map_hash(&effect_entry)?;
if calculated_hash != time_map_hash {
    return Err(Error::TimeMapInconsistency);
}
```

## Benefits

1. **Causal Consistency**: The integration ensures that all effects are applied in a causally consistent order.
2. **Deterministic Replay**: The system can be deterministically replayed using the unified log and time map.
3. **Temporal Queries**: The system can efficiently query log entries based on time criteria.
4. **Verification**: Effects can verify their consistency with the time map.

## Module Organization

The Time Map integration functionality is now consolidated in the `time_map.rs` module, which provides:

1. **Feature-based Implementations**: Different implementations based on whether the `domain` feature is enabled
2. **Full Integration APIs**: All the necessary methods for integrating Time Maps with the Log system
3. **Comprehensive Test Coverage**: Tests for both feature configurations

## Usage Examples

### Attaching a Time Map to a Log Entry

```rust
let mut entry = LogEntry::new_effect(...);
LogTimeMapIntegration::attach_time_map(&mut entry, &current_time_map)?;
storage.append(entry)?;
```

### Replaying with Time Map Verification

```rust
let engine = ReplayEngine::with_time_map(
    storage.clone(),
    ReplayOptions::default(),
    callback,
    time_map.clone()
);
let result = engine.run()?;
```

### Querying Logs by Time Range

```rust
let entries = LogTimeMapIntegration::query_time_range(
    &time_map,
    &storage,
    Timestamp::new(1000),
    Timestamp::new(2000)
)?;
``` 