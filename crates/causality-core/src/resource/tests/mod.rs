// Resource tests module

mod resource_type_registry_tests;
mod cross_domain_protocol_tests;
mod resource_storage_tests;

// Re-export tests for integration testing
pub use resource_type_registry_tests::*;
pub use cross_domain_protocol_tests::*;
pub use resource_storage_tests::*; 