// Resource pool and management module
//
// This module provides abstractions for managing shared resources in a pool
// for efficient allocation and reuse.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::sync::Arc;

use super::lock::{DeterministicMutex, SharedMutex};

/// A resource pool that manages a collection of resources
#[derive(Debug)]
pub struct ResourcePool<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Available resources that can be acquired
    available: HashMap<K, V>,
    
    /// Resources that are currently in use
    in_use: HashSet<K>,
    
    /// Queue of waiters for resources
    waiters: VecDeque<K>,
}

impl<K, V> ResourcePool<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new empty resource pool
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
            in_use: HashSet::new(),
            waiters: VecDeque::new(),
        }
    }
    
    /// Create a new resource pool with an initial set of resources
    pub fn with_resources(resources: HashMap<K, V>) -> Self {
        Self {
            available: resources,
            in_use: HashSet::new(),
            waiters: VecDeque::new(),
        }
    }
    
    /// Add a new resource to the pool
    pub fn add_resource(&mut self, key: K, resource: V) {
        self.available.insert(key, resource);
    }
    
    /// Get the number of available resources
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
    
    /// Get the number of resources in use
    pub fn in_use_count(&self) -> usize {
        self.in_use.len()
    }
    
    /// Get the total number of resources
    pub fn total_count(&self) -> usize {
        self.available.len() + self.in_use.len()
    }
    
    /// Check if a resource with the given key is available
    pub fn is_available(&self, key: &K) -> bool {
        self.available.contains_key(key)
    }
    
    /// Check if a resource with the given key is in use
    pub fn is_in_use(&self, key: &K) -> bool {
        self.in_use.contains(key)
    }
    
    /// Try to acquire a resource by key
    pub fn try_acquire(&mut self, key: &K) -> Option<V> {
        if let Some(resource) = self.available.remove(key) {
            self.in_use.insert(key.clone());
            Some(resource)
        } else {
            if !self.in_use.contains(key) {
                // Resource doesn't exist at all
                None
            } else {
                // Resource exists but is in use, add to waiters
                self.waiters.push_back(key.clone());
                None
            }
        }
    }
    
    /// Release a resource back to the pool
    pub fn release(&mut self, key: K, resource: V) -> bool {
        if self.in_use.remove(&key) {
            // If there are waiters for this resource, handle them
            if let Some(pos) = self.waiters.iter().position(|k| k == &key) {
                self.waiters.remove(pos);
            }
            
            self.available.insert(key, resource);
            true
        } else {
            false
        }
    }
    
    /// Remove a resource from the pool entirely
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(resource) = self.available.remove(key) {
            Some(resource)
        } else if self.in_use.remove(key) {
            // The resource is in use, so we can't return it
            // but we've removed it from the in_use set so it won't
            // be returned to the pool when released
            None
        } else {
            None
        }
    }
    
    /// Get an iterator over all available resource keys
    pub fn available_keys(&self) -> impl Iterator<Item = &K> {
        self.available.keys()
    }
    
    /// Get an iterator over all in-use resource keys
    pub fn in_use_keys(&self) -> impl Iterator<Item = &K> {
        self.in_use.iter()
    }
    
    /// Get an iterator over all waiting resource keys
    pub fn waiter_keys(&self) -> impl Iterator<Item = &K> {
        self.waiters.iter()
    }
}

impl<K, V> Default for ResourcePool<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-safe resource manager that wraps a resource pool
pub struct ResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    /// The inner resource pool protected by a mutex
    pool: DeterministicMutex<ResourcePool<K, V>>,
}

