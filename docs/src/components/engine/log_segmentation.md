<!-- Segmentation of logs -->
<!-- Original file: docs/src/log_segmentation.md -->

# Log Segmentation in Causality

This document describes the implementation of log segmentation in the Causality Unified Log System.

## Overview

Log segmentation is a critical feature of the Causality log system that partitions log entries into manageable segments. This approach offers several benefits:

1. **Performance**: By working with smaller chunks of data, read and write operations remain efficient even as the log grows to millions of entries.
2. **Maintenance**: Individual segments can be archived, compressed, or pruned without affecting the entire log.
3. **Scalability**: The system can handle logs of any size, limited only by available disk space.
4. **Query Efficiency**: Time-based and content-based queries can focus on relevant segments, improving search performance.

## Core Components

### LogSegment

The `LogSegment` represents a single chunk of log entries with metadata:

```rust
pub struct LogSegment {
    /// The segment info
    info: SegmentInfo,
    /// The entries in this segment
    entries: Vec<LogEntry>,
    /// Whether this segment has been modified since loading
    modified: bool,
    /// Additional metadata for the segment
    metadata: HashMap<String, String>,
}
```

Each segment maintains:
- Unique ID and path on disk
- Creation timestamp and time range
- Read-only status
- Entry count and size metrics

### SegmentManagerOptions

Configurable options for log segmentation:

```rust
pub struct SegmentManagerOptions {
    /// Base directory for storing segments
    pub base_dir: PathBuf,
    /// Maximum number of active segments in memory
    pub max_active_segments: usize,
    /// Whether to compress inactive segments
    pub compress_inactive: bool,
    /// Rotation criteria
    pub rotation_criteria: Vec<RotationCriteria>,
    /// Segment naming pattern
    pub segment_name_pattern: String,
    /// Whether to auto-flush on rotation
    pub auto_flush: bool,
    /// Segment index directory
    pub index_dir: Option<PathBuf>,
}
```

### RotationCriteria

Conditions that trigger segment rotation:

```rust
pub enum RotationCriteria {
    /// Rotate based on entry count
    EntryCount(usize),
    /// Rotate based on segment size in bytes
    Size(u64),
    /// Rotate based on time interval
    TimeInterval(Duration),
    /// Custom rotation function
    Custom(Box<dyn Fn(&LogSegment) -> bool + Send + Sync>),
}
```

### LogSegmentManager

The central component that manages multiple segments:

```rust
pub struct LogSegmentManager {
    /// Options for the segment manager
    options: SegmentManagerOptions,
    /// Currently active segment for writing
    active_segment: Arc<Mutex<LogSegment>>,
    /// Cached segments (recently used)
    cached_segments: Arc<RwLock<HashMap<String, Arc<Mutex<LogSegment>>>>>,
    /// Index of all segments
    segment_index: Arc<RwLock<BTreeMap<Timestamp, SegmentIndexEntry>>>,
    /// Storage configuration
    storage_config: StorageConfig,
    /// Last rotation timestamp
    last_rotation: Arc<Mutex<DateTime<Utc>>>,
}
```

## How It Works

### 1. Segment Creation and Rotation

When a log system initializes, it creates an active segment for writing. As log entries are appended, the `LogSegmentManager` checks rotation criteria after each write. When a threshold is met (size, count, or time), the manager:

1. Creates a new active segment
2. Marks the old segment as read-only
3. Adds metadata to the segment index for efficient lookups
4. Manages cached segments to control memory usage

```rust
// Check if we need to rotate
self.check_rotation()?;

// Append to the active segment
let mut active = self.active_segment.lock()?;
active.append(entry)?;
```

### 2. Segment Caching

To balance performance and memory usage, the system maintains a configurable cache of recently used segments:

```rust
fn manage_cache(&self) -> Result<()> {
    // If the cache is not over limit, do nothing
    if cached.len() <= self.options.max_active_segments {
        return Ok(());
    }
    
    // Keep the most recently used segments
    let to_keep = segments.split_off(segments.len() - self.options.max_active_segments);
    
    // Re-insert the segments to keep
    for (id, segment) in to_keep {
        cached.insert(id, segment);
    }
    
    Ok(())
}
```

