//! Linear type system and linearity tracking
//!
//! This module implements the linear type system, ensuring resources
//! are used exactly once (linear), at most once (affine), or any number
//! of times (unrestricted).

use std::marker::PhantomData;
use std::cell::Cell;
use crate::system::error::LinearityError;
use super::base::{Linear, Affine, Relevant, Unrestricted};

//-----------------------------------------------------------------------------
// Linearity Traits
//-----------------------------------------------------------------------------

/// Marker trait for types with linearity constraints
pub trait Linearity: Sized + 'static {
    /// Whether this linearity allows dropping without use
    const CAN_DROP: bool;
    
    /// Whether this linearity allows multiple uses
    const CAN_COPY: bool;
    
    /// Derived: Whether this linearity requires at least one use
    fn must_use() -> bool {
        !Self::CAN_DROP  // CAN_DROP=false == must_use=true
    }
    
    /// Derived: Whether this linearity requires exactly one use
    fn use_once() -> bool {
        !Self::CAN_COPY  // CAN_COPY=false == use_once=true
    }
}

/// Linear types must be used exactly once
impl Linearity for Linear {
    const CAN_DROP: bool = false;
    const CAN_COPY: bool = false;
}

/// Affine types can be used at most once
impl Linearity for Affine {
    const CAN_DROP: bool = true;
    const CAN_COPY: bool = false;
}

/// Relevant types must be used at least once
impl Linearity for Relevant {
    const CAN_DROP: bool = false;
    const CAN_COPY: bool = true;
}

/// Unrestricted types can be used any number of times
impl Linearity for Unrestricted {
    const CAN_DROP: bool = true;
    const CAN_COPY: bool = true;
}

//-----------------------------------------------------------------------------
// Linear Resource Management
//-----------------------------------------------------------------------------

/// A linear resource that must be consumed exactly once
#[derive(Debug)]
pub struct LinearResource<T, L: Linearity = Linear> {
    value: Option<T>,
    /// Tracks whether the resource has been used at least once
    used: Cell<bool>,
    _phantom: PhantomData<L>,
}

impl<T, L: Linearity> LinearResource<T, L> {
    /// Create a new linear resource
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            used: Cell::new(false),
            _phantom: PhantomData,
        }
    }
    
    /// Consume the resource, returning its value
    pub fn consume(mut self) -> Result<T, LinearityError> {
        self.used.set(true);
        self.value.take()
            .ok_or(LinearityError::MultipleUse)
    }
    
    /// Try to consume the resource without moving self
    pub fn try_consume(&mut self) -> Result<T, LinearityError> {
        self.used.set(true);
        self.value.take()
            .ok_or(LinearityError::MultipleUse)
    }
    
    /// Check if the resource has been consumed
    pub fn is_consumed(&self) -> bool {
        self.value.is_none()
    }
    
    /// Check if the resource has been used at least once
    pub fn has_been_used(&self) -> bool {
        self.used.get()
    }
}

impl<T, L> Drop for LinearResource<T, L> 
where
    L: Linearity,
{
    fn drop(&mut self) {
        if self.value.is_some() && !self.used.get() {
            // Resource hasn't been consumed or used, check linearity rules
            if L::must_use() && L::CAN_COPY {
                // Relevant: must be used at least once (can copy)
                panic!("Relevant resource dropped without being used");
            } else if !L::CAN_DROP {
                // Linear: must be consumed exactly once (cannot drop)
                panic!("Linear resource dropped without being consumed");
            }
            // Affine and Unrestricted: can be dropped without use
        }
    }
}

impl<T> LinearResource<T, Affine> {
    /// Drop an affine resource without consuming it
    pub fn drop_resource(self) {
        // Affine resources can be dropped
        drop(self);
    }
}

impl<T: Clone> LinearResource<T, Unrestricted> {
    /// Copy an unrestricted resource
    pub fn copy_resource(&self) -> Result<T, LinearityError> {
        self.used.set(true);
        self.value.as_ref()
            .cloned()
            .ok_or(LinearityError::UseAfterDrop)
    }
}

impl<T: Clone> LinearResource<T, Relevant> {
    /// Copy a relevant resource (marks as used)
    pub fn copy_resource(&self) -> Result<T, LinearityError> {
        self.used.set(true);
        self.value.as_ref()
            .cloned()
            .ok_or(LinearityError::UseAfterDrop)
    }
}

//-----------------------------------------------------------------------------
// Linearity Checking
//-----------------------------------------------------------------------------

/// Operations supported based on the 2x2 matrix of CAN_DROP × CAN_COPY
/// 
/// |                | CAN_DROP=false       | CAN_DROP=true       |
/// |----------------|----------------------|---------------------|
/// | CAN_COPY=false | Linear: consume once | Affine: consume≤1   |
/// | CAN_COPY=true  | Relevant: use≥1      | Unrestricted: any   |
///
/// Semantic aliases:
/// - must_use() = !CAN_DROP (Linear & Relevant)
/// - use_once() = !CAN_COPY (Linear & Affine)

