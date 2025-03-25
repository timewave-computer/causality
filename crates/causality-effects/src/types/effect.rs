//! Effect trait definitions
//!
//! This module defines the core traits that all effects must implement.

use std::any::{Any, TypeId};
use std::fmt::Debug;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::EffectError;
use crate::types::id::EffectTypeId;

/// Marker trait for types that can be used as an effect parameter
pub trait Parameter: Send + Sync + 'static {}

// Implement Parameter for common types
impl<T: Send + Sync + 'static> Parameter for T {}

/// Marker trait for types that can be used as an effect outcome
pub trait Outcome: Send + Sync + 'static {}

// Implement Outcome for common types
impl<T: Send + Sync + 'static> Outcome for T {}

/// Trait that all effects must implement
#[async_trait]
pub trait Effect: Send + Sync + Debug + 'static {
    /// The parameter type for this effect
    type Param: Parameter + Debug;
    
    /// The outcome type for this effect
    type Outcome: Outcome + Debug;
    
    /// Get the unique type ID for this effect
    fn type_id(&self) -> EffectTypeId;
    
    /// Execute this effect with the given parameter and context
    async fn execute(
        &self,
        param: Self::Param,
        context: &Context,
    ) -> Result<Self::Outcome, EffectError>;
    
    /// Returns the type ID of the parameter type
    fn param_type_id(&self) -> TypeId {
        TypeId::of::<Self::Param>()
    }
    
    /// Returns the type ID of the outcome type
    fn outcome_type_id(&self) -> TypeId {
        TypeId::of::<Self::Outcome>()
    }
    
    /// Validate the given parameter for this effect
    fn validate_param(&self, param: &Self::Param) -> Result<(), EffectError> {
        // Default implementation performs no validation
        Ok(())
    }
    
    /// Convert this effect to a boxed Any trait object
    fn as_any(&self) -> &dyn Any;
}

/// Trait for effects that query data but don't modify state
#[async_trait]
pub trait QueryEffect: Effect {
    /// Returns true, indicating this is a read-only effect
    fn is_read_only(&self) -> bool {
        true
    }
}

/// Trait for effects that modify state
#[async_trait]
pub trait TransactionEffect: Effect {
    /// Returns false, indicating this is a state-modifying effect
    fn is_read_only(&self) -> bool {
        false
    }
}

/// Type-erased effect that can be used for dynamic dispatch
#[async_trait]
pub trait AnyEffect: Send + Sync + Debug {
    /// Get the unique type ID for this effect
    fn type_id(&self) -> EffectTypeId;
    
    /// Returns the type ID of the parameter type
    fn param_type_id(&self) -> TypeId;
    
    /// Returns the type ID of the outcome type
    fn outcome_type_id(&self) -> TypeId;
    
    /// Execute this effect with the given parameter and context
    ///
    /// The parameter must be of the correct type, or this will return an error
    async fn execute_any(
        &self,
        param: Box<dyn Any + Send>,
        context: &Context,
    ) -> Result<Box<dyn Any + Send>, EffectError>;
    
    /// Returns true if this effect is read-only
    fn is_read_only(&self) -> bool;
    
    /// Convert this effect to a boxed Any trait object
    fn as_any(&self) -> &dyn Any;
    
    /// Validate the given parameter for this effect
    fn validate_param_any(&self, param: Box<dyn Any + Send>) -> Result<(), EffectError>;
}

// Implement AnyEffect for all Effect implementors
#[async_trait]
impl<E: Effect> AnyEffect for E {
    fn type_id(&self) -> EffectTypeId {
        self.type_id()
    }
    
    fn param_type_id(&self) -> TypeId {
        self.param_type_id()
    }
    
    fn outcome_type_id(&self) -> TypeId {
        self.outcome_type_id()
    }
    
