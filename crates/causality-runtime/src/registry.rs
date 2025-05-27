// Effect Registration and Execution Management
// Defines traits and implementations for registering effect handlers and executing effects.

use crate::engine::EffectContext; // TODO: Replace with actual context type from engine
use crate::error::{EngineError, EngineResult}; // TODO: Define these errors
use causality_core::effect::{
    Effect, EffectError, EffectId, EffectOutcome, EffectResult, EffectType, EffectTypeId,
};
use causality_core::resource::ResourceAccessError; // Keep relevant core imports if needed
use causality_core::identity::IdentityId; // Keep relevant core imports if needed
// Import handler trait from the core crate
use causality_core::effect::handler::EffectHandler;

use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;
use tracing::{debug, error, info, warn}; // For logging

// --- Registry Error ---

// Note: Consider merging EffectRegistryError into a broader EngineError enum if appropriate.
#[derive(Error, Debug)]
pub enum EffectRegistryError {
    #[error("Handler not found for effect type: {0}")]
    HandlerNotFound(EffectTypeId),

    #[error("Duplicate registration for effect type: {0}")]
    DuplicateRegistration(EffectTypeId),

    #[error("Effect handler encountered an error: {0}")]
    HandlerError(String), // Should probably wrap a Box<dyn StdError + Send + Sync>

    #[error("Effect execution error: {0}")]
    ExecutionError(#[from] EffectError), // Reuse core EffectError where applicable

    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    #[error("Internal registry error: {0}")]
    InternalError(String),
}

// --- Core Registry Traits ---

/// Trait for registering effect handlers.
///
/// Implementations manage a collection of handlers mapped to `EffectTypeId`s.
#[async_trait]
pub trait EffectRegistrar: Debug + Send + Sync {
    /// Registers a handler for a specific effect type.
    ///
    /// Returns an error if a handler for this type is already registered.
    async fn register_handler(
        &self,
        handler: Box<dyn EffectHandler>, // Use the EffectHandler trait
    ) -> Result<(), EffectRegistryError>;