impl<K, V> ResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new empty resource manager
    pub fn new() -> Self {
        Self {
            pool: DeterministicMutex::new(ResourcePool::new()),
        }
    }
    
    /// Create a new resource manager with an initial set of resources
    pub fn with_resources(resources: HashMap<K, V>) -> Self {
        Self {
            pool: DeterministicMutex::new(ResourcePool::with_resources(resources)),
        }
    }
    
    /// Add a new resource to the pool
    pub fn add_resource(&self, key: K, resource: V) {
        let mut pool = self.pool.lock();
        pool.add_resource(key, resource);
    }
    
    /// Get the number of available resources
    pub fn available_count(&self) -> usize {
        let pool = self.pool.lock();
        pool.available_count()
    }
    
    /// Get the number of resources in use
    pub fn in_use_count(&self) -> usize {
        let pool = self.pool.lock();
        pool.in_use_count()
    }
    
    /// Get the total number of resources
    pub fn total_count(&self) -> usize {
        let pool = self.pool.lock();
        pool.total_count()
    }
    
    /// Check if a resource with the given key is available
    pub fn is_available(&self, key: &K) -> bool {
        let pool = self.pool.lock();
        pool.is_available(key)
    }
    
    /// Check if a resource with the given key is in use
    pub fn is_in_use(&self, key: &K) -> bool {
        let pool = self.pool.lock();
        pool.is_in_use(key)
    }
    
    /// Try to acquire a resource by key
    pub fn try_acquire(&self, key: &K) -> Option<V> {
        let mut pool = self.pool.lock();
        pool.try_acquire(key)
    }
    
    /// Release a resource back to the pool
    pub fn release(&self, key: K, resource: V) -> bool {
        let mut pool = self.pool.lock();
        pool.release(key, resource)
    }
    
    /// Remove a resource from the pool entirely
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut pool = self.pool.lock();
        pool.remove(key)
    }
    
    /// Get a snapshot of all available resource keys
    pub fn available_keys(&self) -> HashSet<K> {
        let pool = self.pool.lock();
        pool.available_keys().cloned().collect()
    }
    
    /// Get a snapshot of all in-use resource keys
    pub fn in_use_keys(&self) -> HashSet<K> {
        let pool = self.pool.lock();
        pool.in_use_keys().cloned().collect()
    }
    
    /// Get a snapshot of all waiting resource keys
    pub fn waiter_keys(&self) -> Vec<K> {
        let pool = self.pool.lock();
        pool.waiter_keys().cloned().collect()
    }
}

impl<K, V> Default for ResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A shared resource manager that can be cloned
#[derive(Clone)]
pub struct SharedResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    /// The inner resource manager
    manager: Arc<ResourceManager<K, V>>,
}

impl<K, V> SharedResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new empty shared resource manager
    pub fn new() -> Self {
        Self {
            manager: Arc::new(ResourceManager::new()),
        }
    }
    
    /// Create a new shared resource manager with an initial set of resources
    pub fn with_resources(resources: HashMap<K, V>) -> Self {
        Self {
            manager: Arc::new(ResourceManager::with_resources(resources)),
        }
    }
    
    /// Add a new resource to the pool
    pub fn add_resource(&self, key: K, resource: V) {
        self.manager.add_resource(key, resource);
    }
    
    /// Get the number of available resources
    pub fn available_count(&self) -> usize {
        self.manager.available_count()
    }
    
    /// Get the number of resources in use
    pub fn in_use_count(&self) -> usize {
        self.manager.in_use_count()
    }
    
    /// Get the total number of resources
    pub fn total_count(&self) -> usize {
        self.manager.total_count()
    }
    
    /// Check if a resource with the given key is available
    pub fn is_available(&self, key: &K) -> bool {
        self.manager.is_available(key)
    }
    
    /// Check if a resource with the given key is in use
    pub fn is_in_use(&self, key: &K) -> bool {
        self.manager.is_in_use(key)
    }
    
    /// Try to acquire a resource by key
    pub fn try_acquire(&self, key: &K) -> Option<V> {
        self.manager.try_acquire(key)
    }
    
    /// Release a resource back to the pool
    pub fn release(&self, key: K, resource: V) -> bool {
        self.manager.release(key, resource)
    }
    
    /// Remove a resource from the pool entirely
    pub fn remove(&self, key: &K) -> Option<V> {
        self.manager.remove(key)
    }
    
    /// Get a snapshot of all available resource keys
    pub fn available_keys(&self) -> HashSet<K> {
        self.manager.available_keys()
    }
    
    /// Get a snapshot of all in-use resource keys
    pub fn in_use_keys(&self) -> HashSet<K> {
        self.manager.in_use_keys()
    }
    
    /// Get a snapshot of all waiting resource keys
    pub fn waiter_keys(&self) -> Vec<K> {
        self.manager.waiter_keys()
    }
}

impl<K, V> Default for SharedResourceManager<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A handle to a resource that automatically returns it to the pool when dropped
pub struct ResourceHandle<K, V>
where
    K: Eq + Hash + Clone,
{
    /// The key associated with this resource
    key: Option<K>,
    
    /// The resource itself
    resource: Option<V>,
    
    /// Reference to the resource manager
    manager: SharedResourceManager<K, V>,
}

