// CosmWasm Instantiate Effect
//
// This module provides the implementation of the CosmWasm instantiate effect,
// which allows instantiating (deploying) contracts on CosmWasm-based chains.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::any::Any;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_core::effect::{
    Effect, EffectContext, EffectId, EffectOutcome, EffectResult, EffectError,
    DomainEffect, ResourceEffect, ResourceOperation, EffectTypeId, EffectType
};
use causality_core::resource::ContentId;

use super::{CosmWasmEffect, CosmWasmEffectType, CosmWasmGasParams, COSMWASM_DOMAIN_ID};
use super::execute::Coin;

/// Parameters for CosmWasm instantiate effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmInstantiateParams {
    /// Chain ID to operate on
    pub chain_id: String,
    
    /// Code ID to instantiate
    pub code_id: u64,
    
    /// Label for the contract
    pub label: String,
    
    /// Sender address
    pub sender: String,
    
    /// Admin address (optional)
    pub admin: Option<String>,
    
    /// Instantiate message (JSON)
    pub msg: Value,
    
    /// Funds to send with instantiate
    pub funds: Vec<Coin>,
    
    /// Gas parameters
    pub gas_params: Option<CosmWasmGasParams>,
}

/// CosmWasm Instantiate Effect implementation
pub struct CosmWasmInstantiateEffect {
    /// Unique identifier
    id: EffectId,
    
    /// Instantiate parameters
    params: CosmWasmInstantiateParams,
    
    /// Resource ID representing the code
    code_resource_id: ContentId,
    
    /// Resource ID representing the user account
    account_resource_id: ContentId,
}

impl CosmWasmInstantiateEffect {
    /// Create a new CosmWasm instantiate effect
    pub fn new(
        params: CosmWasmInstantiateParams,
        code_resource_id: ContentId,
        account_resource_id: ContentId,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            params,
            code_resource_id,
            account_resource_id,
        }
    }
    
    /// Create a new CosmWasm instantiate effect with a specific ID
    pub fn with_id(
        id: EffectId,
        params: CosmWasmInstantiateParams,
        code_resource_id: ContentId,
        account_resource_id: ContentId,
    ) -> Self {
        Self {
            id,
            params,
            code_resource_id,
            account_resource_id,
        }
    }
    
    /// Get the parameters for this instantiate
    pub fn params(&self) -> &CosmWasmInstantiateParams {
        &self.params
    }
    
    /// Get the code ID
    pub fn code_id(&self) -> u64 {
        self.params.code_id
    }
    
    /// Get the label
    pub fn label(&self) -> &str {
        &self.params.label
    }
    
    /// Get the sender address
    pub fn sender(&self) -> &str {
        &self.params.sender
    }
    
    /// Get the admin address
    pub fn admin(&self) -> Option<&str> {
        self.params.admin.as_deref()
    }
    
    /// Get the instantiate message
    pub fn msg(&self) -> &Value {
        &self.params.msg
    }
    
    /// Get the funds being sent
    pub fn funds(&self) -> &[Coin] {
        &self.params.funds
    }
}

impl fmt::Debug for CosmWasmInstantiateEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CosmWasmInstantiateEffect")
            .field("id", &self.id)
            .field("chain_id", &self.params.chain_id)
            .field("code_id", &self.params.code_id)
            .field("label", &self.params.label)
            .field("sender", &self.params.sender)
            .field("admin", &self.params.admin)
            .field("msg", &self.params.msg)
            .field("funds", &self.params.funds)
            .finish()
    }
}

#[async_trait::async_trait]
impl Effect for CosmWasmInstantiateEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("cosmwasm.instantiate".to_string())
    }
    
    fn description(&self) -> String {
        format!("CosmWasm Instantiate: Code {} on {}", 
                self.params.code_id, 
                self.params.chain_id)
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult {
        // Verify capability for code access
        if !context.has_capability(&format!("cosmwasm.code.{}", self.params.code_id)) {
            return Err(EffectError::PermissionDenied(
                format!("Missing capability to use code: {}", self.params.code_id)
            ));
        }
        
        // Verify capability for account access
        if !context.has_capability(&format!("cosmwasm.account.{}", self.params.admin)) {
            return Err(EffectError::PermissionDenied(
                format!("Missing capability to use account: {}", self.params.admin)
            ));
        }
        
        // Prepare outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.clone());
        outcome_data.insert("code_id".to_string(), self.params.code_id.to_string());
        outcome_data.insert("admin".to_string(), self.params.admin.clone());
        outcome_data.insert("label".to_string(), self.params.label.clone());
        outcome_data.insert("msg".to_string(), self.params.msg.to_string());
        
        // Add funds information
        if !self.params.funds.is_empty() {
            let funds_str = self.params.funds.iter()
                .map(|c| format!("{}{}", c.amount, c.denom))
                .collect::<Vec<String>>()
                .join(",");
            outcome_data.insert("funds".to_string(), funds_str);
        }
        
        // In a real implementation, we would execute the instantiate here
        // For now, we'll just return a simulated result with a contract address
        outcome_data.insert("contract_address".to_string(), "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string());
        
        Ok(EffectOutcome::Success {
            result: Some("contract_instantiated".to_string()),
            data: outcome_data,
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl DomainEffect for CosmWasmInstantiateEffect {
    fn domain_id(&self) -> &str {
        COSMWASM_DOMAIN_ID
    }
    
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.clone());
        params
    }
}

#[async_trait]
impl ResourceEffect for CosmWasmInstantiateEffect {
    fn resource_id(&self) -> &ContentId {
        &self.code_resource_id
    }
    
    fn operation(&self) -> ResourceOperation {
        ResourceOperation::Create
    }
}

#[async_trait]
impl CosmWasmEffect for CosmWasmInstantiateEffect {
    fn cosmwasm_effect_type(&self) -> CosmWasmEffectType {
        CosmWasmEffectType::Instantiate
    }
    
    fn chain_id(&self) -> &str {
        &self.params.chain_id
    }
    
    fn gas_params(&self) -> Option<CosmWasmGasParams> {
        self.params.gas_params.clone()
    }
} 