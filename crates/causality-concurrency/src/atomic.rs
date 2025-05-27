// Atomic operations module
//
// This module provides atomic primitives and operations for thread-safe memory access.
// It simplifies the use of atomic types from the standard library and provides
// higher-level abstractions.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// A thread-safe counter that can be incremented atomically
#[derive(Debug)]
pub struct AtomicCounter {
    /// The underlying atomic value
    counter: AtomicU64,
}

impl AtomicCounter {
    /// Create a new counter with an initial value
    pub fn new(initial: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial),
        }
    }
    
    /// Get the current value
    pub fn get(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
    
    /// Set the counter to a specific value
    pub fn set(&self, value: u64) {
        self.counter.store(value, Ordering::SeqCst);
    }
    
    /// Increment the counter and get the new value
    pub fn increment(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    
    /// Decrement the counter and get the new value
    pub fn decrement(&self) -> u64 {
        self.counter.fetch_sub(1, Ordering::SeqCst) - 1
    }
    
    /// Atomically update the counter using a function
    pub fn update<F>(&self, f: F) -> u64
    where
        F: Fn(u64) -> u64,
    {
        let mut current = self.get();
        loop {
            let new_value = f(current);
            match self.counter.compare_exchange(
                current,
                new_value,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return new_value,
                Err(actual) => current = actual,
            }
        }
    }
}

/// A shared atomic counter that can be cloned
#[derive(Debug, Clone)]
pub struct SharedCounter(Arc<AtomicCounter>);

impl SharedCounter {
    /// Create a new shared counter with an initial value
    pub fn new(initial: u64) -> Self {
        Self(Arc::new(AtomicCounter::new(initial)))
    }
    
    /// Get the current value
    pub fn get(&self) -> u64 {
        self.0.get()
    }
    
    /// Set the counter to a specific value
    pub fn set(&self, value: u64) {
        self.0.set(value);
    }
    
    /// Increment the counter and get the new value
    pub fn increment(&self) -> u64 {
        self.0.increment()
    }
    
    /// Decrement the counter and get the new value
    pub fn decrement(&self) -> u64 {
        self.0.decrement()
    }
    
    /// Atomically update the counter using a function
    pub fn update<F>(&self, f: F) -> u64
    where
        F: Fn(u64) -> u64,
    {
        self.0.update(f)
    }
}

/// A boolean flag that can be set or cleared atomically
#[derive(Debug)]
pub struct AtomicFlag {
    /// The underlying atomic boolean
    flag: AtomicBool,
}

impl AtomicFlag {
    /// Create a new flag with an initial value
    pub fn new(initial: bool) -> Self {
        Self {
            flag: AtomicBool::new(initial),
        }
    }
    
    /// Get the current value
    pub fn get(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
    
    /// Set the flag to a specific value
    pub fn set(&self, value: bool) {
        self.flag.store(value, Ordering::SeqCst);
    }
    
    /// Set the flag to true and return the previous value
    pub fn set_true(&self) -> bool {
        self.flag.swap(true, Ordering::SeqCst)
    }
    
    /// Set the flag to false and return the previous value
    pub fn set_false(&self) -> bool {
        self.flag.swap(false, Ordering::SeqCst)
    }
    
    /// Toggle the flag and return the new value
    pub fn toggle(&self) -> bool {
        !self.flag.fetch_xor(true, Ordering::SeqCst)
    }
    
    /// Compare and set the flag
    pub fn compare_and_set(&self, expected: bool, new: bool) -> bool {
        self.flag
            .compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

/// A shared atomic flag that can be cloned
#[derive(Debug, Clone)]
pub struct SharedFlag(Arc<AtomicFlag>);

impl SharedFlag {
    /// Create a new shared flag with an initial value
    pub fn new(initial: bool) -> Self {
        Self(Arc::new(AtomicFlag::new(initial)))
    }
    
    /// Get the current value
    pub fn get(&self) -> bool {
        self.0.get()
    }
    
    /// Set the flag to a specific value
    pub fn set(&self, value: bool) {
        self.0.set(value);
    }
    
    /// Set the flag to true and return the previous value
    pub fn set_true(&self) -> bool {
        self.0.set_true()
    }
    
    /// Set the flag to false and return the previous value
    pub fn set_false(&self) -> bool {
        self.0.set_false()
    }
    
    /// Toggle the flag and return the new value
    pub fn toggle(&self) -> bool {
        self.0.toggle()
    }
    
    /// Compare and set the flag
    pub fn compare_and_set(&self, expected: bool, new: bool) -> bool {
        self.0.compare_and_set(expected, new)
    }
}

/// A generic thread-safe cell for any value
#[derive(Debug)]
pub struct AtomicCell<T> {
    /// The underlying value protected by a read-write lock
    value: RwLock<T>,
}

impl<T: Clone> AtomicCell<T> {
    /// Create a new cell with an initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: RwLock::new(initial),
        }
    }
    
    /// Get a clone of the current value
    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }
    
    /// Set the cell to a specific value
    pub fn set(&self, value: T) {
        let mut guard = self.value.write().unwrap();
        *guard = value;
    }
    
    /// Update the cell using a function and return the new value
    pub fn update<F>(&self, f: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        let mut guard = self.value.write().unwrap();
        let new_value = f(guard.clone());
        *guard = new_value.clone();
        new_value
    }
    
    /// Read the cell using a function without modifying it
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.value.read().unwrap();
        f(&*guard)
    }
    
    /// Modify the cell using a function and return a result
    pub fn modify<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.value.write().unwrap();
        f(&mut *guard)
    }
}

