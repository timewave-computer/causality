use causality_tel::combinators::{query::{Query, FilterOperator}, Combinator, Literal};
use std::collections::HashMap;

fn main() {
    println!("Testing the Query module");
    
    // Create a new query
    let mut query = Query::default();
    
    // Add filter
    query.add_filter("age".to_string(), FilterOperator::GreaterThan, Literal::Int(30));
    
    // Add sort
    query.add_sort("name".to_string(), true);
    
    // Add projection
    query.add_projection("name".to_string());
    query.add_projection("age".to_string());
    
    // Set limit and offset
    query.set_limit(10);
    query.set_offset(0);
    
    // Convert to combinator
    let combinator = query.to_combinator();
    
    // Print the combinator
    println!("Query Combinator: {:#?}", combinator);
    
    // Create a mock query result
    let mut result = HashMap::new();
    result.insert("name".to_string(), Literal::String("John".to_string()));
    result.insert("age".to_string(), Literal::Int(35));
    
    // Get content ID
    let content_id = query.result_content_id(&vec![result]);
    println!("Result Content ID: {}", content_id);
    
    println!("Query module test completed successfully!");
} 