/// Trait for single-use consumption (Linear and Affine)
pub trait SingleUse: Sized {
    /// Consume the value exactly once
    fn consume(self) -> Result<(), LinearityError>;
}

/// Trait for droppable types (Affine and Unrestricted)
pub trait Droppable: Sized {
    /// Explicitly drop the value without consuming
    fn drop_unused(self) -> Result<(), LinearityError>;
}

/// Trait for copyable types (Relevant and Unrestricted)
pub trait Copyable {
    /// The inner type that can be cloned
    type Item: Clone;
    
    /// Copy the value (marks as used for Relevant types)
    fn copy_value(&self) -> Result<Self::Item, LinearityError>;
}

/// Trait for types that must be used (Linear and Relevant)
pub trait MustUse: Sized {
    /// Check if the value has been used
    fn check_used(&self) -> bool;
    
    /// Ensure the value will be used before drop
    fn ensure_will_be_used(&self) -> Result<(), LinearityError>;
}

/// Implementation for LinearResource based on linearity
impl<T, L: Linearity> SingleUse for LinearResource<T, L> {
    fn consume(self) -> Result<(), LinearityError> {
        self.consume()?;
        Ok(())
    }
}

impl<T> Droppable for LinearResource<T, Affine> {
    fn drop_unused(self) -> Result<(), LinearityError> {
        // Affine can be dropped without use
        drop(self);
        Ok(())
    }
}

impl<T> Droppable for LinearResource<T, Unrestricted> {
    fn drop_unused(self) -> Result<(), LinearityError> {
        // Unrestricted can be dropped without use
        drop(self);
        Ok(())
    }
}

impl<T: Clone> Copyable for LinearResource<T, Relevant> {
    type Item = T;
    
    fn copy_value(&self) -> Result<Self::Item, LinearityError> {
        self.copy_resource()
    }
}

impl<T: Clone> Copyable for LinearResource<T, Unrestricted> {
    type Item = T;
    
    fn copy_value(&self) -> Result<Self::Item, LinearityError> {
        self.copy_resource()
    }
}

impl<T> MustUse for LinearResource<T, Linear> {
    fn check_used(&self) -> bool {
        self.has_been_used()
    }
    
    fn ensure_will_be_used(&self) -> Result<(), LinearityError> {
        if !self.has_been_used() && self.value.is_some() {
            Err(LinearityError::NotUsed)
        } else {
            Ok(())
        }
    }
}

impl<T> MustUse for LinearResource<T, Relevant> {
    fn check_used(&self) -> bool {
        self.has_been_used()
    }
    
    fn ensure_will_be_used(&self) -> Result<(), LinearityError> {
        if !self.has_been_used() && self.value.is_some() {
            Err(LinearityError::NotUsed)
        } else {
            Ok(())
        }
    }
}

/// Helper trait to check linearity at compile time
pub trait LinearityCheck<L: Linearity> {
    /// Check if this type can be dropped
    fn can_drop() -> bool { L::CAN_DROP }
    
    /// Check if this type can be copied
    fn can_copy() -> bool { L::CAN_COPY }
    
    /// Check if this type must be used (alias for !can_drop)
    fn must_use() -> bool { L::must_use() }
    
    /// Check if this type can only be used once (alias for !can_copy)
    fn use_once() -> bool { L::use_once() }
}

impl<T, L: Linearity> LinearityCheck<L> for LinearResource<T, L> {}

//-----------------------------------------------------------------------------
// Linearity Combinators
//-----------------------------------------------------------------------------

/// Split a linear pair into its components
pub fn split_linear_pair<A, B>(pair: LinearResource<(A, B)>) -> Result<(LinearResource<A>, LinearResource<B>), LinearityError> {
    let (a, b) = pair.consume()?;
    Ok((LinearResource::new(a), LinearResource::new(b)))
}

/// Combine two linear resources into a pair
pub fn join_linear_pair<A, B>(a: LinearResource<A>, b: LinearResource<B>) -> Result<LinearResource<(A, B)>, LinearityError> {
    let a_val = a.consume()?;
    let b_val = b.consume()?;
    Ok(LinearResource::new((a_val, b_val)))
}

/// Apply a linear function to a linear argument
pub fn apply_linear<A, B, F>(f: LinearResource<F>, arg: LinearResource<A>) -> Result<LinearResource<B>, LinearityError>
where
    F: FnOnce(A) -> B,
{
    let f_val = f.consume()?;
    let arg_val = arg.consume()?;
    Ok(LinearResource::new(f_val(arg_val)))
} 