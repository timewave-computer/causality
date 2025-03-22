#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;
    use uuid::Uuid;
    
    use crate::address::Address;
    use crate::resource::{ResourceId, ResourceAPI, Right, CapabilityRef, MemoryResourceAPI};
    use crate::program_account::{
        ProgramAccount, ProgramAccountId, AccountData, AccountState,
        ProgramAccountEffectAdapter, ProgramAccountEffectAdapterImpl
    };
    use crate::effect::{
        Effect, EffectManager, EffectRegistry, EffectResult, EffectOutcome, 
        ProgramAccountEffect, ResourceChange, ResourceChangeType
    };
    use crate::effect::boundary::{
        EffectContext, ExecutionBoundary, ChainBoundary, BoundaryCrossing,
        BoundaryAuthentication, CrossingDirection
    };
    
    // Simple test effect implementation
    struct TestEffect {
        name: String,
        resource_api: Arc<dyn ResourceAPI>,
        expected_boundary: ExecutionBoundary,
        success: bool,
    }
    
    impl TestEffect {
        fn new(
            name: &str, 
            resource_api: Arc<dyn ResourceAPI>,
            expected_boundary: ExecutionBoundary,
            success: bool
        ) -> Self {
            Self {
                name: name.to_string(),
                resource_api,
                expected_boundary,
                success,
            }
        }
    }
    
    #[async_trait]
    impl Effect for TestEffect {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            "Test effect for unit testing"
        }
        
        fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
            vec![(ResourceId::from("test:resource"), Right::Read)]
        }
        
        async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
            // Check if executing at expected boundary
            assert_eq!(context.boundary, self.expected_boundary, 
                       "Effect executing at wrong boundary");
            
            // Create a test resource change
            let resource_change = ResourceChange {
                resource_id: ResourceId::from("test:resource"),
                change_type: ResourceChangeType::Updated,
                previous_state_hash: Some("previous_hash".to_string()),
                new_state_hash: "new_hash".to_string(),
            };
            
            if self.success {
                let mut outcome = EffectOutcome {
                    execution_id: context.execution_id,
                    success: true,
                    result: Some(serde_json::json!({ "test": "data" })),
                    error: None,
                    resource_changes: vec![resource_change],
                    metadata: context.parameters,
                };
                Ok(outcome)
            } else {
                let mut outcome = EffectOutcome {
                    execution_id: context.execution_id,
                    success: false,
                    result: None,
                    error: Some("Test failure".to_string()),
                    resource_changes: vec![],
                    metadata: context.parameters,
                };
                Ok(outcome)
            }
        }
        
        fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
            boundary == self.expected_boundary
        }
        
        fn preferred_boundary(&self) -> ExecutionBoundary {
            self.expected_boundary
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    impl ProgramAccountEffect for TestEffect {
        fn can_apply_to(&self, account: &dyn ProgramAccount) -> bool {
            account.account_type() == "test_account"
        }
        
        fn display_name(&self) -> &str {
            "Test Effect"
        }
        
        fn display_parameters(&self) -> HashMap<String, String> {
            let mut params = HashMap::new();
            params.insert("icon".to_string(), "test-icon".to_string());
            params
        }
    }
    
    // Test program account implementation
    #[derive(Clone)]
    struct TestAccount {
        id: ProgramAccountId,
        owner: Address,
        state: AccountState,
    }
    
    impl TestAccount {
        fn new() -> Self {
            Self {
                id: ProgramAccountId::new(Uuid::new_v4().to_string()),
                owner: Address::from_string("test_owner".to_string()),
                state: AccountState::Active,
            }
        }
    }
    
    #[async_trait]
    impl ProgramAccount for TestAccount {
        fn id(&self) -> &ProgramAccountId {
            &self.id
        }
        
        fn owner(&self) -> &Address {
            &self.owner
        }
        
        fn state(&self) -> &AccountState {
            &self.state
        }
        
        fn set_state(&mut self, new_state: AccountState) {
            self.state = new_state;
        }
        
        fn account_type(&self) -> &str {
            "test_account"
        }
        
        fn get_data(&self) -> AccountData {
            AccountData::Json(serde_json::json!({
                "id": self.id.to_string(),
                "owner": self.owner.to_string(),
                "state": "active"
            }))
        }
        
        async fn update_data(&mut self, _data: &AccountData) -> Result<(), String> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_effect_execution_inside_boundary() {
        // Create resource API
        let resource_api = Arc::new(MemoryResourceAPI::new());
        
        // Create effect registry and manager
        let mut registry = EffectRegistry::new();
        let test_effect = Arc::new(TestEffect::new(
            "test_inside", 
            resource_api.clone(),
            ExecutionBoundary::InsideSystem,
            true
        ));
        
        registry.register(test_effect.clone());
        let manager = Arc::new(EffectManager::new(registry, resource_api.clone()));
        
        // Create test context
        let test_resource_id = ResourceId::from("test:resource");
        let user_address = Address::from_string("test_user".to_string());
        
        // Create test resource
        resource_api.create_resource(
            &test_resource_id,
            "test_type",
            &user_address,
            b"test_data",
            None,
        ).await.unwrap();
        
        // Grant capability
        let cap = resource_api.grant_capability(
            &test_resource_id,
            &user_address,
            vec![Right::Read],
            None,
        ).await.unwrap();
        
        // Create effect context
        let context = EffectContext::new_inside(user_address.clone())
            .with_capability(cap);
        
        // Execute effect
        let outcome = manager.execute_effect("test_inside", context).await.unwrap();
        
        // Verify outcome
        assert!(outcome.success);
        assert_eq!(outcome.resource_changes.len(), 1);
        assert_eq!(outcome.resource_changes[0].resource_id, test_resource_id);
    }
    
    #[tokio::test]
    async fn test_boundary_crossing() {
        // Create resource API
        let resource_api = Arc::new(MemoryResourceAPI::new());
        
        // Create effect registry and manager
        let mut registry = EffectRegistry::new();
        let test_effect = Arc::new(TestEffect::new(
            "test_cross", 
            resource_api.clone(),
            ExecutionBoundary::InsideSystem,
            true
        ));
        
        registry.register(test_effect.clone());
        let manager = Arc::new(EffectManager::new(registry, resource_api.clone()));
        
        // Create test context for outside boundary
        let test_resource_id = ResourceId::from("test:resource");
        let user_address = Address::from_string("test_user".to_string());
        
        // Create test resource
        resource_api.create_resource(
            &test_resource_id,
            "test_type",
            &user_address,
            b"test_data",
            None,
        ).await.unwrap();
        
        // Grant capability
        let cap = resource_api.grant_capability(
            &test_resource_id,
            &user_address,
            vec![Right::Read],
            None,
        ).await.unwrap();
        
        // Create outside context (boundary crossing will happen)
        let context = EffectContext::new_outside(user_address.clone())
            .with_capability(cap);
        
        // Execute effect - should cross boundary from outside to inside
        let outcome = manager.execute_effect("test_cross", context).await.unwrap();
        
        // Verify outcome
        assert!(outcome.success);
        
        // Check boundary crossing records in registry
        let crossings = manager.registry().crossing_registry().get_by_direction(
            CrossingDirection::Inbound
        );
        
        assert!(crossings.len() > 0);
    }
    
    #[tokio::test]
    async fn test_program_account_effect_adapter() {
        // Create resource API
        let resource_api = Arc::new(MemoryResourceAPI::new());
        
        // Create effect registry and manager
        let registry = Arc::new(EffectRegistry::new());
        let effect_manager = Arc::new(EffectManager::new(registry.clone(), resource_api.clone()));
        
        // Create test effect
        let test_effect = Arc::new(TestEffect::new(
            "test_pa", 
            resource_api.clone(),
            ExecutionBoundary::InsideSystem,
            true
        ));
        
        // Register effect
        effect_manager.registry_mut().register(test_effect.clone());
        
        // Create program account effect adapter
        let mut effect_adapter = ProgramAccountEffectAdapterImpl::new(
            effect_manager.clone(), 
            resource_api.clone()
        );
        
        // Create test account
        let test_account = Arc::new(Mutex::new(TestAccount::new()));
        let account_id = test_account.lock().unwrap().id().clone();
        
        // Register account
        effect_adapter.register_account(
            test_account as Arc<dyn ProgramAccount>
        ).unwrap();
        
        // Create test resource
        let test_resource_id = ResourceId::from("test:resource");
        let user_address = Address::from_string("test_user".to_string());
        
        resource_api.create_resource(
            &test_resource_id,
            "test_type",
            &user_address,
            b"test_data",
            None,
        ).await.unwrap();
        
        // Grant capability
        let cap = resource_api.grant_capability(
            &test_resource_id,
            &user_address,
            vec![Right::Read],
            None,
        ).await.unwrap();
        
        // Register capabilities with account
        effect_adapter.register_account_capabilities(
            &account_id,
            vec![cap],
        ).unwrap();
        
        // Get available effects
        let available_effects = effect_adapter.get_available_effects(&account_id).await.unwrap();
        
        // Should find our test effect
        assert_eq!(available_effects.len(), 1);
        assert_eq!(available_effects[0].name, "test_pa");
        
        // Execute the effect
        let params = HashMap::new();
        let outcome = effect_adapter.execute_effect(
            &account_id,
            "test_pa",
            params,
        ).await.unwrap();
        
        // Verify outcome
        assert!(outcome.success);
        assert_eq!(outcome.resource_changes.len(), 1);
        assert_eq!(outcome.resource_changes[0].resource_id, test_resource_id);
    }
} 