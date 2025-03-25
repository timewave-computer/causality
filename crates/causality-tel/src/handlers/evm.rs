// EVM TEL handler
// Original file: src/tel/handlers/evm.rs

//! EVM TEL handlers
//!
//! This module provides TEL handlers for Ethereum Virtual Machine (EVM) compatible chains,
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
use causality_domain_evm::{EvmAdapter, EvmConfig};
use crate::crypto;

/// EVM transfer effect implementation
pub struct EvmTransferEffect {
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
    
    /// EVM adapter configuration
    config: EvmConfig,
    
    /// Gas limit for the transaction
    gas_limit: u64,
    
    /// Gas price for the transaction
    gas_price: u64,
}

impl EvmTransferEffect {
    /// Create a new EVM transfer effect
    pub fn new(
        source: Address,
        destination: Address,
        amount: Quantity,
        token: ContentId,
        domain_id: DomainId,
        config: EvmConfig,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            source,
            destination,
            amount,
            token,
            domain_id,
            config,
            gas_limit,
            gas_price,
        }
    }
}

#[async_trait]
impl Effect for EvmTransferEffect {
    fn name(&self) -> &str {
        "evm_transfer"
    }
    
    fn description(&self) -> &str {
        "Transfer assets on an EVM-compatible chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, crate::resource::Right)> {
        vec![(self.token.clone(), crate::resource::Right::Transfer)]
    }
    
    async fn execute(&self, context: EffectContext) -> crate::effect::EffectResult<EffectOutcome> {
        // In a real implementation, this would connect to an EVM node and send a transaction
        // For now, we'll just create a simulated outcome
        
        // Create a simulated transaction hash using the RandomEffect
        let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
        let random_u64 = random_effect.gen_u64(&context)
            .await
            .unwrap_or(0);
        
        let tx_hash = format!("0x{:064x}", random_u64);
        
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
                "gas_price": self.gas_price,
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
impl TransferEffect for EvmTransferEffect {
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
    
    fn fee(&self) -> Option<Quantity> {
        // Calculate fee based on gas parameters
        Some(Quantity::from(self.gas_limit as u128 * self.gas_price as u128))
    }
    
    fn validate(&self) -> Result<(), String> {
        // Perform EVM-specific validation
        if !self.source.to_string().starts_with("0x") {
            return Err("EVM source address must start with 0x".to_string());
        }
        
        if !self.destination.to_string().starts_with("0x") {
            return Err("EVM destination address must start with 0x".to_string());
        }
        
        // Validate amount is greater than zero
        if self.amount.is_zero() {
            return Err("Transfer amount cannot be zero".to_string());
        }
        
        // Validate gas parameters
        if self.gas_limit == 0 {
            return Err("Gas limit cannot be zero".to_string());
        }
        
        if self.gas_price == 0 {
            return Err("Gas price cannot be zero".to_string());
        }
        
        Ok(())
    }
}

/// EVM transfer TEL handler implementation
#[derive(Debug)]
pub struct EvmTransferHandler {
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Supported token types
    supported_tokens: Vec<String>,
}

impl EvmTransferHandler {
    /// Create a new EVM transfer handler
    pub fn new(domain_registry: Arc<DomainRegistry>) -> Self {
        Self {
            domain_registry,
            // Default supported tokens
            supported_tokens: vec![
                "ETH".to_string(),
                "ERC20".to_string(),
                "ERC721".to_string(),
            ],
        }
    }
    
    /// Add a supported token type
    pub fn add_supported_token(&mut self, token_type: impl Into<String>) {
        self.supported_tokens.push(token_type.into());
    }
    
    /// Get EVM configuration for a domain
    fn get_evm_config(&self, domain_id: &DomainId) -> Result<EvmConfig, anyhow::Error> {
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)
            .ok_or_else(|| anyhow::anyhow!("Domain not found: {}", domain_id))?;
        
        // Ensure it's an EVM domain
        if domain_info.domain_type != DomainType::EVM {
            return Err(anyhow::anyhow!("Domain {} is not an EVM domain", domain_id));
        }
        
        // Extract EVM-specific configuration
        let rpc_url = domain_info.metadata.get("rpc_url")
            .ok_or_else(|| anyhow::anyhow!("RPC URL not found for domain {}", domain_id))?
            .clone();
        
        let chain_id = domain_info.metadata.get("chain_id")
            .and_then(|id| id.parse::<u64>().ok())
            .unwrap_or(1); // Default to Ethereum mainnet
        
        // Create EVM configuration
        let config = EvmConfig {
            rpc_url,
            chain_id,
            // Other fields would be populated from domain_info.metadata
            private_key: None, // Would be provided in the effect context
            timeout: None,     // Optional timeout
        };
        
        Ok(config)
    }
}

#[async_trait]
impl TelHandler for EvmTransferHandler {
    fn effect_type(&self) -> &'static str {
        "transfer"
    }
    
