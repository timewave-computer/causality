// Resource Query Module
//
// This module provides a comprehensive query system for resources,
// supporting advanced filtering, sorting, and pagination operations.
// It implements the capabilities required by ADR12.11.

mod engine;
mod filter;
mod sort;
mod pagination;
mod index;
mod builder;

pub use engine::{QueryEngine, QueryOptions, QueryExecution};
pub use filter::{Filter, FilterCondition, FilterOperator, FilterExpression};
pub use sort::{Sort, SortDirection, SortOptions};
pub use pagination::{Pagination, PaginationOptions, PaginationResult};
pub use index::{ResourceIndex, IndexKey, IndexType, IndexEntry};
pub use builder::{QueryBuilder, FilterBuilder, SortBuilder};

use std::fmt::Debug;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource::Resource;
use crate::resource_types::ResourceType;

/// Error that can occur during a query operation
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Invalid filter: {0}")]
    InvalidFilter(String),
    
    #[error("Invalid sort: {0}")]
    InvalidSort(String),
    
    #[error("Invalid pagination: {0}")]
    InvalidPagination(String),
    
    #[error("Resource type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Index error: {0}")]
    IndexError(String),
}

/// Result type for query operations
pub type QueryResult<T> = Result<T, QueryError>;

/// Resource query definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    /// Resource type to query
    pub resource_type: Option<ResourceType>,
    
    /// Filter expression
    pub filter: Option<FilterExpression>,
    
    /// Sort options
    pub sort: Vec<Sort>,
    
    /// Pagination options
    pub pagination: Option<Pagination>,
    
    /// Include specific fields only
    pub include_fields: Option<Vec<String>>,
    
    /// Exclude specific fields
    pub exclude_fields: Option<Vec<String>>,
    
    /// Additional query parameters
    pub parameters: HashMap<String, String>,
}

impl ResourceQuery {
    /// Create a new empty resource query
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a query for a specific resource type
    pub fn for_type(resource_type: ResourceType) -> Self {
        Self {
            resource_type: Some(resource_type),
            filter: None,
            sort: Vec::new(),
            pagination: None,
            include_fields: None,
            exclude_fields: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Add a filter to the query
    pub fn with_filter(mut self, filter: FilterExpression) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Add a sort option to the query
    pub fn with_sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.sort.push(Sort {
            field: field.into(),
            direction,
        });
        self
    }
    
    /// Add pagination to the query
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.pagination = Some(Pagination {
            limit: Some(limit),
            offset: Some(offset),
        });
        self
    }
    
    /// Include only specific fields in the result
    pub fn include_only(mut self, fields: Vec<String>) -> Self {
        self.include_fields = Some(fields);
        self
    }
    
    /// Exclude specific fields from the result
    pub fn exclude(mut self, fields: Vec<String>) -> Self {
        self.exclude_fields = Some(fields);
        self
    }
    
    /// Add a query parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

impl Default for ResourceQuery {
    fn default() -> Self {
        Self {
            resource_type: None,
            filter: None,
            sort: Vec::new(),
            pagination: None,
            include_fields: None,
            exclude_fields: None,
            parameters: HashMap::new(),
        }
    }
} 