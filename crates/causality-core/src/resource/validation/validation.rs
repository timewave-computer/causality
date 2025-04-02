// Resource validator implementation
// This file contains the main resource validator implementation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::resource::{
    ResourceId, ResourceTypeId, ResourceState, ResourceSchema,
    Resource,
};
use crate::capability::Capability;
use super::context::{ValidationContext, ValidationPhase, ValidationOptions};
use super::result::{ValidationResult, ValidationIssue, ValidationError, ValidationStatus, ValidationSeverity};
use super::state::{StateTransitionValidator, StateTransitionRule};
use super::schema::{SchemaValidator, SchemaCompatibility};
use super::permission::{PermissionValidator, ResourcePermission};
use super::custom::{CustomValidator, CustomValidationRule};

use causality_types::ContentId;

/// Configuration for the resource validator
#[derive(Debug, Clone)]
pub struct ResourceValidatorConfig {
    /// Default validation options
    pub default_options: ValidationOptions,
    
    /// Whether to enable validation result caching
    pub enable_caching: bool,
    
    /// Maximum cache size
    pub max_cache_size: usize,
    
    /// Whether to validate schemas
    pub validate_schemas: bool,
    
    /// Whether to validate state transitions
    pub validate_state_transitions: bool,
    
    /// Whether to validate permissions
    pub validate_permissions: bool,
    
    /// Whether to register custom validators
    pub enable_custom_validators: bool,
}

impl Default for ResourceValidatorConfig {
    fn default() -> Self {
        Self {
            default_options: ValidationOptions::default(),
            enable_caching: true,
            max_cache_size: 1000,
            validate_schemas: true,
            validate_state_transitions: true,
            validate_permissions: true,
            enable_custom_validators: true,
        }
    }
}

/// Interface for validation components
#[async_trait]
pub trait Validator: Send + Sync {
    /// Validate a resource
    async fn validate(&self, context: &ValidationContext) -> Result<ValidationResult, ValidationError>;
    
    /// Validate using specific options
    async fn validate_with_options(
        &self, 
        context: &ValidationContext,
        options: ValidationOptions,
    ) -> Result<ValidationResult, ValidationError>;
    
    /// Validator name
    fn name(&self) -> &str;
}

/// Main resource validator
#[derive(Debug)]
pub struct ResourceValidator {
    /// Validator configuration
    config: ResourceValidatorConfig,
    
    /// Schema validator
    schema_validator: Arc<SchemaValidator>,
    
    /// State transition validator
    state_validator: Arc<StateTransitionValidator>,
    
    /// Permission validator
    permission_validator: Arc<PermissionValidator>,
    
    /// Custom validators by name
    custom_validators: RwLock<HashMap<String, Arc<dyn CustomValidator>>>,
    
    /// Validation result cache
    validation_cache: RwLock<HashMap<ContentId, ValidationResult>>,
}

