use causality::crypto::{
    ContentAddressed, ContentId, HashOutput, HashAlgorithm, HashFactory
};
use causality::content_addressed_storage::{
    ContentAddressedStorage, StorageFactory
};
use causality::log::content_addressed_fact::{
    ContentAddressedFact, FactId, FactContent, FactProof, 
    ContentAddressedFactRegistry, FactRegistryFactory, FactFilter
};
use causality::log::fact_types::{FactType, RegisterFact, ZKProofFact};
use causality::types::Timestamp;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper function to create a test register fact
fn create_test_register_fact() -> ContentAddressedFact {
    ContentAddressedFact::new(
        FactId::generate(),
        FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id: "register123".to_string(),
            initial_data: vec![1, 2, 3, 4],
            owner: "owner1".to_string(),
            domain: "domain1".to_string(),
        }),
        Timestamp(1625097600),
        "test-domain".to_string(),
        FactContent::Binary(vec![0, 1, 2, 3, 4]),
        Some(FactProof::Signature {
            signature: vec![10, 20, 30],
            public_key: vec![40, 50, 60],
        }),
        Vec::new(),
    ).with_metadata("created_by", "test").with_metadata("source", "test_suite")
}

/// Helper function to create a test ZK proof fact
fn create_test_zkproof_fact() -> ContentAddressedFact {
    ContentAddressedFact::new(
        FactId::generate(),
        FactType::ZKProofFact(ZKProofFact::ProofVerification {
            verification_key_id: "key123".to_string(),
            proof_hash: "0xabc123".to_string(),
            public_inputs: vec!["input1".to_string(), "input2".to_string()],
            success: true,
        }),
        Timestamp(1625097700),
        "zk-domain".to_string(),
        FactContent::Json(r#"{"proof": "verified", "timestamp": 1625097700}"#.to_string()),
        None,
        Vec::new(),
    )
}

/// Helper function to create a test block fact
fn create_test_block_fact() -> ContentAddressedFact {
    ContentAddressedFact::new(
        FactId::generate(),
        FactType::BlockFact,
        Timestamp(1625097800),
        "eth-domain".to_string(),
        FactContent::Json(r#"{"block_number": 12345, "block_hash": "0xdef456"}"#.to_string()),
        None,
        Vec::new(),
    )
}

#[test]
fn test_content_addressed_fact_creation() {
    // Create a fact
    let fact = create_test_register_fact();
    
    // Test content addressing
    assert!(fact.verify());
    
    // Check content ID
    let content_id = fact.content_id();
    assert!(!content_id.to_string().is_empty());
    
    // Check fact ID
    assert!(fact.id.as_str().starts_with("fact:"));
    
    // Check metadata
    assert_eq!(fact.metadata.get("created_by"), Some(&"test".to_string()));
    assert_eq!(fact.metadata.get("source"), Some(&"test_suite".to_string()));
}

#[test]
fn test_content_addressed_fact_serialization() {
    // Create a fact
    let fact = create_test_register_fact();
    
    // Compute hash and content ID
    let content_hash = fact.content_hash();
    let content_id = fact.content_id();
    
    // Serialize to bytes
    let bytes = fact.to_bytes();
    
    // Deserialize from bytes
    let deserialized = ContentAddressedFact::from_bytes(&bytes).unwrap();
    
    // Verify the deserialized fact matches the original
    assert_eq!(deserialized.id.as_str(), fact.id.as_str());
    assert_eq!(deserialized.timestamp, fact.timestamp);
    assert_eq!(deserialized.origin_domain, fact.origin_domain);
    
    // Verify content addressing still works
    assert!(deserialized.verify());
    assert_eq!(deserialized.content_hash(), content_hash);
    assert_eq!(deserialized.content_id(), content_id);
    
    // Verify fact proof
    match &deserialized.proof {
        Some(FactProof::Signature { signature, public_key }) => {
            match &fact.proof {
                Some(FactProof::Signature { signature: original_sig, public_key: original_key }) => {
                    assert_eq!(signature, original_sig);
                    assert_eq!(public_key, original_key);
                },
                _ => panic!("Proof mismatch after deserialization"),
            }
        },
        _ => panic!("Missing proof after deserialization"),
    }
    
    // Verify fact type
    match &deserialized.fact_type {
        FactType::RegisterFact(RegisterFact::RegisterCreation { register_id, initial_data, owner, domain }) => {
            assert_eq!(register_id, "register123");
            assert_eq!(initial_data, &vec![1, 2, 3, 4]);
            assert_eq!(owner, "owner1");
            assert_eq!(domain, "domain1");
        },
        _ => panic!("Fact type mismatch after deserialization"),
    }
}

#[test]
fn test_fact_registry_basic() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Create a fact
    let fact = create_test_register_fact();
    let fact_id = fact.id.clone();
    
    // Store the fact
    let content_id = registry.register_fact(fact.clone()).unwrap();
    
    // Retrieve by content ID
    let retrieved = registry.get_fact(&content_id).unwrap();
    assert_eq!(retrieved.id.as_str(), fact.id.as_str());
    
    // Retrieve by fact ID
    let retrieved_by_id = registry.get_fact_by_id(&fact_id).unwrap();
    assert_eq!(retrieved_by_id.id.as_str(), fact.id.as_str());
    
    // Check registry count
    assert_eq!(registry.count(), 1);
}

#[test]
fn test_fact_registry_multiple_facts() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Create multiple facts
    let fact1 = create_test_register_fact();
    let fact2 = create_test_zkproof_fact();
    let fact3 = create_test_block_fact();
    
    // Store the facts
    let content_id1 = registry.register_fact(fact1.clone()).unwrap();
    let content_id2 = registry.register_fact(fact2.clone()).unwrap();
    let content_id3 = registry.register_fact(fact3.clone()).unwrap();
    
    // Verify each fact can be retrieved
    let retrieved1 = registry.get_fact(&content_id1).unwrap();
    let retrieved2 = registry.get_fact(&content_id2).unwrap();
    let retrieved3 = registry.get_fact(&content_id3).unwrap();
    
    assert_eq!(retrieved1.id.as_str(), fact1.id.as_str());
    assert_eq!(retrieved2.id.as_str(), fact2.id.as_str());
    assert_eq!(retrieved3.id.as_str(), fact3.id.as_str());
    
    // Check registry count
    assert_eq!(registry.count(), 3);
}

