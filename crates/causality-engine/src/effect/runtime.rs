//! Effect runtime implementation
//!
//! This module provides the implementation of the EffectRuntime
//! interface defined in the causality-effects crate.

use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;

// Import core types with clear, consistent names
use causality_core::effect::runtime::context::Context;
use causality_core::effect::runtime::error::EffectResult;
use causality_core::effect::runtime::core::handler::EffectHandler;
use causality_core::effect::runtime::types::{Effect, EffectTypeId};
// Important: we don't import any traits from causality_core::effect::runtime::runtime 
// because they cause object safety issues

use super::registry::EffectRegistry;
use super::capability::CapabilityManager;
use super::executor::EffectExecutor;

// Define our own object-safe trait versions
#[async_trait]
pub trait RuntimeBase: Send + Sync + 'static {
    fn register_handler(
        &mut self,
        effect_type: EffectTypeId,
        handler: Arc<dyn EffectHandler>,
    );
    
    fn has_handler(&self, effect_type: &EffectTypeId) -> bool;
    
    fn registered_effect_types(&self) -> Vec<EffectTypeId>;
}

#[async_trait]
pub trait Runtime: RuntimeBase {
    // Non-generic method that can be used in trait objects
    async fn execute_any(
        &self,
        effect: &dyn Effect<Param = Box<dyn std::any::Any + Send + Sync>, Outcome = Box<dyn std::any::Any + Send + Sync>>,
        param: Box<dyn std::any::Any + Send + Sync>,
        context: &dyn Context,
    ) -> EffectResult<Box<dyn std::any::Any + Send + Sync>>;
}

/// Global effect runtime
static EFFECT_RUNTIME: RwLock<Option<Arc<EngineEffectRuntime>>> = RwLock::new(None);

/// Get the global effect runtime 
/// Uses a concrete type instead of dyn trait to avoid object safety issues
pub fn get_effect_runtime() -> Arc<EngineEffectRuntime> {
    let runtime = EFFECT_RUNTIME.read().unwrap();
    match &*runtime {
        Some(runtime) => runtime.clone(),
        None => {
            // Create a new runtime on demand
            drop(runtime); // Release the read lock
            
            let factory = EngineEffectRuntimeFactory::new();
            let runtime = factory.create_runtime();
            
            // Store in global
            let mut global_runtime = EFFECT_RUNTIME.write().unwrap();
            *global_runtime = Some(runtime.clone());
            
            runtime
        }
    }
}

/// Set the global effect runtime
pub fn set_effect_runtime(runtime: Arc<EngineEffectRuntime>) {
    let mut global_runtime = EFFECT_RUNTIME.write().unwrap();
    *global_runtime = Some(runtime);
}

/// Create a factory for tests and applications
pub fn create_runtime_factory() -> EngineEffectRuntimeFactory {
    EngineEffectRuntimeFactory::new()
}

/// The concrete implementation of our runtime
pub struct EngineEffectRuntime {
    /// The registry of effect handlers
    registry: Arc<EffectRegistry>,
    
    /// The capability manager for verifying capabilities
    capability_verifier: Arc<CapabilityManager>,
    
    /// The executor for executing effects
    executor: Arc<EffectExecutor>,
}

impl EngineEffectRuntime {
    /// Create a new effect runtime
    pub fn new() -> Self {
        let registry = Arc::new(EffectRegistry::new());
        let capability_verifier = Arc::new(CapabilityManager::new());
        let executor = Arc::new(EffectExecutor::new(
            registry.clone(),
            capability_verifier.clone(),
        ));
        
        Self {
            registry,
            capability_verifier,
            executor,
        }
    }
    
    /// Type-safe wrapper for execute_any
    pub async fn execute<E: Effect>(
        &self,
        effect: &E,
        param: E::Param,
        context: &dyn Context,
    ) -> EffectResult<E::Outcome> {
        self.executor.execute(effect, param, context).await
    }
}

impl Default for EngineEffectRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EngineEffectRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineEffectRuntime")
            .field("registry", &self.registry)
            .field("capability_verifier", &self.capability_verifier)
            .finish()
    }
}

// Use only our local trait definitions
impl RuntimeBase for EngineEffectRuntime {
    fn register_handler(
        &mut self,
        effect_type: EffectTypeId,
        handler: Arc<dyn EffectHandler>,
    ) {
        self.registry.register(effect_type, handler);
    }
    
    fn has_handler(&self, effect_type: &EffectTypeId) -> bool {
        self.registry.has_handler(effect_type)
    }
    
