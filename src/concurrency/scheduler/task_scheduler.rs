// Task scheduler for concurrent tasks
//
// This module provides a scheduler for managing concurrent tasks.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::future::Future;
use std::pin::Pin;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::error::{Error, Result};
use crate::types::ResourceId;
use crate::concurrency::primitives::{TaskId, ResourceManager, SharedResourceManager, ResourceGuard};

/// A task scheduler for managing concurrent tasks
///
/// The task scheduler manages the execution of tasks, handling resource
/// acquisition, prioritization, and scheduling.
#[derive(Clone)]
pub struct TaskScheduler {
    /// The resource manager to use for acquiring resources
    resource_manager: SharedResourceManager,
    /// The tasks currently running
    tasks: Arc<Mutex<HashMap<TaskId, TaskInfo>>>,
    /// The task queue for pending tasks
    task_queue: Arc<Mutex<VecDeque<TaskId>>>,
    /// The maximum number of concurrent tasks
    max_concurrent_tasks: Arc<Mutex<usize>>,
}

/// Information about a task
#[derive(Clone)]
struct TaskInfo {
    /// The task ID
    id: TaskId,
    /// The task priority (higher is more important)
    priority: usize,
    /// The resources required by the task
    required_resources: HashSet<ResourceId>,
    /// The resources currently held by the task
    held_resources: HashSet<ResourceId>,
    /// The time the task was created
    created_at: Instant,
    /// The time the task was started, if it's running
    started_at: Option<Instant>,
    /// The sender for the task result
    result_sender: Option<oneshot::Sender<Box<dyn Any + Send + 'static>>>,
    /// The join handle for the task
    join_handle: Option<JoinHandle<()>>,
}

