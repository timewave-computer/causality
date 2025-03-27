// Task scheduling utilities
//
// This module provides a task scheduler for handling concurrent task execution
// with resource dependencies.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use causality_types::{Error, Result};
use causality_crypto::ContentId;

use super::lock::DeterministicMutex;
use super::resource_manager::{ResourceManager, SharedResourceManager};
use super::task_id::{TaskId, TaskPriority};

/// Metrics for the task scheduler.
#[derive(Debug, Default, Clone)]
pub struct TaskSchedulerMetrics {
    /// Total number of tasks processed since scheduler creation
    pub total_tasks: usize,
    /// Number of tasks currently queued
    pub queued_tasks: usize,
    /// Number of tasks currently running
    pub running_tasks: usize,
    /// Number of tasks completed successfully
    pub completed_tasks: usize,
    /// Number of tasks that failed
    pub failed_tasks: usize,
    /// Average task waiting time in milliseconds
    pub avg_wait_time_ms: u64,
    /// Average task execution time in milliseconds
    pub avg_execution_time_ms: u64,
}

/// State of a task in the scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// Task is queued and waiting for execution
    Queued,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed(String),
    /// Task was cancelled before execution
    Cancelled,
}

/// Information about a task in the scheduler.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Unique identifier for the task
    pub id: TaskId,
    /// Priority of the task
    pub priority: TaskPriority,
    /// Resources required by the task
    pub required_resources: HashSet<ContentId>,
    /// Current state of the task
    pub state: TaskState,
    /// Time when the task was created
    pub created_at: Instant,
    /// Time when the task started execution (if started)
    pub started_at: Option<Instant>,
    /// Time when the task completed execution (if completed)
    pub completed_at: Option<Instant>,
}

impl TaskInfo {
    /// Create a new task info
    pub fn new(
        id: TaskId,
        priority: TaskPriority,
        required_resources: HashSet<ContentId>,
    ) -> Self {
        Self {
            id,
            priority,
            required_resources,
            state: TaskState::Queued,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
        }
    }
    
    /// Get the waiting time for the task in milliseconds
    pub fn wait_time_ms(&self) -> u64 {
        match self.started_at {
            Some(started) => started
                .duration_since(self.created_at)
                .as_millis() as u64,
            None => Instant::now()
                .duration_since(self.created_at)
                .as_millis() as u64,
        }
    }
    
    /// Get the execution time for the task in milliseconds
    pub fn execution_time_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(started), Some(completed)) => Some(completed
                .duration_since(started)
                .as_millis() as u64),
            (Some(started), None) => Some(Instant::now()
                .duration_since(started)
                .as_millis() as u64),
            _ => None,
        }
    }
    
    /// Check if the task has completed (successfully or with error)
    pub fn is_completed(&self) -> bool {
        matches!(self.state, TaskState::Completed | TaskState::Failed(_))
    }
    
    /// Check if the task is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, TaskState::Running)
    }
    
    /// Check if the task is queued
    pub fn is_queued(&self) -> bool {
        matches!(self.state, TaskState::Queued)
    }
    
    /// Check if the task was cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.state, TaskState::Cancelled)
    }
}

/// A scheduler for concurrent task execution.
pub struct TaskScheduler {
    /// Resource manager for handling resource allocation
    resource_manager: SharedResourceManager,
    /// Tasks managed by the scheduler
    tasks: DeterministicMutex<HashMap<TaskId, TaskInfo>>,
    /// Queue of tasks waiting to be executed
    task_queue: DeterministicMutex<VecDeque<TaskId>>,
    /// Maximum number of concurrent tasks
    max_concurrent_tasks: DeterministicMutex<usize>,
    /// Metrics for the scheduler
    metrics: DeterministicMutex<TaskSchedulerMetrics>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(resource_manager: SharedResourceManager) -> Self {
        Self {
            resource_manager,
            tasks: DeterministicMutex::new(HashMap::new()),
            task_queue: DeterministicMutex::new(VecDeque::new()),
            max_concurrent_tasks: DeterministicMutex::new(4), // Default concurrency
            metrics: DeterministicMutex::new(TaskSchedulerMetrics::default()),
        }
    }
    
