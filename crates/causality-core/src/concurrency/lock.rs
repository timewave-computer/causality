// Concurrency locking mechanisms
//
// This module provides higher-level abstractions over Rust's standard
// synchronization primitives, focusing on causality's deterministic concurrency needs.

use std::sync::{Arc, Mutex, RwLock};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::marker::PhantomData;

/// A mutex for protecting data with exclusive access
///
/// This implementation ensures deterministic behavior in concurrent contexts.
#[derive(Debug)]
pub struct DeterministicMutex<T> {
    /// The inner mutex and its data
    inner: Mutex<T>,
}

impl<T> DeterministicMutex<T> {
    /// Create a new mutex
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }
    
    /// Acquire a lock on the mutex
    pub fn lock(&self) -> DeterministicMutexGuard<'_, T> {
        // We unwrap here because in a deterministic context, mutex poisoning
        // should never happen. If it does, it's a fatal error.
        DeterministicMutexGuard {
            guard: self.inner.lock().unwrap(),
        }
    }
    
    /// Try to acquire a lock on the mutex
    pub fn try_lock(&self) -> Option<DeterministicMutexGuard<'_, T>> {
        match self.inner.try_lock() {
            Ok(guard) => Some(DeterministicMutexGuard { guard }),
            Err(_) => None,
        }
    }

    /// Get a reference to the inner Mutex
    pub fn as_inner(&self) -> &Mutex<T> {
        &self.inner
    }
}

impl<T> Default for DeterministicMutex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A guard for a deterministic mutex
pub struct DeterministicMutexGuard<'a, T> {
    /// The inner mutex guard
    guard: std::sync::MutexGuard<'a, T>,
}

impl<'a, T> Deref for DeterministicMutexGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for DeterministicMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for DeterministicMutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeterministicMutexGuard")
            .field("value", &self.guard)
            .finish()
    }
}

/// A shared mutex that can be cloned
#[derive(Debug, Clone)]
pub struct SharedMutex<T>(Arc<DeterministicMutex<T>>);

impl<T> SharedMutex<T> {
    /// Create a new shared mutex
    pub fn new(data: T) -> Self {
        Self(Arc::new(DeterministicMutex::new(data)))
    }
    
    /// Acquire a lock on the mutex
    pub fn lock(&self) -> DeterministicMutexGuard<'_, T> {
        self.0.lock()
    }
    
    /// Try to acquire a lock on the mutex
    pub fn try_lock(&self) -> Option<DeterministicMutexGuard<'_, T>> {
        self.0.try_lock()
    }
}

impl<T> Default for SharedMutex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A read-write lock that allows multiple readers or a single writer
#[derive(Debug)]
pub struct DeterministicRwLock<T> {
    /// The inner read-write lock and its data
    inner: RwLock<T>,
}

impl<T> DeterministicRwLock<T> {
    /// Create a new read-write lock
    pub fn new(data: T) -> Self {
        Self {
            inner: RwLock::new(data),
        }
    }
    
    /// Acquire a read lock
    pub fn read(&self) -> DeterministicReadGuard<'_, T> {
        // Unwrap is safe in deterministic context
        DeterministicReadGuard {
            guard: self.inner.read().unwrap(),
        }
    }
    
    /// Try to acquire a read lock
    pub fn try_read(&self) -> Option<DeterministicReadGuard<'_, T>> {
        match self.inner.try_read() {
            Ok(guard) => Some(DeterministicReadGuard { guard }),
            Err(_) => None,
        }
    }
    
    /// Acquire a write lock
    pub fn write(&self) -> DeterministicWriteGuard<'_, T> {
        // Unwrap is safe in deterministic context
        DeterministicWriteGuard {
            guard: self.inner.write().unwrap(),
        }
    }
    
    /// Try to acquire a write lock
    pub fn try_write(&self) -> Option<DeterministicWriteGuard<'_, T>> {
        match self.inner.try_write() {
            Ok(guard) => Some(DeterministicWriteGuard { guard }),
            Err(_) => None,
        }
    }

    /// Get a reference to the inner RwLock
    pub fn as_inner(&self) -> &RwLock<T> {
        &self.inner
    }
}

impl<T> Default for DeterministicRwLock<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A read guard for a deterministic read-write lock
pub struct DeterministicReadGuard<'a, T> {
    /// The inner read guard
    guard: std::sync::RwLockReadGuard<'a, T>,
}

impl<'a, T> Deref for DeterministicReadGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for DeterministicReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeterministicReadGuard")
            .field("value", &self.guard)
            .finish()
    }
}

/// A write guard for a deterministic read-write lock
pub struct DeterministicWriteGuard<'a, T> {
    /// The inner write guard
    guard: std::sync::RwLockWriteGuard<'a, T>,
}

impl<'a, T> Deref for DeterministicWriteGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for DeterministicWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for DeterministicWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeterministicWriteGuard")
            .field("value", &self.guard)
            .finish()
    }
}

