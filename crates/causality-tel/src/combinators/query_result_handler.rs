//! Query Result Handler Module for TEL
//!
//! This module implements result set handling functionality for the TEL query system,
//! including transformations, exporters, and streaming capabilities.

use std::collections::HashMap;
use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use std::pin::Pin;

use tokio::sync::mpsc::{self, Receiver, Sender};
use futures::stream::{Stream, StreamExt};
use pin_utils::pin_mut;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use super::query::{QueryResult, QueryError};
use crate::combinators::Literal;

/// Traits and utilities for handling query results
pub mod handlers {
    use super::*;
    
    /// Trait for handling query results
    pub trait ResultHandler: Send + Sync {
        /// Process a query result
        fn handle_result(&mut self, result: &QueryResult) -> Result<(), QueryError>;
        
        /// Called when a result stream is completed
        fn complete(&self) -> Result<(), QueryError>;
    }
    
    /// A handler that writes results to a file
    pub struct FileResultHandler {
        /// The output file
        file: File,
        /// The format to write in
        format: OutputFormat,
    }
    
    impl FileResultHandler {
        /// Create a new file result handler
        pub fn new(path: impl AsRef<Path>, format: OutputFormat) -> io::Result<Self> {
            let file = File::create(path)?;
            Ok(Self { file, format })
        }
    }
    
    impl ResultHandler for FileResultHandler {
        fn handle_result(&mut self, result: &QueryResult) -> Result<(), QueryError> {
            match self.format {
                OutputFormat::Json => {
                    // Write JSON to the file
                    let json = serde_json::to_string_pretty(result)
                        .map_err(|e| QueryError::Other(format!("JSON serialization error: {}", e)))?;
                    
                    self.file.try_clone()
                        .map_err(|e| QueryError::Other(format!("File error: {}", e)))?
                        .write_all(json.as_bytes())
                        .map_err(|e| QueryError::Other(format!("File write error: {}", e)))?;
                },
                OutputFormat::Csv => {
                    // CSV header row (first time only)
                    if result.results.first().is_some() {
                        let keys: Vec<_> = result.results[0].keys().collect();
                        let header = keys.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(",");
                        
                        writeln!(self.file.try_clone()
                            .map_err(|e| QueryError::Other(format!("File error: {}", e)))?, 
                            "{}", header)
                            .map_err(|e| QueryError::Other(format!("File write error: {}", e)))?;
                        
                        // Write each row
                        for row in &result.results {
                            let values: Vec<String> = keys.iter()
                                .map(|k| match row.get(*k) {
                                    Some(Literal::String(s)) => format!("\"{}\"", s.replace("\"", "\"\"")),
                                    Some(lit) => format!("{:?}", lit),
                                    None => "".to_string(),
                                })
                                .collect();
                            
                            writeln!(self.file.try_clone()
                                .map_err(|e| QueryError::Other(format!("File error: {}", e)))?, 
                                "{}", values.join(","))
                                .map_err(|e| QueryError::Other(format!("File write error: {}", e)))?;
                        }
                    }
                },
            }
            
            Ok(())
        }
        
        fn complete(&self) -> Result<(), QueryError> {
            // Flush the file
            self.file.try_clone()
                .map_err(|e| QueryError::Other(format!("File error: {}", e)))?
                .flush()
                .map_err(|e| QueryError::Other(format!("File flush error: {}", e)))?;
            
            Ok(())
        }
    }
    
    /// A handler that collects results in memory
    pub struct CollectingResultHandler {
        /// The collected results
        pub results: Vec<HashMap<String, Literal>>,
        /// Total count of all results
        pub total_count: usize,
    }
    
    impl CollectingResultHandler {
        /// Create a new collecting result handler
        pub fn new() -> Self {
            Self {
                results: Vec::new(),
                total_count: 0,
            }
        }
        
