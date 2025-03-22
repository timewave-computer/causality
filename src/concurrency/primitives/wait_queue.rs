// Deterministic wait queue for resource management
//
// This module provides a deterministic wait queue for resource contention,
// ensuring that resource acquisition is predictable and reproducible in both
// native execution and the ZK VM.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::{Error, Result};
use crate::types::ResourceId;
use super::TaskId;

/// A shared wait queue that can be cloned and shared between components
pub type SharedWaitQueue = Arc<WaitQueue>;

/// Create a shared instance of the wait queue
pub fn shared() -> SharedWaitQueue {
    Arc::new(WaitQueue::new())
}

/// A deterministic wait queue for resource contention
///
/// The wait queue ensures that resource acquisition follows a predictable
/// order, which is essential for deterministic execution and proof generation.
#[derive(Debug)]
pub struct WaitQueue {
    // Map of resources to queues of requestors
    queues: Mutex<HashMap<ResourceId, VecDeque<String>>>,
    // Map of requestors to the resources they own
    // This is used for deadlock detection
    owned_resources: Mutex<HashMap<String, HashSet<ResourceId>>>,
    // Map of requestors to the resources they are waiting for
    waiting_for: Mutex<HashMap<String, ResourceId>>,
    /// The queue of wakers for tasks waiting
    wakers: Mutex<VecDeque<(TaskId, Waker)>>,
}

impl WaitQueue {
    /// Create a new wait queue
    pub fn new() -> Self {
        WaitQueue {
            queues: Mutex::new(HashMap::new()),
            owned_resources: Mutex::new(HashMap::new()),
            waiting_for: Mutex::new(HashMap::new()),
            wakers: Mutex::new(VecDeque::new()),
        }
    }
    
    /// Add a requestor to the wait queue for a resource
    ///
    /// The `owned_resources` parameter is the set of resources that the requestor
    /// currently owns. This is used for deadlock detection.
    pub fn add_requestor(
        &self,
        resource: ResourceId,
        requestor: String,
        owned_resources: HashSet<ResourceId>,
    ) -> Result<()> {
        // Update queues
        let mut queues = self.queues.lock().map_err(|_| 
            Error::InternalError("Failed to lock wait queues".to_string()))?;
            
        let queue = queues.entry(resource.clone()).or_insert_with(VecDeque::new);
        
        // Only add the requestor if they're not already in the queue
        if !queue.contains(&requestor) {
            queue.push_back(requestor.clone());
        }
        
        // Update owned resources
        let mut owned = self.owned_resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock owned resources".to_string()))?;
            
        owned.insert(requestor.clone(), owned_resources);
        
        // Update waiting_for
        let mut waiting = self.waiting_for.lock().map_err(|_| 
            Error::InternalError("Failed to lock waiting for resources".to_string()))?;
            
        waiting.insert(requestor, resource);
        
        // Check for deadlocks
        if self.has_deadlock() {
            return Err(Error::ResourceDeadlock);
        }
        
        Ok(())
    }
    
    /// Remove a requestor from the wait queue for a resource
    pub fn remove_requestor(&self, resource: ResourceId, requestor: &str) -> Result<()> {
        // Update queues
        let mut queues = self.queues.lock().map_err(|_| 
            Error::InternalError("Failed to lock wait queues".to_string()))?;
            
        if let Some(queue) = queues.get_mut(&resource) {
            // Find and remove the requestor
            let position = queue.iter().position(|r| r == requestor);
            if let Some(pos) = position {
                queue.remove(pos);
            }
            
            // If the queue is empty, remove it from the map
            if queue.is_empty() {
                queues.remove(&resource);
            }
        }
        
        // Remove from owned resources
        let mut owned = self.owned_resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock owned resources".to_string()))?;
            
        owned.remove(requestor);
        
        // Remove from waiting_for
        let mut waiting = self.waiting_for.lock().map_err(|_| 
            Error::InternalError("Failed to lock waiting for resources".to_string()))?;
            
        waiting.remove(requestor);
        
        Ok(())
    }
    
    /// Get the next requestor for a resource
    ///
    /// Returns None if there are no requestors for the resource.
    pub fn get_next_requestor(&self, resource: ResourceId) -> Result<Option<String>> {
        let queues = self.queues.lock().map_err(|_| 
            Error::InternalError("Failed to lock wait queues".to_string()))?;
            
        if let Some(queue) = queues.get(&resource) {
            Ok(queue.front().cloned())
        } else {
            Ok(None)
        }
    }
    
    /// Update the resource ownership for a requestor
    pub fn update_owned_resources(
        &self,
        requestor: &str,
        resources: HashSet<ResourceId>,
    ) -> Result<()> {
        let mut owned = self.owned_resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock owned resources".to_string()))?;
            
        if owned.contains_key(requestor) {
            owned.insert(requestor.to_string(), resources);
        }
        
        // Check for deadlocks after updating ownership
        if self.has_deadlock() {
            return Err(Error::ResourceDeadlock);
        }
        
        Ok(())
    }
    
