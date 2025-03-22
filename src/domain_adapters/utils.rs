// Shared Utilities for Cross-Adapter Operations
//
// This module provides utilities for working with multiple VM adapters
// and facilitating cross-VM operations.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::types::DomainId;
use crate::zk::{RiscVProgram, Witness, Proof};
use crate::domain_adapters::interfaces::{
    VmAdapter, 
    VmType, 
    ZkProofAdapter, 
    CrossVmAdapter,
    MultiVmAdapterConfig
};

/// Cross-VM operation request
#[derive(Debug, Clone)]
pub struct CrossVmRequest {
    /// Source domain ID
    pub source_domain: DomainId,
    /// Target domain ID
    pub target_domain: DomainId,
    /// Operation name
    pub operation: String,
    /// Program to execute
    pub program: Option<RiscVProgram>,
    /// Witness data
    pub witness: Option<Witness>,
    /// Proof data
    pub proof: Option<Proof>,
    /// Public inputs
    pub public_inputs: HashMap<String, Vec<u8>>,
    /// Private inputs
    pub private_inputs: HashMap<String, Vec<u8>>,
    /// Operation options
    pub options: HashMap<String, String>,
}

/// Cross-VM operation response
#[derive(Debug, Clone)]
pub struct CrossVmResponse {
    /// Operation status
    pub status: CrossVmStatus,
    /// Result data
    pub data: Vec<u8>,
    /// Generated proof (if applicable)
    pub proof: Option<Proof>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Cross-VM operation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossVmStatus {
    /// Operation successful
    Success,
    /// Operation failed
    Failure,
    /// Operation pending
    Pending,
    /// Operation partially complete
    Partial,
}

/// Cross-VM operation broker
///
/// This struct manages cross-VM operations between different adapters.
#[derive(Debug)]
pub struct CrossVmBroker {
    /// Registered adapters by domain ID
    adapters: HashMap<DomainId, Box<dyn VmAdapter>>,
    /// Cross-VM operation handlers
    handlers: HashMap<(VmType, VmType), Box<dyn CrossVmHandler>>,
}

impl CrossVmBroker {
    /// Create a new cross-VM broker
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            handlers: HashMap::new(),
        }
    }
    
    /// Register an adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn VmAdapter>) -> Result<()> {
        let domain_id = adapter.domain_id().clone();
        self.adapters.insert(domain_id, adapter);
        Ok(())
    }
    
    /// Register a cross-VM handler
    pub fn register_handler(
        &mut self,
        source_vm: VmType,
        target_vm: VmType,
        handler: Box<dyn CrossVmHandler>,
    ) -> Result<()> {
        self.handlers.insert((source_vm, target_vm), handler);
        Ok(())
    }
    
    /// Execute a cross-VM operation
    pub fn execute(
        &self,
        request: CrossVmRequest,
    ) -> Result<CrossVmResponse> {
        // Get source and target adapters
        let source_adapter = self.adapters.get(&request.source_domain)
            .ok_or_else(|| Error::NotFoundError(format!(
                "Source adapter not found for domain: {}",
                request.source_domain.as_ref()
            )))?;
        
        let target_adapter = self.adapters.get(&request.target_domain)
            .ok_or_else(|| Error::NotFoundError(format!(
                "Target adapter not found for domain: {}",
                request.target_domain.as_ref()
            )))?;
        
        // Get VM types
        let source_vm = source_adapter.vm_type();
        let target_vm = target_adapter.vm_type();
        
        // Get handler
        let handler = self.handlers.get(&(source_vm.clone(), target_vm.clone()))
            .ok_or_else(|| Error::NotFoundError(format!(
                "No handler found for VM types: {:?} -> {:?}",
                source_vm, target_vm
            )))?;
        
        // Execute the operation
        handler.handle_request(&request, source_adapter.as_ref(), target_adapter.as_ref())
    }
    
    /// Get an adapter by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<&Box<dyn VmAdapter>> {
        self.adapters.get(domain_id)
    }
    
    /// Get a handler for a pair of VM types
    pub fn get_handler(&self, source_vm: &VmType, target_vm: &VmType) -> Option<&Box<dyn CrossVmHandler>> {
        self.handlers.get(&(source_vm.clone(), target_vm.clone()))
    }
}

/// Cross-VM operation handler
pub trait CrossVmHandler: Send + Sync {
    /// Handle a cross-VM operation request
    fn handle_request(
        &self,
        request: &CrossVmRequest,
        source_adapter: &dyn VmAdapter,
        target_adapter: &dyn VmAdapter,
    ) -> Result<CrossVmResponse>;
    
    /// Get the source VM type
    fn source_vm_type(&self) -> VmType;
    
    /// Get the target VM type
    fn target_vm_type(&self) -> VmType;
}

/// Helper for proof translation between VMs
pub fn translate_proof(
    proof: &Proof,
    source_vm: &VmType,
    target_vm: &VmType,
) -> Result<Proof> {
    // This is a placeholder implementation
    // In a real implementation, this would translate proof formats between VMs
    
    if source_vm == target_vm {
        // No translation needed for same VM types
        return Ok(proof.clone());
    }
    
    // For now, just return a clone of the proof
    // In a real implementation, this would perform actual translation
    Ok(proof.clone())
}

