// Observation extraction functionality
//
// This module provides functionality for extracting facts from blockchain data.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use causality_types::{Error, Result};
use crate::indexer::BlockData;

/// Errors that can occur during extraction
#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Data error: {0}")]
    Data(String),
    
    #[error("Rule error: {0}")]
    Rule(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// A fact extracted from blockchain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// Unique identifier for the fact
    pub id: String,
    /// The chain ID the fact was extracted from
    pub chain_id: String,
    /// The block height the fact was extracted from
    pub block_height: u64,
    /// The block hash the fact was extracted from
    pub block_hash: String,
    /// The type of the fact
    pub fact_type: String,
    /// The fact data, serialized as JSON
    pub data: serde_json::Value,
    /// Metadata associated with the fact
    pub metadata: HashMap<String, String>,
}

impl ExtractedFact {
    /// Create a new extracted fact
    pub fn new(
        id: String,
        chain_id: String,
        block_height: u64,
        block_hash: String,
        fact_type: String,
        data: serde_json::Value,
    ) -> Self {
        ExtractedFact {
            id,
            chain_id,
            block_height,
            block_hash,
            fact_type,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the fact
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// A rule for extracting facts from blockchain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    /// Unique identifier for the rule
    pub id: String,
    /// The type of fact this rule extracts
    pub fact_type: String,
    /// A description of what this rule extracts
    pub description: String,
    /// Chain ID this rule applies to (or "*" for all chains)
    pub chain_id: String,
    /// A JSON pointer pattern to match in block data
    pub json_pointer: String,
    /// Additional filter conditions that must be satisfied
    pub conditions: Vec<FilterCondition>,
}

/// A filter condition for an extraction rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    /// JSON pointer to the value to check
    pub path: String,
    /// Operator for comparison
    pub operator: String,
    /// Value to compare against
    pub value: serde_json::Value,
}

impl FilterCondition {
    /// Check if a condition is satisfied
    pub fn is_satisfied(&self, data: &serde_json::Value) -> Result<bool> {
        // Get the value from the path
        let value_path = self.path.replace("/", ".");
        let value = jsonpath_lib::select(data, &value_path)
            .map_err(|e| Error::Data(format!("Failed to extract value: {}", e)))?;
            
        if value.is_empty() {
            return Ok(false);
        }
        
        let value = &value[0];
        
        // Compare using the specified operator
        match self.operator.as_str() {
            "==" => Ok(value == &self.value),
            "!=" => Ok(value != &self.value),
            ">" => {
                if let (Some(a), Some(b)) = (value.as_f64(), self.value.as_f64()) {
                    Ok(a > b)
                } else {
                    Err(Error::Data("Comparison requires numeric values".to_string()))
                }
            },
            ">=" => {
                if let (Some(a), Some(b)) = (value.as_f64(), self.value.as_f64()) {
                    Ok(a >= b)
                } else {
                    Err(Error::Data("Comparison requires numeric values".to_string()))
                }
            },
            "<" => {
                if let (Some(a), Some(b)) = (value.as_f64(), self.value.as_f64()) {
                    Ok(a < b)
                } else {
                    Err(Error::Data("Comparison requires numeric values".to_string()))
                }
            },
            "<=" => {
                if let (Some(a), Some(b)) = (value.as_f64(), self.value.as_f64()) {
                    Ok(a <= b)
                } else {
                    Err(Error::Data("Comparison requires numeric values".to_string()))
                }
            },
            "contains" => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    Ok(a.contains(b))
                } else {
                    Err(Error::Data("Contains comparison requires string values".to_string()))
                }
            },
            "startsWith" => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    Ok(a.starts_with(b))
                } else {
                    Err(Error::Data("StartsWith comparison requires string values".to_string()))
                }
            },
            "endsWith" => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    Ok(a.ends_with(b))
                } else {
                    Err(Error::Data("EndsWith comparison requires string values".to_string()))
                }
            },
            _ => Err(Error::Data(format!("Unknown operator: {}", self.operator))),
        }
    }
}

/// An interface for extracting facts from blockchain data
#[async_trait]
pub trait FactExtractor: Send + Sync {
    /// Extract facts from block data
    async fn extract_facts(&self, block_data: &BlockData) -> Result<Vec<ExtractedFact>, ExtractionError>;
    
    /// Add an extraction rule
    fn add_rule(&self, rule: ExtractionRule) -> Result<(), ExtractionError>;
    
    /// Load rules from a TOML string
    fn load_rules_from_toml(&self, toml_str: &str) -> Result<(), ExtractionError>;
}