    fn tel_function_name(&self) -> &'static str {
        "transfer"
    }
    
    fn domain_type(&self) -> &'static str {
        "evm"
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
impl ConstraintTelHandler<dyn TransferEffect> for EvmTransferHandler {
    async fn create_constrained_effect(&self, params: Value, context: &EffectContext) -> Result<Arc<dyn TransferEffect>, anyhow::Error> {
        // Parse parameters into TransferParams
        let transfer_params: TransferParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Failed to parse transfer params: {}", e))?;
        
        // Get EVM configuration for the domain
        let evm_config = self.get_evm_config(&transfer_params.domain_id)?;
        
        // Extract additional parameters specific to EVM
        let gas_limit = transfer_params.additional.get("gas_limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(21000); // Default gas limit for ETH transfers
        
        let gas_price = transfer_params.additional.get("gas_price")
            .and_then(|v| v.as_u64())
            .unwrap_or(20_000_000_000); // 20 gwei default
        
        // Create the effect
        let effect = EvmTransferEffect::new(
            transfer_params.from,
            transfer_params.to,
            transfer_params.amount,
            transfer_params.token,
            transfer_params.domain_id,
            evm_config,
            gas_limit,
            gas_price,
        );
        
        // Validate the effect
        effect.validate()
            .map_err(|e| anyhow::anyhow!("Invalid EVM transfer: {}", e))?;
        
        Ok(Arc::new(effect))
    }
}

#[async_trait]
impl TransferTelHandler for EvmTransferHandler {
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
    async fn test_evm_transfer_handler() {
        // Create a domain registry with a test domain
        let mut domain_registry = DomainRegistry::new();
        
        let domain_id = DomainId::new("ethereum:mainnet");
        let domain_info = DomainInfo {
            id: domain_id.clone(),
            name: "Ethereum Mainnet".to_string(),
            domain_type: DomainType::EVM,
            metadata: {
                let mut map = HashMap::new();
                map.insert("rpc_url".to_string(), "https://mainnet.infura.io/v3/test".to_string());
                map.insert("chain_id".to_string(), "1".to_string());
                map
            },
        };
        
        domain_registry.register_domain(domain_info);
        
        // Create a handler
        let handler = EvmTransferHandler::new(Arc::new(domain_registry));
        
        // Create transfer parameters
        let params = json!({
            "from": "0x1234567890123456789012345678901234567890",
            "to": "0x0987654321098765432109876543210987654321",
            "amount": 1000000000000000000, // 1 ETH
            "token": "ETH",
            "domain_id": "ethereum:mainnet",
            "gas_limit": 21000,
            "gas_price": 20000000000
        });
        
        // Create context
        let context = EffectContext {
            execution_id: {
                // Generate a unique content ID for testing
                let test_data = "test-execution-context-evm-transfer";
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
        assert_eq!(effect.name(), "evm_transfer");
        
        // Execute effect
        let outcome = effect.execute(context).await.unwrap();
        
        // Check outcome
        assert!(outcome.success);
        assert!(outcome.result.is_some());
    }
} 
