// Resource verification in TEL
// Original file: src/tel/resource/verify.rs

// ZK verification module for TEL resources
//
// This module provides zero-knowledge proof verification
// capabilities for resource operations.
// Migrated to use the unified ResourceRegister model.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crate::crypto::ContentId;
use causality_resource::ResourceRegister;
use crate::operation::{RegisterOperationType, Operation};
use causality_tel::{Proof, OperationId};
use causality_tel::{TelError, TelResult};
use causality_tel::{ResourceOperation, ResourceOperationType};

/// Verification result for a proof
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub is_valid: bool,
    /// Reason for failure, if any
    pub reason: Option<String>,
    /// Time taken for verification (ms)
    pub time_taken_ms: u64,
    /// Additional metadata from verification
    pub metadata: HashMap<String, String>,
}

impl VerificationResult {
    /// Create a new successful verification result
    pub fn success(time_taken_ms: u64) -> Self {
        Self {
            is_valid: true,
            reason: None,
            time_taken_ms,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new failed verification result
    pub fn failure(reason: String, time_taken_ms: u64) -> Self {
        Self {
            is_valid: false,
            reason: Some(reason),
            time_taken_ms,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the verification result
    pub fn with_metadata(mut self, key: &str, value: String) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// Type of verification key
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationKeyType {
    /// Groth16 verification key
    Groth16,
    /// PlonK verification key
    PlonK,
    /// STARK verification key
    Stark,
    /// BulletProofs verification key
    BulletProofs,
    /// Custom verification key type
    Custom(String),
}

/// Configuration for ZK verification
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// Whether verification is enabled
    pub enabled: bool,
    /// Timeout for verification (ms)
    pub timeout_ms: u64,
    /// Whether to cache verification results
    pub enable_caching: bool,
    /// Maximum size of verification cache
    pub max_cache_size: usize,
    /// Whether to verify proofs in parallel when possible
    pub parallel_verification: bool,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 5000, // 5 seconds
            enable_caching: true,
            max_cache_size: 1000,
            parallel_verification: true,
        }
    }
}

/// Cache for verification results
#[derive(Debug)]
struct VerificationCache {
    /// Cached verification results
    results: HashMap<OperationId, VerificationResult>,
    /// Maximum size of the cache
    max_size: usize,
}

impl VerificationCache {
    /// Create a new verification cache
    fn new(max_size: usize) -> Self {
        Self {
            results: HashMap::new(),
            max_size,
        }
    }
    
    /// Get a cached verification result
    fn get(&self, operation_id: &OperationId) -> Option<&VerificationResult> {
        self.results.get(operation_id)
    }
    
    /// Cache a verification result
    fn put(&mut self, operation_id: OperationId, result: VerificationResult) {
        // If cache is full, remove a random entry
        if self.results.len() >= self.max_size {
            if let Some(key) = self.results.keys().next().cloned() {
                self.results.remove(&key);
            }
        }
        
        // Cache the result
        self.results.insert(operation_id, result);
    }
    
    /// Clear the cache
    fn clear(&mut self) {
        self.results.clear();
    }
}

/// Verifier for zero-knowledge proofs of resource operations
pub struct ZkVerifier {
    /// Configuration for the verifier
    config: RwLock<VerifierConfig>,
    /// Cache of verification results
    cache: RwLock<VerificationCache>,
    /// Registry of verification keys
    verification_keys: RwLock<HashMap<String, Vec<u8>>>,
}

impl ZkVerifier {
    /// Create a new ZK verifier
    pub fn new(config: VerifierConfig) -> Self {
        Self {
            cache: RwLock::new(VerificationCache::new(config.max_cache_size)),
            config: RwLock::new(config),
            verification_keys: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new ZK verifier with default configuration
    pub fn default() -> Self {
        Self::new(VerifierConfig::default())
    }
    
    /// Verify a resource operation's proof
    pub fn verify_operation(&self, operation: &ResourceOperation) -> TelResult<VerificationResult> {
        // Check if verification is enabled
        let config = self.config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire config lock".to_string()))?;
            
        if !config.enabled {
            return Ok(VerificationResult::success(0));
        }
        
        // Check if operation has a proof
        if operation.proof.is_none() {
            return Ok(VerificationResult::failure(
                "Operation does not have a proof".to_string(), 
                0
            ));
        }
        
        // Check if result is cached
        if config.enable_caching {
            let cache = self.cache.read().map_err(|_| 
                TelError::InternalError("Failed to acquire cache lock".to_string()))?;
                
            if let Some(result) = cache.get(&operation.operation_id) {
                return Ok(result.clone());
            }
        }
        
        // Get verification key
        let verification_key = if let Some(key) = &operation.verification_key {
            key
        } else {
            return Ok(VerificationResult::failure(
                "Operation does not have a verification key".to_string(), 
                0
            ));
        };
        
        // Start the verification timer
        let start_time = std::time::Instant::now();
        
        // Verify the proof
        let is_valid = self.verify_proof(
            operation.proof.as_ref().unwrap(),
            verification_key,
            operation
        )?;
        
        // Calculate verification time
        let time_taken_ms = start_time.elapsed().as_millis() as u64;
        
        // Create verification result
        let result = if is_valid {
            VerificationResult::success(time_taken_ms)
        } else {
            VerificationResult::failure(
                "Proof verification failed".to_string(), 
                time_taken_ms
            )
        };
        
        // Cache the result if caching is enabled
        if config.enable_caching {
            let mut cache = self.cache.write().map_err(|_| 
                TelError::InternalError("Failed to acquire cache lock".to_string()))?;
                
            cache.put(operation.operation_id.clone(), result.clone());
        }
        
        Ok(result)
    }
    
    /// Verify a unified resource register operation's proof
    pub fn verify_register_operation(&self, register_id: &ContentId, operation: &Operation) -> TelResult<VerificationResult> {
        // Start verification time tracking
        let start_time = std::time::Instant::now();
        
        // Get operation type
        let operation_type = match &operation.operation_type {
            crate::operation::OperationType::Register(op_type) => op_type,
            _ => return Err(TelError::InvalidOperation("Not a register operation".to_string())),
        };
        
        // Check if verification is enabled
        let config = self.config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire config lock".to_string()))?;
            
        if !config.enabled {
            return Ok(VerificationResult::success(0));
        }
        
        // Check for proof in the operation
        let proof = operation.proof.as_ref()
            .ok_or_else(|| TelError::VerificationError("No proof provided for operation".to_string()))?;
        
        // Get verification key ID from the proof
        let key_id = proof.metadata.get("verification_key_id")
            .ok_or_else(|| TelError::VerificationError("No verification key ID in proof".to_string()))?;
            
        // Get verification key
        let verification_key = self.get_verification_key(key_id)?
            .ok_or_else(|| TelError::VerificationError(format!("Verification key not found: {}", key_id)))?;
        
        // Check if result is cached
        if config.enable_caching {
            // Generate an operation ID based on register ID and operation hash
            let operation_hash = operation.hash();
            let operation_id = OperationId::from(operation_hash.as_bytes().to_vec());
            
            let cache = self.cache.read().map_err(|_| 
                TelError::InternalError("Failed to acquire cache lock".to_string()))?;
                
            if let Some(result) = cache.get(&operation_id) {
                return Ok(result.clone());
            }
        }
        
        // Verify the proof
        let is_valid = self.verify_proof(
            &proof, 
            &verification_key, 
            &operation.convert_to_resource_operation()
        )?;
        
        // Calculate time taken
        let time_taken = start_time.elapsed().as_millis() as u64;
        
        // Create verification result
        let result = if is_valid {
            VerificationResult::success(time_taken)
                .with_metadata("operation_type", format!("{:?}", operation_type))
                .with_metadata("register_id", register_id.to_string())
        } else {
            VerificationResult::failure("Proof verification failed".to_string(), time_taken)
                .with_metadata("operation_type", format!("{:?}", operation_type))
                .with_metadata("register_id", register_id.to_string())
        };
        
        // Cache the result if caching is enabled
        if config.enable_caching {
            let operation_hash = operation.hash();
            let operation_id = OperationId::from(operation_hash.as_bytes().to_vec());
            
            let mut cache = self.cache.write().map_err(|_| 
                TelError::InternalError("Failed to acquire cache lock".to_string()))?;
                
            cache.put(operation_id, result.clone());
        }
        
        Ok(result)
    }
    
    /// Register a verification key
    pub fn register_verification_key(&self, key_id: &str, key: Vec<u8>) -> TelResult<()> {
        let mut verification_keys = self.verification_keys.write().map_err(|_| 
            TelError::InternalError("Failed to acquire verification keys lock".to_string()))?;
            
        verification_keys.insert(key_id.to_string(), key);
        
        Ok(())
    }
    
    /// Get a verification key
    pub fn get_verification_key(&self, key_id: &str) -> TelResult<Option<Vec<u8>>> {
        let verification_keys = self.verification_keys.read().map_err(|_| 
            TelError::InternalError("Failed to acquire verification keys lock".to_string()))?;
            
        Ok(verification_keys.get(key_id).cloned())
    }
    
    /// Update the configuration
    pub fn configure(&self, config: VerifierConfig) -> TelResult<()> {
        // Update the configuration
        let mut current_config = self.config.write().map_err(|_| 
            TelError::InternalError("Failed to acquire config lock".to_string()))?;
            
        *current_config = config.clone();
        
        // If cache size has changed, update the cache
        if config.max_cache_size != current_config.max_cache_size {
            let mut cache = self.cache.write().map_err(|_| 
                TelError::InternalError("Failed to acquire cache lock".to_string()))?;
                
            // Create a new cache with the updated size
            let new_cache = VerificationCache::new(config.max_cache_size);
            *cache = new_cache;
        }
        
        Ok(())
    }
    
    /// Clear the verification cache
    pub fn clear_cache(&self) -> TelResult<()> {
        let mut cache = self.cache.write().map_err(|_| 
            TelError::InternalError("Failed to acquire cache lock".to_string()))?;
            
        cache.clear();
        
        Ok(())
    }
    
    /// Verify a proof against a verification key
    fn verify_proof(
        &self, 
        proof: &Proof, 
        verification_key: &[u8],
        operation: &ResourceOperation
    ) -> TelResult<bool> {
        // In a real implementation, this would perform actual ZK verification
        // based on the proof type, verification key, and operation data
        
        // For simulation purposes, we'll return valid for proofs that have
        // data that matches the operation ID
        let valid = proof.0.iter()
            .zip(operation.operation_id.0.iter())
            .fold(true, |acc, (a, b)| acc && (a == b));
            
        // Include a small delay to simulate verification time
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(valid)
    }
    
    /// Verify a ResourceRegister directly
    pub fn verify_resource_register(&self, register: &ResourceRegister, operation_type: RegisterOperationType) -> TelResult<VerificationResult> {
        // Start verification time tracking
        let start_time = std::time::Instant::now();
        
        // Create an Operation from the ResourceRegister
        let operation = Operation {
            id: ContentId::from_bytes(&register.id.to_bytes()),
            operation_type: crate::operation::OperationType::Register(operation_type),
            inputs: vec![],
            outputs: vec![register.id.clone()],
            parameters: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
            proof: None, // We expect the proof to be in the register metadata
        };
        
        // Extract proof from register metadata
        let proof_data = register.metadata.get("zk_proof")
            .ok_or_else(|| TelError::VerificationError("No proof found in register metadata".to_string()))?;
            
        let proof = serde_json::from_str::<Proof>(proof_data)
            .map_err(|e| TelError::VerificationError(format!("Failed to parse proof: {}", e)))?;
        
        // Get verification key ID from the proof
        let key_id = proof.metadata.get("verification_key_id")
            .ok_or_else(|| TelError::VerificationError("No verification key ID in proof".to_string()))?;
            
        // Get verification key
        let verification_key = self.get_verification_key(key_id)?
            .ok_or_else(|| TelError::VerificationError(format!("Verification key not found: {}", key_id)))?;
        
        // Create a resource operation for verification
        let resource_operation = ResourceOperation {
            id: ContentId::from_bytes(&register.id.to_bytes()),
            operation_type: ResourceOperationType::CreateResource,
            resource_id: register.id.clone(),
            parameters: HashMap::new(),
            metadata: register.metadata.clone(),
            proof: Some(proof),
        };
        
        // Verify the proof
        let is_valid = self.verify_proof(&resource_operation.proof.as_ref().unwrap(), 
                                        &verification_key, 
                                        &resource_operation)?;
        
        // Calculate time taken
        let time_taken = start_time.elapsed().as_millis() as u64;
        
        // Create verification result
        let result = if is_valid {
            VerificationResult::success(time_taken)
                .with_metadata("operation_type", format!("{:?}", operation_type))
                .with_metadata("register_id", register.id.to_string())
        } else {
            VerificationResult::failure("Proof verification failed".to_string(), time_taken)
                .with_metadata("operation_type", format!("{:?}", operation_type))
                .with_metadata("register_id", register.id.to_string())
        };
        
        Ok(result)
    }
}

/// Shared ZK verifier for use across multiple components
pub struct SharedZkVerifier {
    /// The ZK verifier
    verifier: Arc<ZkVerifier>,
}

impl SharedZkVerifier {
    /// Create a new shared ZK verifier
    pub fn new(config: VerifierConfig) -> Self {
        Self {
            verifier: Arc::new(ZkVerifier::new(config)),
        }
    }
    
    /// Create a new shared ZK verifier with default configuration
    pub fn default() -> Self {
        Self {
            verifier: Arc::new(ZkVerifier::default()),
        }
    }
    
    /// Get the underlying verifier
    pub fn verifier(&self) -> &Arc<ZkVerifier> {
        &self.verifier
    }
    
    /// Verify a resource operation's proof
    pub fn verify_operation(&self, operation: &ResourceOperation) -> TelResult<VerificationResult> {
        self.verifier.verify_operation(operation)
    }
    
    /// Verify a unified resource register operation's proof
    pub fn verify_register_operation(&self, register_id: &ContentId, operation: &Operation) -> TelResult<VerificationResult> {
        self.verifier.verify_register_operation(register_id, operation)
    }
    
    /// Verify a ResourceRegister directly
    pub fn verify_resource_register(&self, register: &ResourceRegister, operation_type: RegisterOperationType) -> TelResult<VerificationResult> {
        self.verifier.verify_resource_register(register, operation_type)
    }
    
    /// Register a verification key
    pub fn register_verification_key(&self, key_id: &str, key: Vec<u8>) -> TelResult<()> {
        self.verifier.register_verification_key(key_id, key)
    }
    
    /// Update the configuration
    pub fn configure(&self, config: VerifierConfig) -> TelResult<()> {
        self.verifier.configure(config)
    }
    
    /// Clear the verification cache
    pub fn clear_cache(&self) -> TelResult<()> {
        self.verifier.clear_cache()
    }
} 