/// A shared read-write lock that can be cloned
#[derive(Debug, Clone)]
pub struct SharedRwLock<T>(Arc<DeterministicRwLock<T>>);

impl<T> SharedRwLock<T> {
    /// Create a new shared read-write lock
    pub fn new(data: T) -> Self {
        Self(Arc::new(DeterministicRwLock::new(data)))
    }
    
    /// Acquire a read lock
    pub fn read(&self) -> DeterministicReadGuard<'_, T> {
        self.0.read()
    }
    
    /// Try to acquire a read lock
    pub fn try_read(&self) -> Option<DeterministicReadGuard<'_, T>> {
        self.0.try_read()
    }
    
    /// Acquire a write lock
    pub fn write(&self) -> DeterministicWriteGuard<'_, T> {
        self.0.write()
    }
    
    /// Try to acquire a write lock
    pub fn try_write(&self) -> Option<DeterministicWriteGuard<'_, T>> {
        self.0.try_write()
    }
}

impl<T> Default for SharedRwLock<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A simple interior mutability cell (similar to RefCell but with unsafe operations)
///
/// Use only when you need interior mutability without thread safety.
/// For thread-safe interior mutability, use AtomicCell from the atomic module.
#[derive(Debug)]
pub struct Cell<T> {
    /// The inner value
    value: UnsafeCell<T>,
    /// Marker to prevent the struct from being Sync
    _not_sync: PhantomData<*const ()>,
}

// Instead of using negative trait bounds, we'll simply not implement Sync
// and add a marker to ensure the type is not Send + Sync
// This is safe because UnsafeCell<T> is not Sync

// impl<T> !Sync for Cell<T> {}
// PhantomData<*const ()> is !Sync, so this prevents the struct from being Sync
impl<T> Cell<T> {
    /// Create a new cell
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            _not_sync: PhantomData,
        }
    }
    
    /// Set the value in the cell
    pub fn set(&self, value: T) {
        unsafe {
            *self.value.get() = value;
        }
    }
    
    /// Get a copy of the value in the cell
    pub fn get(&self) -> T 
    where 
        T: Copy,
    {
        unsafe {
            *self.value.get()
        }
    }
    
    /// Replace the value in the cell and return the old value
    pub fn replace(&self, value: T) -> T {
        let old = self.get_clone();
        self.set(value);
        old
    }
    
    /// Get a mutable reference to the value in the cell
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
    
    /// Update the value in the cell using a function
    pub fn update<F>(&self, f: F) -> T
    where
        F: FnOnce(T) -> T,
        T: Copy,
    {
        let old = self.get();
        let new = f(old);
        self.set(new);
        new
    }
}

impl<T: Clone> Cell<T> {
    /// Get a clone of the value in the cell
    pub fn get_clone(&self) -> T {
        unsafe {
            (*self.value.get()).clone()
        }
    }
}

impl<T: Default> Default for Cell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Cell<T> 
where
    T: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cell")
            .field("value", &self.get_clone())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_mutex() {
        let mutex = DeterministicMutex::new(42);
        {
            let mut guard = mutex.lock();
            assert_eq!(*guard, 42);
            *guard = 84;
        }
        assert_eq!(*mutex.lock(), 84);
    }
    
    #[test]
    fn test_shared_mutex() {
        let mutex = SharedMutex::new(42);
        let mutex2 = mutex.clone();
        
        // Modify through one handle
        {
            let mut guard = mutex.lock();
            assert_eq!(*guard, 42);
            *guard = 84;
        }
        
        // Access through the other handle
        assert_eq!(*mutex2.lock(), 84);
    }
    
    #[test]
    fn test_deterministic_rwlock() {
        let rwlock = DeterministicRwLock::new(42);
        
        // Multiple readers
        {
            let guard1 = rwlock.read();
            let guard2 = rwlock.read();
            assert_eq!(*guard1, 42);
            assert_eq!(*guard2, 42);
        }
        
        // Single writer
        {
            let mut guard = rwlock.write();
            *guard = 84;
        }
        
        assert_eq!(*rwlock.read(), 84);
    }
    
    #[test]
    fn test_shared_rwlock() {
        let rwlock = SharedRwLock::new(42);
        let rwlock2 = rwlock.clone();
        
        // Modify through one handle
        {
            let mut guard = rwlock.write();
            *guard = 84;
        }
        
        // Access through the other handle
        assert_eq!(*rwlock2.read(), 84);
    }
    
    #[test]
    fn test_cell() {
        let cell = Cell::new(42);
        
        assert_eq!(cell.get(), 42);
        cell.set(84);
        assert_eq!(cell.get(), 84);
        
        let updated = cell.update(|v| v * 2);
        assert_eq!(updated, 168);
        assert_eq!(cell.get(), 168);
        
        let replaced = cell.replace(42);
        assert_eq!(replaced, 168);
        assert_eq!(cell.get(), 42);
    }
} 