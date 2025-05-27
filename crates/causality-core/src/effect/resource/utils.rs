// Resource Effect Utilities
//
// This module provides utility functions for working with resource effects.

use std::collections::HashMap;

use crate::effect::{
    EffectError, EffectOutcome, EffectResult, EffectTypeId
};
use causality_error::{CausalityError, ErrorCode, ErrorDomain, Result as CausalityResult, TypesError};
use super::resource::{ResourceEffect, ResourceOperation};

/// Create a resource effect for reading a resource
pub fn create_read_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
    ResourceEffect::new(resource_type, resource_id, ResourceOperation::Read)
}

/// Create a resource effect for writing to a resource
pub fn create_write_effect(resource_type: &str, resource_id: &str, content: &str) -> ResourceEffect {
    ResourceEffect::new(resource_type, resource_id, ResourceOperation::Update)
        .with_parameter("content", content)
}

/// Create a resource effect for creating a resource
pub fn create_create_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
    ResourceEffect::new(resource_type, resource_id, ResourceOperation::Create)
}

/// Create a resource effect for deleting a resource
pub fn create_delete_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
    ResourceEffect::new(resource_type, resource_id, ResourceOperation::Delete)
}

/// Create a resource effect for archiving a resource
pub fn create_archive_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
    ResourceEffect::new(resource_type, resource_id, ResourceOperation::Custom("archive".to_string()))
}

/// Extract resource data from an effect outcome
pub fn extract_resource_data(outcome: &EffectOutcome) -> Option<HashMap<String, String>> {
    if outcome.status == crate::effect::outcome::EffectStatus::Success {
        Some(outcome.data.clone())
    } else {
        None
    }
}

/// Extract resource content from an effect outcome
pub fn extract_resource_content(outcome: &EffectOutcome) -> Option<String> {
    if outcome.status == crate::effect::outcome::EffectStatus::Success {
        outcome.data.get("content").cloned()
    } else {
        None
    }
}

/// Convert EffectError to the appropriate ResourceError variant within TypesError
pub fn convert_effect_error_to_resource_error(error: EffectError) -> TypesError {
    match error {
        EffectError::MissingCapability(cap) => 
            TypesError::ResourceError(format!("Missing capability: {}", cap)),
        EffectError::MissingResource(id) => 
            TypesError::ResourceError(format!("Missing resource: {}", id)),
        EffectError::Other(msg) => 
            TypesError::ResourceError(format!("Other resource error: {}", msg)),
        EffectError::HandlerNotFound(type_id) => 
            TypesError::ResourceError(format!("Handler not found: {}", type_id)),
        EffectError::ValidationError(msg) => 
            TypesError::ResourceError(format!("Validation error: {}", msg)),
        EffectError::SerializationError(msg) => 
            TypesError::ResourceError(format!("Serialization error: {}", msg)),
        EffectError::ResourceAccessDenied(msg) => 
            TypesError::ResourceError(format!("Access denied: {}", msg)),
        EffectError::Timeout(msg) => 
            TypesError::ResourceError(format!("Timeout: {}", msg)),
        EffectError::NotFound(msg) => 
            TypesError::ResourceError(format!("Not found: {}", msg)),
        EffectError::InvalidOperation(msg) | EffectError::InvalidParameter(msg) | EffectError::InvalidArgument(msg) => 
            TypesError::ResourceError(format!("Invalid operation/parameter: {}", msg)),
        EffectError::DuplicateRegistration(msg) => 
            TypesError::ResourceError(format!("Duplicate registration: {}", msg)),
        EffectError::SystemError(msg) | EffectError::RegistryError(msg) => 
            TypesError::ResourceError(format!("System error: {}", msg)),
        EffectError::PermissionDenied(msg) => 
            TypesError::ResourceError(format!("Permission denied: {}", msg)),
        EffectError::AlreadyExists(msg) => 
            TypesError::ResourceError(format!("Already exists: {}", msg)),
        EffectError::ExecutionFailed(msg) => 
            TypesError::ResourceError(format!("Execution failed: {}", msg)),
    }
} 