// CosmWasm TEL handler
// Original file: src/tel/handlers/cosmwasm.rs

//! CosmWasm TEL handlers
//!
//! This module provides TEL handlers for CosmWasm-compatible chains,
//! implementing effect creation for transfer, storage, and query operations.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use causality_types::Address;
use crate::resource::{ContentId, Quantity};
use crate::domain::{DomainId, DomainRegistry, DomainType};
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult,
    TransferEffect, QueryEffect, StorageEffect,
    random::{RandomEffectFactory, RandomType}
};
use causality_tel::{
    TelHandler, ConstraintTelHandler, TransferTelHandler,
    StorageTelHandler, QueryTelHandler, TransferParams
};
use causality_domain_cosmwasm::{CosmWasmAdapter, CosmWasmConfig};
use crate::crypto;

/// CosmWasm transfer effect implementation
pub struct CosmWasmTransferEffect {
    /// Source address
    source: Address,
    
    /// Destination address
    destination: Address,
    
    /// Amount to transfer
    amount: Quantity,
    
    /// Token/resource ID
    token: ContentId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// CosmWasm adapter configuration
    config: CosmWasmConfig,
    
    /// Gas limit for the transaction
    gas_limit: u64,
    
    /// Memo field
    memo: Option<String>,
}

impl CosmWasmTransferEffect {
    /// Create a new CosmWasm transfer effect
    pub fn new(
        source: Address,
        destination: Address,
        amount: Quantity,
        token: ContentId,
        domain_id: DomainId,
        config: CosmWasmConfig,
        gas_limit: u64,
        memo: Option<String>,
    ) -> Self {
        Self {
            source,
            destination,
            amount,
            token,
            domain_id,
            config,
            gas_limit,
            memo,
        }
    }
}

#[async_trait]
impl Effect for CosmWasmTransferEffect {
    fn name(&self) -> &str {
        "cosmwasm_transfer"
    }
    
    fn description(&self) -> &str {
        "Transfer assets on a CosmWasm-compatible chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, crate::resource::Right)> {
        vec![(self.token.clone(), crate::resource::Right::Transfer)]
    }
    
    async fn execute(&self, context: EffectContext) -> crate::effect::EffectResult<EffectOutcome> {
        // In a real implementation, this would connect to a CosmWasm chain and send a transaction
        // For now, we'll just create a simulated outcome
        
        // Create a simulated transaction hash using the RandomEffect
        let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
        let random_u64 = random_effect.gen_u64(&context)
            .await
            .unwrap_or(0);
        
        let tx_hash = format!("cosmos{:016x}", random_u64);
        
        // Create a successful outcome
        let outcome = EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::json!({
                "transaction_hash": tx_hash,
                "from": self.source.to_string(),
                "to": self.destination.to_string(),
                "amount": self.amount.to_string(),
                "token": self.token.to_string(),
                "gas_used": self.gas_limit / 2,
                "memo": self.memo,
            })),
            error: None,
            resource_changes: vec![],
            metadata: HashMap::new(),
        };
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: crate::effect::ExecutionBoundary) -> bool {
        boundary == crate::effect::ExecutionBoundary::OutsideSystem
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        crate::effect::ExecutionBoundary::OutsideSystem
    }
}

#[async_trait]
impl TransferEffect for CosmWasmTransferEffect {
    fn source(&self) -> &Address {
        &self.source
    }
    
    fn destination(&self) -> &Address {
        &self.destination
    }
    
    fn amount(&self) -> &Quantity {
        &self.amount
    }
    
    fn token(&self) -> &ContentId {
        &self.token
    }
    
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn validate(&self) -> Result<(), String> {
        // Perform CosmWasm-specific validation
        
        // Validate addresses (CosmWasm addresses should be bech32 format)
        let source_str = self.source.to_string();
        if !source_str.starts_with("cosmos") && !source_str.starts_with("osmo") && !source_str.starts_with("juno") {
            return Err("CosmWasm source address must be in bech32 format with a valid prefix".to_string());
        }
        
        let dest_str = self.destination.to_string();
        if !dest_str.starts_with("cosmos") && !dest_str.starts_with("osmo") && !dest_str.starts_with("juno") {
            return Err("CosmWasm destination address must be in bech32 format with a valid prefix".to_string());
        }
        
        // Validate amount is greater than zero
        if self.amount.is_zero() {
            return Err("Transfer amount cannot be zero".to_string());
        }
        
        // Validate gas parameters
        if self.gas_limit == 0 {
            return Err("Gas limit cannot be zero".to_string());
        }
        
        // Validate memo length
        if let Some(memo) = &self.memo {
            if memo.len() > 256 {
                return Err("Memo is too long (max 256 characters)".to_string());
            }
        }
        
        Ok(())
    }
}

/// CosmWasm transfer TEL handler implementation
#[derive(Debug)]
pub struct CosmWasmTransferHandler {
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Supported token types
    supported_tokens: Vec<String>,
}

impl CosmWasmTransferHandler {
    /// Create a new CosmWasm transfer handler
    pub fn new(domain_registry: Arc<DomainRegistry>) -> Self {
        Self {
            domain_registry,
            // Default supported tokens
            supported_tokens: vec![
                "ATOM".to_string(),
                "OSMO".to_string(),
                "JUNO".to_string(),
                "CW20".to_string(),
                "CW721".to_string(),
            ],
        }
    }
    
    /// Add a supported token type
    pub fn add_supported_token(&mut self, token_type: impl Into<String>) {
        self.supported_tokens.push(token_type.into());
    }
    
    /// Get CosmWasm configuration for a domain
    fn get_cosmwasm_config(&self, domain_id: &DomainId) -> Result<CosmWasmConfig, anyhow::Error> {
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)
            .ok_or_else(|| anyhow::anyhow!("Domain not found: {}", domain_id))?;
        