        /// Get the collected results as a single QueryResult
        pub fn get_result(&self) -> QueryResult {
            QueryResult {
                results: self.results.clone(),
                total_count: self.total_count,
            }
        }
    }
    
    impl ResultHandler for CollectingResultHandler {
        fn handle_result(&mut self, result: &QueryResult) -> Result<(), QueryError> {
            // Extend results directly without cloning the entire collection
            self.results.extend(result.results.clone());
            self.total_count += result.total_count;
            
            Ok(())
        }
        
        fn complete(&self) -> Result<(), QueryError> {
            // Nothing to do on completion
            Ok(())
        }
    }
}

/// Format for exporting query results
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// Trait for transforming query results
pub trait ResultTransformer: Send + Sync {
    /// Transform a query result
    fn transform(&self, result: QueryResult) -> Result<QueryResult, QueryError>;
}

/// A transformer that maps field values using a function
pub struct FieldMapper {
    /// Field mappings: field name -> new field name
    field_mappings: HashMap<String, String>,
}

impl FieldMapper {
    /// Create a new field mapper
    pub fn new() -> Self {
        Self {
            field_mappings: HashMap::new(),
        }
    }
    
    /// Add a field mapping
    pub fn map_field(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.field_mappings.insert(from.into(), to.into());
        self
    }
}

impl ResultTransformer for FieldMapper {
    fn transform(&self, result: QueryResult) -> Result<QueryResult, QueryError> {
        // Create a new result with mapped fields
        let mapped_results = result.results.into_iter().map(|record| {
            let mut new_record = HashMap::new();
            
            for (key, value) in record {
                if let Some(new_key) = self.field_mappings.get(&key) {
                    new_record.insert(new_key.clone(), value);
                } else {
                    new_record.insert(key, value);
                }
            }
            
            new_record
        }).collect();
        
        Ok(QueryResult {
            results: mapped_results,
            total_count: result.total_count,
        })
    }
}

/// A transformer that filters results based on a predicate
pub struct ResultFilter {
    /// The predicate function (field name, value) -> should keep
    predicate: Box<dyn Fn(&HashMap<String, Literal>) -> bool + Send + Sync>,
}

impl ResultFilter {
    /// Create a new result filter with the given predicate
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&HashMap<String, Literal>) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
        }
    }
}

impl ResultTransformer for ResultFilter {
    fn transform(&self, result: QueryResult) -> Result<QueryResult, QueryError> {
        // Filter the results
        let filtered_results = result.results.into_iter()
            .filter(|record| (self.predicate)(record))
            .collect();
        
        Ok(QueryResult {
            results: filtered_results,
            total_count: result.total_count,
        })
    }
}

/// A streaming query result
pub struct ResultStream {
    /// The stream of query results
    receiver: Receiver<Result<QueryResult, QueryError>>,
}

impl ResultStream {
    /// Create a new result stream with the given capacity
    pub fn new(capacity: usize) -> (Self, Sender<Result<QueryResult, QueryError>>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { receiver }, sender)
    }
    
    /// Convert the stream to a standard Stream trait
    pub fn into_stream(self) -> impl Stream<Item = Result<QueryResult, QueryError>> {
        Box::pin(futures::stream::unfold(self.receiver, |mut rx| async move {
            rx.recv().await.map(|msg| (msg, rx))
        }))
    }
    
    /// Apply a transformer to this stream
    pub fn with_transformer(
        self, 
        transformer: impl ResultTransformer + 'static
    ) -> impl Stream<Item = Result<QueryResult, QueryError>> {
        let stream = self.into_stream();
        Box::pin(stream.map(move |result| {
            result.and_then(|r| transformer.transform(r))
        }))
    }
    
    /// Collect all results from this stream into a single result
    pub async fn collect(self) -> Result<QueryResult, QueryError> {
        let mut all_results = Vec::new();
        let mut total_count = 0;
        
        let stream = self.into_stream();
        pin_mut!(stream);
        
        while let Some(result) = stream.next().await {
            let result = result?;
            total_count += result.total_count;
            all_results.extend(result.results);
        }
        
        Ok(QueryResult {
            results: all_results,
            total_count,
        })
    }
    
    /// Process this stream with a result handler
    pub async fn process_with_handler(
        self,
        handler: &mut impl handlers::ResultHandler
    ) -> Result<(), QueryError> {
        let stream = self.into_stream();
        pin_mut!(stream);
        
        while let Some(result) = stream.next().await {
            let result = result?;
            handler.handle_result(&result)?;
        }
        
        handler.complete()
    }
}

