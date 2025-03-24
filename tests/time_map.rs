use causality::crypto::{
    ContentAddressed, ContentId, HashOutput, HashAlgorithm, HashFactory
};
use causality::content_addressed_storage::{
    ContentAddressedStorage, StorageFactory
};
use causality::domain::map::content_addressed_time_map::{
    ContentAddressedTimeMap, ContentAddressedTimeMapEntry, SharedContentAddressedTimeMap
};
use causality::domain::map::TimeRange;
use causality::types::Timestamp;
use std::collections::HashMap;
use chrono::Utc;

#[test]
fn test_time_map_entry_content_addressing() {
    // Create a time map entry
    let entry = ContentAddressedTimeMapEntry::new(
        "test-domain".to_string(),
        100,
        "0xabc123".to_string(),
        Timestamp(1625097600),
        "test",
    )
    .with_confidence(0.95)
    .with_verification(true)
    .with_metadata("source_url", "https://example.com/api");
    
    // Get its content hash
    let hash = entry.content_hash();
    
    // Verify it's properly content-addressed
    assert!(entry.verify());
    
    // Test serialization
    let bytes = entry.to_bytes();
    let deserialized = ContentAddressedTimeMapEntry::from_bytes(&bytes).unwrap();
    
    // Verify the deserialized entry matches the original
    assert_eq!(deserialized.domain_id, entry.domain_id);
    assert_eq!(deserialized.height, entry.height);
    assert_eq!(deserialized.hash, entry.hash);
    assert_eq!(deserialized.timestamp, entry.timestamp);
    assert_eq!(deserialized.confidence, entry.confidence);
    assert_eq!(deserialized.verified, entry.verified);
    assert_eq!(deserialized.source, entry.source);
    assert_eq!(deserialized.metadata, entry.metadata);
    
    // Verify the deserialized entry has the same content hash
    assert_eq!(deserialized.content_hash(), hash);
}

#[test]
fn test_content_addressed_time_map() {
    // Create a time map
    let mut time_map = ContentAddressedTimeMap::new();
    
    // Create time map entries
    let entry1 = ContentAddressedTimeMapEntry::new(
        "domain1".to_string(),
        100,
        "0xabc123".to_string(),
        Timestamp(1625097600),
        "test",
    );
    
    let entry2 = ContentAddressedTimeMapEntry::new(
        "domain2".to_string(),
        200,
        "0xdef456".to_string(),
        Timestamp(1625184000),
        "test",
    );
    
    // Add entries to the time map
    time_map.update_domain(&entry1);
    time_map.update_domain(&entry2);
    
    // Check the time map state
    assert_eq!(time_map.len(), 2);
    assert!(time_map.contains_domain(&"domain1".to_string()));
    assert!(time_map.contains_domain(&"domain2".to_string()));
    
    // Get all domains
    let domains = time_map.domains();
    assert_eq!(domains.len(), 2);
    assert!(domains.contains(&&"domain1".to_string()));
    assert!(domains.contains(&&"domain2".to_string()));
    
    // Remove a domain
    let removed = time_map.remove_domain(&"domain1".to_string());
    assert!(removed);
    assert_eq!(time_map.len(), 1);
    assert!(!time_map.contains_domain(&"domain1".to_string()));
    
    // Content addressing
    let hash = time_map.content_hash();
    assert!(time_map.verify());
    
    // Serialization
    let bytes = time_map.to_bytes();
    let deserialized = ContentAddressedTimeMap::from_bytes(&bytes).unwrap();
    
    assert_eq!(deserialized.len(), time_map.len());
    assert_eq!(deserialized.contains_domain(&"domain2".to_string()), 
              time_map.contains_domain(&"domain2".to_string()));
    assert_eq!(deserialized.content_hash(), hash);
}

#[test]
fn test_shared_time_map() {
    // Create storage
    let storage = StorageFactory::create_memory_storage();
    
    // Create shared time map
    let time_map = SharedContentAddressedTimeMap::new(storage);
    
    // Create time map entries
    let entry1 = ContentAddressedTimeMapEntry::new(
        "domain1".to_string(),
        100,
        "0xabc123".to_string(),
        Timestamp(1625097600),
        "test",
    );
    
    let entry2 = ContentAddressedTimeMapEntry::new(
        "domain2".to_string(),
        200,
        "0xdef456".to_string(),
        Timestamp(1625184000),
        "test",
    );
    
    let entry3 = ContentAddressedTimeMapEntry::new(
        "domain3".to_string(),
        300,
        "0xghi789".to_string(),
        Timestamp(1625270400),
        "test",
    );
    
    // Add entries
    let id1 = time_map.update_domain(entry1.clone()).unwrap();
    let id2 = time_map.update_domain(entry2.clone()).unwrap();
    let id3 = time_map.update_domain(entry3.clone()).unwrap();
    
    // Retrieve entries
    let retrieved1 = time_map.get_entry(&"domain1".to_string()).unwrap();
    let retrieved2 = time_map.get_entry(&"domain2".to_string()).unwrap();
    let retrieved3 = time_map.get_entry(&"domain3".to_string()).unwrap();
    
    assert_eq!(retrieved1.height, 100);
    assert_eq!(retrieved2.height, 200);
    assert_eq!(retrieved3.height, 300);
    
    // Test time range query
    let range = TimeRange::new(
        Timestamp(1625097500),
        Timestamp(1625200000), // Includes domain1 and domain2
    );
    
    let results = time_map.query_by_time(&range).unwrap();
    assert_eq!(results.len(), 2);
    
    // Check that the right domains are returned
    let domain_ids: Vec<String> = results.iter()
        .map(|e| e.domain_id.clone())
        .collect();
    
    assert!(domain_ids.contains(&"domain1".to_string()));
    assert!(domain_ids.contains(&"domain2".to_string()));
    assert!(!domain_ids.contains(&"domain3".to_string()));
    
    // Test storing the time map
    let map_id = time_map.store().unwrap();
    assert!(!map_id.to_string().is_empty());
    
    // Test getting all entries
    let all_entries = time_map.get_all_entries().unwrap();
    assert_eq!(all_entries.len(), 3);
}

#[test]
fn test_time_map_updates() {
    // Create storage
    let storage = StorageFactory::create_memory_storage();
    
    // Create shared time map
    let time_map = SharedContentAddressedTimeMap::new(storage);
    
    // Create initial entry
    let entry = ContentAddressedTimeMapEntry::new(
        "domain1".to_string(),
        100,
        "0xabc123".to_string(),
        Timestamp(1625097600),
        "test",
    );
    
    // Add entry
    time_map.update_domain(entry.clone()).unwrap();
    
    // Create updated entry
    let updated_entry = ContentAddressedTimeMapEntry::new(
        "domain1".to_string(),
        101,
        "0xabc124".to_string(),
        Timestamp(1625097900),
        "test",
    );
    
    // Update entry
    time_map.update_domain(updated_entry.clone()).unwrap();
    
    // Retrieve entry
    let retrieved = time_map.get_entry(&"domain1".to_string()).unwrap();
    
    // Should be the updated entry
    assert_eq!(retrieved.height, 101);
    assert_eq!(retrieved.hash, "0xabc124".to_string());
    assert_eq!(retrieved.timestamp, Timestamp(1625097900));
} 