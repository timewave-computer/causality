//! Effect execution engine
//!
//! This module provides the core functionality for executing effects
//! through their registered handlers.


use std::fmt;
use std::sync::Arc;

use causality_core::effect::runtime::context::Context;
use causality_core::effect::runtime::error::{EffectError, EffectResult};
use causality_core::effect::runtime::types::Effect;
use causality_core::effect::runtime::types::id::EffectTypeId;

use super::registry::EffectRegistry;
use super::capability::CapabilityManager;

/// Engine for executing effects
///
/// This component is responsible for executing effects with their
/// registered handlers, ensuring that capabilities are verified
/// and errors are properly handled.
pub struct EffectExecutor {
    /// The registry of effect handlers
    registry: Arc<EffectRegistry>,
    
    /// The capability manager for verifying capabilities
    capability_manager: Arc<CapabilityManager>,
}

impl EffectExecutor {
    /// Create a new effect executor
    pub fn new(
        registry: Arc<EffectRegistry>,
        capability_manager: Arc<CapabilityManager>,
    ) -> Self {
        Self {
            registry,
            capability_manager,
        }
    }
    
    /// Execute an effect with the given parameter and context
    pub async fn execute<E: Effect>(
        &self,
        effect: &E,
        param: E::Param,
        context: &Context,
    ) -> EffectResult<E::Outcome> {
        // Step 1: Check if the effect requires capability verification
        if effect_requires_capability_check(effect) {
            // Verify capabilities
            self.capability_manager.verify_capabilities(effect, context).await?;
        }
        
        // Step 2: Find the handler for this effect type
        let handler = self.registry.get_handler(&effect.type_id())?;
        
        // Step 3: Handle the effect
        let boxed_param = Box::new(param);
        let result = handler.handle(&effect.type_id(), boxed_param, context).await
            .map_err(|err| EffectError::HandlerError {
                effect_type: effect.type_id(),
                handler_type: format!("{:?}", handler),
                message: format!("{}", err),
            })?;
        
        // Step 4: Downcast the result to the expected outcome type
        let outcome = result.downcast::<E::Outcome>()
            .map_err(|_| EffectError::TypeMismatch {
                effect_type: effect.type_id(),
                expected: std::any::type_name::<E::Outcome>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        Ok(*outcome)
    }
    
    /// Get a reference to the registry
    pub fn registry(&self) -> &EffectRegistry {
        &self.registry
    }
    
    /// Get a reference to the capability manager
    pub fn capability_manager(&self) -> &CapabilityManager {
        &self.capability_manager
    }
}

impl fmt::Debug for EffectExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectExecutor")
            .field("registry", &self.registry)
            .field("capability_manager", &self.capability_manager)
            .finish()
    }
}

/// Determine if an effect requires capability verification
///
/// This function is a placeholder for a more sophisticated system
/// that would determine if an effect requires capability verification.
fn effect_requires_capability_check<E: Effect>(_effect: &E) -> bool {
    // For now, we assume all effects require capability verification
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::any::Any;
    use causality_core::effect::runtime::context::Context;
    use causality_core::effect::runtime::core::handler::EffectHandler;
    use causality_core::effect::runtime::types::id::EffectTypeId;
    use async_trait::async_trait;
    // A simple test effect
    #[derive(Debug)]
    struct TestEffect;
    
    #[derive(Debug)]
    struct TestParam {
        value: String,
    }
    
    #[derive(Debug)]
    struct TestOutcome {
        result: String,
    }
    
    #[async_trait]
    impl Effect for TestEffect {
        type Param = TestParam;
        type Outcome = TestOutcome;
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new("test.effect")
        }
        
        async fn execute(
            &self,
            param: Self::Param,
            _context: &Context,
        ) -> Result<Self::Outcome, EffectError> {
            Ok(TestOutcome {
                result: format!("Processed: {}", param.value),
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    // A test handler
    #[derive(Debug)]
    struct TestHandler;
    
    #[async_trait]
    impl EffectHandler for TestHandler {
        async fn can_handle(&self, effect_type: &EffectTypeId) -> bool {
            effect_type.to_string() == "test.effect"
        }
        
        async fn handle(
            &self,
            _effect_type: &EffectTypeId,
            param: Box<dyn Any + Send>,
            _context: &Context,
        ) -> Result<Box<dyn Any + Send>, EffectError> {
            // Downcast the parameter to the expected type
            let param = param.downcast::<TestParam>()
                .map_err(|_| EffectError::internal_error("Invalid parameter type"))?;
            
            // Process the parameter
            let outcome = TestOutcome {
                result: format!("Handled: {}", param.value),
            };
            
            // Box the outcome
            Ok(Box::new(outcome))
        }
    }
    
    #[tokio::test]
    async fn test_effect_execution() {
        // Set up the test environment
        let registry = Arc::new(EffectRegistry::new());
        let capability_manager = Arc::new(CapabilityManager::new());
        let executor = EffectExecutor::new(registry.clone(), capability_manager);
        
        // Register a handler
        let effect_type = EffectTypeId::new("test.effect");
        let handler = Arc::new(TestHandler);
        registry.register(effect_type.clone(), handler);
        
        // Create an effect and parameter
        let effect = TestEffect;
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::new();
        
        // Execute the effect
        let outcome = executor.execute(&effect, param, &context).await.unwrap();
        
        // Verify the outcome
        assert_eq!(outcome.result, "Handled: hello");
    }
} 