//! Effect runtime implementation for the Causality Engine
//!
//! This module implements the EffectRuntime interface defined in the causality-effects crate.
//! It provides the concrete implementation for executing effects in the Causality system.

pub mod runtime;
pub mod registry;
pub mod capability;
pub mod executor;
pub mod content_addressable_executor;
pub mod resource;
pub mod factory;

/// Re-export all public items from submodules
pub use runtime::{
    EngineEffectRuntime, 
    EngineEffectRuntimeFactory,
    Runtime,
    RuntimeBase,
    get_effect_runtime,
    set_effect_runtime,
    create_runtime_factory,
};
pub use registry::EffectRegistry;
pub use capability::CapabilityManager;
pub use executor::EffectExecutor;
pub use content_addressable_executor::{
    ContentAddressableExecutor,
    SecuritySandbox,
    DefaultSecuritySandbox,
    ExecutionContext,
    CodeRepository,
    CodeEntry,
    ContentAddressed,
    ExecutionValue,
    ContextId,
    ExecutionEvent,
};
pub use resource::{
    ResourceEffectError,
    ResourceQueryEffect,
    ResourceStoreEffect,
    ResourceGetEffect,
    ResourceDeleteEffect,
};

// Re-export key types for public API
pub use factory::EmptyEffect;
pub use causality_core::effect::EffectType;

use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use causality_core::{ContentId, EffectOutcome as CoreEffectOutcome};
use causality_error::{EngineResult as Result, EngineError as Error};
use causality_types::DomainId;

