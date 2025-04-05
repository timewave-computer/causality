// Resource Query Filters
//
// This module provides a robust filtering system for resource queries,
// supporting complex conditions and expressions.

use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_types::ContentId;
use crate::resource::Resource;
use super::QueryError;

/// Filter condition for querying resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCondition {
    /// Field to filter on (can be a path like "metadata.created_at")
    pub field: String,
    
    /// Operator to apply
    pub operator: FilterOperator,
    
    /// Value to compare against
    pub value: FilterValue,
}

/// Filter value that can be any of multiple types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    /// String value
    String(String),
    
    /// Integer value
    Integer(i64),
    
    /// Float value
    Float(f64),
    
    /// Boolean value
    Boolean(bool),
    
    /// Content ID value
    ContentId(ContentId),
    
    /// Array of values
    Array(Vec<FilterValue>),
    
    /// Null value
    Null,
}

impl FilterValue {
    /// Convert a JSON value to a filter value
    pub fn from_json(value: &Value) -> Result<Self, QueryError> {
        match value {
            Value::String(s) => Ok(FilterValue::String(s.clone())),
            Value::Number(n) => {
                if n.is_i64() {
                    Ok(FilterValue::Integer(n.as_i64().unwrap()))
                } else {
                    Ok(FilterValue::Float(n.as_f64().unwrap()))
                }
            },
            Value::Bool(b) => Ok(FilterValue::Boolean(*b)),
            Value::Array(arr) => {
                let mut values = Vec::new();
                for v in arr {
                    values.push(FilterValue::from_json(v)?);
                }
                Ok(FilterValue::Array(values))
            },
            Value::Null => Ok(FilterValue::Null),
            _ => Err(QueryError::InvalidQuery(
                format!("Unsupported JSON value: {:?}", value)
            )),
        }
    }
    
    /// Get the value as a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FilterValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get the value as an integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FilterValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    /// Get the value as a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FilterValue::Float(f) => Some(*f),
            FilterValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    /// Get the value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FilterValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, FilterValue::Null)
    }
    
    /// Get the value as a string slice or a formatted representation
    pub fn display_value(&self) -> String {
        match self {
            FilterValue::String(s) => s.clone(),
            FilterValue::Integer(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Boolean(b) => b.to_string(),
            FilterValue::ContentId(id) => id.to_string(),
            FilterValue::Array(arr) => {
                let values: Vec<String> = arr.iter()
                    .map(|v| v.display_value())
                    .collect();
                format!("[{}]", values.join(", "))
            },
            FilterValue::Null => "null".to_string(),
        }
    }
}

/// Filter operators for comparing values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Equal to
    #[serde(rename = "eq")]
    Equal,
    
    /// Not equal to
    #[serde(rename = "neq")]
    NotEqual,
    
    /// Greater than
    #[serde(rename = "gt")]
    GreaterThan,
    
    /// Greater than or equal to
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    
    /// Less than
    #[serde(rename = "lt")]
    LessThan,
    
    /// Less than or equal to
    #[serde(rename = "lte")]
    LessThanOrEqual,
    
    /// Contains substring (for strings)
    #[serde(rename = "contains")]
    Contains,
    
    /// Starts with substring (for strings)
    #[serde(rename = "startsWith")]
    StartsWith,
    
    /// Ends with substring (for strings)
    #[serde(rename = "endsWith")]
    EndsWith,
    
    /// In a list of values
    #[serde(rename = "in")]
    In,
    
    /// Not in a list of values
    #[serde(rename = "notIn")]
    NotIn,
    
    /// Is null
    #[serde(rename = "isNull")]
    IsNull,
    
    /// Is not null
    #[serde(rename = "isNotNull")]
    IsNotNull,
    
    /// Matches regular expression (for strings)
    #[serde(rename = "regex")]
    Regex,
}

/// Filter expression for query engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterExpression {
    /// Single filter condition
    #[serde(rename = "condition")]
    Condition(FilterCondition),
    
    /// Logical AND of two expressions
    #[serde(rename = "and")]
    And(Box<FilterExpression>, Box<FilterExpression>),
    
    /// Logical OR of two expressions
    #[serde(rename = "or")]
    Or(Box<FilterExpression>, Box<FilterExpression>),
    
    /// Logical NOT of an expression
    #[serde(rename = "not")]
    Not(Box<FilterExpression>),
}

