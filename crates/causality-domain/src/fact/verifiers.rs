// Fact verifiers for domains
// Original file: src/domain/fact/verifiers.rs

// Fact Verifiers Module for Causality
//
// This module defines the verifiers used for fact verification in Causality.

use std::collections::HashMap;
use std::fmt::Debug;

use crate::error::Error;
use crate::fact::types::FactType;
use crate::fact::verification::{FactVerification, FactVerifier};

/// A basic verifier that always succeeds
#[derive(Debug)]
pub struct AlwaysSuccessVerifier {
    name: String,
}

impl AlwaysSuccessVerifier {
    /// Create a new always-success verifier
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

impl Default for AlwaysSuccessVerifier {
    fn default() -> Self {
        Self::new("always-success")
    }
}

impl FactVerifier for AlwaysSuccessVerifier {
    /// Get the verifier name
    fn name(&self) -> &str {
        &self.name
    }
    
    /// Verify a fact - always succeeds
    fn verify(&self, _fact: &FactType) -> Result<FactVerification, Error> {
        let mut metadata = HashMap::new();
        metadata.insert("verifier".to_string(), self.name.clone());
        metadata.insert("method".to_string(), "always-success".to_string());
        
        Ok(FactVerification::success().with_metadata("result", "success"))
    }
}

/// A basic verifier that always fails
#[derive(Debug)]
pub struct AlwaysFailVerifier {
    name: String,
    error_message: String,
}

impl AlwaysFailVerifier {
    /// Create a new always-fail verifier
    pub fn new(name: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            error_message: error_message.into(),
        }
    }
}

impl Default for AlwaysFailVerifier {
    fn default() -> Self {
        Self::new("always-fail", "Verification failed by design")
    }
}

impl FactVerifier for AlwaysFailVerifier {
    /// Get the verifier name
    fn name(&self) -> &str {
        &self.name
    }
    
    /// Verify a fact - always fails
    fn verify(&self, _fact: &FactType) -> Result<FactVerification, Error> {
        let mut metadata = HashMap::new();
        metadata.insert("verifier".to_string(), self.name.clone());
        metadata.insert("method".to_string(), "always-fail".to_string());
        
        Ok(FactVerification::failure(&self.error_message))
    }
}

/// A composite verifier that combines multiple verifiers
#[derive(Debug)]
pub struct CompositeVerifier {
    name: String,
    verifiers: Vec<Box<dyn FactVerifier>>,
    require_all: bool,
}

impl CompositeVerifier {
    /// Create a new composite verifier
    pub fn new(name: impl Into<String>, verifiers: Vec<Box<dyn FactVerifier>>, require_all: bool) -> Self {
        Self {
            name: name.into(),
            verifiers,
            require_all,
        }
    }
    
    /// Add a verifier to the composite
    pub fn add_verifier(&mut self, verifier: Box<dyn FactVerifier>) {
        self.verifiers.push(verifier);
    }
}

impl FactVerifier for CompositeVerifier {
    /// Get the verifier name
    fn name(&self) -> &str {
        &self.name
    }
    
    /// Verify a fact using all contained verifiers
    fn verify(&self, fact: &FactType) -> Result<FactVerification, Error> {
        let mut results = Vec::new();
        let mut all_verified = true;
        let mut any_verified = false;
        
        for verifier in &self.verifiers {
            match verifier.verify(fact) {
                Ok(result) => {
                    if result.is_valid() {
                        any_verified = true;
                    } else {
                        all_verified = false;
                    }
                    results.push(result);
                },
                Err(e) => {
                    return Err(Error::FactVerification(format!(
                        "Verifier '{}' failed: {}", 
                        verifier.name(), 
                        e
                    )));
                }
            }
        }
        
        let verified = if self.require_all { all_verified } else { any_verified };
        let mut metadata = HashMap::new();
        metadata.insert("verifier".to_string(), self.name.clone());
        metadata.insert("method".to_string(), "composite".to_string());
        metadata.insert("require_all".to_string(), self.require_all.to_string());
        metadata.insert("verifier_count".to_string(), self.verifiers.len().to_string());
        
        if verified {
            Ok(FactVerification {
                verified: true,
                metadata,
                error: None,
            })
        } else {
            let error_msg = if self.require_all {
                "Not all verifiers succeeded"
            } else {
                "No verifiers succeeded"
            };
            
            Ok(FactVerification {
                verified: false,
                metadata,
                error: Some(error_msg.to_string()),
            })
        }
    }
    
    /// Get the verifier metadata
    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "composite".to_string());
        metadata.insert("verifier_count".to_string(), self.verifiers.len().to_string());
        metadata.insert("require_all".to_string(), self.require_all.to_string());
        metadata
    }
}