### 3. Time-Based Queries

Time-based queries select segments that might contain entries in the specified time range before filtering entries:

```rust
pub fn get_entries_in_range(
    &self,
    start_time: Timestamp,
    end_time: Timestamp
) -> Result<Vec<LogEntry>> {
    // Get the relevant segments
    let segments = self.get_segments_in_range(start_time, end_time)?;
    
    // Filter entries within each segment
    for segment_arc in segments {
        for entry in segment.entries() {
            if entry.timestamp >= start_time && entry.timestamp <= end_time {
                entries.push(entry.clone());
            }
        }
    }
    
    // Sort by timestamp
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    Ok(entries)
}
```

### 4. Segment Merging

For maintenance purposes, segments can be merged to consolidate log data:

```rust
pub fn merge_segments(&self, segment_ids: &[String]) -> Result<String> {
    // Load all segments to merge
    for id in segment_ids {
        if let Some(segment_arc) = self.get_segment(id)? {
            entries.extend(segment.entries().iter().cloned());
        }
    }
    
    // Sort entries by timestamp
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    // Create the merged segment
    let mut merged_segment = LogSegment::new(new_id.clone());
    
    // Add entries to the merged segment
    for entry in entries {
        merged_segment.append(entry)?;
    }
    
    Ok(new_id)
}
```

## Configuration Examples

### Default Configuration

```rust
let options = SegmentManagerOptions::default();
// Results in:
// - 10,000 entries per segment
// - 10MB maximum segment size
// - Daily rotation
// - 2 segments in memory cache
```

### High-Performance Configuration

```rust
let options = SegmentManagerOptions {
    max_active_segments: 10, // Keep more segments in memory
    rotation_criteria: vec![
        RotationCriteria::Size(100 * 1024 * 1024), // 100MB segments
    ],
    auto_flush: false, // Manual flushing for better throughput
    ..Default::default()
};
```

### Time-Sensitive Configuration

```rust
let options = SegmentManagerOptions {
    rotation_criteria: vec![
        RotationCriteria::TimeInterval(Duration::hours(1)), // Hourly segments
    ],
    segment_name_pattern: "log_{timestamp}_{hour}", // Custom naming
    ..Default::default()
};
```

## Integration with Storage

The segment manager works with the underlying storage system to persist segments:

1. Active segments are kept in memory for fast writes
2. Inactive segments are flushed to disk and loaded on demand
3. The segment index provides metadata for efficient lookups

```rust
// Flush all active segments to disk
pub fn flush(&self) -> Result<()> {
    // Flush the active segment
    let mut active = self.active_segment.lock()?;
    active.flush()?;
    
    // Flush all cached segments
    for (_, segment) in cached.iter() {
        let mut segment = segment.lock()?;
        segment.flush()?;
    }
    
    Ok(())
}
```

## Benefits

1. **Scalability**: The system can handle logs of any size by rotating segments.
2. **Performance**: Read and write operations remain efficient even with large logs.
3. **Maintenance**: Individual segments can be archived, compressed, or pruned.
4. **Query Efficiency**: Time-based and content-based queries focus on relevant segments.
5. **Resource Management**: Memory usage is controlled through caching and rotation.

## Usage Example

```rust
// Create a segment manager
let options = SegmentManagerOptions {
    base_dir: PathBuf::from("logs"),
    max_active_segments: 5,
    rotation_criteria: vec![
        RotationCriteria::EntryCount(5000),
        RotationCriteria::TimeInterval(Duration::days(1)),
    ],
    ..Default::default()
};

let manager = LogSegmentManager::new(options, storage_config)?;

// Append log entries
manager.append(log_entry)?;

// Query entries in a time range
let entries = manager.get_entries_in_range(start_time, end_time)?;

// Merge segments for maintenance
let new_segment_id = manager.merge_segments(&["segment1", "segment2"])?;

// Flush and close
manager.flush()?;
manager.close()?;
``` 