#[test]
fn test_fact_registry_filtering() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Create multiple facts
    let fact1 = create_test_register_fact();
    let fact2 = create_test_zkproof_fact();
    let fact3 = create_test_block_fact();
    
    // Store the facts
    registry.register_fact(fact1.clone()).unwrap();
    registry.register_fact(fact2.clone()).unwrap();
    registry.register_fact(fact3.clone()).unwrap();
    
    // Filter by fact type
    let filter_by_type = FactFilter {
        fact_type: Some(FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id: "".to_string(),
            initial_data: vec![],
            owner: "".to_string(),
            domain: "".to_string(),
        })),
        resource_id: None,
        time_range: None,
        origin_domain: None,
    };
    
    let register_facts = registry.query_facts(&filter_by_type, None).unwrap();
    assert_eq!(register_facts.len(), 1);
    assert_eq!(register_facts[0].id.as_str(), fact1.id.as_str());
    
    // Filter by time range
    let filter_by_time = FactFilter {
        fact_type: None,
        resource_id: None,
        time_range: Some((Timestamp(1625097700), Timestamp(1625097800))),
        origin_domain: None,
    };
    
    let time_filtered_facts = registry.query_facts(&filter_by_time, None).unwrap();
    assert_eq!(time_filtered_facts.len(), 2);
    
    // Filter by domain
    let filter_by_domain = FactFilter {
        fact_type: None,
        resource_id: None,
        time_range: None,
        origin_domain: Some("eth-domain".to_string()),
    };
    
    let domain_filtered_facts = registry.query_facts(&filter_by_domain, None).unwrap();
    assert_eq!(domain_filtered_facts.len(), 1);
    assert_eq!(domain_filtered_facts[0].id.as_str(), fact3.id.as_str());
    
    // Filter by resource ID
    let filter_by_resource = FactFilter {
        fact_type: None,
        resource_id: Some("register123".to_string()),
        time_range: None,
        origin_domain: None,
    };
    
    let resource_filtered_facts = registry.query_facts(&filter_by_resource, None).unwrap();
    assert_eq!(resource_filtered_facts.len(), 1);
    assert_eq!(resource_filtered_facts[0].id.as_str(), fact1.id.as_str());
}