impl FilterExpression {
    /// Create a new filter condition
    pub fn condition(
        field: impl Into<String>,
        operator: FilterOperator,
        value: FilterValue,
    ) -> Self {
        Self::Condition(FilterCondition {
            field: field.into(),
            operator,
            value,
        })
    }
    
    /// Create an AND filter from two expressions
    pub fn and(left: FilterExpression, right: FilterExpression) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }
    
    /// Create an OR filter from two expressions
    pub fn or(left: FilterExpression, right: FilterExpression) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }
    
    /// Create a NOT filter from an expression
    pub fn not(expression: FilterExpression) -> Self {
        Self::Not(Box::new(expression))
    }
    
    /// Create a simple condition from string inputs
    pub fn condition_str(
        field: impl Into<String>,
        operator_str: &str,
        value_str: &str,
    ) -> Self {
        // Parse operator string to FilterOperator
        let operator = match operator_str {
            "=" | "==" | "eq" => FilterOperator::Equal,
            "!=" | "<>" | "ne" => FilterOperator::NotEqual,
            ">" | "gt" => FilterOperator::GreaterThan,
            ">=" | "gte" => FilterOperator::GreaterThanOrEqual,
            "<" | "lt" => FilterOperator::LessThan,
            "<=" | "lte" => FilterOperator::LessThanOrEqual,
            "contains" => FilterOperator::Contains,
            "startswith" => FilterOperator::StartsWith,
            "endswith" => FilterOperator::EndsWith,
            "in" => FilterOperator::In,
            "notin" => FilterOperator::NotIn,
            "null" => FilterOperator::IsNull,
            "!null" | "notnull" => FilterOperator::IsNotNull,
            "regex" => FilterOperator::Regex,
            _ => FilterOperator::Equal, // Default to Equal
        };
        
        // Create a value from the string
        let value = match operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => FilterValue::Null,
            FilterOperator::In | FilterOperator::NotIn => {
                // Split comma-separated values
                let values = value_str
                    .split(',')
                    .map(|v| FilterValue::String(v.trim().to_string()))
                    .collect();
                FilterValue::Array(values)
            },
            _ => FilterValue::String(value_str.to_string()),
        };
        
        Self::Condition(FilterCondition {
            field: field.into(),
            operator,
            value,
        })
    }
    
    /// Check if this filter matches a resource
    pub fn matches<R>(&self, resource: &R) -> Result<bool, QueryError> 
    where 
        R: Resource + ?Sized,
    {
        match self {
            FilterExpression::Condition(condition) => {
                // Use direct field access through the Resource trait methods
                if condition.field == "id" {
                    // Check resource ID matching
                    let id = resource.id();
                    match &condition.value {
                        FilterValue::String(s) => {
                            match condition.operator {
                                FilterOperator::Equal => Ok(id.to_string() == *s),
                                FilterOperator::NotEqual => Ok(id.to_string() != *s),
                                _ => Ok(false), // Unsupported operator for ID
                            }
                        },
                        FilterValue::ContentId(cid) => {
                            match condition.operator {
                                FilterOperator::Equal => Ok(id.to_content_id() == *cid),
                                FilterOperator::NotEqual => Ok(id.to_content_id() != *cid),
                                _ => Ok(false), // Unsupported operator for ID
                            }
                        },
                        _ => Ok(false), // ID doesn't match other types
                    }
                } else if condition.field == "type" || condition.field == "resource_type" {
                    // Check resource type matching
                    let rtype = resource.resource_type();
                    match &condition.value {
                        FilterValue::String(s) => {
                            match condition.operator {
                                FilterOperator::Equal => Ok(rtype.to_string() == *s),
                                FilterOperator::NotEqual => Ok(rtype.to_string() != *s),
                                _ => Ok(false), // Unsupported operator for type
                            }
                        },
                        _ => Ok(false), // Type doesn't match other types
                    }
                } else if condition.field == "state" {
                    // Check resource state matching
                    let state = resource.state();
                    match &condition.value {
                        FilterValue::String(s) => {
                            match condition.operator {
                                FilterOperator::Equal => Ok(state.to_string() == *s),
                                FilterOperator::NotEqual => Ok(state.to_string() != *s),
                                _ => Ok(false), // Unsupported operator for state
                            }
                        },
                        _ => Ok(false), // State doesn't match other types
                    }
                } else if condition.field.starts_with("metadata.") {
                    // Check metadata
                    let key = condition.field.strip_prefix("metadata.").unwrap_or(&condition.field);
                    let value = resource.get_metadata(key);
                    
                    match &condition.operator {
                        FilterOperator::Equal => {
                            if let FilterValue::String(s) = &condition.value {
                                Ok(value.as_deref() == Some(s))
                            } else {
                                Ok(false)
                            }
                        },
                        FilterOperator::NotEqual => {
                            if let FilterValue::String(s) = &condition.value {
                                Ok(value.as_deref() != Some(s))
                            } else {
                                Ok(false)
                            }
                        },
                        FilterOperator::Contains => {
                            if let FilterValue::String(s) = &condition.value {
                                Ok(value.as_deref().map(|v| v.contains(s)).unwrap_or(false))
                            } else {
                                Ok(false)
                            }
                        },
                        FilterOperator::StartsWith => {
                            if let FilterValue::String(s) = &condition.value {
                                Ok(value.as_deref().map(|v| v.starts_with(s)).unwrap_or(false))
                            } else {
                                Ok(false)
                            }
                        },
                        FilterOperator::EndsWith => {
                            if let FilterValue::String(s) = &condition.value {
                                Ok(value.as_deref().map(|v| v.ends_with(s)).unwrap_or(false))
                            } else {
                                Ok(false)
                            }
                        },
                        FilterOperator::IsNull => Ok(value.is_none()),
                        FilterOperator::IsNotNull => Ok(value.is_some()),
                        _ => Ok(false), // Other operators not supported for metadata
                    }
                } else {
                    // Default to false for unknown fields
                    Ok(false)
                }
            },
            FilterExpression::And(left, right) => {
                let left_result = left.matches(resource)?;
                if !left_result {
                    return Ok(false); // Short-circuit
                }
                let right_result = right.matches(resource)?;
                Ok(right_result)
            },
            FilterExpression::Or(left, right) => {
                let left_result = left.matches(resource)?;
                if left_result {
                    return Ok(true); // Short-circuit
                }
                let right_result = right.matches(resource)?;
                Ok(right_result)
            },
            FilterExpression::Not(expr) => {
                let result = expr.matches(resource)?;
                Ok(!result)
            },
        }
    }
}

