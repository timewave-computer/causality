//! Schema Evolution
//!
//! This module provides the schema evolution system that allows for
//! automatic schema changes following predefined rules.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::schema::{Error, Result, Schema, SchemaField, SchemaType, SchemaVersion};

/// Types of schema changes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeType {
    /// Add a new optional field
    AddOptionalField,
    /// Add a new field with a default value
    AddDefaultField,
    /// Remove an unused field
    RemoveUnusedField,
    /// Change a field type (coercible)
    ChangeFieldType,
    /// Rename a field
    RenameField,
    /// Add a new enum variant
    AddEnumVariant,
    /// Remove an enum variant
    RemoveEnumVariant,
    /// Change a field to optional
    MakeFieldOptional,
    /// Change a field description
    ChangeFieldDescription,
    /// Add or update field metadata
    UpdateFieldMetadata,
    /// Custom evolution rule
    Custom(String),
}

impl ChangeType {
    /// Get the string representation of the change type
    pub fn as_str(&self) -> String {
        match self {
            ChangeType::AddOptionalField => "add-optional-field".to_string(),
            ChangeType::AddDefaultField => "add-default-field".to_string(),
            ChangeType::RemoveUnusedField => "remove-unused-field".to_string(),
            ChangeType::ChangeFieldType => "change-field-type".to_string(),
            ChangeType::RenameField => "rename-field".to_string(),
            ChangeType::AddEnumVariant => "add-enum-variant".to_string(),
            ChangeType::RemoveEnumVariant => "remove-enum-variant".to_string(),
            ChangeType::MakeFieldOptional => "make-field-optional".to_string(),
            ChangeType::ChangeFieldDescription => "change-field-description".to_string(),
            ChangeType::UpdateFieldMetadata => "update-field-metadata".to_string(),
            ChangeType::Custom(name) => format!("custom-{}", name),
        }
    }
    
    /// Get the change type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "add-optional-field" => Some(ChangeType::AddOptionalField),
            "add-default-field" => Some(ChangeType::AddDefaultField),
            "remove-unused-field" => Some(ChangeType::RemoveUnusedField),
            "change-field-type" => Some(ChangeType::ChangeFieldType),
            "rename-field" => Some(ChangeType::RenameField),
            "add-enum-variant" => Some(ChangeType::AddEnumVariant),
            "remove-enum-variant" => Some(ChangeType::RemoveEnumVariant),
            "make-field-optional" => Some(ChangeType::MakeFieldOptional),
            "change-field-description" => Some(ChangeType::ChangeFieldDescription),
            "update-field-metadata" => Some(ChangeType::UpdateFieldMetadata),
            s if s.starts_with("custom-") => {
                let name = s.strip_prefix("custom-")?.to_string();
                Some(ChangeType::Custom(name))
            },
            _ => None,
        }
    }
    
    /// Check if this change type is safe (doesn't break existing code)
    pub fn is_safe(&self) -> bool {
        match self {
            ChangeType::AddOptionalField |
            ChangeType::AddDefaultField |
            ChangeType::RemoveUnusedField |
            ChangeType::AddEnumVariant |
            ChangeType::MakeFieldOptional |
            ChangeType::ChangeFieldDescription |
            ChangeType::UpdateFieldMetadata => true,
            
            // These changes require more careful examination
            ChangeType::ChangeFieldType |
            ChangeType::RenameField |
            ChangeType::RemoveEnumVariant |
            ChangeType::Custom(_) => false,
        }
    }
    
    /// Check if this change type requires data migration
    pub fn requires_migration(&self) -> bool {
        match self {
            ChangeType::RenameField |
            ChangeType::RemoveEnumVariant |
            ChangeType::Custom(_) => true,
            
            ChangeType::ChangeFieldType => true, // May require migration depending on the types
            
            ChangeType::AddOptionalField |
            ChangeType::AddDefaultField |
            ChangeType::RemoveUnusedField |
            ChangeType::AddEnumVariant |
            ChangeType::MakeFieldOptional |
            ChangeType::ChangeFieldDescription |
            ChangeType::UpdateFieldMetadata => false,
        }
    }
}

