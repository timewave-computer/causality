// Tests for the Causality Unified Log System

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

use causality::error::Result;
use causality::log::{
    EffectEntry, EntryData, EntryType, EventEntry, EventSeverity, FactEntry, FileLogStorage,
    LogEntry, LogStorage, MemoryLogStorage, ReplayCallback, ReplayEngine, ReplayOptions,
    ReplayResult, StorageConfig,
};
use causality::types::{DomainId, ResourceId};

#[test]
fn test_create_log_entries() {
    // Test creating different types of log entries

    // Create an event entry
    let event_entry = LogEntry {
        id: "event1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Event,
        data: EntryData::Event(EventEntry {
            event_name: "test_event".to_string(),
            severity: EventSeverity::Info,
            component: "test".to_string(),
            details: serde_json::json!({"test": "value"}),
            resources: None,
            domains: None,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: None,
    };

    assert_eq!(event_entry.entry_type, EntryType::Event);
    assert_eq!(event_entry.trace_id, Some("trace1".to_string()));

    // Create a resource ID and domain ID for other entry types
    let resource_id = ResourceId::new("test_resource");
    let domain_id = DomainId::new("test_domain");

    // Create an effect entry
    let effect_entry = LogEntry {
        id: "effect1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Effect,
        data: EntryData::Effect(EffectEntry {
            effect_type: causality::log::EffectType::Create,
            resources: vec![resource_id.clone()],
            domains: vec![domain_id.clone()],
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: None,
    };

    assert_eq!(effect_entry.entry_type, EntryType::Effect);

    // Create a fact entry
    let fact_entry = LogEntry {
        id: "fact1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Fact,
        data: EntryData::Fact(FactEntry {
            domain: domain_id.clone(),
            block_height: 100,
            block_hash: Some("test_hash".to_string()),
            observed_at: 12345,
            fact_type: "test_fact".to_string(),
            resources: vec![resource_id.clone()],
            data: serde_json::json!({"test": "value"}),
            verified: true,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: None,
    };

    assert_eq!(fact_entry.entry_type, EntryType::Fact);
}

#[test]
fn test_memory_log_storage() -> Result<()> {
    // Test the memory log storage implementation

    let storage = MemoryLogStorage::new();

    // Add some test entries
    for i in 0..10 {
        let entry = LogEntry {
            id: format!("entry{}", i),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: format!("test_event{}", i),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"test": format!("value{}", i)}),
                resources: None,
                domains: None,
            }),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
        };

        storage.append(entry)?;
    }

    // Check entry count
    assert_eq!(storage.entry_count()?, 10);

    // Read all entries
    let entries = storage.read(0, 20)?;
    assert_eq!(entries.len(), 10);

    // Read a subset of entries
    let subset = storage.read(3, 4)?;
    assert_eq!(subset.len(), 4);
    assert_eq!(subset[0].id, "entry3");
    assert_eq!(subset[3].id, "entry6");

    // Read out of bounds
    let empty = storage.read(20, 5)?;
    assert_eq!(empty.len(), 0);

    Ok(())
}

#[test]
fn test_file_log_storage() -> Result<()> {
    // Create a temporary directory for the test
    let test_dir = tempdir()?;

    // Create storage config
    let config = StorageConfig::new()
        .with_max_segment_entries(5) // Small for testing
        .with_sync_on_write(true);

    // Create storage
    let storage = FileLogStorage::new(test_dir.path(), config)?;

    // Add some test entries
    for i in 0..12 {
        let entry = LogEntry {
            id: format!("entry{}", i),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: format!("test_event{}", i),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"test": format!("value{}", i)}),
                resources: None,
                domains: None,
            }),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
        };

        storage.append(entry)?;
    }

    // Check entry count
    assert_eq!(storage.entry_count()?, 12);

    // Read all entries
    let entries = storage.read(0, 20)?;
    assert_eq!(entries.len(), 12);

    // Read across segment boundaries
    let subset = storage.read(3, 7)?;
    assert_eq!(subset.len(), 7);
    assert_eq!(subset[0].id, "entry3");
    assert_eq!(subset[6].id, "entry9");

    // Flush to ensure all data is written
    storage.flush()?;

    // Close the storage
    storage.close()?;

    Ok(())
}

