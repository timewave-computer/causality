//! Succinct ZK-VM integration module
//! 
//! This module provides integration with the Succinct ZK-VM, allowing for
//! provable computation with zero-knowledge proofs.

pub mod types;
pub mod adapter;
pub mod bridge;

pub use types::{
    ProgramId,
    PublicInputs,
    VerificationKey,
    ProofData,
    ProofOptions,
    ExecutionStats,
};

pub use bridge::{
    SuccinctVmBridge,
    create_succinct_vm_bridge,
};

pub use adapter::SuccinctAdapter;

/// Re-export the primary ZK adapter interface
pub use crate::zk::ZkVirtualMachine;

/// Default provider function for getting a succinct adapter
pub fn default_adapter() -> crate::error::Result<SuccinctAdapter> {
    SuccinctAdapter::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_module_exports() {
        // This test ensures that all public exports are available
        // It doesn't test functionality, just validates compilation
        
        let _program_id = ProgramId::new("test_program");
        let _inputs = PublicInputs::new();
        let _key = SuccinctVm::generate_verification_key("test_program").unwrap();
        
        // Test that adapter implements ZkVirtualMachine
        fn assert_impl<T: ZkVirtualMachine>() {}
        assert_impl::<SuccinctAdapter>();
    }
    
    #[test]
    fn test_integrated_workflow() {
        // This test is commented out as it requires a real Succinct API key
        // Uncomment and add a valid API key to run this test
        
        /*
        // Create a Succinct adapter
        let adapter = default_adapter().unwrap()
            .with_api_key("test-api-key");
        
        // Simple program that just returns some data
        let source_code = r#"
fn main() {
    let input = env::get_input("value").unwrap();
    let value: u32 = serde_json::from_str(&input).unwrap();
    
    // Double the value and return it
    let result = value * 2;
    env::set_output("result", &result);
}
        "#;
        
        // Compile the program
        let program_id = adapter.compile_program(source_code, Some("double-value")).unwrap();
        
        // Prepare inputs
        let mut public_inputs = PublicInputs::new();
        public_inputs.add("value", &42u32).unwrap();
        
        let private_inputs = HashMap::new();
        
        // Generate a proof
        let proof = adapter.prove(
            &program_id,
            &public_inputs,
            &private_inputs,
            None,
        ).unwrap();
        
        // Verify the proof
        let is_valid = adapter.verify(&program_id, &proof, &public_inputs).unwrap();
        assert!(is_valid);
        
        // In a real implementation, we would check the journal for the output result
        assert!(proof.journal.is_some());
        */
    }
} 