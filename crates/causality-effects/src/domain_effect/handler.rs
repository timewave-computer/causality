// Domain Effect Handler
//
// This module implements a handler for domain effects that integrates
// with the Effect System's handler mechanism.

use std::sync::Arc;
use std::collections::HashMap;

use async_trait::async_trait;

use causality_domain::adapter::DomainAdapterRegistry;
use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
use crate::handler::{EffectHandler, HandlerResult};
use crate::domain_effect::{
    DomainAdapterEffect, DomainContext, DomainQueryEffect,
    DomainTransactionEffect, DomainTimeMapEffect, DomainCapabilityEffect,
    domain_registry::{EffectDomainRegistry, DomainEffectHandler},
    domain_selection::{DomainSelectionEffect, DomainSelectionHandler},
    evm_effects::{EvmContractCallEffect, EvmStateQueryEffect, EvmGasEstimationEffect},
    cosmwasm_effects::{CosmWasmExecuteEffect, CosmWasmQueryEffect, CosmWasmInstantiateEffect, CosmWasmCodeUploadEffect},
    zk_effects::{ZkProveEffect, ZkVerifyEffect, ZkWitnessEffect, ZkProofCompositionEffect}
};

/// Handler for domain effects
///
/// This handler provides integration between the Effect System's handler
/// framework and domain-specific effects.
pub struct DomainEffectHandlerAdapter {
    /// Domain registry
    registry: Arc<EffectDomainRegistry>,
}

impl DomainEffectHandlerAdapter {
    /// Create a new domain effect handler adapter
    pub fn new(registry: Arc<EffectDomainRegistry>) -> Self {
        Self {
            registry,
        }
    }
    
    /// Get the domain registry
    pub fn registry(&self) -> &EffectDomainRegistry {
        &self.registry
    }
}

