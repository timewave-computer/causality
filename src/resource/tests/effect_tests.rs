use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::address::Address;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectId, 
    ExecutionBoundary, ResourceChange
};
use crate::resource::{ResourceId, ResourceAPI, CapabilityRef, Right};
use crate::resource::api::{ResourceMetadata, ResourceState};
use crate::resource::memory_api::MemoryResourceAPI;

// Simple test effect that validates the new effect trait
#[derive(Debug)]
struct TestResourceEffect {
    id: EffectId,
    resource_id: ResourceId,
    resource_api: Arc<dyn ResourceAPI>,
    action: &'static str,
}

impl TestResourceEffect {
    fn new(resource_id: ResourceId, resource_api: Arc<dyn ResourceAPI>, action: &'static str) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource_id,
            resource_api,
            action
        }
    }
}

#[async_trait]
impl Effect for TestResourceEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }

    fn name(&self) -> &str {
        "test_resource_effect"
    }

    fn display_name(&self) -> String {
        format!("Test Resource Effect ({})", self.action)
    }

    fn description(&self) -> String {
        format!("Test effect that performs a {} action on a resource", self.action)
    }

    fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Synchronous execution not supported
        Err(crate::effect::EffectError::ExecutionError(
            "This effect must be executed asynchronously".to_string()
        ))
    }

    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Find capability for the resource
        let capability = context.capabilities
            .iter()
            .find(|cap| cap.capability().resource_id() == self.resource_id && cap.capability().has_right(&Right::Write))
            .ok_or_else(|| crate::effect::EffectError::CapabilityError(
                format!("Missing write capability for resource {}", self.resource_id)
            ))?;

        // Perform the action based on the specified type
        let mut data = HashMap::new();
        data.insert("resource_id".to_string(), self.resource_id.clone());
        data.insert("action".to_string(), self.action.to_string());

        let resource_change = match self.action {
            "create" => {
                // Create a new resource with sample data
                let metadata = ResourceMetadata::new("test");
                self.resource_api.create_resource(
                    capability,
                    &self.resource_id,
                    "test",
                    vec![1, 2, 3, 4],
                    metadata,
                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
                
                ResourceChange::Created { resource_id: self.resource_id.clone() }
            },
            "update" => {
                // Update an existing resource
                self.resource_api.update_resource(
                    capability,
                    &self.resource_id,
                    Some(vec![5, 6, 7, 8]),
                    None,
                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
                
                ResourceChange::Updated { resource_id: self.resource_id.clone() }
            },
            "delete" => {
                // Delete an existing resource
                self.resource_api.delete_resource(
                    capability,
                    &self.resource_id,
                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
                
                ResourceChange::Deleted { resource_id: self.resource_id.clone() }
            },
            other => {
                // Unknown action
                return Err(crate::effect::EffectError::InvalidParameter(
                    format!("Unknown action: {}", other)
                ));
            }
        };

        // Return successful outcome with appropriate data
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: true,
            data,
            error: None,
            execution_id: Some(context.execution_id),
            resource_changes: vec![resource_change],
            metadata: HashMap::new(),
        })
    }

    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        matches!(boundary, ExecutionBoundary::InsideSystem)
    }

    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::InsideSystem
    }

    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("resource_id".to_string(), self.resource_id.clone());
        params.insert("action".to_string(), self.action.to_string());
        params
    }

    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        Vec::new()
    }

    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        None
    }

    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_resource_effect_execution() {
    // Create a resource API
    let admin = Address::from("admin:0x1234");
    let api = Arc::new(MemoryResourceAPI::new(admin.clone()));
    let root_cap = api.root_capability();
    
    // Create a resource ID
    let resource_id = "test:resource:123".to_string();
    
    // Create effect context with necessary capabilities
    let mut context = EffectContext {
        execution_id: uuid::Uuid::new_v4(),
        capabilities: vec![root_cap.clone()],
        execution_boundary: ExecutionBoundary::InsideSystem,
        parameters: HashMap::new(),
    };
    
    // Create a test effect for creating a resource
    let create_effect = TestResourceEffect::new(
        resource_id.clone(),
        api.clone(),
        "create"
    );
    
    // Execute the create effect
    let create_result = create_effect.execute_async(&context).await;
    assert!(create_result.is_ok(), "Create effect failed: {:?}", create_result.err());
    let create_outcome = create_result.unwrap();
    assert!(create_outcome.success);
    assert_eq!(create_outcome.resource_changes.len(), 1);
    
    // Verify the resource was created
    let exists = api.resource_exists(&root_cap, &resource_id)
        .await
        .expect("Failed to check existence");
    assert!(exists, "Resource should exist after create effect");
    
    // Create a test effect for updating the resource
    let update_effect = TestResourceEffect::new(
        resource_id.clone(),
        api.clone(),
        "update"
    );
    
    // Execute the update effect
    let update_result = update_effect.execute_async(&context).await;
    assert!(update_result.is_ok(), "Update effect failed: {:?}", update_result.err());
    let update_outcome = update_result.unwrap();
    assert!(update_outcome.success);
    
    // Verify the resource was updated
    let resource = api.get_resource(&root_cap, &resource_id)
        .await
        .expect("Failed to get resource");
    assert_eq!(resource.data(&root_cap).await.unwrap(), &[5, 6, 7, 8]);
    
    // Create a test effect for deleting the resource
    let delete_effect = TestResourceEffect::new(
        resource_id.clone(),
        api.clone(),
        "delete"
    );
    
    // Execute the delete effect
    let delete_result = delete_effect.execute_async(&context).await;
    assert!(delete_result.is_ok(), "Delete effect failed: {:?}", delete_result.err());
    let delete_outcome = delete_result.unwrap();
    assert!(delete_outcome.success);
    
    // Verify the resource was deleted
    let exists_after_delete = api.resource_exists(&root_cap, &resource_id)
        .await
        .expect("Failed to check existence");
    assert!(!exists_after_delete, "Resource should not exist after delete effect");
} 