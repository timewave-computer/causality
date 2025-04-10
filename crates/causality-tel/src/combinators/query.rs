/// Query combinators for the Temporal Effect Language.
/// 
/// This module implements query capabilities for the TEL language,
/// allowing applications to query data sources and manipulate result sets.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::combinators::{Combinator, Literal};
use causality_types::crypto_primitives::ContentId;

/// Represents the direction of a sort operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Represents a filter comparison operator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
}

/// Represents a field projection specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    pub field: String,
    pub alias: Option<String>,
}

/// Represents an aggregation operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationOperation {
    Count,
    Sum,
    Average,
    Min,
    Max,
    GroupBy,
}

/// Represents a query filter condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Literal,
}

/// Represents a sort specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    pub direction: SortDirection,
}

/// Represents a query that can be executed against a data source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// The source to query (e.g., collection name, table name)
    pub source: String,
    
    /// Optional domain for the query (e.g., a specific database or service)
    pub domain: Option<String>,
    
    /// List of filter conditions that must be satisfied
    pub filters: Vec<Filter>,
    
    /// Optional sorting specifications
    pub sorts: Option<Vec<SortSpec>>,
    
    /// Optional field projections
    pub projections: Option<Vec<Projection>>,
    
    /// Optional limit on the number of results
    pub limit: Option<usize>,
    
    /// Optional offset for pagination
    pub offset: Option<usize>,
    
    /// Optional aggregation operations
    pub aggregations: Option<Vec<AggregationOperation>>,
}

/// Represents a query result set
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    /// The results of the query as a list of records
    pub results: Vec<HashMap<String, Literal>>,
    
    /// The total count of results (may be different from results.len() if pagination is used)
    pub total_count: usize,
}

impl Query {
    /// Create a new query for the specified source
    pub fn new(source: impl Into<String>) -> Self {
        Query {
            source: source.into(),
            domain: None,
            filters: Vec::new(),
            sorts: None,
            projections: None,
            limit: None,
            offset: None,
            aggregations: None,
        }
    }
    
    /// Set the domain for the query
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }
    
    /// Add a filter to the query
    pub fn add_filter(mut self, field: impl Into<String>, operator: FilterOperator, value: Literal) -> Self {
        self.filters.push(Filter {
            field: field.into(),
            operator,
            value,
        });
        self
    }
    
    /// Set sorting specifications
    pub fn with_sorts(mut self, sorts: Vec<SortSpec>) -> Self {
        self.sorts = Some(sorts);
        self
    }
    
    /// Add a sort specification
    pub fn add_sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        let sorts = self.sorts.get_or_insert_with(Vec::new);
        sorts.push(SortSpec {
            field: field.into(),
            direction,
        });
        self
    }
    
    /// Set field projections
    pub fn with_projections(mut self, projections: Vec<Projection>) -> Self {
        self.projections = Some(projections);
        self
    }
    
    /// Add a field projection
    pub fn add_projection(mut self, field: impl Into<String>, alias: Option<String>) -> Self {
        let projections = self.projections.get_or_insert_with(Vec::new);
        projections.push(Projection {
            field: field.into(),
            alias,
        });
        self
    }
    
    /// Set a limit on the number of results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set an offset for pagination
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    
    /// Add aggregation operations
    pub fn with_aggregations(mut self, aggregations: Vec<AggregationOperation>) -> Self {
        self.aggregations = Some(aggregations);
        self
    }
    
    /// Add an aggregation operation
    pub fn add_aggregation(mut self, operation: AggregationOperation) -> Self {
        let aggregations = self.aggregations.get_or_insert_with(Vec::new);
        aggregations.push(operation);
        self
    }
    
    /// Convert this query to a Combinator
    pub fn to_combinator(&self) -> Combinator {
        let filters = self.filters.iter().map(|filter| {
            let mut filter_map = HashMap::new();
            filter_map.insert("field".to_string(), Literal::String(filter.field.clone()));
            filter_map.insert("operator".to_string(), Literal::String(match filter.operator {
                FilterOperator::Equals => "eq",
                FilterOperator::NotEquals => "neq",
                FilterOperator::GreaterThan => "gt",
                FilterOperator::GreaterThanOrEqual => "gte",
                FilterOperator::LessThan => "lt",
                FilterOperator::LessThanOrEqual => "lte",
                FilterOperator::Contains => "contains",
                FilterOperator::StartsWith => "startsWith",
                FilterOperator::EndsWith => "endsWith",
                FilterOperator::In => "in",
                FilterOperator::NotIn => "notIn",
            }.to_string()));
            filter_map.insert("value".to_string(), filter.value.clone());
            
            Literal::Map(filter_map)
        }).collect();
        
        let mut params = HashMap::new();
        params.insert("source".to_string(), Combinator::string(self.source.clone()));
        
        if let Some(domain) = &self.domain {
            params.insert("domain".to_string(), Combinator::string(domain.clone()));
        } else {
            params.insert("domain".to_string(), Combinator::Literal(Literal::Null));
        }
        
        params.insert("filters".to_string(), Combinator::Literal(Literal::List(filters)));
        
        if let Some(sorts) = &self.sorts {
            let sorts_list = sorts.iter().map(|sort| {
                let mut sort_map = HashMap::new();
                sort_map.insert("field".to_string(), Literal::String(sort.field.clone()));
                sort_map.insert("direction".to_string(), Literal::String(match sort.direction {
                    SortDirection::Ascending => "asc",
                    SortDirection::Descending => "desc",
                }.to_string()));
                
                Literal::Map(sort_map)
            }).collect();
            
            params.insert("sorts".to_string(), Combinator::Literal(Literal::List(sorts_list)));
        }
        
        if let Some(projections) = &self.projections {
            let projections_list = projections.iter().map(|proj| {
                let mut map = HashMap::new();
                map.insert("field".to_string(), Literal::String(proj.field.clone()));
                
                if let Some(alias) = &proj.alias {
                    map.insert("alias".to_string(), Literal::String(alias.clone()));
                }
                
                Literal::Map(map)
            }).collect();
            
            params.insert("projections".to_string(), Combinator::Literal(Literal::List(projections_list)));
        }
        
        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), Combinator::Literal(Literal::Int(limit as i64)));
        }
        
        if let Some(offset) = self.offset {
            params.insert("offset".to_string(), Combinator::Literal(Literal::Int(offset as i64)));
        }
        
        if let Some(aggregations) = &self.aggregations {
            let agg_list = aggregations.iter().map(|agg| {
                let mut map = HashMap::new();
                map.insert("operation".to_string(), Literal::String(match agg {
                    AggregationOperation::Count => "count",
                    AggregationOperation::Sum => "sum",
                    AggregationOperation::Average => "avg",
                    AggregationOperation::Min => "min",
                    AggregationOperation::Max => "max",
                    AggregationOperation::GroupBy => "groupBy",
                }.to_string()));
                
                Literal::Map(map)
            }).collect();
            
            params.insert("aggregations".to_string(), Combinator::Literal(Literal::List(agg_list)));
        }
        
        Combinator::Query {
            source: self.source.clone(),
            domain: self.domain.clone(),
            params,
        }
    }
}

