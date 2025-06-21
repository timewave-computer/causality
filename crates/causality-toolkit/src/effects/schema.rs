//! Effect schema system for automatic mock and test generation

use crate::effects::core::{AlgebraicEffect, EffectCategory, FailureMode};
use causality_core::system::content_addressing::{ContentAddressable, EntityId};
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Definition of a parameter in an effect schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Parameter name
    pub name: String,
    /// Parameter type definition
    pub param_type: TypeDef,
    /// Whether this parameter is optional
    pub optional: bool,
    /// Human-readable description
    pub description: Option<String>,
}

/// Type definition for effect parameters and return types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeDef {
    /// Unit type ()
    Unit,
    /// Boolean type
    Bool,
    /// Unsigned integer with bit width
    UInt(u8),
    /// Signed integer with bit width  
    SInt(u8),
    /// String type
    String,
    /// Address type (for blockchain addresses)
    Address,
    /// EntityId type (content-addressed identifier)
    EntityId,
    /// Timestamp type
    Timestamp,
    /// Option type
    Option(Box<TypeDef>),
    /// Array type with element type
    Array(Box<TypeDef>),
    /// Tuple type with element types
    Tuple(Vec<TypeDef>),
    /// Custom type with name
    Custom(String),
}

/// Metadata about an effect for automatic processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectMetadata {
    /// Category of this effect
    pub category: EffectCategory,
    /// Expected execution duration
    pub expected_duration: Duration,
    /// Common failure modes
    pub failure_modes: Vec<FailureMode>,
    /// Whether effect can be executed in parallel
    pub parallelizable: bool,
    /// Whether effect has side effects
    pub has_side_effects: bool,
    /// Computational cost estimate
    pub computational_cost: u32,
    /// Gas cost estimate for blockchain operations
    pub gas_cost: u64,
}

/// Complete schema definition for an effect type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSchema {
    /// Content-addressed identifier for this schema
    pub effect_id: EntityId,
    /// Human-readable name
    pub name: String,
    /// Input parameter definitions
    pub parameters: Vec<ParameterDef>,
    /// Return type definition
    pub returns: TypeDef,
    /// Error type definition
    pub error_type: TypeDef,
    /// Effect metadata
    pub metadata: EffectMetadata,
    /// Schema version for compatibility
    pub schema_version: String,
}

impl EffectSchema {
    /// Generate schema from an AlgebraicEffect type
    /// 
    /// This is a simplified implementation that creates basic schemas.
    /// In a full implementation, this would use reflection or macros to
    /// automatically derive parameter definitions from the effect struct.
    pub fn from_effect<T: AlgebraicEffect>() -> Self {
        let name = T::effect_name().to_string();
        
        // Create a simple schema with basic metadata
        // In a full implementation, this would inspect T's fields automatically
        let schema = EffectSchema {
            effect_id: Self::generate_schema_id(&name),
            name,
            parameters: Vec::new(), // Would be populated by reflection/macros
            returns: TypeDef::Custom("Result".to_string()), // Would be derived from T::Result
            error_type: TypeDef::Custom("Error".to_string()), // Would be derived from T::Error
            metadata: EffectMetadata {
                category: T::effect_category(),
                expected_duration: T::expected_duration(),
                failure_modes: T::failure_modes(),
                parallelizable: T::is_parallelizable(),
                has_side_effects: T::has_side_effects(),
                computational_cost: T::computational_cost(),
                gas_cost: T::gas_cost(),
            },
            schema_version: "0.1.0".to_string(),
        };
        
        schema
    }
    
    /// Create a manual schema with specified parameters
    pub fn new(
        name: String,
        parameters: Vec<ParameterDef>,
        returns: TypeDef,
        error_type: TypeDef,
        metadata: EffectMetadata,
    ) -> Self {
        EffectSchema {
            effect_id: Self::generate_schema_id(&name),
            name,
            parameters,
            returns,
            error_type,
            metadata,
            schema_version: "0.1.0".to_string(),
        }
    }
    
