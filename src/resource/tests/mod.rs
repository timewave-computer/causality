// Resource system tests

mod register_tests;
mod lifecycle_tests;
mod transition_tests;
mod versioning_tests;
mod epoch_tests;
mod summarization_tests;
mod archival_tests;
mod garbage_collection_tests;
mod capability_tests;
mod api_tests;
mod storage_tests;

// Re-export tests
pub use register_tests::*;
pub use lifecycle_tests::*;
pub use transition_tests::*;
pub use versioning_tests::*;
pub use epoch_tests::*;
pub use summarization_tests::*;
pub use archival_tests::*;
pub use garbage_collection_tests::*;
pub use capability_tests::*;
pub use api_tests::*;

// Re-export test utilities
pub use register_tests::create_test_register;
pub use capability_tests::create_test_capability;

// Resource tests module

// Import test modules
mod resource_register_tests;
mod effect_tests;
mod storage_tests;
mod capability_tests;
mod api_tests;
mod archival_test;
mod archival_integration_test;
mod summarization_test;
mod summarization_integration_test;
mod epoch_test;
mod versioning_test;
mod garbage_collection_test;
mod effect_template_integration_tests;

// Re-export test helpers for use in other tests
pub use resource_register_tests::create_test_resource;
pub use resource_register_tests::create_test_resource_with_id;
pub use capability_tests::create_test_capability;
pub use capability_tests::create_test_capability_with_id;
pub use api_tests::create_test_api;
pub use api_tests::create_test_api_with_id;
pub use storage_tests::create_test_storage;
pub use storage_tests::create_test_storage_with_id; 