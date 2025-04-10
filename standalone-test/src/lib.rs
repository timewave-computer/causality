pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use hex;

// Simplified ContentId implementation for testing
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(Vec<u8>);

impl ContentId {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
    
    pub fn to_string(&self) -> String {
        hex::encode(&self.0)
    }
}

// Simplified Literal type for testing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Literal>),
    Map(HashMap<String, Literal>),
    Null,
}

// Simplified QueryResult structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub results: Vec<HashMap<String, Literal>>,
    pub total_count: usize,
}

/// Generate a content ID for a query result - identical to the original implementation
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_standalone_content_id() {
        // Create a simple QueryResult
        let mut record = HashMap::new();
        record.insert("id".to_string(), Literal::String("1".to_string()));
        record.insert("name".to_string(), Literal::String("Test User".to_string()));
        
        let query_result = QueryResult {
            results: vec![record],
            total_count: 1,
        };
        
        // Generate the content ID
        let content_id = result_content_id(&query_result);
        
        // Verify that the content ID is valid (not empty)
        assert!(!content_id.to_string().is_empty());
        
        // Verify determinism - generating another content ID from the same data should yield the same result
        let serialized = serde_json::to_vec(&query_result).expect("Failed to serialize");
        let expected_content_id = ContentId::from_bytes(&serialized);
        
        assert_eq!(content_id, expected_content_id);
    }
}