    /// Generate a deterministic schema ID from the effect name
    fn generate_schema_id(name: &str) -> EntityId {
        // Create a simple deterministic ID from the name
        let mut bytes = [0u8; 32];
        let name_bytes = name.as_bytes();
        let copy_len = std::cmp::min(name_bytes.len(), 32);
        bytes[0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
        EntityId::from_bytes(bytes)
    }
    
    /// Get parameter definition by name
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterDef> {
        self.parameters.iter().find(|p| p.name == name)
    }
    
    /// Check if schema has a parameter with given name
    pub fn has_parameter(&self, name: &str) -> bool {
        self.get_parameter(name).is_some()
    }
    
    /// Get required parameters (non-optional)
    pub fn required_parameters(&self) -> Vec<&ParameterDef> {
        self.parameters.iter().filter(|p| !p.optional).collect()
    }
    
    /// Get optional parameters
    pub fn optional_parameters(&self) -> Vec<&ParameterDef> {
        self.parameters.iter().filter(|p| p.optional).collect()
    }
    
    /// Validate that this schema is well-formed
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Check for duplicate parameter names
        let mut param_names = std::collections::BTreeSet::new();
        for param in &self.parameters {
            if !param_names.insert(&param.name) {
                return Err(SchemaError::DuplicateParameter(param.name.clone()));
            }
        }
        
        // Check that name is not empty
        if self.name.is_empty() {
            return Err(SchemaError::EmptyName);
        }
        
        Ok(())
    }
}

impl ContentAddressable for EffectSchema {
    fn content_id(&self) -> EntityId {
        self.effect_id
    }
}

/// Errors that can occur during schema operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum SchemaError {
    #[error("Duplicate parameter name: {0}")]
    DuplicateParameter(String),
    
    #[error("Effect name cannot be empty")]
    EmptyName,
    
    #[error("Invalid parameter type: {0}")]
    InvalidParameterType(String),
    
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),
}

impl ParameterDef {
    /// Create a new parameter definition
    pub fn new(name: String, param_type: TypeDef) -> Self {
        ParameterDef {
            name,
            param_type,
            optional: false,
            description: None,
        }
    }
    
    /// Make this parameter optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Add a description to this parameter
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl TypeDef {
    /// Check if this type represents a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeDef::UInt(_) | TypeDef::SInt(_))
    }
    
    /// Check if this type represents a string-like type
    pub fn is_string_like(&self) -> bool {
        matches!(self, TypeDef::String | TypeDef::Address)
    }
    
    /// Check if this type is optional
    pub fn is_optional(&self) -> bool {
        matches!(self, TypeDef::Option(_))
    }
    
