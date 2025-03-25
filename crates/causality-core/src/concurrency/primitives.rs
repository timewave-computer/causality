// Low-level concurrency primitives
// Original file: src/concurrency/primitives.rs

// Concurrency primitives for Causality
//
// This module provides the core primitive types and structures
// for concurrent programming, including task identifiers and 
// resource management.

mod task_id;
mod wait_queue;
mod resource_guard;

// Re-export the primitives
pub use task_id::TaskId;
pub use wait_queue::{WaitQueue, SharedWaitQueue};
pub use resource_guard::{ResourceGuard, ResourceManager, SharedResourceManager};

// Helper functions
pub fn shared_resource_manager() -> SharedResourceManager {
    resource_guard::shared_resource_manager()
}

pub fn shared_wait_queue() -> SharedWaitQueue {
    wait_queue::shared_wait_queue()
} 