//! Cross-language interoperability utilities for the Causality toolkit.

use causality_core::{Value, EntityId};
use anyhow::Result;

/// Helper for cross-language interoperability
#[derive(Debug, Clone)]
pub struct InteropHelper {
    language_mappings: std::collections::HashMap<String, String>,
}

impl InteropHelper {
    /// Create a new interop helper
    pub fn new() -> Self {
        Self {
            language_mappings: std::collections::HashMap::new(),
        }
    }
    
    /// Add a language mapping
    pub fn add_mapping(&mut self, from_lang: String, to_lang: String) {
        self.language_mappings.insert(from_lang, to_lang);
    }
    
    /// Convert a value between languages
    pub fn convert_value(&self, value: &Value, target_lang: &str) -> Result<Value> {
        // Mock conversion - just returns the same value for now
        Ok(value.clone())
    }
    
    /// Marshal a value for FFI transfer
    pub fn marshal_for_ffi(&self, value: &Value) -> Result<Vec<u8>> {
        // Mock FFI marshalling - serialize to bytes
        use causality_core::system::serialization::SszEncode;
        let mut bytes = Vec::new();
        value.ssz_append(&mut bytes);
        Ok(bytes)
    }
    
    /// Unmarshal a value from FFI transfer
    pub fn unmarshal_from_ffi(&self, data: &[u8]) -> Result<Value> {
        // Mock FFI unmarshalling - deserialize from bytes
        use causality_core::system::serialization::SszDecode;
        Value::from_ssz_bytes(data).map_err(|e| anyhow::anyhow!("FFI unmarshal error: {:?}", e))
    }
    
    /// Convert from OCaml to Lisp representation
    pub fn ocaml_to_lisp(&self, value: &Value) -> Result<Value> {
        // Mock OCaml to Lisp conversion - just return the same value
        Ok(value.clone())
    }
    
    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        let mut langs: Vec<String> = self.language_mappings.keys().cloned().collect();
        langs.extend(self.language_mappings.values().cloned());
        langs.sort();
        langs.dedup();
        langs
    }
    
    /// Check if a language is supported
    pub fn supports_language(&self, lang: &str) -> bool {
        self.language_mappings.contains_key(lang) || 
        self.language_mappings.values().any(|v| v == lang)
    }
}

impl Default for InteropHelper {
    fn default() -> Self {
        Self::new()
    }
} 