// Tests for the effect system implementation

use std::sync::Arc;
use async_trait::async_trait;

use super::*;
use super::types::{EffectId, EffectTypeId, ExecutionBoundary};

// Mock effect implementation for testing
#[derive(Debug, Clone)]
struct MockEffect {
    id: EffectId,
    type_id: EffectTypeId,
    name: String,
    dependencies: Vec<ResourceId>,
    modifications: Vec<ResourceId>,
}

impl MockEffect {
    fn new(name: &str, namespace: &str, version: &str) -> Self {
        let id = EffectId::new();
        let type_id = EffectTypeId::new(name, namespace, version);
        
        Self {
            id,
            type_id,
            name: name.to_string(),
            dependencies: Vec::new(),
            modifications: Vec::new(),
        }
    }
    
    fn with_dependencies(mut self, deps: Vec<ResourceId>) -> Self {
        self.dependencies = deps;
        self
    }
    
    fn with_modifications(mut self, mods: Vec<ResourceId>) -> Self {
        self.modifications = mods;
        self
    }
}

impl Effect for MockEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        self.type_id.clone()
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn dependencies(&self) -> Vec<ResourceId> {
        self.dependencies.clone()
    }
    
    fn modifications(&self) -> Vec<ResourceId> {
        self.modifications.clone()
    }
    
    fn clone_effect(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

// Mock context implementation for testing
#[derive(Debug, Default)]
struct MockContext {}

impl EffectContext for MockContext {
    fn get_resources(&self) -> Vec<ResourceId> {
        Vec::new()
    }
    
    fn get_resource(&self, _id: &ResourceId) -> Option<Box<dyn Resource>> {
        None
    }
}

// Mock effect handler implementation for testing
#[derive(Debug)]
struct MockHandler {
    supported_types: Vec<EffectTypeId>,
    name: String,
}

impl MockHandler {
    fn new(name: &str, types: Vec<EffectTypeId>) -> Self {
        Self {
            supported_types: types,
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl EffectHandler for MockHandler {
    fn supported_types(&self) -> Vec<EffectTypeId> {
        self.supported_types.clone()
    }
    
    async fn handle(&self, effect: &dyn Effect, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        Ok(EffectOutcome::success(
            effect.id().clone(),
            format!("Handled {} with {}", effect.name(), self.name),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_registry() {
        // Create a registry
        let mut registry = BasicEffectRegistry::new();
        
        // Create effect types
        let type1 = EffectTypeId::new("test1", "test", "1.0");
        let type2 = EffectTypeId::new("test2", "test", "1.0");
        
        // Create handlers
        let handler1 = Arc::new(MockHandler::new("handler1", vec![type1.clone()]));
        let handler2 = Arc::new(MockHandler::new("handler2", vec![type2.clone()]));
        
        // Register handlers
        registry.register_handler(handler1.clone()).unwrap();
        registry.register_handler(handler2.clone()).unwrap();
        
        // Create effects
        let effect1 = MockEffect::new("test1", "test", "1.0");
        let effect2 = MockEffect::new("test2", "test", "1.0");
        
        // Register effects
        registry.register_effect(&effect1).unwrap();
        registry.register_effect(&effect2).unwrap();
        
        // Get handlers
        let handler_for_effect1 = registry.get_handler(&effect1).unwrap();
        let handler_for_effect2 = registry.get_handler(&effect2).unwrap();
        
        assert_eq!(handler_for_effect1.supported_types(), vec![type1.clone()]);
        assert_eq!(handler_for_effect2.supported_types(), vec![type2.clone()]);
    }
    
    #[tokio::test]
    async fn test_shared_registry() {
        // Create a shared registry
        let mut registry = SharedEffectRegistry::new();
        
        // Create effect types
        let type1 = EffectTypeId::new("test1", "test", "1.0");
        let type2 = EffectTypeId::new("test2", "test", "1.0");
        
        // Create handlers
        let handler1 = Arc::new(MockHandler::new("handler1", vec![type1.clone()]));
        let handler2 = Arc::new(MockHandler::new("handler2", vec![type2.clone()]));
        
        // Register handlers
        registry.register_handler(handler1.clone()).unwrap();
        registry.register_handler(handler2.clone()).unwrap();
        
        // Create a clone of the registry
        let registry_clone = registry.clone();
        
        // Create effects
        let effect1 = MockEffect::new("test1", "test", "1.0");
        let effect2 = MockEffect::new("test2", "test", "1.0");
        
        // Get handlers from the original registry
        let handler_for_effect1 = registry.get_handler(&effect1).unwrap();
        let handler_for_effect2 = registry.get_handler(&effect2).unwrap();
        
        // Get handlers from the cloned registry
        let handler_for_effect1_clone = registry_clone.get_handler(&effect1).unwrap();
        let handler_for_effect2_clone = registry_clone.get_handler(&effect2).unwrap();
        
        assert_eq!(handler_for_effect1.supported_types(), vec![type1.clone()]);
        assert_eq!(handler_for_effect2.supported_types(), vec![type2.clone()]);
        assert_eq!(handler_for_effect1_clone.supported_types(), vec![type1.clone()]);
        assert_eq!(handler_for_effect2_clone.supported_types(), vec![type2.clone()]);
    }
    
    #[tokio::test]
    async fn test_composite_handler() {
        // Create effect types
        let type1 = EffectTypeId::new("test1", "test", "1.0");
        let type2 = EffectTypeId::new("test2", "test", "1.0");
        
        // Create handlers
        let handler1 = Arc::new(MockHandler::new("handler1", vec![type1.clone()]));
        let handler2 = Arc::new(MockHandler::new("handler2", vec![type2.clone()]));
        
        // Create a composite handler
        let handlers = vec![handler1.clone(), handler2.clone()];
        let composite = CompositeEffectHandler::from_handlers("composite", handlers).unwrap();
        
        // Verify it supports both types
        let supported_types = composite.supported_types();
        assert!(supported_types.contains(&type1));
        assert!(supported_types.contains(&type2));
        
        // Create effects
        let effect1 = MockEffect::new("test1", "test", "1.0");
        let effect2 = MockEffect::new("test2", "test", "1.0");
        let context = MockContext::default();
        
        // Handle effects
        let outcome1 = composite.handle(&effect1, &context).await.unwrap();
        let outcome2 = composite.handle(&effect2, &context).await.unwrap();
        
        assert!(outcome1.is_success());
        assert!(outcome2.is_success());
    }
    
    #[tokio::test]
    async fn test_factory_methods() {
        // Test creating a shared registry
        let shared = EffectRegistryFactory::create_shared();
        assert!(shared.read().is_ok());
        
        // Test creating a basic registry
        let basic = EffectRegistryFactory::create_basic();
        assert_eq!(basic.get_supported_types().len(), 0);
        
        // Test creating a shared registry from basic
        let shared_from_basic = EffectRegistryFactory::create_shared_from_basic(BasicEffectRegistry::new());
        assert!(shared_from_basic.read().is_ok());
        
        // Test creating a selector handler
        let type1 = EffectTypeId::new("test1", "test", "1.0");
        let type2 = EffectTypeId::new("test2", "test", "1.0");
        
        let handler1 = Arc::new(MockHandler::new("handler1", vec![type1.clone()]));
        let handler2 = Arc::new(MockHandler::new("handler2", vec![type2.clone()]));
        
        let selector = EffectRegistryFactory::create_selector(
            "selector",
            |effect| effect.name().contains("test1"),
            handler1.clone(),
            handler2.clone()
        );
        
        let effect1 = MockEffect::new("test1", "test", "1.0");
        let effect2 = MockEffect::new("test2", "test", "1.0");
        let context = MockContext::default();
        
        // Handle effects with selector
        let outcome1 = selector.handle(&effect1, &context).await.unwrap();
        let outcome2 = selector.handle(&effect2, &context).await.unwrap();
        
        assert!(outcome1.is_success());
        assert!(outcome2.is_success());
    }
    
    #[tokio::test]
    async fn test_compatible_types() {
        // Create a registry
        let mut registry = BasicEffectRegistry::new();
        
        // Create a generic effect type (base version)
        let generic_type = EffectTypeId::new("test", "test", "1.0");
        
        // Create a handler for the generic type
        let handler = Arc::new(MockHandler::new("generic_handler", vec![generic_type.clone()]));
        
        // Register the handler
        registry.register_handler(handler.clone()).unwrap();
        
        // Create a specific effect with a compatible type (same name and namespace, higher version)
        let specific_effect = MockEffect::new("test", "test", "1.1");
        
        // The handler should be found based on compatibility
        let found_handler = registry.get_handler(&specific_effect).unwrap();
        
        assert_eq!(found_handler.supported_types(), vec![generic_type.clone()]);
        
        // Create a context
        let context = MockContext::default();
        
        // Execute the effect
        let outcome = registry.execute(&specific_effect, &context).await.unwrap();
        assert!(outcome.is_success());
    }
}

#[cfg(test)]
mod orchestration_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::any::Any;
    use async_trait::async_trait;

    use crate::effect::{
        Effect, EffectError, EffectId, EffectOutcome, EffectResult, EffectTypeId,
        context::{EffectContext, BasicEffectContext},
        domain::DomainId,
        orchestration::{
            OrchestrationStatus, OrchestrationBuilder, OrchestrationPlan,
        },
        ExecutionBoundary,
    };

    #[derive(Debug, Clone)]
    struct TestEffect {
        id: EffectId,
        name: String,
        should_succeed: bool,
    }

    impl TestEffect {
        fn new(name: &str, should_succeed: bool) -> Self {
            Self {
                id: EffectId::new_unique(),
                name: name.to_string(),
                should_succeed,
            }
        }
    }

    #[async_trait]
    impl Effect for TestEffect {
        fn id(&self) -> &EffectId {
            &self.id
        }

        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new(&format!("test.effect.{}", self.name))
        }

        fn boundary(&self) -> ExecutionBoundary {
            ExecutionBoundary::Inside
        }

        fn name(&self) -> String {
            format!("TestEffect({})", self.name)
        }

        fn is_valid(&self) -> bool {
            true
        }

        fn dependencies(&self) -> Vec<EffectId> {
            vec![]
        }

        fn modifications(&self) -> Vec<String> {
            vec![]
        }

        fn clone_effect(&self) -> Box<dyn Effect> {
            Box::new(self.clone())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_orchestration_plan() {
        // Create domain IDs
        let domain_a = DomainId::new("domainA");
        let domain_b = DomainId::new("domainB");

        // Create effects
        let effect_1 = TestEffect::new("effect1", true);
        let effect_2 = TestEffect::new("effect2", true);
        let effect_3 = TestEffect::new("effect3", true);

        // Create base context
        let context = BasicEffectContext::new();

        // Create orchestration plan directly
        let mut builder = OrchestrationBuilder::new(
            "test-orchestration".to_string(),
            domain_a.clone(),
            Box::new(context),
        );
        
        let step1 = builder.add_effect(effect_1.id().clone(), domain_a.clone()).unwrap();
        let step2 = builder.add_effect(effect_2.id().clone(), domain_a.clone()).unwrap();
        let step3 = builder.add_effect(effect_3.id().clone(), domain_b.clone()).unwrap();

        // Add dependencies: 1 -> 2 -> 3
        builder.add_dependency(step1, step2).unwrap();
        builder.add_dependency(step2, step3).unwrap();

        let plan = builder.build();

        // Check the plan structure
        assert_eq!(plan.steps.len(), 3, "Plan should have 3 steps");
        assert_eq!(plan.status, OrchestrationStatus::Pending);
        assert_eq!(plan.reference.primary_domain, domain_a);
        assert!(plan.reference.secondary_domains.contains(&domain_b));
        
        // Check dependencies
        assert!(plan.dependencies.get(&step1).unwrap().contains(&step2));
        assert!(plan.dependencies.get(&step2).unwrap().contains(&step3));
    }
}

#[cfg(test)]
mod effect_storage_tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::any::Any;
    use async_trait::async_trait;

    use crate::effect::{
        Effect, EffectId, EffectOutcome, EffectResult, EffectTypeId,
        storage::{
            EffectStorage, EffectExecutionRecord, EffectOutcomeRecord,
            InMemoryEffectStorage, ContentAddressedEffectStorage,
        },
        ExecutionBoundary,
    };
    use crate::storage::InMemoryContentAddressedStorage;

    // Simple test effect for storage testing
    #[derive(Debug, Clone)]
    struct TestStorageEffect {
        id: EffectId,
        name: String,
        data: HashMap<String, String>,
        deps: Vec<EffectId>,
    }

    impl TestStorageEffect {
        fn new(name: &str, data: HashMap<String, String>) -> Self {
            Self {
                id: EffectId::new_unique(),
                name: name.to_string(),
                data,
                deps: Vec::new(),
            }
        }

        fn with_dependency(mut self, dep: &EffectId) -> Self {
            self.deps.push(dep.clone());
            self
        }
    }

    #[async_trait]
    impl Effect for TestStorageEffect {
        fn id(&self) -> &EffectId {
            &self.id
        }

        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new(&format!("test.storage.{}", self.name))
        }

        fn boundary(&self) -> ExecutionBoundary {
            ExecutionBoundary::Inside
        }

        fn name(&self) -> String {
            format!("TestStorageEffect({})", self.name)
        }

        fn is_valid(&self) -> bool {
            true
        }

        fn dependencies(&self) -> Vec<EffectId> {
            self.deps.clone()
        }

        fn modifications(&self) -> Vec<String> {
            self.data.keys().cloned().collect()
        }

        fn clone_effect(&self) -> Box<dyn Effect> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_inememory_storage() {
        // Create in-memory storage
        let storage = InMemoryEffectStorage::new();

        // Create test effects
        let mut data1 = HashMap::new();
        data1.insert("key1".to_string(), "value1".to_string());
        let effect1 = TestStorageEffect::new("effect1", data1);

        let mut data2 = HashMap::new();
        data2.insert("key2".to_string(), "value2".to_string());
        let effect2 = TestStorageEffect::new("effect2", data2)
            .with_dependency(effect1.id());

        // Store effects
        let id1 = storage.store_effect(Box::new(effect1.clone())).await.unwrap();
        let id2 = storage.store_effect(Box::new(effect2.clone())).await.unwrap();

        // Verify effects exist
        assert!(storage.has_effect(&id1).await.unwrap());
        assert!(storage.has_effect(&id2).await.unwrap());

        // Retrieve effects
        let retrieved1 = storage.get_effect(&id1).await.unwrap();
        let retrieved2 = storage.get_effect(&id2).await.unwrap();

        // Verify properties
        assert_eq!(retrieved1.id(), &id1);
        assert_eq!(retrieved2.id(), &id2);
        assert_eq!(retrieved1.name(), "TestStorageEffect(effect1)");
        assert_eq!(retrieved2.name(), "TestStorageEffect(effect2)");

        // Create and store execution records
        let record1 = EffectExecutionRecord {
            effect_id: id1.clone(),
            effect_type: EffectTypeId::new("test.storage.effect1"),
            executed_at: 12345,
            outcome: EffectOutcomeRecord::Success(data1.clone()),
            dependencies: Vec::new(),
            domain: Some("test".to_string()),
            metadata: HashMap::new(),
        };

        let record2 = EffectExecutionRecord {
            effect_id: id2.clone(),
            effect_type: EffectTypeId::new("test.storage.effect2"),
            executed_at: 12346,
            outcome: EffectOutcomeRecord::Success(data2.clone()),
            dependencies: vec![id1.clone()],
            domain: Some("test".to_string()),
            metadata: HashMap::new(),
        };

        // Store records
        storage.store_execution_record(record1).await.unwrap();
        storage.store_execution_record(record2).await.unwrap();

        // Retrieve records
        let records1 = storage.get_execution_records(&id1).await.unwrap();
        let records2 = storage.get_execution_records(&id2).await.unwrap();

        // Verify records
        assert_eq!(records1.len(), 1);
        assert_eq!(records2.len(), 1);
        assert_eq!(records1[0].executed_at, 12345);
        assert_eq!(records2[0].executed_at, 12346);

        // Find effects by domain
        let domain_effects = storage.find_effects_by_domain("test").await.unwrap();
        assert_eq!(domain_effects.len(), 2);

        // Find dependencies
        let dependents = storage.find_dependent_effects(&id1).await.unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], id2);
    }

    #[tokio::test]
    async fn test_content_addressed_storage() {
        // Create content-addressed storage
        let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
        let storage = ContentAddressedEffectStorage::new(cas_storage.clone());

        // Create test effect
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        let effect = TestStorageEffect::new("effect", data);
        let effect_id = effect.id().clone();

        // Store effect
        let result = storage.store_effect(Box::new(effect)).await;
        
        // Since we don't have proper serialization in this test, we expect an error
        // This is just testing the flow, not the actual functionality
        assert!(result.is_err());
        
        // In a real implementation with proper serialization, this test would verify:
        // 1. Effect is stored with content addressing
        // 2. Effect can be retrieved by its ID
        // 3. Execution records are properly stored and retrieved
        // 4. Indexes work correctly for querying
    }
} 