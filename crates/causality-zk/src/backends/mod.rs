//! ZK backend implementations

#[cfg(feature = "mock")]
pub mod mock_backend;

#[cfg(feature = "sp1")]
pub mod sp1_backend;

#[cfg(feature = "risc0")]
pub mod risc0_backend;

// Valence backend is always available since it uses HTTP client
pub mod valence_backend;

use crate::{ZkCircuit, ZkProof, ZkWitness, error::{ProofResult, VerificationError}};

/// Backend type enum for selecting ZK backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    #[cfg(feature = "mock")]
    Mock,
    #[cfg(feature = "sp1")]
    SP1,
    #[cfg(feature = "risc0")]
    Risc0,
    Valence,
}

/// Trait for ZK proof backends
pub trait ZkBackend: Send + Sync {
    /// Generate a proof from circuit and witness
    fn generate_proof(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> ProofResult<ZkProof>;
    
    /// Verify a proof with public inputs (simplified as i64 values)
    fn verify_proof(&self, proof: &ZkProof, public_inputs: &[i64]) -> Result<bool, VerificationError>;
    
    /// Get backend name for identification
    fn backend_name(&self) -> &'static str;
    
    /// Check if backend is available
    fn is_available(&self) -> bool;
}

/// Backend configuration for different backend types
#[derive(Debug, Clone)]
pub enum BackendConfig {
    #[cfg(feature = "mock")]
    Mock(mock_backend::MockConfig),
    #[cfg(feature = "sp1")]
    SP1(sp1_backend::Sp1Config),
    #[cfg(feature = "risc0")]
    Risc0, // TODO: Add Risc0Config when implemented
    Valence(valence_backend::ValenceConfig),
}

impl Default for BackendConfig {
    fn default() -> Self {
        #[cfg(feature = "mock")]
        return BackendConfig::Mock(mock_backend::MockConfig::default());
        
        #[cfg(all(not(feature = "mock"), feature = "sp1"))]
        return BackendConfig::SP1(sp1_backend::Sp1Config::default());
        
        // If no ZK features are enabled, default to Valence
        #[cfg(all(not(feature = "mock"), not(feature = "sp1")))]
        return BackendConfig::Valence(valence_backend::ValenceConfig::default());
    }
}

/// Create backend instance based on type
pub fn create_backend(backend_type: BackendType) -> Box<dyn ZkBackend> {
    match backend_type {
        #[cfg(feature = "mock")]
        BackendType::Mock => Box::new(mock_backend::MockBackend::new()),
        #[cfg(feature = "sp1")]
        BackendType::SP1 => Box::new(sp1_backend::Sp1Backend::new()),
        #[cfg(feature = "risc0")]
        BackendType::Risc0 => Box::new(risc0_backend::Risc0Backend::new()),
        BackendType::Valence => Box::new(valence_backend::ValenceBackend::new()),
    }
}

/// Create backend instance with configuration
pub fn create_backend_with_config(config: BackendConfig) -> Box<dyn ZkBackend> {
    match config {
        #[cfg(feature = "mock")]
        BackendConfig::Mock(config) => Box::new(mock_backend::MockBackend::with_config(config)),
        #[cfg(feature = "sp1")]
        BackendConfig::SP1(config) => Box::new(sp1_backend::Sp1Backend::with_config(config)),
        #[cfg(feature = "risc0")]
        BackendConfig::Risc0 => Box::new(risc0_backend::Risc0Backend::new()),
        BackendConfig::Valence(config) => Box::new(valence_backend::ValenceBackend::with_config(config)),
    }
}

/// Get default backend for current feature configuration
pub fn default_backend() -> Box<dyn ZkBackend> {
    create_backend_with_config(BackendConfig::default())
}

/// Get all available backends
pub fn available_backends() -> Vec<BackendType> {
    let mut backends = vec![BackendType::Valence]; // Valence is always available
    
    #[cfg(feature = "mock")]
    backends.push(BackendType::Mock);
    
    #[cfg(feature = "sp1")]
    backends.push(BackendType::SP1);
    
    #[cfg(feature = "risc0")]
    backends.push(BackendType::Risc0);
    
    backends
}

/// Check if a specific backend type is available
pub fn is_backend_available(backend_type: BackendType) -> bool {
    let available = available_backends();
    available.contains(&backend_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_availability() {
        let available = available_backends();
        assert!(!available.is_empty());
        assert!(available.contains(&BackendType::Valence));
        
        // Mock backend should be available in test builds
        #[cfg(feature = "mock")]
        assert!(available.contains(&BackendType::Mock));
    }
    
    #[test]
    fn test_backend_creation() {
        let backend = default_backend();
        assert!(!backend.backend_name().is_empty());
    }
    
    #[test]
    fn test_valence_backend_always_available() {
        assert!(is_backend_available(BackendType::Valence));
        
        let backend = create_backend(BackendType::Valence);
        assert_eq!(backend.backend_name(), "valence");
    }
    
    #[cfg(feature = "mock")]
    #[test]
    fn test_mock_backend_creation() {
        let backend = create_backend(BackendType::Mock);
        assert_eq!(backend.backend_name(), "mock");
    }
    
    #[cfg(feature = "sp1")]
    #[test]
    fn test_sp1_backend_creation() {
        let backend = create_backend(BackendType::SP1);
        assert_eq!(backend.backend_name(), "sp1");
    }
} 