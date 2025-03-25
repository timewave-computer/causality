// Succinct bridge functionality
// Original file: src/domain_adapters/succinct/bridge.rs

// Bridge module to adapt the SuccinctAdapter to implement ZkVirtualMachine
//
// This module provides the bridge between our Succinct adapter and the
// ZkVirtualMachine trait used by the core ZK system.

use std::collections::HashMap;

use causality_types::{Error, Result};
use causality_core::{ZkVirtualMachine, Witness, Proof, StateTransition, VmState, MemoryAccess, RiscVProgram};
use causality_domain_succinct::{
    SuccinctAdapter, PublicInputs, ProofData, ProgramId, ProofOptions
};

/// Wrapper to adapt the SuccinctAdapter to the ZkVirtualMachine trait
#[derive(Debug)]
pub struct SuccinctVmBridge {
    /// The underlying Succinct adapter
    adapter: SuccinctAdapter,
    /// The compiled program ID
    program_id: Option<ProgramId>,
    /// Public inputs for the program
    public_inputs: PublicInputs,
    /// Private inputs for the program
    private_inputs: HashMap<String, Vec<u8>>,
    /// Proof options
    options: ProofOptions,
    /// The guest program source code
    source_code: Option<String>,
    /// Generated proof data
    proof_data: Option<ProofData>,
}

impl SuccinctVmBridge {
    /// Create a new Succinct VM bridge
    pub fn new(adapter: SuccinctAdapter) -> Self {
        Self {
            adapter,
            program_id: None,
            public_inputs: PublicInputs::new(),
            private_inputs: HashMap::new(),
            options: ProofOptions::default(),
            source_code: None,
            proof_data: None,
        }
    }
    
    /// Set the program ID
    pub fn with_program_id(mut self, program_id: ProgramId) -> Self {
        self.program_id = Some(program_id);
        self
    }
    
    /// Set the public inputs
    pub fn with_public_inputs(mut self, public_inputs: PublicInputs) -> Self {
        self.public_inputs = public_inputs;
        self
    }
    
    /// Set the private inputs
    pub fn with_private_inputs(mut self, private_inputs: HashMap<String, Vec<u8>>) -> Self {
        self.private_inputs = private_inputs;
        self
    }
    
    /// Set the proof options
    pub fn with_options(mut self, options: ProofOptions) -> Self {
        self.options = options;
        self
    }
    
    /// Set the source code
    pub fn with_source_code(mut self, source_code: String) -> Self {
        self.source_code = Some(source_code);
        self
    }
    
    /// Get the underlying adapter
    pub fn adapter(&self) -> &SuccinctAdapter {
        &self.adapter
    }
    
    /// Get the program ID
    pub fn program_id(&self) -> Option<&ProgramId> {
        self.program_id.as_ref()
    }
    
    /// Get the proof data
    pub fn proof_data(&self) -> Option<&ProofData> {
        self.proof_data.as_ref()
    }
    
    /// Convert a RISC-V program to Succinct source code
    fn convert_program_to_source(program: &RiscVProgram) -> String {
        // This is a simplified implementation that would be replaced
        // with actual conversion logic in a real implementation
        
        let mut source = String::new();
        
        source.push_str(&format!("// Program: {}\n", program.name.as_deref().unwrap_or("unnamed")));
        source.push_str(&format!("// Entry point: {}\n\n", program.entry_point));
        
        for section in &program.sections {
            source.push_str(&format!("// Section: {}\n", section.name));
            source.push_str("// Content: [binary data]\n\n");
        }
        
        source
    }
    
    /// Convert Succinct proof data to a Proof
    fn convert_proof_data_to_proof(proof_data: &ProofData) -> Proof {
        Proof {
            data: proof_data.data.clone(),
            proof_type: proof_data.proof_type().to_string(),
            public_inputs: proof_data.public_inputs().values.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
    
    /// Convert a Proof to Succinct proof data
    fn convert_proof_to_proof_data(&self, proof: &Proof) -> Result<ProofData> {
        if let Some(program_id) = &self.program_id {
            let public_inputs = PublicInputs {
                values: proof.public_inputs.clone(),
            };
            
            Ok(ProofData::new(
                proof.data.clone(),
                proof.proof_type.clone(),
                program_id.clone(),
                public_inputs,
            ))
        } else {
            Err(Error::ZkError("No program ID set".to_string()))
        }
    }
}

impl ZkVirtualMachine for SuccinctVmBridge {
    fn load_program(&mut self, program: RiscVProgram) -> Result<()> {
        let source = Self::convert_program_to_source(&program);
        self.source_code = Some(source.clone());
        
        let program_id = self.adapter.compile_program(&source, None)?;
        self.program_id = Some(program_id);
        
        Ok(())
    }
    
    fn generate_witness(&mut self) -> Result<Witness> {
        // Succinct doesn't have a separate witness generation step,
        // so we'll create a dummy witness that can be used later
        
        // In a real implementation, we might execute the program locally
        // to generate a trace, or use Succinct's API to get execution details
        
        let witness = Witness {
            transitions: vec![
                StateTransition {
                    before: VmState {
                        registers: vec![0; 32],
                        pc: 0,
                        cycle: 0,
                    },
                    after: VmState {
                        registers: vec![0; 32],
                        pc: 4,
                        cycle: 1,
                    },
                    instruction: 0x13, // NOP (addi x0, x0, 0)
                    pc: 0,
                    next_pc: 4,
                },
            ],
            memory_accesses: vec![],
            additional_data: HashMap::new(),
        };
        
        Ok(witness)
    }
    
    fn generate_proof(&self, _witness: &Witness) -> Result<Proof> {
        let program_id = self.program_id.as_ref()
            .ok_or_else(|| Error::ZkError("No program ID set".to_string()))?;
        
        // Convert public and private inputs to Succinct format
        let public_inputs = self.public_inputs.clone();
        let private_inputs = self.private_inputs.clone();
        let options = Some(self.options.clone());
        
        // Generate proof using the Succinct adapter
        let proof_data = self.adapter.prove(
            program_id,
            &public_inputs,
            &private_inputs,
            options,
        )?;
        
        // Convert to our Proof format
        let proof = Self::convert_proof_data_to_proof(&proof_data);
        
        Ok(proof)
    }
    
    fn verify_proof(&self, proof: &Proof) -> Result<bool> {
        let program_id = self.program_id.as_ref()
            .ok_or_else(|| Error::ZkError("No program ID set".to_string()))?;
        
        // Convert to Succinct format
        let proof_data = self.convert_proof_to_proof_data(proof)?;
        
        // Verify using the Succinct adapter
        let result = self.adapter.verify(
            program_id,
            &proof_data,
            &PublicInputs {
                values: proof.public_inputs.clone(),
            },
        )?;
        
        Ok(result)
    }
}

/// Create a new Succinct VM bridge
pub fn create_succinct_vm_bridge() -> Result<Box<dyn ZkVirtualMachine>> {
    let adapter = SuccinctAdapter::new()?;
    Ok(Box::new(SuccinctVmBridge::new(adapter)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bridge_creation() {
        let adapter = SuccinctAdapter::new().unwrap();
        let bridge = SuccinctVmBridge::new(adapter);
        
        assert!(bridge.program_id().is_none());
        assert!(bridge.proof_data().is_none());
    }
    
    #[test]
    fn test_convert_program_to_source() {
        let program = RiscVProgram::new(Vec::new(), "test".to_string());
        let source = SuccinctVmBridge::convert_program_to_source(&program);
        
        assert!(source.contains("// Program: test"));
        assert!(source.contains("// Entry point: 0"));
    }
} 