/// Export a query result to a file in the specified format
pub fn export_result(
    result: &QueryResult,
    path: impl AsRef<Path>,
    format: OutputFormat
) -> Result<(), QueryError> {
    let file = File::create(path)
        .map_err(|e| QueryError::Other(format!("Failed to create file: {}", e)))?;
    
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(file, result)
                .map_err(|e| QueryError::Other(format!("Failed to write JSON: {}", e)))?;
        }
        OutputFormat::Csv => {
            let mut writer = csv::Writer::from_writer(file);
            
            // Write header row if there are results
            if let Some(first_row) = result.results.first() {
                let headers: Vec<String> = first_row.keys().cloned().collect();
                writer.write_record(&headers)
                    .map_err(|e| QueryError::Other(format!("Failed to write CSV header: {}", e)))?;
                
                // Write data rows
                for row in &result.results {
                    let values: Vec<String> = headers.iter()
                        .map(|key| match row.get(key) {
                            Some(Literal::String(s)) => s.clone(),
                            Some(lit) => format!("{:?}", lit),
                            None => String::new(),
                        })
                        .collect();
                    
                    writer.write_record(&values)
                        .map_err(|e| QueryError::Other(format!("Failed to write CSV row: {}", e)))?;
                }
            }
            
            writer.flush()
                .map_err(|e| QueryError::Other(format!("Failed to flush CSV writer: {}", e)))?;
        }
    }
    
    Ok(())
}

/// Convert a query result to a JSON value
pub fn result_to_json(result: &QueryResult) -> JsonValue {
    let results = result.results.iter().map(|row| {
        let mut map = serde_json::Map::new();
        
        for (key, value) in row {
            let json_value = match value {
                Literal::Int(n) => JsonValue::Number((*n).into()),
                Literal::Float(n) => {
                    if let Some(num) = serde_json::Number::from_f64(*n) {
                        JsonValue::Number(num)
                    } else {
                        JsonValue::Null
                    }
                },
                Literal::String(s) => JsonValue::String(s.clone()),
                Literal::Bool(b) => JsonValue::Bool(*b),
                Literal::Null => JsonValue::Null,
                Literal::List(items) => {
                    let json_items = items.iter().map(|item| literal_to_json(item)).collect();
                    JsonValue::Array(json_items)
                },
                Literal::Map(entries) => {
                    let mut json_map = serde_json::Map::new();
                    for (k, v) in entries {
                        json_map.insert(k.clone(), literal_to_json(v));
                    }
                    JsonValue::Object(json_map)
                },
            };
            
            map.insert(key.clone(), json_value);
        }
        
        JsonValue::Object(map)
    }).collect();
    
    let mut root = serde_json::Map::new();
    root.insert("results".to_string(), JsonValue::Array(results));
    root.insert("total_count".to_string(), JsonValue::Number(result.total_count.into()));
    
    JsonValue::Object(root)
}

