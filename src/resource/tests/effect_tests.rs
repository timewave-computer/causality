use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::address::Address;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectId, 
    ExecutionBoundary, ResourceChange
};
use crate::resource::{ContentId, ResourceAPI, CapabilityRef, Right};
use crate::resource::api::{ResourceMetadata, ResourceState};
use crate::resource::memory_api::MemoryResourceAPI;
use crate::crypto::ContentId;

// Simple test effect that validates the new effect trait
#[derive(Debug)]
struct TestResourceEffect {
    id: EffectId,
    resource_id: ContentId,
    resource_api: Arc<dyn ResourceAPI>,
    action: &'static str,
}

impl TestResourceEffect {
    fn new(resource_id: ContentId, resource_api: Arc<dyn ResourceAPI>, action: &'static str) -> Self {
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
    // Setup resource API
    let resource_api = Arc::new(MemoryResourceAPI::new());
    
    // Create a resource
    let resource_id = ContentId::new("test-resource");
    let metadata = ResourceMetadata::new("Test Resource", "A test resource");
    resource_api.create_resource(&resource_id, metadata.clone()).unwrap();
    
    // Create capability for this resource
    let cap_ref = CapabilityRef::new(
        "test-capability",
        Address::local("test-system")
    );
    
    resource_api.grant_capability(
        &resource_id,
        &cap_ref,
        Right::Read
    ).unwrap();
    
    // Create a test effect to modify the resource
    let effect = TestResourceEffect::new(
        resource_id.clone(),
        resource_api.clone(),
        "modify"
    );
    
    // Create context with required capabilities
    let mut context = EffectContext::new();
    
    // Add a content-addressed execution ID
    context.add_param("execution_id", ContentId::nil().to_string());
    
    // Add capability
    context.add_capability(cap_ref.clone(), Right::Read);
    
    // Execute the effect
    let outcome = effect.execute_async(&context).await.unwrap();
    
    // Verify the outcome
    assert!(outcome.success, "Effect execution failed: {:?}", outcome.error);
    assert!(outcome.data.contains_key("action"));
    assert_eq!(outcome.data.get("action").unwrap(), "modify");
    
    // Verify resource changes
    assert_eq!(outcome.resource_changes.len(), 1);
    let resource_change = &outcome.resource_changes[0];
    assert_eq!(resource_change.resource_id, resource_id);
    assert_eq!(resource_change.change_type, "MODIFY");
} 
