// Compatibility checking for content-addressed code
//
// This module provides functionality for checking if code is compatible
// with the current execution environment.

use std::collections::HashSet;
use std::fmt::Debug;

use crate::effect::EffectType;
use crate::error::{Error, Result};
use crate::effect_adapters::repository::CodeMetadata;

/// Compatibility checker for content-addressed code
///
/// This struct checks if code is compatible with the current execution
/// environment based on various metadata properties.
#[derive(Debug, Clone)]
pub struct CompatibilityChecker {
    /// The current version of the system
    current_version: String,
    /// The set of supported formats
    supported_formats: HashSet<String>,
    /// The set of allowed effect types
    allowed_effects: HashSet<EffectType>,
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        let mut supported_formats = HashSet::new();
        supported_formats.insert("risc-v".to_string());
        supported_formats.insert("json".to_string());
        
        Self {
            current_version: "0.1.0".to_string(),
            supported_formats,
            allowed_effects: HashSet::new(),
        }
    }
}

impl CompatibilityChecker {
    /// Create a new compatibility checker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the current version
    pub fn with_version(mut self, version: String) -> Self {
        self.current_version = version;
        self
    }
    
    /// Add a supported format
    pub fn add_supported_format(mut self, format: String) -> Self {
        self.supported_formats.insert(format);
        self
    }
    
    /// Allow an effect type
    pub fn allow_effect(mut self, effect_type: EffectType) -> Self {
        self.allowed_effects.insert(effect_type);
        self
    }
    
    /// Check if code is compatible with the current execution environment
    pub fn check_compatibility(&self, metadata: &CodeMetadata) -> Result<()> {
        // Check if the format is supported
        if !self.supported_formats.contains(&metadata.format) {
            return Err(Error::CodeIncompatible(format!("Unsupported code format: {}", metadata.format)));
        }
        
        // Check version compatibility
        if let Some(required_version) = &metadata.required_version {
            // For now, we just check for exact match
            // In a real implementation, we would use semantic versioning
            if required_version != &self.current_version {
                return Err(Error::CodeIncompatible(format!("Incompatible version requirement: {} (current: {})", required_version, self.current_version)));
            }
        }
        
        // Check effect permissions
        if let Some(required_effects) = &metadata.required_effects {
            for effect in required_effects {
                if !self.allowed_effects.contains(effect) {
                    return Err(Error::CodeIncompatible(format!("Unauthorized effect: {:?}", effect)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the current version
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
    
    /// Check if a format is supported
    pub fn is_format_supported(&self, format: &str) -> bool {
        self.supported_formats.contains(format)
    }
    
    /// Check if an effect is allowed
    pub fn is_effect_allowed(&self, effect_type: &EffectType) -> bool {
        self.allowed_effects.contains(effect_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_adapters::repository::CodeMetadataBuilder;
    
    #[test]
    fn test_format_compatibility() {
        let checker = CompatibilityChecker::new();
        
        // Test supported format
        let metadata = CodeMetadataBuilder::new()
            .with_format("risc-v".to_string())
            .build();
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test unsupported format
        let metadata = CodeMetadataBuilder::new()
            .with_format("unknown".to_string())
            .build();
        assert!(checker.check_compatibility(&metadata).is_err());
    }
    
    #[test]
    fn test_version_compatibility() {
        let checker = CompatibilityChecker::new();
        
        // Test compatible version
        let metadata = CodeMetadataBuilder::new()
            .with_format("risc-v".to_string())
            .with_required_version(Some("0.1.0".to_string()))
            .build();
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test incompatible version
        let metadata = CodeMetadataBuilder::new()
            .with_format("risc-v".to_string())
            .with_required_version(Some("0.2.0".to_string()))
            .build();
        assert!(checker.check_compatibility(&metadata).is_err());
    }
    
    #[test]
    fn test_effect_compatibility() {
        // Create a checker with allowed effects
        let effect_type = EffectType::Create;
        let checker = CompatibilityChecker::new()
            .allow_effect(effect_type.clone());
        
        // Test code with allowed effect
        let mut required_effects = HashSet::new();
        required_effects.insert(effect_type.clone());
        let metadata = CodeMetadataBuilder::new()
            .with_format("risc-v".to_string())
            .with_required_effects(Some(required_effects))
            .build();
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test code with unauthorized effect
        let unauthorized_effect = EffectType::Read;
        let mut required_effects = HashSet::new();
        required_effects.insert(unauthorized_effect.clone());
        let metadata = CodeMetadataBuilder::new()
            .with_format("risc-v".to_string())
            .with_required_effects(Some(required_effects))
            .build();
        assert!(checker.check_compatibility(&metadata).is_err());
    }
} 