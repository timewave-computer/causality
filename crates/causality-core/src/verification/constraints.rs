// Constraint verification functionality
//
// This module provides abstractions for constraint-based verification.

use std::marker::PhantomData;
use std::fmt::Debug;
use std::collections::HashSet;

/// A trait for constraints that can be checked against a value
pub trait Constraint<T> {
    /// The error type returned when the constraint is violated
    type Error;
    
    /// Check if the constraint is satisfied for the given value
    fn check(&self, value: &T) -> Result<(), Self::Error>;
    
    /// Get a unique identifier for this constraint
    fn id(&self) -> &str;
    
    /// Get a description of this constraint
    fn description(&self) -> &str;
}

/// A set of constraints that can be checked together
#[derive(Debug)]
pub struct ConstraintSet<T, C, E> {
    /// The constraints in this set
    constraints: Vec<C>,
    
    /// Phantom data for the value type and error type
    _phantom: PhantomData<(T, E)>,
}

impl<T, C, E> ConstraintSet<T, C, E>
where
    C: Constraint<T, Error = E>,
{
    /// Create a new empty constraint set
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            _phantom: PhantomData,
        }
    }
    
    /// Create a constraint set with the given constraints
    pub fn with_constraints(constraints: Vec<C>) -> Self {
        Self {
            constraints,
            _phantom: PhantomData,
        }
    }
    
    /// Add a constraint to this set
    pub fn add(&mut self, constraint: C) {
        self.constraints.push(constraint);
    }
    
    /// Get the number of constraints in this set
    pub fn len(&self) -> usize {
        self.constraints.len()
    }
    
    /// Check if this set is empty
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
    
    /// Check if all constraints are satisfied for the given value
    pub fn check_all(&self, value: &T) -> Result<(), E> {
        for constraint in &self.constraints {
            constraint.check(value)?;
        }
        Ok(())
    }
    
    /// Get all constraints that are violated for the given value
    pub fn check_all_collect_errors(&self, value: &T) -> Vec<(String, E)> {
        let mut errors = Vec::new();
        
        for constraint in &self.constraints {
            if let Err(error) = constraint.check(value) {
                errors.push((constraint.id().to_string(), error));
            }
        }
        
        errors
    }
    
    /// Get the IDs of all constraints in this set
    pub fn constraint_ids(&self) -> HashSet<String> {
        self.constraints.iter()
            .map(|c| c.id().to_string())
            .collect()
    }
}

impl<T, C, E> Default for ConstraintSet<T, C, E>
where
    C: Constraint<T, Error = E>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A verifier that checks constraints against values
pub trait ConstraintVerifier<T> {
    /// The error type returned when verification fails
    type Error;
    
    /// Verify that the value satisfies all constraints
    fn verify(&self, value: &T) -> Result<(), Self::Error>;
    
    /// Check if the value satisfies all constraints
    fn is_valid(&self, value: &T) -> bool {
        self.verify(value).is_ok()
    }
    
    /// Get a list of errors for all violated constraints
    fn collect_errors(&self, value: &T) -> Vec<Self::Error>;
}

/// A constraint that is implemented using a function
pub struct FnConstraint<T, E, F> {
    /// The constraint ID
    id: String,
    
    /// The constraint description
    description: String,
    
    /// The constraint checking function
    check_fn: F,
    
    /// Phantom data for the value and error types
    _phantom: PhantomData<(T, E)>,
}

impl<T, E, F> FnConstraint<T, E, F>
where
    F: Fn(&T) -> Result<(), E>,
{
    /// Create a new function-based constraint
    pub fn new(id: impl Into<String>, description: impl Into<String>, check_fn: F) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            check_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T, E, F> Constraint<T> for FnConstraint<T, E, F>
where
    F: Fn(&T) -> Result<(), E>,
{
    type Error = E;
    
    fn check(&self, value: &T) -> Result<(), Self::Error> {
        (self.check_fn)(value)
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

impl<T, E, F> Debug for FnConstraint<T, E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnConstraint")
            .field("id", &self.id)
            .field("description", &self.description)
            .finish()
    }
}

/// A named constraint with additional metadata
#[derive(Debug)]
pub struct NamedConstraint<C> {
    /// The inner constraint
    inner: C,
    
    /// The constraint ID
    id: String,
    
    /// The constraint description
    description: String,
}

impl<C> NamedConstraint<C> {
    /// Create a new named constraint
    pub fn new(inner: C, id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            inner,
            id: id.into(),
            description: description.into(),
        }
    }
    
    /// Get the inner constraint
    pub fn inner(&self) -> &C {
        &self.inner
    }
}

