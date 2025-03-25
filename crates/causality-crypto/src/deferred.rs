// Deferred cryptographic operations
// Original file: src/crypto/deferred.rs

// Deferred Hashing Implementation
//
// This module provides support for deferred hash computation,
// allowing hash operations to be performed outside of the zkVM
// for improved performance.

use std::collections::HashMap;
use causality_crypto::{DeferredHashId, HashAlgorithm, HashFunction, HashOutput, HashError};
use std::sync::{Arc, Mutex};

/// Input for a deferred hash operation
#[derive(Debug, Clone)]
pub struct DeferredHashInput {
    /// Unique identifier for this hash operation
    pub id: DeferredHashId,
    /// Data to be hashed
    pub data: Vec<u8>,
    /// Algorithm to use for hashing
    pub algorithm: HashAlgorithm,
}

impl DeferredHashInput {
    /// Create a new deferred hash input
    pub fn new(data: Vec<u8>, algorithm: HashAlgorithm) -> Self {
        Self {
            id: DeferredHashId::new(),
            data,
            algorithm,
        }
    }
    
    /// Create a new deferred hash input with a specific ID
    pub fn with_id(id: DeferredHashId, data: Vec<u8>, algorithm: HashAlgorithm) -> Self {
        Self {
            id,
            data,
            algorithm,
        }
    }
}

/// Execution context with deferred hashing support
pub struct DeferredHashingContext {
    /// Requests for hash operations (to be processed after execution)
    deferred_hash_inputs: Vec<DeferredHashInput>,
    /// Results of computed hash operations
    hash_results: Mutex<HashMap<DeferredHashId, HashOutput>>,
    /// Hash function registry for different algorithms
    hash_functions: HashMap<HashAlgorithm, Arc<dyn HashFunction>>,
}

impl DeferredHashingContext {
    /// Create a new context with the specified hash functions
    pub fn new(hash_functions: HashMap<HashAlgorithm, Arc<dyn HashFunction>>) -> Self {
        Self {
            deferred_hash_inputs: Vec::new(),
            hash_results: Mutex::new(HashMap::new()),
            hash_functions,
        }
    }
    
    /// Create a new context with default hash functions
    pub fn default() -> Result<Self, HashError> {
        use causality_crypto::{HashFactory, Blake3HashFunction};
        
        let mut hash_functions = HashMap::new();
        
        // Add Blake3 by default
        hash_functions.insert(
            HashAlgorithm::Blake3,
            Arc::new(Blake3HashFunction::new()) as Arc<dyn HashFunction>
        );
        
        // Add Poseidon if enabled
        #[cfg(feature = "poseidon")]
        {
            use causality_crypto::PoseidonHashFunction;
            hash_functions.insert(
                HashAlgorithm::Poseidon,
                Arc::new(PoseidonHashFunction::new()) as Arc<dyn HashFunction>
            );
        }
        
        Ok(Self::new(hash_functions))
    }
    
    /// Get access to the deferred hash inputs
    pub fn deferred_inputs(&self) -> &[DeferredHashInput] {
        &self.deferred_hash_inputs
    }
    
    /// Add a hash function for a specific algorithm
    pub fn add_hash_function(&mut self, algorithm: HashAlgorithm, function: Arc<dyn HashFunction>) {
        self.hash_functions.insert(algorithm, function);
    }
}

impl causality_crypto::DeferredHashing for DeferredHashingContext {
    /// Request a hash computation (creates a placeholder)
    fn request_hash(
        &mut self, 
        data: &[u8], 
        algorithm: HashAlgorithm
    ) -> DeferredHashId {
        // Create a unique ID for this request
        let id = DeferredHashId::new();
        
        // Store the request for later processing
        self.deferred_hash_inputs.push(DeferredHashInput::with_id(
            id.clone(),
            data.to_vec(),
            algorithm,
        ));
        
        id
    }
    
    /// Check if a deferred hash result is available
    fn has_hash_result(&self, id: &DeferredHashId) -> bool {
        self.hash_results.lock().unwrap().contains_key(id)
    }
    
    /// Get the result of a deferred hash operation
    fn get_hash_result(&self, id: &DeferredHashId) -> Option<HashOutput> {
        self.hash_results.lock().unwrap().get(id).cloned()
    }
    
