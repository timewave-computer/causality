// Task identification module
//
// This module provides types for identifying and prioritizing tasks in the
// concurrent execution system.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Priority level for task execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
    /// Critical tasks that must execute as soon as possible
    Critical = 100,
    /// High-priority tasks
    High = 75,
    /// Normal priority tasks
    Normal = 50,
    /// Low-priority tasks (background processing)
    Low = 25,
    /// Custom priority value
    Custom(usize),
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

impl TaskPriority {
    /// Get the numeric value of the priority
    pub fn value(&self) -> usize {
        match self {
            TaskPriority::Critical => 100,
            TaskPriority::High => 75,
            TaskPriority::Normal => 50,
            TaskPriority::Low => 25,
            TaskPriority::Custom(value) => *value,
        }
    }
    
    /// Create a custom priority with the given value
    pub fn custom(value: usize) -> Self {
        TaskPriority::Custom(value)
    }
}

/// Unique identifier for a task
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TaskId {
    /// The unique ID of the task
    id: usize,
    /// The priority of the task
    priority: TaskPriority,
}

// A static counter for generating unique task IDs
static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);

impl TaskId {
    /// Create a new task ID with the given ID and priority
    pub fn new(id: usize, priority: TaskPriority) -> Self {
        Self {
            id,
            priority,
        }
    }
    
    /// Create a new task ID with an automatically generated ID
    pub fn auto(priority: TaskPriority) -> Self {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        Self::new(id, priority)
    }
    
    /// Get the priority of the task
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
    
    /// Get the raw ID of the task
    pub fn raw_id(&self) -> usize {
        self.id
    }
    
    /// Convert the task ID to a string representation
    pub fn as_str(&self) -> String {
        format!("task-{}", self.id)
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}", self.id)
    }
}

impl fmt::Debug for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}(priority={})", self.id, self.priority.value())
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
        
        let custom_high = TaskPriority::Custom(80);
        let custom_low = TaskPriority::Custom(30);
        
        assert!(custom_high > TaskPriority::High);
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