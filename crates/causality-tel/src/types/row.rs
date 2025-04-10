//! Row Type System for TEL
//!
//! This module implements the row type system, supporting extensible records
//! with row polymorphism, extension, restriction, and disjointness operations.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Serialize, Deserialize};

use super::{TelType, BaseType};

/// A row type represented as a set of fields and an optional extension variable
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RowType {
    /// The fields in the row
    pub fields: BTreeMap<String, TelType>,
    
    /// The extension variable, if any
    pub extension: Option<String>,
}

/// A row constraint to enforce properties of row types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowConstraint {
    /// The row lacks a field (r lacks field)
    Lacks(String, String),
    
    /// The rows are disjoint (r1 # r2)
    Disjoint(String, String),
    
    /// The row is the union of two other rows (r1 + r2 = r3)
    Union(String, String, String),
}

impl RowType {
    /// Create a new empty row type with no extension
    pub fn empty() -> Self {
        RowType {
            fields: BTreeMap::new(),
            extension: None,
        }
    }
    
    /// Create a new row type with the given fields and no extension
    pub fn from_fields(fields: BTreeMap<String, TelType>) -> Self {
        RowType {
            fields,
            extension: None,
        }
    }
    
    /// Create a new row type with the given fields and extension
    pub fn with_extension(fields: BTreeMap<String, TelType>, extension: String) -> Self {
        RowType {
            fields,
            extension: Some(extension),
        }
    }
    
    /// Create a row type with just an extension variable
    pub fn from_extension(extension: String) -> Self {
        RowType {
            fields: BTreeMap::new(),
            extension: Some(extension),
        }
    }
    
    /// Check if the row type has a field
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }
    
    /// Get the type of a field
    pub fn get_field(&self, field: &str) -> Option<&TelType> {
        self.fields.get(field)
    }
    
    /// Add a field to the row type
    pub fn with_field(mut self, field: String, field_type: TelType) -> Result<Self, RowError> {
        if self.has_field(&field) {
            return Err(RowError::FieldAlreadyExists(field));
        }
        self.fields.insert(field, field_type);
        Ok(self)
    }
    
    /// Remove a field from the row type
    pub fn without_field(mut self, field: &str) -> Result<Self, RowError> {
        if !self.has_field(field) {
            return Err(RowError::FieldDoesNotExist(field.to_string()));
        }
        self.fields.remove(field);
        Ok(self)
    }
    
    /// Restrict a row type to only include the specified fields
    pub fn restrict<I>(&self, fields: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let field_set: BTreeSet<String> = fields.into_iter().collect();
        
        let restricted_fields = self.fields
            .iter()
            .filter(|(field, _)| field_set.contains(*field))
            .map(|(field, field_type)| (field.clone(), field_type.clone()))
            .collect();
        
        // Keep the extension, as it might contain other fields
        RowType {
            fields: restricted_fields,
            extension: self.extension.clone(),
        }
    }
    
    /// Combine two row types if they have no fields in common
    pub fn union(&self, other: &RowType) -> Result<RowType, RowError> {
        // Check for field conflicts
        for field in self.fields.keys() {
            if other.has_field(field) {
                return Err(RowError::FieldConflict(field.clone()));
            }
        }
        
        // Merge fields
        let mut merged_fields = self.fields.clone();
        for (field, field_type) in &other.fields {
            merged_fields.insert(field.clone(), field_type.clone());
        }
        
        // Handle extensions
        let extension = match (&self.extension, &other.extension) {
            (Some(ext1), Some(ext2)) => {
                // Both have extensions - create a union constraint
                // For now, we'll use the first extension and assume a constraint exists
                Some(ext1.clone())
            },
            (Some(ext), None) => Some(ext.clone()),
            (None, Some(ext)) => Some(ext.clone()),
            (None, None) => None,
        };
        
        Ok(RowType {
            fields: merged_fields,
            extension,
        })
    }
    
    /// Check if this row type is a subtype of another row type
    pub fn is_subtype(&self, other: &RowType) -> bool {
        // All fields in other must be in self with compatible types
        for (field, other_type) in &other.fields {
            match self.get_field(field) {
                Some(self_type) => {
                    if !self_type.is_subtype(other_type) {
                        return false;
                    }
                },
                None => return false,
            }
        }
        
        // Handle extensions
        match (&self.extension, &other.extension) {
            // If other has an extension and self doesn't, not a subtype
            (None, Some(_)) => false,
            // Other cases are compatible
            _ => true,
        }
    }
}

impl fmt::Display for RowType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        
        for (field, field_type) in &self.fields {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", field, field_type)?;
            first = false;
        }
        
        if let Some(ext) = &self.extension {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "| {}", ext)?;
        }
        
        Ok(())
    }
}

