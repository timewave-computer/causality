// CosmWasm Instantiate Effect
//
// This module provides the implementation of the CosmWasm instantiate effect,
// which allows instantiating (deploying) contracts on CosmWasm-based chains.

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

#[async_trait]
impl Effect for CosmWasmInstantiateEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new("cosmwasm.instantiate")
    }
    
    fn display_name(&self) -> String {
        "CosmWasm Contract Instantiate".to_string()
    }
    
    fn description(&self) -> String {
        format!(
            "Instantiate contract from code ID {} with label '{}' on chain {} with sender {}",
            self.params.code_id,
            self.params.label,
            self.params.chain_id,
            self.params.sender
        )
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.clone());
        params.insert("code_id".to_string(), self.params.code_id.to_string());
        params.insert("label".to_string(), self.params.label.clone());
        params.insert("sender".to_string(), self.params.sender.clone());
        
        if let Some(admin) = &self.params.admin {
            params.insert("admin".to_string(), admin.clone());
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
        if !context.has_capability(&self.code_resource_id, &causality_core::capability::Right::Create) {
            return Err(EffectError::CapabilityError(
                format!("Missing create capability for code resource: {}", self.code_resource_id)
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
        outcome_data.insert("code_id".to_string(), self.params.code_id.to_string());
        outcome_data.insert("label".to_string(), self.params.label.clone());
        outcome_data.insert("sender".to_string(), self.params.sender.clone());
        
        // Add mock contract address
        let mock_contract_address = format!("cosmos1mock{}", self.params.code_id);
        outcome_data.insert("contract_address".to_string(), mock_contract_address);
        
        // Return success outcome
        Ok(EffectOutcome::success(outcome_data)
            .with_affected_resource(self.code_resource_id.clone())
            .with_affected_resource(self.account_resource_id.clone()))
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