<!-- Module for time -->
<!-- Original file: docs/src/time_module.md -->

# Time Module Documentation

## Overview

The Time Module provides a comprehensive system for managing and synchronizing time across different domains in a distributed system. It handles time representation, synchronization, and drift management to ensure consistent timelines between domains.

## Components

### Core Types (`types.rs`)

#### TimePoint

Represents a specific observed moment in a domain's timeline.

```rust
pub struct TimePoint {
    pub height: BlockHeight,      // Block height associated with this time point
    pub hash: BlockHash,          // Block hash associated with this time point
    pub timestamp: Timestamp,     // Timestamp in seconds
    pub confidence: f64,          // Confidence level (0.0-1.0)
    pub verified: bool,           // Whether this time point has been verified
    pub source: String,           // Source of this time point
}
```

#### TimeRange

Represents a range of time between two timestamps.

```rust
pub struct TimeRange {
    pub start: Timestamp,         // Starting time in seconds
    pub end: Timestamp,           // Ending time in seconds
    pub start_inclusive: bool,    // Whether this range is inclusive at the start
    pub end_inclusive: bool,      // Whether this range is inclusive at the end
}
```

#### TimeWindow

Represents a snapshot of a domain at a specific time range.

```rust
pub struct TimeWindow {
    pub domain_id: DomainId,      // Domain identifier
    pub range: TimeRange,         // Range of time this window covers
    pub start_height: BlockHeight, // Block height at the start of the window
    pub end_height: Option<BlockHeight>, // Block height at the end of the window
    pub start_hash: BlockHash,    // Block hash at the start of the window
    pub end_hash: Option<BlockHash>, // Block hash at the end of the window
    pub created_at: DateTime<Utc>, // When this window was created
    pub metadata: HashMap<String, String>, // Additional metadata
}
```

### Time Map (`map.rs`)

The time map is a core data structure that tracks the observed state of domains over time.

#### TimeMapEntry

An entry in the time map for a specific domain.

```rust
pub struct TimeMapEntry {
    pub domain_id: DomainId,      // Domain identifier
    pub height: BlockHeight,      // Block height
    pub hash: BlockHash,          // Block hash
    pub timestamp: Timestamp,     // Timestamp
    pub observed_at: DateTime<Utc>, // When this entry was observed
    pub confidence: f64,          // Confidence in this entry (0.0-1.0)
    pub verified: bool,           // Whether this entry is verified
    pub source: String,           // Source of this entry
    pub metadata: HashMap<String, String>, // Additional metadata
}
```

#### TimeMap

A collection of time map entries for different domains.

```rust
pub struct TimeMap {
    pub entries: HashMap<DomainId, TimeMapEntry>, // Map from domain ID to time map entry
    pub created_at: DateTime<Utc>, // When this time map was created
    pub version: u64,             // Version of this time map
    pub metadata: HashMap<String, String>, // Additional metadata
}
```

#### SharedTimeMap

Thread-safe wrapper around a time map.

```rust
pub struct SharedTimeMap {
    inner: Arc<RwLock<TimeMap>>,
    history: Arc<Mutex<TimeMapHistory>>,
}
```

### Time Synchronization (`sync.rs`)

The time synchronization system ensures that time is consistently tracked and synchronized across domains.

#### TimeSyncConfig

Configuration for the time synchronization manager.

```rust
pub struct TimeSyncConfig {
    pub sync_interval: u64,       // How often to attempt synchronization (in seconds)
    pub sync_timeout: u64,        // Timeout for sync operations (in seconds)
    pub max_time_difference: u64, // Maximum time difference allowed between domains
    pub min_confidence: f64,      // Minimum confidence threshold for accepting time points
    pub history_size: usize,      // Number of history points to maintain
    pub verify_time_points: bool, // Whether to verify time points cryptographically
}
```

#### TimeSyncManager

The main component for managing time synchronization across domains.

```rust
pub struct TimeSyncManager {
    config: TimeSyncConfig,       // Configuration for time sync
    time_map: SharedTimeMap,      // Shared time map
    providers: Arc<RwLock<HashMap<DomainId, TimePointProvider>>>, // Time point providers
    recent_points: Arc<RwLock<Vec<(DomainId, TimePoint)>>>, // Recently observed time points
    event_tx: broadcast::Sender<SyncResult>, // Broadcast channel for sync events
    running: Arc<RwLock<bool>>,   // Running flag
}
```

## Usage Examples

### Creating a Time Point

```rust
use causality::domain::time::types::TimePoint;

let time_point = TimePoint::new(
    100,                        // Block height
    "block_hash_123".into(),    // Block hash
    1620000000,                 // Timestamp (Unix time)
)
.with_confidence(0.9)            // Set confidence level
.with_verification(true)         // Mark as verified
.with_source("rpc");             // Set the source
```

### Working with Time Ranges

