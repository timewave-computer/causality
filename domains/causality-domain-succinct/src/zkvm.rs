// zkVM implementation
// Original file: src/domain_adapters/zkvm.rs

//! ZK-VM Powered Domain Adapters
//!
//! This module provides the foundation for domain adapters that use ZK-VM backends
//! like RISC Zero or Succinct for generating zero-knowledge proofs of effect execution.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use causality_types::DomainId;
use crate::domain_adapters::{
    interfaces::{
        VmType,
        VmAdapter,
        CompilationAdapter,
        ZkProofAdapter,
        VmAdapterFactory,
    },
    validation::{
        ValidationContext,
        ValidationResult,
        EffectValidator,
    },
};

/// ZK-VM backend type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZkVmBackend {
    /// RISC Zero VM backend
    RiscZero,
    /// Succinct VM backend
    Succinct,
    /// Custom ZK-VM backend
    Custom(String),
}

impl ZkVmBackend {
    /// Get the name of the ZK-VM backend
    pub fn name(&self) -> String {
        match self {
            Self::RiscZero => "risc0".to_string(),
            Self::Succinct => "succinct".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
    
    /// Create a ZK-VM backend from a string
    pub fn from_str(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "risc0" => Ok(Self::RiscZero),
            "succinct" => Ok(Self::Succinct),
            _ => Ok(Self::Custom(name.to_string())),
        }
    }
}

/// Configuration for ZK-VM adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVmAdapterConfig {
    /// Domain ID for this adapter
    pub domain_id: DomainId,
    /// Target blockchain VM type
    pub target_vm_type: VmType,
    /// ZK-VM backend to use
    pub zkvm_backend: ZkVmBackend,
    /// Guest program ELF binary path (for local execution)
    pub guest_program_path: Option<String>,
    /// Guest program image ID (for remote execution)
    pub guest_program_id: Option<String>,
    /// API endpoint for remote proving service
    pub proving_api_endpoint: Option<String>,
    /// Authentication token for proving service
    pub auth_token: Option<String>,
    /// Debug mode flag
    pub debug_mode: bool,
    /// Additional configuration parameters
    pub extra_config: HashMap<String, String>,
}

/// Base trait for ZK-VM-powered domain adapters
pub trait ZkVmDomainAdapter: VmAdapter + CompilationAdapter + ZkProofAdapter {
    /// Get the ZK-VM backend type
    fn zkvm_backend(&self) -> &ZkVmBackend;
    
    /// Get the target VM type (e.g., EVM, Solana VM)
    fn target_vm_type(&self) -> VmType;
    
    /// Generate a ZK proof for the given effect
    fn generate_proof(
        &self,
        effect_type: &str,
        params: &serde_json::Value,
        private_inputs: &serde_json::Value,
    ) -> Result<ZkProof>;
    
    /// Verify a ZK proof on the target blockchain
    fn verify_proof_on_chain(
        &self,
        proof: &ZkProof,
        verifier_contract: Option<&str>,
    ) -> Result<String>;
    
    /// Get the proof verification data for the target blockchain
    fn get_verification_data(
        &self,
        proof: &ZkProof,
    ) -> Result<serde_json::Value>;
}

/// ZK proof data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    /// ZK-VM backend type
    pub backend: ZkVmBackend,
    /// Proof data (format depends on backend)
    pub proof_data: Vec<u8>,
    /// Public inputs for verification
    pub public_inputs: Vec<String>,
    /// Target VM type for verification
    pub target_vm: VmType,
    /// Proof generation metadata
    pub metadata: HashMap<String, String>,
}

impl ZkProof {
    /// Create a new ZK proof
    pub fn new(
        backend: ZkVmBackend,
        proof_data: Vec<u8>,
        public_inputs: Vec<String>,
        target_vm: VmType,
    ) -> Self {
        Self {
            backend,
            proof_data,
            public_inputs,
            target_vm,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the proof
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Base implementation for ZK-VM-powered domain adapters
pub struct BaseZkVmAdapter<T: ZkVmDomainAdapter> {
    /// The inner adapter implementation
    inner: T,
    /// Validator for effect validation
    validator: Box<dyn EffectValidator>,
}

impl<T: ZkVmDomainAdapter> BaseZkVmAdapter<T> {
    /// Create a new base ZK-VM adapter
    pub fn new(inner: T, validator: Box<dyn EffectValidator>) -> Self {
        Self { inner, validator }
    }
    
    /// Get a reference to the inner adapter
    pub fn inner(&self) -> &T {
        &self.inner
    }
    
    /// Validate an effect before execution
    pub fn validate_effect(
        &self, 
        effect_type: &str, 
        params: &serde_json::Value,
    ) -> Result<ValidationResult> {
        let context = ValidationContext::new(
            self.inner.domain_id().clone(),
            self.inner.target_vm_type(),
            effect_type.to_string(),
        ).with_params(params.clone());
        
        self.validator.validate(&context)
    }
}

/// Factory for creating ZK-VM-powered domain adapters
pub trait ZkVmAdapterFactory<T: ZkVmDomainAdapter>: VmAdapterFactory {
    /// Create a new ZK-VM adapter from configuration
    fn create_zkvm_adapter(&self, config: ZkVmAdapterConfig) -> Result<T>;
    
    /// Get the supported ZK-VM backends
    fn supported_zkvm_backends(&self) -> Vec<ZkVmBackend>;
    
    /// Get the supported target VM types
    fn supported_target_vms(&self) -> Vec<VmType>;
    
    /// Check if a ZK-VM backend is supported
    fn supports_zkvm_backend(&self, backend: &ZkVmBackend) -> bool {
        self.supported_zkvm_backends().contains(backend)
    }
    
    /// Check if a target VM type is supported
    fn supports_target_vm(&self, vm_type: &VmType) -> bool {
        self.supported_target_vms().contains(vm_type)
    }
}

/// Registry for ZK-VM-powered domain adapters
#[derive(Debug, Default)]
pub struct ZkVmAdapterRegistry {
    /// Registered factories for ZK-VM adapters
    factories: HashMap<String, Box<dyn VmAdapterFactory>>,
    /// Registered ZK-VM adapters
    adapters: HashMap<DomainId, Box<dyn VmAdapter>>,
}

impl ZkVmAdapterRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            adapters: HashMap::new(),
        }
    }
    
    /// Register a ZK-VM adapter factory
    pub fn register_factory(&mut self, factory: Box<dyn VmAdapterFactory>) -> Result<()> {
        let name = factory.name().to_string();
        self.factories.insert(name, factory);
        Ok(())
    }
    
    /// Register a ZK-VM adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn VmAdapter>) -> Result<()> {
        let domain_id = adapter.domain_id().clone();
        self.adapters.insert(domain_id, adapter);
        Ok(())
    }
    
    /// Get a factory by name
    pub fn get_factory(&self, name: &str) -> Option<&Box<dyn VmAdapterFactory>> {
        self.factories.get(name)
    }
    
    /// Get an adapter by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<&Box<dyn VmAdapter>> {
        self.adapters.get(domain_id)
    }
    
    /// List all factories
    pub fn list_factories(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
    
    /// List all adapters
    pub fn list_adapters(&self) -> Vec<DomainId> {
        self.adapters.keys().cloned().collect()
    }
} 