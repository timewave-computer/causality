// CosmWasm Execute Effect
//
// This module provides the implementation of the CosmWasm execute effect,
// which allows executing contract calls on CosmWasm-based chains.

use std::collections::HashMap;
use std::fmt;
use std::any::Any;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_types::content::ContentId;
use causality_core::effect::{
    Effect, EffectContext, EffectResult, EffectError, EffectOutcome, EffectType,
    domain::DomainEffect
};

use super::{CosmWasmEffect, CosmWasmEffectType, CosmWasmGasParams, CosmWasmEffectHandler, COSMWASM_DOMAIN_ID};

/// Parameters for CosmWasm execute effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmExecuteParams {
    /// Chain ID to operate on
    pub chain_id: String,
    
    /// Contract address
    pub contract_address: String,
    
    /// Sender address
    pub sender: String,
    
    /// Execute message (JSON)
    pub msg: Value,
    
    /// Funds to send with execute
    pub funds: Vec<Coin>,
    
    /// Gas parameters
    pub gas_params: Option<CosmWasmGasParams>,
}

/// Coin representation for funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    /// Denom (token symbol)
    pub denom: String,
    
    /// Amount
    pub amount: String,
}

/// CosmWasm Execute Effect implementation
pub struct CosmWasmExecuteEffect {
    /// Unique identifier
    id: String,
    
    /// Execute parameters
    params: CosmWasmExecuteParams,
    
    /// Resource ID representing the contract
    contract_resource_id: ContentId,
    
    /// Resource ID representing the user account
    account_resource_id: ContentId,
}

impl CosmWasmExecuteEffect {
    /// Create a new CosmWasm execute effect
    pub fn new(
        params: CosmWasmExecuteParams,
        contract_resource_id: ContentId,
        account_resource_id: ContentId,
    ) -> Self {
        Self {
            id: format!("exec-{}-{}", params.chain_id, params.contract_address),
            params,
            contract_resource_id,
            account_resource_id,
        }
    }
    
    /// Create a new CosmWasm execute effect with a specific ID
    pub fn with_id(
        id: String,
        params: CosmWasmExecuteParams,
        contract_resource_id: ContentId,
        account_resource_id: ContentId,
    ) -> Self {
        Self {
            id,
            params,
            contract_resource_id,
            account_resource_id,
        }
    }
    
    /// Get the parameters for this execute
    pub fn params(&self) -> &CosmWasmExecuteParams {
        &self.params
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.params.contract_address
    }
    
    /// Get the sender address
    pub fn sender(&self) -> &str {
        &self.params.sender
    }
    
    /// Get the execute message
    pub fn msg(&self) -> &Value {
        &self.params.msg
    }
    
    /// Get the funds being sent
    pub fn funds(&self) -> &[Coin] {
        &self.params.funds
    }
}

impl fmt::Debug for CosmWasmExecuteEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CosmWasmExecuteEffect")
            .field("id", &self.id)
            .field("chain_id", &self.params.chain_id)
            .field("contract_address", &self.params.contract_address)
            .field("sender", &self.params.sender)
            .field("msg", &self.params.msg)
            .field("funds", &self.params.funds)
            .finish()
    }
}

#[async_trait::async_trait]
impl Effect for CosmWasmExecuteEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("cosmwasm.execute".to_string())
    }
    
    fn description(&self) -> String {
        format!("CosmWasm Contract Execute: {} on {}", 
                self.params.contract_address, 
                self.params.chain_id)
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult {
        // Verify capability for contract access
        if !context.has_capability(&format!("cosmwasm.contract.{}", self.params.contract_address)) {
            return Err(EffectError::PermissionDenied(
                format!("Missing capability to call contract: {}", self.params.contract_address)
            ));
        }
        
        // Verify capability for account access
        if !context.has_capability(&format!("cosmwasm.account.{}", self.params.sender)) {
            return Err(EffectError::PermissionDenied(
                format!("Missing capability to use account: {}", self.params.sender)
            ));
        }
        
        // Prepare outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.clone());
        outcome_data.insert("contract_address".to_string(), self.params.contract_address.clone());
        outcome_data.insert("sender".to_string(), self.params.sender.clone());
        outcome_data.insert("msg".to_string(), self.params.msg.to_string());
        
        // Add funds information
        if !self.params.funds.is_empty() {
            let funds_str = self.params.funds.iter()
                .map(|c| format!("{}{}", c.amount, c.denom))
                .collect::<Vec<String>>()
                .join(",");
            outcome_data.insert("funds".to_string(), funds_str);
        }
        
        // In a real implementation, we would execute the contract call here
        // For now, we'll just return a simulated success
        
        Ok(EffectOutcome::Success {
            result: Some("contract_executed".to_string()),
            data: outcome_data,
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DomainEffect for CosmWasmExecuteEffect {
    fn domain_id(&self) -> String {
        COSMWASM_DOMAIN_ID.to_string()
    }
    
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.clone());
        params
    }
}

#[async_trait::async_trait]
impl CosmWasmEffect for CosmWasmExecuteEffect {
    fn cosmwasm_effect_type(&self) -> CosmWasmEffectType {
        CosmWasmEffectType::Execute
    }
    
    fn chain_id(&self) -> &str {
        &self.params.chain_id
    }
    
    fn gas_params(&self) -> Option<CosmWasmGasParams> {
        self.params.gas_params.clone()
    }
    
    async fn handle_with_handler(&self, handler: &dyn CosmWasmEffectHandler, context: &dyn EffectContext) -> EffectResult {
        handler.handle_cosmwasm_effect(self, context).await
    }
} 