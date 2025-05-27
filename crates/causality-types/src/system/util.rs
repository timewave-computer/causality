//! Utility System
//!
//! This module consolidates utility types and functions for the causality-types crate,
//! including trait interfaces, transformation utilities, time functions, and other
//! helper types that support the core framework functionality.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};

use crate::system::serialization::{Decode, Encode, SimpleSerialize, DecodeError};
use crate::system::provider::AsRegistry;

//-----------------------------------------------------------------------------
// Identifiable and Resolvable Traits
//-----------------------------------------------------------------------------

/// Trait for types that can be identified by an ID
pub trait AsIdentifiable {
    /// The ID type for this type
    type Id;

    /// Returns the ID for this instance
    fn id(&self) -> Self::Id;
    
    /// Compute the hash of this instance
    fn compute_hash(&self) -> [u8; 32] {
        // Default implementation - should be overridden by implementors
        [0u8; 32]
    }
}

/// Trait for types that can be resolved from their IDs
pub trait AsResolvable<T> {
    /// The ID type for resolving T
    type Id;

    /// Attempts to resolve an ID to an instance of T
    fn resolve(&self, id: &Self::Id) -> Option<T>;
}

//-----------------------------------------------------------------------------
// Data Transformation Utilities
//-----------------------------------------------------------------------------

/// Type alias for a simple transformation function between two types
pub type TransformFn<S, T> = Arc<dyn Fn(&S) -> Result<T> + Send + Sync>;

/// Create a simple transformation that always succeeds with the same output
pub fn constant_transform<S, T>(output: T) -> TransformFn<S, T>
where
    T: Clone + Send + Sync + 'static,
{
    Arc::new(move |_input| Ok(output.clone()))
}

/// Create an identity transformation
pub fn identity_transform<T>() -> TransformFn<T, T>
where
    T: Clone + Send + Sync + 'static,
{
    Arc::new(|input| Ok(input.clone()))
}

/// Create a transformation that applies a mapping function
pub fn mapping_transform<S, T, F>(mapper: F) -> TransformFn<S, T>
where
    F: Fn(&S) -> Result<T> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    Arc::new(mapper)
}

/// Create a transformation that applies validation before conversion
pub fn validated_transform<S, T, V, F>(validator: V, mapper: F) -> TransformFn<S, T>
where
    V: Fn(&S) -> Result<()> + Send + Sync + 'static,
    F: Fn(&S) -> Result<T> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    Arc::new(move |input| {
        validator(input)?;
        mapper(input)
    })
}

//-----------------------------------------------------------------------------
// Time Utilities
//-----------------------------------------------------------------------------

/// Get the current time in milliseconds since Unix epoch.
/// This function is intended for use by clock implementations that need
/// to access the real system time.
#[inline]
pub fn get_current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get the current time in microseconds since Unix epoch
#[inline]
pub fn get_current_time_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

/// Get the current time in nanoseconds since Unix epoch
#[inline]
pub fn get_current_time_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Convert a Duration to milliseconds
pub fn duration_to_ms(duration: Duration) -> u64 {
    duration.as_millis() as u64
}

/// Convert milliseconds to a Duration
pub fn ms_to_duration(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

/// Helper function to measure execution time
pub fn measure_time<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = SystemTime::now();
    let result = f();
    let duration = start.elapsed().unwrap_or_default();
    (result, duration)
}

//-----------------------------------------------------------------------------
// SSZ Duration Type
//-----------------------------------------------------------------------------

/// A wrapper around `std::time::Duration` to allow SSZ (de)serialization.
/// Serializes as (seconds: u64, nanos: u32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SszDuration {
    pub secs: u64,
    pub nanos: u32,
}

impl Encode for SszDuration {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.secs.to_le_bytes());
        bytes.extend_from_slice(&self.nanos.to_le_bytes());
        bytes
    }
}

impl Decode for SszDuration {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 12 { // 8 bytes for u64, 4 bytes for u32
            return Err(DecodeError { 
                message: format!("SszDuration bytes length must be 12, got {}", bytes.len()) 
            });
        }
        let mut secs_bytes = [0u8; 8];
        secs_bytes.copy_from_slice(&bytes[0..8]);
        let secs = u64::from_le_bytes(secs_bytes);

        let mut nanos_bytes = [0u8; 4];
        nanos_bytes.copy_from_slice(&bytes[8..12]);
        let nanos = u32::from_le_bytes(nanos_bytes);

        Ok(SszDuration { secs, nanos })
    }
}

impl SimpleSerialize for SszDuration {}

/// Custom Duration wrapper with additional functionality
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDuration(pub Duration);

