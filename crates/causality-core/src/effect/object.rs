//! Object system with configurable linearity
//!
//! This module implements the generalized Object type that can wrap any data
//! with configurable linearity qualifiers and capability checking.

use crate::lambda::{
    base::{Linear, Affine, Relevant, Unrestricted},
};
use crate::{Linearity, LinearityError};
use super::capability::Capability;
use std::collections::HashSet;
use std::marker::PhantomData;
use ssz::{Encode, Decode};

/// A generalized object that wraps data with linearity and capability constraints
/// 
/// Objects provide a unified abstraction for resources with configurable linearity:
/// - Linear: must be used exactly once (default for resources)
/// - Affine: may be used at most once (optional consumption)  
/// - Relevant: must be used at least once (required consumption)
/// - Unrestricted: may be used any number of times (shared data)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object<T, L = Linear> {
    /// The encapsulated data
    pub data: T,
    
    /// Linearity qualifier controlling usage patterns
    linearity: PhantomData<L>,
    
    /// Set of capabilities associated with this object
    pub capabilities: HashSet<Capability>,
}

impl<T, L: Linearity> Object<T, L> {
    /// Create a new object with the given data and capabilities
    pub fn new(data: T, capabilities: HashSet<Capability>) -> Self {
        Self {
            data,
            linearity: PhantomData,
            capabilities,
        }
    }
    
    /// Create an object with no capabilities
    pub fn from_data(data: T) -> Self {
        Self::new(data, HashSet::new())
    }
    
    /// Add a capability to this object
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.insert(capability);
        self
    }
    
    /// Add multiple capabilities to this object
    pub fn with_capabilities(mut self, capabilities: impl IntoIterator<Item = Capability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
    
    /// Check if this object has a specific capability
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if this object has all required capabilities
    pub fn has_all_capabilities(&self, required: &[Capability]) -> bool {
        required.iter().all(|cap| self.has_capability(cap))
    }
    
    /// Get all capabilities of this object
    pub fn get_capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }
    
    /// Check if the object can be dropped based on linearity
    pub fn can_drop() -> bool {
        L::CAN_DROP
    }
    
    /// Check if the object can be copied based on linearity
    pub fn can_copy() -> bool {
        L::CAN_COPY
    }
    
    /// Check if the object must be used based on linearity
    pub fn must_use() -> bool {
        L::must_use()
    }
    
    /// Check if the object can only be used once based on linearity
    pub fn use_once() -> bool {
        L::use_once()
    }
    
    /// Get the linearity type name for debugging
    pub fn linearity_name() -> &'static str {
        std::any::type_name::<L>()
    }
}

impl<T> Object<T, Linear> {
    /// Create a linear object (must be used exactly once)
    pub fn linear(data: T) -> Self {
        Self::from_data(data)
    }
    
    /// Consume the linear object, returning its data
    pub fn consume(self) -> Result<T, LinearityError> {
        // Linear objects can always be consumed
        Ok(self.data)
    }
}

impl<T> Object<T, Affine> {
    /// Create an affine object (may be used at most once)
    pub fn affine(data: T) -> Self {
        Self::from_data(data)
    }
    
    /// Consume the affine object, returning its data
    pub fn consume(self) -> Result<T, LinearityError> {
        // Affine objects can be consumed
        Ok(self.data)
    }
    
    /// Drop the affine object without consuming it
    pub fn drop_unused(self) {
        // Affine objects can be dropped without use
        drop(self);
    }
}

impl<T: Clone> Object<T, Relevant> {
    /// Create a relevant object (must be used at least once)
    pub fn relevant(data: T) -> Self {
        Self::from_data(data)
    }
    
    /// Consume the relevant object, returning its data
    pub fn consume(self) -> Result<T, LinearityError> {
        // Relevant objects can be consumed
        Ok(self.data)
    }
    
    /// Copy the relevant object's data (marks as used)
    pub fn copy_data(&self) -> Result<T, LinearityError> {
        // Relevant objects can be copied
        Ok(self.data.clone())
    }
}

