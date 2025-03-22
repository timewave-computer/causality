// Concurrency primitives for Causality
//
// This module provides concurrency primitives and patterns for
// safe, deterministic concurrent execution.

// Import and export the submodules
pub mod primitives;
pub mod patterns;
pub mod scheduler;

// Re-export commonly used items from the primitives module
pub use primitives::{
    TaskId,
    ResourceGuard,
    ResourceManager,
    SharedResourceManager,
    WaitQueue,
    SharedWaitQueue,
};

// Re-export commonly used patterns
pub use patterns::{
    fork, fork_join, fork_try_join, fork_each,
    race, race_ok, race_result, race_until,
    barrier, resource_barrier, timeout_barrier, wait_for_resources, Barrier,
    timeout, timeout_result, with_timeout, WithTimeout, timeout_with_retry,
};

// Re-export the scheduler
pub use scheduler::TaskScheduler;

// Helper functions
pub fn shared_resource_manager() -> SharedResourceManager {
    primitives::shared_resource_manager()
}

pub fn shared_wait_queue() -> SharedWaitQueue {
    primitives::shared_wait_queue()
} 