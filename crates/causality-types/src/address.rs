// Address format and parsing utilities
// Original file: src/address.rs

//! Content addressing module
//!
//! This module provides types and utilities for content addressing in the Causality system.
//! It enables content-addressable resources, allowing for integrity verification and 
//! reliable references.

use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;
use crate::crypto_primitives::{HashAlgorithm, ContentId};

/// Error types for content addressing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    /// Invalid format for content hash
    InvalidFormat,
    /// Hash algorithm not supported
    UnsupportedAlgorithm,
    /// Error during hashing operation
    HashingError,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::InvalidFormat => write!(f, "Invalid content hash format"),
            AddressError::UnsupportedAlgorithm => write!(f, "Unsupported hashing algorithm"),
            AddressError::HashingError => write!(f, "Error during hashing operation"),
        }
    }
}

impl std::error::Error for AddressError {}

/// Address represents an identity in the system, such as a user, account, or system component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    /// The string representation of the address
    inner: String,
}

impl Address {
    /// Create a new address from a string
    pub fn new(address: String) -> Self {
        Self { inner: address }
    }
    
    /// Get the string representation of the address
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    
    /// Get the byte representation of the address
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<String> for Address {
    fn from(address: String) -> Self {
        Self { inner: address }
    }
}

impl From<&str> for Address {
    fn from(address: &str) -> Self {
        Self { inner: address.to_string() }
    }
}

/// A pool of addresses, useful for address allocation and management
pub struct AddressPool {
    addresses: Vec<Address>,
}

impl AddressPool {
    /// Create a new empty address pool
    pub fn new() -> Self {
        Self { addresses: Vec::new() }
    }
    
    /// Create a new address pool with the given addresses
    pub fn with_addresses(addresses: Vec<Address>) -> Self {
        Self { addresses }
    }
    
    /// Add an address to the pool
    pub fn add_address(&mut self, address: Address) {
        self.addresses.push(address);
    }
    
    /// Remove an address from the pool
    pub fn remove_address(&mut self, address: &Address) -> bool {
        if let Some(index) = self.addresses.iter().position(|a| a == address) {
            self.addresses.remove(index);
            true
        } else {
            false
        }
    }
    
    /// Check if the pool contains the given address
    pub fn contains(&self, address: &Address) -> bool {
        self.addresses.contains(address)
    }
    
    /// Get the number of addresses in the pool
    pub fn len(&self) -> usize {
        self.addresses.len()
    }
    
    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }
} 