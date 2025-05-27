//! Service Provider Interfaces (SPI)
//!
//! This module consolidates all Service Provider Interface (SPI) trait definitions for the 
//! Causality framework. These interfaces establish the contract between the core system and 
//! extension components, enabling flexible dependency injection and component substitution.
//!
//! The module includes interfaces for:
//! - Context management (execution, runtime, and expression contexts)
//! - Resource handling and storage
//! - Message passing and communication
//! - Effect handling and processing
//! - Registry and service discovery

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::any::Any;

use crate::primitive::ids::{DomainId, EntityId, ExprId, HandlerId, ResourceId, ValueExprId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::expression::ast::Expr as TypesExpr;
use crate::expression::result::{ExprError as LispError, ExprResult};
use crate::expression::value::ValueExpr;
use crate::resource::{Nullifier, Resource};

//-----------------------------------------------------------------------------
// Context Provider Interfaces
//-----------------------------------------------------------------------------

/// Expression Evaluation Context
///
/// Provides the most basic, read-only view of the state necessary for evaluating expressions.
/// It operates at a low level, focusing on synchronous access to immutable data like resources,
/// values, and expression ASTs.
pub trait AsExprContext {
    /// Get a specific field from a resource (identified by its ID), returning its `ValueExpr`.
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>>;

    /// Evaluate a sub-expression within the current context, returning its `ValueExpr`.
    /// This does not mutate external state.
    fn evaluate_expr(&self, expr: &TypesExpr) -> Result<ValueExpr>;

    /// Check if a resource, identified by its ID, exists and is available (e.g., not nullified locally).
    fn is_resource_available(&self, id: &ResourceId) -> Result<bool>;
}

/// Synchronous Execution Context
///
/// Extends `AsExprContext` to provide capabilities for synchronous state mutation.
/// Designed for the direct, immediate execution of operations where changes are applied 
/// to a local or transaction-scoped state.
#[async_trait]
pub trait AsExecutionContext: AsExprContext + Send + Sync {
    /// Creates a new resource and makes it available in the context.
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId>;

    /// Creates a new resource derived from an existing resource with new data payload.
    /// The original resource remains immutable and unchanged.
    async fn derive_resource_data(
        &mut self,
        id: &ResourceId,
        new_data: ValueExpr,
    ) -> Result<()>;

    /// Marks a resource as nullified (permanently consumed) using a `Nullifier`.
    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()>;

    /// Locks a resource (identified by its ID), indicating it's in use by the current transaction/operation.
    async fn lock_resource(&mut self, id: &ResourceId) -> Result<()>;

    /// Unlocks a resource (identified by its ID), releasing it from the current transaction/operation.
    async fn unlock_resource(&mut self, id: &ResourceId) -> Result<()>;

    /// Checks if a resource exists within this context (by ID), e.g., was created or added.
    async fn has_resource(&self, id: &ResourceId) -> Result<bool>;

    /// Checks if a resource, identified by its ID, has been nullified within this context.
    async fn is_nullified(&self, id: &ResourceId) -> Result<bool>;
}

/// Asynchronous Runtime Context
///
/// Represents the broadest operational environment. An asynchronous interface for interacting 
/// with the wider runtime system, including potentially persistent storage, inter-domain 
/// communication, and global system services.
#[async_trait]
pub trait AsRuntimeContext: Send + Sync {
    /// Retrieves an owned resource by its ID (potentially involving async I/O).
    async fn get_resource(&self, id: &ResourceId) -> Result<Option<Resource>>;

    /// Get a resource by ID (synchronous version)
    fn get_resource_sync(&self, id: &ResourceId) -> Result<Option<Resource>>;

    /// Retrieves the actual ValueExpr data for a given ValueExprId.
    async fn get_value_expr_by_id(&self, id: &ValueExprId) -> Result<Option<ValueExpr>>;

    /// Get a value expression by ID (synchronous version)
    fn get_value_expr_by_id_sync(&self, id: &ValueExprId) -> Result<Option<ValueExpr>>;

    /// Retrieves the intended input resource IDs for the current effect execution.
    async fn get_input_resource_ids(&self) -> Result<Vec<ResourceId>>;

    /// Creates a new resource within the runtime's global state management and returns its ID.
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId>;

    /// Creates a new resource derived from an existing resource with new data payload.
    async fn derive_resource_data(
        &mut self,
        id: &ResourceId,
        new_data: ValueExpr,
    ) -> Result<Resource>;

    /// Synchronous version of derive_resource_data
    fn derive_resource_data_sync(
        &mut self,
        _id: &ResourceId,
        _new_data: ValueExpr,
    ) -> Option<Result<Resource>> {
        None // Default implementation returns None to indicate async call is required
    }

    /// Marks a resource as nullified (permanently consumed) at the runtime level.
    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()>;

    /// Sends a message to another domain via the runtime's messaging infrastructure.
    async fn send_message(
        &mut self,
        target_domain: DomainId,
        message_payload: ValueExpr,
    ) -> Result<()>;

    /// Gets the current causal time from the runtime's global clock.
    async fn current_time(&self) -> Result<Timestamp>;

    /// Synchronous version of current_time
    fn current_time_sync(&self) -> Result<Timestamp>;

    /// Retrieves an `Expr` AST by its ID (synchronous version for when ASTs are expected to be in memory).
    fn get_expr_sync(&self, id: &ExprId) -> Result<Option<TypesExpr>>;

    /// Stores a `ValueExpr` in the runtime's state manager and returns its `ValueExprId`.
    async fn store_value_expr(&self, value_expr: ValueExpr) -> Result<ValueExprId>;
}

/// Static Expression Context
///
/// Provides access to static expressions and symbols that don't change during execution.
pub trait StaticExprContext {
    /// Get a symbol from the context by its name, returning an ExprResult.
    fn get_static_symbol(&self, name: &Str) -> Option<ExprResult>;

    /// Get an expression (AST) from the context by its ID.
    fn get_expr(&self, id: &ExprId) -> Option<&TypesExpr>;
}

/// TEL Context Interface
///
/// Specialized context interface for TEL (Transaction Expression Language) operations.
pub trait TelContextInterface: AsExprContext + Send + Sync {
    /// Get metadata for a specific handler (if any).
    fn get_handler_metadata(&self, handler_id: &HandlerId) -> Option<()>;

    /// Get the current domain ID, if any.
    fn domain_id(&self) -> Option<DomainId>;

    /// Call a registered host function by name.
    fn call_host_function(
        &mut self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Result<ValueExpr, LispError>;

    /// Get a symbol's value directly from this context.
    fn get_symbol(&self, name: &Str) -> Option<ValueExpr>;

    /// Evaluate an expression using this context's specific evaluation rules.
    fn evaluate(&self, expr: &TypesExpr) -> ExprResult;

    /// Get an initial binding for a symbol (typically for Lisp environments).
    fn get_initial_binding(&self, name: &Str) -> Option<ValueExpr>;

    /// Resolve a Lisp symbol to its full ExprResult.
    fn resolve_lisp_symbol(&self, name: &Str) -> Option<ExprResult>;
}

/// Asynchronous TEL Context Interface
///
/// Extends TelContextInterface with asynchronous capabilities.
#[async_trait]
pub trait AsyncTelContextInterface: TelContextInterface {
    /// Check if a given effect, identified by its ID, has completed.
    async fn is_effect_completed(&self, effect_id: &ExprId) -> Result<bool, LispError>;
}

//-----------------------------------------------------------------------------
// Domain-Scoped Provider Interface
//-----------------------------------------------------------------------------

/// Domain-Scoped Provider Interface
///
/// Trait for services that operate within a specific domain context.
#[async_trait]
pub trait AsDomainScoped: Send + Sync {
    /// Get the domain ID that this service is scoped to
    fn domain_id(&self) -> DomainId;

    /// Check if this service can handle operations for the given domain
    fn can_handle_domain(&self, domain_id: &DomainId) -> bool {
        self.domain_id() == *domain_id
    }

    /// Initialize the service for its domain
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the service gracefully
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Effect Handler Interface
//-----------------------------------------------------------------------------

/// Type-erased Effect Handler
///
/// Provides a uniform interface for handling different types of effects.
pub trait ErasedEffectHandler: Send + Sync {
    /// Handle an effect with the given context
    fn handle_effect(
        &self,
        effect: &dyn Any,
        context: &mut dyn AsExecutionContext,
    ) -> Result<Vec<Box<dyn Any>>>;

    /// Get the type name of effects this handler can process
    fn effect_type(&self) -> &'static str;

    /// Get the handler's unique identifier
    fn handler_id(&self) -> HandlerId;

    /// Check if this handler can process the given effect type
    fn can_handle(&self, effect_type: &str) -> bool {
        self.effect_type() == effect_type
    }
}

//-----------------------------------------------------------------------------
// Messaging Interface
//-----------------------------------------------------------------------------

/// Messaging Provider Interface
///
/// Defines the interface for message passing between domains and components.
#[async_trait]
pub trait AsMessenger: Send + Sync {
    /// Send a message to a target domain
    async fn send_message(
        &mut self,
        target_domain: DomainId,
        message: ValueExpr,
    ) -> Result<()>;

    /// Send a message with a correlation ID for request-response patterns
    async fn send_message_with_correlation(
        &mut self,
        target_domain: DomainId,
        message: ValueExpr,
        correlation_id: EntityId,
    ) -> Result<()>;

    /// Receive messages for a specific domain
    async fn receive_messages(&mut self, domain: DomainId) -> Result<Vec<ValueExpr>>;

    /// Check if there are pending messages for a domain
    async fn has_pending_messages(&self, domain: DomainId) -> Result<bool>;

    /// Subscribe to messages for a specific domain
    async fn subscribe(&mut self, domain: DomainId) -> Result<()>;

    /// Unsubscribe from messages for a specific domain
    async fn unsubscribe(&mut self, domain: DomainId) -> Result<()>;
}

//-----------------------------------------------------------------------------
// Registry Interface
//-----------------------------------------------------------------------------

/// Registry Provider Interface
///
/// Provides service discovery and registration capabilities.
#[async_trait]
pub trait AsRegistry<T>: Send + Sync {
    /// Register a service with the given key
    async fn register(&mut self, key: String, service: T) -> Result<()>;

    /// Unregister a service by key
    async fn unregister(&mut self, key: &str) -> Result<Option<T>>;

    /// Look up a service by key
    async fn lookup(&self, key: &str) -> Result<Option<&T>>;

    /// Get all registered service keys
    async fn list_keys(&self) -> Result<Vec<String>>;

    /// Check if a service is registered for the given key
    async fn contains(&self, key: &str) -> Result<bool> {
        Ok(self.lookup(key).await?.is_some())
    }

    /// Clear all registered services
    async fn clear(&mut self) -> Result<()>;
}

//-----------------------------------------------------------------------------
// Request Dispatcher Interface
//-----------------------------------------------------------------------------

/// Request Dispatcher Interface
///
/// Provides routing and dispatching capabilities for requests.
#[async_trait]
pub trait AsRequestDispatcher: Send + Sync {
    /// Dispatch a request to the appropriate handler
    async fn dispatch_request(
        &self,
        request_type: &str,
        request_data: ValueExpr,
        context: &mut dyn AsExecutionContext,
    ) -> Result<ValueExpr>;

    /// Register a handler for a specific request type
    async fn register_handler(
        &mut self,
        request_type: String,
        handler: Box<dyn ErasedEffectHandler>,
    ) -> Result<()>;

    /// Unregister a handler for a specific request type
    async fn unregister_handler(&mut self, request_type: &str) -> Result<bool>;

    /// Check if a handler is registered for the given request type
    async fn has_handler(&self, request_type: &str) -> Result<bool>;

    /// Get all registered request types
    async fn list_request_types(&self) -> Result<Vec<String>>;
}

//-----------------------------------------------------------------------------
// Storage Interfaces
//-----------------------------------------------------------------------------

/// Generic Key-Value Store Interface
///
/// Provides read-only access to a key-value store.
#[async_trait]
pub trait AsKeyValueStore<K, V>: Send + Sync
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync + Clone,
{
    /// Get a value by its key
    async fn get(&self, key: &K) -> Result<Option<V>>;

    /// Check if a key exists in the store
    async fn contains_key(&self, key: &K) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }
}

/// Mutable Key-Value Store Interface
///
/// Extends the read-only store with mutation capabilities.
#[async_trait]
pub trait AsMutableKeyValueStore<K, V>: AsKeyValueStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync + Clone,
{
    /// Set a value for a given key
    async fn set(&mut self, key: K, value: V) -> Result<()>;

    /// Delete a value by its key
    async fn delete(&mut self, key: &K) -> Result<Option<V>>;

    /// Update a value using a function
    async fn update<F>(&mut self, key: &K, updater: F) -> Result<Option<V>>
    where
        F: FnOnce(Option<V>) -> Option<V> + Send;

    /// Get and remove a value atomically
    async fn take(&mut self, key: &K) -> Result<Option<V>> {
        self.delete(key).await
    }
}

//-----------------------------------------------------------------------------
// Helper Types and Implementations
//-----------------------------------------------------------------------------

/// Simple in-memory registry implementation
pub struct MemoryRegistry<T> {
    services: HashMap<String, T>,
}

impl<T> MemoryRegistry<T> {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
}

impl<T> Default for MemoryRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T: Send + Sync> AsRegistry<T> for MemoryRegistry<T> {
    async fn register(&mut self, key: String, service: T) -> Result<()> {
        self.services.insert(key, service);
        Ok(())
    }

    async fn unregister(&mut self, key: &str) -> Result<Option<T>> {
        Ok(self.services.remove(key))
    }

    async fn lookup(&self, key: &str) -> Result<Option<&T>> {
        Ok(self.services.get(key))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.services.keys().cloned().collect())
    }

    async fn clear(&mut self) -> Result<()> {
        self.services.clear();
        Ok(())
    }
}

/// Simple in-memory key-value store implementation
pub struct MemoryStore<K, V> {
    data: HashMap<K, V>,
}

impl<K, V> MemoryStore<K, V>
where
    K: Hash + Eq,
{
    /// Create a new empty store
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<K, V> Default for MemoryStore<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K, V> AsKeyValueStore<K, V> for MemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync + Clone,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        Ok(self.data.get(key).cloned())
    }
}

