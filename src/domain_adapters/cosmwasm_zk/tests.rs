use std::collections::HashMap;

use crate::domain::domain_adapter::{Capabilities, DomainAdapter, ProofSupport};
use crate::effect::{Effect, EffectResult};
use crate::error::Result;
use crate::vm::zk_integration::Proof;

use super::adapter::CosmWasmZkAdapter;
use super::effects::{CompileEffect, ExecuteContractEffect, ProveEffect, VerifyEffect};
use super::types::{Coin, CosmWasmCallData, CosmWasmPublicInputs, VerificationResult};

// Helper function to create a test adapter
fn create_test_adapter() -> CosmWasmZkAdapter {
    CosmWasmZkAdapter::new()
}

#[test]
fn test_adapter_name_and_domain_type() {
    let adapter = create_test_adapter();
    
    assert_eq!(adapter.name(), "cosmwasm_zk");
    assert_eq!(adapter.domain_type(), "cosmwasm_zk");
}

#[test]
fn test_capabilities() {
    let adapter = create_test_adapter();
    let capabilities = adapter.capabilities();
    
    assert!(capabilities.contains(Capabilities::PROOFS));
    assert!(capabilities.contains(Capabilities::VERIFICATION));
    assert!(capabilities.contains(Capabilities::STATE_TRANSITIONS));
    assert!(capabilities.contains(Capabilities::CODE_GENERATION));
    assert!(capabilities.contains(Capabilities::PRIVACY));
}

#[test]
fn test_proof_support() {
    let adapter = create_test_adapter();
    let proof_support = adapter.proof_support();
    
    assert_eq!(proof_support, ProofSupport::FULL);
}

#[test]
fn test_schema() {
    let adapter = create_test_adapter();
    let schema = adapter.schema();
    
    // Check that the schema contains necessary mappings
    assert!(schema.type_mappings.contains_key("uint256"));
    assert!(schema.type_mappings.contains_key("address"));
    assert!(schema.function_mappings.contains_key("constructor"));
    assert!(schema.function_mappings.contains_key("execute"));
    assert!(schema.function_mappings.contains_key("query"));
    assert!(schema.effect_mappings.contains_key("compile"));
    assert!(schema.effect_mappings.contains_key("execute_contract"));
    assert!(schema.effect_mappings.contains_key("prove"));
    assert!(schema.effect_mappings.contains_key("verify"));
}

#[test]
fn test_compile_effect_conversion() {
    // Create a compile effect
    let source = r#"
        #[cosmwasm_entry_points]
        pub mod contract {
            // Contract code here
        }
    "#.to_string();
    let program_id = "test_program".to_string();
    
    let compile_effect = CompileEffect::new(source.clone(), program_id.clone());
    
    // Convert to generic effect
    let effect = compile_effect.to_effect();
    
    // Check effect name and parameters
    assert_eq!(effect.name, "compile");
    assert_eq!(effect.get_param_as_string("source").unwrap(), source);
    assert_eq!(effect.get_param_as_string("program_id").unwrap(), program_id);
    
    // Convert back to compile effect
    let roundtrip_effect = CompileEffect::from_effect(&effect).unwrap();
    
    // Check roundtrip conversion
    assert_eq!(roundtrip_effect.source, source);
    assert_eq!(roundtrip_effect.program_id, program_id);
}

#[test]
fn test_execute_contract_effect_conversion() {
    // Create an execute contract effect
    let contract_address = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string();
    let method = "increment".to_string();
    let inputs = r#"{"value": 5}"#.to_string();
    let funds = Some(vec![
        Coin {
            denom: "uatom".to_string(),
            amount: "100".to_string(),
        }
    ]);
    
    let execute_effect = ExecuteContractEffect::new(
        contract_address.clone(),
        method.clone(),
        inputs.clone(),
        funds.clone(),
    );
    
    // Convert to generic effect
    let effect = execute_effect.to_effect();
    
    // Check effect name and parameters
    assert_eq!(effect.name, "execute_contract");
    assert_eq!(effect.get_param_as_string("contract_address").unwrap(), contract_address);
    assert_eq!(effect.get_param_as_string("method").unwrap(), method);
    assert_eq!(effect.get_param_as_string("inputs").unwrap(), inputs);
    
    // Convert back to execute contract effect
    let roundtrip_effect = ExecuteContractEffect::from_effect(&effect).unwrap();
    
    // Check roundtrip conversion
    assert_eq!(roundtrip_effect.call_data.contract_address, contract_address);
    assert_eq!(roundtrip_effect.call_data.method, method);
    assert_eq!(roundtrip_effect.call_data.inputs, inputs);
}

