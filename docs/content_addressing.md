# Content Addressing for Unified Log System

This document outlines the implementation of content addressing for the Causality Unified Log System, as specified in ADR 008 (Unified Log System) and ADR 006 (Content Addressing).

## Overview

Content addressing has been implemented in the unified log system to ensure the integrity and immutability of log entries. This implementation allows any log entry to be uniquely identified by its content hash, enabling verification that the entry has not been tampered with.

## Implementation Details

### 1. Log Entry Hash Field

Each `LogEntry` now includes an optional `entry_hash` field, which contains a cryptographic hash of the entry's content. The hash is generated using the Blake3 algorithm, which provides fast and secure cryptographic hashing.

```rust
pub struct LogEntry {
    // ... existing fields ...
    
    /// Content hash for entry verification
    pub entry_hash: Option<String>,
}
```

### 2. Hash Generation and Verification

Two primary methods have been added to the `LogEntry` struct to handle content addressing:

#### Generate Hash

```rust
pub fn generate_hash(&mut self) {
    // Generate a hash of the entry content using Blake3
    let mut hasher = blake3::Hasher::new();
    
    // Hash all relevant fields except the entry_hash itself
    // ... hashing logic ...
    
    self.entry_hash = Some(hash);
}
```

#### Verify Hash

```rust
pub fn verify_hash(&self) -> bool {
    if let Some(hash) = &self.entry_hash {
        // Generate a fresh hash of the current entry content
        // Compare it with the stored hash
        // Return true if they match
    } else {
        false // No hash to verify
    }
}
```

### 3. Factory Methods

Factory methods for creating log entries have been updated to automatically generate content hashes:

```rust
pub fn new_with_hash(
    id: String,
    timestamp: DateTime<Utc>,
    entry_type: EntryType,
    data: EntryData,
    trace_id: Option<String>,
    parent_id: Option<String>,
) -> Self {
    let mut entry = LogEntry {
        id,
        timestamp,
        entry_type,
        data,
        trace_id,
        parent_id,
        metadata: HashMap::new(),
        entry_hash: None,
    };
    
    entry.generate_hash();
    entry
}
```

Simplified factory methods were also created:
- `new_event`: Creates an event entry with a hash
- `new_fact`: Creates a fact entry with a hash
- `new_effect`: Creates an effect entry with a hash

### 4. Storage Integration

The `LogStorage` trait has been extended with methods to handle hash verification:

```rust
fn verify_entry_hash(&self, entry: &LogEntry, config: &StorageConfig) -> Result<()> {
    if !config.enforce_hash_verification {
        return Ok(());
    }
    
    if let Some(_) = &entry.entry_hash {
        if !entry.verify_hash() {
            return Err(Error::InvalidHash("Entry hash verification failed".to_string()));
        }
    } else if config.enforce_hash_verification {
        return Err(Error::InvalidHash("Entry is missing a hash".to_string()));
    }
    
    Ok(())
}

fn ensure_valid_hash(&self, entry: &mut LogEntry) -> Result<()> {
    if entry.entry_hash.is_none() || !entry.verify_hash() {
        entry.generate_hash();
    }
    Ok(())
}
```

Both `MemoryLogStorage` and `FileLogStorage` implementations have been updated to verify hashes before storing entries and to ensure entries have valid hashes when required by the configuration.

### 5. Configuration

The `StorageConfig` struct now includes an `enforce_hash_verification` field which controls whether entries must have valid hashes to be stored. This defaults to `true` but can be configured as needed:

```rust
pub struct StorageConfig {
    // ... existing fields ...
    
    /// Whether to enforce hash verification for entries
    pub enforce_hash_verification: bool,
}
```

## Usage Examples

### Creating a Log Entry with Hash

```rust
let event_entry = LogEntry::new_event(
    &resource_id,
    &domain_id,
    "test-event",
    EventSeverity::Info,
    "Test Event",
    None,
);

// The entry automatically has a hash
assert!(event_entry.entry_hash.is_some());
assert!(event_entry.verify_hash());
```

### Verifying Hash After Modification

```rust
let mut entry = LogEntry::new_event(...);

// Modify the entry
entry.metadata.insert("new_field".to_string(), "value".to_string());

// Hash is now invalid
assert!(!entry.verify_hash());

// Regenerate the hash
entry.generate_hash();

// Hash is valid again
assert!(entry.verify_hash());
```

## Benefits

1. **Data Integrity**: Ensures log entries are not tampered with
2. **Immutability**: Changes to entries are detectable
3. **Verification**: Actors can verify the authenticity of logs
4. **Distributed Validation**: Enables independent verification in distributed systems

## Conclusion

Content addressing is now integrated into the unified log system, providing a foundation for immutable, verifiable logs. This implementation satisfies the requirements outlined in the architecture decision records and provides a basis for further features such as log sharing between actors and log reconstruction. 