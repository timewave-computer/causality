//! Simple Data Transformation Utilities
//!
//! This module provides basic transformation utilities for converting between
//! data types in the Causality system. Replaces the previous complex trait-based
//! transformation system with simple function types.

use anyhow::Result;
use std::sync::Arc;

//-----------------------------------------------------------------------------
// Simple Transformation Function Types
//-----------------------------------------------------------------------------

/// Type alias for a simple transformation function between two types
pub type TransformFn<S, T> = Arc<dyn Fn(&S) -> Result<T> + Send + Sync>;

//-----------------------------------------------------------------------------
// Simple Transformation Helpers
//-----------------------------------------------------------------------------

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
