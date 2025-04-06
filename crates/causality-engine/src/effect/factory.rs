// Effect factory module for Causality Engine
//
// This module provides factory functions for creating standard effects
// like transfer, deposit, and withdrawal operations.

use std::fmt::Debug;
use async_trait::async_trait;
use causality_error::Result;
use causality_core::effect::{Effect, EffectType, EffectOutcome, EffectResult};
use std::collections::HashMap;

/// Create a transfer effect
pub fn create_transfer_effect(
    resource_id: String,
    from: String,
    to: String,
) -> Result<Box<dyn Effect>> {
    // Using empty effect until we implement proper transfer
    Ok(Box::new(EmptyEffect::new(&format!("transfer_effect:{}:{}:{}", resource_id, from, to))))
}

/// Create a deposit effect
pub fn create_deposit_effect(
    resource_id: String,
    owner: String,
    _initial_state: String,
) -> Result<Box<dyn Effect>> {
    // Using empty effect until we implement proper deposit
    Ok(Box::new(EmptyEffect::new(&format!("deposit_effect:{}:{}", resource_id, owner))))
}

/// Create a withdrawal effect
pub fn create_withdrawal_effect(
    resource_id: String,
    owner: String,
) -> Result<Box<dyn Effect>> {
    // Using empty effect until we implement proper withdrawal
    Ok(Box::new(EmptyEffect::new(&format!("withdrawal_effect:{}:{}", resource_id, owner))))
}

/// A simple empty effect for testing and placeholder implementations
#[derive(Debug, Clone)]
pub struct EmptyEffect {
    name: String,
}

impl EmptyEffect {
    /// Create a new empty effect with the given name
    pub fn new(name: &str) -> Self {
        EmptyEffect {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Effect for EmptyEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(self.name.clone())
    }
    
    fn description(&self) -> String {
        format!("Empty effect: {}", self.name)
    }
    
    async fn execute(&self, _context: &dyn causality_core::effect::EffectContext) -> EffectResult<EffectOutcome> {
        // Simplified implementation for now - create a workflow from state
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 