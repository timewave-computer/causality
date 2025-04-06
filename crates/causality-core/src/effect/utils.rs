// Effect Utilities
//
// This module provides utility functions for working with the effect system.

use std::sync::Arc;
use std::fmt::Debug;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectOutcome, BasicEffectRegistry, handler::HandlerResult
};
use crate::effect::handler::EffectHandler;
use crate::effect::types::EffectTypeId;
use crate::resource::ResourceManager;


/// Represents a simple effect handler that always returns a successful outcome.
#[derive(Debug)]
pub struct SuccessEffectHandler {
    pub effect_type_id: EffectTypeId,
    pub message: String,
}

#[async_trait]
impl EffectHandler for SuccessEffectHandler {
    fn supported_effect_types(&self) -> Vec<EffectTypeId> {
        vec![self.effect_type_id.clone()]
    }
    
    async fn handle(&self, _effect: &dyn Effect, _context: &dyn EffectContext) -> HandlerResult<EffectOutcome> {
        let mut data = HashMap::new();
        data.insert("message".to_string(), self.message.clone());
        Ok(EffectOutcome::success(data))
    }
}

/// A mock effect handler for testing
#[derive(Debug)]
pub struct MockEffectHandler {
    type_id: EffectTypeId,
    name: String,
    description: String,
}

impl MockEffectHandler {
    pub fn new(type_id: EffectTypeId, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            type_id,
            name: name.into(),
            description: description.into(),
        }
    }
}

#[async_trait]
impl EffectHandler for MockEffectHandler {
    fn supported_effect_types(&self) -> Vec<EffectTypeId> {
        vec![self.type_id.clone()]
    }
    
    async fn handle(&self, _effect: &dyn Effect, _context: &dyn EffectContext) -> HandlerResult<EffectOutcome> {
        Ok(EffectOutcome::success(HashMap::new()))
    }
}

/// Register resource handlers with a default effect registry
pub fn register_resource_handlers(
    _registry: &mut BasicEffectRegistry,
    _resource_manager: Arc<dyn ResourceManager>
) {
    // Placeholder for the commented out register_resource_handler function
}

/// Convert an error to a HashMap for error data handling
pub fn error_to_map<E: std::fmt::Display>(error: E) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("error".to_string(), error.to_string());
    map
}

/// Convert an EffectError to a HashMap with detailed error information
pub fn effect_error_to_map(error: &EffectError) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("error".to_string(), error.to_string());
    
    // Add error type information
    let error_type = match error {
        &EffectError::MissingCapability(_) => "missing_capability",
        &EffectError::MissingResource(_) => "missing_resource",
        &EffectError::ExecutionError(_) => "execution_error",
        &EffectError::HandlerNotFound(_) => "handler_not_found",
        &EffectError::ValidationError(_) => "validation_error",
        &EffectError::SerializationError(_) => "serialization_error",
        &EffectError::ResourceAccessDenied(_) => "resource_access_denied",
        &EffectError::Timeout(_) => "timeout",
        &EffectError::Other(_) => "other",
        &EffectError::NotFound(_) => "not_found",
        &EffectError::InvalidOperation(_) => "invalid_operation",
        &EffectError::InvalidParameter(_) => "invalid_parameter",
        &EffectError::DuplicateRegistration(_) => "duplicate_registration",
        &EffectError::SystemError(_) => "system_error",
        &EffectError::PermissionDenied(_) => "permission_denied",
        &EffectError::RegistryError(_) => "registry_error",
        &EffectError::InvalidArgument(_) => "invalid_argument",
        &EffectError::AlreadyExists(_) => "already_exists",
    };
    
    map.insert("error_type".to_string(), error_type.to_string());
    
    // Extract details based on error type
    match error {
        &EffectError::MissingCapability(ref msg) => {
            map.insert("capability".to_string(), msg.clone());
        },
        &EffectError::MissingResource(ref msg) => {
            map.insert("resource".to_string(), msg.clone());
        },
        &EffectError::NotFound(ref msg) => {
            map.insert("item".to_string(), msg.clone());
        },
        &EffectError::InvalidParameter(ref msg) => {
            map.insert("parameter".to_string(), msg.clone());
        },
        &EffectError::ExecutionError(ref msg) => {
            map.insert("details".to_string(), msg.clone());
        },
        &EffectError::HandlerNotFound(ref msg) => {
            map.insert("handler".to_string(), msg.clone());
        },
        &EffectError::ValidationError(ref msg) => {
            map.insert("validation_details".to_string(), msg.clone());
        },
        &EffectError::SerializationError(ref msg) => {
            map.insert("serialization_details".to_string(), msg.clone());
        },
        &EffectError::ResourceAccessDenied(ref msg) => {
            map.insert("resource".to_string(), msg.clone());
        },
        &EffectError::Timeout(ref msg) => {
            map.insert("operation".to_string(), msg.clone());
        },
        &EffectError::Other(ref msg) => {
            map.insert("details".to_string(), msg.clone());
        },
        &EffectError::InvalidOperation(ref msg) => {
            map.insert("operation".to_string(), msg.clone());
        },
        &EffectError::DuplicateRegistration(ref msg) => {
            map.insert("item".to_string(), msg.clone());
        },
        &EffectError::SystemError(ref msg) => {
            map.insert("system_details".to_string(), msg.clone());
        },
        &EffectError::PermissionDenied(ref msg) => {
            map.insert("permission".to_string(), msg.clone());
        },
        &EffectError::RegistryError(ref msg) => {
            map.insert("registry_details".to_string(), msg.clone());
        },
        &EffectError::InvalidArgument(ref msg) => {
            map.insert("argument".to_string(), msg.clone());
        },
        &EffectError::AlreadyExists(ref msg) => {
            map.insert("object".to_string(), msg.clone());
        }
    }
    
    map
}

/// Helper function to create a failure EffectOutcome from an EffectError
pub fn effect_error_to_outcome(error: EffectError) -> EffectOutcome {
    let data = effect_error_to_map(&error);
    EffectOutcome::failure(error.to_string()).with_data_map(data)
} 