    fn registered_effect_types(&self) -> Vec<EffectTypeId> {
        self.registry.registered_effect_types()
    }
}

// Implement our local Runtime trait
#[async_trait]
impl Runtime for EngineEffectRuntime {
    async fn execute_any(
        &self,
        effect: &dyn Effect<Param = Box<dyn std::any::Any + Send + Sync>, Outcome = Box<dyn std::any::Any + Send + Sync>>,
        param: Box<dyn std::any::Any + Send + Sync>,
        context: &dyn Context,
    ) -> EffectResult<Box<dyn std::any::Any + Send + Sync>> {
        self.executor.execute_any(effect, param, context).await
    }
}

/// Factory for creating effect runtimes
pub struct EngineEffectRuntimeFactory {
    /// Options for creating runtimes
    options: Mutex<EffectRuntimeOptions>,
}

/// Options for creating effect runtimes
#[derive(Debug, Default)]
pub struct EffectRuntimeOptions {
    /// Whether to register the runtime as the global runtime
    register_as_global: bool,
}

impl EngineEffectRuntimeFactory {
    /// Create a new effect runtime factory
    pub fn new() -> Self {
        Self {
            options: Mutex::new(EffectRuntimeOptions::default()),
        }
    }
    
    /// Set whether to register the runtime as the global runtime
    pub fn set_register_as_global(&self, register_as_global: bool) -> &Self {
        let mut options = self.options.lock().unwrap();
        options.register_as_global = register_as_global;
        self
    }
    
    /// Create a new runtime
    pub fn create_runtime(&self) -> Arc<EngineEffectRuntime> {
        let runtime = Arc::new(EngineEffectRuntime::new());
        
        // Register as global if requested
        let options = self.options.lock().unwrap();
        if options.register_as_global {
            set_effect_runtime(runtime.clone());
        }
        
        runtime
    }
}

impl Default for EngineEffectRuntimeFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EngineEffectRuntimeFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let options = match self.options.lock() {
            Ok(options) => format!("{:?}", options),
            Err(_) => "<locked>".to_string(),
        };
        
        f.debug_struct("EngineEffectRuntimeFactory")
            .field("options", &options)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use causality_core::effect::runtime::context::Context;
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
            EffectTypeId::new("test", "effect")
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
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    // A test handler
    #[derive(Debug)]
    struct TestHandler;
    
    #[async_trait]
    impl EffectHandler for TestHandler {
        async fn can_handle(&self, effect_type: &EffectTypeId) -> bool {
            effect_type.to_string() == "test:effect"
        }
        
        async fn handle(
            &self,
            _effect_type: &EffectTypeId,
            param: Box<dyn std::any::Any + Send>,
            _context: &Context,
        ) -> Result<Box<dyn std::any::Any + Send>, EffectError> {
            // Downcast the parameter to the expected type
            let param = param.downcast::<TestParam>()
                .map_err(|_| EffectError::ExecutionError("Invalid parameter type".to_string()))?;
            
            // Process the parameter
            let outcome = TestOutcome {
                result: format!("Handled: {}", param.value),
            };
            
            // Box the outcome
            Ok(Box::new(outcome))
        }
    }
    
    #[tokio::test]
    async fn test_effect_runtime() {
        // Create a runtime
        let mut runtime = EngineEffectRuntime::new();
        
        // Register a handler
        let effect_type = EffectTypeId::new("test", "effect");
        let handler = Arc::new(TestHandler);
        runtime.register_handler(effect_type.clone(), handler);
        
        // Verify registration
        assert!(runtime.has_handler(&effect_type));
        assert_eq!(runtime.registered_effect_types().len(), 1);
        assert_eq!(runtime.registered_effect_types()[0], effect_type);
        
        // Create an effect and parameter
        let effect = TestEffect;
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::new();
        
        // Execute the effect
        let outcome = runtime.execute(&effect, param, &context).await.unwrap();
        
        // Verify the outcome
        assert_eq!(outcome.result, "Handled: hello");
    }
    
    #[tokio::test]
    async fn test_global_runtime() {
        // Reset the global runtime to ensure a clean test
        reset_effect_runtime();
        
        // Create a factory
        let factory = EngineEffectRuntimeFactory::new();
        factory.set_register_as_global(true);
        
        // Create a runtime through the factory
        let runtime = factory.create_runtime();
        
        // Get the global runtime
        let global_runtime = get_effect_runtime();
        
        // They should be the same instance
        assert!(Arc::ptr_eq(&runtime, &global_runtime));
    }
} 