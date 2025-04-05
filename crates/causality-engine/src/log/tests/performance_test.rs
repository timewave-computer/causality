// Tests for the async functionality of the OptimizedLogStorage

use super::super::performance::{BatchConfig, OptimizedLogStorage};
use crate::log::{EntryData, EntryType, LogEntry, LogStorage, MemoryLogStorage};
use causality_types::Timestamp;
use std::collections::HashMap;
use tokio::time::Duration;

#[tokio::test]
async fn test_async_flush() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test with small batch size to ensure we hit async_flush
    let memory_storage = MemoryLogStorage::new();
    let config = BatchConfig {
        max_batch_size: 5,
        flush_interval_ms: 100,
        compress_batches: false,
        compression_level: 0,
    };
    
    let optimized = OptimizedLogStorage::new(memory_storage, Some(config)).await?;
    
    // Add entries below batch size threshold
    for i in 0..3 {
        let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
        optimized.append_entry(entry).await?;
    }
    
    // Should be buffered, not in storage yet
    assert_eq!(optimized.storage.get_entry_count().await?, 0);
    
    // Explicitly call async_flush
    optimized.async_flush().await?;
    
    // Now they should be flushed to storage
    assert_eq!(optimized.storage.get_entry_count().await?, 3);
    
    // Add more entries to automatically trigger a flush
    for i in 3..8 {
        let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
        optimized.append_entry(entry).await?;
    }
    
    // Last entries should have triggered flush
    assert_eq!(optimized.storage.get_entry_count().await?, 8);
    
    // Test background flusher
    let config = BatchConfig {
        max_batch_size: 10, // Bigger batch size to avoid auto-flush
        flush_interval_ms: 100, // Short interval
        compress_batches: false,
        compression_level: 0,
    };
    
    let optimized = OptimizedLogStorage::new(memory_storage, Some(config)).await?;
    
    // Start background flusher
    let _flusher = optimized.start_background_flush()?;
    
    // Add some entries
    for i in 0..3 {
        let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
        optimized.append_entry(entry).await?;
    }
    
    // Wait for background flush
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Entries should be flushed by background task
    assert_eq!(optimized.storage.get_entry_count().await?, 3);
    
    Ok(())
}

// Helper to create a test entry
fn create_test_entry(entry_type: EntryType, timestamp: Timestamp) -> LogEntry {
    LogEntry {
        id: format!("test-entry-{}", rand::random::<u32>()),
        timestamp,
        entry_type,
        data: EntryData::Custom(serde_json::json!({
            "test": "data"
        })),
        trace_id: None,
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: Some(format!("hash-{}", rand::random::<u32>())),
    }
} 