/// A shared atomic cell that can be cloned
#[derive(Debug, Clone)]
pub struct SharedCell<T: Clone>(Arc<AtomicCell<T>>);

impl<T: Clone> SharedCell<T> {
    /// Create a new shared cell with an initial value
    pub fn new(initial: T) -> Self {
        Self(Arc::new(AtomicCell::new(initial)))
    }
    
    /// Get a clone of the current value
    pub fn get(&self) -> T {
        self.0.get()
    }
    
    /// Set the cell to a specific value
    pub fn set(&self, value: T) {
        self.0.set(value);
    }
    
    /// Update the cell using a function and return the new value
    pub fn update<F>(&self, f: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        self.0.update(f)
    }
    
    /// Read the cell using a function without modifying it
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.0.read(f)
    }
    
    /// Modify the cell using a function and return a result
    pub fn modify<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.0.modify(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new(0);
        assert_eq!(counter.get(), 0);
        
        counter.increment();
        assert_eq!(counter.get(), 1);
        
        counter.decrement();
        assert_eq!(counter.get(), 0);
        
        counter.set(42);
        assert_eq!(counter.get(), 42);
        
        let new_value = counter.update(|v| v * 2);
        assert_eq!(new_value, 84);
        assert_eq!(counter.get(), 84);
    }
    
    #[test]
    fn test_shared_counter() {
        let counter = SharedCounter::new(0);
        
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        
        let t1 = thread::spawn(move || {
            for _ in 0..100 {
                counter1.increment();
            }
        });
        
        let t2 = thread::spawn(move || {
            for _ in 0..100 {
                counter2.increment();
            }
        });
        
        t1.join().unwrap();
        t2.join().unwrap();
        
        assert_eq!(counter.get(), 200);
    }
    
    #[test]
    fn test_atomic_flag() {
        let flag = AtomicFlag::new(false);
        assert_eq!(flag.get(), false);
        
        flag.set(true);
        assert_eq!(flag.get(), true);
        
        let prev = flag.set_false();
        assert_eq!(prev, true);
        assert_eq!(flag.get(), false);
        
        let toggled = flag.toggle();
        assert_eq!(toggled, true);
        assert_eq!(flag.get(), true);
    }
    
    #[test]
    fn test_atomic_cell() {
        let cell = AtomicCell::new(String::from("hello"));
        assert_eq!(cell.get(), "hello");
        
        cell.set(String::from("world"));
        assert_eq!(cell.get(), "world");
        
        let new_value = cell.update(|s| format!("{}!", s));
        assert_eq!(new_value, "world!");
        assert_eq!(cell.get(), "world!");
        
        let length = cell.read(|s| s.len());
        assert_eq!(length, 6);
        
        cell.modify(|s| s.push_str(" test"));
        assert_eq!(cell.get(), "world! test");
    }
    
    #[test]
    fn test_shared_cell() {
        let cell = SharedCell::new(Vec::<i32>::new());
        
        let cell1 = cell.clone();
        let cell2 = cell.clone();
        
        let t1 = thread::spawn(move || {
            for i in 0..10 {
                cell1.modify(|v| v.push(i));
            }
        });
        
        let t2 = thread::spawn(move || {
            for i in 10..20 {
                cell2.modify(|v| v.push(i));
            }
        });
        
        t1.join().unwrap();
        t2.join().unwrap();
        
        let result = cell.get();
        assert_eq!(result.len(), 20);
    }
} 