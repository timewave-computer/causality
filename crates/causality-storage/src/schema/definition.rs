// Schema definition utilities
// Original file: src/schema/definition.rs

//! Schema Definition
//!
//! This module provides the core schema definition structures.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use semver::Version;

use crate::schema::{Error, Result};

/// Schema version representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(Version);

impl SchemaVersion {
    /// Create a new schema version from a string
    pub fn new(version_str: &str) -> Result<Self> {
        match Version::parse(version_str) {
            Ok(version) => Ok(SchemaVersion(version)),
            Err(_) => Err(Error::InvalidVersion(version_str.to_string())),
        }
    }
    
    /// Get the version string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
    
    /// Get the major version
    pub fn major(&self) -> u64 {
        self.0.major
    }
    
    /// Get the minor version
    pub fn minor(&self) -> u64 {
        self.0.minor
    }
    
    /// Get the patch version
    pub fn patch(&self) -> u64 {
        self.0.patch
    }
    
    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        self.0.major == other.0.major
    }
    
    /// Increment the major version
    pub fn increment_major(&self) -> Self {
        let mut version = self.0.clone();
        version.major += 1;
        version.minor = 0;
        version.patch = 0;
        SchemaVersion(version)
    }
    
    /// Increment the minor version
    pub fn increment_minor(&self) -> Self {
        let mut version = self.0.clone();
        version.minor += 1;
        version.patch = 0;
        SchemaVersion(version)
    }
    
    /// Increment the patch version
    pub fn increment_patch(&self) -> Self {
        let mut version = self.0.clone();
        version.patch += 1;
        SchemaVersion(version)
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Schema field type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SchemaType {
    /// String type
    String,
    /// Integer type
    Integer,
    /// Floating point type
    Float,
    /// Boolean type
    Boolean,
    /// Array type with element type
    Array(Box<SchemaType>),
    /// Map type with key and value types
    Map(Box<SchemaType>, Box<SchemaType>),
    /// Object type with field definitions
    Object(HashMap<String, SchemaField>),
    /// Enum type with variants
    Enum(Vec<String>),
    /// Any type (dynamically typed)
    Any,
}

impl SchemaType {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &SchemaType) -> bool {
        match (self, other) {
            // Same types are compatible
            (SchemaType::String, SchemaType::String) |
            (SchemaType::Boolean, SchemaType::Boolean) |
            (SchemaType::Any, _) |
            (_, SchemaType::Any) => true,
            
            // Integer can be coerced to float
            (SchemaType::Integer, SchemaType::Float) => true,
            
            // Arrays are compatible if their element types are compatible
            (SchemaType::Array(a), SchemaType::Array(b)) => a.is_compatible_with(b),
            
            // Maps are compatible if both key and value types are compatible
            (SchemaType::Map(ak, av), SchemaType::Map(bk, bv)) => {
                ak.is_compatible_with(bk) && av.is_compatible_with(bv)
            }
            
            // Objects are compatible if all required fields in b are compatible in a
            (SchemaType::Object(a_fields), SchemaType::Object(b_fields)) => {
                for (field_name, field_def) in b_fields {
                    // Skip optional fields in b
                    if field_def.required {
                        match a_fields.get(field_name) {
                            Some(a_field) => {
                                if !a_field.field_type.is_compatible_with(&field_def.field_type) {
                                    return false;
                                }
                            }
                            None => return false,
                        }
                    }
                }
                true
            }
            
            // Enums are compatible if all variants in b are in a
            (SchemaType::Enum(a_variants), SchemaType::Enum(b_variants)) => {
                let a_set: HashSet<_> = a_variants.iter().collect();
                b_variants.iter().all(|v| a_set.contains(v))
            }
            
            // All other combinations are incompatible
            _ => false,
        }
    }
    
    /// Get the string representation of this type
    pub fn type_name(&self) -> String {
        match self {
            SchemaType::String => "string".to_string(),
            SchemaType::Integer => "integer".to_string(),
            SchemaType::Float => "float".to_string(),
            SchemaType::Boolean => "boolean".to_string(),
            SchemaType::Array(elem_type) => format!("array<{}>", elem_type.type_name()),
            SchemaType::Map(key_type, value_type) => {
                format!("map<{}, {}>", key_type.type_name(), value_type.type_name())
            }
            SchemaType::Object(_) => "object".to_string(),
            SchemaType::Enum(_) => "enum".to_string(),
            SchemaType::Any => "any".to_string(),
        }
    }
}

