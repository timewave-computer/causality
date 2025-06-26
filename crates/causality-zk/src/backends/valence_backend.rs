//! Valence backend for zero-knowledge computation
//! 
//! This module provides integration with the Valence coprocessor
//! for efficient zero-knowledge proof generation and verification.

use crate::{ZkCircuit, ZkProof, ZkWitness, VerificationKey};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::collections::BTreeMap;
use log::trace;
use serde_json::Value;

#[cfg(feature = "coprocessor")]
use valence_coprocessor_client::{CoprocessorClient, CoprocessorError, CoprocessorConfig};

#[cfg(not(feature = "coprocessor"))]
mod mock_types {
    use super::*;
    
    #[derive(Debug, thiserror::Error)]
    #[allow(dead_code)] // Allow dead code for mock implementations
    pub enum CoprocessorError {
        #[error("HTTP request failed: {0}")]
        RequestFailed(String),
        
        #[error("IO error: {0}")]
        IoError(String),
        
        #[error("Failed to decode response: {0}")]
        DecodingError(String),
        
        #[error("Service error: {0}")]
        ServiceError(String),
        
        #[error("No data received from service")]
        NoDataReceived,
        
        #[error("Invalid data received from service")]
        InvalidDataReceived,
    }
    
    /// Mock configuration for testing
    #[derive(Debug, Clone)]
    #[allow(dead_code)] // Allow dead code for mock implementations
    pub struct CoprocessorConfig {
        /// Socket address of the coprocessor service
        pub socket: SocketAddr,
        /// Base URL for the coprocessor API
        pub base_url: url::Url,
    }

    impl Default for CoprocessorConfig {
        fn default() -> Self {
            let socket = "127.0.0.1:37281".parse().unwrap();
            let base_url = format!("http://{socket}/api/registry").parse().unwrap();
            
            Self {
                socket,
                base_url,
            }
        }
    }

    /// Mock coprocessor client that mirrors the real API
    #[derive(Debug, Clone)]
    pub struct CoprocessorClient {
        /// Configuration for the client
        _config: CoprocessorConfig,
    }

    impl Default for CoprocessorClient {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CoprocessorClient {
        /// Create a new coprocessor client with the default configuration
        pub fn new() -> Self {
            Self::with_config(CoprocessorConfig::default())
        }
        
        /// Create a new coprocessor client with a custom configuration
        pub fn with_config(config: CoprocessorConfig) -> Self {
            Self { _config: config }
        }
        
        /// Create a new coprocessor client with a custom socket address
        pub fn with_socket(socket: SocketAddr) -> Self {
            let base_url = format!("http://{socket}/api/registry").parse().unwrap();
            let config = CoprocessorConfig {
                socket,
                base_url,
            };
            
            Self::with_config(config)
        }
        
        /// Deploy a program to the coprocessor (mock implementation)
        pub fn deploy_program(&self, _wasm_path: &Path, _elf_path: &Path, nonce: u64) -> Result<String, CoprocessorError> {
            trace!("Mock: Deploying program with nonce {}", nonce);
            Ok(format!("mock_program_{}", nonce))
        }
        
        /// Submit a proof request to the coprocessor (mock implementation)
        pub fn submit_proof_request(&self, program: &str, _args: Option<Value>, _path: &Path) -> Result<String, CoprocessorError> {
            trace!("Mock: Submitting proof request for program '{}'", program);
            Ok(format!("mock_proof_request_{}", program))
        }
        
        /// Get the verification key for a program (mock implementation)
        pub fn get_verification_key(&self, program: &str) -> Result<String, CoprocessorError> {
            trace!("Mock: Getting verification key for program '{}'", program);
            Ok(format!("mock_vk_{}", program))
        }
    }
}

#[cfg(not(feature = "coprocessor"))]
use mock_types::CoprocessorClient;

/// Valence backend for zero-knowledge computation
pub struct ValenceBackend {
    /// Coprocessor client for remote computation
    client: CoprocessorClient,
    
    /// Circuit cache to avoid recompilation
    circuit_cache: BTreeMap<String, String>, // circuit_id -> program_id mapping
    
    /// Verification key manager
    verification_keys: VerificationKeyManager,
}

/// Cached verification key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVerificationKey {
    /// The verification key data
    pub key: VerificationKey,
    
    /// When the key was cached
    pub cached_at: u64,
    
    /// Access count for this key
    pub access_count: u64,
    
    /// Metadata about the key
    pub metadata: BTreeMap<String, String>,
}