/// A schema change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// The type of change
    pub change_type: ChangeType,
    /// The field name affected by the change
    pub field_name: String,
    /// The new field name (for rename operations)
    pub new_field_name: Option<String>,
    /// The new field type (for type change operations)
    pub new_field_type: Option<SchemaType>,
    /// The default value (for add with default operations)
    pub default_value: Option<Value>,
    /// Additional parameters for the change
    pub params: HashMap<String, Value>,
}

impl SchemaChange {
    /// Create a new schema change
    pub fn new(change_type: ChangeType, field_name: impl Into<String>) -> Self {
        SchemaChange {
            change_type,
            field_name: field_name.into(),
            new_field_name: None,
            new_field_type: None,
            default_value: None,
            params: HashMap::new(),
        }
    }
    
    /// Set the new field name for a rename operation
    pub fn with_new_name(mut self, new_name: impl Into<String>) -> Self {
        self.new_field_name = Some(new_name.into());
        self
    }
    
    /// Set the new field type for a type change operation
    pub fn with_new_type(mut self, new_type: SchemaType) -> Self {
        self.new_field_type = Some(new_type);
        self
    }
    
    /// Set the default value for an add with default operation
    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
    
    /// Add a parameter for the change
    pub fn with_param(mut self, key: impl Into<String>, value: Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }
    
    /// Check if this change is allowed by the given evolution rules
    pub fn is_allowed_by(&self, rules: &EvolutionRules) -> bool {
        rules.allows(&self.change_type)
    }
    
    /// Apply this change to a schema
    pub fn apply(&self, schema: &mut Schema) -> Result<()> {
        match self.change_type {
            ChangeType::AddOptionalField => {
                // Check if field already exists
                if schema.fields.contains_key(&self.field_name) {
                    return Err(Error::Evolution(format!(
                        "Field '{}' already exists", self.field_name
                    )));
                }
                
                // Get field type
                let field_type = self.new_field_type.clone()
                    .ok_or_else(|| Error::Evolution(
                        "Missing field type for AddOptionalField".to_string()
                    ))?;
                
                // Add the field
                let field = SchemaField::new(self.field_name.clone(), field_type, false);
                schema.add_field(field);
                
                Ok(())
            },
            ChangeType::AddDefaultField => {
                // Check if field already exists
                if schema.fields.contains_key(&self.field_name) {
                    return Err(Error::Evolution(format!(
                        "Field '{}' already exists", self.field_name
                    )));
                }
                
                // Get field type
                let field_type = self.new_field_type.clone()
                    .ok_or_else(|| Error::Evolution(
                        "Missing field type for AddDefaultField".to_string()
                    ))?;
                
                // Get default value
                let default_value = self.default_value.clone()
                    .ok_or_else(|| Error::Evolution(
                        "Missing default value for AddDefaultField".to_string()
                    ))?;
                
                // Add the field
                let field = SchemaField::new(self.field_name.clone(), field_type, true)
                    .with_default(default_value);
                
                schema.add_field(field);
                
                Ok(())
            },
            ChangeType::RemoveUnusedField => {
                // Check if field exists
                if !schema.fields.contains_key(&self.field_name) {
                    return Err(Error::Evolution(format!(
                        "Field '{}' doesn't exist", self.field_name
                    )));
                }
                
                // Remove the field
                schema.fields.remove(&self.field_name);
                
                Ok(())
            },
            ChangeType::MakeFieldOptional => {
                // Check if field exists
                let field = schema.fields.get_mut(&self.field_name)
                    .ok_or_else(|| Error::Evolution(format!(
                        "Field '{}' doesn't exist", self.field_name
                    )))?;
                
                // Make the field optional
                field.required = false;
                
                Ok(())
            },
            ChangeType::ChangeFieldDescription => {
                // Check if field exists
                let field = schema.fields.get_mut(&self.field_name)
                    .ok_or_else(|| Error::Evolution(format!(
                        "Field '{}' doesn't exist", self.field_name
                    )))?;
                
                // Get description
                if let Some(value) = self.params.get("description") {
                    if let Some(description) = value.as_str() {
                        field.description = Some(description.to_string());
                        Ok(())
                    } else {
                        Err(Error::Evolution("Description must be a string".to_string()))
                    }
                } else {
                    Err(Error::Evolution("Missing description parameter".to_string()))
                }
            },
            ChangeType::UpdateFieldMetadata => {
                // Check if field exists
                let field = schema.fields.get_mut(&self.field_name)
                    .ok_or_else(|| Error::Evolution(format!(
                        "Field '{}' doesn't exist", self.field_name
                    )))?;
                
                // Get metadata key and value
                if let (Some(key), Some(value)) = (
                    self.params.get("key").and_then(|v| v.as_str()),
                    self.params.get("value").and_then(|v| v.as_str()),
                ) {
                    field.metadata.insert(key.to_string(), value.to_string());
                    Ok(())
                } else {
                    Err(Error::Evolution("Missing metadata key or value parameters".to_string()))
                }
            },
            // For more complex changes that require special handling
            ChangeType::ChangeFieldType |
            ChangeType::RenameField |
            ChangeType::AddEnumVariant |
            ChangeType::RemoveEnumVariant |
            ChangeType::Custom(_) => {
                Err(Error::Evolution(format!(
                    "Change type {:?} requires custom migration", self.change_type
                )))
            }
        }
    }
}

