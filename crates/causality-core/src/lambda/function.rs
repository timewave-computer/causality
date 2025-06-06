//! Linear function types (A ⊸ B) for Layer 1
//!
//! This module provides the linear function type for the linear lambda calculus.
//! Linear functions consume their input exactly once, ensuring resource safety.

use super::base::{Type, TypeInner};

//-----------------------------------------------------------------------------
// Type Constructor
//-----------------------------------------------------------------------------

/// Linear function type constructor (A ⊸ B)
pub fn linear_function<L>(input: Type<L>, output: Type<L>) -> Type<L> {
    Type::new(TypeInner::LinearFunction(
        Box::new(input.inner),
        Box::new(output.inner),
    ))
}

//-----------------------------------------------------------------------------
// Linear Function Value
//-----------------------------------------------------------------------------

/// Type-erased linear function for runtime representation
pub struct LinearFunctionValue<A, B> {
    function: Box<dyn FnOnce(A) -> B>,
}

impl<A, B> LinearFunctionValue<A, B> {
    /// Create a new linear function
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + 'static,
    {
        LinearFunctionValue {
            function: Box::new(f),
        }
    }
    
    /// Apply the function, consuming both the function and its argument
    pub fn apply(self, input: A) -> B {
        (self.function)(input)
    }
}

//-----------------------------------------------------------------------------
// Introduction Rule
//-----------------------------------------------------------------------------

/// Introduction rule for linear functions
pub trait LinearFunctionIntro<A, B> {
    /// Introduce a linear function by providing its implementation
    /// lambda : (Meta-operation that creates A ⊸ B)
    fn intro_function<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + 'static;
}

impl<A, B> LinearFunctionIntro<A, B> for LinearFunctionValue<A, B> {
    fn intro_function<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + 'static,
    {
        LinearFunctionValue::new(f)
    }
}

//-----------------------------------------------------------------------------
// Elimination Rule
//-----------------------------------------------------------------------------

/// Elimination rule for linear functions
pub trait LinearFunctionElim<A, B> {
    /// Eliminate a linear function by application
    /// apply : (A ⊸ B) ⊗ A ⊸ B
    fn elim_function(self, input: A) -> B;
}

impl<A, B> LinearFunctionElim<A, B> for LinearFunctionValue<A, B> {
    fn elim_function(self, input: A) -> B {
        self.apply(input)
    }
}

//-----------------------------------------------------------------------------
// Unit Type
//-----------------------------------------------------------------------------

/// Unit value - the unique inhabitant of the unit type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitValue;

/// Introduction rule for unit
pub trait UnitIntro {
    /// Introduce the unit value
    /// unit : 1
    fn intro_unit() -> Self;
}

/// Elimination rule for unit
pub trait UnitElim {
    /// Eliminate unit with a continuation
    /// letunit : 1 ⊗ (1 ⊸ A) ⊸ A
    fn elim_unit<R, F>(self, f: F) -> R
    where
        F: FnOnce() -> R;
}

impl UnitIntro for UnitValue {
    fn intro_unit() -> Self {
        UnitValue
    }
}

impl UnitElim for UnitValue {
    fn elim_unit<R, F>(self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}

impl UnitValue {
    /// Create the unique unit value
    pub const fn new() -> Self {
        UnitValue
    }
}

impl Default for UnitValue {
    fn default() -> Self {
        Self::new()
    }
} 