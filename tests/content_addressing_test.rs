use causality::log::entry::{EntryType, EventSeverity, LogEntry};
use causality::types::{DomainId, ResourceId};
use uuid::Uuid;

/// Tests for content addressing implementation in LogEntry
#[test]
fn test_log_entry_hash_generation() {
    // Create resource and domain IDs for testing
    let resource_id = ResourceId::new(Uuid::new_v4().to_string());
    let domain_id = DomainId::new(Uuid::new_v4().to_string());

    // Create a log entry
    let mut entry = LogEntry::new_event(
        &resource_id,
        &domain_id,
        "test-content",
        EventSeverity::Info,
        "Test Entry",
        None,
    );

    // Verify the entry has a hash and that it's valid
    assert!(entry.entry_hash.is_some());
    assert!(entry.verify_hash());

    // Store the original hash
    let original_hash = entry.entry_hash.clone();

    // Modify the entry
    entry
        .metadata
        .insert("new_field".to_string(), "value".to_string());

    // Hash should now be invalid
    assert!(!entry.verify_hash());
    assert_ne!(entry.entry_hash, original_hash);

    // Regenerate the hash
    entry.generate_hash();

    // Hash should be valid again
    assert!(entry.verify_hash());
}

/// Tests for the factory methods that create entries with hashes
#[test]
fn test_entry_factory_methods() {
    // Create resource and domain IDs for testing
    let resource_id = ResourceId::new(Uuid::new_v4().to_string());
    let domain_id = DomainId::new(Uuid::new_v4().to_string());

    // Test event entry creation
    let event_entry = LogEntry::new_event(
        &resource_id,
        &domain_id,
        "test-event",
        EventSeverity::Info,
        "Test Event Entry",
        None,
    );
    assert!(event_entry.entry_hash.is_some());
    assert!(event_entry.verify_hash());

    // Test fact entry creation
    let fact_entry = LogEntry::new_fact(
        &resource_id,
        &domain_id,
        "test-fact",
        "Test Fact Entry",
        None,
    );
    assert!(fact_entry.entry_hash.is_some());
    assert!(fact_entry.verify_hash());

    // Test effect entry creation
    let effect_entry = LogEntry::new_effect(
        &resource_id,
        &domain_id,
        "test-effect",
        "Test Effect Entry",
        None,
    );
    assert!(effect_entry.entry_hash.is_some());
    assert!(effect_entry.verify_hash());
}
