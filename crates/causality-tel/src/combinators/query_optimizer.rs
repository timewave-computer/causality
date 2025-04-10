//! Query Optimizer Module for TEL
//!
//! This module implements advanced query optimization techniques for the TEL query system,
//! including predicate pushdown, index utilization, join optimization, and cost-based
//! query planning.

use std::collections::{HashMap, HashSet};
use crate::combinators::Literal;
use super::query::{Query, Filter, FilterOperator, QueryError};

/// Statistics about a data source
#[derive(Debug, Clone)]
pub struct SourceStatistics {
    /// Estimated number of rows in the source
    pub row_count: usize,
    /// Available indexes on the source
    pub indexes: HashMap<String, IndexType>,
    /// Cardinality estimates for fields (number of distinct values)
    pub cardinality: HashMap<String, usize>,
    /// Average row size in bytes
    pub avg_row_size: usize,
}

/// Types of indexes available on fields
#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    /// Primary key index (unique)
    PrimaryKey,
    /// Unique index
    Unique,
    /// Non-unique standard index
    Standard,
    /// Full-text search index
    FullText,
    /// Spatial index
    Spatial,
}

/// Methods for query optimization
pub trait QueryOptimizer: Send + Sync {
    /// Optimize a query based on available statistics
    fn optimize(&self, query: &Query, stats: Option<&SourceStatistics>) -> Result<Query, QueryError>;
}

/// A basic query optimizer
pub struct BasicQueryOptimizer;

impl BasicQueryOptimizer {
    /// Create a new basic query optimizer
    pub fn new() -> Self {
        Self
    }
}

impl QueryOptimizer for BasicQueryOptimizer {
    fn optimize(&self, query: &Query, stats: Option<&SourceStatistics>) -> Result<Query, QueryError> {
        let mut optimized = query.clone();
        
        // Apply optimizations in sequence
        self.simplify_filters(&mut optimized);
        self.optimize_projections(&mut optimized);
        self.optimize_sorting(&mut optimized);
        
        // If statistics are available, apply more advanced optimizations
        if let Some(stats) = stats {
            self.optimize_for_indexes(&mut optimized, stats);
        }
        
        Ok(optimized)
    }
}

impl BasicQueryOptimizer {
    /// Simplify and reorganize filters for better performance
    fn simplify_filters(&self, query: &mut Query) {
        // Group filters by field for possible combination
        let mut filters_by_field: HashMap<String, Vec<&Filter>> = HashMap::new();
        
        for filter in &query.filters {
            filters_by_field
                .entry(filter.field.clone())
                .or_insert_with(Vec::new)
                .push(filter);
        }
        
        // Look for redundant or combinable filters
        let mut new_filters = Vec::new();
        
        for (field, field_filters) in filters_by_field {
            // Check for equality filters (which make other filters on the same field redundant)
            let has_equality = field_filters.iter().any(|f| f.operator == FilterOperator::Equals);
            
            if has_equality {
                // If there's an equality filter, just keep one of those
                if let Some(eq_filter) = field_filters.iter().find(|f| f.operator == FilterOperator::Equals) {
                    new_filters.push(Filter {
                        field: field.clone(),
                        operator: FilterOperator::Equals,
                        value: eq_filter.value.clone(),
                    });
                }
                continue;
            }
            
            // Look for range filters that can be combined
            let mut min_value = None;
            let mut max_value = None;
            
            for filter in field_filters {
                match filter.operator {
                    FilterOperator::GreaterThan | FilterOperator::GreaterThanOrEqual => {
                        if let Literal::Int(value) = filter.value {
                            if min_value.is_none() || value > min_value.unwrap() {
                                min_value = Some(value);
                            }
                        }
                    }
                    FilterOperator::LessThan | FilterOperator::LessThanOrEqual => {
                        if let Literal::Int(value) = filter.value {
                            if max_value.is_none() || value < max_value.unwrap() {
                                max_value = Some(value);
                            }
                        }
                    }
                    _ => {
                        // Keep other filters as is
                        new_filters.push(Filter {
                            field: field.clone(),
                            operator: filter.operator.clone(),
                            value: filter.value.clone(),
                        });
                    }
                }
            }
            
            // Add combined range filters if applicable
            if let Some(min) = min_value {
                new_filters.push(Filter {
                    field: field.clone(),
                    operator: FilterOperator::GreaterThanOrEqual,
                    value: Literal::Int(min),
                });
            }
            
            if let Some(max) = max_value {
                new_filters.push(Filter {
                    field: field.clone(),
                    operator: FilterOperator::LessThanOrEqual,
                    value: Literal::Int(max),
                });
            }
        }
        
        // Update query with simplified filters
        query.filters = new_filters;
    }
    