/// A rule for schema evolution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvolutionRule {
    /// The type of change
    pub change_type: ChangeType,
    /// Additional constraints on the rule
    pub constraints: Option<HashMap<String, String>>,
}

impl EvolutionRule {
    /// Create a new evolution rule
    pub fn new(change_type: ChangeType) -> Self {
        EvolutionRule {
            change_type,
            constraints: None,
        }
    }
    
    /// Add a constraint to the rule
    pub fn with_constraint(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let constraints = self.constraints.get_or_insert_with(HashMap::new);
        constraints.insert(key.into(), value.into());
        self
    }
    
    /// Check if the rule matches a change type
    pub fn matches(&self, change_type: &ChangeType) -> bool {
        &self.change_type == change_type
    }
}

/// A set of evolution rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRules {
    /// The set of allowed evolution rules
    pub rules: HashSet<EvolutionRule>,
    /// Whether to allow safe changes by default
    pub allow_safe_by_default: bool,
}

impl EvolutionRules {
    /// Create a new set of evolution rules
    pub fn new() -> Self {
        EvolutionRules {
            rules: HashSet::new(),
            allow_safe_by_default: true,
        }
    }
    
    /// Add a rule to the set
    pub fn add_rule(&mut self, rule: EvolutionRule) -> &mut Self {
        self.rules.insert(rule);
        self
    }
    
    /// Set whether to allow safe changes by default
    pub fn with_allow_safe_by_default(&mut self, allow: bool) -> &mut Self {
        self.allow_safe_by_default = allow;
        self
    }
    
    /// Check if a change type is allowed by the rules
    pub fn allows(&self, change_type: &ChangeType) -> bool {
        // Check if there's a specific rule for this change type
        for rule in &self.rules {
            if rule.matches(change_type) {
                return true;
            }
        }
        
        // If no specific rule, fall back to default behavior
        self.allow_safe_by_default && change_type.is_safe()
    }
    
    /// Add multiple rules from string identifiers
    pub fn add_rules_from_strings(&mut self, rule_names: &[String]) -> Result<&mut Self> {
        for name in rule_names {
            if let Some(change_type) = ChangeType::from_str(name) {
                self.add_rule(EvolutionRule::new(change_type));
            } else {
                return Err(Error::Evolution(format!("Unknown rule: {}", name)));
            }
        }
        Ok(self)
    }
    
    /// Create evolution rules from a schema's allowed_evolution field
    pub fn from_schema(schema: &Schema) -> Result<Self> {
        let mut rules = EvolutionRules::new();
        
        if let Some(allowed) = &schema.allowed_evolution {
            rules.add_rules_from_strings(&allowed.iter().cloned().collect::<Vec<_>>())?;
        }
        
        Ok(rules)
    }
}

impl Default for EvolutionRules {
    fn default() -> Self {
        let mut rules = EvolutionRules::new();
        
        // Add standard safe rules
        rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField))
             .add_rule(EvolutionRule::new(ChangeType::AddDefaultField))
             .add_rule(EvolutionRule::new(ChangeType::RemoveUnusedField))
             .add_rule(EvolutionRule::new(ChangeType::MakeFieldOptional))
             .add_rule(EvolutionRule::new(ChangeType::ChangeFieldDescription))
             .add_rule(EvolutionRule::new(ChangeType::UpdateFieldMetadata));
        
        rules
    }
}

