//! ZK proof integration for effect execution
//!
//! This module provides integration between ZK proofs and effect execution,
//! allowing effects to be verified with zero-knowledge proofs.

use std::collections::BTreeMap;
use std::sync::Arc;
use crate::{
    effect::{
        core::EffectExpr,
        handler_registry::{EffectHandler, EffectHandlerRegistry, EffectResult},
    },
    lambda::base::Value,
    system::error::{Error, Result},
};

/// Hash of an effect for ZK proof generation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectHash {
    /// The SSZ-based hash of the effect
    pub hash: [u8; 32],
}

impl EffectHash {
    /// Generate hash for an effect expression
    pub fn from_effect(effect: &EffectExpr) -> Self {
        use crate::{Hasher, Sha256Hasher};
        
        // Simple hash of effect kind and metadata
        let effect_str = format!("{:?}", effect);
        let hash = Sha256Hasher::hash(effect_str.as_bytes());
        
        Self { hash }
    }
    
    /// Generate hash for effect parameters
    pub fn from_params(effect_tag: &str, params: &[Value]) -> Self {
        use crate::{Hasher, Sha256Hasher};
        
        // Create a simple string representation and hash it
        let mut content = effect_tag.to_string();
        for param in params {
            content.push_str(&format!("{:?}", param));
        }
        
        let hash = Sha256Hasher::hash(content.as_bytes());
        Self { hash }
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
    
    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| Error::serialization("Invalid hex string"))?;
        
        if bytes.len() != 32 {
            return Err(Error::serialization("Invalid hash length"));
        }
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        
        Ok(Self { hash })
    }
}

/// Mock ZK proof structure for minimal implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkProof {
    /// Proof data (simplified)
    pub data: Vec<u8>,
    
    /// Associated effect hash
    pub effect_hash: EffectHash,
}

impl ZkProof {
    /// Create a new mock proof
    pub fn new(effect_hash: EffectHash, data: Vec<u8>) -> Self {
        Self { data, effect_hash }
    }
    
    /// Create a mock proof for testing (improved implementation)
    pub fn mock_proof(effect_hash: EffectHash) -> Self {
        use sha2::{Sha256, Digest};
        
        // Create a properly formatted proof with SNARK-like structure
        let mut proof_data = Vec::new();
        
        // 1. Add proof header
        proof_data.extend_from_slice(b"ZKPF");
        
        // 2. Generate deterministic mock proof components (A, B, C) using effect hash
        let mut hasher_a = Sha256::new();
        hasher_a.update(effect_hash.hash);
        hasher_a.update(b"component_a");
        let component_a = hasher_a.finalize();
        
        let mut hasher_b = Sha256::new();
        hasher_b.update(effect_hash.hash);
        hasher_b.update(b"component_b");
        let component_b = hasher_b.finalize();
        
        let mut hasher_c = Sha256::new();
        hasher_c.update(effect_hash.hash);
        hasher_c.update(b"component_c");
        let component_c = hasher_c.finalize();
        
        proof_data.extend_from_slice(&component_a);
        proof_data.extend_from_slice(&component_b);
        proof_data.extend_from_slice(&component_c);
        
        // 3. Generate commitment hash
        let mut hasher = Sha256::new();
        hasher.update(component_a);
        hasher.update(component_b);
        hasher.update(component_c);
        hasher.update(effect_hash.hash);
        let commitment = hasher.finalize();
        
        // 4. Add commitment to end of proof
        proof_data.extend_from_slice(&commitment);
        
        Self {
            data: proof_data,
            effect_hash,
        }
    }
    
