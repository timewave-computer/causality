//! Sum types (A ⊕ B) for Layer 1
//!
//! This module provides the sum (coproduct) type for the linear lambda calculus.
//! Sum types represent choice - a value of type A ⊕ B is either an A or a B,
//! but not both.

use super::base::{Type, TypeInner};

//-----------------------------------------------------------------------------
// Type Constructor
//-----------------------------------------------------------------------------

/// Linear sum type constructor (A ⊕ B)
pub fn sum<L>(left: Type<L>, right: Type<L>) -> Type<L> {
    Type::new(TypeInner::Sum(
        Box::new(left.inner),
        Box::new(right.inner),
    ))
}

//-----------------------------------------------------------------------------
// Sum Value
//-----------------------------------------------------------------------------

/// Sum value - either left or right, but not both
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SumValue<A, B> {
    /// Left injection into the sum type
    Left(A),
    /// Right injection into the sum type
    Right(B),
}

//-----------------------------------------------------------------------------
// Introduction Rules
//-----------------------------------------------------------------------------

/// Introduction rules for sums
pub trait SumIntro<A, B> {
    /// Introduce a sum with left injection
    /// inl : A ⊸ (A ⊕ B)
    fn intro_left(value: A) -> Self;
    
    /// Introduce a sum with right injection
    /// inr : B ⊸ (A ⊕ B)
    fn intro_right(value: B) -> Self;
}

impl<A, B> SumIntro<A, B> for SumValue<A, B> {
    fn intro_left(value: A) -> Self {
        SumValue::Left(value)
    }
    
    fn intro_right(value: B) -> Self {
        SumValue::Right(value)
    }
}

//-----------------------------------------------------------------------------
// Elimination Rule
//-----------------------------------------------------------------------------

/// Elimination rule for sums
pub trait SumElim<A, B> {
    /// Eliminate a sum by case analysis
    /// case : (A ⊕ B) ⊗ (A ⊸ C) ⊗ (B ⊸ C) ⊸ C
    fn elim_sum<R, F1, F2>(self, left_case: F1, right_case: F2) -> R
    where
        F1: FnOnce(A) -> R,
        F2: FnOnce(B) -> R;
}

impl<A, B> SumElim<A, B> for SumValue<A, B> {
    fn elim_sum<R, F1, F2>(self, left_case: F1, right_case: F2) -> R
    where
        F1: FnOnce(A) -> R,
        F2: FnOnce(B) -> R,
    {
        match self {
            SumValue::Left(a) => left_case(a),
            SumValue::Right(b) => right_case(b),
        }
    }
}

//-----------------------------------------------------------------------------
// Convenience Methods
//-----------------------------------------------------------------------------

impl<A, B> SumValue<A, B> {
    /// Create a left injection (convenience)
    pub fn left(value: A) -> Self {
        <Self as SumIntro<A, B>>::intro_left(value)
    }
    
    /// Create a right injection (convenience)
    pub fn right(value: B) -> Self {
        <Self as SumIntro<A, B>>::intro_right(value)
    }
    
    /// Check if this is a left value
    pub fn is_left(&self) -> bool {
        matches!(self, SumValue::Left(_))
    }
    
    /// Check if this is a right value
    pub fn is_right(&self) -> bool {
        matches!(self, SumValue::Right(_))
    }
    
    /// Try to extract the left value
    pub fn left_value(&self) -> Option<&A> {
        match self {
            SumValue::Left(a) => Some(a),
            SumValue::Right(_) => None,
        }
    }
    
    /// Try to extract the right value
    pub fn right_value(&self) -> Option<&B> {
        match self {
            SumValue::Left(_) => None,
            SumValue::Right(b) => Some(b),
        }
    }
} 