#[test]
fn test_prove_effect_conversion() {
    // Create a prove effect
    let contract_address = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string();
    let method = "increment".to_string();
    let inputs = r#"{"value": 5}"#.to_string();
    let expected_output = Some(r#"{"new_value": 10}"#.to_string());
    let funds = None;
    
    let prove_effect = ProveEffect::new(
        contract_address.clone(),
        method.clone(),
        inputs.clone(),
        expected_output.clone(),
        funds.clone(),
    );
    
    // Convert to generic effect
    let effect = prove_effect.to_effect();
    
    // Check effect name and parameters
    assert_eq!(effect.name, "prove");
    assert_eq!(effect.get_param_as_string("contract_address").unwrap(), contract_address);
    assert_eq!(effect.get_param_as_string("method").unwrap(), method);
    assert_eq!(effect.get_param_as_string("inputs").unwrap(), inputs);
    assert_eq!(effect.get_param_as_string_option("expected_output"), expected_output);
    
    // Convert back to prove effect
    let roundtrip_effect = ProveEffect::from_effect(&effect).unwrap();
    
    // Check roundtrip conversion
    assert_eq!(roundtrip_effect.call_data.contract_address, contract_address);
    assert_eq!(roundtrip_effect.call_data.method, method);
    assert_eq!(roundtrip_effect.call_data.inputs, inputs);
    assert_eq!(roundtrip_effect.expected_output, expected_output);
}

#[test]
fn test_verify_effect_conversion() {
    // Create a verify effect
    let proof = Proof {
        data: vec![1, 2, 3, 4, 5],
        verification_key: vec![0, 1, 2, 3, 4, 5],
    };
    
    let contract_address = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string();
    let method = "increment".to_string();
    let chain_id = "cosmoshub-4".to_string();
    let inputs = r#"{"value": 5}"#.to_string();
    
    let mut public_inputs = CosmWasmPublicInputs::new(
        contract_address.clone(),
        method.clone(),
        chain_id.clone(),
        inputs.clone(),
    );
    
    let expected_output = r#"{"new_value": 10}"#.to_string();
    public_inputs = public_inputs.with_expected_output(expected_output.clone());
    
    // Add some additional data
    public_inputs.additional_data.insert("nonce".to_string(), "12345".to_string());
    
    let verify_effect = VerifyEffect::new(
        proof.clone(),
        public_inputs.clone(),
    );
    
    // Convert to generic effect
    let effect = verify_effect.to_effect();
    
    // Check effect name and parameters
    assert_eq!(effect.name, "verify");
    assert_eq!(effect.get_param_as_string("contract_address").unwrap(), contract_address);
    assert_eq!(effect.get_param_as_string("method").unwrap(), method);
    assert_eq!(effect.get_param_as_string("chain_id").unwrap(), chain_id);
    assert_eq!(effect.get_param_as_string("inputs").unwrap(), inputs);
    assert_eq!(effect.get_param_as_string("expected_output").unwrap(), expected_output);
    assert_eq!(effect.get_param_as_string("additional_nonce").unwrap(), "12345");
}

#[test]
fn test_execute_effect() {
    let adapter = create_test_adapter();
    
    // Create a compile effect
    let source = r#"
        #[cosmwasm_entry_points]
        pub mod contract {
            // Contract code here
        }
    "#.to_string();
    let program_id = "test_program".to_string();
    
    let mut effect = Effect::new("compile");
    effect.add_param("source", source);
    effect.add_param("program_id", program_id);
    
    // Execute the effect
    let result = adapter.execute_effect(&effect);
    
    // Since this is a mock implementation, we expect a successful result
    assert!(result.is_ok());
    
    // Check that the result contains the program_id
    if let Ok(EffectResult::Data(data)) = result {
        assert!(data.contains("program_id"));
    } else {
        panic!("Expected EffectResult::Data");
    }
} 