use std::any::Any;

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(resource_manager: SharedResourceManager) -> Self {
        TaskScheduler {
            resource_manager,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            max_concurrent_tasks: Arc::new(Mutex::new(10)), // Default to 10 concurrent tasks
        }
    }
    
    /// Set the maximum number of concurrent tasks
    pub fn set_max_concurrent_tasks(&self, max: usize) -> Result<()> {
        let mut max_concurrent_tasks = self.max_concurrent_tasks.lock().map_err(|_| 
            Error::InternalError("Failed to lock max_concurrent_tasks".to_string()))?;
            
        *max_concurrent_tasks = max;
        
        Ok(())
    }
    
    /// Get the maximum number of concurrent tasks
    pub fn get_max_concurrent_tasks(&self) -> Result<usize> {
        let max_concurrent_tasks = self.max_concurrent_tasks.lock().map_err(|_| 
            Error::InternalError("Failed to lock max_concurrent_tasks".to_string()))?;
            
        Ok(*max_concurrent_tasks)
    }
    
    /// Get the number of running tasks
    pub fn get_running_tasks_count(&self) -> Result<usize> {
        let tasks = self.tasks.lock().map_err(|_| 
            Error::InternalError("Failed to lock tasks".to_string()))?;
            
        Ok(tasks.values().filter(|task| task.started_at.is_some()).count())
    }
    
    /// Get the number of queued tasks
    pub fn get_queued_tasks_count(&self) -> Result<usize> {
        let queue = self.task_queue.lock().map_err(|_| 
            Error::InternalError("Failed to lock task_queue".to_string()))?;
            
        Ok(queue.len())
    }
    
    /// Schedule a task for execution
    ///
    /// This method adds a task to the scheduler and schedules it
    /// for execution when resources are available.
    pub fn schedule<F, T>(&self, 
        id: TaskId, 
        future: F, 
        required_resources: HashSet<ResourceId>,
        priority: usize,
    ) -> impl Future<Output = Result<T>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Create a channel for the task result
        let (result_tx, result_rx) = oneshot::channel();
        
        // Create task info
        let task_info = TaskInfo {
            id: id.clone(),
            priority,
            required_resources,
            held_resources: HashSet::new(),
            created_at: Instant::now(),
            started_at: None,
            result_sender: Some(result_tx),
            join_handle: None,
        };
        
        // Add the task to the task map
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id.clone(), task_info);
        }
        
        // Add the task to the queue
        {
            let mut queue = self.task_queue.lock().unwrap();
            queue.push_back(id.clone());
        }
        
        // Try to schedule tasks
        self.try_schedule_tasks();
        
        // Return a future that waits for the task to complete
        async move {
            match result_rx.await {
                Ok(result) => {
                    // Try to downcast the result
                    match result.downcast::<Result<T>>() {
                        Ok(result) => *result,
                        Err(_) => Err(Error::InternalError("Failed to downcast task result".to_string())),
                    }
                },
                Err(_) => Err(Error::OperationFailed("Task cancelled or panicked".to_string())),
            }
        }
    }
    
    /// Try to schedule pending tasks
    ///
    /// This method tries to schedule pending tasks from the queue.
    fn try_schedule_tasks(&self) {
        // Get the current state
        let mut tasks = match self.tasks.lock() {
            Ok(tasks) => tasks,
            Err(_) => return,
        };
        
        let mut queue = match self.task_queue.lock() {
            Ok(queue) => queue,
            Err(_) => return,
        };
        
        let max_concurrent_tasks = match self.max_concurrent_tasks.lock() {
            Ok(max) => *max,
            Err(_) => return,
        };
        
        // Count running tasks
        let running_tasks = tasks.values().filter(|task| task.started_at.is_some()).count();
        
        // If we're at max capacity, don't schedule more tasks
        if running_tasks >= max_concurrent_tasks {
            return;
        }
        
        // Try to schedule as many tasks as we can
        let mut scheduled = 0;
        
        // Sort the queue by priority (highest first) and age (oldest first)
        let mut task_ids: Vec<TaskId> = queue.iter().cloned().collect();
        task_ids.sort_by(|a, b| {
            let task_a = &tasks[a];
            let task_b = &tasks[b];
            
            // Sort by priority (higher is more important)
            let prio_cmp = task_b.priority.cmp(&task_a.priority);
            if prio_cmp != std::cmp::Ordering::Equal {
                return prio_cmp;
            }
            
            // If priorities are equal, sort by age (older is more important)
            task_a.created_at.cmp(&task_b.created_at)
        });
        
        // Available slots
        let available_slots = max_concurrent_tasks - running_tasks;
        
        // Try to schedule tasks
        for _ in 0..available_slots {
            if task_ids.is_empty() {
                break;
            }
            
            // Get the next task ID
            let task_id = task_ids.remove(0);
            
            // Try to schedule the task
            if let Some(task) = tasks.get_mut(&task_id) {
                let resources_available = task.required_resources.iter().all(|resource| {
                    self.resource_manager.is_resource_available(resource.clone()).unwrap_or(false)
                });
                
                if resources_available {
                    // Remove the task from the queue
                    queue.retain(|id| *id != task_id);
                    
                    // Schedule the task
                    let scheduler = self.clone();
                    let task_id = task.id.clone();
                    let required_resources = task.required_resources.clone();
                    
                    // Take the result sender
                    let result_sender = task.result_sender.take();
                    
                    // Start the task
                    task.started_at = Some(Instant::now());
                    
                    // Spawn a tokio task for the actual work
                    let handle = tokio::spawn(async move {
                        // Acquire resources
                        let mut guards = Vec::new();
                        for resource in required_resources {
                            match scheduler.resource_manager.acquire_resource::<()>(resource.clone(), &task_id.as_str()).await {
                                Ok(guard) => {
                                    guards.push(guard);
                                },
                                Err(err) => {
                                    // Failed to acquire a resource, release the ones we already have
                                    drop(guards);
                                    
                                    // Send an error result
                                    if let Some(sender) = result_sender {
                                        let _ = sender.send(Box::new(Err::<(), Error>(err)));
                                    }
                                    
                                    return;
                                }
                            }
                        }
                        
                        // TODO: Execute the actual task
                        
                        // Release resources
                        drop(guards);
                        
                        // TODO: Send the result
                    });
                    
                    // Store the join handle
                    task.join_handle = Some(handle);
                    
                    scheduled += 1;
                }
            }
        }
        
        // If we scheduled any tasks, update the queue
        if scheduled > 0 {
            // Rebuild the queue from the remaining task IDs
            queue.clear();
            for id in task_ids {
                queue.push_back(id);
            }
        }
    }
    
    /// Cancel a task
    ///
    /// This method cancels a task, removing it from the scheduler.
    pub fn cancel_task(&self, id: &TaskId) -> Result<()> {
        // Remove the task from the task map
        let mut tasks = self.tasks.lock().map_err(|_| 
            Error::InternalError("Failed to lock tasks".to_string()))?;
            
        if let Some(task) = tasks.remove(id) {
            // If the task is running, abort it
            if let Some(handle) = task.join_handle {
                handle.abort();
            }
            
            // If the task has a result sender, send a cancellation error
            if let Some(sender) = task.result_sender {
                let _ = sender.send(Box::new(Err::<(), Error>(Error::OperationCancelled("Task cancelled".to_string()))));
            }
            
            // Remove the task from the queue
            let mut queue = self.task_queue.lock().map_err(|_| 
                Error::InternalError("Failed to lock task_queue".to_string()))?;
                
            queue.retain(|qid| qid != id);
            
            Ok(())
        } else {
            Err(Error::OperationFailed(format!("Task not found: {:?}", id)))
        }
    }
    
    /// Wait for all tasks to complete
    ///
    /// This method waits for all currently scheduled tasks to complete.
    pub async fn wait_for_all(&self) -> Result<()> {
        loop {
            // Get the number of tasks
            let tasks_count = {
                let tasks = self.tasks.lock().map_err(|_| 
                    Error::InternalError("Failed to lock tasks".to_string()))?;
                    
                tasks.len()
            };
            
            // If there are no tasks, we're done
            if tasks_count == 0 {
                return Ok(());
            }
            
            // Wait a bit
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrency::primitives::ResourceManager;
    
    #[tokio::test]
    async fn test_task_scheduler_basic() -> Result<()> {
        // Create a resource manager
        let resource_manager = Arc::new(ResourceManager::new());
        
        // Create a scheduler
        let scheduler = TaskScheduler::new(resource_manager.clone());
        
        // Set max concurrent tasks
        scheduler.set_max_concurrent_tasks(5)?;
        assert_eq!(scheduler.get_max_concurrent_tasks()?, 5);
        
        // TODO: Test scheduling and executing tasks
        
        Ok(())
    }
    
    // TODO: Add more tests
} 