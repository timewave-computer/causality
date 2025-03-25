// Domain adapter testing utilities
// Original file: src/domain_adapters/testing.rs

// Standardized Testing Framework for Domain Adapters
//
// This module provides utilities for testing domain adapters across different VMs.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use causality_types::Result;
use causality_domain_sdk::{
    VmAdapter, 
    CompilationAdapter, 
    ZkProofAdapter, 
    CrossVmAdapter,
    VmType,
    MultiVmAdapterConfig
};
use causality_core::{RiscVProgram, Witness, Proof};
use causality_types::DomainId;

/// Test configuration for adapter tests
#[derive(Debug, Clone)]
pub struct AdapterTestConfig {
    /// Test name
    pub name: String,
    /// VM type to test
    pub vm_type: VmType,
    /// Domain ID for the test
    pub domain_id: DomainId,
    /// Test timeout
    pub timeout: Duration,
    /// Debug mode
    pub debug: bool,
    /// Test data directory
    pub test_data_dir: Option<String>,
    /// Additional configuration
    pub config: HashMap<String, String>,
}

impl Default for AdapterTestConfig {
    fn default() -> Self {
        Self {
            name: "adapter_test".to_string(),
            vm_type: VmType::ZkVm,
            domain_id: DomainId::new("test-domain"),
            timeout: Duration::from_secs(30),
            debug: false,
            test_data_dir: None,
            config: HashMap::new(),
        }
    }
}

/// Test case for adapter tests
pub struct AdapterTestCase<T: VmAdapter> {
    /// Adapter under test
    pub adapter: T,
    /// Test configuration
    pub config: AdapterTestConfig,
    /// Test source code
    pub source_code: Option<String>,
    /// Compiled program
    pub program: Option<RiscVProgram>,
    /// Public inputs
    pub public_inputs: HashMap<String, Vec<u8>>,
    /// Private inputs
    pub private_inputs: HashMap<String, Vec<u8>>,
    /// Generated witness
    pub witness: Option<Witness>,
    /// Generated proof
    pub proof: Option<Proof>,
    /// Verification result
    pub verification_result: Option<bool>,
}

impl<T: VmAdapter> AdapterTestCase<T> {
    /// Create a new test case
    pub fn new(adapter: T, config: AdapterTestConfig) -> Self {
        Self {
            adapter,
            config,
            source_code: None,
            program: None,
            public_inputs: HashMap::new(),
            private_inputs: HashMap::new(),
            witness: None,
            proof: None,
            verification_result: None,
        }
    }
    
    /// Set the source code
    pub fn with_source_code(mut self, source: impl Into<String>) -> Self {
        self.source_code = Some(source.into());
        self
    }
    
    /// Add a public input
    pub fn with_public_input(mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.public_inputs.insert(key.into(), value.into());
        self
    }
    
    /// Add a private input
    pub fn with_private_input(mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.private_inputs.insert(key.into(), value.into());
        self
    }
}

/// Trait for testing compilation adapters
pub trait CompilationTest {
    /// Test compilation functionality
    fn test_compilation(&mut self) -> Result<()>;
    
    /// Get the compiled program
    fn get_program(&self) -> Option<&RiscVProgram>;
}

/// Trait for testing ZK proof adapters
pub trait ZkProofTest {
    /// Test witness generation
    fn test_witness_generation(&mut self) -> Result<()>;
    
    /// Test proof generation
    fn test_proof_generation(&mut self) -> Result<()>;
    
    /// Test proof verification
    fn test_proof_verification(&mut self) -> Result<()>;
    
    /// Test the full ZK workflow
    fn test_full_zk_workflow(&mut self) -> Result<()>;
    
    /// Get the generated witness
    fn get_witness(&self) -> Option<&Witness>;
    
    /// Get the generated proof
    fn get_proof(&self) -> Option<&Proof>;
    
    /// Get the verification result
    fn get_verification_result(&self) -> Option<bool>;
}

/// Trait for testing cross-VM adapters
pub trait CrossVmTest {
    /// Test program translation
    fn test_translation(&mut self, target_vm: VmType) -> Result<()>;
    
    /// Test remote execution
    fn test_remote_execution(&mut self, target_vm: VmType) -> Result<()>;
}

impl<T> CompilationTest for AdapterTestCase<T>
where
    T: CompilationAdapter,
{
    fn test_compilation(&mut self) -> Result<()> {
        // Skip if no source code
        let source = match &self.source_code {
            Some(s) => s,
            None => return Ok(()),
        };
        
        // Compile the program
        let program = self.adapter.compile_program(source, Some(&self.config.name))?;
        
        // Store the program
        self.program = Some(program);
        
        Ok(())
    }
    
    fn get_program(&self) -> Option<&RiscVProgram> {
        self.program.as_ref()
    }
}

