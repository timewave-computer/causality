//! Tests for TEL type system components
//! 
//! This module contains tests for the type system components,
//! including rows, effects, and other type-related functionality.

mod row_tests;
mod effect_tests;

// Re-export specific tests that should be visible to the parent module
pub use row_tests::*;
pub use effect_tests::*; 