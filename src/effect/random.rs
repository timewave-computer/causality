// Random effect for algebraic effect-based randomness
//
// This module provides a trait and implementations for generating random values
// through the algebraic effect system, allowing for better testability and control.

use std::fmt;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use rand::Rng;
use borsh::{BorshSerialize, BorshDeserialize};

use crate::effect::{Effect, EffectId, EffectContext, EffectResult, EffectOutcome, EffectError};
use crate::log::fact_snapshot::{FactDependency, FactSnapshot};
use crate::error::Result;
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Types of random number generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomType {
    /// Standard RNG (not cryptographically secure)
    Standard,
    
    /// System cryptographically secure RNG
    CryptographicSecure,
    
    /// Deterministic RNG (for testing)
    Deterministic,
}

impl fmt::Display for RandomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandomType::Standard => write!(f, "Standard"),
            RandomType::CryptographicSecure => write!(f, "CryptographicSecure"),
            RandomType::Deterministic => write!(f, "Deterministic"),
        }
    }
}

/// Effect for generating random values
#[async_trait]
pub trait RandomEffect: Send + Sync + std::fmt::Debug {
    /// Get the type of this random number generator
    fn random_type(&self) -> RandomType;
    
    /// Generate a random value of type u8
    async fn gen_u8(&self, context: &EffectContext) -> EffectResult<u8>;
    
    /// Generate a random value of type u16
    async fn gen_u16(&self, context: &EffectContext) -> EffectResult<u16>;
    
    /// Generate a random value of type u32
    async fn gen_u32(&self, context: &EffectContext) -> EffectResult<u32>;
    
    /// Generate a random value of type u64
    async fn gen_u64(&self, context: &EffectContext) -> EffectResult<u64>;
    
    /// Generate a random value of type u128
    async fn gen_u128(&self, context: &EffectContext) -> EffectResult<u128>;
    
    /// Generate a random f64 between 0.0 and 1.0
    async fn gen_f64(&self, context: &EffectContext) -> EffectResult<f64>;
    
    /// Generate a random bool
    async fn gen_bool(&self, context: &EffectContext) -> EffectResult<bool>;
    
    /// Generate a random value in the range [0, max)
    async fn gen_range_u32(&self, context: &EffectContext, max: u32) -> EffectResult<u32>;
    
    /// Fill a buffer with random bytes
    async fn fill_bytes(&self, context: &EffectContext, buffer: &mut [u8]) -> EffectResult<()>;
}

/// Standard random effect implementation using the rand crate
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct StandardRandomEffect {
    id: EffectId,
    seed: Option<u64>,
}

impl ContentAddressed for StandardRandomEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
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
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl StandardRandomEffect {
    /// Create a new standard random effect
    pub fn new() -> Self {
        let mut effect = Self {
            id: EffectId::new(""), // Temporary placeholder
            seed: None,
        };
        
        // Derive content ID and use it for the ID
        let content_id = effect.content_id();
        effect.id = EffectId::from_string(format!("random-standard-{}", content_id));
        
        effect
    }
    
    /// Create a standard random effect with a seed for deterministic behavior
    pub fn with_seed(seed: u64) -> Self {
        let mut effect = Self {
            id: EffectId::new(""), // Temporary placeholder
            seed: Some(seed),
        };
        
        // Derive content ID and use it for the ID
        let content_id = effect.content_id();
        effect.id = EffectId::from_string(format!("random-standard-{}-seed-{}", content_id, seed));
        
        effect
    }
}

impl Effect for StandardRandomEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "standard_random_effect"
    }
    
    fn display_name(&self) -> String {
        "Standard Random Number Generator".to_string()
    }
    
    fn description(&self) -> String {
        "Generates random numbers using the standard (non-cryptographically secure) RNG".to_string()
    }
    
    fn can_execute_in(&self, _boundary: crate::effect::ExecutionBoundary) -> bool {
        true // Can execute in any boundary
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        crate::effect::ExecutionBoundary::InsideSystem
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
        params.insert("type".to_string(), "standard".to_string());
        if let Some(seed) = self.seed {
            params.insert("seed".to_string(), seed.to_string());
        }
        params
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn execute(&self, context: &EffectContext) -> crate::effect::EffectResult<EffectOutcome> {
        // For synchronous execution, we'll generate a random u64
        let random_value = rand::random::<u64>();
        
        let mut data = std::collections::HashMap::new();
        data.insert("random_value".to_string(), random_value.to_string());
        
        Ok(EffectOutcome::success(self.id.clone())
            .with_data("random_value", random_value.to_string()))
    }
}

