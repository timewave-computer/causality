// Domain-specific adapter implementations
//
// This module contains adapter implementations for specific blockchain domains
// as well as schema definitions for effect adapters.

// Schema definitions
pub mod schemas;

// New multi-VM interfaces
pub mod interfaces;

// Shared utilities for cross-adapter operations
pub mod utils;

// Standardized testing framework
pub mod testing;

// Domain-specific effect validation protocols
pub mod validation;

// Cross-VM effect coordination
pub mod coordination;

// EVM domains
pub mod evm;

// CosmWasm domains
pub mod cosmwasm;

// ZK-VM powered adapters
pub mod zkvm;

// ZK-VM powered EVM adapter
pub mod zkvm_evm;

// Integration tests
#[cfg(test)]
pub mod integration_tests;

// Tests for ZK-VM powered EVM adapter
#[cfg(test)]
pub mod zkvm_evm_test;

// Re-export commonly used schema types
pub use schemas::{
    AdapterSchema, 
    DomainId, 
    EffectDefinition, 
    FactDefinition, 
    ProofDefinition
};

// Import and export domain-specific adapters
pub mod succinct;

// Re-export important types for convenience
pub use self::succinct::{
    SuccinctAdapter,
    ProofData,
    PublicInputs,
};

// Re-export multi-VM adapter interfaces
pub use interfaces::{
    VmType,
    VmAdapter,
    CompilationAdapter,
    ZkProofAdapter,
    CrossVmAdapter,
    VmAdapterFactory,
    VmAdapterRegistry,
    MultiVmAdapterConfig,
    SharedAdapterProvider,
};

// Re-export cross-VM utilities
pub use utils::{
    CrossVmBroker,
    CrossVmHandler,
    CrossVmRequest,
    CrossVmResponse,
    CrossVmStatus,
    translate_proof,
    translate_program,
    cross_vm_verify,
    create_adapter_config,
};

// Re-export testing utilities
pub use testing::{
    AdapterTestConfig,
    AdapterTestCase,
    CompilationTest,
    ZkProofTest,
    CrossVmTest,
    create_test_inputs,
    create_test_program,
    create_test_source_code,
};

// Re-export validation utilities
pub use validation::{
    ValidationContext,
    ValidationRule,
    ValidationRuleType,
    ValidationResult,
    ValidationError,
    EffectValidator,
    EffectValidatorFactory,
    EffectValidatorRegistry,
    validate_common_rules,
};

// Re-export coordination utilities
pub use coordination::{
    CoordinationContext,
    CoordinationStep,
    CoordinationPlan,
    CoordinationStatus,
    CoordinationHandler,
    CoordinationExecutor,
    ProofVerificationHandler,
    CoordinationPlanFactory,
};

// Re-export CosmWasm types
pub use cosmwasm::{
    CosmWasmAdapter,
    CosmWasmAdapterConfig,
    CosmWasmAdapterFactory,
    CosmWasmAddress,
    CosmWasmCode,
    CosmWasmMessage,
    CosmWasmQueryResult,
    CosmWasmExecutionResult,
    CosmWasmEffectValidator,
    COSMWASM_VM_TYPE,
};

// Re-export ZK-VM types
pub use zkvm::{
    ZkVmBackend,
    ZkVmAdapterConfig,
    ZkVmDomainAdapter,
    ZkProof,
    BaseZkVmAdapter,
    ZkVmAdapterFactory,
    ZkVmAdapterRegistry,
};

// Re-export ZK-EVM types
pub use zkvm_evm::{
    ZkEvmAdapter,
    ZkEvmAdapterConfig,
    ZkEvmEffectType,
    ZkEvmEffectValidator,
    ZkEvmAdapterFactory,
};

// Domain adapter registry - holds available adapters
#[derive(Debug, Default)]
pub struct DomainAdapterRegistry {
    // Available VM adapters
    vm_adapters: std::collections::HashMap<String, Box<dyn VmAdapter>>,
    // Multi-VM registry
    vm_registry: VmAdapterRegistry,
    // ZK-VM registry
    zkvm_registry: zkvm::ZkVmAdapterRegistry,
}

impl DomainAdapterRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            vm_adapters: std::collections::HashMap::new(),
            vm_registry: VmAdapterRegistry::new(),
            zkvm_registry: zkvm::ZkVmAdapterRegistry::new(),
        }
    }
    
    /// Register a VM adapter with the registry
    pub fn register_vm_adapter(&mut self, adapter: Box<dyn VmAdapter>) {
        let name = adapter.vm_type().name();
        self.vm_adapters.insert(name, adapter);
    }
    
    /// Get a VM adapter by name
    pub fn get_vm_adapter(&self, name: &str) -> Option<&Box<dyn VmAdapter>> {
        self.vm_adapters.get(name)
    }
    
    /// Check if a VM adapter exists
    pub fn has_vm_adapter(&self, name: &str) -> bool {
        self.vm_adapters.contains_key(name)
    }
    
    /// Get the VM adapter registry
    pub fn vm_registry(&self) -> &VmAdapterRegistry {
        &self.vm_registry
    }
    
    /// Get a mutable reference to the VM adapter registry
    pub fn vm_registry_mut(&mut self) -> &mut VmAdapterRegistry {
        &mut self.vm_registry
    }
    
    /// Get the ZK-VM adapter registry
    pub fn zkvm_registry(&self) -> &zkvm::ZkVmAdapterRegistry {
        &self.zkvm_registry
    }
    
    /// Get a mutable reference to the ZK-VM adapter registry
    pub fn zkvm_registry_mut(&mut self) -> &mut zkvm::ZkVmAdapterRegistry {
        &mut self.zkvm_registry
    }
    
    /// Initialize with default adapters
    pub fn with_defaults(mut self) -> crate::error::Result<Self> {
        // Add Succinct adapter as a VM adapter
        let succinct_config = MultiVmAdapterConfig {
            domain_id: DomainId::new("succinct"),
            vm_type: VmType::Succinct,
            api_endpoints: vec![],
            auth: None,
            debug_mode: false,
            extra_config: std::collections::HashMap::new(),
        };
        
        // Register ZK-EVM adapter factory
        let zkvm_evm_factory = Box::new(zkvm_evm::ZkEvmAdapterFactory::new());
        self.zkvm_registry_mut().register_factory(zkvm_evm_factory)?;
        
        // In a real implementation, we would use a factory to create the adapter
        // For now, we'll just create a placeholder adapter
        
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry() {
        let registry = DomainAdapterRegistry::new();
        assert!(!registry.has_vm_adapter("test"));
    }
} 