    /// Optimize projections to only include needed fields
    fn optimize_projections(&self, query: &mut Query) {
        if query.projections.is_none() || query.projections.as_ref().unwrap().is_empty() {
            return;
        }
        
        // Get all fields used in filters and sorts
        let mut used_fields = HashSet::new();
        
        // Add fields from filters
        for filter in &query.filters {
            used_fields.insert(filter.field.clone());
        }
        
        // Add fields from sorts
        if let Some(sorts) = &query.sorts {
            for sort in sorts {
                used_fields.insert(sort.field.clone());
            }
        }
        
        // If projections don't include fields needed for filtering/sorting,
        // we need to add them to the projections
        if let Some(projections) = &mut query.projections {
            let projected_fields: HashSet<_> = projections
                .iter()
                .map(|p| p.field.clone())
                .collect();
            
            for field in used_fields {
                if !projected_fields.contains(&field) {
                    projections.push(super::query::Projection {
                        field: field.clone(),
                        alias: None,
                    });
                }
            }
        }
    }
    
    /// Optimize sorting based on available indexes
    fn optimize_sorting(&self, query: &mut Query) {
        // Without statistics, we can't do much here
        if query.sorts.is_none() || query.sorts.as_ref().unwrap().is_empty() {
            return;
        }
        
        // But we can check for unnecessary sorts when there's a LIMIT clause
        // without an OFFSET - if there's a high selectivity filter (like equals)
        
        // Simple heuristic: if there's an equality filter and a small limit,
        // we can potentially skip sorting
        let has_equality_filter = query.filters.iter().any(|f| f.operator == FilterOperator::Equals);
        let has_small_limit = query.limit.map_or(false, |limit| limit <= 10);
        let has_no_offset = query.offset.is_none() || query.offset == Some(0);
        
        if has_equality_filter && has_small_limit && has_no_offset {
            // For certain cases, sorting can be eliminated or simplified
            // This is a heuristic and might need refinement based on real-world data
            
            // For now, we'll keep the sorts in place as this is just a simplistic example
        }
    }
    
    /// Optimize query based on available indexes
    fn optimize_for_indexes(&self, query: &mut Query, stats: &SourceStatistics) {
        if query.filters.is_empty() {
            return;
        }
        
        // Prioritize filters that can use indexes
        let mut indexed_filters = Vec::new();
        let mut non_indexed_filters = Vec::new();
        
        for filter in &query.filters {
            if stats.indexes.contains_key(&filter.field) {
                indexed_filters.push(filter.clone());
            } else {
                non_indexed_filters.push(filter.clone());
            }
        }
        
        // Sort indexed filters by selectivity (primary key first, then unique, etc.)
        indexed_filters.sort_by(|a, b| {
            let a_index_type = stats.indexes.get(&a.field).unwrap();
            let b_index_type = stats.indexes.get(&b.field).unwrap();
            
            // Primary keys are most selective
            if a_index_type == &IndexType::PrimaryKey && b_index_type != &IndexType::PrimaryKey {
                return std::cmp::Ordering::Less;
            }
            if b_index_type == &IndexType::PrimaryKey && a_index_type != &IndexType::PrimaryKey {
                return std::cmp::Ordering::Greater;
            }
            
            // Then unique indexes
            if a_index_type == &IndexType::Unique && b_index_type != &IndexType::Unique {
                return std::cmp::Ordering::Less;
            }
            if b_index_type == &IndexType::Unique && a_index_type != &IndexType::Unique {
                return std::cmp::Ordering::Greater;
            }
            
            // Then consider cardinality if available
            if let (Some(a_card), Some(b_card)) = (
                stats.cardinality.get(&a.field),
                stats.cardinality.get(&b.field),
            ) {
                // Higher cardinality means more selective
                return b_card.cmp(a_card);
            }
            
            // Default to keeping original order
            std::cmp::Ordering::Equal
        });
        
        // Prioritize equality filters for indexed fields
        let mut equality_indexed_filters = Vec::new();
        let mut other_indexed_filters = Vec::new();
        
        for filter in indexed_filters {
            if filter.operator == FilterOperator::Equals {
                equality_indexed_filters.push(filter);
            } else {
                other_indexed_filters.push(filter);
            }
        }
        
        // Reorder filters: equality indexed first, then other indexed, then non-indexed
        let mut reordered_filters = Vec::new();
        reordered_filters.extend(equality_indexed_filters);
        reordered_filters.extend(other_indexed_filters);
        reordered_filters.extend(non_indexed_filters);
        
        query.filters = reordered_filters;
    }
}