    // TODO: Consider adding unregister_handler?
    // async fn unregister_handler(&self, effect_type_id: &EffectTypeId) -> Result<(), EffectRegistryError>;
}

/// Trait for executing effects.
///
/// Implementations look up the appropriate handler for an effect and invoke it
/// within a given context.
#[async_trait]
pub trait EffectExecutor: Debug + Send + Sync {
    /// Executes a given effect instance within the provided context.
    ///
    /// Looks up the handler based on the effect's type, prepares the execution
    /// environment (context), and invokes the handler's `handle` method.
    async fn execute_effect(
        &self,
        effect: Arc<dyn Effect>, // Use Arc for shared ownership
        context: Arc<dyn EffectContext>, // Use Arc<dyn Trait> for context
    ) -> EffectResult<EffectOutcome>; // Use EffectResult and EffectOutcome from core
}

/// Trait for asynchronous effect execution (potentially offloading).
///
/// This is useful for scenarios where effect execution might be long-running
/// or needs to happen in a separate thread or task pool.
#[async_trait]
pub trait AsyncEffectExecutor: EffectExecutor {
    /// Executes an effect asynchronously.
    ///
    /// The exact mechanism (e.g., spawning a task, sending to a queue) is
    /// implementation-dependent. It might return a handle to track the execution.
    // TODO: Define return type (e.g., JoinHandle, future, or just Result<(), Error>)
    async fn execute_effect_async(
        &self,
        effect: Arc<dyn Effect>, // Use Arc for shared ownership
        context: Arc<dyn EffectContext>, // Use Arc<dyn Trait> for context
    ) -> EngineResult<()>; // TODO: Define appropriate result/handle
}

/// Combined trait for convenience.
#[async_trait]
pub trait EffectRegistry: EffectRegistrar + EffectExecutor + Send + Sync {
    // Provides both registration and execution capabilities.
    // Often, the same struct will implement all necessary traits.
}

// --- Concrete Implementations ---

// --- BasicEffectRegistry (Single-threaded, simple) ---

/// A basic, single-threaded implementation of the `EffectRegistry`.
///
/// Stores handlers in a `HashMap`. Not suitable for concurrent access without
/// external locking.
#[derive(Debug, Default)]
pub struct BasicEffectRegistry {
    handlers: HashMap<EffectTypeId, Box<dyn EffectHandler>>,
}

impl BasicEffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EffectRegistrar for BasicEffectRegistry {
    async fn register_handler(
        &mut self, // Needs mutable access
        handler: Box<dyn EffectHandler>,
    ) -> Result<(), EffectRegistryError> {
        let effect_type_id = handler.effect_type().id().clone();
        debug!(effect_type_id = ?effect_type_id, "Registering handler");
        if self.handlers.contains_key(&effect_type_id) {
            warn!(effect_type_id = ?effect_type_id, "Attempted duplicate handler registration");
            Err(EffectRegistryError::DuplicateRegistration(effect_type_id))
        } else {
            self.handlers.insert(effect_type_id.clone(), handler);
            info!(effect_type_id = ?effect_type_id, "Handler registered successfully");
            Ok(())
        }
    }
}

#[async_trait]
impl EffectExecutor for BasicEffectRegistry {
    async fn execute_effect(
        &self,
        effect: Arc<dyn Effect>,
        context: Arc<dyn EffectContext>,
    ) -> EffectResult<EffectOutcome> {
        let effect_type_id = effect.effect_type().id();
        debug!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Executing effect");

        match self.handlers.get(effect_type_id) {
            Some(handler) => {
                info!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Found handler, attempting execution");
                // Call the handler's handle method
                handler.handle(effect.clone(), context).await
            }
            None => {
                error!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Handler not found");
                Err(EffectError::NotFound(format!(
                    "No handler registered for effect type: {}",
                    effect_type_id
                )))
            }
        }
    }
}

// BasicEffectRegistry typically won't implement AsyncEffectExecutor directly.

#[async_trait]
impl EffectRegistry for BasicEffectRegistry {} // Mark as implementing the combined trait

// --- ThreadSafeEffectRegistry (Using RwLock for concurrency) ---

/// A thread-safe implementation of the `EffectRegistry` using `RwLock`.
///
/// Allows concurrent reads (lookups) and exclusive writes (registrations).
#[derive(Debug, Default)]
pub struct ThreadSafeEffectRegistry {
    handlers: Arc<RwLock<HashMap<EffectTypeId, Arc<dyn EffectHandler>>>>, // Arc for handlers too
}

impl ThreadSafeEffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EffectRegistrar for ThreadSafeEffectRegistry {
    async fn register_handler(
        &self,
        handler: Box<dyn EffectHandler>, // Box the handler first
    ) -> Result<(), EffectRegistryError> {
        let handler = Arc::from(handler); // Convert to Arc
        let effect_type_id = handler.effect_type().id().clone();
        debug!(effect_type_id = ?effect_type_id, "Attempting to acquire write lock for handler registration");
        let mut handlers_guard = self.handlers.write().map_err(|_| {
            EffectRegistryError::ConcurrencyError("Failed to acquire write lock".to_string())
        })?;
        debug!(effect_type_id = ?effect_type_id, "Write lock acquired");

        if handlers_guard.contains_key(&effect_type_id) {
            warn!(effect_type_id = ?effect_type_id, "Attempted duplicate handler registration");
            Err(EffectRegistryError::DuplicateRegistration(effect_type_id))
        } else {
            handlers_guard.insert(effect_type_id.clone(), handler.clone()); // Clone Arc
            info!(effect_type_id = ?effect_type_id, "Handler registered successfully");
            Ok(())
        }
    }
}

#[async_trait]
impl EffectExecutor for ThreadSafeEffectRegistry {
    async fn execute_effect(
        &self,
        effect: Arc<dyn Effect>,
        context: Arc<dyn EffectContext>,
    ) -> EffectResult<EffectOutcome> {
        let effect_type_id = effect.effect_type().id();
        debug!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Attempting to acquire read lock for effect execution");

        let handler_arc = {
            let handlers_guard = self.handlers.read().map_err(|_| {
                EffectRegistryError::ConcurrencyError("Failed to acquire read lock".to_string())
                    .into_effect_error() // Convert to EffectError
            })?;
            debug!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Read lock acquired");
            handlers_guard.get(effect_type_id).cloned() // Clone the Arc<dyn EffectHandler>
        }; // Lock released here

        match handler_arc {
            Some(handler) => {
                info!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Found handler, attempting execution");
                // Execute the handler *outside* the lock
                handler.handle(effect.clone(), context).await
            }
            None => {
                error!(effect_id = ?effect.id(), effect_type_id = ?effect_type_id, "Handler not found");
                Err(EffectError::NotFound(format!(
                    "No handler registered for effect type: {}",
                    effect_type_id
                )))
            }
        }
    }
}

#[async_trait]
impl AsyncEffectExecutor for ThreadSafeEffectRegistry {
    async fn execute_effect_async(
        &self,
        effect: Arc<dyn Effect>,
        context: Arc<dyn EffectContext>,
    ) -> EngineResult<()> { // Use EngineResult
        let effect_clone = effect.clone();
        let context_clone = context.clone_context_arc(); // Use Arc for context
        let registry_clone = self.handlers.clone(); // Clone Arc<RwLock<...>>
        let effect_id_clone = effect.id();
        let effect_type_id_clone = effect.effect_type().id().clone();

        tokio::spawn(async move {
            debug!(effect_id = ?effect_id_clone, effect_type_id = ?effect_type_id_clone, "Async execution task started");

            let handler_arc = {
                // Use try_read first to avoid blocking if write lock is held briefly
                if let Ok(guard) = registry_clone.try_read() {
                    guard.get(&effect_type_id_clone).cloned()
                } else {
                    // Fallback to blocking read if try_read fails
                    match registry_clone.read() {
                        Ok(guard) => guard.get(&effect_type_id_clone).cloned(),
                        Err(e) => {
                            error!(effect_id = ?effect_id_clone, effect_type_id = ?effect_type_id_clone, error = %e, "Failed to acquire read lock for async execution");
                            None
                        }
                    }
                }
            };

            if let Some(handler) = handler_arc {
                info!(effect_id = ?effect_id_clone, effect_type_id = ?effect_type_id_clone, "Handler found for async execution");
                let result = handler.handle(effect_clone, context_clone).await;
                match result {
                    Ok(outcome) => {
                        info!(effect_id = ?effect_id_clone, ?outcome, "Async effect execution finished");
                        // TODO: What to do with the outcome? Log, notify, update state?
                        // Potential place to interact with LogStorage or StateManager
                    }
                    Err(e) => {
                        error!(effect_id = ?effect_id_clone, error = %e, "Async effect execution failed");
                        // TODO: Error handling (retry?, log details, notify?)
                    }
                }
            } else {
                error!(effect_id = ?effect_id_clone, effect_type_id = ?effect_type_id_clone, "Handler not found for async execution");
                // TODO: Error handling (log, notify?)
            }
        });

        Ok(()) // Return immediately after spawning
    }
}

#[async_trait]
impl EffectRegistry for ThreadSafeEffectRegistry {} // Mark as implementing the combined trait

// Helper to convert RegistryError to EffectError for execute_effect
impl EffectRegistryError {
    fn into_effect_error(self) -> EffectError {
        match self {
            EffectRegistryError::HandlerNotFound(id) => EffectError::NotFound(format!(
                "Handler not found for effect type: {}",
                id
            )),
            EffectRegistryError::DuplicateRegistration(id) => {
                EffectError::ConfigurationError(format!("Duplicate registration: {}", id))
            }
            EffectRegistryError::HandlerError(msg) => EffectError::HandlerError(msg),
            EffectRegistryError::ExecutionError(e) => e, // Pass through
            EffectRegistryError::ConcurrencyError(msg) | EffectRegistryError::InternalError(msg) => {
                EffectError::InternalError(msg)
            }
        }
    }
}

// --- Unit Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::{
        Effect, EffectType, EffectName, EffectVersion, EffectTypeId, EffectOutcome, EffectResult,
        EffectStatus, handler::EffectHandler, runtime::EffectRuntimeContext
    };
    use crate::engine::BasicEffectContext; // Assuming BasicEffectContext is in engine
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use std::any::Any;
    use causality_types::resource::{ResourceId, Capability};
    use std::collections::HashSet;

    // --- Mock Effect ---
    #[derive(Debug)]
    struct MockEffect {
        id: EffectId,
        effect_type: EffectType,
        description: String,
        should_succeed: bool,
    }

    impl MockEffect {
        fn new(id: &str, type_name: &str, should_succeed: bool) -> Self {
            Self {
                id: EffectId::new(id),
                effect_type: EffectType::new(type_name, "Mock effect type"),
                description: format!("Mock effect {}", id),
                should_succeed,
            }
        }
    }

    #[async_trait]
    impl Effect for MockEffect {
        fn id(&self) -> EffectId { self.id.clone() }
        fn effect_type(&self) -> EffectType { self.effect_type.clone() }
        fn description(&self) -> String { self.description.clone() }

        // Mock execute - the registry calls handler.handle, not effect.execute
        async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
             panic!("Registry should call handler.handle, not effect.execute");
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // --- Mock Handler ---
    #[derive(Debug)]
    struct MockHandler {
        effect_type: EffectType,
    }

    impl MockHandler {
        fn new(type_name: &str) -> Self {
            Self {
                effect_type: EffectType::new(type_name, "Mock handler type"),
            }
        }
    }

    #[async_trait]
    impl EffectHandler for MockHandler {
        fn effect_type(&self) -> &EffectType {
            &self.effect_type
        }

        async fn handle(
            &self,
            effect: Arc<dyn Effect>,
            _context: Arc<dyn EffectContext>,
        ) -> EffectResult<EffectOutcome> {
            info!("MockHandler handling effect: {:?}", effect.id());
            // Downcast to check the should_succeed flag
            let mock_effect = effect.as_any().downcast_ref::<MockEffect>().unwrap();
            if mock_effect.should_succeed {
                Ok(EffectOutcome::success(HashMap::from([("handled_by".to_string(), self.effect_type.name().to_string())])))
            } else {
                Err(EffectError::HandlerError("Mock handler simulated failure".to_string()))
            }
        }
    }

    // --- Test Context (Using BasicEffectContext from engine) ---
    fn create_test_context(effect_id: EffectId) -> Arc<dyn EffectContext> {
        Arc::new(BasicEffectContext::new(effect_id))
    }

    // --- Basic Registry Tests ---

    #[tokio::test]
    async fn test_basic_registry_register_and_execute() {
        let mut registry = BasicEffectRegistry::new();
        let handler = Box::new(MockHandler::new("TestEffect"));
        let handler_type_id = handler.effect_type().id().clone();

        // Register
        registry.register_handler(handler).await.unwrap();

        // Execute Success
        let effect_success = Arc::new(MockEffect::new("eff-001", "TestEffect", true));
        let context = create_test_context(effect_success.id());
        let outcome = registry.execute_effect(effect_success, context).await.unwrap();

        assert!(outcome.is_success());
        assert_eq!(outcome.data().get("handled_by").unwrap(), "TestEffect");

        // Execute Failure (Handler should return Err)
        let effect_fail = Arc::new(MockEffect::new("eff-002", "TestEffect", false));
        let context_fail = create_test_context(effect_fail.id());
        let result_fail = registry.execute_effect(effect_fail, context_fail).await;

        assert!(result_fail.is_err());
        match result_fail.err().unwrap() {
             EffectError::HandlerError(msg) => assert!(msg.contains("Mock handler simulated failure")),
             _ => panic!("Expected HandlerError"),
        }

        // Execute Unregistered
        let effect_unregistered = Arc::new(MockEffect::new("eff-003", "UnknownEffect", true));
        let context_unregistered = create_test_context(effect_unregistered.id());
        let result = registry.execute_effect(effect_unregistered, context_unregistered).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            EffectError::NotFound(msg) => assert!(msg.contains("UnknownEffect")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_basic_registry_duplicate_registration() {
        let mut registry = BasicEffectRegistry::new();
        let handler1 = Box::new(MockHandler::new("DuplicateEffect"));
        let handler2 = Box::new(MockHandler::new("DuplicateEffect"));

        registry.register_handler(handler1).await.unwrap();
        let result = registry.register_handler(handler2).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            EffectRegistryError::DuplicateRegistration(id) => {
                assert_eq!(id.to_string(), "DuplicateEffect-0.1.0"); // Assuming default version
            }
            _ => panic!("Expected DuplicateRegistration error"),
        }
    }

    // --- ThreadSafe Registry Tests ---

    #[tokio::test]
    async fn test_threadsafe_registry_register_and_execute() {
        let registry = ThreadSafeEffectRegistry::new();
        let handler = Box::new(MockHandler::new("SafeEffect"));
        let handler_type_id = handler.effect_type().id().clone();

        // Register
        registry.register_handler(handler).await.unwrap();

        // Execute Success
        let effect_success = Arc::new(MockEffect::new("ts-eff-001", "SafeEffect", true));
        let context = create_test_context(effect_success.id());
        let outcome = registry.execute_effect(effect_success, context).await.unwrap();

        assert!(outcome.is_success());
        assert_eq!(outcome.data().get("handled_by").unwrap(), "SafeEffect");

        // Execute Failure (Handler returns Err)
        let effect_fail = Arc::new(MockEffect::new("ts-eff-002", "SafeEffect", false));
        let context_fail = create_test_context(effect_fail.id());
        let result_fail = registry.execute_effect(effect_fail, context_fail).await;

        assert!(result_fail.is_err());
         match result_fail.err().unwrap() {
             EffectError::HandlerError(msg) => assert!(msg.contains("Mock handler simulated failure")),
             _ => panic!("Expected HandlerError"),
        }

        // Execute Unregistered
        let effect_unregistered = Arc::new(MockEffect::new("ts-eff-003", "UnknownSafeEffect", true));
        let context_unregistered = create_test_context(effect_unregistered.id());
        let result = registry.execute_effect(effect_unregistered, context_unregistered).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            EffectError::NotFound(msg) => assert!(msg.contains("UnknownSafeEffect")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_threadsafe_registry_async_execute() {
        // Use a channel to signal completion from the spawned task
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<EffectOutcome, EffectError>>(1);

        // Need a handler that sends the result back
        #[derive(Debug)]
        struct SignallingHandler {
            effect_type: EffectType,
            tx: tokio::sync::mpsc::Sender<Result<EffectOutcome, EffectError>>,
        }
        #[async_trait]
        impl EffectHandler for SignallingHandler {
            fn effect_type(&self) -> &EffectType { &self.effect_type }
            async fn handle(&self, effect: Arc<dyn Effect>, _context: Arc<dyn EffectContext>) -> EffectResult<EffectOutcome> {
                let mock_effect = effect.as_any().downcast_ref::<MockEffect>().unwrap();
                let result = if mock_effect.should_succeed {
                    Ok(EffectOutcome::success(HashMap::from([("handled_by".to_string(), self.effect_type.name().to_string())])))
                } else {
                    Err(EffectError::HandlerError("Mock handler simulated failure".to_string()))
                };
                // Send result back before returning
                self.tx.send(result.clone()).await.expect("Failed to send result back");
                result
            }
        }

        let registry = Arc::new(ThreadSafeEffectRegistry::new());
        let handler = Box::new(SignallingHandler {
             effect_type: EffectType::new("AsyncEffect", "Async handler"),
             tx: tx.clone(),
        });
        registry.register_handler(handler).await.unwrap();

        let effect = Arc::new(MockEffect::new("async-eff-001", "AsyncEffect", true));
        let context = create_test_context(effect.id());

        // Execute async
        let result = registry.execute_effect_async(effect.clone(), context.clone()).await;
        assert!(result.is_ok(), "Spawning async task failed");

        // Wait for the result from the channel
        match tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await {
            Ok(Some(Ok(outcome))) => {
                assert!(outcome.is_success());
                assert_eq!(outcome.data().get("handled_by").unwrap(), "AsyncEffect");
            }
            Ok(Some(Err(e))) => panic!("Async execution failed: {}", e),
            Ok(None) => panic!("Channel closed unexpectedly"),
            Err(_) => panic!("Timed out waiting for async execution result"),
        }

        // Test async failure
         let effect_fail = Arc::new(MockEffect::new("async-eff-002", "AsyncEffect", false));
         let context_fail = create_test_context(effect_fail.id());
         let handler_fail = Box::new(SignallingHandler {
             effect_type: EffectType::new("AsyncEffectFail", "Async fail handler"), // Different type to avoid duplicate
             tx: tx.clone(),
         });
         // Need a new registry or unregister/register for the fail test
         let registry_fail = Arc::new(ThreadSafeEffectRegistry::new());
         registry_fail.register_handler(handler_fail).await.unwrap();

         let result_spawn_fail = registry_fail.execute_effect_async(effect_fail.clone(), context_fail.clone()).await;
         assert!(result_spawn_fail.is_ok());

          match tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await {
            Ok(Some(Err(e))) => {
                match e {
                     EffectError::HandlerError(msg) => assert!(msg.contains("Mock handler simulated failure")),
                     _ => panic!("Expected HandlerError from async task, got {:?}", e),
                }
            }
            Ok(Some(Ok(_))) => panic!("Async execution unexpectedly succeeded"),
            Ok(None) => panic!("Channel closed unexpectedly (fail test)"),
            Err(_) => panic!("Timed out waiting for async execution result (fail test)"),
        }
    }

    #[tokio::test]
    async fn test_threadsafe_registry_concurrent_registrations() {
        let registry = Arc::new(ThreadSafeEffectRegistry::new());
        let mut tasks = vec![];

        for i in 0..10 {
            let registry_clone = registry.clone();
            tasks.push(tokio::spawn(async move {
                let handler = Box::new(MockHandler::new(&format!("ConcurrentEffect-{}", i)));
                registry_clone.register_handler(handler).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for result in results {
            assert!(result.unwrap().is_ok()); // Check each registration succeeded
        }

        // Verify all handlers are present
        let handlers_guard = registry.handlers.read().unwrap();
        assert_eq!(handlers_guard.len(), 10);
        assert!(handlers_guard.contains_key(&EffectTypeId::from_str("ConcurrentEffect-5-0.1.0").unwrap())); // Check one
    }

    #[tokio::test]
    async fn test_threadsafe_registry_concurrent_executions() {
        let registry = Arc::new(ThreadSafeEffectRegistry::new());
        let handler = Box::new(MockHandler::new("ConcurrentExec"));
        registry.register_handler(handler).await.unwrap();

        let mut tasks = vec![];
        for i in 0..20 {
            let registry_clone = registry.clone();
            tasks.push(tokio::spawn(async move {
                let effect = Arc::new(MockEffect::new(&format!("conc-eff-{}", i), "ConcurrentExec", true));
                let context = create_test_context(effect.id());
                registry_clone.execute_effect(effect, context).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for result in results {
            let outcome = result.unwrap().unwrap(); // Check outer Result and inner EffectResult
            assert!(outcome.is_success());
            assert_eq!(outcome.data().get("handled_by").unwrap(), "ConcurrentExec");
        }
    }
}

//! Effect Handler Registry Implementation
// TODO: Add necessary imports from causality_core (EffectHandler, EffectTypeId, EffectError, etc.)

use std::fmt::Debug;
use std::sync::Arc;
// use async_trait::async_trait; // Likely not needed here directly

// TODO: Import EffectHandler, EffectTypeId, HandlerResult, Effect, EffectContext, EffectOutcome, EffectError
// Placeholder imports - replace with actual paths
use crate::error::RuntimeError; // Assuming a runtime error type exists
type EffectTypeId = String; // Placeholder
type HandlerResult<T> = Result<T, RuntimeError>; // Placeholder
trait EffectHandler: Send + Sync + Debug { fn supported_effect_types(&self) -> Vec<EffectTypeId>; } // Placeholder

/// A registry of effect handlers
#[derive(Debug, Default)]
pub struct EffectHandlerRegistry {
    /// Handlers by effect type
    handlers: Vec<Arc<dyn EffectHandler>>,
}

impl EffectHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a handler
    pub fn register(&mut self, handler: Arc<dyn EffectHandler>) {
        self.handlers.push(handler);
    }

    /// Find a handler for an effect type
    pub fn find_handler(&self, effect_type: &EffectTypeId) -> Option<Arc<dyn EffectHandler>> {
        // TODO: Ensure EffectTypeId comparison works correctly
        self.handlers.iter()
            .find(|h| h.supported_effect_types().contains(effect_type))
            .cloned()
    }

    /// Get all handlers
    pub fn handlers(&self) -> &[Arc<dyn EffectHandler>] {
        &self.handlers
    }

    // TODO: Potentially add an 'execute' or 'handle' method here that finds and calls the handler
    // This would depend on where the core execution loop resides.
}

// TODO: Add tests for the registry 