/// Simple alias for FilterExpression
pub type Filter = FilterExpression;

/// Evaluate a filter condition against a resource
fn evaluate_condition<R>(condition: &FilterCondition, resource: &R) -> Result<bool, QueryError> 
where 
    R: Resource + Serialize,
{
    // Extract the field value from the resource
    let field_value = extract_field_value(resource, &condition.field)?;
    
    // Compare using the specified operator
    match condition.operator {
        FilterOperator::Equal => compare_equality(&field_value, &condition.value),
        FilterOperator::NotEqual => {
            let equal = compare_equality(&field_value, &condition.value)?;
            Ok(!equal)
        },
        FilterOperator::GreaterThan => compare_ordering(&field_value, &condition.value, Ordering::Greater),
        FilterOperator::GreaterThanOrEqual => {
            let ordering = compare_ordering(&field_value, &condition.value, Ordering::Greater)?;
            if ordering {
                return Ok(true);
            }
            compare_equality(&field_value, &condition.value)
        },
        FilterOperator::LessThan => compare_ordering(&field_value, &condition.value, Ordering::Less),
        FilterOperator::LessThanOrEqual => {
            let ordering = compare_ordering(&field_value, &condition.value, Ordering::Less)?;
            if ordering {
                return Ok(true);
            }
            compare_equality(&field_value, &condition.value)
        },
        FilterOperator::Contains => compare_string_contains(&field_value, &condition.value),
        FilterOperator::StartsWith => compare_string_starts_with(&field_value, &condition.value),
        FilterOperator::EndsWith => compare_string_ends_with(&field_value, &condition.value),
        FilterOperator::In => compare_in_list(&field_value, &condition.value),
        FilterOperator::NotIn => {
            let is_in = compare_in_list(&field_value, &condition.value)?;
            Ok(!is_in)
        },
        FilterOperator::IsNull => Ok(field_value.is_null()),
        FilterOperator::IsNotNull => Ok(!field_value.is_null()),
        FilterOperator::Regex => compare_regex(&field_value, &condition.value),
    }
}

