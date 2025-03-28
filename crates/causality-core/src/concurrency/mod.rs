// Concurrency module
//
// This module provides various concurrency utilities and primitives.

// Core modules
pub mod atomic;     // Atomic operations and primitives
pub mod lock;       // Locking mechanisms
pub mod pool;       // Resource pool implementation
pub mod wait_queue; // Deterministic wait queue for resource management
pub mod resource_guard;   // Resource guard for safe resource access
pub mod resource_manager; // Resource management utilities
pub mod task_scheduler;   // Task scheduling
pub mod task_id;          // Task identifiers

// Pattern modules
pub mod patterns;   // High-level concurrency patterns
pub mod barrier;    // Barrier synchronization
pub mod fork;       // Fork-join parallelism
pub mod race;       // Racing between tasks
pub mod timeout;    // Timeout handling

// Re-exporting core types for easier access
use resource_manager::{ResourceManager, SharedResourceManager};
use wait_queue::{WaitQueue, SharedWaitQueue};

// Re-exporting common concurrency patterns
pub use fork::{fork, join};
pub use race::{race, select};
pub use barrier::Barrier;
pub use barrier::SharedBarrier;
pub use timeout::{with_timeout, TimeoutError};

// Re-exports for public API
pub use atomic::{
    AtomicCell, AtomicCounter, AtomicFlag,
    SharedCell, SharedCounter, SharedFlag,
};

pub use lock::{
    Cell, 
    DeterministicMutex, DeterministicMutexGuard,
    DeterministicRwLock, DeterministicReadGuard, DeterministicWriteGuard,
    SharedMutex, SharedRwLock,
};

pub use pool::{
    ResourcePool, ResourceManager, SharedResourceManager, ResourceHandle,
};

pub use wait_queue::{
    WaitQueue, SharedWaitQueue, WaitFuture,
};

pub use resource_guard::{
    ResourceGuard, WeakResourceRef, ResourceRegisterGuard,
};

pub use resource_manager::{
    ResourceManager as ResourceMgr, SharedResourceManager as SharedResourceMgr,
};

pub use task_scheduler::{
    TaskScheduler, TaskInfo, TaskState, TaskSchedulerMetrics,
};

pub use task_id::{
    TaskId, TaskPriority,
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
pub fn shared_wait_queue() -> SharedWaitQueue {
    wait_queue::shared()
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

// Mark legacy modules as deprecated
#[deprecated(since = "0.9.0", note = "Use the new direct imports instead")]
pub mod primitives {
    //! Legacy module, use direct imports instead
    #[deprecated(since = "0.9.0", note = "Use new modules directly")]
    pub use super::*;
}

#[deprecated(since = "0.9.0", note = "Use patterns module directly")]
pub mod scheduler {
    //! Legacy module, use task_scheduler module directly
    #[deprecated(since = "0.9.0", note = "Use task_scheduler module directly")]
    pub use super::task_scheduler::*;
} 