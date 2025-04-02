//! Effect type ID definitions
//!
//! This module provides types for uniquely identifying effects.

use std::fmt::{Debug, Display};
use std::hash::Hash;

/// Uniquely identifies an effect type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectTypeId {
    /// Domain this effect applies to
    pub domain: String,
    
    /// Name of the effect
    pub name: String,
}

impl EffectTypeId {
    /// Create a new effect type ID
    pub fn new(domain: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            name: name.into(),
        }
    }
}

impl Display for EffectTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain, self.name)
    }
} 