/// An effect that can be applied to resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Unique identifier
    pub id: ContentId,
    /// Domain this effect applies to
    pub domain_id: Option<DomainId>,
    /// Type of effect
    pub effect_type: String,
    /// Parameters for the effect
    pub parameters: HashMap<String, serde_json::Value>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Effect {
    /// Create a new effect
    pub fn new(
        id: ContentId,
        effect_type: impl Into<String>,
        domain_id: Option<DomainId>,
    ) -> Self {
        Self {
            id,
            domain_id,
            effect_type: effect_type.into(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a parameter to this effect
    pub fn with_parameter(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self> {
        let key = key.into();
        let value = serde_json::to_value(value)
            .map_err(|e| Error::InvalidArgument(format!("Failed to serialize parameter '{}': {}", key, e)))?;
        
        self.parameters.insert(key, value);
        Ok(self)
    }
    
    /// Add metadata to this effect
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Convert from a core Effect
    pub fn from_core(effect: Box<dyn causality_core::Effect>) -> Self {
        Self {
            id: ContentId::new(effect.effect_type().to_string()),
            domain_id: None,  // We need to extract this from the effect if available
            effect_type: effect.effect_type().to_string(),
            parameters: HashMap::new(),  // We need to extract this from the effect if available
            metadata: HashMap::new(),    // We need to extract this from the effect if available
        }
    }
    
    /// Convert to a core Effect
    pub fn to_core(&self) -> Box<dyn causality_core::Effect> {
        // Create an implementation of the Effect trait
        struct CoreEffectImpl {
            id: causality_types::ContentId,
            domain_id: Option<causality_types::DomainId>,
            effect_type: String,
            parameters: std::collections::HashMap<String, serde_json::Value>,
            metadata: std::collections::HashMap<String, String>,
        }

        impl std::fmt::Debug for CoreEffectImpl {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("CoreEffectImpl")
                    .field("id", &self.id)
                    .field("domain_id", &self.domain_id)
                    .field("effect_type", &self.effect_type)
                    .finish()
            }
        }

        #[async_trait::async_trait]
        impl causality_core::effect::Effect for CoreEffectImpl {
            fn effect_type(&self) -> causality_core::effect::EffectType {
                causality_core::effect::EffectType::Custom(self.effect_type.clone())
            }
            
            fn description(&self) -> String {
                format!("Effect of type {} with ID {}", self.effect_type, self.id)
            }
            
            async fn execute(&self, _context: &dyn causality_core::effect::context::EffectContext) 
                -> causality_core::effect::outcome::EffectResult<causality_core::effect::outcome::EffectOutcome> {
                // This is just an adapter to the core trait and doesn't actually implement execution
                Err(causality_core::effect::EffectError::Other("Not implemented for adapter".to_string()))
            }
            
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        Box::new(CoreEffectImpl {
            id: self.id.clone(),
            domain_id: self.domain_id.clone(),
            effect_type: self.effect_type.clone(),
            parameters: self.parameters.clone(),
            metadata: self.metadata.clone(),
        })
    }
}

/// The outcome of applying an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOutcome {
    /// The effect that was applied
    pub effect_id: ContentId,
    /// Whether the effect was applied successfully
    pub success: bool,
    /// Result data from the effect application
    pub result: Option<serde_json::Value>,
    /// Error message, if any
    pub error: Option<String>,
    /// Resources affected by this effect
    pub affected_resources: Vec<ContentId>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl EffectOutcome {
    /// Create a successful outcome
    pub fn success(effect_id: ContentId) -> Self {
        Self {
            effect_id,
            success: true,
            result: None,
            error: None,
            affected_resources: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a successful outcome with a result
    pub fn with_result(effect_id: ContentId, result: impl Serialize) -> Result<Self> {
        let result = serde_json::to_value(result)
            .map_err(|e| Error::InvalidArgument(format!("Failed to serialize effect result: {}", e)))?;
        
        Ok(Self {
            effect_id,
            success: true,
            result: Some(result),
            error: None,
            affected_resources: Vec::new(),
            metadata: HashMap::new(),
        })
    }
    
    /// Create a failed outcome
    pub fn failure(effect_id: ContentId, error: impl Into<String>) -> Self {
        Self {
            effect_id,
            success: false,
            result: None,
            error: Some(error.into()),
            affected_resources: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add an affected resource
    pub fn with_affected_resource(mut self, resource_id: ContentId) -> Self {
        self.affected_resources.push(resource_id);
        self
    }
    
    /// Add multiple affected resources
    pub fn with_affected_resources(mut self, resource_ids: Vec<ContentId>) -> Self {
        self.affected_resources.extend(resource_ids);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Convert from a core EffectOutcome
    pub fn from_core(outcome: CoreEffectOutcome) -> Self {
        Self {
            effect_id: outcome.effect_id
                .map(|id| ContentId::new(id.to_string()))
                .unwrap_or_else(|| ContentId::new("random")),
            success: matches!(outcome.status, causality_core::effect::outcome::EffectStatus::Success),
            result: outcome.result.as_string().map(serde_json::Value::from).or_else(||
                outcome.data.get("result").map(|s| serde_json::Value::String(s.clone()))
            ),
            error: outcome.error_message,
            affected_resources: outcome.affected_resources,
            metadata: outcome.data,
        }
    }
    
    /// Convert to a core EffectOutcome
    pub fn to_core(&self) -> CoreEffectOutcome {
        use causality_core::effect::outcome::{EffectStatus, ResultData};
        use causality_core::effect::types::EffectId;
        
        let status = if self.success {
            EffectStatus::Success
        } else {
            EffectStatus::Failure
        };
        
        let result = if let Some(result_value) = &self.result {
            if let Ok(result_str) = serde_json::to_string(result_value) {
                ResultData::Json(result_str)
            } else {
                ResultData::None
            }
        } else {
            ResultData::None
        };
        
        CoreEffectOutcome {
            effect_id: Some(EffectId(self.effect_id.to_string())),
            status,
            data: self.metadata.clone(),
            result,
            error_message: self.error.clone(),
            affected_resources: self.affected_resources.clone(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_creation() {
        let id = ContentId::random();
        let effect = Effect::new(id.clone(), "test_effect", None);
        
        assert_eq!(effect.id, id);
        assert_eq!(effect.effect_type, "test_effect");
        assert!(effect.parameters.is_empty());
        assert!(effect.metadata.is_empty());
    }
    
    #[test]
    fn test_effect_parameters() {
        let id = ContentId::random();
        let effect = Effect::new(id.clone(), "test_effect", None)
            .with_parameter("key1", "value1").unwrap()
            .with_parameter("key2", 42).unwrap();
        
        assert_eq!(effect.parameters.len(), 2);
        assert_eq!(effect.parameters.get("key1").unwrap().as_str().unwrap(), "value1");
        assert_eq!(effect.parameters.get("key2").unwrap().as_i64().unwrap(), 42);
    }
    
    #[test]
    fn test_effect_outcome_success() {
        let id = ContentId::random();
        let outcome = EffectOutcome::success(id.clone());
        
        assert_eq!(outcome.effect_id, id);
        assert!(outcome.success);
        assert!(outcome.error.is_none());
    }
    
    #[test]
    fn test_effect_outcome_failure() {
        let id = ContentId::random();
        let outcome = EffectOutcome::failure(id.clone(), "Test failure");
        
        assert_eq!(outcome.effect_id, id);
        assert!(!outcome.success);
        assert_eq!(outcome.error.unwrap(), "Test failure");
    }
} 