/// An advanced query optimizer with more sophisticated strategies
pub struct AdvancedQueryOptimizer {
    /// Cost model for query operations
    cost_model: QueryCostModel,
}

impl AdvancedQueryOptimizer {
    /// Create a new advanced query optimizer
    pub fn new() -> Self {
        Self {
            cost_model: QueryCostModel::default(),
        }
    }
    
    /// With a custom cost model
    pub fn with_cost_model(cost_model: QueryCostModel) -> Self {
        Self { cost_model }
    }
}

impl QueryOptimizer for AdvancedQueryOptimizer {
    fn optimize(&self, query: &Query, stats: Option<&SourceStatistics>) -> Result<Query, QueryError> {
        let mut optimized = query.clone();
        
        // Apply basic optimizations first
        let basic_optimizer = BasicQueryOptimizer::new();
        basic_optimizer.simplify_filters(&mut optimized);
        basic_optimizer.optimize_projections(&mut optimized);
        
        // Apply additional advanced optimizations
        self.reorder_joins(&mut optimized, stats);
        
        // Apply index-based optimizations if statistics are available
        if let Some(stats) = stats {
            basic_optimizer.optimize_for_indexes(&mut optimized, stats);
            self.apply_index_based_plan(&mut optimized, stats);
        }
        
        Ok(optimized)
    }
}

impl AdvancedQueryOptimizer {
    /// Reorder joins for optimal execution
    fn reorder_joins(&self, query: &mut Query, stats: Option<&SourceStatistics>) {
        // This is a placeholder for join reordering logic
        // In a real implementation, this would analyze join conditions and
        // reorder them based on estimated costs
        
        // Example logic (simplified):
        // 1. Identify join conditions in the query
        // 2. Estimate the cost of different join orders
        // 3. Choose the lowest cost join order
        
        // Since TEL's query model doesn't explicitly represent joins yet,
        // this is left as a placeholder for future implementation
    }
    
    /// Apply an index-based execution plan
    fn apply_index_based_plan(&self, query: &mut Query, stats: &SourceStatistics) {
        // Another placeholder for more sophisticated index-based planning
        // This would include:
        // - Identifying index intersection opportunities
        // - Choosing covering indexes when available
        // - Index-only scans for queries that only need indexed fields
    }
    
    /// Estimate the cost of a query
    fn estimate_cost(&self, query: &Query, stats: &SourceStatistics) -> f64 {
        // Example cost estimation logic
        
        // Base cost for scanning the source
        let mut cost = self.cost_model.base_scan_cost * stats.row_count as f64;
        
        // Adjust for filters
        let selectivity = self.estimate_filter_selectivity(query, stats);
        cost *= selectivity;
        
        // Adjust for sorting if needed
        if let Some(sorts) = &query.sorts {
            if !sorts.is_empty() {
                // Sorting is O(n log n)
                let n = stats.row_count as f64 * selectivity;
                cost += self.cost_model.sort_cost * n * n.log2();
            }
        }
        
        cost
    }
    
