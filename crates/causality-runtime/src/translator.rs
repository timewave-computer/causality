//! TEG to Runtime Effects Translator
//!
//! This module provides functionality to translate a Temporal Effect Graph (TEG)
//! into runtime Effect instances that can be executed by the runtime engine.

use std::sync::Arc;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, info, warn};

use causality_core::effect::{Effect, EffectId, EffectType, EffectContext, EffectOutcome, EffectError};
use causality_ir::graph::{TemporalEffectGraph, EffectNode, ResourceNode};
use causality_types::ContentId;

use crate::error::{RuntimeError, RuntimeResult};

/// Error types for the TEG translator
#[derive(Error, Debug)]
pub enum TranslatorError {
    #[error("Missing effect node: {0}")]
    MissingEffectNode(String),
    
    #[error("Missing resource node: {0}")]
    MissingResourceNode(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Effect creation error: {0}")]
    EffectCreationError(String),
    
    #[error("TEG validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<TranslatorError> for RuntimeError {
    fn from(err: TranslatorError) -> Self {
        RuntimeError::TranslatorError(err.to_string())
    }
}

/// A trait for translating a TEG into Effect instances
#[async_trait::async_trait]
pub trait TegTranslator: std::fmt::Debug + Send + Sync {
    /// Translate a TEG into a collection of Effect instances
    fn translate_teg(&self, teg: &TemporalEffectGraph) -> Result<HashMap<EffectId, Arc<dyn Effect>>, TranslatorError>;
    
    /// Translate a specific effect node from a TEG into an Effect instance
    fn translate_effect_node(&self, node: &EffectNode, teg: &TemporalEffectGraph) -> Result<Arc<dyn Effect>, TranslatorError>;
}

/// Basic implementation of the TegTranslator trait
#[derive(Debug)]
pub struct BasicTegTranslator {
    /// Registry of effect factories by type
    effect_factories: HashMap<String, Box<dyn EffectFactory>>,
}

impl BasicTegTranslator {
    /// Create a new instance of the translator
    pub fn new() -> Self {
        Self {
            effect_factories: HashMap::new(),
        }
    }
    
    /// Register an effect factory for a specific effect type
    pub fn register_factory(&mut self, effect_type: String, factory: Box<dyn EffectFactory>) {
        self.effect_factories.insert(effect_type, factory);
    }
    
    /// Find a factory for a specific effect type
    fn find_factory(&self, effect_type: &str) -> Option<&dyn EffectFactory> {
        self.effect_factories.get(effect_type).map(|f| f.as_ref())
    }
}

#[async_trait::async_trait]
impl TegTranslator for BasicTegTranslator {
    fn translate_teg(&self, teg: &TemporalEffectGraph) -> Result<HashMap<EffectId, Arc<dyn Effect>>, TranslatorError> {
        debug!("Translating TEG with {} effect nodes", teg.effect_nodes.len());
        
        // Validate the TEG first
        if let Err(e) = teg.validate() {
            return Err(TranslatorError::ValidationFailed(e));
        }
        
        let mut result = HashMap::new();
        
        // Translate each effect node
        for (effect_id, effect_node) in &teg.effect_nodes {
            match self.translate_effect_node(effect_node, teg) {
                Ok(effect) => {
                    result.insert(effect_id.clone(), effect);
                },
                Err(e) => {
                    error!(effect_id = ?effect_id, error = %e, "Failed to translate effect node");
                    return Err(e);
                }
            }
        }
        
        info!("Translated {} effect nodes from TEG", result.len());
        Ok(result)
    }
    
