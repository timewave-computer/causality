// CosmWasm Execute Effect
//
// This module provides the implementation of the CosmWasm execute effect,
// which allows executing contract calls on CosmWasm-based chains.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_core::effect::{
    Effect, EffectContext, EffectId, EffectOutcome, EffectResult, EffectError,
    DomainEffect, ResourceEffect, ResourceOperation, EffectTypeId
};
use causality_core::resource::ContentId;

use super::{CosmWasmEffect, CosmWasmEffectType, CosmWasmGasParams, COSMWASM_DOMAIN_ID};

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
    id: EffectId,
    
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
            id: EffectId::new_unique(),
            params,
            contract_resource_id,
            account_resource_id,
        }
    }
    
    /// Create a new CosmWasm execute effect with a specific ID
    pub fn with_id(
        id: EffectId,
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

#[async_trait]
impl Effect for CosmWasmExecuteEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new("cosmwasm.execute")
    }
    
    fn display_name(&self) -> String {
        "CosmWasm Contract Execute".to_string()
    }
    
    fn description(&self) -> String {
        format!(
            "Execute contract {} on chain {} with sender {}",
            self.params.contract_address,
            self.params.chain_id,
            self.params.sender
        )
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.clone());
        params.insert("contract_address".to_string(), self.params.contract_address.clone());
        params.insert("sender".to_string(), self.params.sender.clone());
        
        // Add message type if available
        if let Some(msg_type) = self.params.msg.get("type").and_then(|v| v.as_str()) {
            params.insert("msg_type".to_string(), msg_type.to_string());
        }
        
        // Add funds information
        if !self.params.funds.is_empty() {
            let funds_str = self.params.funds
                .iter()
                .map(|c| format!("{}{}", c.amount, c.denom))
                .collect::<Vec<_>>()
                .join(", ");
            
            params.insert("funds".to_string(), funds_str);
        }
        
        params
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would call out to the CosmWasm chain
        // For now, we'll just return a success outcome
        
        // Check capabilities
        if !context.has_capability(&self.contract_resource_id, &causality_core::capability::Right::Call) {
            return Err(EffectError::CapabilityError(
                format!("Missing call capability for contract resource: {}", self.contract_resource_id)
            ));
        }
        
        if !context.has_capability(&self.account_resource_id, &causality_core::capability::Right::Write) {
            return Err(EffectError::CapabilityError(
                format!("Missing write capability for account resource: {}", self.account_resource_id)
            ));
        }
        
        // Create outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.clone());
        outcome_data.insert("contract_address".to_string(), self.params.contract_address.clone());
        outcome_data.insert("sender".to_string(), self.params.sender.clone());
        
        // Add message type if available
        if let Some(msg_type) = self.params.msg.get("type").and_then(|v| v.as_str()) {
            outcome_data.insert("msg_type".to_string(), msg_type.to_string());
        }
        
        // Return success outcome
        Ok(EffectOutcome::success(outcome_data)
            .with_affected_resource(self.contract_resource_id.clone())
            .with_affected_resource(self.account_resource_id.clone()))
    }
}

#[async_trait]
impl DomainEffect for CosmWasmExecuteEffect {
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
impl ResourceEffect for CosmWasmExecuteEffect {
    fn resource_id(&self) -> &ContentId {
        &self.contract_resource_id
    }
    
    fn operation(&self) -> ResourceOperation {
        ResourceOperation::Update
    }
}

#[async_trait]
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
} 