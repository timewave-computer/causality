// Tests for the effect system

mod effect_execution_tests;
mod boundary_tests;
mod basic_effects;

pub use basic_effects::{TestTransferEffect, TestStorageEffect, TestQueryEffect};

#[cfg(test)]
mod empty_effect_tests {
    use std::collections::HashMap;
    use uuid::Uuid;
    use crate::effect::{EmptyEffect, Effect, EffectContext, ExecutionBoundary};

    #[tokio::test]
    async fn test_empty_effect_creation() {
        let effect = EmptyEffect::new();
        assert_eq!(effect.name(), "empty");
        assert_eq!(effect.description(), "No-op effect".to_string());
        
        let named_effect = EmptyEffect::with_name("custom_empty");
        assert_eq!(named_effect.name(), "custom_empty");
        
        let described_effect = EmptyEffect::with_description("Custom description");
        assert_eq!(described_effect.description(), "Custom description".to_string());
        
        let boundary_effect = EmptyEffect::with_boundary(ExecutionBoundary::External);
        assert_eq!(boundary_effect.preferred_boundary(), ExecutionBoundary::External);
    }

    #[tokio::test]
    async fn test_empty_effect_execution() {
        let effect = EmptyEffect::new();
        let context = EffectContext {
            execution_id: Uuid::new_v4(),
            capabilities: Vec::new(),
            execution_boundary: ExecutionBoundary::InsideSystem,
            parameters: HashMap::new(),
        };
        
        // Test synchronous execution
        let sync_result = effect.execute(&context);
        assert!(sync_result.is_ok(), "Sync execution failed: {:?}", sync_result);
        let sync_outcome = sync_result.unwrap();
        assert!(sync_outcome.success);
        assert!(sync_outcome.error.is_none());
        
        // Test asynchronous execution
        let async_result = effect.execute_async(&context).await;
        assert!(async_result.is_ok(), "Async execution failed: {:?}", async_result);
        let async_outcome = async_result.unwrap();
        assert!(async_outcome.success);
        assert!(async_outcome.error.is_none());
    }

    #[tokio::test]
    async fn test_empty_effect_fact_methods() {
        let effect = EmptyEffect::new();
        
        // Test fact dependencies
        let deps = effect.fact_dependencies();
        assert!(deps.is_empty(), "Empty effect should not have any fact dependencies");
        
        // Test fact snapshot
        let snapshot = effect.fact_snapshot();
        assert!(snapshot.is_none(), "Empty effect should not have a fact snapshot");
        
        // Test fact validation
        let validation = effect.validate_fact_dependencies();
        assert!(validation.is_ok(), "Fact validation should succeed for empty effect");
    }
} 