/// An engine for managing extraction rules
pub struct RuleEngine {
    /// Rules for extracting facts
    rules: RwLock<Vec<ExtractionRule>>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        RuleEngine {
            rules: RwLock::new(Vec::new()),
        }
    }
    
    /// Add a rule to the engine
    pub fn add_rule(&self, rule: ExtractionRule) -> Result<(), ExtractionError> {
        let mut rules = self.rules.write().map_err(|e| ExtractionError::Internal(e.to_string()))?;
            
        rules.push(rule);
        
        Ok(())
    }
    
    /// Load rules from a TOML string
    pub fn load_rules_from_toml(&self, toml_str: &str) -> Result<(), ExtractionError> {
        let rules: Vec<ExtractionRule> = toml::from_str(toml_str)
            .map_err(|e| ExtractionError::Configuration(format!("Failed to parse TOML: {}", e)))?;
            
        let mut current_rules = self.rules.write().map_err(|e| ExtractionError::Internal(e.to_string()))?;
            
        for rule in rules {
            current_rules.push(rule);
        }
        
        Ok(())
    }
    
    /// Get all rules that apply to a chain
    pub fn get_rules_for_chain(&self, chain_id: &str) -> Result<Vec<ExtractionRule>> {
        let rules = self.rules.read().map_err(|e| ExtractionError::Internal(e.to_string()))?;
            
        Ok(rules.iter()
            .filter(|rule| rule.chain_id == "*" || rule.chain_id == chain_id)
            .cloned()
            .collect())
    }
    
    /// Match and extract facts using rules
    pub fn extract_facts(&self, block_data: &BlockData) -> Result<Vec<ExtractedFact>> {
        let chain_id = &block_data.chain_id;
        let rules = self.get_rules_for_chain(chain_id)?;
        
        let mut facts = Vec::new();
        
        for rule in rules {
            let matches = self.apply_rule(&rule, block_data)?;
            facts.extend(matches);
        }
        
        Ok(facts)
    }
    
    /// Apply a rule to block data
    fn apply_rule(&self, rule: &ExtractionRule, block_data: &BlockData) -> Result<Vec<ExtractedFact>> {
        let mut facts = Vec::new();
        
        // Get the data from the JSON pointer pattern
        let pointer_path = rule.json_pointer.replace("/", ".");
        let matches = jsonpath_lib::select(&block_data.data, &pointer_path)
            .map_err(|e| Error::Data(format!("Failed to match JSON pointer: {}", e)))?;
            
        for data_match in matches {
            // Check if all conditions are satisfied
            let all_conditions_satisfied = rule.conditions.iter()
                .map(|condition| condition.is_satisfied(&block_data.data))
                .collect::<Result<Vec<bool>>>()?
                .into_iter()
                .all(|satisfied| satisfied);
                
            if all_conditions_satisfied {
                // Create a fact from the matched data
                let fact_id = format!("{}:{}:{}", 
                    block_data.chain_id, 
                    block_data.height, 
                    rule.id
                );
                
                let fact = ExtractedFact::new(
                    fact_id,
                    block_data.chain_id.clone(),
                    block_data.height,
                    block_data.hash.clone(),
                    rule.fact_type.clone(),
                    data_match.clone(),
                );
                
                facts.push(fact);
            }
        }
        
        Ok(facts)
    }
}

/// A basic implementation of a fact extractor
pub struct BasicExtractor {
    /// The chain ID this extractor is for
    chain_id: String,
    /// The rule engine to use
    rule_engine: Arc<RuleEngine>,
}

impl BasicExtractor {
    /// Create a new basic extractor
    pub fn new(chain_id: String, rule_engine: Arc<RuleEngine>) -> Self {
        BasicExtractor {
            chain_id,
            rule_engine,
        }
    }
}

#[async_trait]
impl FactExtractor for BasicExtractor {
    /// Extract facts from block data
    async fn extract_facts(&self, block_data: &BlockData) -> Result<Vec<ExtractedFact>, ExtractionError> {
        // Verify chain ID
        if block_data.chain_id != self.chain_id {
            return Err(ExtractionError::Data(format!(
                "Chain ID mismatch: expected {}, got {}",
                self.chain_id, block_data.chain_id
            )));
        }
        
        // Use the rule engine to extract facts
        self.rule_engine.extract_facts(block_data)
    }
    
    /// Add an extraction rule
    fn add_rule(&self, rule: ExtractionRule) -> Result<(), ExtractionError> {
        self.rule_engine.add_rule(rule)
    }
    
    /// Load rules from a TOML string
    fn load_rules_from_toml(&self, toml_str: &str) -> Result<(), ExtractionError> {
        self.rule_engine.load_rules_from_toml(toml_str)
    }
} 