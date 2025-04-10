//! Standalone tests for the query module

use crate::combinators::{Combinator, Literal};
use crate::combinators::query::{
    Query, FilterOperator, SortDirection, Projection, AggregationOperation, 
    query, filter, sort, projection, result_content_id
};
use std::collections::HashMap;
use causality_types::crypto_primitives::ContentId;

#[test]
fn test_query_builder() {
    // Create a basic query
    let query = Query::new("users")
        .with_domain("auth")
        .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21))
        .add_sort("name", SortDirection::Ascending)
        .with_limit(10)
        .with_offset(0);
    
    // Verify the query structure
    assert_eq!(query.source, "users");
    assert_eq!(query.domain, Some("auth".to_string()));
    assert_eq!(query.filters.len(), 1);
    assert_eq!(query.filters[0].field, "age");
    assert_eq!(query.filters[0].operator, FilterOperator::GreaterThan);
    assert_eq!(query.filters[0].value, Literal::Int(21));
    
    assert!(query.sorts.is_some());
    let sorts = query.sorts.as_ref().unwrap();
    assert_eq!(sorts.len(), 1);
    assert_eq!(sorts[0].field, "name");
    assert_eq!(sorts[0].direction, SortDirection::Ascending);
    
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, Some(0));
}

#[test]
fn test_query_to_combinator() {
    // Create a query with multiple filters and sort specifications
    let query = Query::new("users")
        .with_domain("auth")
        .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21))
        .add_filter("active", FilterOperator::Equals, Literal::Bool(true))
        .add_sort("name", SortDirection::Ascending)
        .add_sort("age", SortDirection::Descending)
        .add_projection("id", None)
        .add_projection("name", None)
        .with_limit(10)
        .with_offset(0);
    
    // Convert to a combinator
    let combinator = query.to_combinator();
    
    // Verify the combinator structure
    match combinator {
        Combinator::Query { source, domain, params } => {
            assert_eq!(source, "users");
            assert_eq!(domain, Some("auth".to_string()));
            
            // Check filters
            match params.get("filters") {
                Some(Combinator::Literal(Literal::List(filters))) => {
                    assert_eq!(filters.len(), 2);
                    
                    // Each filter should be a map
                    for filter in filters {
                        match filter {
                            Literal::Map(map) => {
                                // Map should have field, operator, and value
                                assert!(map.contains_key("field"));
                                assert!(map.contains_key("operator"));
                                assert!(map.contains_key("value"));
                            },
                            _ => panic!("Expected map literal for filter"),
                        }
                    }
                },
                _ => panic!("Expected list literal for filters"),
            }
            
            // Check sorts
            match params.get("sorts") {
                Some(Combinator::Literal(Literal::List(sort_specs))) => {
                    assert_eq!(sort_specs.len(), 2);
                    
                    // Each sort spec should be a map
                    for sort_spec in sort_specs {
                        match sort_spec {
                            Literal::Map(map) => {
                                // Map should have field and direction
                                assert!(map.contains_key("field"));
                                assert!(map.contains_key("direction"));
                            },
                            _ => panic!("Expected map literal for sort spec"),
                        }
                    }
                },
                _ => panic!("Expected list literal for sort specs"),
            }
            
            // Check projections
            match params.get("projections") {
                Some(Combinator::Literal(Literal::List(projections))) => {
                    assert_eq!(projections.len(), 2);
                    
                    // Each projection should be a map
                    for proj in projections {
                        match proj {
                            Literal::Map(map) => {
                                // Map should have field
                                assert!(map.contains_key("field"));
                            },
                            _ => panic!("Expected map literal for projection"),
                        }
                    }
                },
                _ => panic!("Expected list literal for projections"),
            }
            
            // Check limit
            match params.get("limit") {
                Some(Combinator::Literal(Literal::Int(n))) => assert_eq!(*n, 10),
                _ => panic!("Expected integer literal for limit"),
            }
            
            // Check offset
            match params.get("offset") {
                Some(Combinator::Literal(Literal::Int(n))) => assert_eq!(*n, 0),
                _ => panic!("Expected integer literal for offset"),
            }
        },
        _ => panic!("Expected Query combinator"),
    }
}

#[test]
fn test_query_helpers() {
    // Test the query helper
    let q = query("users");
    assert_eq!(q.source, "users");
    
    // Test filter helper
    let eq_filter = filter("name", FilterOperator::Equals, Literal::String("John".to_string()));
    assert_eq!(eq_filter.field, "name");
    assert_eq!(eq_filter.operator, FilterOperator::Equals);
    assert_eq!(eq_filter.value, Literal::String("John".to_string()));
    
    let gt_filter = filter("age", FilterOperator::GreaterThan, Literal::Int(21));
    assert_eq!(gt_filter.field, "age");
    assert_eq!(gt_filter.operator, FilterOperator::GreaterThan);
    assert_eq!(gt_filter.value, Literal::Int(21));
    
    // Test sort helper
    let asc_sort = sort("name", SortDirection::Ascending);
    assert_eq!(asc_sort.field, "name");
    assert_eq!(asc_sort.direction, SortDirection::Ascending);
    
    // Test projection helper
    let proj = projection("email", Some("contact".to_string()));
    assert_eq!(proj.field, "email");
    assert_eq!(proj.alias, Some("contact".to_string()));
}

#[test]
fn test_content_id_generation() {
    use crate::combinators::query::QueryResult;
    
    // Create a query result
    let mut results = Vec::new();
    let mut map = HashMap::new();
    map.insert("id".to_string(), Literal::String("1".to_string()));
    map.insert("name".to_string(), Literal::String("John".to_string()));
    results.push(map);
    
    let query_result = QueryResult {
        results,
        total_count: 1,
    };
    
    // Test content ID generation
    let content_id = result_content_id(&query_result);
    
    // ContentId should be valid
    assert!(content_id.to_string().len() > 0);
    
    // Creating a ContentId from the same data should yield the same result
    let serialized = serde_json::to_string(&query_result).expect("Failed to serialize");
    let expected_content_id = ContentId::from_bytes(serialized.as_bytes());
    assert_eq!(content_id, expected_content_id);
} 