impl ResourceValidator {
    /// Create a new resource validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ResourceValidatorConfig::default())
    }
    
    /// Create a new resource validator with specific configuration
    pub fn with_config(config: ResourceValidatorConfig) -> Self {
        Self {
            schema_validator: Arc::new(SchemaValidator::new()),
            state_validator: Arc::new(StateTransitionValidator::new()),
            permission_validator: Arc::new(PermissionValidator::new()),
            custom_validators: RwLock::new(HashMap::new()),
            validation_cache: RwLock::new(HashMap::new()),
            config,
        }
    }
    
    /// Register a custom validator
    pub fn register_custom_validator<V: CustomValidator + 'static>(&self, validator: V) -> Result<(), ValidationError> {
        if !self.config.enable_custom_validators {
            return Err(ValidationError::InternalError(
                "Custom validators are disabled by configuration".to_string()
            ));
        }
        
        let mut validators = self.custom_validators.write().map_err(|e| 
            ValidationError::InternalError(format!("Failed to acquire custom validators lock: {}", e))
        )?;
        
        let validator = Arc::new(validator);
        validators.insert(validator.name().to_string(), validator);
        
        Ok(())
    }
    
    /// Validate a resource with default options
    pub async fn validate_resource<R: Resource + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ValidationResult, ValidationError> {
        let context = ValidationContext::new()
            .with_resource_id(resource.id().clone())
            .with_resource_type(resource.resource_type().clone())
            .with_current_state(resource.state().clone())
            .with_schema(resource.schema().clone())
            .with_phase(ValidationPhase::PreExecution);
            
        self.validate(&context).await
    }
    
    /// Validate a state transition
    pub async fn validate_state_transition(
        &self,
        resource_id: &ResourceId,
        current_state: &ResourceState,
        target_state: &ResourceState,
    ) -> Result<ValidationResult, ValidationError> {
        let context = ValidationContext::new()
            .with_resource_id(resource_id.clone())
            .with_current_state(current_state.clone())
            .with_target_state(target_state.clone())
            .with_phase(ValidationPhase::StateOnly);
            
        self.state_validator.validate(&context).await
    }
    
    /// Validate a schema
    pub async fn validate_schema(
        &self,
        schema: &ResourceSchema,
    ) -> Result<ValidationResult, ValidationError> {
        let context = ValidationContext::new()
            .with_schema(schema.clone())
            .with_phase(ValidationPhase::SchemaOnly);
            
        self.schema_validator.validate(&context).await
    }
    
    /// Validate a permission
    pub async fn validate_permission(
        &self,
        permission: &ResourcePermission,
        capabilities: &Capability,
    ) -> Result<ValidationResult, ValidationError> {
        let context = ValidationContext::new()
            .with_capabilities(capabilities.clone())
            .with_phase(ValidationPhase::PermissionOnly);
            
        // Serialize permission to context data
        let permission_data = serde_json::to_string(permission)
            .map_err(|e| ValidationError::InternalError(
                format!("Failed to serialize permission: {}", e)
            ))?;
            
        let context = context.with_string_context("permission", permission_data);
        
        self.permission_validator.validate(&context).await
    }
    
    /// Run a custom validator
    pub async fn run_custom_validator(
        &self,
        validator_name: &str,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        if !self.config.enable_custom_validators {
            return Err(ValidationError::InternalError(
                "Custom validators are disabled by configuration".to_string()
            ));
        }
        
        let validators = self.custom_validators.read().map_err(|e| 
            ValidationError::InternalError(format!("Failed to acquire custom validators lock: {}", e))
        )?;
        
        let validator = validators.get(validator_name).ok_or_else(|| 
            ValidationError::InternalError(format!("Custom validator not found: {}", validator_name))
        )?;
        
        validator.validate(context).await
    }
    
    /// Clear the validation cache
    pub fn clear_cache(&self) -> Result<(), ValidationError> {
        let mut cache = self.validation_cache.write().map_err(|e| 
            ValidationError::InternalError(format!("Failed to acquire validation cache lock: {}", e))
        )?;
        
        cache.clear();
        
        Ok(())
    }
    
    /// Run the complete validation pipeline
    async fn run_validation_pipeline(
        &self,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::success();
        
        // Schema validation
        if self.config.validate_schemas && context.options.validate_schema {
            if let Some(schema) = &context.schema {
                let schema_result = self.schema_validator.validate(context).await?;
                result.merge(schema_result);
                
                if !result.is_valid() {
                    return Ok(result);
                }
            }
        }
        
        // State transition validation
        if self.config.validate_state_transitions && context.options.validate_state {
            if let (Some(current), Some(target)) = (&context.current_state, &context.target_state) {
                let state_result = self.state_validator.validate(context).await?;
                result.merge(state_result);
                
                if !result.is_valid() {
                    return Ok(result);
                }
            }
        }
        
        // Permission validation
        if self.config.validate_permissions && context.options.validate_permissions {
            if let Some(capabilities) = &context.capabilities {
                let permission_result = self.permission_validator.validate(context).await?;
                result.merge(permission_result);
                
                if !result.is_valid() {
                    return Ok(result);
                }
            }
        }
        
        // Custom validators
        if self.config.enable_custom_validators && context.options.validate_custom_rules {
            let validators = self.custom_validators.read().map_err(|e| 
                ValidationError::InternalError(format!("Failed to acquire custom validators lock: {}", e))
            )?;
            
            for validator in validators.values() {
                if validator.is_applicable(context) {
                    let custom_result = validator.validate(context).await?;
                    result.merge(custom_result);
                    
                    if !result.is_valid() {
                        return Ok(result);
                    }
                }
            }
        }
        
        Ok(result)
    }
}

