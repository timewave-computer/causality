// Script handling for TEL
// Original file: src/tel/script.rs

//! TEL script representation and parsing
//!
//! This module defines the structure for representing Transaction Effect Language (TEL)
//! scripts, including parsing and validation functionality.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::domain::DomainId;

/// Represents a TEL script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelScript {
    /// Script version
    pub version: String,
    
    /// Source code of the script
    pub source: String,
    
    /// Parsed operations in the script
    pub operations: Vec<TelOperation>,
    
    /// Script metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TelScript {
    /// Create a new TEL script
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        
        Self {
            version: "1.0".to_string(),
            source,
            operations: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Parse the script source into operations
    pub fn parse(&mut self) -> Result<(), anyhow::Error> {
        // Placeholder for actual parsing logic
        // In a real implementation, this would parse the TEL source code
        // and populate the operations field
        
        // For now, we'll just create a simple parsing error
        Err(anyhow::anyhow!("TEL parsing not yet implemented"))
    }
    
    /// Add an operation to the script
    pub fn add_operation(&mut self, operation: TelOperation) {
        self.operations.push(operation);
    }
    
    /// Get the operations in the script
    pub fn operations(&self) -> &[TelOperation] {
        &self.operations
    }
    
    /// Validate the script
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Placeholder for actual validation logic
        // In a real implementation, this would validate operations, check types, etc.
        
        // For now, just return success
        Ok(())
    }
}

/// Types of TEL operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TelOperationType {
    /// Transfer assets between addresses
    Transfer,
    
    /// Store data on chain
    Store,
    
    /// Query data from chain
    Query,
    
    /// Sequence of operations
    Sequence,
    
    /// Parallel operations
    Parallel,
    
    /// Conditional operation
    Conditional,
    
    /// Custom operation type
    Custom(String),
}

impl TelOperationType {
    /// Convert operation type to string
    pub fn to_string(&self) -> String {
        match self {
            Self::Transfer => "transfer".to_string(),
            Self::Store => "store".to_string(),
            Self::Query => "query".to_string(),
            Self::Sequence => "sequence".to_string(),
            Self::Parallel => "parallel".to_string(),
            Self::Conditional => "conditional".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
    
    /// Parse operation type from string
    pub fn from_string(s: &str) -> Self {
        match s {
            "transfer" => Self::Transfer,
            "store" => Self::Store,
            "query" => Self::Query,
            "sequence" => Self::Sequence,
            "parallel" => Self::Parallel,
            "conditional" => Self::Conditional,
            _ => Self::Custom(s.to_string()),
        }
    }
}

/// Represents a TEL operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelOperation {
    /// Operation type
    pub operation_type: TelOperationType,
    
    /// Function name
    pub function_name: String,
    
    /// Parameters for the operation
    pub parameters: Value,
    
    /// Domain ID where the operation should be executed
    pub domain_id: Option<DomainId>,
    
    /// Child operations (for composite operations like sequence)
    #[serde(default)]
    pub children: Vec<TelOperation>,
}

impl TelOperation {
    /// Create a new TEL operation
    pub fn new(
        operation_type: TelOperationType,
        function_name: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self {
            operation_type,
            function_name: function_name.into(),
            parameters,
            domain_id: None,
            children: Vec::new(),
        }
    }
    
    /// Create a new transfer operation
    pub fn transfer(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        token: impl Into<String>,
        domain_id: Option<DomainId>,
    ) -> Self {
        let parameters = serde_json::json!({
            "from": from.into(),
            "to": to.into(),
            "amount": amount,
            "token": token.into(),
        });
        
        let mut operation = Self::new(
            TelOperationType::Transfer,
            "transfer",
            parameters,
        );
        
        operation.domain_id = domain_id;
        operation
    }
    
    /// Create a new store operation
    pub fn store(
        register_id: impl Into<String>,
        fields: Vec<String>,
        strategy: impl Into<String>,
        domain_id: Option<DomainId>,
    ) -> Self {
        let parameters = serde_json::json!({
            "register_id": register_id.into(),
            "fields": fields,
            "strategy": strategy.into(),
        });
        
        let mut operation = Self::new(
            TelOperationType::Store,
            "store",
            parameters,
        );
        
        operation.domain_id = domain_id;
        operation
    }
    
    /// Create a new query operation
    pub fn query(
        query_type: impl Into<String>,
        parameters: HashMap<String, Value>,
        domain_id: Option<DomainId>,
    ) -> Self {
        let mut json_params = serde_json::Map::new();
        for (k, v) in parameters {
            json_params.insert(k, v);
        }
        
        let mut operation = Self::new(
            TelOperationType::Query,
            query_type.into(),
            Value::Object(json_params),
        );
        
        operation.domain_id = domain_id;
        operation
    }
    
    /// Create a sequence of operations
    pub fn sequence(operations: Vec<TelOperation>) -> Self {
        Self {
            operation_type: TelOperationType::Sequence,
            function_name: "sequence".to_string(),
            parameters: Value::Null,
            domain_id: None,
            children: operations,
        }
    }
    
    /// Create parallel operations
    pub fn parallel(operations: Vec<TelOperation>) -> Self {
        Self {
            operation_type: TelOperationType::Parallel,
            function_name: "parallel".to_string(),
            parameters: Value::Null,
            domain_id: None,
            children: operations,
        }
    }
    
    /// Create a conditional operation
    pub fn conditional(
        condition: TelOperation,
        then_operation: TelOperation,
        else_operation: Option<TelOperation>,
    ) -> Self {
        let mut children = vec![condition, then_operation];
        if let Some(else_op) = else_operation {
            children.push(else_op);
        }
        
        Self {
            operation_type: TelOperationType::Conditional,
            function_name: "if_then_else".to_string(),
            parameters: Value::Null,
            domain_id: None,
            children,
        }
    }
    
    /// Set the domain ID for this operation
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Add a child operation
    pub fn add_child(&mut self, child: TelOperation) {
        self.children.push(child);
    }
}

/// TEL parser for converting source code to operations
pub struct TelParser;

impl TelParser {
    /// Parse TEL source code into a script
    pub fn parse(source: &str) -> Result<TelScript, anyhow::Error> {
        let mut script = TelScript::new(source);
        script.parse()?;
        Ok(script)
    }
    
    /// Parse TEL source code into operations
    pub fn parse_operations(source: &str) -> Result<Vec<TelOperation>, anyhow::Error> {
        let script = Self::parse(source)?;
        Ok(script.operations().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_operations() {
        // Create a transfer operation
        let transfer = TelOperation::transfer(
            "0x1234",
            "0x5678",
            100,
            "ETH",
            Some(DomainId::new("ethereum:mainnet")),
        );
        
        assert_eq!(transfer.operation_type, TelOperationType::Transfer);
        assert_eq!(transfer.function_name, "transfer");
        
        // Create a store operation
        let fields = vec![
            "balance".to_string(),
            "owner".to_string(),
        ];
        
        let store = TelOperation::store(
            "register-123",
            fields,
            "on_chain",
            Some(DomainId::new("ethereum:mainnet")),
        );
        
        assert_eq!(store.operation_type, TelOperationType::Store);
        assert_eq!(store.function_name, "store");
        
        // Create a sequence of operations
        let sequence = TelOperation::sequence(vec![transfer, store]);
        
        assert_eq!(sequence.operation_type, TelOperationType::Sequence);
        assert_eq!(sequence.children.len(), 2);
    }
} 