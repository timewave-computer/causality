// Committee extraction utilities
// Original file: src/committee/extraction.rs

//! Fact Extraction
//!
//! This module provides functionality for extracting facts from blockchain data
//! using configurable extraction rules.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_core::{Result, Error};
use causality_core::indexer::{ChainBlock, ChainTransaction};
use crate::log::{LogEntry, EntryType, EntryData};

/// A fact extracted from blockchain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// The domain this fact belongs to
    pub domain: String,
    /// The fact type
    pub fact_type: String,
    /// The fact data
    pub data: serde_json::Value,
    /// The source block hash
    pub source_block_hash: String,
    /// The source transaction hash (if applicable)
    pub source_tx_hash: Option<String>,
    /// Metadata about the fact
    pub metadata: HashMap<String, String>,
}

impl ExtractedFact {
    /// Convert this extracted fact to a log entry
    pub fn to_log_entry(&self) -> LogEntry {
        let entry_data = EntryData::Fact {
            domain: self.domain.clone(),
            fact_type: self.fact_type.clone(),
            data: self.data.clone(),
            source: Some(format!("block:{},tx:{}", 
                self.source_block_hash,
                self.source_tx_hash.clone().unwrap_or_default()
            )),
        };
        
        LogEntry::new(EntryType::Fact, entry_data)
    }
}

/// A condition for fact extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ExtractionCondition {
    /// Check if a field equals a value
    Equals {
        field: String,
        value: serde_json::Value,
    },
    /// Check if a field matches a regex
    Matches {
        field: String,
        pattern: String,
    },
    /// Check if a field contains a value
    Contains {
        field: String,
        value: serde_json::Value,
    },
    /// Check if a field is greater than a value
    GreaterThan {
        field: String,
        value: serde_json::Value,
    },
    /// Check if a field is less than a value
    LessThan {
        field: String,
        value: serde_json::Value,
    },
    /// Combine conditions with AND
    And(Vec<ExtractionCondition>),
    /// Combine conditions with OR
    Or(Vec<ExtractionCondition>),
    /// Negate a condition
    Not(Box<ExtractionCondition>),
}

/// A mapping for fact data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// The source field path
    pub source: String,
    /// The destination field
    pub destination: String,
    /// Optional transformation
    pub transform: Option<String>,
}

/// A rule for extracting facts from blockchain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    /// Unique rule ID
    pub id: String,
    /// The chain ID this rule applies to
    pub chain_id: String,
    /// The domain for extracted facts
    pub domain: String,
    /// The fact type
    pub fact_type: String,
    /// Whether this rule applies to blocks
    pub apply_to_blocks: bool,
    /// Whether this rule applies to transactions
    pub apply_to_transactions: bool,
    /// The condition for extraction
    pub condition: ExtractionCondition,
    /// Field mappings for the extracted fact
    pub field_mappings: Vec<FieldMapping>,
    /// Additional static fields to include
    pub static_fields: HashMap<String, serde_json::Value>,
}

/// Interface for a fact extractor
#[async_trait]
pub trait FactExtractor: Send + Sync {
    /// Extract facts from a block
    async fn extract_from_block(&self, block: &ChainBlock) -> Result<Vec<ExtractedFact>>;
    
    /// Extract facts from a transaction
    async fn extract_from_transaction(&self, tx: &ChainTransaction) -> Result<Vec<ExtractedFact>>;
    
    /// Get the rules used by this extractor
    fn get_rules(&self) -> Vec<ExtractionRule>;
}

/// An engine for managing extraction rules
pub struct RuleEngine {
    /// Rules by chain ID
    rules: RwLock<HashMap<String, Vec<ExtractionRule>>>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        RuleEngine {
            rules: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a rule to the engine
    pub fn add_rule(&self, rule: ExtractionRule) -> Result<()> {
        let chain_id = rule.chain_id.clone();
        
        let mut rules = self.rules.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock on rules".to_string())
        })?;
        
        let chain_rules = rules.entry(chain_id).or_insert_with(Vec::new);
        
        // Check for duplicate rule ID
        if chain_rules.iter().any(|r| r.id == rule.id) {
            return Err(Error::Configuration(format!(
                "Rule with ID '{}' already exists for chain ID '{}'",
                rule.id, rule.chain_id
            )));
        }
        
        chain_rules.push(rule);
        