#[async_trait]
impl Validator for ResourceValidator {
    async fn validate(&self, context: &ValidationContext) -> Result<ValidationResult, ValidationError> {
        self.validate_with_options(context, self.config.default_options.clone()).await
    }
    
    async fn validate_with_options(
        &self, 
        context: &ValidationContext,
        options: ValidationOptions,
    ) -> Result<ValidationResult, ValidationError> {
        // Check cache if enabled
        if self.config.enable_caching && options.enable_caching {
            if let Some(resource_id) = &context.resource_id {
                let cache = self.validation_cache.read().map_err(|e| 
                    ValidationError::InternalError(format!("Failed to acquire validation cache lock: {}", e))
                )?;
                
                // Use ContentId for cache key (since ResourceId might be different types)
                let cache_key = resource_id.content_hash().map_err(|e|
                    ValidationError::InternalError(format!("Failed to hash resource ID: {}", e))
                )?;
                
                if let Some(cached_result) = cache.get(&cache_key) {
                    return Ok(cached_result.clone());
                }
            }
        }
        
        // Create a context with the specified options
        let mut adjusted_context = context.clone();
        adjusted_context.options = options;
        
        // Run validation pipeline
        let result = self.run_validation_pipeline(&adjusted_context).await?;
        
        // Cache result if enabled
        if self.config.enable_caching && adjusted_context.options.enable_caching {
            if let Some(resource_id) = &adjusted_context.resource_id {
                if result.is_valid() {
                    let mut cache = self.validation_cache.write().map_err(|e| 
                        ValidationError::InternalError(format!("Failed to acquire validation cache lock: {}", e))
                    )?;
                    
                    // Ensure cache doesn't grow too large
                    if cache.len() >= self.config.max_cache_size {
                        cache.clear();
                    }
                    
                    // Use ContentId for cache key
                    let cache_key = resource_id.content_hash().map_err(|e|
                        ValidationError::InternalError(format!("Failed to hash resource ID: {}", e))
                    )?;
                    
                    cache.insert(cache_key, result.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    fn name(&self) -> &str {
        "ResourceValidator"
    }
}

/// Validation pipeline that orchestrates the validation process
pub struct ValidationPipeline {
    /// Main resource validator
    validator: Arc<ResourceValidator>,
    
    /// Additional validators to run in sequence
    additional_validators: Vec<Arc<dyn Validator>>,
}

impl ValidationPipeline {
    /// Create a new validation pipeline
    pub fn new(validator: Arc<ResourceValidator>) -> Self {
        Self {
            validator,
            additional_validators: Vec::new(),
        }
    }
    
    /// Add an additional validator to the pipeline
    pub fn add_validator<V: Validator + 'static>(&mut self, validator: V) {
        self.additional_validators.push(Arc::new(validator));
    }
    
    /// Validate a resource
    pub async fn validate<R: Resource + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = self.validator.validate_resource(resource).await?;
        
        // Run additional validators
        for validator in &self.additional_validators {
            let context = ValidationContext::new()
                .with_resource_id(resource.id().clone())
                .with_resource_type(resource.resource_type().clone())
                .with_current_state(resource.state().clone())
                .with_schema(resource.schema().clone())
                .with_phase(ValidationPhase::PreExecution);
                
            let additional_result = validator.validate(&context).await?;
            result.merge(additional_result);
            
            if !result.is_valid() {
                return Ok(result);
            }
        }
        
        Ok(result)
    }
    
    /// Validate using a context
    pub async fn validate_context(
        &self,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = self.validator.validate(context).await?;
        
        // Run additional validators
        for validator in &self.additional_validators {
            let additional_result = validator.validate(context).await?;
            result.merge(additional_result);
            
            if !result.is_valid() {
                return Ok(result);
            }
        }
        
        Ok(result)
    }
} 