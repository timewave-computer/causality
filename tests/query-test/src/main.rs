// This test imports components from the main TEL crate
use causality_tel::{Query, QueryResult, FilterOperator, SortDirection, result_content_id};
use causality_tel::combinators::{Combinator, Literal};
use std::collections::HashMap;

fn main() {
    println!("Testing the Query module");
    
    // Create a new query
    let query = Query::new("users")
        .add_filter("age", FilterOperator::GreaterThan, Literal::Int(30))
        .add_sort("name", SortDirection::Ascending)
        .add_projection("name", None)
        .add_projection("age", None)
        .with_limit(10)
        .with_offset(0);
    
    // Convert to combinator
    let combinator = query.to_combinator();
    
    // Print the combinator
    println!("Query Combinator: {:#?}", combinator);
    
    // Create a mock query result
    let mut record = HashMap::new();
    record.insert("name".to_string(), Literal::String("John".to_string()));
    record.insert("age".to_string(), Literal::Int(35));
    
    let query_result = QueryResult {
        results: vec![record],
        total_count: 1,
    };
    
    // Get content ID
    let content_id = result_content_id(&query_result);
    println!("Result Content ID: {}", content_id);
    
    println!("Query module test completed successfully!");
} 