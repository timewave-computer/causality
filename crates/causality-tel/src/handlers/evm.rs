// EVM-specific TEL handlers
// This is a stub implementation

//! EVM-specific TEL handlers
//!
//! This module implements TEL handlers for the Ethereum Virtual Machine domain.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use anyhow::{anyhow, Result};


use crate::handlers::{
    BaseTelHandler, ConstraintTelHandler, TelHandler, 
    TransferTelHandler, EffectContext, EffectOutcome, Effect,
    TransferEffect, EffectResult, Quantity, EffectStatus
};

/// EVM transfer handler
#[derive(Debug)]
pub struct EvmTransferHandler {
    base: BaseTelHandler<dyn TransferEffect>,
}

impl EvmTransferHandler {
    /// Create a new EVM transfer handler
    pub fn new() -> Self {
        Self {
            base: BaseTelHandler::new(
                "transfer",
                "transfer",
                "evm",
            ),
        }
    }
}

/// Simple EVM transfer effect
#[derive(Debug)]
pub struct EvmTransferEffect {
    pub from: String,
    pub to: String,
    #[allow(dead_code)]
    token: String,
    pub amount: crate::resource::Quantity,
}

impl Effect for EvmTransferEffect {
    fn effect_type(&self) -> &'static str {
        "transfer"
    }

    fn apply(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Stub implementation
        Ok(EffectOutcome {
            effect_type: "transfer".to_string(),
            status: EffectStatus::Success,
            output: None,
            error: None,
        })
    }
}

impl TransferEffect for EvmTransferEffect {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn amount(&self) -> &dyn Quantity {
        &self.amount
    }
}

#[async_trait]
impl TelHandler for EvmTransferHandler {
    fn effect_type(&self) -> &'static str {
        self.base.effect_type()
    }

    fn tel_function_name(&self) -> &'static str {
        self.base.tel_function_name()
    }

    fn domain_type(&self) -> &'static str {
        self.base.domain_type()
    }

    async fn create_effect(&self, params: JsonValue, context: &EffectContext) -> Result<Arc<dyn Effect>, anyhow::Error> {
        let transfer_effect = self.create_constrained_effect(params, context).await?;
        // Manual upcast from TransferEffect to Effect using a new Arc
        let effect: Arc<dyn Effect> = Arc::new(EvmTransferEffectWrapper(transfer_effect));
        Ok(effect)
    }
}

/// Wrapper to allow upcasting from TransferEffect to Effect
#[derive(Debug)]
struct EvmTransferEffectWrapper(Arc<dyn TransferEffect>);

impl Effect for EvmTransferEffectWrapper {
    fn effect_type(&self) -> &'static str {
        self.0.effect_type()
    }

    fn apply(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        self.0.apply(context)
    }
}

#[async_trait]
impl ConstraintTelHandler<dyn TransferEffect> for EvmTransferHandler {
    async fn create_constrained_effect(&self, params: JsonValue, _context: &EffectContext) -> Result<Arc<dyn TransferEffect>, anyhow::Error> {
        // Parse parameters from JSON
        let from = params["from"].as_str()
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?
            .to_string();
            
        let to = params["to"].as_str()
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?
            .to_string();
            
        let token = params["token"].as_str()
            .or_else(|| params["asset"].as_str())
            .ok_or_else(|| anyhow!("Missing 'token' or 'asset' parameter"))?
            .to_string();
            
        let amount_str = params.get("amount").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let amount = crate::resource::Quantity::new(&amount_str);
        
        // Create the effect
        let effect = EvmTransferEffect {
            from,
            to,
            token,
            amount: amount,
        };
        
        Ok(Arc::new(effect))
    }
}

impl TransferTelHandler for EvmTransferHandler {
    fn supported_tokens(&self) -> Vec<String> {
        vec![
            "ETH".to_string(),
            "USDC".to_string(),
            "USDT".to_string(),
            "DAI".to_string(),
        ]
    }
} 
