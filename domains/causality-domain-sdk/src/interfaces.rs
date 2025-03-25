// Domain adapter interfaces
// Original file: src/domain_adapters/interfaces.rs

// Domain Adapter Interfaces
//
// This module provides standardized interfaces for domain adapters,
// with a focus on multi-VM support for cross-chain operations.

use std::any::Any;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;

use causality_types::{Error, Result};
use causality_types::DomainId;
use causality_core::{RiscVProgram, Witness, Proof};

/// Virtual Machine type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VmType {
    /// Succinct VM
    Succinct,
    /// Ethereum VM
    Evm,
    /// CosmWasm VM
    CosmWasm,
    /// General zkVM
    ZkVm,
    /// Custom VM
    Custom(String),
}

impl VmType {
    /// Get the name of the VM type
    pub fn name(&self) -> String {
        match self {
            VmType::Succinct => "succinct".to_string(),
            VmType::Evm => "evm".to_string(),
            VmType::CosmWasm => "cosmwasm".to_string(),
            VmType::ZkVm => "zkvm".to_string(),
            VmType::Custom(name) => name.clone(),
        }
    }
    
    /// Create a VmType from a string
    pub fn from_str(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "succinct" => VmType::Succinct,
            "evm" => VmType::Evm,
            "cosmwasm" => VmType::CosmWasm,
            "zkvm" => VmType::ZkVm,
            _ => VmType::Custom(name.to_string()),
        }
    }
}

/// Multi-VM adapter configuration
#[derive(Debug, Clone)]
pub struct MultiVmAdapterConfig {
    /// Domain ID
    pub domain_id: DomainId,
    /// VM type
    pub vm_type: VmType,
    /// API endpoints
    pub api_endpoints: Vec<String>,
    /// Authentication data
    pub auth: Option<HashMap<String, String>>,
    /// Debug mode
    pub debug_mode: bool,
    /// Additional configuration
    pub extra_config: HashMap<String, String>,
}

impl Default for MultiVmAdapterConfig {
    fn default() -> Self {
        Self {
            domain_id: DomainId::new("default"),
            vm_type: VmType::ZkVm,
            api_endpoints: vec![],
            auth: None,
            debug_mode: false,
            extra_config: HashMap::new(),
        }
    }
}

/// Base trait for all VM adapters
pub trait VmAdapter: Debug + Send + Sync {
    /// Get the VM type this adapter supports
    fn vm_type(&self) -> VmType;
    
    /// Get the domain ID this adapter is associated with
    fn domain_id(&self) -> &DomainId;
    
    /// Check if this adapter supports a specific feature
    fn supports_feature(&self, feature: &str) -> bool;
    
    /// Get adapter configuration
    fn config(&self) -> MultiVmAdapterConfig;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Trait for adapters that support program compilation
pub trait CompilationAdapter: VmAdapter {
    /// Compile a program from source code
    fn compile_program(&mut self, source: &str, name: Option<&str>) -> Result<RiscVProgram>;
    
    /// Get supported source languages
    fn supported_languages(&self) -> Vec<String>;
}

/// Trait for adapters that support ZK proofs
pub trait ZkProofAdapter: VmAdapter {
    /// Generate a witness for a program
    fn generate_witness(
        &self,
        program: &RiscVProgram,
        public_inputs: &HashMap<String, Vec<u8>>,
        private_inputs: &HashMap<String, Vec<u8>>,
    ) -> Result<Witness>;
    
    /// Generate a proof from a witness
    fn generate_proof(
        &self,
        program: &RiscVProgram,
        witness: &Witness,
        options: Option<HashMap<String, String>>,
    ) -> Result<Proof>;
    
    /// Verify a proof
    fn verify_proof(
        &self,
        program: &RiscVProgram,
        proof: &Proof,
        public_inputs: &HashMap<String, Vec<u8>>,
    ) -> Result<bool>;
}

/// Trait for adapters that support cross-VM operations
pub trait CrossVmAdapter: VmAdapter {
    /// Get supported target VM types for cross-VM operations
    fn supported_target_vms(&self) -> Vec<VmType>;
    
    /// Translate a program to a format compatible with a target VM
    fn translate_program(
        &self,
        program: &RiscVProgram,
        target_vm: &VmType,
    ) -> Result<Vec<u8>>;
    
