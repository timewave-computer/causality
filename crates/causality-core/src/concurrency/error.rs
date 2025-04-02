// Concurrency errors
//
// This module defines error types for the concurrency subsystem.

use std::fmt::{self, Display};
use std::error::Error as StdError;
use causality_types::ContentId;

/// Error type for wait queue operations
#[derive(Debug)]
pub enum WaitQueueError {
    /// The operation failed due to a lock acquisition failure
    LockError(String),
    
    /// A deadlock was detected in the wait graph
    ResourceDeadlock,
    
    /// The requested resource does not exist
    ResourceNotFound(ContentId),
    
    /// The requestor does not exist
    RequestorNotFound(String),
    
    /// An internal error occurred
    InternalError(String),
}

impl Display for WaitQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
            Self::ResourceDeadlock => write!(f, "Resource deadlock detected"),
            Self::ResourceNotFound(id) => write!(f, "Resource not found: {:?}", id),
            Self::RequestorNotFound(id) => write!(f, "Requestor not found: {}", id),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl StdError for WaitQueueError {}

/// Error type for task scheduler operations
#[derive(Debug)]
pub enum TaskSchedulerError {
    /// A task with the same ID already exists
    TaskAlreadyExists(String),
    
    /// The requested task does not exist
    TaskNotFound(String),
    
    /// The task is not in a runnable state
    TaskNotRunnable(String),
    
    /// The task is not in a running state
    TaskNotRunning(String),
    
    /// The task is not in a cancellable state
    TaskNotInCancellableState(String),
    
    /// The required resources are not available
    ResourcesUnavailable(String),
    
    /// An error occurred in the resource manager
    ResourceManagerError(String),
    
    /// An internal error occurred
    InternalError(String),
}

impl Display for TaskSchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskAlreadyExists(msg) => write!(f, "Task already exists: {}", msg),
            Self::TaskNotFound(msg) => write!(f, "Task not found: {}", msg),
            Self::TaskNotRunnable(msg) => write!(f, "Task not runnable: {}", msg),
            Self::TaskNotRunning(msg) => write!(f, "Task not running: {}", msg),
            Self::TaskNotInCancellableState(msg) => write!(f, "Task not cancellable: {}", msg),
            Self::ResourcesUnavailable(msg) => write!(f, "Resources unavailable: {}", msg),
            Self::ResourceManagerError(msg) => write!(f, "Resource manager error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl StdError for TaskSchedulerError {} 