#[async_trait]
impl EffectHandler for DomainEffectHandlerAdapter {
    async fn handle(&self, effect: Arc<dyn Effect>, context: &EffectContext) -> HandlerResult {
        // Check if this is a domain effect
        if !self.registry.can_handle_effect(effect.as_ref()) {
            return HandlerResult::NotApplicable;
        }
        
        // Try to downcast to specific domain effect types
        if let Some(domain_effect) = effect.as_any().downcast_ref::<DomainQueryEffect>() {
            let result = self.registry.execute_domain_effect(domain_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(domain_effect) = effect.as_any().downcast_ref::<DomainTransactionEffect>() {
            let result = self.registry.execute_domain_effect(domain_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(domain_effect) = effect.as_any().downcast_ref::<DomainTimeMapEffect>() {
            let result = self.registry.execute_domain_effect(domain_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(domain_effect) = effect.as_any().downcast_ref::<DomainCapabilityEffect>() {
            let result = self.registry.execute_domain_effect(domain_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(selection_effect) = effect.as_any().downcast_ref::<DomainSelectionEffect>() {
            // Domain selection effect
            let result = self.registry.execute_selection(selection_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(evm_call_effect) = effect.as_any().downcast_ref::<EvmContractCallEffect>() {
            // EVM contract call effect
            let result = self.execute_evm_contract_call(evm_call_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(evm_query_effect) = effect.as_any().downcast_ref::<EvmStateQueryEffect>() {
            // EVM state query effect
            let result = self.execute_evm_state_query(evm_query_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(evm_gas_effect) = effect.as_any().downcast_ref::<EvmGasEstimationEffect>() {
            // EVM gas estimation effect
            let result = self.execute_evm_gas_estimation(evm_gas_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(cosmwasm_execute_effect) = effect.as_any().downcast_ref::<CosmWasmExecuteEffect>() {
            // CosmWasm execute effect
            let result = self.execute_cosmwasm_execute(cosmwasm_execute_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(cosmwasm_query_effect) = effect.as_any().downcast_ref::<CosmWasmQueryEffect>() {
            // CosmWasm query effect
            let result = self.execute_cosmwasm_query(cosmwasm_query_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(cosmwasm_instantiate_effect) = effect.as_any().downcast_ref::<CosmWasmInstantiateEffect>() {
            // CosmWasm instantiate effect
            let result = self.execute_cosmwasm_instantiate(cosmwasm_instantiate_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(cosmwasm_upload_effect) = effect.as_any().downcast_ref::<CosmWasmCodeUploadEffect>() {
            // CosmWasm code upload effect
            let result = self.execute_cosmwasm_upload(cosmwasm_upload_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(zk_prove_effect) = effect.as_any().downcast_ref::<ZkProveEffect>() {
            // ZK prove effect
            let result = self.execute_zk_prove(zk_prove_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(zk_verify_effect) = effect.as_any().downcast_ref::<ZkVerifyEffect>() {
            // ZK verify effect
            let result = self.execute_zk_verify(zk_verify_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(zk_witness_effect) = effect.as_any().downcast_ref::<ZkWitnessEffect>() {
            // ZK witness effect
            let result = self.execute_zk_witness(zk_witness_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        } else if let Some(zk_composition_effect) = effect.as_any().downcast_ref::<ZkProofCompositionEffect>() {
            // ZK composition effect
            let result = self.execute_zk_composition(zk_composition_effect, context).await;
            return match result {
                Ok(outcome) => HandlerResult::Handled(outcome),
                Err(err) => HandlerResult::Error(Box::new(err)),
            };
        }
        
        // This shouldn't happen if can_handle_effect is correct
        HandlerResult::NotApplicable
    }
    
    fn can_handle(&self, effect: &dyn Effect) -> bool {
        self.registry.can_handle_effect(effect) ||
        effect.as_any().downcast_ref::<EvmContractCallEffect>().is_some() ||
        effect.as_any().downcast_ref::<EvmStateQueryEffect>().is_some() ||
        effect.as_any().downcast_ref::<EvmGasEstimationEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmExecuteEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmQueryEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmInstantiateEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmCodeUploadEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkProveEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkVerifyEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkWitnessEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkProofCompositionEffect>().is_some()
    }
}

impl DomainEffectHandlerAdapter {
    // Execute an EVM contract call effect
    async fn execute_evm_contract_call(
        &self, 
        effect: &EvmContractCallEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Handle the effect based on whether it's a view call or transaction call
        if effect.is_view() {
            // For view calls, we would typically use observe_fact
            // In a real implementation, this would be more complex with encoding, etc.
            let mut query = causality_domain::fact::FactQuery {
                domain_id: domain_id.clone(),
                fact_type: format!("evm.call.{}", effect.function_name()),
                parameters: HashMap::new(),
                block_height: None,
                block_hash: None,
                timestamp: None,
            };
            
            // Add function parameters
            query.parameters.insert("contract_address".to_string(), effect.contract_address().to_string());
            query.parameters.insert("function_name".to_string(), effect.function_name().to_string());
            
            // Add args
            for (i, arg) in effect.args().iter().enumerate() {
                query.parameters.insert(format!("arg_{}", i), arg.clone());
            }
            
            // Execute query
            let fact_result = adapter.observe_fact(&query).await
                .map_err(|e| EffectError::ExecutionError(format!("EVM view call failed: {}", e)))?;
            
            // Map result
            let result = if let Some(result) = fact_result.data.get("result") {
                result.clone()
            } else {
                "".to_string()
            };
            
            effect.map_outcome(Ok(result))
        } else {
            // For transaction calls, we would use submit_transaction
            // In a real implementation, this would be more complex with encoding, etc.
            let mut tx = causality_domain::domain::Transaction {
                domain_id: domain_id.clone(),
                tx_type: "evm.contract_call".to_string(),
                parameters: HashMap::new(),
            };
            
            // Add transaction parameters
            tx.parameters.insert("contract_address".to_string(), effect.contract_address().to_string());
            tx.parameters.insert("function_name".to_string(), effect.function_name().to_string());
            
            // Add args
            for (i, arg) in effect.args().iter().enumerate() {
                tx.parameters.insert(format!("arg_{}", i), arg.clone());
            }
            
            // Submit transaction
            let tx_id = adapter.submit_transaction(&tx).await
                .map_err(|e| EffectError::ExecutionError(format!("EVM transaction call failed: {}", e)))?;
            
            effect.map_outcome(Ok(tx_id))
        }
    }
    
    // Execute an EVM state query effect
    async fn execute_evm_state_query(
        &self, 
        effect: &EvmStateQueryEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create query for the state lookup
        // In a real implementation, this would need to be more sophisticated
        let mut query = causality_domain::fact::FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "evm.state_query".to_string(),
            parameters: HashMap::new(),
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Add query parameters based on the query type
        use crate::domain_effect::evm_effects::EvmStateQueryType;
        match effect.query_type() {
            EvmStateQueryType::Balance(address) => {
                query.fact_type = "evm.balance".to_string();
                query.parameters.insert("address".to_string(), address.clone());
            },
            EvmStateQueryType::Storage(address, slot) => {
                query.fact_type = "evm.storage".to_string();
                query.parameters.insert("address".to_string(), address.clone());
                query.parameters.insert("slot".to_string(), slot.clone());
            },
            EvmStateQueryType::Code(address) => {
                query.fact_type = "evm.code".to_string();
                query.parameters.insert("address".to_string(), address.clone());
            },
            EvmStateQueryType::Nonce(address) => {
                query.fact_type = "evm.nonce".to_string();
                query.parameters.insert("address".to_string(), address.clone());
            },
            EvmStateQueryType::Block(block_id) => {
                query.fact_type = "evm.block".to_string();
                query.parameters.insert("block_id".to_string(), block_id.clone());
            },
            EvmStateQueryType::Transaction(tx_hash) => {
                query.fact_type = "evm.transaction".to_string();
                query.parameters.insert("tx_hash".to_string(), tx_hash.clone());
            },
            EvmStateQueryType::Receipt(tx_hash) => {
                query.fact_type = "evm.receipt".to_string();
                query.parameters.insert("tx_hash".to_string(), tx_hash.clone());
            },
            EvmStateQueryType::GasPrice => {
                query.fact_type = "evm.gas_price".to_string();
            },
            EvmStateQueryType::GasLimit => {
                query.fact_type = "evm.gas_limit".to_string();
            },
            EvmStateQueryType::ChainId => {
                query.fact_type = "evm.chain_id".to_string();
            },
        }
        
        // Execute query
        let fact_result = adapter.observe_fact(&query).await
            .map_err(|e| EffectError::ExecutionError(format!("EVM state query failed: {}", e)))?;
        
        // Map result
        let result_key = match effect.query_type() {
            EvmStateQueryType::Balance(_) => "balance",
            EvmStateQueryType::Storage(_, _) => "storage_value",
            EvmStateQueryType::Code(_) => "code",
            EvmStateQueryType::Nonce(_) => "nonce",
            EvmStateQueryType::Block(_) => "block",
            EvmStateQueryType::Transaction(_) => "transaction",
            EvmStateQueryType::Receipt(_) => "receipt",
            EvmStateQueryType::GasPrice => "gas_price",
            EvmStateQueryType::GasLimit => "gas_limit",
            EvmStateQueryType::ChainId => "chain_id",
        };
        
        let result = fact_result.data.get(result_key)
            .cloned()
            .unwrap_or_else(|| "".to_string());
        
        effect.map_outcome(Ok(result))
    }
    
    // Execute an EVM gas estimation effect
    async fn execute_evm_gas_estimation(
        &self, 
        effect: &EvmGasEstimationEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // In a real implementation, we would estimate the gas of a transaction
        // For this mock implementation, we'll just return a fixed value
        effect.map_outcome(Ok(100000))
    }
    
    // Execute a CosmWasm contract execute effect
    async fn execute_cosmwasm_execute(
        &self, 
        effect: &CosmWasmExecuteEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for the contract execution
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "cosmwasm.execute".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("contract_address".to_string(), effect.contract_address().to_string());
        tx.parameters.insert("msg".to_string(), effect.msg().to_string());
        
        // Add sender if provided
        if let Some(sender) = context.params.get("sender").or_else(|| context.params.get("caller")) {
            tx.parameters.insert("sender".to_string(), sender.clone());
        }
        
        // Add funds if provided
        if let Some(funds) = effect.funds() {
            for (i, (denom, amount)) in funds.iter().enumerate() {
                tx.parameters.insert(format!("fund_{}_denom", i), denom.clone());
                tx.parameters.insert(format!("fund_{}_amount", i), amount.to_string());
            }
            tx.parameters.insert("funds_count".to_string(), funds.len().to_string());
        }
        
        // Submit transaction
        let tx_hash = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("CosmWasm execute failed: {}", e)))?;
        
        effect.map_outcome(Ok(tx_hash))
    }
    
    // Execute a CosmWasm contract query effect
    async fn execute_cosmwasm_query(
        &self, 
        effect: &CosmWasmQueryEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create query for the contract
        let mut query = causality_domain::fact::FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "cosmwasm.query".to_string(),
            parameters: HashMap::new(),
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Add query parameters
        query.parameters.insert("contract_address".to_string(), effect.contract_address().to_string());
        query.parameters.insert("query".to_string(), effect.query().to_string());
        
        // Execute query
        let fact_result = adapter.observe_fact(&query).await
            .map_err(|e| EffectError::ExecutionError(format!("CosmWasm query failed: {}", e)))?;
        
        // Extract result data
        let result = fact_result.data.get("result")
            .cloned()
            .unwrap_or_else(|| "{}".to_string());
        
        effect.map_outcome(Ok(result))
    }
    
    // Execute a CosmWasm contract instantiate effect
    async fn execute_cosmwasm_instantiate(
        &self, 
        effect: &CosmWasmInstantiateEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for the contract instantiation
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "cosmwasm.instantiate".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("code_id".to_string(), effect.code_id().to_string());
        tx.parameters.insert("msg".to_string(), effect.msg().to_string());
        tx.parameters.insert("label".to_string(), effect.label().to_string());
        
        // Add sender if provided
        if let Some(sender) = context.params.get("sender").or_else(|| context.params.get("caller")) {
            tx.parameters.insert("sender".to_string(), sender.clone());
        }
        
        // Add admin if provided in effect parameters
        if let Some(admin) = context.params.get("admin") {
            tx.parameters.insert("admin".to_string(), admin.clone());
        }
        
        // Add funds if provided
        if let Some(funds) = effect.funds() {
            for (i, (denom, amount)) in funds.iter().enumerate() {
                tx.parameters.insert(format!("fund_{}_denom", i), denom.clone());
                tx.parameters.insert(format!("fund_{}_amount", i), amount.to_string());
            }
            tx.parameters.insert("funds_count".to_string(), funds.len().to_string());
        }
        
        // Submit transaction
        let tx_receipt = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("CosmWasm instantiate failed: {}", e)))?;
        
        // For instantiation, we need both the transaction hash and the new contract address
        // In a real implementation, this would parse the receipt to extract the contract address
        // For this mock implementation, we'll just assume the receipt is the tx hash and create a fake address
        let contract_address = format!("cosmos1instantiated{}", effect.code_id());
        
        effect.map_outcome(Ok((tx_receipt, contract_address)))
    }
    
    // Execute a CosmWasm code upload effect
    async fn execute_cosmwasm_upload(
        &self, 
        effect: &CosmWasmCodeUploadEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for the code upload
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "cosmwasm.upload".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("wasm_bytecode".to_string(), effect.wasm_bytecode().to_string());
        
        // Add sender if provided
        if let Some(sender) = context.params.get("sender").or_else(|| context.params.get("caller")) {
            tx.parameters.insert("sender".to_string(), sender.clone());
        }
        
        // Submit transaction
        let tx_receipt = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("CosmWasm upload failed: {}", e)))?;
        
        // For code upload, we need both the transaction hash and the new code ID
        // In a real implementation, this would parse the receipt to extract the code ID
        // For this mock implementation, we'll just assume the receipt is the tx hash and create a fake code ID
        let code_id = 123u64; // In a real implementation, this would be parsed from the receipt
        
        effect.map_outcome(Ok((tx_receipt, code_id)))
    }
    
    // Execute a ZK prove effect
    async fn execute_zk_prove(
        &self, 
        effect: &ZkProveEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for the proof generation
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "zk.prove".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("circuit_id".to_string(), effect.circuit_id().to_string());
        tx.parameters.insert("private_inputs".to_string(), effect.private_inputs().to_string());
        
        // Add public inputs
        for (i, input) in effect.public_inputs().iter().enumerate() {
            tx.parameters.insert(format!("public_input_{}", i), input.clone());
        }
        tx.parameters.insert("public_inputs_count".to_string(), effect.public_inputs().len().to_string());
        
        // Submit transaction to generate proof
        let proof_hash = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("ZK proof generation failed: {}", e)))?;
        
        effect.map_outcome(Ok(proof_hash))
    }
    
    // Execute a ZK verify effect
    async fn execute_zk_verify(
        &self, 
        effect: &ZkVerifyEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create query for the proof verification
        let mut query = causality_domain::fact::FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "zk.verify".to_string(),
            parameters: HashMap::new(),
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Add query parameters
        query.parameters.insert("verification_key_id".to_string(), effect.verification_key_id().to_string());
        query.parameters.insert("proof".to_string(), effect.proof().to_string());
        
        // Add public inputs
        for (i, input) in effect.public_inputs().iter().enumerate() {
            query.parameters.insert(format!("public_input_{}", i), input.clone());
        }
        query.parameters.insert("public_inputs_count".to_string(), effect.public_inputs().len().to_string());
        
        // Execute verification query
        let fact_result = adapter.observe_fact(&query).await
            .map_err(|e| EffectError::ExecutionError(format!("ZK proof verification failed: {}", e)))?;
        
        // Extract success result
        let success = fact_result.data.get("success")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        
        effect.map_outcome(Ok(success))
    }
    
    // Execute a ZK witness effect
    async fn execute_zk_witness(
        &self, 
        effect: &ZkWitnessEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for witness creation
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "zk.witness".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("circuit_id".to_string(), effect.circuit_id().to_string());
        tx.parameters.insert("witness_data".to_string(), effect.witness_data().to_string());
        
        // Submit transaction
        let witness_hash = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("ZK witness creation failed: {}", e)))?;
        
        effect.map_outcome(Ok(witness_hash))
    }
    
    // Execute a ZK proof composition effect
    async fn execute_zk_composition(
        &self, 
        effect: &ZkProofCompositionEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get domain adapter
        let domain_id = effect.domain_id();
        let adapter = self.registry.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
        
        // Create transaction for proof composition
        let mut tx = causality_domain::domain::Transaction {
            domain_id: domain_id.clone(),
            tx_type: "zk.compose".to_string(),
            parameters: HashMap::new(),
        };
        
        // Add transaction parameters
        tx.parameters.insert("composition_circuit_id".to_string(), effect.composition_circuit_id().to_string());
        
        // Add source proof hashes
        for (i, hash) in effect.source_proof_hashes().iter().enumerate() {
            tx.parameters.insert(format!("source_proof_hash_{}", i), hash.clone());
        }
        tx.parameters.insert("source_proof_count".to_string(), effect.source_proof_hashes().len().to_string());
        
        // Submit transaction
        let result_proof_hash = adapter.submit_transaction(&tx).await
            .map_err(|e| EffectError::ExecutionError(format!("ZK proof composition failed: {}", e)))?;
        
        effect.map_outcome(Ok(result_proof_hash))
    }
}

/// Create a domain effect handler adapter
pub fn create_domain_handler(registry: Arc<EffectDomainRegistry>) -> DomainEffectHandlerAdapter {
    DomainEffectHandlerAdapter::new(registry)
}

/// Create a domain effect handler adapter with a new registry
pub fn create_domain_handler_with_new_registry() -> (Arc<EffectDomainRegistry>, DomainEffectHandlerAdapter) {
    let registry = Arc::new(EffectDomainRegistry::new());
    let handler = DomainEffectHandlerAdapter::new(registry.clone());
    (registry, handler)
} 