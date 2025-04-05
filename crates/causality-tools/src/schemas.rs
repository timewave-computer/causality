// Schema definitions for effect adapters
// This module handles loading and validating adapter schemas

use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

/// Represents an effect adapter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterSchema {
    /// Name of the adapter
    pub name: String,
    
    /// Version of the adapter
    pub version: String,
    
    /// Target language for code generation
    pub language: String,
    
    /// Supported effect types
    pub effects: Vec<EffectDefinition>,
    
    /// Schema metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Definition of an effect type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDefinition {
    /// Effect type name
    pub name: String,
    
    /// Function to create the effect
    pub function: String,
    
    /// Parameters for the effect
    pub parameters: Vec<Parameter>,
    
    /// Documentation for the effect
    #[serde(default)]
    pub documentation: String,
}

/// Definition of a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub type_name: String,
    
    /// Is the parameter required
    #[serde(default)]
    pub required: bool,
    
    /// Default value if not specified
    #[serde(default)]
    pub default_value: Option<String>,
    
    /// Documentation for the parameter
    #[serde(default)]
    pub documentation: String,
}

/// Load a schema from a file
pub fn load_schema(path: &Path) -> Result<AdapterSchema> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read schema file: {}", path.display()))?;
    
    let schema: AdapterSchema = serde_json::from_str(&content)
        .context(format!("Failed to parse schema file: {}", path.display()))?;
    
    Ok(schema)
}

/// Validate a schema for completeness and correctness
pub fn validate_schema(schema: &AdapterSchema) -> Result<()> {
    // Basic validation
    if schema.name.is_empty() {
        anyhow::bail!("Schema must have a name");
    }
    
    if schema.version.is_empty() {
        anyhow::bail!("Schema must have a version");
    }
    
    if schema.effects.is_empty() {
        anyhow::bail!("Schema must define at least one effect");
    }
    
    // Validate each effect
    for effect in &schema.effects {
        if effect.name.is_empty() {
            anyhow::bail!("Effect must have a name");
        }
        
        if effect.function.is_empty() {
            anyhow::bail!("Effect must have a function name");
        }
        
        // For now, don't require parameters
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_validation() {
        let schema = AdapterSchema {
            name: "test".to_string(),
            version: "1.0".to_string(),
            language: "rust".to_string(),
            effects: vec![
                EffectDefinition {
                    name: "transfer".to_string(),
                    function: "transfer".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "from".to_string(),
                            type_name: "String".to_string(),
                            required: true,
                            default_value: None,
                            documentation: "Source address".to_string(),
                        },
                        Parameter {
                            name: "to".to_string(),
                            type_name: "String".to_string(),
                            required: true,
                            default_value: None,
                            documentation: "Destination address".to_string(),
                        },
                    ],
                    documentation: "Transfer assets".to_string(),
                }
            ],
            metadata: std::collections::HashMap::new(),
        };
        
        let result = validate_schema(&schema);
        assert!(result.is_ok());
    }
} 