        // Ensure it's a CosmWasm domain
        if domain_info.domain_type != DomainType::CosmWasm {
            return Err(anyhow::anyhow!("Domain {} is not a CosmWasm domain", domain_id));
        }
        
        // Extract CosmWasm-specific configuration
        let grpc_url = domain_info.metadata.get("grpc_url")
            .ok_or_else(|| anyhow::anyhow!("gRPC URL not found for domain {}", domain_id))?
            .clone();
        
        let chain_id = domain_info.metadata.get("chain_id")
            .ok_or_else(|| anyhow::anyhow!("Chain ID not found for domain {}", domain_id))?
            .clone();
        
        // Create CosmWasm configuration
        let config = CosmWasmConfig {
            grpc_url,
            chain_id,
            // Other fields would be populated from domain_info.metadata
            gas_adjustment: 1.3, // Default gas adjustment
            denom: domain_info.metadata.get("denom")
                .cloned()
                .unwrap_or_else(|| "uatom".to_string()),
        };
        
        Ok(config)
    }
}

#[async_trait]
impl TelHandler for CosmWasmTransferHandler {
    fn effect_type(&self) -> &'static str {
        "transfer"
    }
    
    fn tel_function_name(&self) -> &'static str {
        "transfer"
    }
    
    fn domain_type(&self) -> &'static str {
        "cosmwasm"
    }
    
    async fn create_effect(&self, params: Value, context: &EffectContext) -> Result<Arc<dyn Effect>, anyhow::Error> {
        // Parse parameters into TransferParams
        let transfer_params: TransferParams = serde_json::from_value(params.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse transfer params: {}", e))?;
        
        // Create a constrained effect
        let effect = self.create_constrained_effect(params, context).await?;
        
        // Return as a general effect
        Ok(effect)
    }
}

#[async_trait]
impl ConstraintTelHandler<dyn TransferEffect> for CosmWasmTransferHandler {
    async fn create_constrained_effect(&self, params: Value, context: &EffectContext) -> Result<Arc<dyn TransferEffect>, anyhow::Error> {
        // Parse parameters into TransferParams
        let transfer_params: TransferParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Failed to parse transfer params: {}", e))?;
        
        // Get CosmWasm configuration for the domain
        let cosmwasm_config = self.get_cosmwasm_config(&transfer_params.domain_id)?;
        
        // Extract additional parameters specific to CosmWasm
        let gas_limit = transfer_params.additional.get("gas_limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(200000); // Default gas limit for CosmWasm transfers
        
        let memo = transfer_params.additional.get("memo")
            .and_then(|v| v.as_str())
            .map(String::from);
        
        // Create the effect
        let effect = CosmWasmTransferEffect::new(
            transfer_params.from,
            transfer_params.to,
            transfer_params.amount,
            transfer_params.token,
            transfer_params.domain_id,
            cosmwasm_config,
            gas_limit,
            memo,
        );
        
        // Validate the effect
        effect.validate()
            .map_err(|e| anyhow::anyhow!("Invalid CosmWasm transfer: {}", e))?;
        
        Ok(Arc::new(effect))
    }
}

#[async_trait]
impl TransferTelHandler for CosmWasmTransferHandler {
    fn supported_tokens(&self) -> Vec<String> {
        self.supported_tokens.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use serde_json::json;
    use crate::domain::{DomainRegistry, DomainInfo, DomainType};
    use crate::effect::EffectContext;
    use crate::tel::TransferParams;
    
    #[tokio::test]
    async fn test_cosmwasm_transfer_handler() {
        // Create a domain registry with a test domain
        let mut domain_registry = DomainRegistry::new();
        
        let domain_id = DomainId::new("cosmos:osmosis");
        let domain_info = DomainInfo {
            id: domain_id.clone(),
            name: "Osmosis".to_string(),
            domain_type: DomainType::CosmWasm,
            metadata: {
                let mut map = HashMap::new();
                map.insert("grpc_url".to_string(), "http://grpc.osmosis.zone:9090".to_string());
                map.insert("chain_id".to_string(), "osmosis-1".to_string());
                map.insert("denom".to_string(), "uosmo".to_string());
                map
            },
        };
        
        domain_registry.register_domain(domain_info);
        
        // Create a handler
        let handler = CosmWasmTransferHandler::new(Arc::new(domain_registry));
        
        // Create transfer parameters
        let params = json!({
            "from": "osmo1abcdefghijklmnopqrstuvwxyz12345678",
            "to": "osmo1zyxwvutsrqponmlkjihgfedcba87654321",
            "amount": 1000000, // 1 OSMO (1,000,000 uosmo)
            "token": "OSMO",
            "domain_id": "cosmos:osmosis",
            "gas_limit": 200000,
            "memo": "Transfer via TEL handler"
        });
        
        // Create context
        let context = EffectContext {
            execution_id: {
                // Generate a unique content ID for testing
                let test_data = "test-execution-context-cosmwasm-transfer";
                let hasher = crypto::hash::HashFactory::default().create_hasher().unwrap();
                let hash = hasher.hash(test_data.as_bytes());
                crypto::hash::ContentId::from(hash)
            },
            invoker: Address::new("test-user"),
            boundary: crate::effect::ExecutionBoundary::OutsideSystem,
            capabilities: Vec::new(),
            parameters: HashMap::new(),
        };
        
        // Create effect
        let effect = handler.create_effect(params, &context).await.unwrap();
        
        // Check effect type
        assert_eq!(effect.name(), "cosmwasm_transfer");
        
        // Execute effect
        let outcome = effect.execute(context).await.unwrap();
        
        // Check outcome
        assert!(outcome.success);
        assert!(outcome.result.is_some());
    }
} 
