// Effect log entry implementation
// Original file: src/log/entry/effect_entry.rs

// Effect entry implementation for Causality Unified Log System
//
// This module provides the EffectEntry struct for representing effects in the log.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use causality_types::{ContentId, DomainId};
use causality_core::effect::EffectType;

/// Wrapper for EffectType to implement Serialize and Deserialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEffectType(String);

impl From<EffectType> for SerializableEffectType {
    fn from(effect_type: EffectType) -> Self {
        SerializableEffectType(effect_type.to_string())
    }
}

impl From<SerializableEffectType> for EffectType {
    fn from(serializable: SerializableEffectType) -> Self {
        serializable.0.parse().unwrap_or_else(|_| EffectType::Custom(serializable.0))
    }
}

impl std::ops::Deref for SerializableEffectType {
    type Target = String;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An entry representing an effect operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectEntry {
    /// The type of effect
    pub effect_type: SerializableEffectType,
    /// The resources affected by this effect
    pub resources: Vec<ContentId>,
    /// The domains involved in this effect
    pub domains: Vec<DomainId>,
    /// The hash of the effect code
    pub code_hash: Option<String>,
    /// The serialized effect parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// The result of the effect execution
    pub result: Option<serde_json::Value>,
    /// Whether the effect was successful
    pub success: bool,
    /// An error message, if the effect failed
    pub error: Option<String>,
}

impl EffectEntry {
    /// Create a new effect entry
    pub fn new(
        effect_type: EffectType,
        resources: Vec<ContentId>,
        domains: Vec<DomainId>,
        code_hash: Option<String>,
        parameters: HashMap<String, serde_json::Value>,
        result: Option<serde_json::Value>,
        success: bool,
        error: Option<String>,
    ) -> Self {
        Self {
            effect_type: effect_type.into(),
            resources,
            domains,
            code_hash,
            parameters,
            result,
            success,
            error,
        }
    }
    
    /// Get the effect type
    pub fn effect_type(&self) -> EffectType {
        self.effect_type.clone().into()
    }
    
    /// Get the resources affected by this effect
    pub fn resources(&self) -> &[ContentId] {
        &self.resources
    }
    
    /// Get the domains involved in this effect
    pub fn domains(&self) -> &[DomainId] {
        &self.domains
    }
    
    /// Get the code hash for this effect
    pub fn code_hash(&self) -> Option<&str> {
        self.code_hash.as_deref()
    }
    
    /// Get the parameters for this effect
    pub fn parameters(&self) -> &HashMap<String, serde_json::Value> {
        &self.parameters
    }
    
    /// Get a specific parameter for this effect
    pub fn get_parameter(&self, name: &str) -> Option<&serde_json::Value> {
        self.parameters.get(name)
    }
    
    /// Get the result of this effect
    pub fn result(&self) -> Option<&serde_json::Value> {
        self.result.as_ref()
    }
    
    /// Check if this effect was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Get the error message, if any
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    
    /// Set the result of this effect
    pub fn with_result(mut self, result: serde_json::Value, success: bool) -> Self {
        self.result = Some(result);
        self.success = success;
        self.error = None;
        self
    }
    
    /// Set an error for this effect
    pub fn with_error(mut self, error: String) -> Self {
        self.result = None;
        self.success = false;
        self.error = Some(error);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_entry_creation() {
        let effect_type = EffectType::Transfer;
        let resources = vec![ContentId::new(1), ContentId::new(2)];
        let domains = vec![DomainId::new(1), DomainId::new(2)];
        let code_hash = Some("abc123".to_string());
        let mut parameters = HashMap::new();
        parameters.insert("amount".to_string(), serde_json::json!(100));
        
        let entry = EffectEntry::new(
            effect_type,
            resources.clone(),
            domains.clone(),
            code_hash.clone(),
            parameters.clone(),
            None,
            false,
            None,
        );
        
        assert_eq!(entry.effect_type(), EffectType::Transfer);
        assert_eq!(entry.resources(), &resources);
        assert_eq!(entry.domains(), &domains);
        assert_eq!(entry.code_hash(), code_hash.as_deref());
        assert_eq!(entry.parameters().get("amount").unwrap(), &serde_json::json!(100));
        assert!(entry.result().is_none());
        assert!(!entry.is_success());
        assert!(entry.error().is_none());
        
        // Test with result
        let with_result = entry.clone().with_result(serde_json::json!("ok"), true);
        assert_eq!(with_result.result().unwrap(), serde_json::json!("ok"));
        assert!(with_result.is_success());
        assert!(with_result.error().is_none());
        
        // Test with error
        let with_error = entry.with_error("failed".to_string());
        assert!(with_error.result().is_none());
        assert!(!with_error.is_success());
        assert_eq!(with_error.error().unwrap(), "failed");
    }
} 