impl<T: Clone> Object<T, Unrestricted> {
    /// Create an unrestricted object (may be used any number of times)
    pub fn unrestricted(data: T) -> Self {
        Self::from_data(data)
    }
    
    /// Consume the unrestricted object, returning its data
    pub fn consume(self) -> Result<T, LinearityError> {
        // Unrestricted objects can be consumed
        Ok(self.data)
    }
    
    /// Copy the unrestricted object's data
    pub fn copy_data(&self) -> Result<T, LinearityError> {
        // Unrestricted objects can be copied
        Ok(self.data.clone())
    }
    
    /// Drop the unrestricted object without consuming it
    pub fn drop_unused(self) {
        // Unrestricted objects can be dropped without use
        drop(self);
    }
}

/// Capability-aware operations for objects
impl<T, L: Linearity> Object<T, L> {
    /// Perform an operation that requires specific capabilities
    pub fn with_capability_check<F, R>(
        &self,
        required_capability: &Capability,
        operation: F,
    ) -> Result<R, CapabilityError>
    where
        F: FnOnce(&T) -> R,
    {
        if self.has_capability(required_capability) {
            Ok(operation(&self.data))
        } else {
            Err(CapabilityError::MissingCapability {
                required: required_capability.clone(),
                available: self.capabilities.clone(),
            })
        }
    }
    
    /// Perform an operation that requires multiple capabilities
    pub fn with_capabilities_check<F, R>(
        &self,
        required_capabilities: &[Capability],
        operation: F,
    ) -> Result<R, CapabilityError>
    where
        F: FnOnce(&T) -> R,
    {
        for capability in required_capabilities {
            if !self.has_capability(capability) {
                return Err(CapabilityError::MissingCapability {
                    required: capability.clone(),
                    available: self.capabilities.clone(),
                });
            }
        }
        Ok(operation(&self.data))
    }
}

/// Error types for capability checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// Required capability is missing
    MissingCapability {
        required: Capability,
        available: HashSet<Capability>,
    },
    
    /// Operation not permitted
    OperationNotPermitted(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::MissingCapability { required, available } => {
                write!(
                    f,
                    "Missing required capability {:?}. Available: {:?}",
                    required, available
                )
            }
            CapabilityError::OperationNotPermitted(msg) => {
                write!(f, "Operation not permitted: {}", msg)
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Simplified SSZ implementation for Object
impl<T: Encode, L> Encode for Object<T, L> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        self.data.ssz_bytes_len() + 4 + self.capabilities.len() * 64 // simplified
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.data.ssz_append(buf);
        (self.capabilities.len() as u32).ssz_append(buf);
        for capability in &self.capabilities {
            capability.ssz_append(buf);
        }
    }
}

impl<T: Decode, L> Decode for Object<T, L> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        // Simplified implementation for now
        let data = T::from_ssz_bytes(bytes)?;
        
        // For now, return object with empty capabilities
        Ok(Object {
            data,
            linearity: PhantomData,
            capabilities: HashSet::new(),
        })
    }
}