    /// Set the maximum number of concurrent tasks
    pub fn set_max_concurrent_tasks(&self, max: usize) {
        let mut max_tasks = self.max_concurrent_tasks.lock();
        *max_tasks = max;
    }
    
    /// Get the maximum number of concurrent tasks
    pub fn get_max_concurrent_tasks(&self) -> usize {
        let max_tasks = self.max_concurrent_tasks.lock();
        *max_tasks
    }
    
    /// Get the number of currently running tasks
    pub fn get_running_tasks_count(&self) -> usize {
        let tasks = self.tasks.lock();
        tasks.values()
            .filter(|task| task.is_running())
            .count()
    }
    
    /// Get the number of currently queued tasks
    pub fn get_queued_tasks_count(&self) -> usize {
        let queue = self.task_queue.lock();
        queue.len()
    }
    
    /// Schedule a task for execution
    pub fn schedule(
        &self,
        id: TaskId, 
        priority: TaskPriority,
        required_resources: HashSet<ContentId>,
    ) -> Result<()> {
        // Create task info
        let task_info = TaskInfo::new(
            id.clone(),
            priority,
            required_resources,
        );
        
        // Add to tasks map
        let mut tasks = self.tasks.lock();
        if tasks.contains_key(&id) {
            return Err(Error::OperationFailed(
                format!("Task with ID {} already exists", id)
            ));
        }
        tasks.insert(id.clone(), task_info);
        
        // Add to queue
        let mut queue = self.task_queue.lock();
        queue.push_back(id);
        
        // Update metrics
        let mut metrics = self.metrics.lock();
        metrics.total_tasks += 1;
        metrics.queued_tasks += 1;
        
        Ok(())
    }
    
    /// Try to execute the next task in the queue
    pub fn try_execute_next(&self) -> Result<Option<TaskId>> {
        // Check if we can run more tasks
        let max_tasks = *self.max_concurrent_tasks.lock();
        let running_count = self.get_running_tasks_count();
        
        if running_count >= max_tasks {
            return Ok(None);
        }
        
        // Get the next task ID from the queue
        let mut queue = self.task_queue.lock();
        let task_id = match queue.pop_front() {
            Some(id) => id,
            None => return Ok(None),
        };
        
        // Get the task info
        let mut tasks = self.tasks.lock();
        let task = match tasks.get_mut(&task_id) {
            Some(task) => task,
            None => {
                // This shouldn't happen, but if it does, just return none
                return Ok(None);
            }
        };
        
        // Check if the task has been cancelled
        if task.is_cancelled() {
            // Update metrics
            let mut metrics = self.metrics.lock();
            metrics.queued_tasks -= 1;
            
            return Ok(None);
        }
        
        // Try to acquire all required resources
        // For simplicity, we don't actually acquire them here, just check availability
        let resources_available = task.required_resources.iter().all(|res| {
            self.resource_manager.is_resource_available(res.clone()).unwrap_or(false)
        });
        
        if !resources_available {
            // Resources not available, put back in queue
            queue.push_back(task_id);
            return Ok(None);
        }
        
        // Update task state
        task.state = TaskState::Running;
                    task.started_at = Some(Instant::now());
                    
        // Update metrics
        let mut metrics = self.metrics.lock();
        metrics.queued_tasks -= 1;
        metrics.running_tasks += 1;
        
        Ok(Some(task_id))
    }
    
