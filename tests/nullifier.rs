use causality::crypto::{
    ContentAddressed, ContentId, HashOutput, HashAlgorithm, HashFactory
};
use causality::crypto::nullifier::{
    Nullifier, NullifierTracking, NullifierStatus, NullifierFactory, NullifierError
};
use borsh::{BorshSerialize, BorshDeserialize};
use std::collections::HashMap;

// Test structure implementing ContentAddressed
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct TestResource {
    id: u64,
    name: String,
    data: Vec<u8>,
}

impl ContentAddressed for TestResource {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hasher = HashFactory::default().create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, causality::crypto::hash::HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| causality::crypto::hash::HashError::SerializationError(e.to_string()))
    }
}

#[test]
fn test_nullifier_creation() {
    let resource = TestResource {
        id: 1,
        name: "Test Resource".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    
    // Create a nullifier for the resource
    let nullifier = Nullifier::new(&resource).unwrap();
    
    // Check that the nullifier has the right content ID
    assert_eq!(nullifier.content_id, resource.content_id());
    
    // Check that the nullifier value is not zero
    assert!(nullifier.value.iter().any(|&b| b != 0));
    
    // Create a nullifier with metadata
    let nullifier_with_meta = Nullifier::new(&resource)
        .unwrap()
        .with_metadata("purpose", "test")
        .with_metadata("created_at", "2023-07-01");
    
    // Check metadata
    assert_eq!(nullifier_with_meta.get_metadata("purpose"), Some(&"test".to_string()));
    assert_eq!(nullifier_with_meta.get_metadata("created_at"), Some(&"2023-07-01".to_string()));
}

#[test]
fn test_nullifier_tracking() {
    // Create a registry
    let registry = NullifierFactory::create_smt_registry();
    
    // Create a test resource
    let resource = TestResource {
        id: 2,
        name: "Tracking Test".to_string(),
        data: vec![5, 6, 7, 8, 9],
    };
    
    // Get the hash of the resource
    let hash = resource.content_hash();
    
    // Generate a nullifier for the resource
    let nullifier = registry.generate_nullifier(&hash).unwrap();
    
    // Initially, the nullifier should not be in the registry
    assert_eq!(registry.get_status(&nullifier), NullifierStatus::NotFound);
    assert!(!registry.is_spent(&nullifier));
    
    // Register the nullifier
    registry.register_nullifier(&nullifier).unwrap();
    
    // Now it should be registered but not spent
    assert_eq!(registry.get_status(&nullifier), NullifierStatus::Registered);
    assert!(!registry.is_spent(&nullifier));
    
    // Mark the nullifier as spent
    registry.mark_spent(&nullifier).unwrap();
    
    // Now it should be spent
    assert_eq!(registry.get_status(&nullifier), NullifierStatus::Spent);
    assert!(registry.is_spent(&nullifier));
    
    // Trying to register again should fail
    let register_result = registry.register_nullifier(&nullifier);
    assert!(register_result.is_err());
    
    // Trying to mark as spent again should fail
    let spend_result = registry.mark_spent(&nullifier);
    assert!(spend_result.is_err());
}

#[test]
fn test_multiple_nullifiers() {
    // Create a registry
    let registry = NullifierFactory::create_smt_registry();
    
    // Create multiple resources
    let resources = vec![
        TestResource {
            id: 10,
            name: "Resource 1".to_string(),
            data: vec![1, 1, 1],
        },
        TestResource {
            id: 20,
            name: "Resource 2".to_string(),
            data: vec![2, 2, 2],
        },
        TestResource {
            id: 30,
            name: "Resource 3".to_string(),
            data: vec![3, 3, 3],
        },
    ];
    
    // Create nullifiers for all resources
    let nullifiers: Vec<_> = resources.iter()
        .map(|r| registry.generate_nullifier(&r.content_hash()).unwrap())
        .collect();
    
    // Register all nullifiers
    for nullifier in &nullifiers {
        registry.register_nullifier(nullifier).unwrap();
    }
    
    // Mark the first and third nullifiers as spent
    registry.mark_spent(&nullifiers[0]).unwrap();
    registry.mark_spent(&nullifiers[2]).unwrap();
    
    // Check status
    assert_eq!(registry.get_status(&nullifiers[0]), NullifierStatus::Spent);
    assert_eq!(registry.get_status(&nullifiers[1]), NullifierStatus::Registered);
    assert_eq!(registry.get_status(&nullifiers[2]), NullifierStatus::Spent);
    
    // Check is_spent
    assert!(registry.is_spent(&nullifiers[0]));
    assert!(!registry.is_spent(&nullifiers[1]));
    assert!(registry.is_spent(&nullifiers[2]));
}

#[test]
fn test_error_handling() {
    // Create a registry
    let registry = NullifierFactory::create_smt_registry();
    
    // Create a resource
    let resource = TestResource {
        id: 100,
        name: "Error Test".to_string(),
        data: vec![0, 1, 2, 3],
    };
    
    // Generate a nullifier
    let nullifier = registry.generate_nullifier(&resource.content_hash()).unwrap();
    
    // Trying to mark as spent before registering should fail
    let result = registry.mark_spent(&nullifier);
    assert!(matches!(result, Err(NullifierError::NotFound(_))));
    
    // Register the nullifier
    registry.register_nullifier(&nullifier).unwrap();
    
    // Trying to register again should fail
    let result = registry.register_nullifier(&nullifier);
    assert!(matches!(result, Err(NullifierError::AlreadyExists(_))));
    
    // Mark as spent
    registry.mark_spent(&nullifier).unwrap();
    
    // Trying to mark as spent again should fail
    let result = registry.mark_spent(&nullifier);
    assert!(matches!(result, Err(NullifierError::AlreadyExists(_))));
} 