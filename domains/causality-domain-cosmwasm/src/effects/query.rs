// CosmWasm Query Effect
//
// This module provides the implementation of the CosmWasm query effect,
// which allows querying contracts and state on CosmWasm-based chains.

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

use causality_types::content::ContentId as CausalityContentId;
use causality_core::effect::domain::DomainEffect as CausalityDomainEffect;

use super::{CosmWasmEffect, CosmWasmEffectType, CosmWasmGasParams, CosmWasmEffectHandler, COSMWASM_DOMAIN_ID};

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
    /// Contract query
    Contract { contract_address: String },
    /// System query
    System,
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

#[async_trait::async_trait]
impl Effect for CosmWasmQueryEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("cosmwasm.query".to_string())
    }
    
    fn description(&self) -> String {
        match &self.params.target {
            QueryTarget::Contract { contract_address } => {
                format!("CosmWasm Query: Contract {} on {}", 
                        contract_address, 
                        self.params.chain_id)
            }
            QueryTarget::System => {
                format!("CosmWasm Query: System on {}", 
                        self.params.chain_id)
            }
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
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult {
        // Verify capability for resource access
        if !context.has_capability(&format!("cosmwasm.query.{}", self.params.chain_id)) {
            return Err(EffectError::PermissionDenied(
                format!("Missing capability to query chain: {}", self.params.chain_id)
            ));
        }
        
        // For contract queries, verify additional permission
        if let QueryTarget::Contract { contract_address } = &self.params.target {
            if !context.has_capability(&format!("cosmwasm.contract.{}", contract_address)) {
                return Err(EffectError::PermissionDenied(
                    format!("Missing capability to query contract: {}", contract_address)
                ));
            }
        }
        
        // Prepare outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.clone());
        
        // Add target information
        match &self.params.target {
            QueryTarget::Contract { contract_address } => {
                outcome_data.insert("target_type".to_string(), "contract".to_string());
                outcome_data.insert("contract_address".to_string(), contract_address.clone());
            }
            QueryTarget::System => {
                outcome_data.insert("target_type".to_string(), "system".to_string());
            }
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
        
        outcome_data.insert("query_msg".to_string(), self.params.query_msg.to_string());
        
        // In a real implementation, we would execute the query here
        // For now, we'll just return a simulated result
        
        Ok(EffectOutcome::Success {
            result: Some("query_executed".to_string()),
            data: outcome_data,
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
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
    
    fn gas_params(&self) -> Option<CosmWasmGasParams> {
        None // Queries don't use gas
    }
    
    async fn handle_with_handler(&self, handler: &dyn CosmWasmEffectHandler, context: &dyn EffectContext) -> EffectResult {
        handler.handle_cosmwasm_effect(self, context).await
    }
} 