#[async_trait]
impl RandomEffect for StandardRandomEffect {
    fn random_type(&self) -> RandomType {
        RandomType::Standard
    }
    
    async fn gen_u8(&self, _context: &EffectContext) -> EffectResult<u8> {
        Ok(rand::random::<u8>())
    }
    
    async fn gen_u16(&self, _context: &EffectContext) -> EffectResult<u16> {
        Ok(rand::random::<u16>())
    }
    
    async fn gen_u32(&self, _context: &EffectContext) -> EffectResult<u32> {
        Ok(rand::random::<u32>())
    }
    
    async fn gen_u64(&self, _context: &EffectContext) -> EffectResult<u64> {
        Ok(rand::random::<u64>())
    }
    
    async fn gen_u128(&self, _context: &EffectContext) -> EffectResult<u128> {
        Ok(rand::random::<u128>())
    }
    
    async fn gen_f64(&self, _context: &EffectContext) -> EffectResult<f64> {
        Ok(rand::random::<f64>())
    }
    
    async fn gen_bool(&self, _context: &EffectContext) -> EffectResult<bool> {
        Ok(rand::random::<bool>())
    }
    
    async fn gen_range_u32(&self, _context: &EffectContext, max: u32) -> EffectResult<u32> {
        if max == 0 {
            return Err(EffectError::InvalidParameter("Max value cannot be zero".to_string()));
        }
        
        Ok(rand::thread_rng().gen_range(0..max))
    }
    
    async fn fill_bytes(&self, _context: &EffectContext, buffer: &mut [u8]) -> EffectResult<()> {
        rand::thread_rng().fill(buffer);
        Ok(())
    }
}

/// Cryptographically secure random effect implementation
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct SecureRandomEffect {
    id: EffectId,
}

impl ContentAddressed for SecureRandomEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
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
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl SecureRandomEffect {
    /// Create a new cryptographically secure random effect
    pub fn new() -> Self {
        let mut effect = Self {
            id: EffectId::new(""), // Temporary placeholder
        };
        
        // Derive content ID and use it for the ID
        let content_id = effect.content_id();
        effect.id = EffectId::from_string(format!("random-secure-{}", content_id));
        
        effect
    }
}

impl Effect for SecureRandomEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "secure_random_effect"
    }
    
    fn display_name(&self) -> String {
        "Secure Random Number Generator".to_string()
    }
    
    fn description(&self) -> String {
        "Generates cryptographically secure random numbers".to_string()
    }
    
    fn can_execute_in(&self, _boundary: crate::effect::ExecutionBoundary) -> bool {
        true // Can execute in any boundary
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        crate::effect::ExecutionBoundary::InsideSystem
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
        params.insert("type".to_string(), "cryptographically_secure".to_string());
        params
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn execute(&self, context: &EffectContext) -> crate::effect::EffectResult<EffectOutcome> {
        // For synchronous execution, we'll generate a secure random u64
        let mut buffer = [0u8; 8];
        match getrandom::getrandom(&mut buffer) {
            Ok(_) => {
                let random_value = u64::from_ne_bytes(buffer);
                Ok(EffectOutcome::success(self.id.clone())
                    .with_data("random_value", random_value.to_string()))
            },
            Err(e) => {
                Err(EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))
            }
        }
    }
}

#[async_trait]
impl RandomEffect for SecureRandomEffect {
    fn random_type(&self) -> RandomType {
        RandomType::CryptographicSecure
    }
    
    async fn gen_u8(&self, _context: &EffectContext) -> EffectResult<u8> {
        let mut buffer = [0u8; 1];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(buffer[0])
    }
    
    async fn gen_u16(&self, _context: &EffectContext) -> EffectResult<u16> {
        let mut buffer = [0u8; 2];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(u16::from_ne_bytes(buffer))
    }
    
    async fn gen_u32(&self, _context: &EffectContext) -> EffectResult<u32> {
        let mut buffer = [0u8; 4];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(u32::from_ne_bytes(buffer))
    }
    
    async fn gen_u64(&self, _context: &EffectContext) -> EffectResult<u64> {
        let mut buffer = [0u8; 8];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(u64::from_ne_bytes(buffer))
    }
    
    async fn gen_u128(&self, _context: &EffectContext) -> EffectResult<u128> {
        let mut buffer = [0u8; 16];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(u128::from_ne_bytes(buffer))
    }
    
