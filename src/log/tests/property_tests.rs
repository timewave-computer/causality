//! Property-based tests for the Causality log system
//!
//! This module contains property-based tests that verify invariants of the log system.
//! These tests use the proptest crate to generate random inputs and ensure the system
//! behaves correctly under a wide range of conditions.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use proptest::prelude::*;
use proptest::collection::{vec, hash_map};
use proptest::option;

use crate::error::Result;
use crate::log::entry::{EntryType, EntryData, LogEntry, EventEntry, EventSeverity, EffectEntry, FactEntry};
use crate::log::storage::{LogStorage, MemoryLogStorage};
use crate::types::{ResourceId, DomainId};

/// Strategies for generating test data

/// Strategy for generating timestamps
fn timestamp_strategy() -> impl Strategy<Value = u64> {
    // Generate plausible timestamps (between 2020 and 2030)
    (1577836800u64..1893456000u64)
}

/// Strategy for generating event severities
fn severity_strategy() -> impl Strategy<Value = EventSeverity> {
    prop_oneof![
        Just(EventSeverity::Debug),
        Just(EventSeverity::Info),
        Just(EventSeverity::Warning),
        Just(EventSeverity::Error),
        Just(EventSeverity::Critical)
    ]
}

/// Strategy for generating resource IDs
fn resource_id_strategy() -> impl Strategy<Value = ResourceId> {
    (0..100u64).prop_map(|id| ResourceId::new(id.to_string()))
}

/// Strategy for generating domain IDs
fn domain_id_strategy() -> impl Strategy<Value = DomainId> {
    (0..20u64).prop_map(|id| DomainId::new(id.to_string()))
}

/// Strategy for generating entry types
fn entry_type_strategy() -> impl Strategy<Value = EntryType> {
    prop_oneof![
        Just(EntryType::Event),
        Just(EntryType::Effect),
        Just(EntryType::Fact)
    ]
}

/// Strategy for generating event entries
fn event_entry_strategy() -> impl Strategy<Value = EventEntry> {
    (
        any::<String>().prop_filter_map("Empty strings not allowed", |s| {
            if s.is_empty() { None } else { Some(s) }
        }),
        severity_strategy(),
        any::<String>().prop_filter_map("Empty strings not allowed", |s| {
            if s.is_empty() { None } else { Some(s) }
        }),
        option::of(vec(resource_id_strategy(), 0..5)),
        option::of(vec(domain_id_strategy(), 0..3))
    ).prop_map(|(event_name, severity, component, resources, domains)| {
        EventEntry {
            event_name,
            severity,
            component,
            details: serde_json::json!({"test": "value"}),
            resources,
            domains,
        }
    })
}

/// Strategy for generating effect entries
fn effect_entry_strategy() -> impl Strategy<Value = EffectEntry> {
    (
        vec(resource_id_strategy(), 1..5),
        vec(domain_id_strategy(), 1..3)
    ).prop_map(|(resources, domains)| {
        EffectEntry {
            effect_type: crate::log::EffectType::Create,
            resources,
            domains,
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }
    })
}

/// Strategy for generating fact entries
fn fact_entry_strategy() -> impl Strategy<Value = FactEntry> {
    (
        domain_id_strategy(),
        (0..1000u64),
        option::of(any::<String>()),
        (0..100000u64),
        any::<String>().prop_filter_map("Empty strings not allowed", |s| {
            if s.is_empty() { None } else { Some(s) }
        }),
        vec(resource_id_strategy(), 1..5)
    ).prop_map(|(domain, block_height, block_hash, observed_at, fact_type, resources)| {
        FactEntry {
            domain,
            block_height,
            block_hash,
            observed_at,
            fact_type,
            resources,
            data: serde_json::json!({"test": "value"}),
            verified: true,
        }
    })
}

/// Strategy for generating log entries
fn log_entry_strategy() -> impl Strategy<Value = LogEntry> {
    (
        any::<String>().prop_filter_map("Empty strings not allowed", |s| {
            if s.is_empty() { None } else { Some(s) }
        }),
        timestamp_strategy(),
        entry_type_strategy(),
        option::of(any::<String>()),
        option::of(any::<String>()),
        hash_map(any::<String>(), any::<String>(), 0..5)
    ).prop_flat_map(|(id, timestamp, entry_type, trace_id, parent_id, metadata)| {
        let data_strategy = match entry_type {
            EntryType::Event => event_entry_strategy()
                .prop_map(EntryData::Event)
                .boxed(),
            EntryType::Effect => effect_entry_strategy()
                .prop_map(EntryData::Effect)
                .boxed(),
            EntryType::Fact => fact_entry_strategy()
                .prop_map(EntryData::Fact)
                .boxed(),
        };
        
        data_strategy.prop_map(move |data| {
            LogEntry {
                id: id.clone(),
                entry_type,
                timestamp,
                data,
                trace_id: trace_id.clone(),
                parent_id: parent_id.clone(),
                metadata: metadata.clone(),
                entry_hash: None,
            }
        })
    })
}