/// Helper for program translation between VMs
pub fn translate_program(
    program: &RiscVProgram,
    source_vm: &VmType,
    target_vm: &VmType,
) -> Result<RiscVProgram> {
    // This is a placeholder implementation
    // In a real implementation, this would translate program formats between VMs
    
    if source_vm == target_vm {
        // No translation needed for same VM types
        return Ok(program.clone());
    }
    
    // For now, just return a clone of the program
    // In a real implementation, this would perform actual translation
    Ok(program.clone())
}

/// Verify a proof across different VMs
pub fn cross_vm_verify(
    program: &RiscVProgram,
    proof: &Proof,
    public_inputs: &HashMap<String, Vec<u8>>,
    source_adapter: &dyn ZkProofAdapter,
    target_adapter: &dyn ZkProofAdapter,
) -> Result<bool> {
    // First verify with source adapter
    let source_result = source_adapter.verify_proof(program, proof, public_inputs)?;
    
    if !source_result {
        return Ok(false);
    }
    
    // Translate the proof for target VM
    let source_vm = source_adapter.vm_type();
    let target_vm = target_adapter.vm_type();
    
    let translated_proof = translate_proof(proof, &source_vm, &target_vm)?;
    let translated_program = translate_program(program, &source_vm, &target_vm)?;
    
    // Verify with target adapter
    target_adapter.verify_proof(&translated_program, &translated_proof, public_inputs)
}

/// Default handler for Succinct to EVM
#[derive(Debug)]
pub struct SuccinctToEvmHandler;

impl CrossVmHandler for SuccinctToEvmHandler {
    fn handle_request(
        &self,
        request: &CrossVmRequest,
        source_adapter: &dyn VmAdapter,
        target_adapter: &dyn VmAdapter,
    ) -> Result<CrossVmResponse> {
        // Check if adapters implement required traits
        let source_zk = source_adapter.as_any().downcast_ref::<dyn ZkProofAdapter>()
            .ok_or_else(|| Error::InvalidArgument(
                "Source adapter does not implement ZkProofAdapter".to_string()
            ))?;
        
        // Get program and inputs
        let program = match &request.program {
            Some(p) => p,
            None => return Err(Error::InvalidArgument("Program required".to_string())),
        };
        
        // Generate proof using source adapter
        let witness = match &request.witness {
            Some(w) => w.clone(),
            None => source_zk.generate_witness(program, &request.public_inputs, &request.private_inputs)?,
        };
        
        let proof = match &request.proof {
            Some(p) => p.clone(),
            None => source_zk.generate_proof(program, &witness, None)?,
        };
        
        // Translate for target VM
        let source_vm = source_adapter.vm_type();
        let target_vm = target_adapter.vm_type();
        
        // For now, just return success with the proof
        // In a real implementation, this would execute on the target VM
        let mut metadata = HashMap::new();
        metadata.insert("source_vm".to_string(), source_vm.name());
        metadata.insert("target_vm".to_string(), target_vm.name());
        
        Ok(CrossVmResponse {
            status: CrossVmStatus::Success,
            data: Vec::new(),
            proof: Some(proof),
            error: None,
            metadata,
        })
    }
    
    fn source_vm_type(&self) -> VmType {
        VmType::Succinct
    }
    
    fn target_vm_type(&self) -> VmType {
        VmType::Evm
    }
}

/// Create a configuration for a VM adapter
pub fn create_adapter_config(
    domain_id: impl Into<DomainId>,
    vm_type: VmType,
    endpoint: impl Into<String>,
) -> MultiVmAdapterConfig {
    let domain_id = domain_id.into();
    let endpoint = endpoint.into();
    
    MultiVmAdapterConfig {
        domain_id,
        vm_type,
        api_endpoints: vec![endpoint],
        auth: None,
        debug_mode: false,
        extra_config: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    
    #[derive(Debug)]
    struct MockAdapter {
        domain_id: DomainId,
        vm_type: VmType,
    }
    
    impl VmAdapter for MockAdapter {
        fn vm_type(&self) -> VmType {
            self.vm_type.clone()
        }
        
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        fn supports_feature(&self, _feature: &str) -> bool {
            true
        }
        
        fn config(&self) -> MultiVmAdapterConfig {
            MultiVmAdapterConfig {
                domain_id: self.domain_id.clone(),
                vm_type: self.vm_type.clone(),
                ..Default::default()
            }
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    
    #[test]
    fn test_broker_registration() {
        let mut broker = CrossVmBroker::new();
        
        let adapter1 = MockAdapter {
            domain_id: DomainId::new("domain1"),
            vm_type: VmType::Succinct,
        };
        
        let adapter2 = MockAdapter {
            domain_id: DomainId::new("domain2"),
            vm_type: VmType::Evm,
        };
        
        let result1 = broker.register_adapter(Box::new(adapter1));
        let result2 = broker.register_adapter(Box::new(adapter2));
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        assert!(broker.get_adapter(&DomainId::new("domain1")).is_some());
        assert!(broker.get_adapter(&DomainId::new("domain2")).is_some());
    }
} 