    /// Estimate the combined selectivity of all filters
    fn estimate_filter_selectivity(&self, query: &Query, stats: &SourceStatistics) -> f64 {
        if query.filters.is_empty() {
            return 1.0; // No filters means all rows are selected
        }
        
        // Simple multiplicative model for filter selectivity
        // This is simplified and doesn't account for correlations between fields
        let mut selectivity = 1.0;
        
        for filter in &query.filters {
            let field_selectivity = match filter.operator {
                FilterOperator::Equals => {
                    if let Some(cardinality) = stats.cardinality.get(&filter.field) {
                        // For equality, selectivity is 1/cardinality
                        1.0 / *cardinality as f64
                    } else {
                        // Default guess for equality
                        0.1
                    }
                }
                FilterOperator::GreaterThan | FilterOperator::LessThan => {
                    // Range queries typically select more rows
                    0.3
                }
                FilterOperator::GreaterThanOrEqual | FilterOperator::LessThanOrEqual => {
                    0.35
                }
                FilterOperator::Contains | FilterOperator::StartsWith | FilterOperator::EndsWith => {
                    // Text searches can be quite selective
                    0.05
                }
                _ => 0.2, // Default for other operators
            };
            
            selectivity *= field_selectivity;
        }
        
        // Ensure we don't go below a minimum selectivity
        selectivity.max(0.001)
    }
}

/// Cost model parameters for query optimization
#[derive(Debug, Clone)]
pub struct QueryCostModel {
    /// Base cost for a full table scan (per row)
    pub base_scan_cost: f64,
    /// Cost for evaluating a filter (per row)
    pub filter_cost: f64,
    /// Cost for sorting (per row)
    pub sort_cost: f64,
    /// Cost for a join operation (per row)
    pub join_cost: f64,
    /// Cost of using an index (per lookup)
    pub index_cost: f64,
}

impl Default for QueryCostModel {
    fn default() -> Self {
        Self {
            base_scan_cost: 1.0,
            filter_cost: 0.1,
            sort_cost: 0.5,
            join_cost: 2.0,
            index_cost: 0.2,
        }
    }
}

