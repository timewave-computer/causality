// Task ID and priority types
//
// This module provides types for identifying tasks and their priorities
// for the task scheduler.

use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Task priority levels
///
/// Used to determine which tasks should be executed first when
/// resources are limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum TaskPriority {
    /// Lowest priority, executed last
    Low = 0,
    /// Normal priority, executed after high priority tasks
    Normal = 1,
    /// High priority, executed after critical tasks
    High = 2,
    /// Critical priority, executed first
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

impl Display for TaskPriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Normal => write!(f, "Normal"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Task ID
///
/// A unique identifier for a task in the scheduler. Consists of a sequence number
/// and a priority level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId {
    /// Sequence number for the task
    pub seq: u64,
    /// Priority level for the task
    pub priority: TaskPriority,
}

// A static counter for generating unique task IDs
static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);

impl TaskId {
    /// Create a new task ID with the given sequence number and priority
    pub fn new(seq: u64, priority: TaskPriority) -> Self {
        Self { seq, priority }
    }
    
    /// Create a new task ID with an automatically generated ID
    pub fn auto(priority: TaskPriority) -> Self {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        Self::new(id as u64, priority)
    }
    
    /// Get the priority of the task
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
    
    /// Get the raw ID of the task
    pub fn raw_id(&self) -> u64 {
        self.seq
    }
    
    /// Convert the task ID to a string representation
    pub fn as_str(&self) -> String {
        format!("task-{}", self.seq)
    }
}

impl Hash for TaskId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.seq.hash(state);
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}({})", self.seq, self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_priority() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        
        // The Custom variant doesn't exist in TaskPriority enum
        // We need to use the actual variants that exist
        let custom_high = TaskPriority::High;
        let custom_low = TaskPriority::Low;
        
        assert!(custom_high > TaskPriority::Normal);
        assert!(custom_high < TaskPriority::Critical);
        assert!(custom_low > TaskPriority::Low);
        assert!(custom_low < TaskPriority::Normal);
    }
    
    #[test]
    fn test_task_id_creation() {
        let task1 = TaskId::new(1, TaskPriority::Normal);
        let task2 = TaskId::new(2, TaskPriority::High);
        
        assert_ne!(task1, task2);
        assert_eq!(task1.priority(), TaskPriority::Normal);
        assert_eq!(task2.priority(), TaskPriority::High);
        assert_eq!(task1.raw_id(), 1);
        assert_eq!(task2.raw_id(), 2);
    }
    
    #[test]
    fn test_auto_task_id() {
        let task1 = TaskId::auto(TaskPriority::Normal);
        let task2 = TaskId::auto(TaskPriority::Normal);
        
        assert_ne!(task1, task2);
        assert_eq!(task1.priority(), TaskPriority::Normal);
        assert_eq!(task2.priority(), TaskPriority::Normal);
    }
    
    #[test]
    fn test_task_id_string_representation() {
        let task = TaskId::new(42, TaskPriority::High);
        
        assert_eq!(task.as_str(), "task-42");
        assert_eq!(task.to_string(), "Task-42");
    }
} 