/// Schema field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: SchemaType,
    /// Whether the field is required
    pub required: bool,
    /// Default value for the field (if any)
    pub default_value: Option<serde_json::Value>,
    /// Field description
    pub description: Option<String>,
    /// Field metadata
    pub metadata: HashMap<String, String>,
}

impl SchemaField {
    /// Create a new schema field
    pub fn new<S: Into<String>>(name: S, field_type: SchemaType, required: bool) -> Self {
        SchemaField {
            name: name.into(),
            field_type,
            required,
            default_value: None,
            description: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set a default value for the field
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default_value = Some(value);
        self
    }
    
    /// Set a description for the field
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Add metadata to the field
    pub fn with_metadata<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Check if this field is compatible with another field
    pub fn is_compatible_with(&self, other: &SchemaField) -> bool {
        // If other is required, this must be required too
        if other.required && !self.required {
            return false;
        }
        
        // Check if the types are compatible
        self.field_type.is_compatible_with(&other.field_type)
    }
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: SchemaVersion,
    /// Root schema fields
    pub fields: HashMap<String, SchemaField>,
    /// Allowed evolution rules
    pub allowed_evolution: Option<HashSet<String>>,
    /// Schema description
    pub description: Option<String>,
    /// Schema metadata
    pub metadata: HashMap<String, String>,
}

impl Schema {
    /// Create a new schema
    pub fn new<S: Into<String>>(name: S, version_str: &str) -> Result<Self> {
        let version = SchemaVersion::new(version_str)?;
        
        Ok(Schema {
            name: name.into(),
            version,
            fields: HashMap::new(),
            allowed_evolution: None,
            description: None,
            metadata: HashMap::new(),
        })
    }
    
    /// Add a field to the schema
    pub fn add_field(&mut self, field: SchemaField) -> &mut Self {
        self.fields.insert(field.name.clone(), field);
        self
    }
    
    /// Set allowed evolution rules
    pub fn with_allowed_evolution(&mut self, rules: HashSet<String>) -> &mut Self {
        self.allowed_evolution = Some(rules);
        self
    }
    
    /// Set a description for the schema
    pub fn with_description<S: Into<String>>(&mut self, description: S) -> &mut Self {
        self.description = Some(description.into());
        self
    }
    