impl<K, V> ResourceHandle<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new resource handle
    pub fn new(key: K, resource: V, manager: SharedResourceManager<K, V>) -> Self {
        Self {
            key: Some(key),
            resource: Some(resource),
            manager,
        }
    }
    
    /// Get a reference to the resource
    pub fn get(&self) -> Option<&V> {
        self.resource.as_ref()
    }
    
    /// Get a mutable reference to the resource
    pub fn get_mut(&mut self) -> Option<&mut V> {
        self.resource.as_mut()
    }
    
    /// Get the key associated with this resource
    pub fn key(&self) -> Option<&K> {
        self.key.as_ref()
    }
    
    /// Manually release the resource back to the pool
    pub fn release(&mut self) {
        if let (Some(key), Some(resource)) = (self.key.take(), self.resource.take()) {
            self.manager.release(key, resource);
        }
    }
}

impl<K, V> Drop for ResourceHandle<K, V>
where
    K: Eq + Hash + Clone,
{
    fn drop(&mut self) {
        self.release();
    }
}

/// A wait queue for resources
#[derive(Debug)]
pub struct WaitQueue<T> {
    /// The inner queue protected by a mutex
    queue: DeterministicMutex<VecDeque<T>>,
}

impl<T> WaitQueue<T> {
    /// Create a new empty wait queue
    pub fn new() -> Self {
        Self {
            queue: DeterministicMutex::new(VecDeque::new()),
        }
    }
    
    /// Add an item to the end of the queue
    pub fn enqueue(&self, item: T) {
        let mut queue = self.queue.lock();
        queue.push_back(item);
    }
    
    /// Add an item to the front of the queue
    pub fn enqueue_front(&self, item: T) {
        let mut queue = self.queue.lock();
        queue.push_front(item);
    }
    
    /// Remove and return the item at the front of the queue
    pub fn dequeue(&self) -> Option<T> {
        let mut queue = self.queue.lock();
        queue.pop_front()
    }
    
    /// Get the number of items in the queue
    pub fn len(&self) -> usize {
        let queue = self.queue.lock();
        queue.len()
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock();
        queue.is_empty()
    }
    
    /// Clear the queue
    pub fn clear(&self) {
        let mut queue = self.queue.lock();
        queue.clear();
    }
}

impl<T> Default for WaitQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A shared wait queue that can be cloned
#[derive(Clone)]
pub struct SharedWaitQueue<T> {
    /// The inner wait queue
    queue: Arc<WaitQueue<T>>,
}

impl<T> SharedWaitQueue<T> {
    /// Create a new empty shared wait queue
    pub fn new() -> Self {
        Self {
            queue: Arc::new(WaitQueue::new()),
        }
    }
    
    /// Add an item to the end of the queue
    pub fn enqueue(&self, item: T) {
        self.queue.enqueue(item);
    }
    
    /// Add an item to the front of the queue
    pub fn enqueue_front(&self, item: T) {
        self.queue.enqueue_front(item);
    }
    
    /// Remove and return the item at the front of the queue
    pub fn dequeue(&self) -> Option<T> {
        self.queue.dequeue()
    }
    
    /// Get the number of items in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Clear the queue
    pub fn clear(&self) {
        self.queue.clear();
    }
}

impl<T> Default for SharedWaitQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions to create resource management components
pub mod helpers {
    use super::*;
    
    /// Create a new shared resource manager
    pub fn new_resource_manager<K, V>() -> SharedResourceManager<K, V> 
    where
        K: Eq + Hash + Clone,
    {
        SharedResourceManager::new()
    }
    
