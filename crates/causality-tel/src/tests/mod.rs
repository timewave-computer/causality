//! TEL integration tests
//! 
//! This module contains integration tests for the TEL implementation.

mod integration_tests;
mod query_tests;
mod end_to_end_tests;
mod regression_tests;
mod category_tests;
mod adjunction_tests;
mod resource_preservation_tests;

// Re-export specific tests that should be visible to the parent module
pub use integration_tests::*;
pub use query_tests::*; 