impl<T> ZkProofTest for AdapterTestCase<T>
where
    T: ZkProofAdapter,
{
    fn test_witness_generation(&mut self) -> Result<()> {
        // Skip if no program
        let program = match &self.program {
            Some(p) => p,
            None => return Ok(()),
        };
        
        // Generate witness
        let witness = self.adapter.generate_witness(
            program,
            &self.public_inputs,
            &self.private_inputs,
        )?;
        
        // Store the witness
        self.witness = Some(witness);
        
        Ok(())
    }
    
    fn test_proof_generation(&mut self) -> Result<()> {
        // Skip if no program or witness
        let (program, witness) = match (&self.program, &self.witness) {
            (Some(p), Some(w)) => (p, w),
            _ => return Ok(()),
        };
        
        // Generate proof
        let proof = self.adapter.generate_proof(program, witness, None)?;
        
        // Store the proof
        self.proof = Some(proof);
        
        Ok(())
    }
    
    fn test_proof_verification(&mut self) -> Result<()> {
        // Skip if no program or proof
        let (program, proof) = match (&self.program, &self.proof) {
            (Some(p), Some(pr)) => (p, pr),
            _ => return Ok(()),
        };
        
        // Verify proof
        let result = self.adapter.verify_proof(program, proof, &self.public_inputs)?;
        
        // Store the result
        self.verification_result = Some(result);
        
        Ok(())
    }
    
    fn test_full_zk_workflow(&mut self) -> Result<()> {
        // Run the full workflow
        self.test_witness_generation()?;
        self.test_proof_generation()?;
        self.test_proof_verification()?;
        
        // Check the result
        if let Some(result) = self.verification_result {
            if !result {
                return Err(causality_types::Error::ValidationError(
                    "Proof verification failed".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn get_witness(&self) -> Option<&Witness> {
        self.witness.as_ref()
    }
    
    fn get_proof(&self) -> Option<&Proof> {
        self.proof.as_ref()
    }
    
    fn get_verification_result(&self) -> Option<bool> {
        self.verification_result
    }
}

impl<T> CrossVmTest for AdapterTestCase<T>
where
    T: CrossVmAdapter,
{
    fn test_translation(&mut self, target_vm: VmType) -> Result<()> {
        // Skip if no program
        let program = match &self.program {
            Some(p) => p,
            None => return Ok(()),
        };
        
        // Translate the program
        let _translated = self.adapter.translate_program(program, &target_vm)?;
        
        Ok(())
    }
    
    fn test_remote_execution(&mut self, target_vm: VmType) -> Result<()> {
        // Skip if no program
        let program = match &self.program {
            Some(p) => p,
            None => return Ok(()),
        };
        
        // Execute remotely
        let _result = self.adapter.execute_remote(program, &target_vm, &self.public_inputs)?;
        
        Ok(())
    }
}

/// Utility for creating test inputs
pub fn create_test_inputs() -> (HashMap<String, Vec<u8>>, HashMap<String, Vec<u8>>) {
    let mut public_inputs = HashMap::new();
    let mut private_inputs = HashMap::new();
    
    // Add some test inputs
    public_inputs.insert("value".to_string(), vec![42]);
    private_inputs.insert("secret".to_string(), vec![12, 34, 56]);
    
    (public_inputs, private_inputs)
}

/// Sample RISC-V program for testing
pub fn create_test_program() -> RiscVProgram {
    // This is a placeholder implementation
    RiscVProgram {
        name: Some("test_program".to_string()),
        entry_point: "main".to_string(),
        sections: Vec::new(),
        symbols: HashMap::new(),
        memory_size: 8192,
    }
}

/// Sample source code for testing
pub fn create_test_source_code() -> String {
    r#"
    .section .text
    .globl main
    
    main:
        // Load value from input
        la a0, input
        lw a1, 0(a0)
        
        // Double the value
        add a1, a1, a1
        
        // Store result
        la a0, output
        sw a1, 0(a0)
        
        // Return
        ret
        
    .section .data
    input:
        .word 0
    output:
        .word 0
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    
    // Mock adapter for testing
    #[derive(Debug)]
    struct MockAdapter {
        vm_type: VmType,
        domain_id: DomainId,
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
    
    impl CompilationAdapter for MockAdapter {
        fn compile_program(&mut self, _source: &str, _name: Option<&str>) -> Result<RiscVProgram> {
            Ok(create_test_program())
        }
        
        fn supported_languages(&self) -> Vec<String> {
            vec!["riscv".to_string()]
        }
    }
    
    #[test]
    fn test_compilation_test() {
        let adapter = MockAdapter {
            vm_type: VmType::ZkVm,
            domain_id: DomainId::new("test"),
        };
        
        let config = AdapterTestConfig::default();
        
        let mut test_case = AdapterTestCase::new(adapter, config)
            .with_source_code(create_test_source_code());
        
        let result = test_case.test_compilation();
        assert!(result.is_ok());
        assert!(test_case.get_program().is_some());
    }
} 