// Schema validation module
// This file contains components for validating resources against their schemas.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use jsonschema::{JSONSchema, CompilationOptions};
use thiserror::Error;
use serde_json::Value;
use serde::{Serialize, Deserialize};

use crate::resource::{ResourceId, ResourceTypeId, ResourceSchema};
use crate::content::ContentHash;

use super::context::ValidationContext;
use super::result::{ValidationResult, ValidationIssue, ValidationError, ValidationSeverity};
use super::validation::Validator;

/// Error types specific to schema validation
#[derive(Error, Debug, Clone)]
pub enum SchemaValidationError {
    /// Invalid schema format
    #[error("Invalid schema format: {0}")]
    InvalidSchemaFormat(String),
    
    /// Schema compilation error
    #[error("Schema compilation error: {0}")]
    SchemaCompilationError(String),
    
    /// Schema validation error
    #[error("Schema validation error: {0}")]
    ValidationError(String),
    
    /// Schema compatibility error
    #[error("Schema compatibility error: {0}")]
    CompatibilityError(String),
    
    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    SerializationError(String),
    
    /// Missing required data
    #[error("Missing required data for schema validation: {0}")]
    MissingData(String),
}

/// Schema compatibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaCompatibility {
    /// Schemas are fully compatible
    Full,
    
    /// Forward compatible only (old schema can validate new data)
    Forward,
    
    /// Backward compatible only (new schema can validate old data)
    Backward,
    
    /// Schemas are incompatible
    Incompatible,
}