/// Verification key cache manager
pub struct VerificationKeyManager {
    /// In-memory cache of verification keys
    key_cache: BTreeMap<String, CachedVerificationKey>,
    
    /// Maximum number of keys to cache
    max_cache_size: usize,
    
    /// Cache hit statistics
    cache_hits: u64,
    
    /// Cache miss statistics  
    cache_misses: u64,
}

impl Default for VerificationKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationKeyManager {
    /// Create a new verification key manager
    pub fn new() -> Self {
        Self {
            key_cache: BTreeMap::new(),
            max_cache_size: 1000,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Get verification key from cache
    pub fn get_from_cache(&mut self, key_lookup: &str) -> Option<CachedVerificationKey> {
        if let Some(cached_key) = self.key_cache.get(key_lookup) {
            self.cache_hits += 1;
            Some(cached_key.clone())
        } else {
            self.cache_misses += 1;
            None
        }
    }
    
    /// Store verification key in cache
    pub fn store_in_cache(&mut self, key_lookup: String, cached_key: CachedVerificationKey) {
        // Implement LRU eviction if cache is full
        if self.key_cache.len() >= self.max_cache_size {
            // Simple eviction strategy: remove oldest entry
            if let Some(oldest_key) = self.find_oldest_key() {
                self.key_cache.remove(&oldest_key);
            }
        }
        
        self.key_cache.insert(key_lookup, cached_key);
    }
    
    /// Find the oldest cached key for eviction
    fn find_oldest_key(&self) -> Option<String> {
        self.key_cache
            .iter()
            .min_by_key(|(_, v)| v.cached_at)
            .map(|(k, _)| k.clone())
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        let total_requests = self.cache_hits + self.cache_misses;
        let hit_rate = if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }
}

impl ValenceBackend {
    /// Create a new Valence backend
    pub fn new() -> Self {
        Self {
            client: CoprocessorClient::new(),
            circuit_cache: BTreeMap::new(),
            verification_keys: VerificationKeyManager::new(),
        }
    }
    
    /// Generate a zero-knowledge proof using the Valence coprocessor
    pub async fn generate_proof(
        &mut self,
        circuit: &ZkCircuit,
        witness: &[u32],
        public_inputs: &[u32],
    ) -> Result<ZkProof> {
        let program_id = self.get_or_deploy_program(circuit).await?;
        
        // Create temporary witness file
        let witness_path = std::env::temp_dir().join(format!("witness_{}.json", circuit.id));
        let witness_data = serde_json::json!({
            "witness": witness,
            "public_inputs": public_inputs
        });
        std::fs::write(&witness_path, witness_data.to_string())?;
        
        trace!("Submitting proof generation request to Valence coprocessor");
        let proof_response = self.client.submit_proof_request(
            &program_id,
            Some(serde_json::json!({"inputs": public_inputs})),
            &witness_path
        )?;
        
        // Get verification key from coprocessor
        let vk_data = self.client.get_verification_key(&program_id)?;
        
        // Cache the verification key
        let cached_key = CachedVerificationKey {
            key: VerificationKey {
                key_data: vec![0u32; 32], // Mock key data  
                circuit_hash: circuit.id.clone(),
                proof_system: "groth16".to_string(),
            },
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            access_count: 1,
            metadata: BTreeMap::new(),
        };
        
        self.verification_keys.store_in_cache(
            vk_data.clone(),
            cached_key,
        );
        
        let verification_key = VerificationKey {
            key_data: vec![0u32; 32],
            circuit_hash: circuit.id.clone(),
            proof_system: "groth16".to_string(),
        };
        
        let mut proof = ZkProof {
            id: String::new(), // Will be computed
            circuit_id: circuit.id.clone(),
            proof_data: proof_response.into_bytes(),
            verification_key,
            public_inputs: public_inputs.iter().flat_map(|&x| x.to_le_bytes()).collect(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Compute content-based ID
        proof.id = proof.compute_content_id();
        
        // Clean up temporary file
        let _ = std::fs::remove_file(witness_path);
        
        Ok(proof)
    }
    
    /// Get or deploy a program to the coprocessor
    async fn get_or_deploy_program(&mut self, circuit: &ZkCircuit) -> Result<String> {
        // Check if circuit is already cached
        if let Some(cached_program_id) = self.circuit_cache.get(&circuit.id) {
            trace!("Using cached program: {}", cached_program_id);
            return Ok(cached_program_id.clone());
        }
        
        // Deploy new program
        trace!("Deploying new program for circuit: {}", circuit.id);
        
        // Create temporary WASM and ELF files for the circuit
        let temp_dir = std::env::temp_dir();
        let wasm_path = temp_dir.join(format!("circuit_{}.wasm", circuit.id));
        let elf_path = temp_dir.join(format!("circuit_{}.elf", circuit.id));
        
        // Write mock WASM and ELF data
        std::fs::write(&wasm_path, b"mock wasm bytecode")?;
        std::fs::write(&elf_path, b"mock elf bytecode")?;
        
        let program_id = self.client.deploy_program(&wasm_path, &elf_path, 0)?;
        self.circuit_cache.insert(circuit.id.clone(), program_id.clone());
        
        // Clean up temporary files
        let _ = std::fs::remove_file(wasm_path);
        let _ = std::fs::remove_file(elf_path);
        
        Ok(program_id)
    }
    
    /// Verify a zero-knowledge proof using the Valence coprocessor
    pub async fn verify_proof(
        &mut self,
        proof: &ZkProof,
        _public_inputs: &[u32],
    ) -> Result<bool> {
        // For the mock implementation, we just return true for valid-looking proofs
        // In a real implementation, this would use the coprocessor's verification capabilities
        
        if proof.proof_data.is_empty() {
            return Ok(false);
        }
        
        if proof.verification_key.key_data.is_empty() {
            return Ok(false);
        }
        
        trace!("Mock verification of proof for circuit: {}", proof.circuit_id);
        Ok(true)
    }
    
    /// Get cached circuit information
    pub fn get_cached_circuits(&self) -> Vec<String> {
        self.circuit_cache.keys().cloned().collect()
    }
    
    /// Clear circuit cache
    pub fn clear_circuit_cache(&mut self) {
        self.circuit_cache.clear();
    }
    
    /// Get verification key cache statistics
    pub fn get_verification_key_stats(&self) -> (u64, u64, f64) {
        self.verification_keys.get_cache_stats()
    }
}

impl Default for ValenceBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the Valence backend
#[derive(Debug, Clone)]
pub struct ValenceConfig {
    /// Socket address for the Valence coprocessor service
    pub socket: SocketAddr,
    
    /// Maximum retry attempts for requests
    pub max_retries: usize,
    
    /// Timeout for requests in seconds
    pub timeout_seconds: u64,
}

impl Default for ValenceConfig {
    fn default() -> Self {
        Self {
            socket: "127.0.0.1:37281".parse().unwrap(),
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

impl ValenceBackend {
    /// Create a new Valence backend with configuration
    pub fn with_config(config: ValenceConfig) -> Self {
        Self {
            client: CoprocessorClient::with_socket(config.socket),
            circuit_cache: BTreeMap::new(),
            verification_keys: VerificationKeyManager::new(),
        }
    }
}

impl crate::backends::ZkBackend for ValenceBackend {
    fn generate_proof(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> crate::error::ProofResult<ZkProof> {
        // Create a simple mock proof for now since we don't have a real coprocessor running
        // In a real implementation, this would use the coprocessor client
        
        let verification_key = VerificationKey {
            key_data: vec![0u32; 32],
            circuit_hash: circuit.id.clone(),
            proof_system: "groth16".to_string(),
        };
        
        // Create deterministic proof data based on inputs
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(circuit.id.as_bytes());
        hasher.update(&witness.private_inputs);
        hasher.update(&witness.execution_trace);
        let proof_data = hasher.finalize().to_vec();
        
        let mut proof = ZkProof {
            id: String::new(), // Will be computed
            circuit_id: circuit.id.clone(),
            proof_data,
            verification_key,
            public_inputs: circuit.public_inputs.iter().flat_map(|&x| x.to_le_bytes()).collect(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Compute content-based ID
        proof.id = proof.compute_content_id();
        
        Ok(proof)
    }
    
    fn verify_proof(&self, proof: &ZkProof, _public_inputs: &[i64]) -> Result<bool, crate::error::VerificationError> {
        // Simple mock verification - always return true for valid-looking proofs
        // In a real implementation, this would use the coprocessor client
        
        if proof.proof_data.is_empty() {
            return Err(crate::error::VerificationError::InvalidProof("Empty proof data".to_string()));
        }
        
        if proof.verification_key.key_data.is_empty() {
            return Err(crate::error::VerificationError::InvalidProof("Empty verification key".to_string()));
        }
        
        // Mock verification always succeeds for non-empty proofs
        Ok(true)
    }
    
    fn backend_name(&self) -> &'static str {
        "valence"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

impl Clone for ValenceBackend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            circuit_cache: self.circuit_cache.clone(),
            verification_keys: VerificationKeyManager::new(), // Reset cache on clone
        }
    }
} 