/// Generate source statistics for a given source
pub fn generate_stats_for_source(
    source: &str,
    sample_data: Option<&[HashMap<String, Literal>]>
) -> SourceStatistics {
    let mut stats = SourceStatistics {
        row_count: 1000, // Default assumption
        indexes: HashMap::new(),
        cardinality: HashMap::new(),
        avg_row_size: 256, // Default assumption
    };
    
    // If sample data is provided, use it to generate better statistics
    if let Some(data) = sample_data {
        stats.row_count = data.len();
        
        if !data.is_empty() {
            // Get all field names from the first record
            let field_names: Vec<String> = data[0].keys().cloned().collect();
            
            // Estimate cardinality for each field
            for field in field_names {
                let distinct_values: HashSet<_> = data.iter()
                    .filter_map(|record| record.get(&field).cloned())
                    .collect();
                
                stats.cardinality.insert(field.clone(), distinct_values.len());
                
                // Assume ID or key fields might have indexes
                if field.ends_with("id") || field.ends_with("_id") || field == "id" {
                    stats.indexes.insert(field, IndexType::Standard);
                }
            }
            
            // Estimate average row size
            let total_size: usize = data.iter()
                .map(|record| {
                    record.iter()
                        .map(|(k, v)| {
                            k.len() + match v {
                                Literal::Int(_) => 8,
                                Literal::Float(_) => 8,
                                Literal::String(s) => s.len(),
                                Literal::Bool(_) => 1,
                                Literal::Null => 0,
                                Literal::List(l) => l.len() * 8,
                                Literal::Map(m) => m.len() * 16,
                            }
                        })
                        .sum::<usize>()
                })
                .sum();
            
            stats.avg_row_size = if data.is_empty() { 
                256 
            } else { 
                total_size / data.len() 
            };
        }
    }
    
    // Add source-specific customizations if needed
    if source.ends_with("s") {
        // For tables with plural names, assume they might have more rows
        stats.row_count = stats.row_count.max(5000);
        
        // Common primary key patterns
        if !stats.indexes.contains_key("id") {
            stats.indexes.insert("id".to_string(), IndexType::PrimaryKey);
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinators::query::SortDirection;
    
    #[test]
    fn test_basic_optimizer() {
        // Create a test query with redundant filters
        let mut query = Query::new("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(18))
            .add_filter("age", FilterOperator::LessThan, Literal::Int(65))
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21));
        
        // Create the optimizer
        let optimizer = BasicQueryOptimizer::new();
        
        // Optimize filters
        optimizer.simplify_filters(&mut query);
        
        // Should have just 2 filters now (age >= 21 and age <= 65)
        assert_eq!(query.filters.len(), 2);
        
        // Verify the filters
        let has_ge_21 = query.filters.iter().any(|f| 
            f.field == "age" && 
            f.operator == FilterOperator::GreaterThanOrEqual && 
            f.value == Literal::Int(21)
        );
        
        let has_le_65 = query.filters.iter().any(|f| 
            f.field == "age" && 
            f.operator == FilterOperator::LessThanOrEqual && 
            f.value == Literal::Int(65)
        );
        
        assert!(has_ge_21, "Should have age >= 21 filter");
        assert!(has_le_65, "Should have age <= 65 filter");
    }
    
    #[test]
    fn test_optimize_projections() {
        // Create a query with projections and filters on different fields
        let mut query = Query::new("users");
        
        // Add just name projection
        query.projections = Some(vec![
            super::super::query::Projection {
                field: "name".to_string(),
                alias: None,
            }
        ]);
        
        // Add filter on age
        query.filters = vec![
            Filter {
                field: "age".to_string(),
                operator: FilterOperator::GreaterThan,
                value: Literal::Int(21),
            }
        ];
        
        // Add sort on email
        query.sorts = Some(vec![
            super::super::query::SortSpec {
                field: "email".to_string(),
                direction: SortDirection::Ascending,
            }
        ]);
        
        // Create the optimizer
        let optimizer = BasicQueryOptimizer::new();
        
        // Optimize projections
        optimizer.optimize_projections(&mut query);
        
        // Should have added projections for age and email
        assert_eq!(query.projections.as_ref().unwrap().len(), 3);
        
        let has_name = query.projections.as_ref().unwrap().iter()
            .any(|p| p.field == "name");
        
        let has_age = query.projections.as_ref().unwrap().iter()
            .any(|p| p.field == "age");
        
        let has_email = query.projections.as_ref().unwrap().iter()
            .any(|p| p.field == "email");
        
        assert!(has_name, "Should keep name projection");
        assert!(has_age, "Should add age projection for filter");
        assert!(has_email, "Should add email projection for sort");
    }
    
    #[test]
    fn test_index_optimization() {
        // Create a test query
        let mut query = Query::new("users")
            .add_filter("name", FilterOperator::Equals, Literal::String("John".to_string()))
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21))
            .add_filter("id", FilterOperator::Equals, Literal::String("12345".to_string()));
        
        // Create source statistics with indexes
        let mut stats = SourceStatistics {
            row_count: 10000,
            indexes: HashMap::new(),
            cardinality: HashMap::new(),
            avg_row_size: 256,
        };
        
        // Add indexes
        stats.indexes.insert("id".to_string(), IndexType::PrimaryKey);
        stats.indexes.insert("name".to_string(), IndexType::Standard);
        
        // Add cardinality estimates
        stats.cardinality.insert("id".to_string(), 10000);
        stats.cardinality.insert("name".to_string(), 1000);
        stats.cardinality.insert("age".to_string(), 100);
        
        // Create the optimizer
        let optimizer = BasicQueryOptimizer::new();
        
        // Optimize for indexes
        optimizer.optimize_for_indexes(&mut query, &stats);
        
        // ID should be first (primary key), then name (indexed), then age (not indexed)
        assert_eq!(query.filters[0].field, "id");
        assert_eq!(query.filters[1].field, "name");
        assert_eq!(query.filters[2].field, "age");
    }
    
    #[test]
    fn test_advanced_optimizer() {
        // Create a test query
        let query = Query::new("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21))
            .add_filter("name", FilterOperator::StartsWith, Literal::String("J".to_string()))
            .add_sort("created_at", SortDirection::Descending);
        
        // Generate statistics
        let stats = generate_stats_for_source("users", None);
        
        // Create the optimizer
        let optimizer = AdvancedQueryOptimizer::new();
        
        // Optimize the query
        let optimized = optimizer.optimize(&query, Some(&stats))
            .expect("Optimization failed");
        
        // Verify the result
        assert_eq!(optimized.source, "users");
        assert!(!optimized.filters.is_empty());
        
        // Check cost estimation
        let cost = optimizer.estimate_cost(&optimized, &stats);
        assert!(cost > 0.0, "Cost should be positive");
    }
} 