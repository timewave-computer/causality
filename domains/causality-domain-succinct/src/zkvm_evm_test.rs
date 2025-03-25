// zkVM-EVM integration tests
// Original file: src/domain_adapters/zkvm_evm_test.rs

//! Tests for ZK-VM powered EVM adapter
//!
//! This module contains tests for the ZK-VM powered EVM adapter.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use causality_types::Result;
    use causality_types::DomainId;
    use crate::domain_adapters::{
        interfaces::{VmType, VmAdapter, CompilationAdapter, ZkProofAdapter},
        zkvm::{ZkVmBackend, ZkVmAdapterConfig, ZkVmDomainAdapter, ZkProof},
        zkvm_evm::{ZkEvmAdapter, ZkEvmAdapterConfig, ZkEvmEffectType, ZkEvmEffectValidator, ZkEvmAdapterFactory},
    };
    
    #[test]
    fn test_zkvm_evm_adapter_creation() -> Result<()> {
        // Create ZK-VM EVM adapter configuration
        let zkvm_config = ZkVmAdapterConfig {
            domain_id: DomainId::new("ethereum-test"),
            target_vm_type: VmType::Evm,
            zkvm_backend: ZkVmBackend::RiscZero,
            guest_program_path: Some("examples/zkvm_guest_program.rs".to_string()),
            guest_program_id: None,
            proving_api_endpoint: None,
            auth_token: None,
            debug_mode: true,
            extra_config: HashMap::new(),
        };
        
        let evm_config = ZkEvmAdapterConfig {
            base_config: zkvm_config,
            chain_id: 1337, // Local test chain
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            gas_price: Some("1".to_string()),
            verifier_contract: None,
            private_key: None,
        };
        
        // Create adapter
        let adapter = ZkEvmAdapter::new(evm_config);
        
        // Verify adapter properties
        assert_eq!(adapter.domain_id().as_ref(), "ethereum-test");
        assert_eq!(adapter.vm_type(), VmType::ZkVm);
        assert_eq!(adapter.target_vm_type(), VmType::Evm);
        assert_eq!(adapter.zkvm_backend(), &ZkVmBackend::RiscZero);
        
        Ok(())
    }
    
    #[test]
    fn test_zkvm_evm_adapter_factory() -> Result<()> {
        // Create factory
        let factory = ZkEvmAdapterFactory::new();
        
        // Verify factory properties
        assert_eq!(factory.name(), "zk_evm");
        assert_eq!(factory.supported_vm_types(), vec![VmType::ZkVm]);
        assert_eq!(factory.supported_zkvm_backends(), vec![ZkVmBackend::RiscZero, ZkVmBackend::Succinct]);
        assert_eq!(factory.supported_target_vms(), vec![VmType::Evm]);
        
        // Verify factory can create adapter
        let zkvm_config = ZkVmAdapterConfig {
            domain_id: DomainId::new("ethereum-test"),
            target_vm_type: VmType::Evm,
            zkvm_backend: ZkVmBackend::RiscZero,
            guest_program_path: Some("examples/zkvm_guest_program.rs".to_string()),
            guest_program_id: None,
            proving_api_endpoint: None,
            auth_token: None,
            debug_mode: true,
            extra_config: {
                let mut map = HashMap::new();
                map.insert("chain_id".to_string(), "1337".to_string());
                map.insert("rpc_endpoints".to_string(), "http://localhost:8545".to_string());
                map
            },
        };
        
        let adapter = factory.create_zkvm_adapter(zkvm_config)?;
        
        // Verify adapter properties
        assert_eq!(adapter.domain_id().as_ref(), "ethereum-test");
        assert_eq!(adapter.vm_type(), VmType::ZkVm);
        assert_eq!(adapter.target_vm_type(), VmType::Evm);
        assert_eq!(adapter.zkvm_backend(), &ZkVmBackend::RiscZero);
        
        Ok(())
    }
    
    #[test]
    fn test_zkvm_evm_adapter_proof_generation() -> Result<()> {
        // Create adapter
        let zkvm_config = ZkVmAdapterConfig {
            domain_id: DomainId::new("ethereum-test"),
            target_vm_type: VmType::Evm,
            zkvm_backend: ZkVmBackend::RiscZero,
            guest_program_path: Some("examples/zkvm_guest_program.rs".to_string()),
            guest_program_id: None,
            proving_api_endpoint: None,
            auth_token: None,
            debug_mode: true,
            extra_config: HashMap::new(),
        };
        
        let evm_config = ZkEvmAdapterConfig {
            base_config: zkvm_config,
            chain_id: 1337, // Local test chain
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            gas_price: Some("1".to_string()),
            verifier_contract: None,
            private_key: None,
        };
        
        let adapter = ZkEvmAdapter::new(evm_config);
        
        // Deploy contract parameters
        let deploy_params = serde_json::json!({
            "bytecode": "0x608060405234801561001057600080fd5b5061017f806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063a6f9dae11461003b578063e79a198f14610057575b600080fd5b610055600480360381019061005091906100f9565b610073565b005b61005f6100b3565b60405161006e9190610135565b60405180910390f35b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff161461010b576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161010290610393565b60405180910390fd5b8060008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690555050565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b6000813590506100f381610525565b92915050565b60006020828403121561010f5761010e610520565b5b600061011d848285016100e4565b91505092915050565b61012f8161048f565b82525050565b600060208201905061014a6000830184610126565b92915050565b600061015b6000836104e4565b9150610166826104f4565b600082019050919050565b6000610178610183565b905061018482826104c1565b919050565b6000604051905090565b600067ffffffffffffffff8211156101a8576101a76104f9565b5b6101b1826104f4565b9050602081019050919050565b60006101d56101d083610189565b9150610530565b9050919050565b6101e58161048f565b81146101f057600080fd5b50565b60006102018235836101dc565b61020a826104f4565b90508260208301111561021f5761021e61051b565b5b9250929050565b60005b8381101561023f578082015181840152602081019050610224565b8381111561024e576000848401525b50505050565b600061025f82610488565b61026981856104e4565b935061027981856020860161022e565b80840191505092915050565b60006102926002836104e4565b915061029d826104f4565b602082019050919050565b600061012c604051905081810181811067ffffffffffffffff821117156102d2576102d16104f956b5b6102db826104f456b91506102e78261028556b102f281516104c156b82525060006020606060005b848110156103375761032786820151680100000000000000008361020156b915060046020860101955080600101915061030a56b508082602001818101526000602080840181015b818110156103785760208787030151915061035a8261025456b81526020019060010161034e56b5083855260008091602084010191505082830360200191505050505056b600061039e60148361050a56b91506103a98261051e56b0208201905091905056b600060208201608084016000868152602001898152602003600082526103d98261039156b915081602083015282604083015281606083015250505050505056b60008183019050828152602080820191909152019056b600060208201905091905056b61041d8161045856b8252505056b61042c8161046d56b8252505056b600061043d8261046656b610447818561048856b935061045781856020860161047756b8101905091905056b60006190448110610466576000905061046356b9056b60008151905091905056b6000811515905091905056b600073ffffffffffffffffffffffffffffffffffffffff8216905091905056b60005b838110156104955780820151818401526020810190506104505b50505056b60006104b68235836103f956b6104bf8261042856b905091905056b6104ca826104f456b810181811067ffffffffffffffff821117156104e9576104e86104f956b5b8060405250505056b60008190509291505056b6000601f19601f830116905091905056b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b600080fd5b600080fd5b600080fd5b680200000000000000009056b61052e816101dd56b811461053957600080fd5b5056fea26469706673582212204e37624c1c79d8e48c97c18fb61bfeb35db3c7d57afacbef6b0f855e13a35e1464736f6c63430008070033",
            "constructor_args": "0x",
        });
        
        // Private inputs
        let private_inputs = serde_json::json!({
            "wallet_address": "0x1234567890123456789012345678901234567890",
            "nonce": 42,
            "gas_price": "5000000000",
            "gas_limit": 3000000,
            "chain_id": 1,
        });
        
        // Generate proof
        let proof = adapter.generate_proof(
            ZkEvmEffectType::DeployContract.as_str(),
            &deploy_params,
            &private_inputs,
        )?;
        
        // Verify proof properties
        assert_eq!(proof.backend, ZkVmBackend::RiscZero);
        assert_eq!(proof.target_vm, VmType::Evm);
        assert_eq!(proof.metadata.get("effect_type"), Some(&"deploy_contract".to_string()));
        
        // Execute function parameters
        let execute_params = serde_json::json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "function": "transfer(address,uint256)",
            "args": [
                "0x2222222222222222222222222222222222222222",
                "1000000000000000000"
            ],
        });
        
        // Generate proof
        let proof = adapter.generate_proof(
            ZkEvmEffectType::ExecuteFunction.as_str(),
            &execute_params,
            &private_inputs,
        )?;
        
        // Verify proof properties
        assert_eq!(proof.backend, ZkVmBackend::RiscZero);
        assert_eq!(proof.target_vm, VmType::Evm);
        assert_eq!(proof.metadata.get("effect_type"), Some(&"execute_function".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_zkvm_evm_effect_validator() -> Result<()> {
        // Create validator
        let validator = ZkEvmEffectValidator::new();
        
        // Verify validator supports effect types
        assert!(validator.supports_effect_type(ZkEvmEffectType::DeployContract.as_str()));
        assert!(validator.supports_effect_type(ZkEvmEffectType::ExecuteFunction.as_str()));
        assert!(validator.supports_effect_type(ZkEvmEffectType::TransferEth.as_str()));
        assert!(validator.supports_effect_type(ZkEvmEffectType::UpdateState.as_str()));
        assert!(!validator.supports_effect_type("unknown_effect"));
        
        Ok(())
    }
} 