    /// Verify this proof (improved implementation)
    pub fn verify(&self) -> bool {
        // Improved verification that checks proof structure and cryptographic validity
        
        // 1. Basic sanity checks
        if self.data.is_empty() || self.data.len() < 32 {
            return false;
        }
        
        // 2. Check proof format - should have proper header
        if self.data.len() < 4 || &self.data[0..4] != b"ZKPF" {
            return false;
        }
        
        // 3. Verify proof components using cryptographic hash verification
        // Extract proof components (simplified SNARK-like structure)
        let proof_offset = 4;
        if self.data.len() < proof_offset + 96 { // Need at least 3 * 32 bytes for proof components
            return false;
        }
        
        // Extract proof components (A, B, C in a SNARK-like proof)
        let component_a = &self.data[proof_offset..proof_offset + 32];
        let component_b = &self.data[proof_offset + 32..proof_offset + 64];
        let component_c = &self.data[proof_offset + 64..proof_offset + 96];
        
        // 4. Verify components are non-zero (basic EC point validation)
        let a_nonzero = component_a.iter().any(|&x| x != 0);
        let b_nonzero = component_b.iter().any(|&x| x != 0);
        let c_nonzero = component_c.iter().any(|&x| x != 0);
        
        if !a_nonzero || !b_nonzero || !c_nonzero {
            return false;
        }
        
        // 5. Verify proof commitment to effect hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(component_a);
        hasher.update(component_b);
        hasher.update(component_c);
        hasher.update(self.effect_hash.hash);
        
        let commitment = hasher.finalize();
        
        // Check if proof contains valid commitment (simplified)
        let expected_commitment = &self.data[self.data.len() - 32..];
        commitment.as_slice() == expected_commitment
    }
}

/// ZK-verified effect handler that requires proof verification
pub struct ZkVerifiedEffectHandler {
    /// The underlying effect handler
    inner_handler: Arc<dyn EffectHandler>,
    
    /// Cache of verified effect hashes
    verified_cache: BTreeMap<EffectHash, bool>,
    
    /// Whether to require proofs (for testing)
    require_proofs: bool,
}

impl ZkVerifiedEffectHandler {
    /// Create a new ZK-verified effect handler
    pub fn new(inner_handler: Arc<dyn EffectHandler>) -> Self {
        Self {
            inner_handler,
            verified_cache: BTreeMap::new(),
            require_proofs: false, // Default to not requiring proofs for minimal implementation
        }
    }
    
    /// Create with proof requirement
    pub fn with_proof_requirement(mut self, require_proofs: bool) -> Self {
        self.require_proofs = require_proofs;
        self
    }
    
    /// Verify effect execution with ZK proof
    pub fn verify_effect_execution(&mut self, effect_hash: &EffectHash, proof: &ZkProof) -> Result<bool> {
        // Check cache first
        if let Some(&cached_result) = self.verified_cache.get(effect_hash) {
            return Ok(cached_result);
        }
        
        // Verify the proof matches the effect hash
        let is_valid = proof.effect_hash == *effect_hash && proof.verify();
        
        // Cache the result
        self.verified_cache.insert(effect_hash.clone(), is_valid);
        
        Ok(is_valid)
    }
    
    /// Generate a mock ZK proof for effect execution (improved implementation)
    pub fn generate_mock_proof(&self, effect_hash: &EffectHash, params: &[Value]) -> Result<ZkProof> {
        // Improved implementation that considers parameters in proof generation
        use sha2::{Sha256, Digest};
        
        // Create parameter-dependent proof by hashing the parameters
        let mut param_hasher = Sha256::new();
        for param in params {
            match param {
                Value::Int(i) => param_hasher.update(i.to_le_bytes()),
                Value::Bool(b) => param_hasher.update([if *b { 1u8 } else { 0u8 }]),
                Value::Unit => param_hasher.update(b"unit"),
                Value::Symbol(symbol) => param_hasher.update(symbol.value.as_bytes()),
                Value::String(string) => param_hasher.update(string.value.as_bytes()),
                Value::Product(left, right) => {
                    // Hash both components of the product
                    let left_str = format!("{:?}", left);
                    let right_str = format!("{:?}", right);
                    param_hasher.update(left_str.as_bytes());
                    param_hasher.update(right_str.as_bytes());
                }
                Value::Sum { tag, value } => {
                    // Hash the tag and value
                    param_hasher.update([*tag]);
                    let value_str = format!("{:?}", value);
                    param_hasher.update(value_str.as_bytes());
                }
                Value::Record { fields } => {
                    // Hash all fields in the record
                    for (key, value) in fields {
                        param_hasher.update(key.as_bytes());
                        let value_str = format!("{:?}", value);
                        param_hasher.update(value_str.as_bytes());
                    }
                }
            }
        }
        let param_hash = param_hasher.finalize();
        
        // Create modified effect hash that includes parameter influence
        let mut combined_hash = effect_hash.hash;
        for i in 0..32 {
            combined_hash[i] ^= param_hash[i];
        }
        
        let modified_effect_hash = EffectHash { hash: combined_hash };
        Ok(ZkProof::mock_proof(modified_effect_hash))
    }
}

