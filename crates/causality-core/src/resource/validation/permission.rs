// Permission validation module
// This file contains components for validating resource permissions.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource_types::{ResourceId, ResourceTypeId};
use crate::capability::Capability;

use super::context::ValidationContext;
use super::result::{ValidationResult, ValidationIssue, ValidationError, ValidationSeverity};
use super::validation::Validator;

/// Error types specific to permission validation
#[derive(Error, Debug, Clone)]
pub enum PermissionValidationError {
    /// Missing required capability
    #[error("Missing required capability: {0}")]
    MissingCapability(String),
    
    /// Insufficient capabilities
    #[error("Insufficient capabilities for operation: {0}")]
    InsufficientCapabilities(String),
    
    /// Permission not found
    #[error("Permission not found: {0}")]
    PermissionNotFound(String),
    
    /// Invalid permission format
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Internal error
    #[error("Internal permission validation error: {0}")]
    InternalError(String),
}

/// Resource permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    /// Permission ID
    pub id: String,
    
    /// Permission name
    pub name: String,
    
    /// Resource type this permission applies to
    pub resource_type: Option<ResourceTypeId>,
    
    /// Required capabilities to grant this permission
    pub required_capabilities: HashSet<String>,
    
    /// Allowed operations
    pub allowed_operations: HashSet<String>,
    
    /// Condition for this permission
    pub condition: Option<String>,
    
    /// Permission scope
    pub scope: PermissionScope,
}

/// Permission scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    /// Global permission (applies to all resources)
    Global,
    
    /// Domain-specific permission
    Domain(String),
    
    /// Resource-specific permission
    Resource(ResourceId),
    
    /// Resource type-specific permission
    ResourceType(ResourceTypeId),
}

/// Permission verification options
#[derive(Debug, Clone)]
pub struct PermissionVerificationOptions {
    /// Skip capability verification
    pub skip_capability_verification: bool,
    
    /// Allow if any capability matches
    pub allow_partial_capability_match: bool,
    
    /// Check conditions
    pub check_conditions: bool,
}

impl Default for PermissionVerificationOptions {
    fn default() -> Self {
        Self {
            skip_capability_verification: false,
            allow_partial_capability_match: false,
            check_conditions: true,
        }
    }
}

/// Permission validator for resources
#[derive(Debug)]
pub struct PermissionValidator {
    /// Permission registry
    permissions: RwLock<HashMap<String, ResourcePermission>>,
    
    /// Permission by resource type
    permissions_by_type: RwLock<HashMap<ResourceTypeId, Vec<String>>>,
}

