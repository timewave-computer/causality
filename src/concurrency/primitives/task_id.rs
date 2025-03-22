// Task identifier for concurrency operations
//
// Provides a unique identifier for tasks within the concurrency system,
// allowing tracking and management of resources across concurrent operations.

use std::fmt;
use rand::random;
use std::time::{SystemTime, UNIX_EPOCH};

/// A unique identifier for tasks in the concurrency system
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TaskId {
    id: String,
}

impl TaskId {
    /// Create a new task ID with a unique identifier
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        let random_part = random::<u64>();
        let id = format!("task-{}-{}", timestamp, random_part);
        
        TaskId { id }
    }
    
    /// Get the task ID as a string
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        TaskId { id: s }
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        TaskId { id: s.to_string() }
    }
}

impl fmt::Debug for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskId({})", self.id)
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
} 