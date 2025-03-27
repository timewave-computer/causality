// Resource Query Sorting
//
// This module provides sorting capabilities for resource queries,
// allowing results to be ordered by different fields and directions.

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::resource::{Resource, ContentId};
use super::QueryError;

/// Sort direction for query results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Sort in ascending order
    #[serde(rename = "asc")]
    Ascending,
    
    /// Sort in descending order
    #[serde(rename = "desc")]
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Sort specification for a single field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sort {
    /// Field to sort by
    pub field: String,
    
    /// Sort direction
    #[serde(default)]
    pub direction: SortDirection,
}

impl Sort {
    /// Create a new sort specification
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }
    
    /// Create a new ascending sort
    pub fn ascending(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Ascending)
    }
    
    /// Create a new descending sort
    pub fn descending(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Descending)
    }
    
    /// Compare two resources using this sort specification
    pub fn compare<R: Resource>(&self, a: &R, b: &R) -> Result<Ordering, QueryError> {
        // Convert resources to JSON for field extraction
        let a_json = serde_json::to_value(a)
            .map_err(|e| QueryError::SerializationError(e.to_string()))?;
        
        let b_json = serde_json::to_value(b)
            .map_err(|e| QueryError::SerializationError(e.to_string()))?;
        
        // Extract field values
        let a_value = extract_field_value(&a_json, &self.field)?;
        let b_value = extract_field_value(&b_json, &self.field)?;
        
        // Compare values
        let ordering = compare_values(a_value, b_value)?;
        
        // Apply sort direction
        match self.direction {
            SortDirection::Ascending => Ok(ordering),
            SortDirection::Descending => Ok(ordering.reverse()),
        }
    }
}

/// Options for controlling sort behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    /// Sort fields in order of precedence
    pub fields: Vec<Sort>,
    
    /// Whether nulls should sort first or last
    #[serde(default = "default_nulls_last")]
    pub nulls_last: bool,
    
    /// Case sensitivity for string comparisons
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
}

fn default_nulls_last() -> bool {
    true
}

fn default_case_sensitive() -> bool {
    true
}

impl Default for SortOptions {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            nulls_last: true,
            case_sensitive: true,
        }
    }
}

impl SortOptions {
    /// Create a new sort options with the given fields
    pub fn new(fields: Vec<Sort>) -> Self {
        Self {
            fields,
            ..Default::default()
        }
    }
    
    /// Add a sort field
    pub fn with_field(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.fields.push(Sort::new(field, direction));
        self
    }
    
    /// Set nulls last
    pub fn with_nulls_last(mut self, nulls_last: bool) -> Self {
        self.nulls_last = nulls_last;
        self
    }
    
    /// Set case sensitivity
    pub fn with_case_sensitivity(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }
    
    /// Compare two resources using these sort options
    pub fn compare<R: Resource>(&self, a: &R, b: &R) -> Result<Ordering, QueryError> {
        for sort in &self.fields {
            match sort.compare(a, b) {
                Ok(Ordering::Equal) => continue, // Try next field
                Ok(ordering) => return Ok(ordering),
                Err(e) => return Err(e),
            }
        }
        
        // If all fields are equal, use content ID as tiebreaker to ensure stable sort
        let a_id = a.resource_id();
        let b_id = b.resource_id();
        
        Ok(a_id.to_string().cmp(&b_id.to_string()))
    }
}

/// Extract a field value from a JSON object by path
/// 
/// This function allows accessing nested fields using dot notation
fn extract_field_value<'a>(json: &'a Value, field_path: &'a str) -> Result<&'a Value, QueryError> {
    let path_parts: Vec<&str> = field_path.split('.').collect();
    
    let mut current = json;
    for part in &path_parts {
        match current {
            Value::Object(obj) => {
                current = obj.get(*part).ok_or_else(|| 
                    QueryError::FieldNotFound(format!("Field not found for sorting: {}", part))
                )?;
            },
            _ => return Err(QueryError::FieldNotFound(
                format!("Cannot navigate into non-object at path part: {}", part)
            )),
        }
    }
    
    Ok(current)
}

/// Compare two JSON values
fn compare_values(a: &Value, b: &Value) -> Result<Ordering, QueryError> {
    match (a, b) {
        // Handle nulls
        (Value::Null, Value::Null) => Ok(Ordering::Equal),
        (Value::Null, _) => Ok(Ordering::Less),  // Nulls first by default
        (_, Value::Null) => Ok(Ordering::Greater),
        
        // Compare strings
        (Value::String(a_str), Value::String(b_str)) => {
            Ok(a_str.cmp(b_str))
        },
        
        // Compare numbers
        (Value::Number(a_num), Value::Number(b_num)) => {
            if let (Some(a_i64), Some(b_i64)) = (a_num.as_i64(), b_num.as_i64()) {
                return Ok(a_i64.cmp(&b_i64));
            }
            
            let a_f64 = a_num.as_f64().ok_or_else(|| 
                QueryError::TypeMismatch {
                    expected: "Number".to_string(),
                    actual: format!("{:?}", a),
                }
            )?;
            
            let b_f64 = b_num.as_f64().ok_or_else(|| 
                QueryError::TypeMismatch {
                    expected: "Number".to_string(),
                    actual: format!("{:?}", b),
                }
            )?;
            
            match a_f64.partial_cmp(&b_f64) {
                Some(ordering) => Ok(ordering),
                None => Err(QueryError::InvalidQuery(
                    format!("Cannot compare floats: {} and {}", a_f64, b_f64)
                )),
            }
        },
        
        // Compare booleans
        (Value::Bool(a_bool), Value::Bool(b_bool)) => {
            Ok(a_bool.cmp(b_bool))
        },
        
        // Compare arrays (lexicographically)
        (Value::Array(a_arr), Value::Array(b_arr)) => {
            for (a_item, b_item) in a_arr.iter().zip(b_arr.iter()) {
                match compare_values(a_item, b_item)? {
                    Ordering::Equal => continue,
                    ordering => return Ok(ordering),
                }
            }
            
            // If one array is a prefix of the other, the shorter one comes first
            Ok(a_arr.len().cmp(&b_arr.len()))
        },
        
        // Compare objects by their string representation (not ideal, but a fallback)
        (Value::Object(_), Value::Object(_)) => {
            let a_str = a.to_string();
            let b_str = b.to_string();
            Ok(a_str.cmp(&b_str))
        },
        
        // Different types cannot be compared
        _ => Err(QueryError::TypeMismatch {
            expected: format!("{:?}", a),
            actual: format!("{:?}", b),
        }),
    }
} 