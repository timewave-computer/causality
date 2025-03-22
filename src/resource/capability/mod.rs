//! Capability-based access control system for resources
//!
//! This module provides a capability-based approach to resource access control,
//! enabling fine-grained delegation and revocation of access rights.

// Core capability types and implementations
mod types;
mod repository;
mod service;
mod integration;
mod validation;
mod delegation;
mod proof;

// Re-export core types and traits
pub use types::*;
pub use repository::*;
pub use service::*;
pub use integration::*;
pub use validation::*;
pub use delegation::*;
pub use proof::*;

// Tests and mock implementations
#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;
#[cfg(test)]
pub use mock::*; 