/// Convert a literal to a JSON value
fn literal_to_json(literal: &Literal) -> JsonValue {
    match literal {
        Literal::Int(n) => JsonValue::Number((*n).into()),
        Literal::Float(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                JsonValue::Number(num)
            } else {
                JsonValue::Null
            }
        },
        Literal::String(s) => JsonValue::String(s.clone()),
        Literal::Bool(b) => JsonValue::Bool(*b),
        Literal::Null => JsonValue::Null,
        Literal::List(items) => {
            let json_items = items.iter().map(|item| literal_to_json(item)).collect();
            JsonValue::Array(json_items)
        },
        Literal::Map(entries) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in entries {
                json_map.insert(k.clone(), literal_to_json(v));
            }
            JsonValue::Object(json_map)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_field_mapper() {
        // Create a test result
        let result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("1".to_string()));
                    map.insert("name".to_string(), Literal::String("John".to_string()));
                    map.insert("age".to_string(), Literal::Int(30));
                    map
                },
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("2".to_string()));
                    map.insert("name".to_string(), Literal::String("Jane".to_string()));
                    map.insert("age".to_string(), Literal::Int(25));
                    map
                },
            ],
            total_count: 2,
        };
        
        // Create a field mapper
        let mapper = FieldMapper::new()
            .map_field("name", "full_name")
            .map_field("age", "years");
        
        // Transform the result
        let transformed = mapper.transform(result).expect("Transformation failed");
        
        // Check the transformation
        assert_eq!(transformed.total_count, 2);
        assert_eq!(transformed.results.len(), 2);
        
        assert!(transformed.results[0].contains_key("id"));
        assert!(!transformed.results[0].contains_key("name"));
        assert!(transformed.results[0].contains_key("full_name"));
        assert!(!transformed.results[0].contains_key("age"));
        assert!(transformed.results[0].contains_key("years"));
        
        assert_eq!(
            transformed.results[0].get("full_name"), 
            Some(&Literal::String("John".to_string()))
        );
        assert_eq!(
            transformed.results[0].get("years"), 
            Some(&Literal::Int(30))
        );
    }
    
    #[test]
    fn test_result_filter() {
        // Create a test result
        let result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("1".to_string()));
                    map.insert("age".to_string(), Literal::Int(30));
                    map
                },
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("2".to_string()));
                    map.insert("age".to_string(), Literal::Int(25));
                    map
                },
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("3".to_string()));
                    map.insert("age".to_string(), Literal::Int(40));
                    map
                },
            ],
            total_count: 3,
        };
        
        // Create a filter for age > 25
        let filter = ResultFilter::new(|record| {
            if let Some(Literal::Int(age)) = record.get("age") {
                *age > 25
            } else {
                false
            }
        });
        
        // Transform the result
        let filtered = filter.transform(result).expect("Filtering failed");
        
        // Check the filtering
        assert_eq!(filtered.total_count, 3); // Total count doesn't change
        assert_eq!(filtered.results.len(), 2); // Only 2 records match the filter
        
        // Should have records with age 30 and 40
        let ages: Vec<i64> = filtered.results.iter()
            .filter_map(|record| {
                if let Some(Literal::Int(age)) = record.get("age") {
                    Some(*age)
                } else {
                    None
                }
            })
            .collect();
        
        assert_eq!(ages, vec![30, 40]);
    }
    
    #[test]
    fn test_json_export() {
        // Create a temporary directory for the test file
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test_export.json");
        
        // Create a test result
        let result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("1".to_string()));
                    map.insert("name".to_string(), Literal::String("John".to_string()));
                    map
                },
            ],
            total_count: 1,
        };
        
        // Export to JSON
        export_result(&result, &file_path, OutputFormat::Json)
            .expect("Export failed");
        
        // Read and verify the JSON
        let json_str = std::fs::read_to_string(&file_path)
            .expect("Failed to read file");
        
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .expect("Failed to parse JSON");
        
        assert_eq!(json["total_count"], 1);
        assert!(json["results"].is_array());
        assert_eq!(json["results"].as_array().unwrap().len(), 1);
        assert_eq!(json["results"][0]["id"], "1");
        assert_eq!(json["results"][0]["name"], "John");
    }
} 