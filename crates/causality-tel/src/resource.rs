// This file defines resource-related types and interfaces for the Temporal Effect Language (TEL)

use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

/// A unique identifier for a resource in the TEL system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(String);

impl ResourceId {
    /// Create a new ResourceId
    pub fn new(id: &str) -> Self {
        ResourceId(id.to_string())
    }

    /// Get the string representation of the ResourceId
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ResourceId(s.to_string()))
    }
}

/// Represents a quantity of a resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity {
    value: String,
}

impl Quantity {
    /// Create a new Quantity
    pub fn new(value: &str) -> Self {
        Quantity { value: value.to_string() }
    }

    /// Get the string representation of the Quantity
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for Quantity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Quantity { value: s.to_string() })
    }
}

// Add implementation of crate::handlers::Quantity for Quantity
impl crate::handlers::Quantity for Quantity {
    fn as_str(&self) -> &str {
        &self.value
    }
}

// Implement the trait from handlers module
impl crate::handlers::ResourceId for ResourceId {
    fn as_str(&self) -> &str {
        &self.0
    }
} 