    /// Add metadata to the schema
    pub fn with_metadata<S: Into<String>>(&mut self, key: S, value: S) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Check if this schema is compatible with another schema
    pub fn is_compatible_with(&self, other: &Schema) -> bool {
        // Check all required fields in other schema
        for (field_name, field_def) in &other.fields {
            if field_def.required {
                match self.fields.get(field_name) {
                    Some(self_field) => {
                        if !self_field.is_compatible_with(field_def) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
        }
        true
    }
    
    /// Convert the schema to JSON
    pub fn to_json(&self) -> Result<String> {
        match serde_json::to_string_pretty(self) {
            Ok(json) => Ok(json),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }
    
    /// Load a schema from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        match serde_json::from_str(json) {
            Ok(schema) => Ok(schema),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }
    
    /// Convert the schema to TOML
    pub fn to_toml(&self) -> Result<String> {
        match toml::to_string_pretty(self) {
            Ok(toml) => Ok(toml),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }
    
    /// Load a schema from TOML
    pub fn from_toml(toml: &str) -> Result<Self> {
        match toml::from_str(toml) {
            Ok(schema) => Ok(schema),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }
    
    /// Increment the schema version
    pub fn increment_version(&mut self, major: bool, minor: bool, patch: bool) -> &mut Self {
        let new_version = if major {
            self.version.increment_major()
        } else if minor {
            self.version.increment_minor()
        } else if patch {
            self.version.increment_patch()
        } else {
            // No change
            return self;
        };
        
        self.version = new_version;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_schema_version() {
        let version = SchemaVersion::new("1.2.3").unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
        
        let incremented = version.increment_minor();
        assert_eq!(incremented.as_str(), "1.3.0");
        
        assert!(version.is_compatible_with(&incremented));
        
        let major_bump = version.increment_major();
        assert_eq!(major_bump.as_str(), "2.0.0");
        assert!(!version.is_compatible_with(&major_bump));
    }
    
    #[test]
    fn test_schema_type_compatibility() {
        // Same types
        assert!(SchemaType::String.is_compatible_with(&SchemaType::String));
        assert!(SchemaType::Integer.is_compatible_with(&SchemaType::Integer));
        
        // Integer to float coercion
        assert!(SchemaType::Integer.is_compatible_with(&SchemaType::Float));
        assert!(!SchemaType::Float.is_compatible_with(&SchemaType::Integer));
        
        // Array compatibility
        let string_array = SchemaType::Array(Box::new(SchemaType::String));
        let int_array = SchemaType::Array(Box::new(SchemaType::Integer));
        
        assert!(string_array.is_compatible_with(&string_array));
        assert!(!string_array.is_compatible_with(&int_array));
        
        // Map compatibility
        let string_string_map = SchemaType::Map(
            Box::new(SchemaType::String),
            Box::new(SchemaType::String)
        );
        let string_int_map = SchemaType::Map(
            Box::new(SchemaType::String),
            Box::new(SchemaType::Integer)
        );
        
        assert!(string_string_map.is_compatible_with(&string_string_map));
        assert!(!string_string_map.is_compatible_with(&string_int_map));
        
        // Any type compatibility
        assert!(SchemaType::Any.is_compatible_with(&SchemaType::String));
        assert!(SchemaType::String.is_compatible_with(&SchemaType::Any));
        
        // Enum compatibility
        let enum_a = SchemaType::Enum(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let enum_b = SchemaType::Enum(vec!["a".to_string(), "b".to_string()]);
        
        assert!(enum_a.is_compatible_with(&enum_b));
        assert!(!enum_b.is_compatible_with(&enum_a));
    }
    
    #[test]
    fn test_schema_field() {
        let field = SchemaField::new("name", SchemaType::String, true)
            .with_description("A name field")
            .with_default(json!("default"))
            .with_metadata("indexed", "true");
        
        assert_eq!(field.name, "name");
        assert_eq!(field.required, true);
        assert_eq!(field.default_value, Some(json!("default")));
        assert_eq!(field.description, Some("A name field".to_string()));
        assert_eq!(field.metadata.get("indexed"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_schema_creation() {
        let mut schema = Schema::new("TestSchema", "1.0.0").unwrap();
        
        schema.add_field(SchemaField::new("name", SchemaType::String, true));
        schema.add_field(SchemaField::new("age", SchemaType::Integer, false));
        
        let mut allowed_rules = HashSet::new();
        allowed_rules.insert("add-optional-field".to_string());
        allowed_rules.insert("remove-unused-field".to_string());
        
        schema.with_allowed_evolution(allowed_rules);
        schema.with_description("Test schema");
        schema.with_metadata("author", "Test Author");
        
        assert_eq!(schema.name, "TestSchema");
        assert_eq!(schema.version.as_str(), "1.0.0");
        assert_eq!(schema.fields.len(), 2);
        assert!(schema.fields.contains_key("name"));
        assert!(schema.fields.contains_key("age"));
        assert_eq!(schema.description, Some("Test schema".to_string()));
        assert_eq!(schema.metadata.get("author"), Some(&"Test Author".to_string()));
        
        // Check allowed rules
        if let Some(rules) = &schema.allowed_evolution {
            assert!(rules.contains("add-optional-field"));
            assert!(rules.contains("remove-unused-field"));
        } else {
            panic!("Allowed rules should be set");
        }
    }
    
    #[test]
    fn test_schema_serialization() {
        let mut schema = Schema::new("TestSchema", "1.0.0").unwrap();
        schema.add_field(SchemaField::new("name", SchemaType::String, true));
        schema.add_field(SchemaField::new("age", SchemaType::Integer, false));
        
        // Test JSON serialization
        let json = schema.to_json().unwrap();
        let deserialized = Schema::from_json(&json).unwrap();
        
        assert_eq!(deserialized.name, schema.name);
        assert_eq!(deserialized.version.as_str(), schema.version.as_str());
        assert_eq!(deserialized.fields.len(), schema.fields.len());
        
        // Test TOML serialization
        let toml = schema.to_toml().unwrap();
        let deserialized = Schema::from_toml(&toml).unwrap();
        
        assert_eq!(deserialized.name, schema.name);
        assert_eq!(deserialized.version.as_str(), schema.version.as_str());
        assert_eq!(deserialized.fields.len(), schema.fields.len());
    }
} 