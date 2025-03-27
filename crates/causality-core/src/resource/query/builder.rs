// Resource Query Builder
//
// This module provides a fluent interface for building resource queries.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::resource::ResourceType;
use super::{
    ResourceQuery, FilterExpression, FilterOperator, 
    FilterValue, FilterCondition, Sort, SortDirection, Pagination
};

/// Builder for constructing resource queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// The query being built
    query: ResourceQuery,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: ResourceQuery::default(),
        }
    }
    
    /// Create a query builder for a specific resource type
    pub fn for_type(resource_type: ResourceType) -> Self {
        Self {
            query: ResourceQuery::for_type(resource_type),
        }
    }
    
    /// Set the resource type for the query
    pub fn resource_type(mut self, resource_type: ResourceType) -> Self {
        self.query.resource_type = Some(resource_type);
        self
    }
    
    /// Add a filter to the query
    pub fn filter(mut self, filter: FilterExpression) -> Self {
        self.query.filter = Some(filter);
        self
    }
    
    /// Add a sort field to the query
    pub fn sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.query.sort.push(Sort {
            field: field.into(),
            direction,
        });
        self
    }
    
    /// Add an ascending sort
    pub fn sort_asc(self, field: impl Into<String>) -> Self {
        self.sort(field, SortDirection::Ascending)
    }
    
    /// Add a descending sort
    pub fn sort_desc(self, field: impl Into<String>) -> Self {
        self.sort(field, SortDirection::Descending)
    }
    
    /// Set pagination parameters
    pub fn paginate(mut self, limit: usize, offset: usize) -> Self {
        self.query.pagination = Some(Pagination {
            limit: Some(limit),
            offset: Some(offset),
        });
        self
    }
    
    /// Set the limit for results
    pub fn limit(mut self, limit: usize) -> Self {
        let pagination = self.query.pagination.get_or_insert(Pagination::default());
        pagination.limit = Some(limit);
        self
    }
    
    /// Set the offset for results
    pub fn offset(mut self, offset: usize) -> Self {
        let pagination = self.query.pagination.get_or_insert(Pagination::default());
        pagination.offset = Some(offset);
        self
    }
    
    /// Include only specific fields in the result
    pub fn include_fields(mut self, fields: Vec<String>) -> Self {
        self.query.include_fields = Some(fields);
        self
    }
    
    /// Exclude specific fields from the result
    pub fn exclude_fields(mut self, fields: Vec<String>) -> Self {
        self.query.exclude_fields = Some(fields);
        self
    }
    
    /// Add a query parameter
    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Use the filter builder for more complex filters
    pub fn where_field(self, field: impl Into<String>) -> FilterBuilder {
        FilterBuilder::new(self, field.into())
    }
    
    /// Add a condition that a field equals a value
    pub fn where_equals(self, field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::Equal,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add a condition that a field does not equal a value
    pub fn where_not_equals(self, field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::NotEqual,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add a condition that a field is greater than a value
    pub fn where_greater_than(self, field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::GreaterThan,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add a condition that a field is less than a value
    pub fn where_less_than(self, field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::LessThan,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add a condition that a field contains a substring
    pub fn where_contains(self, field: impl Into<String>, value: impl Into<String>) -> Self {
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::Contains,
            value: FilterValue::String(value.into()),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add a condition that a field is in a list of values
    pub fn where_in<T: Into<FilterValue>>(self, field: impl Into<String>, values: Vec<T>) -> Self {
        let filter_values = values.into_iter()
            .map(|v| v.into())
            .collect();
        
        let condition = FilterCondition {
            field: field.into(),
            operator: FilterOperator::In,
            value: FilterValue::Array(filter_values),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.add_filter(filter)
    }
    
    /// Add an AND filter expression
    pub fn and(self, expressions: Vec<FilterExpression>) -> Self {
        let filter = FilterExpression::And(expressions);
        self.add_filter(filter)
    }
    
    /// Add an OR filter expression
    pub fn or(self, expressions: Vec<FilterExpression>) -> Self {
        let filter = FilterExpression::Or(expressions);
        self.add_filter(filter)
    }
    
    /// Add a NOT filter expression
    pub fn not(self, expression: FilterExpression) -> Self {
        let filter = FilterExpression::Not(Box::new(expression));
        self.add_filter(filter)
    }
    
    /// Add a filter to the query, handling AND combinations
    fn add_filter(mut self, filter: FilterExpression) -> Self {
        match self.query.filter {
            None => {
                self.query.filter = Some(filter);
            },
            Some(FilterExpression::And(ref mut expressions)) => {
                expressions.push(filter);
            },
            Some(existing) => {
                self.query.filter = Some(FilterExpression::And(vec![existing, filter]));
            }
        }
        
        self
    }
    
    /// Build the final query
    pub fn build(self) -> ResourceQuery {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing filter conditions
#[derive(Debug, Clone)]
pub struct FilterBuilder {
    /// The query builder this filter will be added to
    query_builder: QueryBuilder,
    
    /// The field to filter on
    field: String,
}

impl FilterBuilder {
    /// Create a new filter builder
    pub fn new(query_builder: QueryBuilder, field: String) -> Self {
        Self {
            query_builder,
            field,
        }
    }
    
    /// Filter for equality
    pub fn equals(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::Equal,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for inequality
    pub fn not_equals(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::NotEqual,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for greater than
    pub fn greater_than(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::GreaterThan,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for greater than or equal
    pub fn greater_than_or_equal(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::GreaterThanOrEqual,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for less than
    pub fn less_than(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::LessThan,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for less than or equal
    pub fn less_than_or_equal(self, value: impl Into<FilterValue>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::LessThanOrEqual,
            value: value.into(),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for contains substring
    pub fn contains(self, value: impl Into<String>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::Contains,
            value: FilterValue::String(value.into()),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for starts with substring
    pub fn starts_with(self, value: impl Into<String>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::StartsWith,
            value: FilterValue::String(value.into()),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for ends with substring
    pub fn ends_with(self, value: impl Into<String>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::EndsWith,
            value: FilterValue::String(value.into()),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for value in a list
    pub fn in_list<T: Into<FilterValue>>(self, values: Vec<T>) -> QueryBuilder {
        let filter_values = values.into_iter()
            .map(|v| v.into())
            .collect();
        
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::In,
            value: FilterValue::Array(filter_values),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for value not in a list
    pub fn not_in_list<T: Into<FilterValue>>(self, values: Vec<T>) -> QueryBuilder {
        let filter_values = values.into_iter()
            .map(|v| v.into())
            .collect();
        
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::NotIn,
            value: FilterValue::Array(filter_values),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for field is null
    pub fn is_null(self) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::IsNull,
            value: FilterValue::Null,
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter for field is not null
    pub fn is_not_null(self) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::IsNotNull,
            value: FilterValue::Null,
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
    
    /// Filter by regex pattern
    pub fn matches_regex(self, pattern: impl Into<String>) -> QueryBuilder {
        let condition = FilterCondition {
            field: self.field,
            operator: FilterOperator::Regex,
            value: FilterValue::String(pattern.into()),
        };
        
        let filter = FilterExpression::Condition(condition);
        self.query_builder.add_filter(filter)
    }
}

/// Builder for constructing sort specifications
#[derive(Debug, Clone)]
pub struct SortBuilder {
    /// The query builder this sort will be added to
    query_builder: QueryBuilder,
}

impl SortBuilder {
    /// Create a new sort builder
    pub fn new(query_builder: QueryBuilder) -> Self {
        Self { query_builder }
    }
    
    /// Add an ascending sort
    pub fn asc(self, field: impl Into<String>) -> Self {
        let query_builder = self.query_builder.sort(field, SortDirection::Ascending);
        Self { query_builder }
    }
    
    /// Add a descending sort
    pub fn desc(self, field: impl Into<String>) -> Self {
        let query_builder = self.query_builder.sort(field, SortDirection::Descending);
        Self { query_builder }
    }
    
    /// Build the final query
    pub fn build(self) -> ResourceQuery {
        self.query_builder.build()
    }
} 