    fn translate_effect_node(&self, node: &EffectNode, teg: &TemporalEffectGraph) -> Result<Arc<dyn Effect>, TranslatorError> {
        debug!(effect_id = ?node.id, effect_type = ?node.effect_type, "Translating effect node");
        
        // Find the appropriate factory for this effect type
        let factory = self.find_factory(&node.effect_type)
            .ok_or_else(|| TranslatorError::EffectCreationError(
                format!("No factory registered for effect type: {}", node.effect_type)
            ))?;
        
        // Use the factory to create the effect
        let effect = factory.create_effect(node, teg)?;
        
        debug!(effect_id = ?node.id, "Effect node translated successfully");
        Ok(effect)
    }
}

impl Default for BasicTegTranslator {
    fn default() -> Self {
        Self::new()
    }
}

/// A trait for creating Effect instances from TEG nodes
pub trait EffectFactory: std::fmt::Debug + Send + Sync {
    /// Get the effect type this factory handles
    fn effect_type(&self) -> &str;
    
    /// Create an Effect instance from a TEG node
    fn create_effect(&self, node: &EffectNode, teg: &TemporalEffectGraph) -> Result<Arc<dyn Effect>, TranslatorError>;
}

/// A generic effect implementation for use with the translator
#[derive(Debug)]
pub struct GenericEffect {
    /// The effect ID
    id: EffectId,
    
    /// The effect type
    effect_type: EffectType,
    
    /// The effect parameters
    parameters: HashMap<String, serde_json::Value>,
    
    /// The effect resources (resource ID -> access mode)
    resources: HashMap<String, String>,
    
    /// The effect metadata
    metadata: HashMap<String, String>,
}

impl GenericEffect {
    /// Create a new generic effect
    pub fn new(
        id: EffectId,
        effect_type: EffectType,
        parameters: HashMap<String, serde_json::Value>,
        resources: HashMap<String, String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            effect_type,
            parameters,
            resources,
            metadata,
        }
    }
    
    /// Create a generic effect from a TEG node
    pub fn from_teg_node(node: &EffectNode) -> Result<Self, TranslatorError> {
        let id = node.id.clone();
        let effect_type = EffectType::new(&node.effect_type, &node.description);
        
        let parameters = node.parameters.clone();
        
        // Convert the JSON map to the expected format
        let mut resources = HashMap::new();
        for (res_id, access_mode) in &node.resources {
            resources.insert(res_id.clone(), access_mode.to_string());
        }
        
        let metadata = node.metadata.clone();
        
        Ok(Self {
            id,
            effect_type,
            parameters,
            resources,
            metadata,
        })
    }
}

#[async_trait::async_trait]
impl Effect for GenericEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn effect_type(&self) -> EffectType {
        self.effect_type.clone()
    }
    
    fn description(&self) -> String {
        self.effect_type.description().to_string()
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> Result<EffectOutcome, EffectError> {
        // This is a generic implementation that doesn't actually do anything
        error!(effect_id = ?self.id, "GenericEffect cannot be executed directly - handlers should be used instead");
        
        Err(EffectError::ExecutionError(
            "GenericEffect cannot be executed directly - handlers should be used instead".to_string()
        ))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A generic factory for creating GenericEffect instances
#[derive(Debug)]
pub struct GenericEffectFactory {
    effect_type: String,
}

impl GenericEffectFactory {
    /// Create a new generic effect factory
    pub fn new(effect_type: &str) -> Self {
        Self {
            effect_type: effect_type.to_string(),
        }
    }
}

impl EffectFactory for GenericEffectFactory {
    fn effect_type(&self) -> &str {
        &self.effect_type
    }
    
    fn create_effect(&self, node: &EffectNode, _teg: &TemporalEffectGraph) -> Result<Arc<dyn Effect>, TranslatorError> {
        // Check that the node matches the expected type
        if node.effect_type != self.effect_type {
            return Err(TranslatorError::EffectCreationError(
                format!("Node type {} does not match factory type {}", node.effect_type, self.effect_type)
            ));
        }
        
        // Create a generic effect from the node
        let effect = GenericEffect::from_teg_node(node)?;
        
        Ok(Arc::new(effect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_ir::graph::{EffectNode, ResourceNode, Edge, EdgeId};
    use std::collections::HashMap;
    
    // TODO: Add tests for the translator
    // This will require creating a test TEG
} 