    /// Mark a task as completed
    pub fn complete_task(&self, task_id: &TaskId, success: bool, error_msg: Option<String>) -> Result<()> {
        let mut tasks = self.tasks.lock();
        let task = match tasks.get_mut(task_id) {
            Some(task) => task,
            None => {
                return Err(Error::OperationFailed(
                    format!("Task with ID {} not found", task_id)
                ));
            }
        };
        
        // Check if the task is running
        if !task.is_running() {
            return Err(Error::OperationFailed(
                format!("Task with ID {} is not running", task_id)
            ));
        }
        
        // Update task state
        task.completed_at = Some(Instant::now());
        if success {
            task.state = TaskState::Completed;
                        } else {
            task.state = TaskState::Failed(error_msg.unwrap_or_else(|| "Unknown error".to_string()));
        }
        
        // Update metrics
        let mut metrics = self.metrics.lock();
        metrics.running_tasks -= 1;
        if success {
            metrics.completed_tasks += 1;
        } else {
            metrics.failed_tasks += 1;
        }
        
        // Update average times
        if let Some(exec_time) = task.execution_time_ms() {
            let current_avg = metrics.avg_execution_time_ms;
            let current_count = metrics.completed_tasks + metrics.failed_tasks - 1;
            
            if current_count > 0 {
                metrics.avg_execution_time_ms = 
                    (current_avg * (current_count as u64) + exec_time) / (current_count as u64 + 1);
            } else {
                metrics.avg_execution_time_ms = exec_time;
            }
        }
        
        let wait_time = task.wait_time_ms();
        let current_avg = metrics.avg_wait_time_ms;
        let current_count = metrics.total_tasks - metrics.queued_tasks - 1;
        
        if current_count > 0 {
            metrics.avg_wait_time_ms = 
                (current_avg * (current_count as u64) + wait_time) / (current_count as u64 + 1);
        } else {
            metrics.avg_wait_time_ms = wait_time;
        }
        
        Ok(())
    }
    
    /// Cancel a task
    pub fn cancel_task(&self, task_id: &TaskId) -> Result<()> {
        let mut tasks = self.tasks.lock();
        let task = match tasks.get_mut(task_id) {
            Some(task) => task,
            None => {
                return Err(Error::OperationFailed(
                    format!("Task with ID {} not found", task_id)
                ));
            }
        };
        
        // Check if the task can be cancelled
        if !task.is_queued() {
            return Err(Error::OperationFailed(
                format!("Task with ID {} is not in a cancellable state", task_id)
            ));
        }
        
        // Update task state
        task.state = TaskState::Cancelled;
        
        // No need to remove from queue now, it will be skipped when processed
            
            Ok(())
    }
    
    /// Get the current metrics for the scheduler
    pub fn get_metrics(&self) -> TaskSchedulerMetrics {
        let metrics = self.metrics.lock();
        metrics.clone()
    }
    
    /// Get a task by ID
    pub fn get_task(&self, task_id: &TaskId) -> Option<TaskInfo> {
        let tasks = self.tasks.lock();
        tasks.get(task_id).cloned()
    }
    