    /// Perform all deferred hash computations
    fn compute_deferred_hashes(&mut self) {
        let mut results = self.hash_results.lock().unwrap();
        
        for input in &self.deferred_hash_inputs {
            // Get the appropriate hash function
            if let Some(hash_function) = self.hash_functions.get(&input.algorithm) {
                // Compute the hash
                let hash = hash_function.hash(&input.data);
                
                // Store the result
                results.insert(input.id.clone(), hash);
            }
        }
    }
}

/// A batch processor for deferred hash computations
pub struct DeferredHashBatchProcessor {
    /// Hash function registry for different algorithms
    hash_functions: HashMap<HashAlgorithm, Arc<dyn HashFunction>>,
}

impl DeferredHashBatchProcessor {
    /// Create a new batch processor with the specified hash functions
    pub fn new(hash_functions: HashMap<HashAlgorithm, Arc<dyn HashFunction>>) -> Self {
        Self { hash_functions }
    }
    
    /// Create a new batch processor with default hash functions
    pub fn default() -> Result<Self, HashError> {
        use causality_crypto::{HashFactory, Blake3HashFunction};
        
        let mut hash_functions = HashMap::new();
        
        // Add Blake3 by default
        hash_functions.insert(
            HashAlgorithm::Blake3,
            Arc::new(Blake3HashFunction::new()) as Arc<dyn HashFunction>
        );
        
        // Add Poseidon if enabled
        #[cfg(feature = "poseidon")]
        {
            use causality_crypto::PoseidonHashFunction;
            hash_functions.insert(
                HashAlgorithm::Poseidon,
                Arc::new(PoseidonHashFunction::new()) as Arc<dyn HashFunction>
            );
        }
        
        Ok(Self::new(hash_functions))
    }
    
    /// Process a batch of deferred hash inputs
    pub fn process_batch(&self, inputs: &[DeferredHashInput]) -> HashMap<DeferredHashId, HashOutput> {
        let mut results = HashMap::new();
        
        for input in inputs {
            // Get the appropriate hash function
            if let Some(hash_function) = self.hash_functions.get(&input.algorithm) {
                // Compute the hash
                let hash = hash_function.hash(&input.data);
                
                // Store the result
                results.insert(input.id.clone(), hash);
            }
        }
        
        results
    }
    
    /// Add a hash function for a specific algorithm
    pub fn add_hash_function(&mut self, algorithm: HashAlgorithm, function: Arc<dyn HashFunction>) {
        self.hash_functions.insert(algorithm, function);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_crypto::HashFactory;
    
    #[test]
    fn test_deferred_hashing_context() {
        // Create a context
        let mut context = DeferredHashingContext::default().unwrap();
        
        // Request some hashes
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        
        let id1 = context.request_hash(data1, HashAlgorithm::Blake3);
        let id2 = context.request_hash(data2, HashAlgorithm::Blake3);
        
        // No results should be available yet
        assert!(!context.has_hash_result(&id1));
        assert!(!context.has_hash_result(&id2));
        
        // Compute the hashes
        context.compute_deferred_hashes();
        
        // Results should now be available
        assert!(context.has_hash_result(&id1));
        assert!(context.has_hash_result(&id2));
        
        // Verify results
        let factory = HashFactory::default();
        let hasher = factory.create_hasher().unwrap();
        
        let expected1 = hasher.hash(data1);
        let expected2 = hasher.hash(data2);
        
        assert_eq!(context.get_hash_result(&id1).unwrap(), expected1);
        assert_eq!(context.get_hash_result(&id2).unwrap(), expected2);
    }
    
    #[test]
    fn test_batch_processor() {
        // Create a batch processor
        let processor = DeferredHashBatchProcessor::default().unwrap();
        
        // Create some inputs
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        
        let input1 = DeferredHashInput::new(data1.to_vec(), HashAlgorithm::Blake3);
        let input2 = DeferredHashInput::new(data2.to_vec(), HashAlgorithm::Blake3);
        
        let id1 = input1.id.clone();
        let id2 = input2.id.clone();
        
        // Process the batch
        let results = processor.process_batch(&[input1, input2]);
        
        // Verify results
        let factory = HashFactory::default();
        let hasher = factory.create_hasher().unwrap();
        
        let expected1 = hasher.hash(data1);
        let expected2 = hasher.hash(data2);
        
        assert_eq!(results.get(&id1).unwrap(), &expected1);
        assert_eq!(results.get(&id2).unwrap(), &expected2);
    }
} 