    /// Check if there is a deadlock in the wait graph
    ///
    /// This uses a cycle detection algorithm on the wait graph to determine
    /// if there is a deadlock. A deadlock exists if there is a cycle in the
    /// wait graph, where A waits for a resource owned by B, B waits for a
    /// resource owned by C, and C waits for a resource owned by A.
    fn has_deadlock(&self) -> bool {
        // Get the locks on the data structures
        let queues = match self.queues.lock() {
            Ok(queues) => queues,
            Err(_) => return false, // If we can't get the lock, assume no deadlock
        };
        
        let owned_resources = match self.owned_resources.lock() {
            Ok(owned) => owned,
            Err(_) => return false,
        };
        
        let waiting_for = match self.waiting_for.lock() {
            Ok(waiting) => waiting,
            Err(_) => return false,
        };
        
        // Build a graph of requestor dependencies
        // A -> B means A is waiting for a resource owned by B
        let mut graph: HashMap<&str, HashSet<&str>> = HashMap::new();
        
        // For each requestor waiting for a resource
        for (requestor, resource) in waiting_for.iter() {
            // Find all other requestors that own this resource
            for (other_requestor, resources) in owned_resources.iter() {
                // Skip self-dependencies
                if requestor == other_requestor {
                    continue;
                }
                
                // If the other requestor owns the resource this requestor is waiting for
                if resources.contains(resource) {
                    // Add an edge from this requestor to the other requestor
                    graph.entry(requestor)
                         .or_insert_with(HashSet::new)
                         .insert(other_requestor);
                }
            }
        }
        
        // Perform cycle detection using depth-first search
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        
        // For each node in the graph
        for node in graph.keys() {
            if self.has_cycle(&graph, *node, &mut visited, &mut path) {
                return true;
            }
        }
        
        false
    }
    
    /// Helper function for cycle detection using depth-first search
    fn has_cycle<'a>(
        &self,
        graph: &HashMap<&'a str, HashSet<&'a str>>,
        node: &'a str,
        visited: &mut HashSet<&'a str>,
        path: &mut HashSet<&'a str>,
    ) -> bool {
        // If we've already fully explored this node, no cycle here
        if visited.contains(node) {
            return false;
        }
        
        // If we're already in the current path, we found a cycle
        if path.contains(node) {
            return true;
        }
        
        // Add the node to our current path
        path.insert(node);
        
        // Visit all neighbors
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if self.has_cycle(graph, neighbor, visited, path) {
                    return true;
                }
            }
        }
        
        // Remove the node from the current path
        path.remove(node);
        
        // Mark as fully visited
        visited.insert(node);
        
        false
    }
    
    /// Get the number of requestors waiting for a resource
    pub fn queue_length(&self, resource: ResourceId) -> Result<usize> {
        let queues = self.queues.lock().map_err(|_| 
            Error::InternalError("Failed to lock wait queues".to_string()))?;
            
        if let Some(queue) = queues.get(&resource) {
            Ok(queue.len())
        } else {
            Ok(0)
        }
    }
    
    /// Get all resources that have waiters
    pub fn resources_with_waiters(&self) -> Result<HashSet<ResourceId>> {
        let queues = self.queues.lock().map_err(|_| 
            Error::InternalError("Failed to lock wait queues".to_string()))?;
            
        Ok(queues.keys().cloned().collect())
    }
    
    /// Get the resources that a requestor is waiting for
    pub fn requestor_waiting_for(&self, requestor: &str) -> Result<Option<ResourceId>> {
        let waiting = self.waiting_for.lock().map_err(|_| 
            Error::InternalError("Failed to lock waiting for resources".to_string()))?;
            
        Ok(waiting.get(requestor).cloned())
    }
    
    /// Get all the resources owned by a requestor
    pub fn get_owned_resources(&self, requestor: &str) -> Result<Option<HashSet<ResourceId>>> {
        let owned = self.owned_resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock owned resources".to_string()))?;
            
        Ok(owned.get(requestor).cloned())
    }
    
    /// Check if a requestor is at the front of a resource's wait queue
    pub fn is_next_requestor(&self, resource: ResourceId, requestor: &str) -> Result<bool> {
        match self.get_next_requestor(resource)? {
            Some(next) => Ok(next == requestor),
            None => Ok(false),
        }
    }

    /// Add a task to the wait queue
    pub fn enqueue(&self, task_id: TaskId, waker: Waker) {
        let mut wakers = self.wakers.lock().unwrap();
        wakers.push_back((task_id, waker));
    }

    /// Remove a task from the wait queue
    pub fn remove(&self, task_id: &TaskId) -> Option<Waker> {
        let mut wakers = self.wakers.lock().unwrap();
        let index = wakers.iter().position(|(id, _)| id == task_id)?;
        let (_, waker) = wakers.remove(index).unwrap();
        Some(waker)
    }

    /// Wake the next task in the queue
    pub fn wake_next(&self) -> Option<TaskId> {
        let mut wakers = self.wakers.lock().unwrap();
        if let Some((task_id, waker)) = wakers.pop_front() {
            waker.wake();
            Some(task_id)
        } else {
            None
        }
    }

    /// Wake all tasks in the queue
    pub fn wake_all(&self) -> Vec<TaskId> {
        let mut wakers = self.wakers.lock().unwrap();
        let mut task_ids = Vec::with_capacity(wakers.len());
        
        while let Some((task_id, waker)) = wakers.pop_front() {
            waker.wake();
            task_ids.push(task_id);
        }
        
        task_ids
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        let wakers = self.wakers.lock().unwrap();
        wakers.is_empty()
    }

    /// Get the number of tasks in the queue
    pub fn len(&self) -> usize {
        let wakers = self.wakers.lock().unwrap();
        wakers.len()
    }

    /// Create a future that will be resolved when this task is woken
    pub fn wait(&self, task_id: TaskId) -> WaitFuture {
        WaitFuture {
            wait_queue: Arc::new(self.clone()),
            task_id,
        }
    }
}