/// Extract a field value from a resource
fn extract_field_value<R>(resource: &R, field_path: &str) -> Result<FilterValue, QueryError> 
where
    R: Resource + Serialize,
{
    // Convert the resource to a JSON value
    let resource_json = serde_json::to_value(resource)
        .map_err(|e| QueryError::SerializationError(e.to_string()))?;
    
    // Split the field path by dots
    let path_parts: Vec<&str> = field_path.split('.').collect();
    
    // Navigate the JSON structure
    let mut current = &resource_json;
    for part in &path_parts {
        match current {
            Value::Object(obj) => {
                current = obj.get(*part).ok_or_else(|| 
                    QueryError::FieldNotFound(format!("Field not found: {}", part))
                )?;
            },
            _ => return Err(QueryError::FieldNotFound(
                format!("Cannot navigate into non-object at path part: {}", part)
            )),
        }
    }
    
    // Convert the JSON value to a FilterValue
    FilterValue::from_json(current)
}

/// Enum for comparison ordering
enum Ordering {
    Less,
    Greater,
}

/// Compare two values for equality
fn compare_equality(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(b_str)) => Ok(a_str == b_str),
        (FilterValue::Integer(a_int), FilterValue::Integer(b_int)) => Ok(a_int == b_int),
        (FilterValue::Float(a_float), FilterValue::Float(b_float)) => Ok(a_float == b_float),
        (FilterValue::Integer(a_int), FilterValue::Float(b_float)) => Ok((*a_int as f64) == *b_float),
        (FilterValue::Float(a_float), FilterValue::Integer(b_int)) => Ok(*a_float == (*b_int as f64)),
        (FilterValue::Boolean(a_bool), FilterValue::Boolean(b_bool)) => Ok(a_bool == b_bool),
        (FilterValue::ContentId(a_id), FilterValue::ContentId(b_id)) => Ok(a_id == b_id),
        (FilterValue::Null, FilterValue::Null) => Ok(true),
        _ => Err(QueryError::TypeMismatch {
            expected: format!("{:?}", b),
            actual: format!("{:?}", a),
        }),
    }
}

/// Compare two values for ordering
fn compare_ordering(a: &FilterValue, b: &FilterValue, order: Ordering) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(b_str)) => {
            match order {
                Ordering::Less => Ok(a_str < b_str),
                Ordering::Greater => Ok(a_str > b_str),
            }
        },
        (FilterValue::Integer(a_int), FilterValue::Integer(b_int)) => {
            match order {
                Ordering::Less => Ok(a_int < b_int),
                Ordering::Greater => Ok(a_int > b_int),
            }
        },
        (FilterValue::Float(a_float), FilterValue::Float(b_float)) => {
            match order {
                Ordering::Less => Ok(a_float < b_float),
                Ordering::Greater => Ok(a_float > b_float),
            }
        },
        (FilterValue::Integer(a_int), FilterValue::Float(b_float)) => {
            let a_float = *a_int as f64;
            match order {
                Ordering::Less => Ok(a_float < *b_float),
                Ordering::Greater => Ok(a_float > *b_float),
            }
        },
        (FilterValue::Float(a_float), FilterValue::Integer(b_int)) => {
            let b_float = *b_int as f64;
            match order {
                Ordering::Less => Ok(*a_float < b_float),
                Ordering::Greater => Ok(*a_float > b_float),
            }
        },
        _ => Err(QueryError::TypeMismatch {
            expected: format!("{:?}", b),
            actual: format!("{:?}", a),
        }),
    }
}

/// Compare string contains
fn compare_string_contains(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(b_str)) => {
            Ok(a_str.contains(b_str))
        },
        _ => Err(QueryError::TypeMismatch {
            expected: "String".to_string(),
            actual: format!("{:?}", a),
        }),
    }
}

/// Compare string starts with
fn compare_string_starts_with(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(b_str)) => {
            Ok(a_str.starts_with(b_str))
        },
        _ => Err(QueryError::TypeMismatch {
            expected: "String".to_string(),
            actual: format!("{:?}", a),
        }),
    }
}

/// Compare string ends with
fn compare_string_ends_with(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(b_str)) => {
            Ok(a_str.ends_with(b_str))
        },
        _ => Err(QueryError::TypeMismatch {
            expected: "String".to_string(),
            actual: format!("{:?}", a),
        }),
    }
}

/// Compare value in list
fn compare_in_list(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match b {
        FilterValue::Array(values) => {
            for value in values {
                if let Ok(true) = compare_equality(a, value) {
                    return Ok(true);
                }
            }
            Ok(false)
        },
        _ => Err(QueryError::TypeMismatch {
            expected: "Array".to_string(),
            actual: format!("{:?}", b),
        }),
    }
}

