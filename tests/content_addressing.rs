use borsh::{BorshSerialize, BorshDeserialize};
use std::sync::Arc;
use sparse_merkle_tree::default_store::DefaultStore;

use causality::crypto::{
    ContentAddressed, ContentAddressedSmt, ContentId, HashOutput, 
    HashAlgorithm, HashFactory, SmtKeyValue, MerkleSmt
};
use causality::crypto::hash::HashError;

// A simple test struct that implements ContentAddressed
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
struct TestObject {
    id: u64,
    name: String,
    data: Vec<u8>,
}

// Implement ContentAddressed for TestObject
impl ContentAddressed for TestObject {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher from the registry
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization of the object
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

#[test]
fn test_content_addressing_basics() {
    // Create a test object
    let obj1 = TestObject {
        id: 1,
        name: "Test Object".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    
    // Create an identical object
    let obj2 = TestObject {
        id: 1,
        name: "Test Object".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    
    // Create a different object
    let obj3 = TestObject {
        id: 2,
        name: "Different Object".to_string(),
        data: vec![5, 4, 3, 2, 1],
    };
    
    // Get content hashes
    let hash1 = obj1.content_hash();
    let hash2 = obj2.content_hash();
    let hash3 = obj3.content_hash();
    
    // Identical objects should have identical hashes
    assert_eq!(hash1, hash2);
    
    // Different objects should have different hashes
    assert_ne!(hash1, hash3);
    
    // Verify objects
    assert!(obj1.verify());
    assert!(obj2.verify());
    assert!(obj3.verify());
    
    // Test ContentId
    let id1 = obj1.content_id();
    let id2 = obj2.content_id();
    let id3 = obj3.content_id();
    
    // Identical objects should have identical content IDs
    assert_eq!(id1, id2);
    
    // Different objects should have different content IDs
    assert_ne!(id1, id3);
    
    // Test string representation and parsing
    let id_str = id1.to_string();
    let parsed_id = ContentId::parse(&id_str).unwrap();
    assert_eq!(id1, parsed_id);
}

#[test]
fn test_smt_content_addressing() {
    // Create an SMT
    let smt = Arc::new(MerkleSmt::new(DefaultStore::default()));
    
    // Create a test object
    let obj = TestObject {
        id: 42,
        name: "SMT Test Object".to_string(),
        data: vec![10, 20, 30, 40, 50],
    };
    
    // Store the object and get its content hash and proof
    let (content_hash, proof) = smt.store_with_proof(&obj).unwrap();
    
    // Get the current root
    let root = smt.root();
    
    // Verify the inclusion
    assert!(smt.verify_inclusion(&root, &content_hash, &proof));
    
    // Retrieve the object with proof
    let (retrieved_obj, _) = smt.get_with_proof::<TestObject>(&content_hash).unwrap();
    
    // The retrieved object should match the original
    assert_eq!(obj, retrieved_obj);
}

#[test]
fn test_serialization_roundtrip() {
    // Create a test object
    let original = TestObject {
        id: 100,
        name: "Serialization Test".to_string(),
        data: vec![1, 3, 5, 7, 9],
    };
    
    // Serialize
    let bytes = original.to_bytes();
    
    // Deserialize
    let deserialized = TestObject::from_bytes(&bytes).unwrap();
    
    // Should match the original
    assert_eq!(original, deserialized);
    
    // Content hashes should match
    assert_eq!(original.content_hash(), deserialized.content_hash());
}

#[test]
fn test_algorithm_awareness() {
    // Create a test object
    let obj = TestObject {
        id: 200,
        name: "Algorithm Test".to_string(),
        data: vec![2, 4, 6, 8],
    };
    
    // Get content hash (default algorithm - Blake3)
    let hash = obj.content_hash();
    
    // The algorithm should be Blake3
    assert_eq!(hash.algorithm(), HashAlgorithm::Blake3);
    
    // Test hash string format includes algorithm
    let hash_str = hash.to_hex_string();
    assert!(hash_str.starts_with("blake3:"));
    
    // Test parsing from string
    let parsed_hash = HashOutput::from_hex_string(&hash_str).unwrap();
    assert_eq!(hash, parsed_hash);
}

#[test]
fn test_content_id_conversion() {
    // Create a test object
    let obj = TestObject {
        id: 300,
        name: "ContentId Test".to_string(),
        data: vec![5, 10, 15, 20],
    };
    
    // Get content ID
    let content_id = obj.content_id();
    
    // Convert to string
    let id_str = content_id.to_string();
    
    // Should start with cid: prefix
    assert!(id_str.starts_with("cid:"));
    
    // Parse back
    let parsed_id = ContentId::parse(&id_str).unwrap();
    
    // Should match original
    assert_eq!(content_id, parsed_id);
    
    // Should fail to parse invalid string
    assert!(ContentId::parse("invalid:content:id").is_err());
} 