#[test]
fn test_fact_registry_combined_filtering() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Create multiple facts with different timestamps but same domain
    let mut fact1 = create_test_register_fact();
    fact1.timestamp = Timestamp(1000);
    fact1.origin_domain = "combined-test".to_string();
    
    let mut fact2 = create_test_register_fact();
    fact2.timestamp = Timestamp(2000);
    fact2.origin_domain = "combined-test".to_string();
    
    let mut fact3 = create_test_zkproof_fact();
    fact3.timestamp = Timestamp(3000);
    fact3.origin_domain = "combined-test".to_string();
    
    // Store the facts
    registry.register_fact(fact1.clone()).unwrap();
    registry.register_fact(fact2.clone()).unwrap();
    registry.register_fact(fact3.clone()).unwrap();
    
    // Combined filter: specific domain + time range + fact type
    let combined_filter = FactFilter {
        fact_type: Some(FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id: "".to_string(),
            initial_data: vec![],
            owner: "".to_string(),
            domain: "".to_string(),
        })),
        resource_id: None,
        time_range: Some((Timestamp(500), Timestamp(2500))),
        origin_domain: Some("combined-test".to_string()),
    };
    
    let filtered_facts = registry.query_facts(&combined_filter, None).unwrap();
    assert_eq!(filtered_facts.len(), 2);
    
    // Verify the correct facts were returned
    let result_ids: Vec<String> = filtered_facts.iter()
        .map(|f| f.id.as_str().to_string())
        .collect();
    
    assert!(result_ids.contains(&fact1.id.as_str().to_string()));
    assert!(result_ids.contains(&fact2.id.as_str().to_string()));
    assert!(!result_ids.contains(&fact3.id.as_str().to_string()));
}

#[test]
fn test_fact_registry_clear() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Add some facts
    registry.register_fact(create_test_register_fact()).unwrap();
    registry.register_fact(create_test_zkproof_fact()).unwrap();
    registry.register_fact(create_test_block_fact()).unwrap();
    
    // Verify count
    assert_eq!(registry.count(), 3);
    
    // Clear the registry
    registry.clear();
    
    // Verify count is now 0
    assert_eq!(registry.count(), 0);
    
    // Try to query after clearing
    let filter = FactFilter {
        fact_type: None,
        resource_id: None,
        time_range: None,
        origin_domain: None,
    };
    
    let facts = registry.query_facts(&filter, None).unwrap();
    assert_eq!(facts.len(), 0);
}

#[test]
fn test_fact_with_dependencies() {
    // Create a registry
    let registry = FactRegistryFactory::create_memory_registry();
    
    // Create and register a fact
    let fact1 = create_test_register_fact();
    let content_id1 = registry.register_fact(fact1.clone()).unwrap();
    
    // Create a fact that depends on the first fact
    let mut fact2 = create_test_zkproof_fact();
    fact2.dependencies = vec![content_id1.clone()];
    
    // Register the dependent fact
    let content_id2 = registry.register_fact(fact2.clone()).unwrap();
    
    // Retrieve and verify
    let retrieved = registry.get_fact(&content_id2).unwrap();
    assert_eq!(retrieved.dependencies.len(), 1);
    assert_eq!(retrieved.dependencies[0], content_id1);
    
    // Try to create a fact with a non-existent dependency
    let mut fact3 = create_test_block_fact();
    let invalid_id = ContentId::from(HashFactory::default().create_hasher().unwrap().hash(b"nonexistent"));
    fact3.dependencies = vec![invalid_id];
    
    // This should fail due to the dependency not existing
    let result = registry.register_fact(fact3);
    assert!(result.is_err());
} 