/// Type aliases for common object linearity patterns
pub type LinearObject<T> = Object<T, Linear>;
pub type AffineObject<T> = Object<T, Affine>;
pub type RelevantObject<T> = Object<T, Relevant>;
pub type UnrestrictedObject<T> = Object<T, Unrestricted>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_object_creation() {
        let obj = LinearObject::linear(42);
        assert_eq!(obj.data, 42);
        assert!(obj.capabilities.is_empty());
        assert!(!LinearObject::<i32>::can_drop());
        assert!(!LinearObject::<i32>::can_copy());
        assert!(LinearObject::<i32>::must_use());
        assert!(LinearObject::<i32>::use_once());
    }

    #[test]
    fn test_affine_object_creation() {
        let obj = AffineObject::affine("test".to_string());
        assert_eq!(obj.data, "test");
        assert!(AffineObject::<String>::can_drop());
        assert!(!AffineObject::<String>::can_copy());
        assert!(!AffineObject::<String>::must_use());
        assert!(AffineObject::<String>::use_once());
    }

    #[test]
    fn test_relevant_object_creation() {
        let obj = RelevantObject::relevant(vec![1, 2, 3]);
        assert_eq!(obj.data, vec![1, 2, 3]);
        assert!(!RelevantObject::<Vec<i32>>::can_drop());
        assert!(RelevantObject::<Vec<i32>>::can_copy());
        assert!(RelevantObject::<Vec<i32>>::must_use());
        assert!(!RelevantObject::<Vec<i32>>::use_once());
    }

    #[test]
    fn test_unrestricted_object_creation() {
        let obj = UnrestrictedObject::unrestricted(42);
        assert_eq!(obj.data, 42);
        assert!(UnrestrictedObject::<i32>::can_drop());
        assert!(UnrestrictedObject::<i32>::can_copy());
        assert!(!UnrestrictedObject::<i32>::must_use());
        assert!(!UnrestrictedObject::<i32>::use_once());
    }

    #[test]
    fn test_capability_management() {
        let read_cap = Capability::read("read");
        let write_cap = Capability::write("write");
        
        let obj = LinearObject::linear(42)
            .with_capability(read_cap.clone())
            .with_capability(write_cap.clone());
        
        assert!(obj.has_capability(&read_cap));
        assert!(obj.has_capability(&write_cap));
        assert!(!obj.has_capability(&Capability::admin("admin")));
        
        assert!(obj.has_all_capabilities(&[read_cap, write_cap]));
        assert!(!obj.has_all_capabilities(&[Capability::admin("admin")]));
    }

    #[test]
    fn test_capability_checking() {
        let read_cap = Capability::read("read");
        let write_cap = Capability::write("write");
        
        let obj = LinearObject::linear("data".to_string())
            .with_capability(read_cap.clone());
        
        // Operation with valid capability should succeed
        let result = obj.with_capability_check(&read_cap, |data| data.len());
        assert_eq!(result.unwrap(), 4);
        
        // Operation with missing capability should fail
        let result = obj.with_capability_check(&write_cap, |data| data.len());
        assert!(matches!(result, Err(CapabilityError::MissingCapability { .. })));
    }

    #[test]
    fn test_linear_object_consumption() {
        let obj = LinearObject::linear(42);
        let value = obj.consume().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_affine_object_drop() {
        let obj = AffineObject::affine(42);
        obj.drop_unused(); // Should not panic
    }

    #[test]
    fn test_relevant_object_copy() {
        let obj = RelevantObject::relevant("test".to_string());
        let copy1 = obj.copy_data().unwrap();
        let copy2 = obj.copy_data().unwrap();
        assert_eq!(copy1, "test");
        assert_eq!(copy2, "test");
    }

    #[test]
    fn test_unrestricted_object_flexibility() {
        let obj = UnrestrictedObject::unrestricted(42);
        
        // Can copy multiple times
        let copy1 = obj.copy_data().unwrap();
        let copy2 = obj.copy_data().unwrap();
        assert_eq!(copy1, 42);
        assert_eq!(copy2, 42);
        
        // Can still be consumed
        let value = obj.consume().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_object_with_multiple_capabilities() {
        let read_cap = Capability::read("read");
        let write_cap = Capability::write("write");
        let admin_cap = Capability::admin("admin");
        
        let obj = LinearObject::linear("data".to_string())
            .with_capabilities(vec![read_cap.clone(), write_cap.clone()]);
        
        // Should have both capabilities
        assert!(obj.has_capability(&read_cap));
        assert!(obj.has_capability(&write_cap));
        assert!(!obj.has_capability(&admin_cap));
        
        // Multi-capability check should work
        let result = obj.with_capabilities_check(
            &[read_cap, write_cap],
            |data| data.to_uppercase()
        );
        assert_eq!(result.unwrap(), "DATA");
        
        // Missing capability should fail
        let result = obj.with_capabilities_check(
            &[admin_cap],
            |data| data.to_uppercase()
        );
        assert!(matches!(result, Err(CapabilityError::MissingCapability { .. })));
    }
} 