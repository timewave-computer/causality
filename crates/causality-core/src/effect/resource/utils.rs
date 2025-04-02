// Resource Effect Utilities
//
// This module provides utility functions for working with resource effects.

use std::collections::HashMap;
use std::sync::Arc;

use crate::effect::{
    Effect, EffectContext, EffectType, EffectError, 
    EffectResult, EffectOutcome, EffectRegistry, BasicEffectRegistry
};
use crate::resource::{Resource, ResourceManager};
use crate::resource_types::ResourceId;
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

/// Convert between resource effect error and resource error
pub fn convert_effect_error_to_resource_error(error: EffectError) -> crate::resource::ResourceError {
    match error {
        EffectError::NotFound(resource) => 
            crate::resource::ResourceError::ResourceError(format!("Resource not found: {}", resource)),
        EffectError::PermissionDenied(msg) => 
            crate::resource::ResourceError::ValidationError(format!("Permission denied: {}", msg)),
        EffectError::InvalidOperation(msg) => 
            crate::resource::ResourceError::ResourceError(format!("Invalid operation: {}", msg)),
        EffectError::ExecutionError(msg) => 
            crate::resource::ResourceError::ResourceError(format!("Execution error: {}", msg)),
        EffectError::Other(msg) => 
            crate::resource::ResourceError::ResourceError(msg),
        _ => crate::resource::ResourceError::ResourceError("Unknown error".to_string()),
    }
} 