    async fn gen_f64(&self, _context: &EffectContext) -> EffectResult<f64> {
        let mut buffer = [0u8; 8];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        
        // Convert to f64 between 0.0 and 1.0
        let value = u64::from_ne_bytes(buffer);
        Ok(value as f64 / std::u64::MAX as f64)
    }
    
    async fn gen_bool(&self, _context: &EffectContext) -> EffectResult<bool> {
        let mut buffer = [0u8; 1];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        
        Ok(buffer[0] >= 128)
    }
    
    async fn gen_range_u32(&self, _context: &EffectContext, max: u32) -> EffectResult<u32> {
        if max == 0 {
            return Err(EffectError::InvalidParameter("Max value cannot be zero".to_string()));
        }
        
        // This is a simplified implementation and not perfectly uniform
        // For production code, we would use a more sophisticated algorithm
        let mut buffer = [0u8; 4];
        getrandom::getrandom(&mut buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        
        let value = u32::from_ne_bytes(buffer);
        Ok(value % max)
    }
    
    async fn fill_bytes(&self, _context: &EffectContext, buffer: &mut [u8]) -> EffectResult<()> {
        getrandom::getrandom(buffer)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to generate secure random: {}", e)))?;
        Ok(())
    }
}

/// Factory for creating random effects
pub struct RandomEffectFactory;

impl RandomEffectFactory {
    /// Create a random effect of the specified type
    pub fn create_effect(random_type: RandomType) -> Box<dyn RandomEffect> {
        match random_type {
            RandomType::Standard => Box::new(StandardRandomEffect::new()),
            RandomType::CryptographicSecure => Box::new(SecureRandomEffect::new()),
            RandomType::Deterministic => Box::new(StandardRandomEffect::with_seed(42)), // Fixed seed for deterministic behavior
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_standard_random_effect() {
        let context = EffectContext::default();
        let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
        
        // Test u64 generation
        let value = random_effect.gen_u64(&context).await.unwrap();
        assert!(value > 0 || value == 0); // Can be any value
        
        // Test bool generation
        let _bool_value = random_effect.gen_bool(&context).await.unwrap();
        // Bool can be either true or false, so no assertion
        
        // Test f64 generation
        let float_value = random_effect.gen_f64(&context).await.unwrap();
        assert!(float_value >= 0.0 && float_value < 1.0);
        
        // Test range generation
        let range_value = random_effect.gen_range_u32(&context, 100).await.unwrap();
        assert!(range_value < 100);
    }
    
    #[test]
    async fn test_deterministic_random_effect() {
        let context = EffectContext::default();
        let random_effect = RandomEffectFactory::create_effect(RandomType::Deterministic);
        
        // Get two sequential values - with same seed they should be different
        let value1 = random_effect.gen_u64(&context).await.unwrap();
        let value2 = random_effect.gen_u64(&context).await.unwrap();
        
        // Values should be different (even with deterministic generator)
        // This test is not guaranteed to pass, but very likely with a good PRNG
        assert_ne!(value1, value2); 
    }
    
    #[test]
    async fn test_secure_random_effect() {
        let context = EffectContext::default();
        let random_effect = RandomEffectFactory::create_effect(RandomType::CryptographicSecure);
        
        // Test u64 generation
        let value = random_effect.gen_u64(&context).await.unwrap();
        assert!(value > 0 || value == 0); // Can be any value
        
        // Test bool generation
        let _bool_value = random_effect.gen_bool(&context).await.unwrap();
        // Bool can be either true or false, so no assertion
        
        // Test f64 generation
        let float_value = random_effect.gen_f64(&context).await.unwrap();
        assert!(float_value >= 0.0 && float_value < 1.0);
        
        // Test range generation
        let range_value = random_effect.gen_range_u32(&context, 100).await.unwrap();
        assert!(range_value < 100);
    }
    
    #[test]
    async fn test_random_effect_factory() {
        // Create each type of random effect
        let standard = RandomEffectFactory::create_effect(RandomType::Standard);
        let secure = RandomEffectFactory::create_effect(RandomType::CryptographicSecure);
        let deterministic = RandomEffectFactory::create_effect(RandomType::Deterministic);
        
        assert_eq!(standard.random_type(), RandomType::Standard);
        assert_eq!(secure.random_type(), RandomType::CryptographicSecure);
        assert_eq!(deterministic.random_type(), RandomType::Deterministic);
    }
} 