// Relationship Validation Effect
//
// This module provides an effect that validates resource operations against
// relationship constraints, ensuring that operations don't violate the relationship
// rules between resources.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::address::Address;
use crate::effect::{Effect, AsyncEffect, EffectContext, EffectOutcome, EffectResult, EffectId};
use crate::error::{Error, Result};
use crate::resource::{ResourceId, RegisterOperationType};
use crate::types::DomainId;

/// A validation effect that ensures resource operations don't violate relationship constraints
pub struct RelationshipStateValidationEffect {
    id: EffectId,
    resource_id: ResourceId,
    operation_type: RegisterOperationType,
    domain_id: DomainId,
    inner_effect: Arc<dyn Effect>,
    description: String,
}

impl RelationshipStateValidationEffect {
    /// Create a new validation effect
    pub fn new(
        resource_id: ResourceId,
        operation_type: RegisterOperationType,
        domain_id: DomainId,
        inner_effect: Arc<dyn Effect>,
        description: Option<String>,
    ) -> Self {
        Self {
            id: EffectId::new_v4(),
            resource_id,
            operation_type,
            domain_id,
            inner_effect,
            description: description.unwrap_or_else(|| format!(
                "Relationship validation for {:?} on resource {} in domain {}",
                operation_type, resource_id, domain_id
            )),
        }
    }
    
    /// Wrap an existing effect with relationship validation
    pub fn wrap(
        resource_id: ResourceId,
        operation_type: RegisterOperationType,
        domain_id: DomainId,
        inner_effect: Arc<dyn Effect>,
    ) -> Arc<dyn Effect> {
        Arc::new(Self::new(
            resource_id,
            operation_type,
            domain_id,
            inner_effect,
            None,
        ))
    }
}

impl Effect for RelationshipStateValidationEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "relationship_state_validation"
    }
    
    fn display_name(&self) -> String {
        format!("RelationshipStateValidation({})", self.resource_id)
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    fn execute(&self, _context: &EffectContext) -> Result<EffectOutcome> {
        Err(Error::OperationNotSupported("RelationshipStateValidationEffect requires async execution".into()))
    }
    
    fn can_execute_in(&self, boundary: crate::effect::ExecutionBoundary) -> bool {
        // This effect can execute in any boundary where the inner effect can execute
        self.inner_effect.can_execute_in(boundary)
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        // Prefer the same boundary as the inner effect
        self.inner_effect.preferred_boundary()
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("resource_id".to_string(), self.resource_id.to_string());
        params.insert("operation".to_string(), format!("{:?}", self.operation_type));
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl AsyncEffect for RelationshipStateValidationEffect {
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // First validate the relationship constraints
        if let Some(resource_manager) = &context.resource_manager {
            // Get relationship helper
            let lifecycle_manager = resource_manager.get_lifecycle_manager();
            let helper = lifecycle_manager.get_state_transition_helper();
            
            if let Some(helper) = helper {
                // For operations that change state, validate the state transition
                // against relationship constraints
                if matches!(self.operation_type, 
                    RegisterOperationType::Archive | 
                    RegisterOperationType::Consume |
                    RegisterOperationType::Freeze | 
                    RegisterOperationType::Unfreeze |
                    RegisterOperationType::Lock |
                    RegisterOperationType::Unlock
                ) {
                    // Get current state string
                    let current_state = match lifecycle_manager.get_state(&self.resource_id) {
                        Ok(state) => format!("{:?}", state),
                        Err(e) => {
                            return Ok(EffectOutcome {
                                id: self.id.clone(),
                                success: false,
                                data: HashMap::new(),
                                error: Some(format!("Failed to get resource state: {}", e)),
                                execution_id: context.execution_id,
                                resource_changes: Vec::new(),
                                metadata: HashMap::new(),
                            });
                        }
                    };
                    
                    // Get target state string based on operation
                    let target_state = match self.operation_type {
                        RegisterOperationType::Archive => "Archived".to_string(),
                        RegisterOperationType::Consume => "Consumed".to_string(),
                        RegisterOperationType::Freeze => "Frozen".to_string(),
                        RegisterOperationType::Unfreeze => "Active".to_string(),
                        RegisterOperationType::Lock => "Locked".to_string(),
                        RegisterOperationType::Unlock => "Active".to_string(),
                        _ => current_state.clone(), // No state change for other operations
                    };
                    
                    // Validate relationships for this transition
                    match helper.validate_relationships_for_transition(
                        &self.resource_id,
                        &current_state,
                        &target_state
                    ).await {
                        Ok(valid) => {
                            if !valid {
                                return Ok(EffectOutcome {
                                    id: self.id.clone(),
                                    success: false,
                                    data: HashMap::new(),
                                    error: Some(format!(
                                        "Resource operation would violate relationship constraints: invalid transition from {} to {}", 
                                        current_state, target_state
                                    )),
                                    execution_id: context.execution_id,
                                    resource_changes: Vec::new(),
                                    metadata: HashMap::new(),
                                });
                            }
                        },
                        Err(e) => {
                            return Ok(EffectOutcome {
                                id: self.id.clone(),
                                success: false,
                                data: HashMap::new(),
                                error: Some(format!("Failed to validate relationships: {}", e)),
                                execution_id: context.execution_id,
                                resource_changes: Vec::new(),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
        
        // If validation passes, execute the inner effect
        match self.inner_effect.execute(context) {
            Ok(outcome) => Ok(outcome),
            Err(e) => {
                if let Some(async_effect) = self.inner_effect.as_any().downcast_ref::<dyn AsyncEffect>() {
                    // Try async execution if sync failed
                    async_effect.execute_async(context).await
                } else {
                    Err(crate::effect::EffectError::ExecutionError(format!("Inner effect execution failed: {}", e)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EmptyEffect;
    
    #[tokio::test]
    async fn test_relationship_validation_effect() {
        // This would test the relationship validation effect
        // Implementation would set up resources with relationships
        // and verify that operations that would violate constraints are rejected
        
        // For now, just test construction
        let resource_id = ResourceId::from("test-resource");
        let domain_id = DomainId::from("test-domain");
        let inner_effect = Arc::new(EmptyEffect::new());
        
        let effect = RelationshipStateValidationEffect::new(
            resource_id,
            RegisterOperationType::Archive,
            domain_id,
            inner_effect,
            None,
        );
        
        assert_eq!(effect.name(), "relationship_state_validation");
        assert_eq!(effect.display_name(), "RelationshipStateValidation(test-resource)");
    }
} 