impl EffectHandler for ZkVerifiedEffectHandler {
    fn execute(&self, params: Vec<Value>) -> EffectResult {
        // Generate effect hash
        let effect_hash = EffectHash::from_params(self.effect_tag(), &params);
        
        if self.require_proofs {
            // In a full implementation, we would require a valid ZK proof here
            // For minimal implementation, we just log that proof would be required
            println!("ZK: Effect '{}' would require proof with hash {}", 
                    self.effect_tag(), effect_hash.to_hex());
        }
        
        // Execute the underlying handler
        self.inner_handler.execute(params)
    }
    
    fn effect_tag(&self) -> &str {
        self.inner_handler.effect_tag()
    }
    
    fn validate_params(&self, params: &[Value]) -> Result<()> {
        self.inner_handler.validate_params(params)
    }
    
    fn can_execute_with_capabilities(&self, capabilities: &[String]) -> bool {
        self.inner_handler.can_execute_with_capabilities(capabilities)
    }
}

/// Registry extension for ZK-verified effects
pub trait ZkEffectRegistry {
    /// Register a ZK-verified effect handler
    fn register_zk_handler(&self, handler: Arc<dyn EffectHandler>) -> Result<()>;
    
    /// Execute effect with ZK verification
    fn execute_zk_effect(&self, effect_tag: &str, params: Vec<Value>, proof: Option<&ZkProof>) -> EffectResult;
}

impl ZkEffectRegistry for EffectHandlerRegistry {
    fn register_zk_handler(&self, handler: Arc<dyn EffectHandler>) -> Result<()> {
        let zk_handler = Arc::new(ZkVerifiedEffectHandler::new(handler));
        self.register_handler(zk_handler)
    }
    
    fn execute_zk_effect(&self, effect_tag: &str, params: Vec<Value>, proof: Option<&ZkProof>) -> EffectResult {
        if let Some(proof) = proof {
            let effect_hash = EffectHash::from_params(effect_tag, &params);
            
            if proof.effect_hash != effect_hash {
                return Err(Error::serialization("ZK proof does not match effect hash"));
            }
            
            if !proof.verify() {
                return Err(Error::serialization("ZK proof verification failed"));
            }
            
            println!("ZK: Proof verified for effect '{}'", effect_tag);
        }
        
        self.execute_effect(effect_tag, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::handler_registry::SimpleEffectHandler;
    
    #[test]
    fn test_effect_hash_generation() {
        let params = vec![Value::Int(42), Value::Bool(true)];
        let hash1 = EffectHash::from_params("test_effect", &params);
        let hash2 = EffectHash::from_params("test_effect", &params);
        let hash3 = EffectHash::from_params("other_effect", &params);
        
        // Same effect and params should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different effect should produce different hash
        assert_ne!(hash1, hash3);
        
        // Hex conversion should work
        let hex = hash1.to_hex();
        assert_eq!(hex.len(), 64); // 32 bytes * 2 hex chars per byte
        
        let parsed = EffectHash::from_hex(&hex).unwrap();
        assert_eq!(hash1, parsed);
    }
    
    #[test]
    fn test_zk_proof_creation_and_verification() {
        let effect_hash = EffectHash::from_params("test", &[Value::Unit]);
        let proof = ZkProof::mock_proof(effect_hash.clone());
        
        assert_eq!(proof.effect_hash, effect_hash);
        assert!(proof.verify());
    }
    
    #[test]
    fn test_zk_verified_effect_handler() {
        let inner_handler = Arc::new(SimpleEffectHandler::new(
            "test".to_string(),
            |_| Ok(Value::Unit),
        ));
        
        let zk_handler = ZkVerifiedEffectHandler::new(inner_handler);
        
        assert_eq!(zk_handler.effect_tag(), "test");
        
        let result = zk_handler.execute(vec![Value::Int(42)]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_zk_effect_registry() {
        let registry = EffectHandlerRegistry::new();
        
        let handler = Arc::new(SimpleEffectHandler::new(
            "zk_test".to_string(),
            |_| Ok(Value::Bool(true)),
        ));
        
        registry.register_zk_handler(handler).unwrap();
        
        // Test execution without proof
        let result = registry.execute_zk_effect("zk_test", vec![], None);
        assert!(result.is_ok());
        
        // Test execution with valid proof
        let effect_hash = EffectHash::from_params("zk_test", &[]);
        let proof = ZkProof::mock_proof(effect_hash);
        let result = registry.execute_zk_effect("zk_test", vec![], Some(&proof));
        assert!(result.is_ok());
    }
} 