#[async_trait]
impl<K, V> AsMutableKeyValueStore<K, V> for MemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync + Clone,
    V: Send + Sync + Clone,
{
    async fn set(&mut self, key: K, value: V) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    async fn delete(&mut self, key: &K) -> Result<Option<V>> {
        Ok(self.data.remove(key))
    }

    async fn update<F>(&mut self, key: &K, updater: F) -> Result<Option<V>>
    where
        F: FnOnce(Option<V>) -> Option<V> + Send,
    {
        let current = self.data.get(key).cloned();
        let updated = updater(current);
        
        match updated {
            Some(new_value) => {
                let old_value = self.data.insert(key.clone(), new_value);
                Ok(old_value)
            }
            None => {
                Ok(self.data.remove(key))
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {


    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_memory_registry() {
        let mut registry = MemoryRegistry::<String>::new();
        
        // Test registration
        registry.register("service1".to_string(), "value1".to_string()).await.unwrap();
        
        // Test lookup
        let result = registry.lookup("service1").await.unwrap();
        assert_eq!(result, Some(&"value1".to_string()));
        
        // Test contains
        assert!(registry.contains("service1").await.unwrap());
        assert!(!registry.contains("nonexistent").await.unwrap());
        
        // Test list keys
        let keys = registry.list_keys().await.unwrap();
        assert_eq!(keys, vec!["service1".to_string()]);
        
        // Test unregister
        let removed = registry.unregister("service1").await.unwrap();
        assert_eq!(removed, Some("value1".to_string()));
        
        // Verify removal
        assert!(!registry.contains("service1").await.unwrap());
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_memory_store() {
        let mut store = MemoryStore::<String, i32>::new();
        
        // Test set and get
        store.set("key1".to_string(), 42).await.unwrap();
        let result = store.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, Some(42));
        
        // Test contains
        assert!(store.contains_key(&"key1".to_string()).await.unwrap());
        assert!(!store.contains_key(&"nonexistent".to_string()).await.unwrap());
        
        // Test update
        let old_value = store.update(&"key1".to_string(), |old| {
            old.map(|v| v * 2)
        }).await.unwrap();
        assert_eq!(old_value, Some(42));
        
        let new_value = store.get(&"key1".to_string()).await.unwrap();
        assert_eq!(new_value, Some(84));
        
        // Test delete
        let deleted = store.delete(&"key1".to_string()).await.unwrap();
        assert_eq!(deleted, Some(84));
        
        // Verify deletion
        assert!(!store.contains_key(&"key1".to_string()).await.unwrap());
    }
} 