        Ok(())
    }
    
    /// Remove a rule from the engine
    pub fn remove_rule(&self, chain_id: &str, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock on rules".to_string())
        })?;
        
        if let Some(chain_rules) = rules.get_mut(chain_id) {
            let len_before = chain_rules.len();
            chain_rules.retain(|r| r.id != rule_id);
            
            if chain_rules.len() == len_before {
                return Err(Error::Configuration(format!(
                    "Rule with ID '{}' not found for chain ID '{}'",
                    rule_id, chain_id
                )));
            }
            
            Ok(())
        } else {
            Err(Error::Configuration(format!(
                "No rules found for chain ID '{}'",
                chain_id
            )))
        }
    }
    
    /// Get all rules for a chain
    pub fn get_rules_for_chain(&self, chain_id: &str) -> Result<Vec<ExtractionRule>> {
        let rules = self.rules.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock on rules".to_string())
        })?;
        
        if let Some(chain_rules) = rules.get(chain_id) {
            Ok(chain_rules.clone())
        } else {
            // Return empty list instead of error
            Ok(Vec::new())
        }
    }
    
    /// Load rules from a TOML file
    pub fn load_rules_from_toml(&self, toml_str: &str) -> Result<()> {
        let rules: Vec<ExtractionRule> = toml::from_str(toml_str).map_err(|e| {
            Error::Configuration(format!("Failed to parse TOML rules: {}", e))
        })?;
        
        for rule in rules {
            self.add_rule(rule)?;
        }
        
        Ok(())
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A basic fact extractor implementation
pub struct BasicExtractor {
    /// The chain ID
    chain_id: String,
    /// The rule engine
    rule_engine: Arc<RuleEngine>,
}

impl BasicExtractor {
    /// Create a new basic extractor
    pub fn new(chain_id: impl Into<String>, rule_engine: Arc<RuleEngine>) -> Self {
        BasicExtractor {
            chain_id: chain_id.into(),
            rule_engine,
        }
    }
    
    /// Evaluate an extraction condition against data
    fn evaluate_condition(&self, condition: &ExtractionCondition, data: &serde_json::Value) -> bool {
        match condition {
            ExtractionCondition::Equals { field, value } => {
                let field_value = self.get_field_value(data, field);
                field_value.as_ref().map_or(false, |v| v == value)
            },
            ExtractionCondition::Matches { field, pattern } => {
                let field_value = self.get_field_value(data, field);
                if let Some(field_value) = field_value {
                    if let Some(s) = field_value.as_str() {
                        match regex::Regex::new(pattern) {
                            Ok(re) => re.is_match(s),
                            Err(_) => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ExtractionCondition::Contains { field, value } => {
                let field_value = self.get_field_value(data, field);
                if let Some(field_value) = field_value {
                    if let Some(array) = field_value.as_array() {
                        array.contains(value)
                    } else if let Some(obj) = field_value.as_object() {
                        if let Some(key) = value.as_str() {
                            obj.contains_key(key)
                        } else {
                            false
                        }
                    } else if let Some(s) = field_value.as_str() {
                        if let Some(needle) = value.as_str() {
                            s.contains(needle)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ExtractionCondition::GreaterThan { field, value } => {
                let field_value = self.get_field_value(data, field);
                if let Some(field_value) = field_value {
                    if let (Some(a), Some(b)) = (field_value.as_i64(), value.as_i64()) {
                        a > b
                    } else if let (Some(a), Some(b)) = (field_value.as_f64(), value.as_f64()) {
                        a > b
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ExtractionCondition::LessThan { field, value } => {
                let field_value = self.get_field_value(data, field);
                if let Some(field_value) = field_value {
                    if let (Some(a), Some(b)) = (field_value.as_i64(), value.as_i64()) {
                        a < b
                    } else if let (Some(a), Some(b)) = (field_value.as_f64(), value.as_f64()) {
                        a < b
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ExtractionCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, data))
            },
            ExtractionCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, data))
            },
            ExtractionCondition::Not(condition) => {
                !self.evaluate_condition(condition, data)
            },
        }
    }
    
    /// Get a field value from a JSON object using a path
    /// Path format: "field1.field2[0].field3"
    fn get_field_value<'a>(&self, data: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
        let mut current = data;
        
        for part in path.split('.') {
            if let Some((field, index)) = part.split_once('[') {
                // Handle array indexing like "field[0]"
                if let Some(stripped) = index.strip_suffix(']') {
                    if let Ok(idx) = stripped.parse::<usize>() {
                        // First get the field
                        current = current.get(field)?;
                        // Then index into array
                        current = current.get(idx)?;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                // Regular field access
                current = current.get(part)?;
            }
        }
        
        Some(current)
    }
    
    /// Extract data according to field mappings
    fn extract_data(&self, data: &serde_json::Value, rule: &ExtractionRule) -> serde_json::Value {
        let mut result = serde_json::Map::new();
        
        // Apply field mappings
        for mapping in &rule.field_mappings {
            if let Some(value) = self.get_field_value(data, &mapping.source) {
                let mut mapped_value = value.clone();
                
                // Apply transformation if specified
                if let Some(transform) = &mapping.transform {
                    mapped_value = self.apply_transform(value, transform);
                }
                
                result.insert(mapping.destination.clone(), mapped_value);
            }
        }
        
        // Add static fields
        for (key, value) in &rule.static_fields {
            result.insert(key.clone(), value.clone());
        }
        
        serde_json::Value::Object(result)
    }
    
    /// Apply a transformation to a value
    fn apply_transform(&self, value: &serde_json::Value, transform: &str) -> serde_json::Value {
        match transform {
            "toString" => {
                if let Some(s) = value.as_str() {
                    serde_json::Value::String(s.to_string())
                } else {
                    serde_json::Value::String(value.to_string())
                }
            },
            "toNumber" => {
                if let Some(n) = value.as_i64() {
                    serde_json::Value::Number(serde_json::Number::from(n))
                } else if let Some(n) = value.as_f64() {
                    // This will return None if n is not finite
                    if let Some(num) = serde_json::Number::from_f64(n) {
                        serde_json::Value::Number(num)
                    } else {
                        value.clone()
                    }
                } else if let Some(s) = value.as_str() {
                    if let Ok(n) = s.parse::<i64>() {
                        serde_json::Value::Number(serde_json::Number::from(n))
                    } else if let Ok(n) = s.parse::<f64>() {
                        if let Some(num) = serde_json::Number::from_f64(n) {
                            serde_json::Value::Number(num)
                        } else {
                            value.clone()
                        }
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                }
            },
            "toBoolean" => {
                if let Some(b) = value.as_bool() {
                    serde_json::Value::Bool(b)
                } else if let Some(s) = value.as_str() {
                    match s.to_lowercase().as_str() {
                        "true" | "yes" | "1" => serde_json::Value::Bool(true),
                        "false" | "no" | "0" => serde_json::Value::Bool(false),
                        _ => value.clone(),
                    }
                } else if let Some(n) = value.as_i64() {
                    serde_json::Value::Bool(n != 0)
                } else {
                    value.clone()
                }
            },
            "toLowerCase" => {
                if let Some(s) = value.as_str() {
                    serde_json::Value::String(s.to_lowercase())
                } else {
                    value.clone()
                }
            },
            "toUpperCase" => {
                if let Some(s) = value.as_str() {
                    serde_json::Value::String(s.to_uppercase())
                } else {
                    value.clone()
                }
            },
            _ => value.clone(), // Unknown transformation
        }
    }
}

#[async_trait]
impl FactExtractor for BasicExtractor {
    async fn extract_from_block(&self, block: &ChainBlock) -> Result<Vec<ExtractedFact>> {
        let rules = self.rule_engine.get_rules_for_chain(&self.chain_id)?;
        let mut facts = Vec::new();
        
        for rule in rules.iter().filter(|r| r.apply_to_blocks) {
            if self.evaluate_condition(&rule.condition, &block.data) {
                let data = self.extract_data(&block.data, rule);
                
                facts.push(ExtractedFact {
                    domain: rule.domain.clone(),
                    fact_type: rule.fact_type.clone(),
                    data,
                    source_block_hash: block.hash.clone(),
                    source_tx_hash: None,
                    metadata: HashMap::new(),
                });
            }
        }
        
        Ok(facts)
    }
    
    async fn extract_from_transaction(&self, tx: &ChainTransaction) -> Result<Vec<ExtractedFact>> {
        let rules = self.rule_engine.get_rules_for_chain(&self.chain_id)?;
        let mut facts = Vec::new();
        
        for rule in rules.iter().filter(|r| r.apply_to_transactions) {
            if self.evaluate_condition(&rule.condition, &tx.data) {
                let data = self.extract_data(&tx.data, rule);
                
                facts.push(ExtractedFact {
                    domain: rule.domain.clone(),
                    fact_type: rule.fact_type.clone(),
                    data,
                    source_block_hash: tx.block_hash.clone(),
                    source_tx_hash: Some(tx.hash.clone()),
                    metadata: HashMap::new(),
                });
            }
        }
        
        Ok(facts)
    }
    
    fn get_rules(&self) -> Vec<ExtractionRule> {
        self.rule_engine
            .get_rules_for_chain(&self.chain_id)
            .unwrap_or_default()
    }
} 