impl<T, C> Constraint<T> for NamedConstraint<C>
where
    C: Constraint<T>,
{
    type Error = C::Error;
    
    fn check(&self, value: &T) -> Result<(), Self::Error> {
        self.inner.check(value)
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// Helper functions for constraint verification
pub mod helpers {
    use super::*;
    
    /// Create a constraint from a function
    pub fn constraint_from_fn<T, E, F>(
        id: impl Into<String>,
        description: impl Into<String>,
        check_fn: F,
    ) -> impl Constraint<T, Error = E>
    where
        F: Fn(&T) -> Result<(), E>,
    {
        FnConstraint::new(id, description, check_fn)
    }
    
    /// Create a constraint set from a list of constraints
    pub fn constraint_set<T, C, E>(constraints: Vec<C>) -> ConstraintSet<T, C, E>
    where
        C: Constraint<T, Error = E>,
    {
        ConstraintSet::with_constraints(constraints)
    }
    
    /// Create a named version of a constraint
    pub fn named<T, C>(
        constraint: C,
        id: impl Into<String>,
        description: impl Into<String>,
    ) -> NamedConstraint<C>
    where
        C: Constraint<T>,
    {
        NamedConstraint::new(constraint, id, description)
    }
    
    /// Combine multiple constraints into a single constraint
    pub fn combine<T, C, E>(
        id: impl Into<String>,
        description: impl Into<String>,
        constraints: Vec<C>,
    ) -> impl Constraint<T, Error = E>
    where
        C: Constraint<T, Error = E>,
        E: From<String>,
    {
        let id = id.into();
        let description = description.into();
        let constraint_set = ConstraintSet::with_constraints(constraints);
        
        FnConstraint::new(id.clone(), description, move |value: &T| {
            if let Err(errors) = constraint_set.check_all(value) {
                return Err(errors);
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;
    use std::fmt;
    
    // A simple error type for constraint violations
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ConstraintError(String);
    
    impl fmt::Display for ConstraintError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Constraint error: {}", self.0)
        }
    }
    
    impl StdError for ConstraintError {}
    
    // A simple test constraint that checks if a number is positive
    struct PositiveConstraint;
    
    impl Constraint<i32> for PositiveConstraint {
        type Error = ConstraintError;
        
        fn check(&self, value: &i32) -> Result<(), Self::Error> {
            if *value > 0 {
                Ok(())
            } else {
                Err(ConstraintError("Value must be positive".to_string()))
            }
        }
        
        fn id(&self) -> &str {
            "positive"
        }
        
        fn description(&self) -> &str {
            "Checks if a number is positive"
        }
    }
    
    // A simple test constraint that checks if a number is even
    struct EvenConstraint;
    
    impl Constraint<i32> for EvenConstraint {
        type Error = ConstraintError;
        
        fn check(&self, value: &i32) -> Result<(), Self::Error> {
            if *value % 2 == 0 {
                Ok(())
            } else {
                Err(ConstraintError("Value must be even".to_string()))
            }
        }
        
        fn id(&self) -> &str {
            "even"
        }
        
        fn description(&self) -> &str {
            "Checks if a number is even"
        }
    }
    
    #[test]
    fn test_constraint() {
        let positive = PositiveConstraint;
        
        assert!(positive.check(&5).is_ok());
        assert!(positive.check(&0).is_err());
        assert!(positive.check(&-5).is_err());
        
        let even = EvenConstraint;
        
        assert!(even.check(&2).is_ok());
        assert!(even.check(&1).is_err());
    }
    
    #[test]
    fn test_constraint_set() {
        let mut set = ConstraintSet::<i32, Box<dyn Constraint<i32, Error = ConstraintError>>, ConstraintError>::new();
        
        set.add(Box::new(PositiveConstraint));
        set.add(Box::new(EvenConstraint));
        
        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
        
        // Should satisfy all constraints
        assert!(set.check_all(&6).is_ok());
        
        // Should fail the positivity constraint
        assert!(set.check_all(&-2).is_err());
        
        // Should fail the evenness constraint
        assert!(set.check_all(&3).is_err());
        
        // Should fail both constraints
        assert!(set.check_all(&-1).is_err());
        
        // Check collecting errors
        let errors = set.check_all_collect_errors(&-1);
        assert_eq!(errors.len(), 2);
        
        // Check constraint IDs
        let ids = set.constraint_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("positive"));
        assert!(ids.contains("even"));
    }
    
    #[test]
    fn test_fn_constraint() {
        let positive = helpers::constraint_from_fn(
            "positive",
            "Checks if a number is positive",
            |value: &i32| -> Result<(), ConstraintError> {
                if *value > 0 {
                    Ok(())
                } else {
                    Err(ConstraintError("Value must be positive".to_string()))
                }
            },
        );
        
        assert!(positive.check(&5).is_ok());
        assert!(positive.check(&0).is_err());
        assert_eq!(positive.id(), "positive");
        assert_eq!(positive.description(), "Checks if a number is positive");
    }
    
    #[test]
    fn test_named_constraint() {
        let even = EvenConstraint;
        let named = helpers::named(
            even,
            "custom_even",
            "Custom constraint for even numbers",
        );
        
        assert!(named.check(&2).is_ok());
        assert!(named.check(&1).is_err());
        assert_eq!(named.id(), "custom_even");
        assert_eq!(named.description(), "Custom constraint for even numbers");
    }
    
    #[test]
    fn test_combine_constraints() {
        let positive = PositiveConstraint;
        let even = EvenConstraint;
        
        let combined = helpers::combine(
            "positive_and_even",
            "Checks if a number is positive and even",
            vec![positive, even],
        );
        
        assert!(combined.check(&6).is_ok());
        assert!(combined.check(&-2).is_err());
        assert!(combined.check(&3).is_err());
        assert!(combined.check(&-1).is_err());
        assert_eq!(combined.id(), "positive_and_even");
        assert_eq!(combined.description(), "Checks if a number is positive and even");
    }
} 