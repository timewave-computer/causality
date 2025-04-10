//! TEL effect adapter
//!
//! This module provides an adapter between TEL effects and the causality-core
//! effect system. The adapter is responsible for translating TEL effect types
//! and parameters to and from core effect types.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectType as CoreEffectType,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
    EffectResult as CoreEffectResult,
};

use async_trait::async_trait;
use causality_tel::combinators::Combinator;
use causality_tel::types::effect::{TelEffect, EffectError};

use super::executor::TelEffectExecutor;

/// Adapter that bridges TEL effects with causality-core effects
#[derive(Debug)]
pub struct TelEffectAdapter {
    /// Name of the effect
    pub name: String,
    
    /// TEL combinator that represents the effect
    pub combinator: Combinator,
    
    /// Effect metadata
    pub metadata: HashMap<String, String>,
    
    /// Reference to the TEL effect executor
    executor: Option<Arc<TelEffectExecutor>>,
}

impl TelEffectAdapter {
    /// Create a new TEL effect adapter
    pub fn new(name: impl Into<String>, combinator: Combinator) -> Self {
        Self {
            name: name.into(),
            combinator,
            metadata: HashMap::new(),
            executor: None,
        }
    }
    
    /// Add metadata to the effect
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Set the effect executor
    pub fn with_executor(mut self, executor: Arc<TelEffectExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }
}

#[async_trait]
impl CoreEffect for TelEffectAdapter {
    fn effect_type(&self) -> CoreEffectType {
        // Determine the appropriate effect type based on the effect name
        match self.name.as_str() {
            name if name.starts_with("read_") => CoreEffectType::Read,
            name if name.starts_with("write_") => CoreEffectType::Write,
            name if name.starts_with("create_") => CoreEffectType::Create,
            name if name.starts_with("delete_") => CoreEffectType::Delete,
            _ => CoreEffectType::Custom(self.name.clone()),
        }
    }
    
    fn description(&self) -> String {
        format!("TEL effect: {}", self.name)
    }
    
    async fn execute(&self, context: &dyn CoreEffectContext) -> CoreEffectResult<CoreEffectOutcome> {
        // If we have an executor, use it
        if let Some(executor) = &self.executor {
            // Delegate to the executor for actual execution
            return executor.execute_combinator(&self.combinator, context)
                .await
                .map_err(|e| e.into());
        }
        
        // If we don't have an executor, return a dummy outcome with metadata
        // This is suboptimal and should prompt proper configuration
        let mut outcome_data = HashMap::new();
        outcome_data.insert("error".to_string(), "No executor configured for TelEffectAdapter".to_string());
        outcome_data.insert("effect".to_string(), self.name.clone());
        
        Ok(CoreEffectOutcome::error(
            "execution_error".to_string(),
            "No executor configured for TelEffectAdapter".to_string(),
            outcome_data
        ))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Convert from TelEffectAdapter to a CoreEffect trait object
pub fn adapter_to_core_effect(adapter: TelEffectAdapter) -> Box<dyn CoreEffect> {
    Box::new(adapter)
} 