```rust
use causality::domain::time::types::TimeRange;

// Create an inclusive range [1000, 2000]
let range = TimeRange::new(1000, 2000);

// Create an exclusive range (1000, 2000)
let exclusive_range = TimeRange::exclusive(1000, 2000);

// Create a half-open range [1000, 2000)
let half_open = TimeRange::half_open(1000, 2000);

// Check if a timestamp is within the range
let is_in_range = range.contains(1500);  // true

// Find the intersection of two ranges
let range2 = TimeRange::new(1500, 2500);
if let Some(intersection) = range.intersection(&range2) {
    println!("Intersection: {}", intersection);  // [1500, 2000]
}
```

### Managing Time Maps

```rust
use causality::domain::time::map::{TimeMap, TimeMapEntry};
use causality::domain::DomainId;

// Create a new time map
let mut time_map = TimeMap::new();

// Add some domains
let domain1: DomainId = "bitcoin".into();
let domain2: DomainId = "ethereum".into();

let entry1 = TimeMapEntry::new(
    domain1.clone(),
    100,                     // Block height
    "block_hash_btc".into(), // Block hash
    1620000000,              // Timestamp
)
.with_confidence(1.0)
.with_verification(true);

let entry2 = TimeMapEntry::new(
    domain2.clone(),
    13000000,                // Block height
    "block_hash_eth".into(), // Block hash
    1620000120,              // Timestamp
)
.with_confidence(0.9)
.with_verification(true);

// Add entries to the time map
time_map.update_domain(entry1);
time_map.update_domain(entry2);

// Get information about domains
let btc_height = time_map.get_height(&domain1);  // Some(100)
let eth_timestamp = time_map.get_timestamp(&domain2);  // Some(1620000120)

// Filter the time map
let verified_only = time_map.verified_only();
let recent_only = time_map.recent_only(60);  // Only entries from last 60 minutes
```

### Using Time Synchronization

```rust
use causality::domain::time::sync::{TimeSyncManager, TimeSyncConfig};
use causality::domain::time::map::SharedTimeMap;
use causality::domain::DomainId;

// Create a shared time map
let shared_map = SharedTimeMap::new();

// Create a sync config
let config = TimeSyncConfig {
    sync_interval: 60,           // Sync every 60 seconds
    sync_timeout: 30,            // 30-second timeout for sync operations
    max_time_difference: 300,    // Allow up to 5 minutes drift
    min_confidence: 0.7,         // Require at least 70% confidence
    history_size: 100,           // Keep 100 historical time points
    verify_time_points: true,    // Verify time points
};

// Create a sync manager
let sync_manager = TimeSyncManager::new(config, shared_map);

// Register time point providers for domains
let btc_domain: DomainId = "bitcoin".into();
let eth_domain: DomainId = "ethereum".into();

// Create a provider from a closure
let btc_provider = TimeSyncManager::create_provider(|domain_id| {
    // In a real application, this would query the Bitcoin network
    Ok(TimePoint::new(100, "block_hash_btc".into(), 1620000000)
        .with_confidence(1.0)
        .with_verification(true)
        .with_source("bitcoin_rpc"))
});

sync_manager.register_provider(btc_domain.clone(), btc_provider)?;

// Start the synchronization loop
sync_manager.start().await?;

// Manually trigger a sync
let sync_result = sync_manager.sync_now().await?;
println!("Synced {} domains", sync_result.synced_count());

// Calculate drift between domains
let drift = sync_manager.calculate_drift(&btc_domain, &eth_domain)?;
println!("Drift: {} seconds", drift.num_seconds());

// Check if the drift is acceptable
let acceptable = sync_manager.is_drift_acceptable(&btc_domain, &eth_domain)?;
```

## Integration with Other Systems

The Time Module is designed to be integrated with other components of the Causality system:

1. **Fact Observation**: Time synchronization is essential for correctly ordering facts observed from different domains.
2. **Verification**: Verified time points provide a trusted basis for cross-domain operations.
3. **Event Processing**: Consistent time ordering enables reliable event processing across domains.
4. **Consensus**: Time synchronization supports consensus protocols that require timestamp agreement.

## Best Practices

1. **Regular Synchronization**: Configure an appropriate sync interval based on your requirements for time accuracy.
2. **Confidence Thresholds**: Set the minimum confidence threshold based on your system's tolerance for uncertainty.
3. **Verification**: When dealing with critical operations, prefer using verified time points.
4. **Drift Management**: Regularly monitor and handle time drift between domains.
5. **Multiple Sources**: Register multiple time point providers for critical domains to increase reliability.

## Future Enhancements

1. **Consensus-based Time**: Implement consensus algorithms for establishing agreed-upon time across multiple observers.
2. **NTP Integration**: Support integration with Network Time Protocol for better external time synchronization.
3. **Time Anomaly Detection**: Add capabilities to detect and alert on suspicious time patterns or anomalies.
4. **Historical Time Queries**: Enhanced support for querying historical time states and reconstructing timelines.
5. **Performance Optimizations**: Further optimize time synchronization for high-throughput environments. 