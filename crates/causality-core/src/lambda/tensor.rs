//! Tensor product types (A ⊗ B) for Layer 1
//!
//! This module provides the tensor product (linear product) type for the
//! linear lambda calculus. The tensor product requires both components
//! to construct and consumes both when eliminated.

use super::base::{Type, TypeInner};

//-----------------------------------------------------------------------------
// Type Constructor
//-----------------------------------------------------------------------------

/// Linear tensor product type constructor (A ⊗ B)
pub fn tensor<L>(left: Type<L>, right: Type<L>) -> Type<L> {
    Type::new(TypeInner::Product(
        Box::new(left.inner),
        Box::new(right.inner),
    ))
}

//-----------------------------------------------------------------------------
// Tensor Value
//-----------------------------------------------------------------------------

/// Tensor product value - contains both components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorValue<A, B> {
    pub left: A,
    pub right: B,
}

//-----------------------------------------------------------------------------
// Introduction Rule
//-----------------------------------------------------------------------------

/// Introduction rule for tensor products
pub trait TensorIntro<A, B> {
    /// Introduce a tensor product by providing both components
    /// tensor : A ⊸ B ⊸ (A ⊗ B)
    fn intro_tensor(left: A, right: B) -> Self;
}

impl<A, B> TensorIntro<A, B> for TensorValue<A, B> {
    fn intro_tensor(left: A, right: B) -> Self {
        TensorValue { left, right }
    }
}

//-----------------------------------------------------------------------------
// Elimination Rule
//-----------------------------------------------------------------------------

/// Elimination rule for tensor products
pub trait TensorElim<A, B> {
    /// Eliminate a tensor product by providing a continuation
    /// lettensor : (A ⊗ B) ⊸ (A ⊸ B ⊸ C) ⊸ C
    fn elim_tensor<R, F>(self, f: F) -> R
    where
        F: FnOnce(A, B) -> R;
        
    /// Project the first component (π₁)
    fn fst(self) -> A
    where
        Self: Sized,
    {
        self.elim_tensor(|a, _| a)
    }
    
    /// Project the second component (π₂)
    fn snd(self) -> B
    where
        Self: Sized,
    {
        self.elim_tensor(|_, b| b)
    }
}

impl<A, B> TensorElim<A, B> for TensorValue<A, B> {
    fn elim_tensor<R, F>(self, f: F) -> R
    where
        F: FnOnce(A, B) -> R,
    {
        f(self.left, self.right)
    }
}

//-----------------------------------------------------------------------------
// Convenience Methods
//-----------------------------------------------------------------------------

impl<A, B> TensorValue<A, B> {
    /// Create a new tensor product value
    pub fn new(left: A, right: B) -> Self {
        <Self as TensorIntro<A, B>>::intro_tensor(left, right)
    }
    
    /// Destructure the tensor into its components
    pub fn into_parts(self) -> (A, B) {
        self.elim_tensor(|a, b| (a, b))
    }
    
    /// Get references to the components
    pub fn as_parts(&self) -> (&A, &B) {
        (&self.left, &self.right)
    }
}