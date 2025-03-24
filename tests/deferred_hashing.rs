use causality::crypto::{
    DeferredHashing, DeferredHashingContext, DeferredHashBatchProcessor,
    HashAlgorithm, HashFactory, ContentAddressed, ContentId
};

// Import test utilities
use borsh::{BorshSerialize, BorshDeserialize};
use causality::crypto::hash::HashError;

// A simple content-addressed test object for deferred hashing tests
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
struct DeferredHashTest {
    id: u64,
    name: String,
    data: Vec<u8>,
}

impl ContentAddressed for DeferredHashTest {
    fn content_hash(&self) -> causality::crypto::HashOutput {
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
fn test_deferred_hashing_basic() {
    // Create a deferred hashing context
    let mut context = DeferredHashingContext::default().unwrap();
    
    // Create test data
    let data1 = b"Test data for deferred hashing";
    let data2 = b"More test data for deferred hashing";
    
    // Request hash computations
    let hash_id1 = context.request_hash(data1, HashAlgorithm::Blake3);
    let hash_id2 = context.request_hash(data2, HashAlgorithm::Blake3);
    
    // Verify that hash results are not available yet
    assert!(!context.has_hash_result(&hash_id1));
    assert!(!context.has_hash_result(&hash_id2));
    assert_eq!(context.get_hash_result(&hash_id1), None);
    assert_eq!(context.get_hash_result(&hash_id2), None);
    
    // Compute the hashes
    context.compute_deferred_hashes();
    
    // Now the hash results should be available
    assert!(context.has_hash_result(&hash_id1));
    assert!(context.has_hash_result(&hash_id2));
    
    // Verify that the hash results match what we expect
    let hash_factory = HashFactory::default();
    let hasher = hash_factory.create_hasher().unwrap();
    
    let expected_hash1 = hasher.hash(data1);
    let expected_hash2 = hasher.hash(data2);
    
    assert_eq!(context.get_hash_result(&hash_id1).unwrap(), expected_hash1);
    assert_eq!(context.get_hash_result(&hash_id2).unwrap(), expected_hash2);
}

#[test]
fn test_batch_processor() {
    // Create test data
    let data1 = b"Batch processor test data 1";
    let data2 = b"Batch processor test data 2";
    
    // Create inputs
    let inputs = vec![
        causality::crypto::DeferredHashInput::new(data1.to_vec(), HashAlgorithm::Blake3),
        causality::crypto::DeferredHashInput::new(data2.to_vec(), HashAlgorithm::Blake3),
    ];
    
    // Get the IDs
    let id1 = inputs[0].id.clone();
    let id2 = inputs[1].id.clone();
    
    // Create a batch processor
    let processor = DeferredHashBatchProcessor::default().unwrap();
    
    // Process the batch
    let results = processor.process_batch(&inputs);
    
    // Verify the results
    let hash_factory = HashFactory::default();
    let hasher = hash_factory.create_hasher().unwrap();
    
    let expected_hash1 = hasher.hash(data1);
    let expected_hash2 = hasher.hash(data2);
    
    assert_eq!(*results.get(&id1).unwrap(), expected_hash1);
    assert_eq!(*results.get(&id2).unwrap(), expected_hash2);
}

#[test]
fn test_deferred_content_addressing() {
    // Create a deferred hashing context
    let mut context = DeferredHashingContext::default().unwrap();
    
    // Create a test object
    let test_obj = DeferredHashTest {
        id: 42,
        name: "Deferred Hashing Test".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    
    // Serialize the object for hashing
    let serialized = test_obj.to_bytes();
    
    // Request a hash computation for the object
    let hash_id = context.request_hash(&serialized, HashAlgorithm::Blake3);
    
    // Compute the hashes
    context.compute_deferred_hashes();
    
    // Get the hash result
    let hash_result = context.get_hash_result(&hash_id).unwrap();
    
    // Verify that it matches the direct hash
    let direct_hash = test_obj.content_hash();
    
    assert_eq!(hash_result, direct_hash);
    
    // Create a ContentId from the hash
    let content_id = ContentId::from(hash_result);
    
    // Verify that it matches the direct content ID
    let direct_id = test_obj.content_id();
    
    assert_eq!(content_id, direct_id);
} 