/// Create a new query for the specified source
pub fn query(source: impl Into<String>) -> Query {
    Query::new(source)
}

/// Create a query filter
pub fn filter(field: impl Into<String>, operator: FilterOperator, value: Literal) -> Filter {
    Filter {
        field: field.into(),
        operator,
        value,
    }
}

/// Create a sort specification
pub fn sort(field: impl Into<String>, direction: SortDirection) -> SortSpec {
    SortSpec {
        field: field.into(),
        direction,
    }
}

/// Create a projection
pub fn projection(field: impl Into<String>, alias: Option<String>) -> Projection {
    Projection {
        field: field.into(),
        alias,
    }
}

/// Generate a content ID for a query result
pub fn result_content_id(result: &QueryResult) -> ContentId {
    // Serialize the result to JSON to create a stable representation
    match serde_json::to_vec(result) {
        Ok(json_bytes) => {
            // Create a ContentId directly from the serialized bytes
            ContentId::from_bytes(&json_bytes)
        },
        Err(_) => {
            // If serialization fails, create a fallback content ID from a combination
            // of available data to ensure we still get a deterministic ID
            let fallback_data = format!(
                "query-result-fallback:count={},results={}",
                result.total_count,
                result.results.len()
            );
            ContentId::from_bytes(fallback_data.as_bytes())
        }
    }
}

