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