    /// Execute a program on a remote VM
    fn execute_remote(
        &self,
        program: &RiscVProgram,
        target_vm: &VmType,
        inputs: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>>;
}

/// Factory for creating VM adapters
pub trait VmAdapterFactory: Debug + Send + Sync {
    /// Create a new VM adapter
    fn create_adapter(
        &self,
        config: MultiVmAdapterConfig,
    ) -> Result<Box<dyn VmAdapter>>;
    
    /// Get the VM type this factory supports
    fn vm_type(&self) -> VmType;
    
    /// Get the name of this factory
    fn name(&self) -> &str;
}

/// Registry for VM adapters
#[derive(Debug, Default)]
pub struct VmAdapterRegistry {
    /// Registered adapter factories by VM type
    factories: HashMap<VmType, Box<dyn VmAdapterFactory>>,
    /// Adapter instances by domain ID
    adapters: HashMap<DomainId, Box<dyn VmAdapter>>,
}

impl VmAdapterRegistry {
    /// Create a new VM adapter registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            adapters: HashMap::new(),
        }
    }
    
    /// Register an adapter factory
    pub fn register_factory(&mut self, factory: Box<dyn VmAdapterFactory>) -> Result<()> {
        let vm_type = factory.vm_type();
        self.factories.insert(vm_type, factory);
        Ok(())
    }
    
    /// Create and register an adapter
    pub fn create_adapter(&mut self, config: MultiVmAdapterConfig) -> Result<&Box<dyn VmAdapter>> {
        let vm_type = config.vm_type.clone();
        let domain_id = config.domain_id.clone();
        
        // Get the factory for this VM type
        let factory = self.factories.get(&vm_type).ok_or_else(|| {
            Error::NotFoundError(format!("No factory found for VM type: {:?}", vm_type))
        })?;
        
        // Create the adapter
        let adapter = factory.create_adapter(config)?;
        
        // Register the adapter
        self.adapters.insert(domain_id.clone(), adapter);
        
        // Return a reference to the adapter
        self.adapters.get(&domain_id).ok_or_else(|| {
            Error::NotFoundError(format!("Failed to retrieve created adapter for domain: {}", domain_id.as_ref()))
        })
    }
    
    /// Get an adapter by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<&Box<dyn VmAdapter>> {
        self.adapters.get(domain_id)
    }
    
    /// Get an adapter by domain ID and cast to a specific adapter type
    pub fn get_adapter_as<T: 'static>(&self, domain_id: &DomainId) -> Option<&T> {
        self.get_adapter(domain_id)
            .and_then(|adapter| adapter.as_any().downcast_ref::<T>())
    }
    
    /// Remove an adapter
    pub fn remove_adapter(&mut self, domain_id: &DomainId) -> Result<()> {
        self.adapters.remove(domain_id);
        Ok(())
    }
    
    /// Check if an adapter exists
    pub fn has_adapter(&self, domain_id: &DomainId) -> bool {
        self.adapters.contains_key(domain_id)
    }
    
    /// Get all registered VM types
    pub fn vm_types(&self) -> Vec<VmType> {
        self.factories.keys().cloned().collect()
    }
    
    /// Get all registered domain IDs
    pub fn domain_ids(&self) -> Vec<DomainId> {
        self.adapters.keys().cloned().collect()
    }
}

/// Shared adapter provider for dependency injection
#[derive(Debug, Clone)]
pub struct SharedAdapterProvider {
    registry: Arc<std::sync::RwLock<VmAdapterRegistry>>,
}

impl SharedAdapterProvider {
    /// Create a new shared adapter provider
    pub fn new(registry: VmAdapterRegistry) -> Self {
        Self {
            registry: Arc::new(std::sync::RwLock::new(registry)),
        }
    }
    
    /// Get an adapter by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn VmAdapter>> {
        let registry = self.registry.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on adapter registry".to_string())
        })?;
        
        let adapter = registry.get_adapter(domain_id).ok_or_else(|| {
            Error::NotFoundError(format!("No adapter found for domain: {}", domain_id.as_ref()))
        })?;
        
        // This is a limitation - we can't easily convert a &Box<dyn VmAdapter> to an Arc<dyn VmAdapter>
        // In a real implementation, we might keep Arc<dyn VmAdapter> in the registry
        // For now, we'll return an error to indicate this limitation
        Err(Error::NotImplemented("Shared adapter access not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_type() {
        assert_eq!(VmType::Succinct.name(), "succinct");
        assert_eq!(VmType::from_str("evm"), VmType::Evm);
        assert_eq!(VmType::from_str("custom"), VmType::Custom("custom".to_string()));
    }
} 