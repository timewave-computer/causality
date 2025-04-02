// Invocation registry implementation
// Original file: src/invocation/registry.rs

// Invocation registry module
//
// This module provides functionality for registering and managing effect handlers
// and resolving them during invocation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_error::{Error, Result, EngineError};
use causality_types::{DomainId, ContentId};
use crate::invocation::context::InvocationContext;

/// Resource access level for effect handlers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Read-only access to a resource
    ReadOnly,
    /// Read-write access to a resource
    ReadWrite,
}

/// Resource requirement for an effect handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    /// Resource ID
    pub resource_id: ContentId,
    /// Required access level
    pub access_level: AccessLevel,
}

/// Handler registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerRegistration {
    /// Unique handler ID
    pub handler_id: String,
    /// Display name for the handler
    pub display_name: String,
    /// Description of the handler's purpose
    pub description: String,
    /// Target domain this handler operates on
    pub target_domain: DomainId,
    /// Resource requirements for this handler
    pub resources: Vec<ResourceRequirement>,
    /// Handler version
    pub version: String,
    /// Handler metadata
    pub metadata: HashMap<String, String>,
}

impl HandlerRegistration {
    /// Create a new handler registration
    pub fn new(
        handler_id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        target_domain: DomainId,
    ) -> Self {
        HandlerRegistration {
            handler_id: handler_id.into(),
            display_name: display_name.into(),
            description: description.into(),
            target_domain,
            resources: Vec::new(),
            version: "0.1.0".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a resource requirement to this handler
    pub fn with_resource(mut self, resource_id: ContentId, access_level: AccessLevel) -> Self {
        self.resources.push(ResourceRequirement {
            resource_id,
            access_level,
        });
        self
    }
    
    /// Set the handler version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Add metadata to this handler
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Handler input for effect invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerInput {
    /// Action to perform
    pub action: String,
    /// Input parameters
    pub params: serde_json::Value,
    /// Context for this invocation
    pub context: Arc<RwLock<InvocationContext>>,
}

/// Handler output from effect invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerOutput {
    /// Result data
    pub data: serde_json::Value,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl HandlerOutput {
    /// Create a new handler output with the given data
    pub fn new(data: serde_json::Value) -> Self {
        HandlerOutput {
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to this output
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Effect handler trait
#[async_trait]
pub trait EffectHandler: Send + Sync {
    /// Get the registration information for this handler
    fn get_registration(&self) -> HandlerRegistration;
    
    /// Handle an effect invocation
    async fn handle(&self, input: HandlerInput) -> Result<HandlerOutput>;
}

/// Registry for effect handlers
#[derive(Debug)]
pub struct EffectRegistry {
    /// Registered handlers by handler ID
    handlers: RwLock<HashMap<String, Arc<dyn EffectHandler>>>,
    /// Handlers by domain ID
    domain_handlers: RwLock<HashMap<DomainId, Vec<String>>>,
}

impl EffectRegistry {
    /// Create a new effect registry
    pub fn new() -> Self {
        EffectRegistry {
            handlers: RwLock::new(HashMap::new()),
            domain_handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new effect handler
    pub fn register_handler(&self, handler: Arc<dyn EffectHandler>) -> Result<()> {
        let registration = handler.get_registration();
        let handler_id = registration.handler_id.clone();
        let domain_id = registration.target_domain;
        
        // Update the handlers map
        {
            let mut handlers = self.handlers.write().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire write lock on handlers".to_string()))?;
            
            if handlers.contains_key(&handler_id) {
                return Err(Error::InvalidArgument(format!("Handler with ID '{}' already registered", handler_id)));
            }
            
            handlers.insert(handler_id.clone(), handler);
        }
        
        // Update the domain handlers map
        {
            let mut domain_handlers = self.domain_handlers.write().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire write lock on domain handlers".to_string()))?;
            
            domain_handlers
                .entry(domain_id)
                .or_insert_with(Vec::new)
                .push(handler_id);
        }
        
        Ok(())
    }
    
    /// Unregister an effect handler
    pub fn unregister_handler(&self, handler_id: &str) -> Result<()> {
        // Remove from the handlers map
        let handler = {
            let mut handlers = self.handlers.write().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire write lock on handlers".to_string()))?;
            
            handlers.remove(handler_id)
        };
        
        // If it exists, also remove from domain handlers
        if let Some(handler) = handler {
            let domain_id = handler.get_registration().target_domain;
            
            let mut domain_handlers = self.domain_handlers.write().map_err(|_| 
                EngineError::LockAcquisitionError("Failed to acquire write lock on domain handlers".to_string()))?;
            
            if let Some(handlers) = domain_handlers.get_mut(&domain_id) {
                handlers.retain(|id| id != handler_id);
            }
        }
        
        Ok(())
    }
    
    /// Get a handler by ID
    pub fn get_handler(&self, handler_id: &str) -> Result<Option<Arc<dyn EffectHandler>>> {
        let handlers = self.handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on handlers".to_string()))?;
        
        Ok(handlers.get(handler_id).cloned())
    }
    
    /// Get all handlers for a domain
    pub fn get_handlers_for_domain(&self, domain_id: &DomainId) -> Result<Vec<Arc<dyn EffectHandler>>> {
        let domain_handlers = self.domain_handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on domain handlers".to_string()))?;
        
        let handlers = self.handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on handlers".to_string()))?;
        
        let handler_ids = match domain_handlers.get(domain_id) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        let domain_handlers = handler_ids
            .iter()
            .filter_map(|id| handlers.get(id).cloned())
            .collect();
        
        Ok(domain_handlers)
    }
    
    /// Get all registered handlers
    pub fn get_all_handlers(&self) -> Result<Vec<Arc<dyn EffectHandler>>> {
        let handlers = self.handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on handlers".to_string()))?;
        
        let all_handlers = handlers.values().cloned().collect();
        
        Ok(all_handlers)
    }
    
    /// Count the number of registered handlers
    pub fn count_handlers(&self) -> Result<usize> {
        let handlers = self.handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on handlers".to_string()))?;
        
        Ok(handlers.len())
    }
    
    /// Get the registration information for all handlers
    pub fn get_all_registrations(&self) -> Result<Vec<HandlerRegistration>> {
        let handlers = self.handlers.read().map_err(|_| 
            EngineError::LockAcquisitionError("Failed to acquire read lock on handlers".to_string()))?;
        
        let registrations = handlers
            .values()
            .map(|h| h.get_registration())
            .collect();
        
        Ok(registrations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock implementation of EffectHandler for testing
    struct MockHandler {
        registration: HandlerRegistration,
    }
    
    impl MockHandler {
        fn new(id: &str, domain: DomainId) -> Self {
            MockHandler {
                registration: HandlerRegistration::new(
                    id,
                    format!("Mock Handler {}", id),
                    "Test handler for unit tests",
                    domain,
                ),
            }
        }
    }
    
    #[async_trait]
    impl EffectHandler for MockHandler {
        fn get_registration(&self) -> HandlerRegistration {
            self.registration.clone()
        }
        
        async fn handle(&self, _input: HandlerInput) -> Result<HandlerOutput> {
            Ok(HandlerOutput::new(serde_json::json!({
                "status": "success",
                "handler_id": self.registration.handler_id,
            })))
        }
    }
    
    #[tokio::test]
    async fn test_registry_operations() -> Result<()> {
        // Create a registry
        let registry = EffectRegistry::new();
        
        // Initial state should be empty
        assert_eq!(registry.count_handlers()?, 0);
        
        // Register a handler
        let domain1 = DomainId::new();
        let handler1 = Arc::new(MockHandler::new("handler1", domain1.clone()));
        registry.register_handler(handler1.clone())?;
        
        // Should now have one handler
        assert_eq!(registry.count_handlers()?, 1);
        
        // Register another handler for the same domain
        let handler2 = Arc::new(MockHandler::new("handler2", domain1.clone()));
        registry.register_handler(handler2.clone())?;
        
        // Register a handler for a different domain
        let domain2 = DomainId::new();
        let handler3 = Arc::new(MockHandler::new("handler3", domain2.clone()));
        registry.register_handler(handler3.clone())?;
        
        // Should now have three handlers total
        assert_eq!(registry.count_handlers()?, 3);
        
        // Check domain-specific handlers
        let domain1_handlers = registry.get_handlers_for_domain(&domain1)?;
        assert_eq!(domain1_handlers.len(), 2);
        
        let domain2_handlers = registry.get_handlers_for_domain(&domain2)?;
        assert_eq!(domain2_handlers.len(), 1);
        
        // Unregister a handler
        registry.unregister_handler("handler2")?;
        
        // Should now have two handlers
        assert_eq!(registry.count_handlers()?, 2);
        
        // Check domain1 now has one handler
        let domain1_handlers = registry.get_handlers_for_domain(&domain1)?;
        assert_eq!(domain1_handlers.len(), 1);
        
        // Get all registrations
        let registrations = registry.get_all_registrations()?;
        assert_eq!(registrations.len(), 2);
        
        // Get by ID
        let handler = registry.get_handler("handler1")?;
        assert!(handler.is_some());
        
        let missing_handler = registry.get_handler("nonexistent")?;
        assert!(missing_handler.is_none());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_handler_registration_builder() {
        let domain = DomainId::new();
        let resource1 = ContentId::new();
        let resource2 = ContentId::new();
        
        let registration = HandlerRegistration::new(
            "test-handler",
            "Test Handler",
            "A test handler for testing",
            domain,
        )
        .with_resource(resource1, AccessLevel::ReadOnly)
        .with_resource(resource2, AccessLevel::ReadWrite)
        .with_version("1.0.0")
        .with_metadata("author", "Test Author")
        .with_metadata("priority", "high");
        
        assert_eq!(registration.handler_id, "test-handler");
        assert_eq!(registration.display_name, "Test Handler");
        assert_eq!(registration.description, "A test handler for testing");
        assert_eq!(registration.target_domain, domain);
        assert_eq!(registration.resources.len(), 2);
        assert_eq!(registration.version, "1.0.0");
        assert_eq!(registration.metadata.len(), 2);
        assert_eq!(registration.metadata.get("author"), Some(&"Test Author".to_string()));
        assert_eq!(registration.metadata.get("priority"), Some(&"high".to_string()));
        
        // Check resources
        assert_eq!(registration.resources[0].resource_id, resource1);
        assert_eq!(registration.resources[0].access_level, AccessLevel::ReadOnly);
        assert_eq!(registration.resources[1].resource_id, resource2);
        assert_eq!(registration.resources[1].access_level, AccessLevel::ReadWrite);
    }

    #[tokio::test]
    async fn test_handler_output_builder() {
        let output = HandlerOutput::new(serde_json::json!({
            "result": "success",
            "value": 42
        }))
        .with_metadata("processing_time", "10ms")
        .with_metadata("source", "test");
        
        // Verify the output fields
        assert_eq!(output.data["result"], "success");
        assert_eq!(output.data["value"], 42);
        assert_eq!(output.metadata.len(), 2);
        assert_eq!(output.metadata.get("processing_time"), Some(&"10ms".to_string()));
        assert_eq!(output.metadata.get("source"), Some(&"test".to_string()));
    }
} 
