// Concurrency primitives and patterns for the Causality system
// 
// This module provides a unified interface for concurrency primitives and patterns used
// throughout the Causality codebase.

pub mod error;
pub mod lock;
pub mod pool;
pub mod wait_queue;
pub mod task_scheduler;
pub mod patterns;
pub mod task_id;
pub mod atomic;

// Resource management modules (disabled until dependencies are resolved)
// pub mod resource_guard;
// pub mod resource_manager;

// Import necessary std components
use std::sync::Arc;

// Export core error types
pub use error::WaitQueueError;
pub use error::TaskSchedulerError;

// Export core concurrency primitives
pub use tokio::sync::Barrier;
pub use lock::{SharedMutex, DeterministicMutex, DeterministicRwLock};
pub use wait_queue::{WaitQueue, WaitQueue as WaitQueueEntry};
pub use patterns::ConcurrencyPatterns;
pub use task_id::{TaskId, TaskPriority};
pub use task_scheduler::{TaskScheduler, TaskInfo as Task, TaskState as TaskStatus};

// Re-export core async/await functionality
pub use tokio::spawn;
pub use tokio::time::sleep;
pub use tokio::task::yield_now;

// Re-exports for public API
pub use atomic::{
    AtomicCell, AtomicCounter, AtomicFlag,
    SharedCell, SharedCounter, SharedFlag,
};

pub use lock::{
    Cell, 
    DeterministicMutexGuard,
    DeterministicReadGuard, DeterministicWriteGuard,
    SharedRwLock,
};

pub use pool::{
    ResourcePool, ResourceManager, SharedResourceManager, ResourceHandle,
};

pub use wait_queue::{
    WaitFuture,
};

// Helper functions for creating common concurrency primitives
/// Create a new shared resource manager
pub fn shared_resource_manager<K, V>() -> SharedResourceManager<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    SharedResourceManager::new()
}

/// Create a new shared wait queue
pub fn shared_wait_queue() -> Arc<WaitQueue> {
    Arc::new(WaitQueue::new())
}

/// Create a new resource manager (non-shared version)
pub fn new_resource_manager<K, V>() -> ResourceManager<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    ResourceManager::new()
}

/// Create a new wait queue (non-shared version)
pub fn new_wait_queue() -> WaitQueue {
    WaitQueue::new()
}

/// Create a new shared mutex
pub fn shared_mutex<T>(value: T) -> SharedMutex<T> 
where 
    T: Send + 'static,
{
    SharedMutex::new(value)
}

// Mark legacy modules as deprecated
#[deprecated(since = "0.9.0", note = "Use task_scheduler module directly")]
pub mod scheduler {
    //! Legacy module, use task_scheduler module directly
    #[deprecated(since = "0.9.0", note = "Use task_scheduler module directly")]
    pub use super::task_scheduler::*;
}

// Define error and result types that were missing
/// General error type for the concurrency subsystem
pub type Error = std::io::Error;

/// Result type for concurrency operations
pub type Result<T> = std::result::Result<T, Error>;

// Define the TaskResult type since it's missing from pool
/// Result of a task execution
pub type TaskResult<T> = std::result::Result<T, Error>;

/// Thread pool type
pub type ThreadPool = tokio::runtime::Runtime; 