    /// Create a new shared wait queue
    pub fn new_wait_queue<T>() -> SharedWaitQueue<T> {
        SharedWaitQueue::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_pool() {
        let mut pool = ResourcePool::<String, i32>::new();
        
        pool.add_resource("resource1".to_string(), 42);
        pool.add_resource("resource2".to_string(), 84);
        
        assert_eq!(pool.available_count(), 2);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.total_count(), 2);
        
        let r1 = pool.try_acquire(&"resource1".to_string()).unwrap();
        assert_eq!(r1, 42);
        
        assert_eq!(pool.available_count(), 1);
        assert_eq!(pool.in_use_count(), 1);
        assert_eq!(pool.total_count(), 2);
        
        let released = pool.release("resource1".to_string(), 42);
        assert!(released);
        
        assert_eq!(pool.available_count(), 2);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.total_count(), 2);
    }
    
    #[test]
    fn test_resource_manager() {
        let manager = ResourceManager::<String, i32>::new();
        
        manager.add_resource("resource1".to_string(), 42);
        manager.add_resource("resource2".to_string(), 84);
        
        assert_eq!(manager.available_count(), 2);
        assert_eq!(manager.in_use_count(), 0);
        assert_eq!(manager.total_count(), 2);
        
        let r1 = manager.try_acquire(&"resource1".to_string()).unwrap();
        assert_eq!(r1, 42);
        
        assert_eq!(manager.available_count(), 1);
        assert_eq!(manager.in_use_count(), 1);
        assert_eq!(manager.total_count(), 2);
        
        let released = manager.release("resource1".to_string(), 42);
        assert!(released);
        
        assert_eq!(manager.available_count(), 2);
        assert_eq!(manager.in_use_count(), 0);
        assert_eq!(manager.total_count(), 2);
    }
    
    #[test]
    fn test_shared_resource_manager() {
        let manager = SharedResourceManager::<String, i32>::new();
        
        manager.add_resource("resource1".to_string(), 42);
        manager.add_resource("resource2".to_string(), 84);
        
        let manager2 = manager.clone();
        
        assert_eq!(manager.available_count(), 2);
        assert_eq!(manager.in_use_count(), 0);
        assert_eq!(manager.total_count(), 2);
        
        let r1 = manager.try_acquire(&"resource1".to_string()).unwrap();
        assert_eq!(r1, 42);
        
        assert_eq!(manager2.available_count(), 1);
        assert_eq!(manager2.in_use_count(), 1);
        assert_eq!(manager2.total_count(), 2);
        
        let released = manager.release("resource1".to_string(), 42);
        assert!(released);
        
        assert_eq!(manager2.available_count(), 2);
        assert_eq!(manager2.in_use_count(), 0);
        assert_eq!(manager2.total_count(), 2);
    }
    
    #[test]
    fn test_resource_handle() {
        let manager = SharedResourceManager::<String, i32>::new();
        
        manager.add_resource("resource1".to_string(), 42);
        manager.add_resource("resource2".to_string(), 84);
        
        assert_eq!(manager.available_count(), 2);
        
        {
            let resource = manager.try_acquire(&"resource1".to_string()).unwrap();
            let mut handle = ResourceHandle::new("resource1".to_string(), resource, manager.clone());
            
            assert_eq!(manager.available_count(), 1);
            assert_eq!(manager.in_use_count(), 1);
            
            assert_eq!(*handle.get().unwrap(), 42);
            *handle.get_mut().unwrap() = 100;
            assert_eq!(*handle.get().unwrap(), 100);
        }
        
        // The handle is dropped, so the resource should be released back to the pool
        assert_eq!(manager.available_count(), 2);
        assert_eq!(manager.in_use_count(), 0);
        
        // The value should be updated in the pool
        let resource = manager.try_acquire(&"resource1".to_string()).unwrap();
        assert_eq!(resource, 100);
    }
    
    #[test]
    fn test_wait_queue() {
        let queue = WaitQueue::<i32>::new();
        
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        
        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());
        
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.len(), 1);
        
        queue.enqueue_front(4);
        assert_eq!(queue.dequeue(), Some(4));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
        
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_shared_wait_queue() {
        let queue = SharedWaitQueue::<i32>::new();
        let queue2 = queue.clone();
        
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        
        queue.enqueue(1);
        queue.enqueue(2);
        queue2.enqueue(3);
        
        assert_eq!(queue.len(), 3);
        assert_eq!(queue2.len(), 3);
        assert!(!queue.is_empty());
        assert!(!queue2.is_empty());
        
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue2.dequeue(), Some(2));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue2.len(), 1);
        
        queue.enqueue_front(4);
        assert_eq!(queue2.dequeue(), Some(4));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue2.dequeue(), None);
        
        assert_eq!(queue.len(), 0);
        assert_eq!(queue2.len(), 0);
        assert!(queue.is_empty());
        assert!(queue2.is_empty());
    }
} 