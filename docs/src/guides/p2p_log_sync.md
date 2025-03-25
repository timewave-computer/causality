<!-- Guide for P2P log synchronization -->
<!-- Original file: docs/src/p2p_log_sync.md -->

# P2P Log Synchronization

This document describes the peer-to-peer log synchronization system for Causality, which enables Operators to maintain consistent logs across the network.

## Overview

The P2P log synchronization system is a critical component of the Causality framework, allowing different operators to synchronize their log segments and maintain a consistent view of the system state. The system supports differential synchronization to minimize network traffic and includes conflict detection and resolution mechanisms.

## Core Components

### SyncManager

The `SyncManager` is the central coordinator for log synchronization, responsible for:

- Managing the list of peers to synchronize with
- Scheduling regular synchronization operations
- Handling sync events and results
- Providing a control interface for manual sync operations

```rust
// Create a sync manager with HTTP-based protocol
let config = SyncConfig::default();
let storage = Arc::new(FileLogStorage::new("/path/to/logs"));
let protocol = Arc::new(HttpSyncProtocol::new("https://my-operator.example.com/sync", 30));

let mut sync_manager = SyncManager::new(config, storage, protocol);
sync_manager.start()?;

// Add peers to sync with
sync_manager.add_peer(PeerInfo {
    peer_id: "operator-1",
    address: "https://operator-1.example.com/sync",
    last_sync: None,
    status: SyncStatus::InProgress,
    successful_syncs: 0,
    failed_syncs: 0,
})?;
```

### SyncProtocol

The `SyncProtocol` trait defines the interface for different synchronization protocols. The framework includes an HTTP-based implementation, but you can create custom implementations for other transport mechanisms.

```rust
pub trait SyncProtocol: Send + Sync {
    fn get_segment_list(&self, peer_id: &str) -> Result<Vec<SegmentMetadata>>;
    fn get_segment(&self, peer_id: &str, segment_id: &str) -> Result<LogSegment>;
    fn send_segment(&self, peer_id: &str, segment: &LogSegment) -> Result<()>;
    fn get_log_diff(&self, peer_id: &str) -> Result<LogDiff>;
    fn resolve_conflicts(&self, peer_id: &str, conflicts: &[String]) -> Result<Vec<LogEntry>>;
}
```

### SyncableStorage

The `SyncableStorage` trait extends the `LogStorage` trait with synchronization capabilities. Storage implementations that support synchronization should implement this trait.

```rust
pub trait SyncableStorage: LogStorage {
    fn get_segment_metadata(&self) -> Result<Vec<SegmentMetadata>>;
    fn import_segment(&self, segment: LogSegment) -> Result<()>;
    fn export_segment(&self, segment_id: &str) -> Result<LogSegment>;
    fn get_log_differences(&self, remote_segments: &[SegmentMetadata]) -> Result<LogDiff>;
}
```

### Differential Synchronization

The synchronization system uses a differential approach to minimize network traffic:

1. Compare local and remote segment metadata
2. Identify segments that are missing or have different content hashes
3. Transfer only the segments that are needed
4. Detect and resolve conflicts if necessary

This approach is particularly efficient for large log stores, as it avoids unnecessary transfers of segments that are already in sync.

## Configuration Options

The `SyncConfig` struct provides various configuration options for the synchronization system:

```rust
pub struct SyncConfig {
    pub sync_interval: u64,         // Interval between sync attempts (in seconds)
    pub batch_size: usize,          // Maximum number of log entries to sync in a single batch
    pub sync_timeout: u64,          // Timeout for sync operations (in seconds)
    pub max_retries: usize,         // Maximum retries before giving up on a sync
    pub use_differential_sync: bool, // Whether to use differential sync
    pub max_segment_age: u64,       // Maximum age of segments to sync (0 for no limit)
    pub verify_hashes: bool,        // Whether to verify hashes during sync
}
```

## Event Subscription

The synchronization system provides an event-based interface for monitoring sync operations:

```rust
// Subscribe to sync events
let mut sync_events = sync_manager.subscribe();

// Process sync events
tokio::spawn(async move {
    while let Ok(result) = sync_events.recv().await {
        println!("Sync with {}: {:?}", result.peer_id, result.status);
        println!("Synced {} entries in {} segments", 
                 result.entries_synced, result.segments_synced);
    }
});
```

## Conflict Resolution

When conflicts are detected (different entries with the same ID or timestamp), the system provides several resolution strategies:

1. **Latest Wins**: Use the entry with the most recent timestamp
2. **Remote Wins**: Always prefer the remote entry
3. **Local Wins**: Always prefer the local entry
4. **Custom Resolution**: Use a custom resolution function

The default strategy is "Latest Wins", but you can implement custom resolution logic in your `SyncProtocol` implementation.

## Security Considerations

The synchronization system includes several security features:

- Content hash verification to ensure data integrity
- Authentication between peers
- Authorization checks for sync operations
- Rate limiting to prevent DoS attacks

When implementing a custom `SyncProtocol`, be sure to include appropriate security measures for your specific deployment environment.

## Best Practices

### Sync Frequency

Choose an appropriate sync interval based on your application's requirements. More frequent synchronization ensures faster consistency but increases network traffic and processing load.

### Batching

Use appropriate batch sizes to balance between network efficiency and memory usage. Larger batches are more efficient for network transfer but require more memory.

### Monitoring

Monitor sync operations using the event subscription system to detect and address synchronization issues promptly.

```rust
// Monitor sync statistics
let peers = sync_manager.list_peers()?;
for peer in peers {
    println!("Peer: {}", peer.peer_id);
    println!("Successful syncs: {}", peer.successful_syncs);
    println!("Failed syncs: {}", peer.failed_syncs);
    println!("Last sync: {:?}", peer.last_sync);
}
```

### Conflict Handling

Decide on an appropriate conflict resolution strategy based on your application's semantics. For some applications, "Latest Wins" is appropriate, while others may require more sophisticated conflict resolution.

## Example Integration

```rust
// Initialize the sync system
let storage = Arc::new(FileLogStorage::new("/path/to/logs"));
let protocol = Arc::new(HttpSyncProtocol::new("https://my-operator.example.com/sync", 30));
let config = SyncConfig::default();

let mut sync_manager = SyncManager::new(config, storage, protocol);
sync_manager.start()?;

// Add peers
for peer_info in peer_discovery_service.get_peers()? {
    sync_manager.add_peer(peer_info)?;
}

// Start the application
let app = MyApplication::new();
app.run().await?;

// Manually trigger sync when needed
sync_manager.sync_all()?;

// Clean shutdown
sync_manager.stop()?;
```

## Troubleshooting

### Common Issues

- **Sync Timeouts**: Check network connectivity and increase the `sync_timeout` value.
- **Hash Verification Failures**: This may indicate data corruption or tampering. Check log integrity.
- **High Conflict Rate**: Review your application's log writing patterns to minimize conflicts.

### Logging and Debugging

The synchronization system integrates with the standard Rust `log` crate for detailed logging. Enable debug or trace level logging to see detailed information about sync operations:

```rust
// Initialize logging
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("causality::log::sync=debug")).init();
```

## Conclusion

The P2P log synchronization system provides a robust and efficient mechanism for maintaining consistent logs across a network of Causality operators. By leveraging differential synchronization and conflict resolution, the system minimizes network traffic while ensuring data consistency. 