/// Error type for row type operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowError {
    /// Field already exists in the row
    FieldAlreadyExists(String),
    
    /// Field does not exist in the row
    FieldDoesNotExist(String),
    
    /// Field conflict when combining rows
    FieldConflict(String),
    
    /// Cannot remove a field from a row with an extension
    CannotRemoveWithExtension,
}

impl fmt::Display for RowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowError::FieldAlreadyExists(field) => {
                write!(f, "Field '{}' already exists in the row", field)
            },
            RowError::FieldDoesNotExist(field) => {
                write!(f, "Field '{}' does not exist in the row", field)
            },
            RowError::FieldConflict(field) => {
                write!(f, "Field '{}' conflicts when combining rows", field)
            },
            RowError::CannotRemoveWithExtension => {
                write!(f, "Cannot remove a field from a row with an extension")
            },
        }
    }
}

/// Functions for working with row types in the TEL type system
pub mod operations {
    use super::*;
    use crate::types::TypeEnvironment;
    
    /// Extend a record type with a new field
    pub fn extend_record(
        record_type: &TelType,
        field: &str,
        field_type: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match record_type {
            TelType::Record(record) => {
                // Check that the field doesn't already exist
                if record.fields.contains_key(field) {
                    return Err(RowError::FieldAlreadyExists(field.to_string()));
                }
                
                // Create a new record with the field added
                let mut new_fields = record.fields.clone();
                new_fields.insert(field.to_string(), field_type.clone());
                
                Ok(TelType::Record(super::super::RecordType {
                    fields: new_fields,
                    extension: record.extension.clone(),
                }))
            },
            _ => {
                // Only record types can be extended
                Err(RowError::FieldDoesNotExist(
                    "Cannot extend a non-record type".to_string(),
                ))
            },
        }
    }
    
    /// Project a field from a record type
    pub fn project_field(
        record_type: &TelType,
        field: &str,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match record_type {
            TelType::Record(record) => {
                // Get the field type
                if let Some(field_type) = record.fields.get(field) {
                    Ok(field_type.clone())
                } else {
                    Err(RowError::FieldDoesNotExist(field.to_string()))
                }
            },
            _ => {
                // Only record types can be projected
                Err(RowError::FieldDoesNotExist(
                    "Cannot project from a non-record type".to_string(),
                ))
            },
        }
    }
    
    /// Update a field in a record type
    pub fn update_field(
        record_type: &TelType,
        field: &str,
        field_type: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match record_type {
            TelType::Record(record) => {
                // Check that the field exists
                if !record.fields.contains_key(field) {
                    return Err(RowError::FieldDoesNotExist(field.to_string()));
                }
                
                // Create a new record with the field updated
                let mut new_fields = record.fields.clone();
                new_fields.insert(field.to_string(), field_type.clone());
                
                Ok(TelType::Record(super::super::RecordType {
                    fields: new_fields,
                    extension: record.extension.clone(),
                }))
            },
            _ => {
                // Only record types can be updated
                Err(RowError::FieldDoesNotExist(
                    "Cannot update a non-record type".to_string(),
                ))
            },
        }
    }
    
    /// Remove a field from a record type
    pub fn remove_field(
        record_type: &TelType,
        field: &str,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match record_type {
            TelType::Record(record) => {
                // Check that the field exists
                if !record.fields.contains_key(field) {
                    return Err(RowError::FieldDoesNotExist(field.to_string()));
                }
                
                // Create a new record with the field removed
                let mut new_fields = record.fields.clone();
                new_fields.remove(field);
                
                Ok(TelType::Record(super::super::RecordType {
                    fields: new_fields,
                    extension: record.extension.clone(),
                }))
            },
            _ => {
                // Only record types can have fields removed
                Err(RowError::FieldDoesNotExist(
                    "Cannot remove field from a non-record type".to_string(),
                ))
            },
        }
    }
    
    /// Merge two record types if they have disjoint fields
    pub fn merge_records(
        record1: &TelType,
        record2: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match (record1, record2) {
            (TelType::Record(r1), TelType::Record(r2)) => {
                // Check for field conflicts
                for field in r1.fields.keys() {
                    if r2.fields.contains_key(field) {
                        return Err(RowError::FieldConflict(field.clone()));
                    }
                }
                
                // Merge fields
                let mut merged_fields = r1.fields.clone();
                for (field, field_type) in &r2.fields {
                    merged_fields.insert(field.clone(), field_type.clone());
                }
                
                // Handle extensions
                let extension = match (&r1.extension, &r2.extension) {
                    (Some(ext1), Some(ext2)) => {
                        // Both have extensions - must be disjoint
                        if ext1 == ext2 {
                            Some(ext1.clone())
                        } else {
                            // We would need a unification algorithm here
                            None
                        }
                    },
                    (Some(ext), None) => Some(ext.clone()),
                    (None, Some(ext)) => Some(ext.clone()),
                    (None, None) => None,
                };
                
                Ok(TelType::Record(super::super::RecordType {
                    fields: merged_fields,
                    extension,
                }))
            },
            _ => {
                // Only record types can be merged
                Err(RowError::FieldDoesNotExist(
                    "Cannot merge non-record types".to_string(),
                ))
            },
        }
    }
}