#[test]
fn test_replay_engine() -> Result<()> {
    // Create memory storage
    let storage = Arc::new(MemoryLogStorage::new());

    // Create some test resources and domains
    let resource1 = ResourceId::new("resource1");
    let resource2 = ResourceId::new("resource2");
    let domain1 = DomainId::new("domain1");

    // Add some test entries

    // Effect entry
    let effect_entry = LogEntry {
        id: "effect1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Effect,
        data: EntryData::Effect(EffectEntry {
            effect_type: causality::log::EffectType::Create,
            resources: vec![resource1.clone()],
            domains: vec![domain1.clone()],
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
    };

    storage.append(effect_entry)?;

    // Fact entry
    let fact_entry = LogEntry {
        id: "fact1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Fact,
        data: EntryData::Fact(FactEntry {
            domain: domain1.clone(),
            block_height: 100,
            block_hash: Some("test_hash".to_string()),
            observed_at: 12345,
            fact_type: "test_fact".to_string(),
            resources: vec![resource2.clone()],
            data: serde_json::json!({"test": "value"}),
            verified: true,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
    };

    storage.append(fact_entry)?;

    // Create a replay engine
    let engine = ReplayEngine::with_storage(storage.clone());

    // Run replay
    let result = engine.run()?;

    // Verify the result
    assert_eq!(result.status, causality::log::ReplayStatus::Complete);
    assert_eq!(result.processed_entries, 2);
    assert!(result.error.is_none());

    // Check the state
    let state = result.state.unwrap();
    assert_eq!(state.resources.len(), 2);
    assert_eq!(state.domains.len(), 1);
    assert_eq!(state.effects.len(), 1);
    assert_eq!(state.facts.len(), 1);

    // Check specific resource state
    assert!(state.resources.contains_key(&resource1));
    assert!(state.resources.contains_key(&resource2));

    // Check domain state
    let domain_state = state.domains.get(&domain1).unwrap();
    assert_eq!(domain_state.height, 100);
    assert_eq!(domain_state.hash, Some("test_hash".to_string()));

    Ok(())
}

#[test]
fn test_replay_filtering() -> Result<()> {
    // Create memory storage
    let storage = Arc::new(MemoryLogStorage::new());

    // Create two resources
    let resource1 = ResourceId::new("resource1");
    let resource2 = ResourceId::new("resource2");

    // Add entries for resource1
    for i in 0..5 {
        let entry = LogEntry {
            id: format!("resource1_{}", i),
            timestamp: Utc::now(),
            entry_type: EntryType::Effect,
            data: EntryData::Effect(EffectEntry {
                effect_type: causality::log::EffectType::Create,
                resources: vec![resource1.clone()],
                domains: vec![],
                code_hash: None,
                parameters: HashMap::new(),
                result: None,
                success: true,
                error: None,
            }),
            trace_id: Some("trace1".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
        };

        storage.append(entry)?;
    }

    // Add entries for resource2
    for i in 0..5 {
        let entry = LogEntry {
            id: format!("resource2_{}", i),
            timestamp: Utc::now(),
            entry_type: EntryType::Effect,
            data: EntryData::Effect(EffectEntry {
                effect_type: causality::log::EffectType::Create,
                resources: vec![resource2.clone()],
                domains: vec![],
                code_hash: None,
                parameters: HashMap::new(),
                result: None,
                success: true,
                error: None,
            }),
            trace_id: Some("trace2".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
        };

        storage.append(entry)?;
    }

    // Create replay options to filter by resource1
    let mut resources = std::collections::HashSet::new();
    resources.insert(resource1.clone());

    let options = ReplayOptions::new().with_resources(resources);

    // Create a replay engine with the filter
    let engine = ReplayEngine::new(
        storage.clone(),
        options,
        Arc::new(causality::log::NoopReplayCallback),
    );

    // Run replay
    let result = engine.run()?;

    // Verify the result
    assert_eq!(result.processed_entries, 5);

    // Check that only resource1 entries were processed
    let state = result.state.unwrap();
    assert!(state.resources.contains_key(&resource1));
    assert!(!state.resources.contains_key(&resource2));

    Ok(())
}

// Test that checks round-trip serialization of log entries
#[test]
fn test_log_entry_serialization() -> Result<()> {
    let resource_id = ResourceId::new("test_resource");
    let domain_id = DomainId::new("test_domain");

    // Create an effect entry
    let original_entry = LogEntry {
        id: "effect1".to_string(),
        timestamp: Utc::now(),
        entry_type: EntryType::Effect,
        data: EntryData::Effect(EffectEntry {
            effect_type: causality::log::EffectType::Create,
            resources: vec![resource_id.clone()],
            domains: vec![domain_id.clone()],
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }),
        trace_id: Some("trace1".to_string()),
        parent_id: None,
        metadata: HashMap::new(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original_entry)?;

    // Deserialize from JSON
    let deserialized_entry: LogEntry = serde_json::from_str(&json)?;

    // Compare IDs
    assert_eq!(original_entry.id, deserialized_entry.id);
    assert_eq!(original_entry.entry_type, deserialized_entry.entry_type);
    assert_eq!(original_entry.trace_id, deserialized_entry.trace_id);

    // Verify that the structure is properly maintained after serialization
    match deserialized_entry.data {
        EntryData::Effect(effect) => {
            assert!(effect.resources.contains(&resource_id));
            assert!(effect.domains.contains(&domain_id));
            assert_eq!(effect.success, true);
        }
        _ => panic!("Expected Effect entry data"),
    }

    Ok(())
}

#[test]
fn test_log_entry_content_addressing() -> Result<()> {
    // Create a resource ID and domain ID
    let resource_id = ResourceId::new("test_resource");
    let domain_id = DomainId::new("test_domain");

    // Create a new log entry with hash using the factory method
    let mut entry = LogEntry::new_event(
        "test_content_addressing".to_string(),
        EventSeverity::Info,
        "test".to_string(),
        serde_json::json!({"test": "content_addressing"}),
        Some(vec![resource_id.clone()]),
        Some(vec![domain_id.clone()]),
        Some("trace-content-addressing".to_string()),
        None,
    );

    // Verify the entry has a hash
    assert!(entry.entry_hash.is_some());

    // Verify the hash is valid
    assert!(entry.verify_hash());

    // Take a copy of the original hash
    let original_hash = entry.entry_hash.clone();

    // Modify the entry
    entry
        .metadata
        .insert("new_field".to_string(), "new_value".to_string());

    // Hash should now be invalid
    assert!(!entry.verify_hash());
    assert_ne!(entry.entry_hash, original_hash);

    // Regenerate the hash
    entry.generate_hash();

    // Hash should now be valid again
    assert!(entry.verify_hash());
    assert_ne!(entry.entry_hash, original_hash);

    // Let's test a fact entry
    let fact_entry = LogEntry::new_fact(
        domain_id.clone(),
        200,
        Some("fact_hash_test".to_string()),
        98765,
        "content_addressed_fact".to_string(),
        vec![resource_id.clone()],
        serde_json::json!({"test": "fact_content_addressing"}),
        true,
        Some("trace-fact-hash".to_string()),
        None,
    );

    // Verify the fact entry has a valid hash
    assert!(fact_entry.entry_hash.is_some());
    assert!(fact_entry.verify_hash());

    // Let's test an effect entry
    let mut parameters = HashMap::new();
    parameters.insert("test_param".to_string(), serde_json::json!("test_value"));

    let effect_entry = LogEntry::new_effect(
        causality::log::EffectType::Update,
        vec![resource_id.clone()],
        vec![domain_id.clone()],
        Some("test_code_hash".to_string()),
        parameters,
        Some(serde_json::json!({"result": "success"})),
        true,
        None,
        Some("trace-effect-hash".to_string()),
        None,
    );

    // Verify the effect entry has a valid hash
    assert!(effect_entry.entry_hash.is_some());
    assert!(effect_entry.verify_hash());

    Ok(())
}