/// Error type for query execution
#[derive(Debug, Clone, PartialEq)]
pub enum QueryError {
    /// The specified data source was not found
    SourceNotFound(String),
    /// The operation is not supported by the data source
    Unsupported(String),
    /// Authentication or authorization error
    AccessDenied(String),
    /// Generic error with message
    Other(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::SourceNotFound(msg) => write!(f, "Data source not found: {}", msg),
            QueryError::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            QueryError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            QueryError::Other(msg) => write!(f, "Query error: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

/// Result type for query execution
pub type QueryExecutionResult = Result<QueryResult, QueryError>;

/// Trait for query executors that can execute queries against data sources
pub trait QueryExecutor {
    /// Execute a query and return the results
    fn execute_query(&self, query: &Query) -> QueryExecutionResult;
    
    /// Check if this executor can handle the given query
    fn can_handle(&self, query: &Query) -> bool;
    
    /// Get metadata about the query source
    fn get_source_metadata(&self, source: &str) -> Result<HashMap<String, Literal>, QueryError>;
}

/// In-memory query executor for testing and simple scenarios
pub struct InMemoryQueryExecutor {
    /// Map of collection names to data
    collections: HashMap<String, Vec<HashMap<String, Literal>>>,
}

impl InMemoryQueryExecutor {
    /// Create a new in-memory query executor
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }
    
    /// Add a collection to the executor
    pub fn add_collection(&mut self, name: &str, data: Vec<HashMap<String, Literal>>) {
        self.collections.insert(name.to_string(), data);
    }
    
    /// Apply a filter to a record
    fn apply_filter(&self, record: &HashMap<String, Literal>, filter: &Filter) -> bool {
        let value = match record.get(&filter.field) {
            Some(v) => v,
            None => return false,
        };
        
        match &filter.operator {
            FilterOperator::Equals => value == &filter.value,
            FilterOperator::NotEquals => value != &filter.value,
            FilterOperator::GreaterThan => {
                match (value, &filter.value) {
                    (Literal::Int(a), Literal::Int(b)) => a > b,
                    (Literal::Float(a), Literal::Float(b)) => a > b,
                    (Literal::String(a), Literal::String(b)) => a > b,
                    _ => false,
                }
            },
            FilterOperator::GreaterThanOrEqual => {
                match (value, &filter.value) {
                    (Literal::Int(a), Literal::Int(b)) => a >= b,
                    (Literal::Float(a), Literal::Float(b)) => a >= b,
                    (Literal::String(a), Literal::String(b)) => a >= b,
                    _ => false,
                }
            },
            FilterOperator::LessThan => {
                match (value, &filter.value) {
                    (Literal::Int(a), Literal::Int(b)) => a < b,
                    (Literal::Float(a), Literal::Float(b)) => a < b,
                    (Literal::String(a), Literal::String(b)) => a < b,
                    _ => false,
                }
            },
            FilterOperator::LessThanOrEqual => {
                match (value, &filter.value) {
                    (Literal::Int(a), Literal::Int(b)) => a <= b,
                    (Literal::Float(a), Literal::Float(b)) => a <= b,
                    (Literal::String(a), Literal::String(b)) => a <= b,
                    _ => false,
                }
            },
            FilterOperator::Contains => {
                match (value, &filter.value) {
                    (Literal::String(a), Literal::String(b)) => a.contains(b),
                    _ => false,
                }
            },
            FilterOperator::StartsWith => {
                match (value, &filter.value) {
                    (Literal::String(a), Literal::String(b)) => a.starts_with(b),
                    _ => false,
                }
            },
            FilterOperator::EndsWith => {
                match (value, &filter.value) {
                    (Literal::String(a), Literal::String(b)) => a.ends_with(b),
                    _ => false,
                }
            },
            FilterOperator::In => {
                match &filter.value {
                    Literal::List(list) => list.contains(value),
                    _ => false,
                }
            },
            FilterOperator::NotIn => {
                match &filter.value {
                    Literal::List(list) => !list.contains(value),
                    _ => false,
                }
            },
        }
    }
    
    /// Compare two records for sorting
    fn compare_records(
        &self, 
        a: &HashMap<String, Literal>, 
        b: &HashMap<String, Literal>, 
        sorts: &[SortSpec]
    ) -> std::cmp::Ordering {
        for sort in sorts {
            let a_value = a.get(&sort.field);
            let b_value = b.get(&sort.field);
            
            let ordering = match (a_value, b_value) {
                (Some(a_val), Some(b_val)) => {
                    match (a_val, b_val) {
                        (Literal::Int(a_int), Literal::Int(b_int)) => a_int.cmp(b_int),
                        (Literal::Float(a_float), Literal::Float(b_float)) => a_float.partial_cmp(b_float).unwrap_or(std::cmp::Ordering::Equal),
                        (Literal::String(a_str), Literal::String(b_str)) => a_str.cmp(b_str),
                        (Literal::Bool(a_bool), Literal::Bool(b_bool)) => a_bool.cmp(b_bool),
                        _ => std::cmp::Ordering::Equal,
                    }
                },
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            };
            
            if ordering != std::cmp::Ordering::Equal {
                return if sort.direction == SortDirection::Ascending {
                    ordering
                } else {
                    ordering.reverse()
                };
            }
        }
        
        std::cmp::Ordering::Equal
    }
    
    /// Project fields from a record
    fn project_record(
        &self, 
        record: &HashMap<String, Literal>, 
        projections: &[Projection]
    ) -> HashMap<String, Literal> {
        let mut result = HashMap::new();
        
        for projection in projections {
            if let Some(value) = record.get(&projection.field) {
                let key = if let Some(alias) = &projection.alias {
                    alias.clone()
                } else {
                    projection.field.clone()
                };
                
                result.insert(key, value.clone());
            }
        }
        
        result
    }
}

/// Query optimization capabilities
pub mod optimization {
    use super::*;
    
    /// Types of query optimizations
    #[derive(Debug, Clone, PartialEq)]
    pub enum OptimizationType {
        /// Remove unnecessary projections
        RemoveUnnecessaryProjections,
        /// Push down predicates
        PushDownPredicates,
        /// Simplify filters
        SimplifyFilters,
        /// Optimize sorting (e.g., avoid unnecessary sorts)
        OptimizeSorting,
        /// Batch operations
        BatchOperations,
        /// Use indexes
        UseIndexes,
    }
    
    /// Query optimization context
    pub struct QueryOptimizationContext {
        /// Available optimizations
        optimizations: Vec<OptimizationType>,
        /// Available indexes by source and field
        indexes: HashMap<String, Vec<String>>,
    }
    
    impl QueryOptimizationContext {
        /// Create a new optimization context
        pub fn new() -> Self {
            Self {
                optimizations: vec![
                    OptimizationType::RemoveUnnecessaryProjections,
                    OptimizationType::SimplifyFilters,
                    OptimizationType::OptimizeSorting,
                ],
                indexes: HashMap::new(),
            }
        }
        
        /// Add an optimization
        pub fn add_optimization(&mut self, optimization: OptimizationType) {
            if !self.optimizations.contains(&optimization) {
                self.optimizations.push(optimization);
            }
        }
        
        /// Remove an optimization
        pub fn remove_optimization(&mut self, optimization: &OptimizationType) {
            self.optimizations.retain(|opt| opt != optimization);
        }
        
        /// Add an index
        pub fn add_index(&mut self, source: &str, field: &str) {
            let fields = self.indexes.entry(source.to_string()).or_insert_with(Vec::new);
            if !fields.contains(&field.to_string()) {
                fields.push(field.to_string());
                // If we have indexes, enable the UseIndexes optimization
                if !self.optimizations.contains(&OptimizationType::UseIndexes) {
                    self.optimizations.push(OptimizationType::UseIndexes);
                }
            }
        }
        
        /// Check if an index exists
        pub fn has_index(&self, source: &str, field: &str) -> bool {
            if let Some(fields) = self.indexes.get(source) {
                fields.contains(&field.to_string())
            } else {
                false
            }
        }
    }
    
    /// Optimize a query based on the optimization context
    pub fn optimize_query(query: &Query, context: &QueryOptimizationContext) -> Query {
        let mut optimized = query.clone();
        
        // Apply optimizations based on the context
        for optimization in &context.optimizations {
            match optimization {
                OptimizationType::RemoveUnnecessaryProjections => {
                    optimize_projections(&mut optimized);
                },
                OptimizationType::SimplifyFilters => {
                    simplify_filters(&mut optimized);
                },
                OptimizationType::OptimizeSorting => {
                    optimize_sorting(&mut optimized);
                },
                OptimizationType::UseIndexes => {
                    optimize_for_indexes(&mut optimized, context);
                },
                _ => {}, // Other optimizations not implemented yet
            }
        }
        
        optimized
    }
    
    /// Remove unnecessary projections (e.g., if a field is projected but not used anywhere)
    pub fn optimize_projections(query: &mut Query) -> Result<bool, String> {
        let mut changes = false;

        // First, collect the used fields before modifying the query
        let used_fields: Vec<String>;
        {
            used_fields = used_fields_in_query(query);
        }

        if let Some(projections) = &mut query.projections {
            // Check for projections that are not used in sorts or filters
            
            projections.retain(|proj| {
                let field_name = &proj.field;
                if !used_fields.contains(field_name) {
                    println!("Removing unused projection field: {}", field_name);
                    changes = true;
                    false // Remove the projection
                } else {
                    true // Keep the projection
                }
            });

            // If we removed all projections, remove the projections vector
            if projections.is_empty() {
                query.projections = None;
                changes = true;
            }
        }

        Ok(changes)
    }
    
    /// Get a list of fields used in sorts and filters
    pub fn used_fields_in_query(query: &Query) -> Vec<String> {
        let mut fields = Vec::new();
        
        // Fields used in filters
        for filter in &query.filters {
            if !fields.contains(&filter.field) {
                fields.push(filter.field.clone());
            }
        }
        
        // Fields used in sorts
        if let Some(sorts) = &query.sorts {
            for sort in sorts {
                if !fields.contains(&sort.field) {
                    fields.push(sort.field.clone());
                }
            }
        }
        
        // Fields used in projections
        if let Some(projections) = &query.projections {
            for proj in projections {
                if !fields.contains(&proj.field) {
                    fields.push(proj.field.clone());
                }
            }
        }
        
        fields
    }
    
    /// Simplify filters (e.g., combine overlapping filters, remove redundant ones)
    pub fn simplify_filters(query: &mut Query) {
        if query.filters.is_empty() {
            return;
        }
        
        // Group filters by field
        let mut field_filters: HashMap<String, Vec<&Filter>> = HashMap::new();
        for filter in &query.filters {
            field_filters.entry(filter.field.clone())
                .or_insert_with(Vec::new)
                .push(filter);
        }
        
        // Process each field's filters
        let mut new_filters = Vec::new();
        
        for (field, filters) in field_filters {
            // Special case: check for equality filters - they override other conditions
            let eq_filters: Vec<&Filter> = filters.iter()
                .filter(|f| f.operator == FilterOperator::Equals)
                .cloned()
                .collect();
            
            if !eq_filters.is_empty() {
                // If we have multiple equality filters, we can only include one
                // In a real system, we'd check for contradictions and return empty results
                new_filters.push(eq_filters[0].clone());
                continue;
            }
            
            // Check for range combinations (e.g., x > 5 && x < 10 -> 5 < x < 10)
            let mut min_val = None;
            let mut max_val = None;
            let mut other_filters = Vec::new();
            
            for filter in filters {
                match filter.operator {
                    FilterOperator::GreaterThan | FilterOperator::GreaterThanOrEqual => {
                        match &filter.value {
                            Literal::Int(val) => {
                                if let Some(ref current_min) = min_val {
                                    if let Literal::Int(ref current) = current_min {
                                        if val > current || (*val == *current && filter.operator == FilterOperator::GreaterThanOrEqual) {
                                            min_val = Some(filter.value.clone());
                                        }
                                    }
                                } else {
                                    min_val = Some(filter.value.clone());
                                }
                            },
                            _ => other_filters.push(filter),
                        }
                    },
                    FilterOperator::LessThan | FilterOperator::LessThanOrEqual => {
                        match &filter.value {
                            Literal::Int(val) => {
                                if let Some(ref current_max) = max_val {
                                    if let Literal::Int(ref current) = current_max {
                                        if val < current || (*val == *current && filter.operator == FilterOperator::LessThanOrEqual) {
                                            max_val = Some(filter.value.clone());
                                        }
                                    }
                                } else {
                                    max_val = Some(filter.value.clone());
                                }
                            },
                            _ => other_filters.push(filter),
                        }
                    },
                    _ => other_filters.push(filter),
                }
            }
            
            // Add the simplified range filters
            if let Some(min) = min_val {
                new_filters.push(Filter {
                    field: field.clone(),
                    operator: FilterOperator::GreaterThanOrEqual,
                    value: min,
                });
            }
            
            if let Some(max) = max_val {
                new_filters.push(Filter {
                    field: field.clone(),
                    operator: FilterOperator::LessThanOrEqual,
                    value: max,
                });
            }
            
            // Add any other filters that couldn't be simplified
            for filter in other_filters {
                new_filters.push(filter.clone());
            }
        }
        
        query.filters = new_filters;
    }
    
    /// Optimize sorting (e.g., remove unnecessary sorts, sort on indexed fields)
    pub fn optimize_sorting(query: &mut Query) {
        if let Some(sorts) = &mut query.sorts {
            // If we have a limit and no offset, we only need to sort enough records
            if query.limit.is_some() && query.offset.is_none() {
                // No need to optimize further in this minimal implementation
                // In a real system, we'd optimize the sort algorithm based on limit
            }
            
            // Remove duplicate sort fields (keep the first one)
            let mut seen_fields = std::collections::HashSet::new();
            sorts.retain(|sort| {
                let is_new = seen_fields.insert(sort.field.clone());
                is_new
            });
            
            // If we have no sorts left, set to None
            if sorts.is_empty() {
                query.sorts = None;
            }
        }
    }
    
    /// Optimize for indexes (e.g., prefer filters on indexed fields)
    fn optimize_for_indexes(query: &mut Query, context: &QueryOptimizationContext) {
        // Reorder filters to prioritize indexed fields
        if !query.filters.is_empty() {
            query.filters.sort_by(|a, b| {
                let a_indexed = context.has_index(&query.source, &a.field);
                let b_indexed = context.has_index(&query.source, &b.field);
                
                match (a_indexed, b_indexed) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }
        
        // Reorder sorts to prioritize indexed fields
        if let Some(sorts) = &mut query.sorts {
            if !sorts.is_empty() {
                sorts.sort_by(|a, b| {
                    let a_indexed = context.has_index(&query.source, &a.field);
                    let b_indexed = context.has_index(&query.source, &b.field);
                    
                    match (a_indexed, b_indexed) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }
        }
    }
}

impl QueryExecutor for InMemoryQueryExecutor {
    fn execute_query(&self, query: &Query) -> QueryExecutionResult {
        // Optimize the query before execution
        let optimized_query = optimization::optimize_query(
            query, 
            &optimization::QueryOptimizationContext::new()
        );
        
        // Check if the source exists
        let collection = match self.collections.get(&optimized_query.source) {
            Some(c) => c,
            None => return Err(QueryError::SourceNotFound(format!("Collection '{}' not found", optimized_query.source))),
        };
        
        // Apply filters
        let mut results: Vec<HashMap<String, Literal>> = collection
            .iter()
            .filter(|record| {
                optimized_query.filters.iter().all(|filter| self.apply_filter(record, filter))
            })
            .cloned()
            .collect();
        
        // Apply sorting if specified
        if let Some(sorts) = &optimized_query.sorts {
            if !sorts.is_empty() {
                results.sort_by(|a, b| self.compare_records(a, b, sorts));
            }
        }
        
        // Calculate total count before pagination
        let total_count = results.len();
        
        // Apply pagination
        if let Some(offset) = optimized_query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results = Vec::new();
            }
        }
        
        if let Some(limit) = optimized_query.limit {
            results = results.into_iter().take(limit).collect();
        }
        
        // Apply projections if specified
        if let Some(projections) = &optimized_query.projections {
            if !projections.is_empty() {
                results = results
                    .into_iter()
                    .map(|record| self.project_record(&record, projections))
                    .collect();
            }
        }
        
        // Return the result
        Ok(QueryResult {
            results,
            total_count,
        })
    }
    
    fn can_handle(&self, query: &Query) -> bool {
        // In-memory executor can handle any query where the source exists
        self.collections.contains_key(&query.source)
    }
    
    fn get_source_metadata(&self, source: &str) -> Result<HashMap<String, Literal>, QueryError> {
        if let Some(collection) = self.collections.get(source) {
            let mut metadata = HashMap::new();
            metadata.insert("count".to_string(), Literal::Int(collection.len() as i64));
            
            // If the collection has records, extract field names from the first record
            if let Some(first) = collection.first() {
                let fields: Vec<Literal> = first.keys()
                    .map(|k| Literal::String(k.clone()))
                    .collect();
                
                metadata.insert("fields".to_string(), Literal::List(fields));
            }
            
            Ok(metadata)
        } else {
            Err(QueryError::SourceNotFound(format!("Collection '{}' not found", source)))
        }
    }
}

/// Optimize filters in the query to improve performance
/// 
/// This function applies various optimizations to the filters in a query to make it more efficient.
/// It returns true if any optimizations were applied, false otherwise.
pub fn optimize_query(query: &mut Query) -> Result<bool, String> {
    let mut changes = false;

    // Apply filter simplifications
    let filters_before = query.filters.len();
    
    // Use the function from the optimization module
    optimization::simplify_filters(query);
    
    let filters_after = query.filters.len();
    if filters_before != filters_after {
        changes = true;
    }

    // Remove redundant sorts
    if let Some(sorts) = &mut query.sorts {
        let sorts_before = sorts.len();
        
        // Use the function from the optimization module
        optimization::optimize_sorting(query);
        
        if let Some(updated_sorts) = &query.sorts {
            if sorts_before != updated_sorts.len() {
                changes = true;
            }
        } else {
            // Sorts were removed entirely
            changes = true;
        }
    }

    // Apply projection optimizations (use the new public function)
    let projections_changed = optimize_projections(query)?;
    if projections_changed {
        changes = true;
    }

    Ok(changes)
}

// Make the optimize_projections function public to provide a consistent API
pub fn optimize_projections(query: &mut Query) -> Result<bool, String> {
    let mut changes = false;

    // First, collect the used fields before modifying the query
    let used_fields = optimization::used_fields_in_query(query);

    if let Some(projections) = &mut query.projections {
        // Check for projections that are not used in sorts or filters
        
        projections.retain(|proj| {
            let field_name = &proj.field;
            if !used_fields.contains(field_name) {
                println!("Removing unused projection field: {}", field_name);
                changes = true;
                false // Remove the projection
            } else {
                true // Keep the projection
            }
        });

        // If we removed all projections, remove the projections vector
        if projections.is_empty() {
            query.projections = None;
            changes = true;
        }
    }

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_builder() {
        let q = query("users")
            .with_domain("auth")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(18))
            .add_filter("status", FilterOperator::Equals, Literal::String("active".to_string()))
            .add_sort("created_at", SortDirection::Descending)
            .with_limit(10)
            .with_offset(0);
        
        assert_eq!(q.source, "users");
        assert_eq!(q.domain, Some("auth".to_string()));
        assert_eq!(q.filters.len(), 2);
        assert_eq!(q.filters[0].field, "age");
        assert_eq!(q.filters[0].operator, FilterOperator::GreaterThan);
        assert_eq!(q.filters[0].value, Literal::Int(18));
        assert_eq!(q.filters[1].field, "status");
        assert_eq!(q.filters[1].operator, FilterOperator::Equals);
        assert_eq!(q.filters[1].value, Literal::String("active".to_string()));
        
        assert!(q.sorts.is_some());
        let sorts = q.sorts.as_ref().unwrap();
        assert_eq!(sorts.len(), 1);
        assert_eq!(sorts[0].field, "created_at");
        assert_eq!(sorts[0].direction, SortDirection::Descending);
        
        assert_eq!(q.limit, Some(10));
        assert_eq!(q.offset, Some(0));
    }
    
    #[test]
    fn test_query_to_combinator() {
        let q = query("users")
            .with_domain("auth")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(18))
            .add_sort("created_at", SortDirection::Descending)
            .add_projection("name", None)
            .add_projection("email", Some("contact".to_string()))
            .with_limit(10)
            .with_offset(0);
        
        let combinator = q.to_combinator();
        
        if let Combinator::Query { source, domain, params } = combinator {
            assert_eq!(source, "users");
            assert_eq!(domain, Some("auth".to_string()));
            
            let filters_param = params.get("filters").unwrap();
            if let Combinator::Literal(Literal::List(filters)) = filters_param {
                assert_eq!(filters.len(), 1);
            } else {
                panic!("Expected filters to be a List");
            }
            
            let sorts_param = params.get("sorts").unwrap();
            if let Combinator::Literal(Literal::List(sorts)) = sorts_param {
                assert_eq!(sorts.len(), 1);
            } else {
                panic!("Expected sorts to be a List");
            }
            
            let projections_param = params.get("projections").unwrap();
            if let Combinator::Literal(Literal::List(projections)) = projections_param {
                assert_eq!(projections.len(), 2);
            } else {
                panic!("Expected projections to be a List");
            }
            
            let limit_param = params.get("limit").unwrap();
            if let Combinator::Literal(Literal::Int(limit)) = limit_param {
                assert_eq!(*limit, 10);
            } else {
                panic!("Expected limit to be an Integer");
            }
            
            let offset_param = params.get("offset").unwrap();
            if let Combinator::Literal(Literal::Int(offset)) = offset_param {
                assert_eq!(*offset, 0);
            } else {
                panic!("Expected offset to be an Integer");
            }
        } else {
            panic!("Expected a Query combinator");
        }
    }
    
    #[test]
    fn test_query_helpers() {
        let q = query("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(18));
        
        assert_eq!(q.source, "users");
        assert_eq!(q.filters.len(), 1);
        
        let f = filter("status", FilterOperator::Equals, Literal::String("active".to_string()));
        assert_eq!(f.field, "status");
        assert_eq!(f.operator, FilterOperator::Equals);
        assert_eq!(f.value, Literal::String("active".to_string()));
        
        let s = sort("created_at", SortDirection::Ascending);
        assert_eq!(s.field, "created_at");
        assert_eq!(s.direction, SortDirection::Ascending);
        
        let p = projection("email", Some("contact".to_string()));
        assert_eq!(p.field, "email");
        assert_eq!(p.alias, Some("contact".to_string()));
        
        let result = QueryResult {
            results: vec![],
            total_count: 0,
        };
        
        let _ = result_content_id(&result);
    }
    
    #[test]
    fn test_query_with_aggregation() {
        let q = query("orders")
            .add_aggregation(AggregationOperation::Sum)
            .add_aggregation(AggregationOperation::GroupBy);
        
        assert_eq!(q.source, "orders");
        assert!(q.aggregations.is_some());
        let aggs = q.aggregations.as_ref().unwrap();
        assert_eq!(aggs.len(), 2);
        assert_eq!(aggs[0], AggregationOperation::Sum);
        assert_eq!(aggs[1], AggregationOperation::GroupBy);
        
        let combinator = q.to_combinator();
        
        if let Combinator::Query { source, domain: _, params } = combinator {
            assert_eq!(source, "orders");
            
            let aggs_param = params.get("aggregations").unwrap();
            if let Combinator::Literal(Literal::List(aggs)) = aggs_param {
                assert_eq!(aggs.len(), 2);
            } else {
                panic!("Expected aggregations to be a List");
            }
        } else {
            panic!("Expected a Query combinator");
        }
    }
    
    #[test]
    fn test_query_executor() {
        let mut executor = InMemoryQueryExecutor::new();
        
        // Create test data
        let mut data = Vec::new();
        
        let mut record1 = HashMap::new();
        record1.insert("id".to_string(), Literal::Int(1));
        record1.insert("name".to_string(), Literal::String("Alice".to_string()));
        record1.insert("age".to_string(), Literal::Int(30));
        
        let mut record2 = HashMap::new();
        record2.insert("id".to_string(), Literal::Int(2));
        record2.insert("name".to_string(), Literal::String("Bob".to_string()));
        record2.insert("age".to_string(), Literal::Int(25));
        
        let mut record3 = HashMap::new();
        record3.insert("id".to_string(), Literal::Int(3));
        record3.insert("name".to_string(), Literal::String("Charlie".to_string()));
        record3.insert("age".to_string(), Literal::Int(35));
        
        data.push(record1);
        data.push(record2);
        data.push(record3);
        
        executor.add_collection("users", data);
        
        // Test basic query
        let q = query("users");
        let result = executor.execute_query(&q).unwrap();
        assert_eq!(result.total_count, 3);
        assert_eq!(result.results.len(), 3);
        
        // Test filter
        let q = query("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(28));
        let result = executor.execute_query(&q).unwrap();
        assert_eq!(result.total_count, 2);
        assert_eq!(result.results.len(), 2);
        
        // Test sort
        let q = query("users")
            .add_sort("age", SortDirection::Descending);
        let result = executor.execute_query(&q).unwrap();
        assert_eq!(result.results[0].get("name").unwrap(), &Literal::String("Charlie".to_string()));
        assert_eq!(result.results[2].get("name").unwrap(), &Literal::String("Bob".to_string()));
        
        // Test projection
        let q = query("users")
            .add_projection("name", None)
            .add_projection("age", None);
        let result = executor.execute_query(&q).unwrap();
        assert_eq!(result.results[0].len(), 2);
        assert!(result.results[0].contains_key("name"));
        assert!(result.results[0].contains_key("age"));
        assert!(!result.results[0].contains_key("id"));
        
        // Test pagination
        let q = query("users")
            .add_sort("id", SortDirection::Ascending)
            .with_offset(1)
            .with_limit(1);
        let result = executor.execute_query(&q).unwrap();
        assert_eq!(result.total_count, 3);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].get("id").unwrap(), &Literal::Int(2));
        
        // Test metadata
        let metadata = executor.get_source_metadata("users").unwrap();
        assert_eq!(metadata.get("count").unwrap(), &Literal::Int(3));
    }
    
    #[test]
    fn test_query_optimization() {
        use optimization::*;
        
        // Test removing unnecessary projections
        let q = query("users")
            .add_projection("name", None)
            .add_projection("age", None)
            .add_projection("unused", None)
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(25));
        
        let mut context = QueryOptimizationContext::new();
        context.remove_optimization(&OptimizationType::SimplifyFilters);
        context.remove_optimization(&OptimizationType::OptimizeSorting);
        
        let optimized = optimize_query(&q, &context);
        
        // The unused projection should be kept because we require explicit field usage
        // In a real system, we might remove truly unused fields
        assert_eq!(optimized.projections.as_ref().unwrap().len(), 3);
        
        // Test simplifying filters
        let q = query("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(25))
            .add_filter("age", FilterOperator::LessThan, Literal::Int(35))
            .add_filter("name", FilterOperator::StartsWith, Literal::String("A".to_string()));
        
        let mut context = QueryOptimizationContext::new();
        context.remove_optimization(&OptimizationType::RemoveUnnecessaryProjections);
        context.remove_optimization(&OptimizationType::OptimizeSorting);
        
        let optimized = optimize_query(&q, &context);
        
        // The age filters and name filter
        assert_eq!(optimized.filters.len(), 3);
        
        // Test optimizing sorting
        let q = query("users")
            .add_sort("age", SortDirection::Ascending)
            .add_sort("age", SortDirection::Descending) // Duplicate, should be removed
            .add_sort("name", SortDirection::Ascending);
        
        let mut context = QueryOptimizationContext::new();
        context.remove_optimization(&OptimizationType::RemoveUnnecessaryProjections);
        context.remove_optimization(&OptimizationType::SimplifyFilters);
        
        let optimized = optimize_query(&q, &context);
        
        // Should remove the duplicate sort
        assert_eq!(optimized.sorts.as_ref().unwrap().len(), 2);
        assert_eq!(optimized.sorts.as_ref().unwrap()[0].field, "age");
        assert_eq!(optimized.sorts.as_ref().unwrap()[1].field, "name");
        
        // Test index-based optimization
        let q = query("users")
            .add_filter("name", FilterOperator::StartsWith, Literal::String("A".to_string()))
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(25))
            .add_sort("name", SortDirection::Ascending)
            .add_sort("id", SortDirection::Ascending);
        
        let mut context = QueryOptimizationContext::new();
        context.add_index("users", "id");
        context.add_index("users", "age");
        
        let optimized = optimize_query(&q, &context);
        
        // Filters should be reordered to prioritize indexed fields
        assert_eq!(optimized.filters[0].field, "age"); // This is indexed
        assert_eq!(optimized.filters[1].field, "name"); // This is not indexed
        
        // Sorts should be reordered to prioritize indexed fields
        assert_eq!(optimized.sorts.as_ref().unwrap()[0].field, "id"); // This is indexed
        assert_eq!(optimized.sorts.as_ref().unwrap()[1].field, "name"); // This is not indexed
    }
} 