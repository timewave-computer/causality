// Wait queue implementation for async operations
// Deterministic wait queue for resource management
//
// This module provides a deterministic wait queue for resource contention,
// ensuring that resource acquisition is predictable and reproducible.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::future::Future;
use std::pin::Pin;

use std::result::Result;
use causality_types::ContentId;
use super::task_id::TaskId;

/// A shared wait queue that can be cloned and shared between components
pub type SharedWaitQueue = Arc<WaitQueue>;

/// Create a shared instance of the wait queue
pub fn shared() -> SharedWaitQueue {
    Arc::new(WaitQueue::new())
}

/// A deterministic wait queue for resource contention
///
/// The wait queue ensures that resource acquisition follows a predictable
/// order, which is essential for deterministic execution.
#[derive(Debug)]
pub struct WaitQueue {
    /// Map of resources to queues of requestors
    queues: Mutex<HashMap<ContentId, VecDeque<String>>>,
    /// Map of requestors to the resources they own
    owned_resources: Mutex<HashMap<String, HashSet<ContentId>>>,
    /// Map of requestors to the resources they are waiting for
    waiting_for: Mutex<HashMap<String, ContentId>>,
    /// Queue of wakers for tasks waiting
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
        resource: ContentId,
        requestor: String,
        owned_resources: HashSet<ContentId>,
    ) -> Result<(), super::error::WaitQueueError> {
        // Lock the queues
        let mut queues = self.queues.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock wait queues".to_string()))?;
            
        let queue = queues.entry(resource.clone()).or_insert_with(VecDeque::new);
        
        // Only add the requestor if they're not already in the queue
        if !queue.contains(&requestor) {
            queue.push_back(requestor.clone());
        }
        
        // Update owned resources
        let mut owned = self.owned_resources.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock owned resources".to_string()))?;
            
        owned.insert(requestor.clone(), owned_resources);
        
        // Update waiting_for
        let mut waiting = self.waiting_for.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock waiting for resources".to_string()))?;
            
        waiting.insert(requestor, resource);
        
        // Check for deadlocks
        if self.has_deadlock() {
            return Err(super::error::WaitQueueError::ResourceDeadlock);
        }
        
        Ok(())
    }
    
    /// Remove a requestor from the wait queue for a resource
    pub fn remove_requestor(&self, resource: ContentId, requestor: &str) -> Result<(), super::error::WaitQueueError> {
        // Update queues
        let mut queues = self.queues.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock wait queues".to_string()))?;
            
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
            super::error::WaitQueueError::LockError("Failed to lock owned resources".to_string()))?;
            
        owned.remove(requestor);
        
        // Remove from waiting_for
        let mut waiting = self.waiting_for.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock waiting for resources".to_string()))?;
            
        waiting.remove(requestor);
        
        Ok(())
    }
    
    /// Get the next requestor for a resource
    ///
    /// Returns None if there are no requestors for the resource.
    pub fn get_next_requestor(&self, resource: ContentId) -> Result<Option<String>, super::error::WaitQueueError> {
        let queues = self.queues.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock wait queues".to_string()))?;
            
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
        resources: HashSet<ContentId>,
    ) -> Result<(), super::error::WaitQueueError> {
        let mut owned = self.owned_resources.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock owned resources".to_string()))?;
            
        if owned.contains_key(requestor) {
            owned.insert(requestor.to_string(), resources);
        }
        
        // Check for deadlocks after updating ownership
        if self.has_deadlock() {
            return Err(super::error::WaitQueueError::ResourceDeadlock);
        }
        
        Ok(())
    }
    
    /// Check if there is a deadlock in the wait graph
    ///
    /// A deadlock exists if there is a cycle in the wait graph.
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
            for &neighbor in neighbors {
                if self.has_cycle(graph, neighbor, visited, path) {
                    return true;
                }
            }
        }
        
        // Remove the node from the current path
        path.remove(node);
        
        // Mark the node as visited
        visited.insert(node);
        
        false
    }
    
    /// Get the number of requestors waiting for a resource
    pub fn queue_length(&self, resource: ContentId) -> Result<usize, super::error::WaitQueueError> {
        let queues = self.queues.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock wait queues".to_string()))?;
            
        Ok(queues.get(&resource).map_or(0, |q| q.len()))
    }
    
    /// Get the set of resources that have waiters
    pub fn resources_with_waiters(&self) -> Result<HashSet<ContentId>, super::error::WaitQueueError> {
        let queues = self.queues.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock wait queues".to_string()))?;
            
        Ok(queues.keys().cloned().collect())
    }
    
    /// Get the resource that a requestor is waiting for
    pub fn requestor_waiting_for(&self, requestor: &str) -> Result<Option<ContentId>, super::error::WaitQueueError> {
        let waiting = self.waiting_for.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock waiting for resources".to_string()))?;
            
        Ok(waiting.get(requestor).cloned())
    }
    
    /// Get the resources owned by a requestor
    pub fn get_owned_resources(&self, requestor: &str) -> Result<Option<HashSet<ContentId>>, super::error::WaitQueueError> {
        let owned = self.owned_resources.lock().map_err(|_| 
            super::error::WaitQueueError::LockError("Failed to lock owned resources".to_string()))?;
            
        Ok(owned.get(requestor).cloned())
    }
    
    /// Check if a requestor is next in line for a resource
    pub fn is_next_requestor(&self, resource: ContentId, requestor: &str) -> Result<bool, super::error::WaitQueueError> {
        let next = self.get_next_requestor(resource)?;
        Ok(next.as_deref() == Some(requestor))
    }
    
    /// Add a waker to the wait queue
    pub fn enqueue(&self, task_id: TaskId, waker: Waker) {
        let mut wakers = self.wakers.lock().unwrap();
        wakers.push_back((task_id, waker));
    }
    
    /// Remove a waker from the wait queue
    pub fn remove(&self, task_id: &TaskId) -> Option<Waker> {
        let mut wakers = self.wakers.lock().unwrap();
        
        if let Some(pos) = wakers.iter().position(|(id, _)| id == task_id) {
            let (_, waker) = wakers.remove(pos).unwrap();
            Some(waker)
        } else {
            None
        }
    }
    
    /// Wake the next task in the wait queue
    pub fn wake_next(&self) -> Option<TaskId> {
        let mut wakers = self.wakers.lock().unwrap();
        
        if let Some((task_id, waker)) = wakers.pop_front() {
            waker.wake();
            Some(task_id)
        } else {
            None
        }
    }
    
    /// Wake all tasks in the wait queue
    pub fn wake_all(&self) -> Vec<TaskId> {
        let mut wakers = self.wakers.lock().unwrap();
        let mut task_ids = Vec::with_capacity(wakers.len());
        
        while let Some((task_id, waker)) = wakers.pop_front() {
            waker.wake();
            task_ids.push(task_id);
        }
        
        task_ids
    }
    
    /// Check if the wait queue is empty
    pub fn is_empty(&self) -> bool {
        let wakers = self.wakers.lock().unwrap();
        wakers.is_empty()
    }
    
    /// Get the number of tasks in the wait queue
    pub fn len(&self) -> usize {
        let wakers = self.wakers.lock().unwrap();
        wakers.len()
    }
    
    /// Create a future that waits for a resource to be available
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