    /// Get the inner type for Option types
    pub fn inner_type(&self) -> Option<&TypeDef> {
        match self {
            TypeDef::Option(inner) => Some(inner),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::core::{EffectCategory, FailureMode};
    
    // Test effect for schema generation
    #[derive(Debug, Clone)]
    struct TestEffect {
        pub value: u32,
    }
    
    impl ContentAddressable for TestEffect {
        fn content_id(&self) -> EntityId {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&self.value.to_le_bytes());
            EntityId::from_bytes(bytes)
        }
    }
    
    impl AlgebraicEffect for TestEffect {
        type Result = u32;
        type Error = String;
        
        fn effect_name() -> &'static str { "test_effect" }
        fn effect_category() -> EffectCategory { EffectCategory::Compute }
        fn expected_duration() -> Duration { Duration::from_millis(10) }
        fn failure_modes() -> Vec<FailureMode> {
            vec![FailureMode::ComputationFailed]
        }
    }
    
    #[test]
    fn test_schema_generation() {
        let schema = EffectSchema::from_effect::<TestEffect>();
        
        assert_eq!(schema.name, "test_effect");
        assert_eq!(schema.metadata.category, EffectCategory::Compute);
        assert_eq!(schema.metadata.expected_duration, Duration::from_millis(10));
        assert_eq!(schema.metadata.failure_modes, vec![FailureMode::ComputationFailed]);
        assert!(schema.metadata.parallelizable);
        assert!(schema.metadata.has_side_effects);
        assert_eq!(schema.metadata.computational_cost, 1);
        assert_eq!(schema.metadata.gas_cost, 0);
        assert_eq!(schema.schema_version, "0.1.0");
    }
    
    #[test]
    fn test_parameter_def() {
        let param = ParameterDef::new("amount".to_string(), TypeDef::UInt(64))
            .optional()
            .with_description("Transfer amount".to_string());
            
        assert_eq!(param.name, "amount");
        assert_eq!(param.param_type, TypeDef::UInt(64));
        assert!(param.optional);
        assert_eq!(param.description, Some("Transfer amount".to_string()));
    }
    
    #[test]
    fn test_type_def_helpers() {
        assert!(TypeDef::UInt(64).is_numeric());
        assert!(TypeDef::SInt(32).is_numeric());
        assert!(!TypeDef::String.is_numeric());
        
        assert!(TypeDef::String.is_string_like());
        assert!(TypeDef::Address.is_string_like());
        assert!(!TypeDef::Bool.is_string_like());
        
        let opt_type = TypeDef::Option(Box::new(TypeDef::String));
        assert!(opt_type.is_optional());
        assert_eq!(opt_type.inner_type(), Some(&TypeDef::String));
    }
    
    #[test]
    fn test_schema_validation() {
        // Valid schema
        let metadata = EffectMetadata {
            category: EffectCategory::Asset,
            expected_duration: Duration::from_millis(100),
            failure_modes: vec![FailureMode::InsufficientBalance],
            parallelizable: true,
            has_side_effects: true,
            computational_cost: 1,
            gas_cost: 0,
        };
        
        let schema = EffectSchema::new(
            "transfer".to_string(),
            vec![
                ParameterDef::new("from".to_string(), TypeDef::Address),
                ParameterDef::new("to".to_string(), TypeDef::Address),
            ],
            TypeDef::Bool,
            TypeDef::String,
            metadata.clone(),
        );
        
        assert!(schema.validate().is_ok());
        
        // Schema with duplicate parameters
        let invalid_schema = EffectSchema::new(
            "transfer".to_string(),
            vec![
                ParameterDef::new("from".to_string(), TypeDef::Address),
                ParameterDef::new("from".to_string(), TypeDef::Address), // Duplicate
            ],
            TypeDef::Bool,
            TypeDef::String,
            metadata,
        );
        
        assert!(matches!(invalid_schema.validate(), Err(SchemaError::DuplicateParameter(_))));
    }
    
    #[test]
    fn test_schema_parameter_queries() {
        let metadata = EffectMetadata {
            category: EffectCategory::Asset,
            expected_duration: Duration::from_millis(100),
            failure_modes: vec![FailureMode::InsufficientBalance],
            parallelizable: true,
            has_side_effects: true,
            computational_cost: 1,
            gas_cost: 0,
        };
        
        let schema = EffectSchema::new(
            "transfer".to_string(),
            vec![
                ParameterDef::new("from".to_string(), TypeDef::Address),
                ParameterDef::new("to".to_string(), TypeDef::Address),
                ParameterDef::new("memo".to_string(), TypeDef::String).optional(),
            ],
            TypeDef::Bool,
            TypeDef::String,
            metadata,
        );
        
        assert!(schema.has_parameter("from"));
        assert!(schema.has_parameter("to"));
        assert!(schema.has_parameter("memo"));
        assert!(!schema.has_parameter("amount"));
        
        assert_eq!(schema.required_parameters().len(), 2);
        assert_eq!(schema.optional_parameters().len(), 1);
        
        let from_param = schema.get_parameter("from").unwrap();
        assert_eq!(from_param.name, "from");
        assert_eq!(from_param.param_type, TypeDef::Address);
    }
    
    #[test]
    fn test_content_addressing() {
        let schema1 = EffectSchema::from_effect::<TestEffect>();
        let schema2 = EffectSchema::from_effect::<TestEffect>();
        
        // Same effect type should generate same schema ID
        assert_eq!(schema1.content_id(), schema2.content_id());
    }
} 