    /// Get all tasks in the scheduler
    pub fn get_all_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.lock();
        tasks.values().cloned().collect()
    }
    
    /// Get all running tasks
    pub fn get_running_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.lock();
        tasks.values()
            .filter(|task| task.is_running())
            .cloned()
            .collect()
    }
    
    /// Get all queued tasks
    pub fn get_queued_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.lock();
        tasks.values()
            .filter(|task| task.is_queued())
            .cloned()
            .collect()
    }
    
    /// Clean up completed and cancelled tasks older than the specified duration
    pub fn cleanup_old_tasks(&self, older_than: Duration) -> usize {
        let mut tasks = self.tasks.lock();
        let now = Instant::now();
        
        let to_remove: Vec<TaskId> = tasks.iter()
            .filter(|(_, task)| {
                (task.is_completed() || task.is_cancelled()) && 
                match task.completed_at {
                    Some(time) => now.duration_since(time) > older_than,
                    None => false,
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &to_remove {
            tasks.remove(id);
        }
        
        to_remove.len()
    }
}

impl fmt::Debug for TaskScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskScheduler")
            .field("running_tasks", &self.get_running_tasks_count())
            .field("queued_tasks", &self.get_queued_tasks_count())
            .field("max_concurrent_tasks", &self.get_max_concurrent_tasks())
            .field("metrics", &self.get_metrics())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    // Helper function to create a test scheduler
    fn create_test_scheduler() -> TaskScheduler {
        let resource_manager = ResourceManager::<ContentId, ()>::new();
        TaskScheduler::new(Arc::new(resource_manager))
    }
    
    #[test]
    fn test_task_scheduling_basic() {
        let scheduler = create_test_scheduler();
        let task_id = TaskId::new(1, TaskPriority::Normal);
        
        // Schedule a task
        scheduler.schedule(task_id.clone(), TaskPriority::Normal, HashSet::new()).unwrap();
        
        // Check task exists
        let task = scheduler.get_task(&task_id).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.state, TaskState::Queued);
        
        // Try to execute it
        let executed = scheduler.try_execute_next().unwrap();
        assert_eq!(executed, Some(task_id.clone()));
        
        // Check state changed
        let task = scheduler.get_task(&task_id).unwrap();
        assert_eq!(task.state, TaskState::Running);
        
        // Complete the task
        scheduler.complete_task(&task_id, true, None).unwrap();
        
        // Check state changed again
        let task = scheduler.get_task(&task_id).unwrap();
        assert_eq!(task.state, TaskState::Completed);
    }
    
    #[test]
    fn test_task_cancellation() {
        let scheduler = create_test_scheduler();
        let task_id = TaskId::new(1, TaskPriority::Normal);
        
        // Schedule a task
        scheduler.schedule(task_id.clone(), TaskPriority::Normal, HashSet::new()).unwrap();
        
        // Cancel it
        scheduler.cancel_task(&task_id).unwrap();
        
        // Check state
        let task = scheduler.get_task(&task_id).unwrap();
        assert_eq!(task.state, TaskState::Cancelled);
        
        // Try to execute - should skip the cancelled task
        let executed = scheduler.try_execute_next().unwrap();
        assert_eq!(executed, None);
    }
    
    #[test]
    fn test_task_metrics() {
        let scheduler = create_test_scheduler();
        
        // Schedule and execute some tasks
        for i in 0..5 {
            let task_id = TaskId::new(i, TaskPriority::Normal);
            scheduler.schedule(task_id.clone(), TaskPriority::Normal, HashSet::new()).unwrap();
            
            if let Some(id) = scheduler.try_execute_next().unwrap() {
                // Simulate some work
                thread::sleep(Duration::from_millis(10));
                
                // Complete task
                if i % 2 == 0 {
                    scheduler.complete_task(&id, true, None).unwrap();
                } else {
                    scheduler.complete_task(&id, false, Some("Test error".to_string())).unwrap();
                }
            }
        }
        
        // Check metrics
        let metrics = scheduler.get_metrics();
        assert_eq!(metrics.total_tasks, 5);
        assert_eq!(metrics.completed_tasks + metrics.failed_tasks, 4); // 4 completed or failed
        assert!(metrics.avg_execution_time_ms > 0); // Should have some execution time
    }
    
    #[test]
    fn test_concurrent_tasks_limit() {
        let scheduler = create_test_scheduler();
        scheduler.set_max_concurrent_tasks(2);
        
        // Schedule 5 tasks
        for i in 0..5 {
            let task_id = TaskId::new(i, TaskPriority::Normal);
            scheduler.schedule(task_id, TaskPriority::Normal, HashSet::new()).unwrap();
        }
        
        // Should only be able to execute 2
        let task1 = scheduler.try_execute_next().unwrap();
        let task2 = scheduler.try_execute_next().unwrap();
        let task3 = scheduler.try_execute_next().unwrap();
        
        assert!(task1.is_some());
        assert!(task2.is_some());
        assert!(task3.is_none()); // Should not execute 3rd task
        
        // Complete one task
        scheduler.complete_task(&task1.unwrap(), true, None).unwrap();
        
        // Now should be able to execute one more
        let task3 = scheduler.try_execute_next().unwrap();
        assert!(task3.is_some());
    }
} 
