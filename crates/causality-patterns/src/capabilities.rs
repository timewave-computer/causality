// Capability patterns and utilities
// Original file: src/capabilities.rs

// Capabilities Module
//
// This module defines the capability-based authorization system used throughout Causality.
// It provides types and functionality for managing rights, capabilities, and permissions.

use std::fmt;

/// Represents a specific right or permission within the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Right(pub String);

impl Right {
    /// Create a new right with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    /// Get the name of this right
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Right {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Right({})", self.0)
    }
}

/// A capability that grants specific rights
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique identifier for this capability
    pub id: String,
    
    /// The rights granted by this capability
    pub rights: Vec<Right>,
    
    /// Whether this capability can be delegated
    pub delegatable: bool,
}

impl Capability {
    /// Create a new capability with the given rights
    pub fn new(id: impl Into<String>, rights: Vec<Right>, delegatable: bool) -> Self {
        Self {
            id: id.into(),
            rights,
            delegatable,
        }
    }
    
    /// Check if this capability grants a specific right
    pub fn has_right(&self, right: &Right) -> bool {
        self.rights.contains(right)
    }
}

/// Types of capabilities in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityType {
    /// Allows reading data
    Read,
    
    /// Allows modifying data
    Write,
    
    /// Allows executing code or functions
    Execute,
    
    /// Allows delegating capabilities to others
    Delegate,
    
    /// Custom capability type
    Custom(String),
} 