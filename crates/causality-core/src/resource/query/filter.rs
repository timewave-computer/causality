// Resource Query Filters
//
// This module provides a robust filtering system for resource queries,
// supporting complex conditions and expressions.

use std::fmt::{Debug, Display};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::resource::{ContentId, Resource, ResourceType};
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

/// Complex filter expression for combining conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FilterExpression {
    /// Simple condition
    #[serde(rename = "condition")]
    Condition(FilterCondition),
    
    /// Logical AND of multiple expressions
    #[serde(rename = "and")]
    And(Vec<FilterExpression>),
    
    /// Logical OR of multiple expressions
    #[serde(rename = "or")]
    Or(Vec<FilterExpression>),
    
    /// Logical NOT of an expression
    #[serde(rename = "not")]
    Not(Box<FilterExpression>),
}

impl FilterExpression {
    /// Create a simple condition filter
    pub fn condition(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self::Condition(FilterCondition {
            field: field.into(),
            operator,
            value,
        })
    }
    
    /// Create an AND filter from multiple expressions
    pub fn and(expressions: Vec<FilterExpression>) -> Self {
        Self::And(expressions)
    }
    
    /// Create an OR filter from multiple expressions
    pub fn or(expressions: Vec<FilterExpression>) -> Self {
        Self::Or(expressions)
    }
    
    /// Create a NOT filter from an expression
    pub fn not(expression: FilterExpression) -> Self {
        Self::Not(Box::new(expression))
    }
    
    /// Check if a resource matches this filter expression
    pub fn matches(&self, resource: &impl Resource) -> Result<bool, QueryError> {
        match self {
            FilterExpression::Condition(condition) => {
                evaluate_condition(condition, resource)
            },
            FilterExpression::And(expressions) => {
                for expr in expressions {
                    if !expr.matches(resource)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            FilterExpression::Or(expressions) => {
                if expressions.is_empty() {
                    return Ok(true);
                }
                
                for expr in expressions {
                    if expr.matches(resource)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            FilterExpression::Not(expression) => {
                let result = expression.matches(resource)?;
                Ok(!result)
            },
        }
    }
}

/// Simple alias for FilterExpression
pub type Filter = FilterExpression;

/// Evaluate a filter condition against a resource
fn evaluate_condition(condition: &FilterCondition, resource: &impl Resource) -> Result<bool, QueryError> {
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
fn extract_field_value(resource: &impl Resource, field_path: &str) -> Result<FilterValue, QueryError> {
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
            match order {
                Ordering::Less => Ok((*a_int as f64) < *b_float),
                Ordering::Greater => Ok((*a_int as f64) > *b_float),
            }
        },
        (FilterValue::Float(a_float), FilterValue::Integer(b_int)) => {
            match order {
                Ordering::Less => Ok(*a_float < (*b_int as f64)),
                Ordering::Greater => Ok(*a_float > (*b_int as f64)),
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