/// Schema validator for resources
#[derive(Debug)]
pub struct SchemaValidator {
    /// Compiled schema cache
    schema_cache: RwLock<HashMap<ContentHash, Arc<JSONSchema>>>,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self {
            schema_cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Compile a JSON schema
    fn compile_schema(&self, schema: &str) -> Result<JSONSchema, SchemaValidationError> {
        let schema_value: Value = serde_json::from_str(schema)
            .map_err(|e| SchemaValidationError::SerializationError(e.to_string()))?;
            
        let options = CompilationOptions::default();
        
        JSONSchema::options()
            .with_options(options)
            .compile(&schema_value)
            .map_err(|e| SchemaValidationError::SchemaCompilationError(e.to_string()))
    }
    
    /// Get or compile a schema
    async fn get_compiled_schema(&self, schema: &ResourceSchema) -> Result<Arc<JSONSchema>, SchemaValidationError> {
        // Check if schema format is supported
        if schema.format != "json-schema" {
            return Err(SchemaValidationError::InvalidSchemaFormat(
                format!("Unsupported schema format: {}", schema.format)
            ));
        }
        
        // Get content hash for cache lookup
        let hash = schema.content_hash.clone().ok_or_else(|| 
            SchemaValidationError::MissingData("Schema missing content hash".to_string())
        )?;
        
        // Try to get from cache first
        {
            let cache = self.schema_cache.read().map_err(|e| 
                SchemaValidationError::SerializationError(format!("Failed to acquire schema cache lock: {}", e))
            )?;
            
            if let Some(compiled) = cache.get(&hash) {
                return Ok(compiled.clone());
            }
        }
        
        // Compile the schema
        let compiled = self.compile_schema(&schema.definition)?;
        let compiled_arc = Arc::new(compiled);
        
        // Cache it
        {
            let mut cache = self.schema_cache.write().map_err(|e| 
                SchemaValidationError::SerializationError(format!("Failed to acquire schema cache lock: {}", e))
            )?;
            
            cache.insert(hash, compiled_arc.clone());
        }
        
        Ok(compiled_arc)
    }
    
    /// Validate data against a schema
    pub async fn validate_data(
        &self,
        schema: &ResourceSchema,
        data: &Value,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::success();
        
        // Get compiled schema
        let compiled_schema = match self.get_compiled_schema(schema).await {
            Ok(schema) => schema,
            Err(e) => {
                result.add_error(
                    format!("Schema compilation error: {}", e),
                    "SCHEMA_COMPILATION_ERROR",
                    "schema_validator",
                );
                return Ok(result);
            }
        };
        
        // Validate against schema
        let validation = compiled_schema.validate(data);
        
        // Check validation results
        if let Err(errors) = validation {
            for error in errors {
                let path = error.instance_path.to_string();
                let message = error.to_string();
                
                result.add_error(
                    format!("Schema validation error at {}: {}", path, message),
                    "SCHEMA_VALIDATION_ERROR",
                    "schema_validator",
                );
            }
        }
        
        Ok(result)
    }
    
    /// Check if two schemas are compatible
    pub async fn check_compatibility(
        &self,
        old_schema: &ResourceSchema,
        new_schema: &ResourceSchema,
    ) -> Result<SchemaCompatibility, SchemaValidationError> {
        // If schema formats don't match, they're incompatible
        if old_schema.format != new_schema.format {
            return Ok(SchemaCompatibility::Incompatible);
        }
        
        // Only support JSON Schema for now
        if old_schema.format != "json-schema" {
            return Err(SchemaValidationError::InvalidSchemaFormat(
                format!("Unsupported schema format for compatibility check: {}", old_schema.format)
            ));
        }
        
        // Parse schemas
        let old_value: Value = serde_json::from_str(&old_schema.definition)
            .map_err(|e| SchemaValidationError::SerializationError(e.to_string()))?;
            
        let new_value: Value = serde_json::from_str(&new_schema.definition)
            .map_err(|e| SchemaValidationError::SerializationError(e.to_string()))?;
            
        // For simple comparison, we'll check required fields and property types
        // In a real implementation, this would be much more sophisticated
        
        // Check forward compatibility
        let forward_compatible = self.check_forward_compatibility(&old_value, &new_value)?;
        
        // Check backward compatibility
        let backward_compatible = self.check_backward_compatibility(&old_value, &new_value)?;
        
        // Determine compatibility level
        match (forward_compatible, backward_compatible) {
            (true, true) => Ok(SchemaCompatibility::Full),
            (true, false) => Ok(SchemaCompatibility::Forward),
            (false, true) => Ok(SchemaCompatibility::Backward),
            (false, false) => Ok(SchemaCompatibility::Incompatible),
        }
    }
    
    /// Check if the old schema can validate data that conforms to the new schema
    fn check_forward_compatibility(
        &self,
        old_schema: &Value,
        new_schema: &Value,
    ) -> Result<bool, SchemaValidationError> {
        // Simplified check: ensure all required properties in the old schema
        // are also required in the new schema
        
        let old_required = Self::get_required_properties(old_schema)?;
        let new_required = Self::get_required_properties(new_schema)?;
        
        // All required properties in old schema must be in new schema
        for prop in &old_required {
            if !new_required.contains(prop) {
                return Ok(false);
            }
        }
        
        // Check property types are compatible
        let old_properties = Self::get_properties(old_schema)?;
        let new_properties = Self::get_properties(new_schema)?;
        
        for (name, old_type) in old_properties {
            if let Some(new_type) = new_properties.get(&name) {
                if !Self::are_types_compatible(&old_type, &new_type) {
                    return Ok(false);
                }
            } else if old_required.contains(&name) {
                // Required property is missing in new schema
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Check if the new schema can validate data that conforms to the old schema
    fn check_backward_compatibility(
        &self,
        old_schema: &Value,
        new_schema: &Value,
    ) -> Result<bool, SchemaValidationError> {
        // Simplified check: ensure all required properties in the new schema
        // are also required in the old schema
        
        let old_required = Self::get_required_properties(old_schema)?;
        let new_required = Self::get_required_properties(new_schema)?;
        
        // All required properties in new schema must be in old schema
        for prop in &new_required {
            if !old_required.contains(prop) {
                return Ok(false);
            }
        }
        
        // Check property types are compatible
        let old_properties = Self::get_properties(old_schema)?;
        let new_properties = Self::get_properties(new_schema)?;
        
        for (name, new_type) in new_properties {
            if let Some(old_type) = old_properties.get(&name) {
                if !Self::are_types_compatible(&old_type, &new_type) {
                    return Ok(false);
                }
            } else if new_required.contains(&name) {
                // Required property is missing in old schema
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get required properties from a JSON Schema
    fn get_required_properties(schema: &Value) -> Result<Vec<String>, SchemaValidationError> {
        match schema.get("required") {
            Some(Value::Array(required)) => {
                let mut result = Vec::new();
                for item in required {
                    if let Value::String(name) = item {
                        result.push(name.clone());
                    }
                }
                Ok(result)
            }
            Some(_) => Err(SchemaValidationError::InvalidSchemaFormat(
                "Invalid 'required' field format".to_string()
            )),
            None => Ok(Vec::new()),
        }
    }
    
    /// Get properties and their types from a JSON Schema
    fn get_properties(schema: &Value) -> Result<HashMap<String, String>, SchemaValidationError> {
        let mut result = HashMap::new();
        
        if let Some(Value::Object(properties)) = schema.get("properties") {
            for (name, prop) in properties {
                if let Some(Value::String(type_name)) = prop.get("type") {
                    result.insert(name.clone(), type_name.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Check if two JSON Schema types are compatible
    fn are_types_compatible(type1: &str, type2: &str) -> bool {
        // Simple compatibility check
        if type1 == type2 {
            return true;
        }
        
        // Numeric types are somewhat compatible
        if (type1 == "number" && type2 == "integer") || (type1 == "integer" && type2 == "number") {
            return true;
        }
        
        false
    }
}

#[async_trait]
impl Validator for SchemaValidator {
    async fn validate(&self, context: &ValidationContext) -> Result<ValidationResult, ValidationError> {
        // Schema validation requires both schema and data
        let schema = context.schema.as_ref().ok_or_else(|| 
            ValidationError::SchemaError("Missing schema for validation".to_string())
        )?;
        
        // Get data from context
        let data_bytes = context.context_data.get("resource_data").ok_or_else(|| 
            ValidationError::SchemaError("Missing resource data for schema validation".to_string())
        )?;
        
        // Parse JSON data
        let data: Value = serde_json::from_slice(data_bytes)
            .map_err(|e| ValidationError::SchemaError(
                format!("Failed to parse resource data as JSON: {}", e)
            ))?;
        
        // Validate data against schema
        self.validate_data(schema, &data).await
    }
    
    async fn validate_with_options(
        &self, 
        context: &ValidationContext,
        _options: super::context::ValidationOptions,
    ) -> Result<ValidationResult, ValidationError> {
        // Options don't affect schema validation for now
        self.validate(context).await
    }
    
    fn name(&self) -> &str {
        "SchemaValidator"
    }
}

/// Helper function to validate schema compatibility
pub fn validate_schema_compatibility(
    old_schema: &ResourceSchema,
    new_schema: &ResourceSchema,
) -> Result<SchemaCompatibility, SchemaValidationError> {
    let validator = SchemaValidator::new();
    
    // Use a runtime to run the async function
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| SchemaValidationError::SerializationError(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    rt.block_on(validator.check_compatibility(old_schema, new_schema))
} 