impl PermissionValidator {
    /// Create a new permission validator
    pub fn new() -> Self {
        Self {
            permissions: RwLock::new(HashMap::new()),
            permissions_by_type: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a permission
    pub fn register_permission(&self, permission: ResourcePermission) -> Result<(), PermissionValidationError> {
        let permission_id = permission.id.clone();
        
        // Add to main registry
        {
            let mut permissions = self.permissions.write().map_err(|e|
                PermissionValidationError::InternalError(format!("Failed to acquire permissions lock: {}", e))
            )?;
            
            permissions.insert(permission_id.clone(), permission.clone());
        }
        
        // Add to type index if applicable
        if let Some(resource_type) = &permission.resource_type {
            let mut permissions_by_type = self.permissions_by_type.write().map_err(|e|
                PermissionValidationError::InternalError(format!("Failed to acquire permissions by type lock: {}", e))
            )?;
            
            let type_permissions = permissions_by_type
                .entry(resource_type.clone())
                .or_insert_with(Vec::new);
                
            if !type_permissions.contains(&permission_id) {
                type_permissions.push(permission_id);
            }
        }
        
        Ok(())
    }
    
    /// Get a permission by ID
    pub fn get_permission(&self, permission_id: &str) -> Result<Option<ResourcePermission>, PermissionValidationError> {
        let permissions = self.permissions.read().map_err(|e|
            PermissionValidationError::InternalError(format!("Failed to acquire permissions lock: {}", e))
        )?;
        
        Ok(permissions.get(permission_id).cloned())
    }
    
    /// Get permissions for a resource type
    pub fn get_permissions_for_type(&self, resource_type: &ResourceTypeId) -> Result<Vec<ResourcePermission>, PermissionValidationError> {
        let permissions_by_type = self.permissions_by_type.read().map_err(|e|
            PermissionValidationError::InternalError(format!("Failed to acquire permissions by type lock: {}", e))
        )?;
        
        let permissions = self.permissions.read().map_err(|e|
            PermissionValidationError::InternalError(format!("Failed to acquire permissions lock: {}", e))
        )?;
        
        let mut result = Vec::new();
        
        if let Some(permission_ids) = permissions_by_type.get(resource_type) {
            for id in permission_ids {
                if let Some(permission) = permissions.get(id) {
                    result.push(permission.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Verify if capabilities satisfy a permission
    pub fn verify_capabilities(
        &self,
        permission: &ResourcePermission,
        capabilities: &CapabilitySet,
        operation: Option<&str>,
        options: &PermissionVerificationOptions,
    ) -> Result<ValidationResult, PermissionValidationError> {
        let mut result = ValidationResult::success();
        
        // Skip capability verification if requested
        if options.skip_capability_verification {
            return Ok(result);
        }
        
        // Check if the capabilities fulfill the required capabilities
        let mut missing_capabilities = Vec::new();
        
        for required in &permission.required_capabilities {
            if !capabilities.has_capability(required) {
                missing_capabilities.push(required.clone());
            }
        }
        
        if !missing_capabilities.is_empty() {
            if !options.allow_partial_capability_match || missing_capabilities.len() == permission.required_capabilities.len() {
                result.add_error(
                    format!("Missing required capabilities: {}", missing_capabilities.join(", ")),
                    "MISSING_CAPABILITIES",
                    "permission_validator",
                );
            } else {
                result.add_warning(
                    format!("Some capabilities are missing: {}", missing_capabilities.join(", ")),
                    "PARTIAL_CAPABILITIES",
                    "permission_validator",
                );
            }
        }
        
        // Check operation if specified
        if let Some(op) = operation {
            if !permission.allowed_operations.contains(op) {
                result.add_error(
                    format!("Operation '{}' not allowed by permission '{}'", op, permission.id),
                    "OPERATION_NOT_ALLOWED",
                    "permission_validator",
                );
            }
        }
        
        // Check conditions if enabled
        if options.check_conditions {
            if let Some(condition) = &permission.condition {
                // In a real implementation, evaluate the condition here
                // For this example, we'll just add a warning
                result.add_warning(
                    format!("Condition evaluation not implemented: {}", condition),
                    "CONDITION_NOT_EVALUATED",
                    "permission_validator",
                );
            }
        }
        
        Ok(result)
    }
}

#[async_trait]
impl Validator for PermissionValidator {
    async fn validate(&self, context: &ValidationContext) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::success();
        
        // Need capabilities to validate permissions
        let capabilities = match &context.capabilities {
            Some(caps) => caps,
            None => {
                result.add_error(
                    "Missing capabilities for permission validation",
                    "MISSING_CAPABILITIES",
                    "permission_validator",
                );
                return Ok(result);
            }
        };
        
        // Get the operation from context
        let operation = context.get_string_context("operation");
        
        // Determine resource type to check
        if let Some(resource_type) = &context.resource_type {
            // Get permissions for this resource type
            let permissions = self.get_permissions_for_type(resource_type)
                .map_err(|e| ValidationError::PermissionError(e.to_string()))?;
            
            if permissions.is_empty() {
                result.add_warning(
                    format!("No permissions defined for resource type: {}", resource_type),
                    "NO_PERMISSIONS_DEFINED",
                    "permission_validator",
                );
                return Ok(result);
            }
            
            // Verify each permission
            let options = PermissionVerificationOptions::default();
            
            for permission in permissions {
                let perm_result = self.verify_capabilities(
                    &permission,
                    capabilities,
                    operation.as_deref(),
                    &options,
                ).map_err(|e| ValidationError::PermissionError(e.to_string()))?;
                
                // If at least one permission validates successfully, the operation is allowed
                if perm_result.is_valid() {
                    // Success
                    return Ok(ValidationResult::success());
                } else {
                    // Merge results to collect errors
                    result.merge(perm_result);
                }
            }
            
            // If we get here, no permission validated successfully
            if result.is_valid() {
                // No explicit errors, but no permission matched
                result.add_error(
                    format!("No applicable permission for operation on resource type: {}", resource_type),
                    "NO_APPLICABLE_PERMISSION",
                    "permission_validator",
                );
            }
        } else if let Some(resource_id) = &context.resource_id {
            // For resource-specific permissions
            result.add_warning(
                "Resource-specific permission validation not implemented",
                "NOT_IMPLEMENTED",
                "permission_validator",
            );
        } else {
            // No resource type or ID
            result.add_error(
                "Missing resource type or ID for permission validation",
                "MISSING_RESOURCE_INFO",
                "permission_validator",
            );
        }
        
        Ok(result)
    }
    
    async fn validate_with_options(
        &self, 
        context: &ValidationContext,
        _options: super::context::ValidationOptions,
    ) -> Result<ValidationResult, ValidationError> {
        // Options don't affect permission validation for now
        self.validate(context).await
    }
    
    fn name(&self) -> &str {
        "PermissionValidator"
    }
}

/// Helper function to verify permission
pub fn verify_permission(
    permission: &ResourcePermission,
    capabilities: &CapabilitySet,
    operation: Option<&str>,
    options: Option<PermissionVerificationOptions>,
) -> Result<bool, PermissionValidationError> {
    let validator = PermissionValidator::new();
    let options = options.unwrap_or_default();
    
    let result = validator.verify_capabilities(
        permission,
        capabilities,
        operation,
        &options,
    )?;
    
    Ok(result.is_valid())
} 