impl CustomDuration {
    /// Create a new CustomDuration
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Get the inner Duration
    pub fn inner(&self) -> Duration {
        self.0
    }

    /// Convert to milliseconds
    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    /// Convert to microseconds
    pub fn as_micros(&self) -> u128 {
        self.0.as_micros()
    }

    /// Convert to nanoseconds
    pub fn as_nanos(&self) -> u128 {
        self.0.as_nanos()
    }

    /// Add another duration
    pub fn add(&self, other: Duration) -> Self {
        Self(self.0 + other)
    }

    /// Subtract another duration
    pub fn sub(&self, other: Duration) -> Option<Self> {
        self.0.checked_sub(other).map(Self)
    }
}

// Implement From traits for easy conversion
impl From<Duration> for SszDuration {
    fn from(duration: Duration) -> Self {
        SszDuration {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

impl From<SszDuration> for Duration {
    fn from(ssz_duration: SszDuration) -> Self {
        Duration::new(ssz_duration.secs, ssz_duration.nanos)
    }
}

impl From<Duration> for CustomDuration {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<CustomDuration> for Duration {
    fn from(custom: CustomDuration) -> Self {
        custom.0
    }
}

//-----------------------------------------------------------------------------
// Registry Utilities
//-----------------------------------------------------------------------------

/// Simple in-memory registry implementation
pub struct SimpleRegistry<T> {
    items: HashMap<String, T>,
}

impl<T> SimpleRegistry<T> {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Register an item with a key
    pub fn register(&mut self, key: String, item: T) -> Option<T> {
        self.items.insert(key, item)
    }

    /// Get an item by key
    pub fn get(&self, key: &str) -> Option<&T> {
        self.items.get(key)
    }

    /// Get a mutable reference to an item by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.items.get_mut(key)
    }

    /// Remove an item by key
    pub fn remove(&mut self, key: &str) -> Option<T> {
        self.items.remove(key)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.items.keys()
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.items.values()
    }

    /// Get the number of registered items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<T> Default for SimpleRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Collection Utilities
//-----------------------------------------------------------------------------

/// Utility functions for working with collections
pub mod collection {
    use super::*;

    /// Check if two slices have any common elements
    pub fn has_intersection<T>(a: &[T], b: &[T]) -> bool 
    where
        T: PartialEq,
    {
        a.iter().any(|item_a| b.contains(item_a))
    }

    /// Find the intersection of two slices
    pub fn intersection<'a, T>(a: &'a [T], b: &'a [T]) -> Vec<&'a T>
    where
        T: PartialEq,
    {
        a.iter().filter(|item_a| b.contains(item_a)).collect()
    }

    /// Find the union of two slices (removing duplicates)
    pub fn union<'a, T>(a: &'a [T], b: &'a [T]) -> Vec<&'a T>
    where
        T: PartialEq,
    {
        let mut result: Vec<&'a T> = a.iter().collect();
        for item in b {
            if !a.contains(item) {
                result.push(item);
            }
        }
        result
    }

    /// Find elements in the first slice that are not in the second
    pub fn difference<'a, T>(a: &'a [T], b: &'a [T]) -> Vec<&'a T>
    where
        T: PartialEq,
    {
        a.iter().filter(|item_a| !b.contains(item_a)).collect()
    }

    /// Group items by a key function
    pub fn group_by<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<&T>>
    where
        K: Hash + Eq,
        F: Fn(&T) -> K,
    {
        let mut groups = HashMap::new();
        for item in items {
            let key = key_fn(item);
            groups.entry(key).or_insert_with(Vec::new).push(item);
        }
        groups
    }

    /// Partition items into two groups based on a predicate
    pub fn partition<T, F>(items: &[T], predicate: F) -> (Vec<&T>, Vec<&T>)
    where
        F: Fn(&T) -> bool,
    {
        let mut true_items = Vec::new();
        let mut false_items = Vec::new();
        
        for item in items {
            if predicate(item) {
                true_items.push(item);
            } else {
                false_items.push(item);
            }
        }
        
        (true_items, false_items)
    }
}

//-----------------------------------------------------------------------------
// Error Utilities
//-----------------------------------------------------------------------------

/// Helper functions for error handling
pub mod error {
    use super::*;

    /// Convert any error into a string description
    pub fn error_to_string<E: std::error::Error>(error: &E) -> String {
        format!("{}", error)
    }

    /// Create a chain of error descriptions including the source chain
    pub fn error_chain_string<E: std::error::Error>(error: &E) -> String {
        let mut chain = vec![error.to_string()];
        let mut source = error.source();
        
        while let Some(err) = source {
            chain.push(err.to_string());
            source = err.source();
        }
        
        chain.join(" -> ")
    }

    /// Map one error type to another with context
    pub fn map_error<E1, E2, F>(result: Result<(), E1>, mapper: F) -> Result<(), E2>
    where
        F: FnOnce(E1) -> E2,
    {
        result.map_err(mapper)
    }

    /// Add context to an error
    pub fn with_context<T, E>(result: Result<T, E>, context: &str) -> Result<T, anyhow::Error>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        result.with_context(|| context.to_string())
    }
}

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Re-export core trait implementations
pub use crate::primitive::trait_::*;

// Re-export time-related utilities
pub use crate::primitive::time::{AsClock, AsTimestampGenerator};

// Re-export transformation utilities
pub use transform::*;

/// Legacy transformation module for backward compatibility
pub mod transform {
    pub use super::{TransformFn, constant_transform, identity_transform, mapping_transform, validated_transform};
}

/// Time utilities module
pub mod time_utils {
    pub use super::{
        get_current_time_ms, get_current_time_us, get_current_time_ns,
        duration_to_ms, ms_to_duration, measure_time,
    };
}

/// Registry utilities module
pub mod registry {
    pub use super::SimpleRegistry;
    pub use crate::system::provider::{AsRegistry, MemoryRegistry};
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_transformations() {
        // Test constant transform
        let const_transform = constant_transform::<i32, String>("hello".to_string());
        assert_eq!(const_transform(&42).unwrap(), "hello");
        assert_eq!(const_transform(&0).unwrap(), "hello");

        // Test identity transform
        let id_transform = identity_transform::<i32>();
        assert_eq!(id_transform(&42).unwrap(), 42);
        assert_eq!(id_transform(&0).unwrap(), 0);

        // Test mapping transform
        let map_transform = mapping_transform(|x: &i32| Ok(x * 2));
        assert_eq!(map_transform(&21).unwrap(), 42);
        assert_eq!(map_transform(&0).unwrap(), 0);
    }

    #[test]
    fn test_time_utilities() {
        let time_ms = get_current_time_ms();
        assert!(time_ms > 0);

        let time_us = get_current_time_us();
        assert!(time_us > time_ms * 1000);

        let duration = Duration::from_millis(100);
        assert_eq!(duration_to_ms(duration), 100);
        assert_eq!(ms_to_duration(100), duration);
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_ssz_duration() {
        let duration = Duration::from_secs(5);
        let ssz_duration = SszDuration::from(duration);
        let recovered_duration = Duration::from(ssz_duration);
        assert_eq!(duration, recovered_duration);

        // Test serialization roundtrip
        let serialized = ssz_duration.as_ssz_bytes();
        let deserialized = SszDuration::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(ssz_duration, deserialized);
    }

    #[test]
    fn test_simple_registry() {
        let mut registry = SimpleRegistry::new();
        
        // Test registration
        registry.register("key1".to_string(), "value1");
        registry.register("key2".to_string(), "value2");
        
        // Test retrieval
        assert_eq!(registry.get("key1"), Some(&"value1"));
        assert_eq!(registry.get("key2"), Some(&"value2"));
        assert_eq!(registry.get("nonexistent"), None);
        
        // Test contains
        assert!(registry.contains_key("key1"));
        assert!(!registry.contains_key("nonexistent"));
        
        // Test length
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        
        // Test removal
        let removed = registry.remove("key1");
        assert_eq!(removed, Some("value1"));
        assert_eq!(registry.len(), 1);
        
        // Test clear
        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_collection_utilities() {
        use collection::*;
        
        let a = vec![1, 2, 3, 4];
        let b = vec![3, 4, 5, 6];
        
        // Test intersection
        assert!(has_intersection(&a, &b));
        let intersect = intersection(&a, &b);
        assert_eq!(intersect, vec![&3, &4]);
        
        // Test union
        let union_result = union(&a, &b);
        assert_eq!(union_result.len(), 6); // 1,2,3,4,5,6
        
        // Test difference
        let diff = difference(&a, &b);
        assert_eq!(diff, vec![&1, &2]);
        
        // Test partition
        let (evens, odds) = partition(&a, |x| *x % 2 == 0);
        assert_eq!(evens, vec![&2, &4]);
        assert_eq!(odds, vec![&1, &3]);
    }

    #[test]
    fn test_error_utilities() {
        use error::*;
        
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error_string = error_to_string(&error);
        assert!(error_string.contains("file not found"));
        
        let chain_string = error_chain_string(&error);
        assert!(chain_string.contains("file not found"));
    }
} 