impl Default for WaitQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Future that resolves when a task is woken from the wait queue
pub struct WaitFuture {
    wait_queue: Arc<WaitQueue>,
    task_id: TaskId,
}

impl Future for WaitFuture {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Register the waker with the wait queue
        self.wait_queue.enqueue(self.task_id.clone(), cx.waker().clone());
        
        // Return pending, as this future will be completed when woken
        Poll::Pending
    }
}

impl Drop for WaitFuture {
    fn drop(&mut self) {
        // Clean up by removing this task from the wait queue if it's still there
        self.wait_queue.remove(&self.task_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wait_queue_basic() -> Result<()> {
        let queue = WaitQueue::new();
        
        // Add a requestor
        queue.add_requestor(
            ResourceId::new("test"),
            "requestor1".to_string(),
            HashSet::new(),
        )?;
        
        // Check queue length
        assert_eq!(queue.queue_length(ResourceId::new("test"))?, 1);
        
        // Get next requestor
        let next = queue.get_next_requestor(ResourceId::new("test"))?;
        assert_eq!(next, Some("requestor1".to_string()));
        
        // Remove the requestor
        queue.remove_requestor(ResourceId::new("test"), "requestor1")?;
        
        // Check queue is empty
        assert_eq!(queue.queue_length(ResourceId::new("test"))?, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_wait_queue_deadlock_detection() -> Result<()> {
        let queue = WaitQueue::new();
        
        // Setup:
        // requestor1 owns resource A
        // requestor2 owns resource B
        // requestor1 wants resource B
        // requestor2 wants resource A
        
        // First, add requestor1 owning resource A
        let mut owned1 = HashSet::new();
        owned1.insert(ResourceId::new("A"));
        
        // First, add requestor2 owning resource B
        let mut owned2 = HashSet::new();
        owned2.insert(ResourceId::new("B"));
        
        // Add requestor1 waiting for resource B
        queue.add_requestor(
            ResourceId::new("B"),
            "requestor1".to_string(),
            owned1,
        )?;
        
        // Add requestor2 waiting for resource A - this should cause a deadlock error
        let result = queue.add_requestor(
            ResourceId::new("A"),
            "requestor2".to_string(),
            owned2,
        );
        
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, Error::ResourceDeadlock));
        }
        
        Ok(())
    }

    #[test]
    fn test_wait_queue_basics() {
        let queue = WaitQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        // We can't easily test the waker functionality in a synchronous test,
        // but we can test the queue operations
        let task_id = TaskId::new();
        let waker = futures::task::noop_waker();
        
        queue.enqueue(task_id.clone(), waker.clone());
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);
        
        let removed = queue.remove(&task_id);
        assert!(removed.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_wait_queue_wake_next() {
        let queue = WaitQueue::new();
        let task_id1 = TaskId::new();
        let task_id2 = TaskId::new();
        let waker = futures::task::noop_waker();
        
        queue.enqueue(task_id1.clone(), waker.clone());
        queue.enqueue(task_id2.clone(), waker.clone());
        
        assert_eq!(queue.len(), 2);
        
        let woken = queue.wake_next();
        assert_eq!(woken, Some(task_id1));
        assert_eq!(queue.len(), 1);
        
        let woken = queue.wake_next();
        assert_eq!(woken, Some(task_id2));
        assert_eq!(queue.len(), 0);
        
        let woken = queue.wake_next();
        assert_eq!(woken, None);
    }

    #[test]
    fn test_wait_queue_wake_all() {
        let queue = WaitQueue::new();
        let task_id1 = TaskId::new();
        let task_id2 = TaskId::new();
        let waker = futures::task::noop_waker();
        
        queue.enqueue(task_id1.clone(), waker.clone());
        queue.enqueue(task_id2.clone(), waker.clone());
        
        assert_eq!(queue.len(), 2);
        
        let woken = queue.wake_all();
        assert_eq!(woken.len(), 2);
        assert!(woken.contains(&task_id1));
        assert!(woken.contains(&task_id2));
        assert_eq!(queue.len(), 0);
    }
} 