//! TEL Effect Factory
//!
//! This module provides factory functions for creating TEL effects
//! and adapters for use with the causality-core effect system.

use std::sync::Arc;

use causality_core::effect::{
    Effect as CoreEffect,
};

use causality_tel::combinators::Combinator;
use super::TelEffectAdapter;
use super::TelEffectExecutor;

/// Create a TEL effect adapter for the given effect name and combinator
pub fn create_effect_adapter(
    name: impl Into<String>,
    combinator: Combinator,
    executor: Option<Arc<TelEffectExecutor>>
) -> TelEffectAdapter {
    let mut adapter = TelEffectAdapter::new(name, combinator);
    
    if let Some(exec) = executor {
        adapter = adapter.with_executor(exec);
    }
    
    adapter
}

/// Convert the adapter to a core effect trait object
pub fn adapter_to_core_effect(adapter: TelEffectAdapter) -> Box<dyn CoreEffect> {
    Box::new(adapter)
}

/// Provide an implementation of the create_effect_adapter for TEL
///
/// This function is intended to be used by the engine integration layer
/// to provide a concrete implementation of the create_effect_adapter 
/// function expected by the TEL crate.
pub fn register_tel_adapter_factory() {
    // This would typically set up a global factory or register with a registry
    // For now, this is a placeholder
}

/// Create a TEL effect and return its core effect implementation
pub fn create_tel_effect(
    name: impl Into<String>,
    parameters: serde_json::Value,
    return_type: causality_tel::types::TelType,
    executor: Option<Arc<TelEffectExecutor>>
) -> Box<dyn CoreEffect> {
    // Create the TEL effect
    let tel_effect = causality_tel::types::effect::TelEffect::new(
        name.into(),
        parameters,
        return_type,
    );
    
    // Create the combinator
    let combinator = tel_effect.combinator.clone();
    
    // Create the adapter
    let mut adapter = create_effect_adapter(tel_effect.name, combinator, executor);
    
    // Add metadata from the effect
    for (key, value) in tel_effect.metadata {
        adapter = adapter.with_metadata(key, value);
    }
    
    // Convert to a core effect
    adapter_to_core_effect(adapter)
} 