/// Helper macros for working with row types
pub mod macros {
    /// Create a record type from a list of field:type pairs
    #[macro_export]
    macro_rules! record {
        // Empty record
        () => {
            TelType::Record(RecordType {
                fields: BTreeMap::new(),
                extension: None,
            })
        };
        
        // Record with fields and no extension
        ($($field:ident : $type:expr),* $(,)?) => {{
            let mut fields = BTreeMap::new();
            $(
                fields.insert(stringify!($field).to_string(), $type);
            )*
            TelType::Record(RecordType {
                fields,
                extension: None,
            })
        }};
        
        // Record with fields and extension
        ($($field:ident : $type:expr),* ; $ext:ident) => {{
            let mut fields = BTreeMap::new();
            $(
                fields.insert(stringify!($field).to_string(), $type);
            )*
            TelType::Record(RecordType {
                fields,
                extension: Some(stringify!($ext).to_string()),
            })
        }};
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TelType, BaseType, RecordType, TypeEnvironment};
    
    #[test]
    fn test_row_type_display() {
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), TelType::Base(BaseType::String));
        fields.insert("age".to_string(), TelType::Base(BaseType::Int));
        
        let row = RowType {
            fields,
            extension: Some("r".to_string()),
        };
        
        // BTreeMap keys are sorted
        assert_eq!(format!("{}", row), "age: Int, name: String, | r");
    }
    
    #[test]
    fn test_row_type_union() {
        let mut fields1 = BTreeMap::new();
        fields1.insert("name".to_string(), TelType::Base(BaseType::String));
        
        let mut fields2 = BTreeMap::new();
        fields2.insert("age".to_string(), TelType::Base(BaseType::Int));
        
        let row1 = RowType {
            fields: fields1,
            extension: None,
        };
        
        let row2 = RowType {
            fields: fields2,
            extension: None,
        };
        
        let row3 = row1.union(&row2).unwrap();
        
        assert!(row3.has_field("name"));
        assert!(row3.has_field("age"));
        assert_eq!(row3.extension, None);
    }
    
    #[test]
    fn test_row_type_union_conflict() {
        let mut fields1 = BTreeMap::new();
        fields1.insert("name".to_string(), TelType::Base(BaseType::String));
        
        let mut fields2 = BTreeMap::new();
        fields2.insert("name".to_string(), TelType::Base(BaseType::String));
        
        let row1 = RowType {
            fields: fields1,
            extension: None,
        };
        
        let row2 = RowType {
            fields: fields2,
            extension: None,
        };
        
        let result = row1.union(&row2);
        assert!(result.is_err());
        match result {
            Err(RowError::FieldConflict(field)) => assert_eq!(field, "name"),
            _ => panic!("Expected field conflict error"),
        }
    }
    
    #[test]
    fn test_row_operations() {
        let mut env = TypeEnvironment::new();
        
        // Create a record type
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), TelType::Base(BaseType::String));
        
        let record = TelType::Record(RecordType {
            fields,
            extension: None,
        });
        
        // Extend the record
        let extended = operations::extend_record(
            &record,
            "age",
            &TelType::Base(BaseType::Int),
            &env,
        ).unwrap();
        
        // Project a field
        let name_type = operations::project_field(&extended, "name", &env).unwrap();
        assert_eq!(name_type, TelType::Base(BaseType::String));
        
        // Update a field
        let updated = operations::update_field(
            &extended,
            "name",
            &TelType::Base(BaseType::Int),
            &env,
        ).unwrap();
        
        // Remove a field
        let removed = operations::remove_field(&updated, "name", &env).unwrap();
        
        if let TelType::Record(record) = removed {
            assert!(!record.fields.contains_key("name"));
            assert!(record.fields.contains_key("age"));
        } else {
            panic!("Expected record type");
        }
    }
    
    #[test]
    fn test_row_subtyping() {
        let mut fields1 = BTreeMap::new();
        fields1.insert("name".to_string(), TelType::Base(BaseType::String));
        fields1.insert("age".to_string(), TelType::Base(BaseType::Int));
        
        let mut fields2 = BTreeMap::new();
        fields2.insert("name".to_string(), TelType::Base(BaseType::String));
        
        let row1 = RowType {
            fields: fields1,
            extension: None,
        };
        
        let row2 = RowType {
            fields: fields2,
            extension: None,
        };
        
        // row1 is a subtype of row2 (it has all fields of row2 with compatible types)
        assert!(row1.is_subtype(&row2));
        
        // row2 is not a subtype of row1 (it's missing a field)
        assert!(!row2.is_subtype(&row1));
    }
} 