// CosmWasm Query Effect
//
// This module provides the implementation of the CosmWasm query effect,
// which allows querying contract state on CosmWasm-based chains.

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

use super::{CosmWasmEffect, CosmWasmEffectType, COSMWASM_DOMAIN_ID};

/// Query target type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryTarget {
    /// Smart contract query
    SmartContract,
    /// Raw contract storage query
    RawStorage,
    /// Balance query
    Balance,
    /// Staking query
    Staking,
    /// Chain info query
    ChainInfo,
}

/// Parameters for CosmWasm query effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmQueryParams {
    /// Chain ID to operate on
    pub chain_id: String,
    
    /// Contract address (for contract queries)
    pub contract_address: Option<String>,
    
    /// Query target
    pub target: QueryTarget,
    
    /// Query message (JSON)
    pub query_msg: Value,
    
    /// Height to query at (optional)
    pub height: Option<u64>,
}

/// CosmWasm Query Effect implementation
pub struct CosmWasmQueryEffect {
    /// Unique identifier
    id: EffectId,
    
    /// Query parameters
    params: CosmWasmQueryParams,
    
    /// Resource ID representing the target
    resource_id: ContentId,
}

impl CosmWasmQueryEffect {
    /// Create a new CosmWasm query effect
    pub fn new(
        params: CosmWasmQueryParams,
        resource_id: ContentId,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            params,
            resource_id,
        }
    }
    
    /// Create a new CosmWasm query effect with a specific ID
    pub fn with_id(
        id: EffectId,
        params: CosmWasmQueryParams,
        resource_id: ContentId,
    ) -> Self {
        Self {
            id,
            params,
            resource_id,
        }
    }
    
    /// Get the parameters for this query
    pub fn params(&self) -> &CosmWasmQueryParams {
        &self.params
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> Option<&str> {
        self.params.contract_address.as_deref()
    }
    
    /// Get the query target
    pub fn target(&self) -> &QueryTarget {
        &self.params.target
    }
    
    /// Get the query message
    pub fn query_msg(&self) -> &Value {
        &self.params.query_msg
    }
}

impl fmt::Debug for CosmWasmQueryEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CosmWasmQueryEffect")
            .field("id", &self.id)
            .field("chain_id", &self.params.chain_id)
            .field("contract_address", &self.params.contract_address)
            .field("target", &self.params.target)
            .field("query_msg", &self.params.query_msg)
            .finish()
    }
}

#[async_trait]
impl Effect for CosmWasmQueryEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new("cosmwasm.query")
    }
    
    fn display_name(&self) -> String {
        "CosmWasm Query".to_string()
    }
    
    fn description(&self) -> String {
        match &self.params.target {
            QueryTarget::SmartContract => {
                format!(
                    "Query contract {} on chain {}",
                    self.params.contract_address.as_deref().unwrap_or("unknown"),
                    self.params.chain_id
                )
            },
            QueryTarget::RawStorage => {
                format!(
                    "Query raw storage for contract {} on chain {}",
                    self.params.contract_address.as_deref().unwrap_or("unknown"),
                    self.params.chain_id
                )
            },
            QueryTarget::Balance => {
                format!(
                    "Query balance on chain {}",
                    self.params.chain_id
                )
            },
            QueryTarget::Staking => {
                format!(
                    "Query staking info on chain {}",
                    self.params.chain_id
                )
            },
            QueryTarget::ChainInfo => {
                format!(
                    "Query chain info for {}",
                    self.params.chain_id
                )
            },
        }
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.clone());
        
        if let Some(contract) = &self.params.contract_address {
            params.insert("contract_address".to_string(), contract.clone());
        }
        
        params.insert("target".to_string(), format!("{:?}", self.params.target));
        
        // Add query type if available
        if let Some(query_type) = self.params.query_msg.get("type").and_then(|v| v.as_str()) {
            params.insert("query_type".to_string(), query_type.to_string());
        }
        
        if let Some(height) = self.params.height {
            params.insert("height".to_string(), height.to_string());
        }
        
        params
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would call out to the CosmWasm chain
        // For now, we'll just return a success outcome with mock data
        
        // Check capabilities
        if !context.has_capability(&self.resource_id, &causality_core::capability::Right::Read) {
            return Err(EffectError::CapabilityError(
                format!("Missing read capability for resource: {}", self.resource_id)
            ));
        }
        
        // Create outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.clone());
        
        if let Some(contract) = &self.params.contract_address {
            outcome_data.insert("contract_address".to_string(), contract.clone());
        }
        
        outcome_data.insert("target".to_string(), format!("{:?}", self.params.target));
        
        // Add mock query result
        match self.params.target {
            QueryTarget::SmartContract => {
                outcome_data.insert("result".to_string(), r#"{"success":true,"data":{"value":"sample_data"}}"#.to_string());
            },
            QueryTarget::RawStorage => {
                outcome_data.insert("result".to_string(), r#"{"key":"sample_key","value":"sample_value"}"#.to_string());
            },
            QueryTarget::Balance => {
                outcome_data.insert("result".to_string(), r#"{"denom":"uatom","amount":"1000000"}"#.to_string());
            },
            QueryTarget::Staking => {
                outcome_data.insert("result".to_string(), r#"{"bonded_tokens":"500000","validators":3}"#.to_string());
            },
            QueryTarget::ChainInfo => {
                outcome_data.insert("result".to_string(), r#"{"chain_id":"cosmoshub-4","height":12345678}"#.to_string());
            },
        }
        
        // Return success outcome
        Ok(EffectOutcome::success(outcome_data)
            .with_affected_resource(self.resource_id.clone()))
    }
}

#[async_trait]
impl DomainEffect for CosmWasmQueryEffect {
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
impl ResourceEffect for CosmWasmQueryEffect {
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn operation(&self) -> ResourceOperation {
        ResourceOperation::Read
    }
}

#[async_trait]
impl CosmWasmEffect for CosmWasmQueryEffect {
    fn cosmwasm_effect_type(&self) -> CosmWasmEffectType {
        CosmWasmEffectType::Query
    }
    
    fn chain_id(&self) -> &str {
        &self.params.chain_id
    }
    
    fn is_read_only(&self) -> bool {
        true
    }
} 