/// Property-based tests for log storage operations
proptest! {
    /// Test that appending and reading entries preserves all entry data
    #[test]
    fn test_append_read_preserves_data(entries in vec(log_entry_strategy(), 1..100)) -> Result<()> {
        let storage = MemoryLogStorage::new();
        
        // Append all entries
        for entry in &entries {
            storage.append(entry.clone())?;
        }
        
        // Check entry count
        prop_assert_eq!(storage.entry_count()?, entries.len());
        
        // Read all entries
        let read_entries = storage.read(0, entries.len())?;
        
        // Verify all entries match
        for (original, read) in entries.iter().zip(read_entries.iter()) {
            prop_assert_eq!(original.id, read.id);
            prop_assert_eq!(original.timestamp, read.timestamp);
            prop_assert_eq!(original.entry_type, read.entry_type);
            prop_assert_eq!(original.trace_id, read.trace_id);
            prop_assert_eq!(original.parent_id, read.parent_id);
            
            // Compare JSON representation of data since the internal structure might differ slightly
            let original_data_json = serde_json::to_value(&original.data)?;
            let read_data_json = serde_json::to_value(&read.data)?;
            prop_assert_eq!(original_data_json, read_data_json);
        }
        
        Ok(())
    }
    
    /// Test that reading with offsets and limits works correctly
    #[test]
    fn test_read_with_offset_and_limit(
        entries in vec(log_entry_strategy(), 10..50),
        offset in 0usize..10,
        limit in 1usize..20
    ) -> Result<()> {
        let storage = MemoryLogStorage::new();
        
        // Append all entries
        for entry in &entries {
            storage.append(entry.clone())?;
        }
        
        // Read with offset and limit
        let read_entries = storage.read(offset, limit)?;
        
        // Calculate expected number of entries
        let expected_count = std::cmp::min(limit, entries.len().saturating_sub(offset));
        
        // Verify count
        prop_assert_eq!(read_entries.len(), expected_count);
        
        // Verify entries match
        if offset < entries.len() {
            for i in 0..read_entries.len() {
                prop_assert_eq!(entries[offset + i].id, read_entries[i].id);
            }
        }
        
        Ok(())
    }
    
    /// Test that content addressing hash verification works correctly
    #[test]
    fn test_content_hash_verification(entries in vec(log_entry_strategy(), 1..20)) -> Result<()> {
        let storage = MemoryLogStorage::new_with_config(
            crate::log::storage::StorageConfig {
                verify_hashes: true,
                enforce_hash_verification: true,
                ..Default::default()
            }
        );
        
        // Append all entries, which should generate and set hashes
        for entry in &entries {
            let mut entry_clone = entry.clone();
            entry_clone.entry_hash = None; // Ensure we start with no hash
            storage.append(entry_clone)?;
        }
        
        // Read all entries
        let read_entries = storage.read(0, entries.len())?;
        
        // Verify all entries have valid hashes
        for entry in &read_entries {
            prop_assert!(entry.entry_hash.is_some());
            prop_assert!(crate::log::entry::verify_entry_hash(entry)?);
        }
        
        Ok(())
    }
}

/// Manual test for entry hash tampering detection
#[test]
fn test_hash_tampering_detection() -> Result<()> {
    let storage = MemoryLogStorage::new_with_config(
        crate::log::storage::StorageConfig {
            verify_hashes: true,
            enforce_hash_verification: true,
            ..Default::default()
        }
    );
    
    // Create a valid entry
    let mut entry = LogEntry {
        id: "test_entry".to_string(),
        timestamp: Utc::now().timestamp() as u64,
        entry_type: EntryType::Event,
        data: EntryData::Event(EventEntry {
            event_name: "test_event".to_string(),
            severity: EventSeverity::Info,
            component: "test".to_string(),
            details: serde_json::json!({"test": "value"}),
            resources: None,
            domains: None,
        }),
        trace_id: None,
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: None,
    };
    
    // Append the entry, which should calculate and set the hash
    storage.append(entry.clone())?;
    
    // Read the entry back
    let read_entries = storage.read(0, 1)?;
    let mut read_entry = read_entries[0].clone();
    
    // Tamper with the entry data but keep the hash the same
    if let EntryData::Event(ref mut event) = read_entry.data {
        event.event_name = "tampered_event".to_string();
    }
    
    // Verifying the tampered entry should fail
    assert!(!crate::log::entry::verify_entry_hash(&read_entry)?);
    
    // Attempting to append the tampered entry should fail
    let result = storage.append(read_entry);
    assert!(result.is_err());
    
    Ok(())
} 