/// Apply a list of changes to a schema
pub fn apply_changes(schema: &mut Schema, changes: &[SchemaChange], rules: &EvolutionRules) -> Result<()> {
    // Check if all changes are allowed
    for change in changes {
        if !change.is_allowed_by(rules) {
            return Err(Error::Evolution(format!(
                "Change {:?} on field '{}' is not allowed by the evolution rules",
                change.change_type, change.field_name
            )));
        }
    }
    
    // Apply all changes
    for change in changes {
        change.apply(schema)?;
    }
    
    // Update the schema version
    schema.increment_version(false, true, false);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::definition::Schema;
    use serde_json::json;
    
    #[test]
    fn test_evolution_rules() {
        let mut rules = EvolutionRules::new();
        
        // Add some rules
        rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField))
             .add_rule(EvolutionRule::new(ChangeType::RemoveUnusedField));
        
        // Test rule matching
        assert!(rules.allows(&ChangeType::AddOptionalField));
        assert!(rules.allows(&ChangeType::RemoveUnusedField));
        
        // Safe change type should be allowed by default
        assert!(rules.allows(&ChangeType::MakeFieldOptional));
        
        // Unsafe change type should not be allowed by default
        assert!(!rules.allows(&ChangeType::RenameField));
        
        // Disable safe by default
        rules.with_allow_safe_by_default(false);
        
        // Now safe change types should not be allowed unless explicitly added
        assert!(!rules.allows(&ChangeType::MakeFieldOptional));
    }
    
    #[test]
    fn test_apply_changes() -> Result<()> {
        // Create a schema
        let mut schema = Schema::new("TestSchema", "1.0.0")?;
        schema.add_field(SchemaField::new("name", SchemaType::String, true));
        schema.add_field(SchemaField::new("age", SchemaType::Integer, true));
        
        // Create evolution rules
        let mut rules = EvolutionRules::new();
        rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField))
             .add_rule(EvolutionRule::new(ChangeType::RemoveUnusedField))
             .add_rule(EvolutionRule::new(ChangeType::AddDefaultField));
        
        // Create changes
        let changes = vec![
            // Add an optional field
            SchemaChange::new(ChangeType::AddOptionalField, "email")
                .with_new_type(SchemaType::String),
            
            // Remove an unused field
            SchemaChange::new(ChangeType::RemoveUnusedField, "age"),
            
            // Add a field with default
            SchemaChange::new(ChangeType::AddDefaultField, "is_active")
                .with_new_type(SchemaType::Boolean)
                .with_default(json!(true)),
        ];
        
        // Apply changes
        apply_changes(&mut schema, &changes, &rules)?;
        
        // Verify the changes were applied
        assert!(schema.fields.contains_key("name"));
        assert!(!schema.fields.contains_key("age"));
        assert!(schema.fields.contains_key("email"));
        assert!(schema.fields.contains_key("is_active"));
        
        // Check field properties
        assert!(!schema.fields.get("email").unwrap().required);
        assert!(schema.fields.get("is_active").unwrap().required);
        assert_eq!(
            schema.fields.get("is_active").unwrap().default_value,
            Some(json!(true))
        );
        
        // Check version was incremented
        assert_eq!(schema.version.as_str(), "1.1.0");
        
        Ok(())
    }
    
    #[test]
    fn test_disallowed_change() {
        // Create a schema
        let mut schema = Schema::new("TestSchema", "1.0.0").unwrap();
        schema.add_field(SchemaField::new("name", SchemaType::String, true));
        
        // Create evolution rules with only AddOptionalField allowed
        let mut rules = EvolutionRules::new();
        rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField));
        
        // Create a disallowed change
        let changes = vec![
            SchemaChange::new(ChangeType::RemoveUnusedField, "name"),
        ];
        
        // Apply changes should fail
        let result = apply_changes(&mut schema, &changes, &rules);
        assert!(result.is_err());
    }
} 