    async fn execute_any(
        &self,
        param: Box<dyn Any + Send>,
        context: &Context,
    ) -> Result<Box<dyn Any + Send>, EffectError> {
        // Downcast the parameter to the correct type
        let param = param.downcast::<E::Param>()
            .map_err(|_| EffectError::InvalidParameterType {
                effect_type: self.type_id(),
                expected: std::any::type_name::<E::Param>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        // Execute the effect
        let outcome = self.execute(*param, context).await?;
        
        // Box the outcome
        Ok(Box::new(outcome))
    }
    
    fn is_read_only(&self) -> bool {
        // By default, effects are not read-only
        // This will be overridden by the QueryEffect and TransactionEffect impls
        false
    }
    
    fn as_any(&self) -> &dyn Any {
        self.as_any()
    }
    
    fn validate_param_any(&self, param: Box<dyn Any + Send>) -> Result<(), EffectError> {
        // Downcast the parameter to the correct type
        let param = param.downcast::<E::Param>()
            .map_err(|_| EffectError::InvalidParameterType {
                effect_type: self.type_id(),
                expected: std::any::type_name::<E::Param>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        // Validate the parameter
        self.validate_param(&param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    
    // Define a test effect
    #[derive(Debug)]
    struct TestEffect;
    
    #[derive(Debug)]
    struct TestParam {
        value: String,
    }
    
    #[derive(Debug)]
    struct TestOutcome {
        result: String,
    }
    
    #[async_trait]
    impl Effect for TestEffect {
        type Param = TestParam;
        type Outcome = TestOutcome;
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new("test.effect")
        }
        
        async fn execute(
            &self,
            param: Self::Param,
            _context: &Context,
        ) -> Result<Self::Outcome, EffectError> {
            Ok(TestOutcome {
                result: format!("Processed: {}", param.value),
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[tokio::test]
    async fn test_effect_execution() {
        let effect = TestEffect;
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::default();
        
        let outcome = effect.execute(param, &context).await.unwrap();
        assert_eq!(outcome.result, "Processed: hello");
    }
    
    #[tokio::test]
    async fn test_any_effect_execution() {
        let effect: Box<dyn AnyEffect> = Box::new(TestEffect);
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::default();
        
        let param_box: Box<dyn Any + Send> = Box::new(param);
        let outcome_box = effect.execute_any(param_box, &context).await.unwrap();
        
        let outcome = outcome_box.downcast::<TestOutcome>().unwrap();
        assert_eq!(outcome.result, "Processed: hello");
    }
    
    // Define a query effect
    #[derive(Debug)]
    struct TestQueryEffect;
    
    #[async_trait]
    impl Effect for TestQueryEffect {
        type Param = TestParam;
        type Outcome = TestOutcome;
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new("test.query")
        }
        
        async fn execute(
            &self,
            param: Self::Param,
            _context: &Context,
        ) -> Result<Self::Outcome, EffectError> {
            Ok(TestOutcome {
                result: format!("Queried: {}", param.value),
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl QueryEffect for TestQueryEffect {}
    
    #[tokio::test]
    async fn test_query_effect() {
        let effect = TestQueryEffect;
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::default();
        
        let outcome = effect.execute(param, &context).await.unwrap();
        assert_eq!(outcome.result, "Queried: hello");
        
        // Test the read-only flag
        assert!(effect.is_read_only());
    }
    
    // Define a transaction effect
    #[derive(Debug)]
    struct TestTransactionEffect;
    
    #[async_trait]
    impl Effect for TestTransactionEffect {
        type Param = TestParam;
        type Outcome = TestOutcome;
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new("test.transaction")
        }
        
        async fn execute(
            &self,
            param: Self::Param,
            _context: &Context,
        ) -> Result<Self::Outcome, EffectError> {
            Ok(TestOutcome {
                result: format!("Transacted: {}", param.value),
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl TransactionEffect for TestTransactionEffect {}
    
    #[tokio::test]
    async fn test_transaction_effect() {
        let effect = TestTransactionEffect;
        let param = TestParam {
            value: "hello".to_string(),
        };
        let context = Context::default();
        
        let outcome = effect.execute(param, &context).await.unwrap();
        assert_eq!(outcome.result, "Transacted: hello");
        
        // Test the read-only flag
        assert!(!effect.is_read_only());
    }
} 