impl Clone for WaitQueue {
    fn clone(&self) -> Self {
        WaitQueue {
            queues: Mutex::new(self.queues.lock().unwrap().clone()),
            owned_resources: Mutex::new(self.owned_resources.lock().unwrap().clone()),
            waiting_for: Mutex::new(self.waiting_for.lock().unwrap().clone()),
            wakers: Mutex::new(self.wakers.lock().unwrap().clone()),
        }
    }
}

/// A future that waits for a resource to be available
#[derive(Debug)]
pub struct WaitFuture {
    wait_queue: Arc<WaitQueue>,
    task_id: TaskId,
}

impl Future for WaitFuture {
    type Output = Result<(), std::io::Error>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Register the waker
        self.wait_queue.enqueue(self.task_id.clone(), cx.waker().clone());
        
        // Return pending to wait for the waker to be called
        Poll::Pending
    }
}

impl Drop for WaitFuture {
    fn drop(&mut self) {
        // When the future is dropped, remove the waker from the queue
        // to avoid any potential memory leaks
        let _ = self.wait_queue.remove(&self.task_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wait_queue_basic() -> Result<(), WaitQueueError> {
        let wait_queue = WaitQueue::new();
        
        // Add some requestors
        wait_queue.add_requestor(
            "resource1".into(),
            "requestor1".to_string(),
            HashSet::new(),
        )?;
        
        wait_queue.add_requestor(
            "resource1".into(),
            "requestor2".to_string(),
            HashSet::new(),
        )?;
        
        // Check queue state
        assert_eq!(wait_queue.queue_length("resource1".into())?, 2);
        assert_eq!(
            wait_queue.get_next_requestor("resource1".into())?,
            Some("requestor1".to_string())
        );
        
        // Remove a requestor
        wait_queue.remove_requestor("resource1".into(), "requestor1")?;
        
        // Check queue state again
        assert_eq!(wait_queue.queue_length("resource1".into())?, 1);
        assert_eq!(
            wait_queue.get_next_requestor("resource1".into())?,
            Some("requestor2".to_string())
        );
        
        Ok(())
    }
    
    #[test]
    fn test_wait_queue_deadlock_detection() -> Result<(), WaitQueueError> {
        let wait_queue = WaitQueue::new();
        
        // Set up a potential deadlock scenario
        
        // Requestor1 owns resource1, waiting for resource2
        let mut owned1 = HashSet::new();
        owned1.insert("resource1".into());
        wait_queue.add_requestor(
            "resource2".into(),
            "requestor1".to_string(),
            owned1,
        )?;
        
        // Requestor2 owns resource2, waiting for resource1
        // This should cause a deadlock
        let mut owned2 = HashSet::new();
        owned2.insert("resource2".into());
        
        // This should fail with ResourceDeadlock
        let result = wait_queue.add_requestor(
            "resource1".into(),
            "requestor2".to_string(),
            owned2,
        );
        
        assert!(matches!(result, Err(WaitQueueError::ResourceDeadlock)));
        
        Ok(())
    }
    
    #[test]
    fn test_wait_queue_wake_next() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        
        struct MockWaker {
            woken: Arc<AtomicBool>,
        }
        
        impl MockWaker {
            fn new() -> (Self, Arc<AtomicBool>) {
                let woken = Arc::new(AtomicBool::new(false));
                (Self { woken: woken.clone() }, woken)
            }
            
            fn into_waker(self) -> Waker {
                use std::task::{RawWaker, RawWakerVTable};
                
                // Not a real implementation, just for testing
                // In a real scenario, you'd implement proper waker logic
                unsafe {
                    let ptr = Box::into_raw(Box::new(self));
                    Waker::from_raw(RawWaker::new(
                        ptr as *const (),
                        &RawWakerVTable::new(
                            |_| RawWaker::new(ptr as *const (), &RawWakerVTable::new(|_| panic!(), |_| {}, |_| {}, |_| {})),
                            |p| {
                                let waker = Box::from_raw(p as *mut MockWaker);
                                waker.woken.store(true, Ordering::SeqCst);
                            },
                            |_| {},
                            |p| drop(Box::from_raw(p as *mut MockWaker)),
                        ),
                    ))
                }
            }
        }
        
        let wait_queue = WaitQueue::new();
        let task_id = TaskId::new(1, Default::default());
        
        let (waker, woken) = MockWaker::new();
        wait_queue.enqueue(task_id.clone(), waker.into_waker());
        
        // Wake the next task
        let woken_id = wait_queue.wake_next();
        assert_eq!(woken_id, Some(task_id));
        assert!(woken.load(Ordering::SeqCst));
    }
} 