/// Compare regex match
fn compare_regex(a: &FilterValue, b: &FilterValue) -> Result<bool, QueryError> {
    match (a, b) {
        (FilterValue::String(a_str), FilterValue::String(pattern)) => {
            // Use regex crate for regular expression matching
            match regex::Regex::new(pattern) {
                Ok(regex) => Ok(regex.is_match(a_str)),
                Err(e) => Err(QueryError::InvalidQuery(
                    format!("Invalid regex pattern: {}", e)
                )),
            }
        },
        _ => Err(QueryError::TypeMismatch {
            expected: "String".to_string(),
            actual: format!("{:?}", a),
        }),
    }
}

impl FilterCondition {
    /// Check if a resource matches this condition
    pub fn matches<R>(&self, resource: &R) -> Result<bool, QueryError> 
    where 
        R: Resource + Serialize + ?Sized,
    {
        // Convert resource to JSON value for field access
        let resource_json = serde_json::to_value(resource)
            .map_err(|e| QueryError::SerializationError(format!("Failed to serialize resource: {}", e)))?;
        
        // Get the field value
        let field_value = match resource_json.get(&self.field) {
            Some(value) => value,
            None => return Ok(false), // Field doesn't exist, no match
        };
        
        // Match based on operator and value
        match self.operator {
            FilterOperator::Equal => {
                if field_value.is_null() && matches!(self.value, FilterValue::Null) {
                    return Ok(true);
                }
                
                match &self.value {
                    FilterValue::String(s) => Ok(field_value.as_str().map(|v| v == s).unwrap_or(false)),
                    FilterValue::Integer(n) => Ok(field_value.as_i64().map(|v| v == *n).unwrap_or(false)),
                    FilterValue::Float(n) => Ok(field_value.as_f64().map(|v| (v - n).abs() < std::f64::EPSILON).unwrap_or(false)),
                    FilterValue::Boolean(b) => Ok(field_value.as_bool().map(|v| v == *b).unwrap_or(false)),
                    FilterValue::Null => Ok(field_value.is_null()),
                    FilterValue::Array(_) => Ok(false), // Can't equal an array
                    FilterValue::ContentId(_) => Ok(false), // Special case not handled here
                }
            },
            FilterOperator::NotEqual => {
                if field_value.is_null() && !matches!(self.value, FilterValue::Null) {
                    return Ok(true);
                }
                
                match &self.value {
                    FilterValue::String(s) => Ok(!field_value.as_str().map(|v| v == s).unwrap_or(true)),
                    FilterValue::Integer(n) => Ok(!field_value.as_i64().map(|v| v == *n).unwrap_or(true)),
                    FilterValue::Float(n) => Ok(!field_value.as_f64().map(|v| (v - n).abs() < std::f64::EPSILON).unwrap_or(true)),
                    FilterValue::Boolean(b) => Ok(!field_value.as_bool().map(|v| v == *b).unwrap_or(true)),
                    FilterValue::Null => Ok(!field_value.is_null()),
                    FilterValue::Array(_) => Ok(true), // Can't equal an array, so not equal is true
                    FilterValue::ContentId(_) => Ok(true), // Special case not handled here
                }
            },
            FilterOperator::GreaterThan => {
                match &self.value {
                    FilterValue::String(s) => Ok(field_value.as_str().map(|v| v > s.as_str()).unwrap_or(false)),
                    FilterValue::Integer(n) => Ok(field_value.as_i64().map(|v| v > *n).unwrap_or(false)),
                    FilterValue::Float(n) => Ok(field_value.as_f64().map(|v| v > *n).unwrap_or(false)),
                    _ => Ok(false), // Can't compare other types
                }
            },
            FilterOperator::GreaterThanOrEqual => {
                match &self.value {
                    FilterValue::String(s) => Ok(field_value.as_str().map(|v| v >= s.as_str()).unwrap_or(false)),
                    FilterValue::Integer(n) => Ok(field_value.as_i64().map(|v| v >= *n).unwrap_or(false)),
                    FilterValue::Float(n) => Ok(field_value.as_f64().map(|v| v >= *n).unwrap_or(false)),
                    _ => Ok(false), // Can't compare other types
                }
            },
            FilterOperator::LessThan => {
                match &self.value {
                    FilterValue::String(s) => Ok(field_value.as_str().map(|v| v < s.as_str()).unwrap_or(false)),
                    FilterValue::Integer(n) => Ok(field_value.as_i64().map(|v| v < *n).unwrap_or(false)),
                    FilterValue::Float(n) => Ok(field_value.as_f64().map(|v| v < *n).unwrap_or(false)),
                    _ => Ok(false), // Can't compare other types
                }
            },
            FilterOperator::LessThanOrEqual => {
                match &self.value {
                    FilterValue::String(s) => Ok(field_value.as_str().map(|v| v <= s.as_str()).unwrap_or(false)),
                    FilterValue::Integer(n) => Ok(field_value.as_i64().map(|v| v <= *n).unwrap_or(false)),
                    FilterValue::Float(n) => Ok(field_value.as_f64().map(|v| v <= *n).unwrap_or(false)),
                    _ => Ok(false), // Can't compare other types
                }
            },
            FilterOperator::Contains => {
                match &self.value {
                    FilterValue::String(s) => {
                        Ok(field_value.as_str().map(|v| v.contains(s)).unwrap_or(false))
                    },
                    _ => Ok(false), // Contains only works with strings
                }
            },
            FilterOperator::StartsWith => {
                match &self.value {
                    FilterValue::String(s) => {
                        Ok(field_value.as_str().map(|v| v.starts_with(s)).unwrap_or(false))
                    },
                    _ => Ok(false), // StartsWith only works with strings
                }
            },
            FilterOperator::EndsWith => {
                match &self.value {
                    FilterValue::String(s) => {
                        Ok(field_value.as_str().map(|v| v.ends_with(s)).unwrap_or(false))
                    },
                    _ => Ok(false), // EndsWith only works with strings
                }
            },
            FilterOperator::In => {
                match &self.value {
                    FilterValue::Array(values) => {
                        let mut result = false;
                        for value in values {
                            match value {
                                FilterValue::String(s) => {
                                    if field_value.as_str().map(|v| v == s).unwrap_or(false) {
                                        result = true;
                                        break;
                                    }
                                },
                                FilterValue::Integer(n) => {
                                    if field_value.as_i64().map(|v| v == *n).unwrap_or(false) {
                                        result = true;
                                        break;
                                    }
                                },
                                FilterValue::Float(n) => {
                                    if field_value.as_f64().map(|v| (v - n).abs() < std::f64::EPSILON).unwrap_or(false) {
                                        result = true;
                                        break;
                                    }
                                },
                                FilterValue::Boolean(b) => {
                                    if field_value.as_bool().map(|v| v == *b).unwrap_or(false) {
                                        result = true;
                                        break;
                                    }
                                },
                                _ => {},
                            }
                        }
                        Ok(result)
                    },
                    _ => Ok(false), // In requires an array
                }
            },
            FilterOperator::NotIn => {
                match &self.value {
                    FilterValue::Array(values) => {
                        let mut in_array = false;
                        for value in values {
                            match value {
                                FilterValue::String(s) => {
                                    if field_value.as_str().map(|v| v == s).unwrap_or(false) {
                                        in_array = true;
                                        break;
                                    }
                                },
                                FilterValue::Integer(n) => {
                                    if field_value.as_i64().map(|v| v == *n).unwrap_or(false) {
                                        in_array = true;
                                        break;
                                    }
                                },
                                FilterValue::Float(n) => {
                                    if field_value.as_f64().map(|v| (v - n).abs() < std::f64::EPSILON).unwrap_or(false) {
                                        in_array = true;
                                        break;
                                    }
                                },
                                FilterValue::Boolean(b) => {
                                    if field_value.as_bool().map(|v| v == *b).unwrap_or(false) {
                                        in_array = true;
                                        break;
                                    }
                                },
                                _ => {},
                            }
                        }
                        Ok(!in_array)
                    },
                    _ => Ok(true), // NotIn requires an array, otherwise always true
                }
            },
            FilterOperator::IsNull => {
                Ok(field_value.is_null())
            },
            FilterOperator::IsNotNull => {
                Ok(!field_value.is_null())
            },
            FilterOperator::Regex => {
                // We don't actually implement regex here, but we need to handle it
                match &self.value {
                    FilterValue::String(_) => {
                        Ok(false) // Not supported in this simple implementation
                    },
                    _ => Ok(false), // Regex only works with strings
                }
            },
        }
    }
} 