//! Integration tests for domain adapters
//!
//! This module contains end-to-end tests for domain adapters,
//! coordination systems, and cross-VM operations.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use crate::error::{Error, Result};
    use crate::types::DomainId;
    use crate::domain_adapters::{
        interfaces::{
            VmType,
            VmAdapter,
            VmAdapterRegistry,
            MultiVmAdapterConfig,
        },
        utils::{
            CrossVmBroker,
            CrossVmHandler,
            CrossVmRequest,
            CrossVmResponse,
            CrossVmStatus,
        },
        evm::{
            EvmAdapter,
            EvmAdapterConfig,
        },
        cosmwasm::{
            CosmWasmAdapter,
            CosmWasmAdapterConfig,
            CosmWasmAddress,
        },
        validation::{
            ValidationContext,
            ValidationResult,
            EffectValidator,
            EffectValidatorRegistry,
        },
        coordination::{
            CoordinationContext,
            CoordinationStep,
            CoordinationPlan,
            CoordinationExecutor,
            CoordinationStatus,
            CoordinationHandler,
        },
    };
    
    // Setup function to create test adapters and broker
    fn setup_test_environment() -> (
        Box<dyn VmAdapter>, // EVM Adapter
        Box<dyn VmAdapter>, // CosmWasm Adapter
        CrossVmBroker,
        EffectValidatorRegistry,
        CoordinationExecutor,
    ) {
        // Create domain IDs
        let ethereum_domain = DomainId::new("ethereum-test");
        let cosmos_domain = DomainId::new("cosmos-test");
        
        // Create EVM adapter
        let evm_config = EvmAdapterConfig {
            domain_id: ethereum_domain.clone(),
            chain_id: 1337, // Local test chain
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            gas_price: Some("1".to_string()),
            auth_token: None,
            debug_mode: true,
        };
        
        let evm_adapter = EvmAdapter::new(evm_config);
        
        // Create CosmWasm adapter
        let cosmwasm_config = CosmWasmAdapterConfig {
            domain_id: cosmos_domain.clone(),
            chain_id: "testing-1".to_string(),
            rpc_endpoints: vec!["http://localhost:26657".to_string()],
            account_prefix: "wasm".to_string(),
            gas_price: None,
            auth_token: None,
            debug_mode: true,
        };
        
        let cosmwasm_adapter = CosmWasmAdapter::new(cosmwasm_config);
        
        // Create broker and register adapters
        let mut broker = CrossVmBroker::new();
        broker.register_adapter(Box::new(evm_adapter.clone())).unwrap();
        broker.register_adapter(Box::new(cosmwasm_adapter.clone())).unwrap();
        
        // Create validator registry
        let mut validator_registry = EffectValidatorRegistry::new();
        
        // Create coordination executor
        let broker_arc = Arc::new(broker.clone());
        let validator_registry_arc = Arc::new(validator_registry.clone());
        let mut executor = CoordinationExecutor::new(broker_arc, validator_registry_arc);
        
        // Register a basic handler
        let handler = Box::new(BasicCoordinationHandler::new("basic_handler"));
        executor.register_handler(handler);
        
        (
            Box::new(evm_adapter),
            Box::new(cosmwasm_adapter),
            broker,
            validator_registry,
            executor,
        )
    }
    
    // Basic coordination handler for testing
    struct BasicCoordinationHandler {
        name: String,
    }
    
    impl BasicCoordinationHandler {
        fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }
    
    impl CoordinationHandler for BasicCoordinationHandler {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn supports_operation(&self, operation: &str) -> bool {
            matches!(operation, "deploy_contract" | "execute_function" | "execute_msg")
        }
        
        fn execute_operation(
            &self,
            step: &CoordinationStep,
            broker: &CrossVmBroker,
        ) -> Result<serde_json::Value> {
            // Get the adapter for this step
            let adapter = broker.get_adapter(&step.domain).ok_or_else(|| {
                Error::NotFoundError(format!("Adapter not found for domain: {}", step.domain.as_ref()))
            })?;
            
            // Mock execution based on VM type and operation
            match adapter.vm_type() {
                VmType::Evm => {
                    // Mock EVM operations
                    if step.operation == "deploy_contract" {
                        Ok(serde_json::json!({
                            "contract_address": "0xdemo123",
                            "transaction_hash": "0xabcdef",
                        }))
                    } else if step.operation == "execute_function" {
                        Ok(serde_json::json!({
                            "success": true,
                            "transaction_hash": "0x123456",
                            "result": "0x",
                        }))
                    } else {
                        Err(Error::NotImplemented(format!("Operation not supported: {}", step.operation)))
                    }
                },
                VmType::CosmWasm => {
                    // Mock CosmWasm operations
                    if step.operation == "deploy_contract" {
                        Ok(serde_json::json!({
                            "contract_address": "wasm1demo123",
                            "code_id": 1,
                        }))
                    } else if step.operation == "execute_msg" {
                        Ok(serde_json::json!({
                            "success": true,
                            "transaction_hash": "ABC123",
                            "events": [
                                {"type": "wasm", "attributes": [{"key": "action", "value": "execute"}]}
                            ],
                        }))
                    } else {
                        Err(Error::NotImplemented(format!("Operation not supported: {}", step.operation)))
                    }
                },
                _ => Err(Error::NotImplemented(format!("VM type not supported: {:?}", adapter.vm_type()))),
            }
        }
    }
    
    #[test]
    fn test_cross_vm_coordination() -> Result<()> {
        // Setup test environment
        let (evm_adapter, cosmwasm_adapter, broker, validator_registry, mut executor) = 
            setup_test_environment();
        
        // Create domain IDs
        let ethereum_domain = evm_adapter.domain_id().clone();
        let cosmos_domain = cosmwasm_adapter.domain_id().clone();
        
        // Create coordination context
        let mut context = CoordinationContext::new("cross_chain_test");
        context.add_domain(ethereum_domain.clone());
        context.add_domain(cosmos_domain.clone());
        context.add_vm_type(VmType::Evm);
        context.add_vm_type(VmType::CosmWasm);
        
        // Create coordination plan
        let mut plan = CoordinationPlan::new(context);
        
        // Step 1: Deploy EVM contract
        let deploy_evm_step = CoordinationStep::new(
            "deploy_evm_contract",
            ethereum_domain.clone(),
            VmType::Evm,
            "deploy_contract",
        );
        deploy_evm_step.add_param("name", serde_json::json!("CrossChainBridge"));
        deploy_evm_step.add_param("bytecode", serde_json::json!("0x..."));
        
        plan.add_step(deploy_evm_step);
        
        // Step 2: Deploy CosmWasm contract
        let deploy_wasm_step = CoordinationStep::new(
            "deploy_wasm_contract",
            cosmos_domain.clone(),
            VmType::CosmWasm,
            "deploy_contract",
        );
        deploy_wasm_step.add_param("name", serde_json::json!("CrossChainReceiver"));
        deploy_wasm_step.add_param("bytecode", serde_json::json!("..."));
        deploy_wasm_step.add_param("init_msg", serde_json::json!({"init": {"owner": "wasm1abc"}}));
        
        plan.add_step(deploy_wasm_step);
        
        // Step 3: Configure EVM contract with CosmWasm contract address
        let configure_evm_step = CoordinationStep::new(
            "configure_evm_contract",
            ethereum_domain.clone(),
            VmType::Evm,
            "execute_function",
        );
        configure_evm_step.add_param("contract", serde_json::json!("0xdemo123"));
        configure_evm_step.add_param("function", serde_json::json!("setCosmosContract"));
        configure_evm_step.add_param("params", serde_json::json!(["wasm1demo123"]));
        configure_evm_step.add_dependency("deploy_evm_contract");
        configure_evm_step.add_dependency("deploy_wasm_contract");
        
        plan.add_step(configure_evm_step);
        
        // Step 4: Configure CosmWasm contract with EVM contract address
        let configure_wasm_step = CoordinationStep::new(
            "configure_wasm_contract",
            cosmos_domain.clone(),
            VmType::CosmWasm,
            "execute_msg",
        );
        configure_wasm_step.add_param("contract", serde_json::json!("wasm1demo123"));
        configure_wasm_step.add_param("msg", serde_json::json!({
            "set_ethereum_contract": {
                "address": "0xdemo123"
            }
        }));
        configure_wasm_step.add_dependency("deploy_evm_contract");
        configure_wasm_step.add_dependency("deploy_wasm_contract");
        
        plan.add_step(configure_wasm_step);
        
        // Validate the plan
        let validation_result = executor.validate_plan(&plan);
        assert!(validation_result.is_valid(), "Plan validation failed: {:?}", validation_result);
        
        // Execute the plan
        let result_plan = executor.execute_plan(plan)?;
        
        // Check that all steps completed successfully
        for step in &result_plan.steps {
            assert_eq!(
                step.status, 
                CoordinationStatus::Completed,
                "Step {} failed: {:?}", 
                step.id, 
                step.status
            );
            assert!(step.result.is_some(), "Step {} has no result", step.id);
        }
        
        // Check some specific results
        let deploy_evm_result = result_plan.get_step("deploy_evm_contract")
            .and_then(|s| s.result.as_ref())
            .expect("Missing deploy_evm_contract result");
        
        let contract_address = deploy_evm_result["contract_address"].as_str()
            .expect("Missing contract_address in result");
        
        assert_eq!(contract_address, "0xdemo123", "Unexpected contract address");
        
        Ok(())
    }
    
    #[test]
    fn test_cross_vm_error_handling() -> Result<()> {
        // Setup test environment
        let (evm_adapter, cosmwasm_adapter, broker, validator_registry, mut executor) = 
            setup_test_environment();
        
        // Create domain IDs
        let ethereum_domain = evm_adapter.domain_id().clone();
        let cosmos_domain = cosmwasm_adapter.domain_id().clone();
        
        // Create coordination context
        let mut context = CoordinationContext::new("error_handling_test");
        context.add_domain(ethereum_domain.clone());
        context.add_domain(cosmos_domain.clone());
        
        // Create coordination plan with an unsupported operation
        let mut plan = CoordinationPlan::new(context);
        
        // Add a step with an unsupported operation
        let invalid_step = CoordinationStep::new(
            "invalid_operation",
            ethereum_domain.clone(),
            VmType::Evm,
            "unsupported_operation", // This operation is not supported
        );
        plan.add_step(invalid_step);
        
        // Add a valid step with a dependency on the invalid step
        let dependent_step = CoordinationStep::new(
            "dependent_step",
            cosmos_domain.clone(),
            VmType::CosmWasm,
            "deploy_contract",
        );
        dependent_step.add_dependency("invalid_operation");
        plan.add_step(dependent_step);
        
        // Execute the plan
        let result_plan = executor.execute_plan(plan)?;
        
        // Check that the invalid step failed
        let invalid_step_result = result_plan.get_step("invalid_operation")
            .expect("Missing invalid_operation step");
        
        match &invalid_step_result.status {
            CoordinationStatus::Failed(msg) => {
                assert!(msg.contains("not supported"), "Unexpected error message: {}", msg);
            },
            status => panic!("Expected Failed status, got {:?}", status),
        }
        
        // Check that the dependent step was not executed
        let dependent_step_result = result_plan.get_step("dependent_step")
            .expect("Missing dependent_step step");
        
        assert_eq!(
            dependent_step_result.status, 
            CoordinationStatus::Pending,
            "Expected Pending status for dependent step"
        );
        
        Ok(())
    }
} 