# Implementing the Effect System

*This guide provides practical implementations for working with the [Effect System](../../architecture/core/effect-system.md).*

*Last updated: 2023-08-20*

## Overview

This guide covers the practical aspects of implementing and working with the Effect System in Causality. It provides code examples, best practices, and implementation patterns for creating and handling effects.

## Prerequisites

Before implementing effect-related functionality in your code, make sure you're familiar with:

- The [Effect System Architecture](../../architecture/core/effect-system.md)
- The [Time System](../../architecture/core/time-system.md)
- The [Resource System](../../architecture/core/resource-system.md)
- The [Agent-Based Resources](../../architecture/core/agent-based-resources.md)

## Implementation Guide

### Required Crates and Imports

```rust
// Core effect types
use causality_types::{
    effects::{
        Effect, EffectType, EffectContext, EffectOutcome,
        EffectError, EffectHandler,
    },
    domain::DomainId,
    fact::FactId,
    common::Value,
    agent::AgentId,
    resource::ResourceId,
};

// Effect system components
use causality_core::{
    effects::{
        EffectEngine, EffectHandlerRegistry,
        EffectValidator, TransactionManager,
    },
    agent::AgentContext,
};
```

### Creating Custom Effects

```rust
/// Custom effect for updating a resource
pub struct UpdateResourceEffect {
    /// Resource to update
    pub resource_id: ResourceId,
    /// New value
    pub value: Value,
    /// Dependencies that must be in the past
    pub dependencies: Vec<FactId>,
    /// Agent context for the effect
    pub agent_context: Option<AgentContext>,
}

impl Effect for UpdateResourceEffect {
    fn dependencies(&self) -> Vec<FactId> {
        self.dependencies.clone()
    }
    
    fn effect_type(&self) -> EffectType {
        EffectType::Resource
    }
    
    fn agent_context(&self) -> Option<&AgentContext> {
        self.agent_context.as_ref()
    }
    
    fn to_json(&self) -> Result<Value, EffectError> {
        Ok(json!({
            "resource_id": self.resource_id,
            "value": self.value,
            "dependencies": self.dependencies,
            "agent_context": self.agent_context,
        }))
    }
}

/// Handler for update resource effects
pub struct UpdateResourceEffectHandler {
    resource_service: Arc<dyn ResourceService>,
}

impl EffectHandler<UpdateResourceEffect> for UpdateResourceEffectHandler {
    async fn handle(
        &self,
        effect: &UpdateResourceEffect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        // Update the resource
        self.resource_service
            .update_resource(
                &effect.resource_id,
                &effect.value,
                context,
            )
            .await?;
        
        Ok(EffectOutcome::Success(json!({
            "resource_id": effect.resource_id,
            "updated": true,
        })))
    }
}
```

### Setting Up Effect Handlers

```rust
/// Set up effect handlers for a domain
async fn setup_effect_handlers(
    domain_id: DomainId,
) -> Result<EffectHandlerRegistry, EffectError> {
    let mut registry = EffectHandlerRegistry::new();
    
    // Register resource effect handler
    let resource_handler = Arc::new(UpdateResourceEffectHandler {
        resource_service: Arc::new(DefaultResourceService::new()),
    });
    registry.register_handler(EffectType::Resource, resource_handler);
    
    // Register time effect handler
    let time_handler = Arc::new(TimeEffectHandler {
        time_service: Arc::new(DefaultTimeService::new()),
    });
    registry.register_handler(EffectType::Time, time_handler);
    
    // Register agent effect handler
    let agent_handler = Arc::new(AgentEffectHandler {
        agent_service: Arc::new(DefaultAgentService::new()),
    });
    registry.register_handler(EffectType::Agent, agent_handler);
    
    Ok(registry)
}
```

### Executing Effects

```rust
/// Execute a single effect
async fn execute_single_effect(
    effect: &dyn Effect,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<Value, EffectError> {
    // Execute the effect
    let outcome = engine.execute_effect(effect, context).await?;
    
    // Handle the outcome
    match outcome {
        EffectOutcome::Success(value) => Ok(value),
        EffectOutcome::Error(error) => Err(error),
        EffectOutcome::Skipped => Ok(Value::Null),
    }
}

/// Execute multiple effects as a transaction
async fn execute_effects_transaction(
    effects: Vec<Box<dyn Effect>>,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<Value, EffectError> {
    // Create a new context for the transaction
    let mut tx_context = context.clone();
    tx_context.effects = effects;
    
    // Execute the transaction
    engine.execute_transaction(tx_context).await
}
```

### Validating Effects

```rust
/// Custom effect validator
pub struct CustomEffectValidator {
    fact_store: Arc<dyn FactStore>,
    resource_service: Arc<dyn ResourceService>,
    agent_service: Arc<dyn AgentService>,
}

impl EffectValidator for CustomEffectValidator {
    async fn has_dependency(&self, dependency: &FactId) 
        -> Result<bool, EffectError> {
        Ok(self.fact_store.has_fact(dependency).await?)
    }
    
    async fn validate_effect<E: Effect>(
        &self,
        effect: &E,
        context: &EffectContext,
    ) -> Result<(), EffectError> {
        // Validate agent context if provided
        if let Some(agent_context) = effect.agent_context() {
            self.validate_agent_context(agent_context, effect.effect_type()).await?;
        }
        
        // Validate effect-specific constraints
        match effect.effect_type() {
            EffectType::Resource => {
                // Validate resource constraints
                if let Some(resource_effect) = effect.as_any()
                    .downcast_ref::<UpdateResourceEffect>() {
                    if !self.resource_service
                        .has_resource(&resource_effect.resource_id)
                        .await? {
                        return Err(EffectError::ResourceNotFound(
                            resource_effect.resource_id.clone()
                        ));
                    }
                }
            }
            EffectType::Time => {
                // Validate time constraints
                // Implementation depends on time system requirements
            }
            EffectType::Agent => {
                // Validate agent-specific constraints
                if let Some(agent_effect) = effect.as_any()
                    .downcast_ref::<AgentEffect>() {
                    if !self.agent_service
                        .has_agent(&agent_effect.agent_id)
                        .await? {
                        return Err(EffectError::AgentNotFound(
                            agent_effect.agent_id.clone()
                        ));
                    }
                }
            }
            _ => {
                // Handle other effect types
            }
        }
        
        Ok(())
    }
    
    async fn validate_agent_context(
        &self,
        agent_context: &AgentContext,
        effect_type: EffectType,
    ) -> Result<(), EffectError> {
        // Validate the agent exists
        if !self.agent_service
            .has_agent(&agent_context.agent_id)
            .await? {
            return Err(EffectError::AgentNotFound(
                agent_context.agent_id.clone()
            ));
        }
        
        // Validate the agent has necessary capabilities
        let capabilities = self.agent_service
            .get_capabilities(&agent_context.agent_id)
            .await?;
            
        // Check if agent has required capabilities for this effect type
        let required_capabilities = self.effect_type_to_capabilities(effect_type);
        for required in required_capabilities {
            if !capabilities.contains(&required) {
                return Err(EffectError::InsufficientCapabilities(
                    agent_context.agent_id.clone(),
                    required,
                ));
            }
        }
        
        Ok(())
    }
    
    fn effect_type_to_capabilities(
        &self,
        effect_type: EffectType,
    ) -> Vec<Capability> {
        match effect_type {
            EffectType::Resource => vec![Capability::ResourceModify],
            EffectType::Time => vec![Capability::TimeObserve],
            EffectType::Agent => vec![Capability::AgentManage],
            _ => vec![],
        }
    }
}
```

### Composing Effects

```rust
/// Compose effects sequentially
async fn compose_effects_sequentially(
    effects: Vec<Box<dyn Effect>>,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<Value, EffectError> {
    let mut result = Value::Null;
    
    for effect in effects {
        result = engine.execute_effect(&*effect, context).await?;
    }
    
    Ok(result)
}

/// Compose effects in parallel
async fn compose_effects_parallel(
    effects: Vec<Box<dyn Effect>>,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<Vec<Value>, EffectError> {
    let handles: Vec<_> = effects.into_iter()
        .map(|effect| {
            let context = context.clone();
            tokio::spawn(async move {
                engine.execute_effect(&*effect, &context).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(handles).await;
    results.into_iter()
        .map(|r| r.map_err(|e| EffectError::ExecutionError(e.to_string())))
        .collect()
}

/// Compose effects conditionally
async fn compose_effects_conditionally(
    condition: bool,
    then_effect: Box<dyn Effect>,
    else_effect: Box<dyn Effect>,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<Value, EffectError> {
    let effect = if condition {
        then_effect
    } else {
        else_effect
    };
    
    engine.execute_effect(&*effect, context).await
}
```

## Agent-Specific Effects

```rust
/// Agent effect for managing agent resources
pub struct AgentEffect {
    /// Agent ID
    pub agent_id: AgentId,
    /// Operation to perform
    pub operation: AgentOperation,
    /// Dependencies that must be in the past
    pub dependencies: Vec<FactId>,
    /// Agent context for authorization
    pub agent_context: Option<AgentContext>,
}

/// Agent operations
pub enum AgentOperation {
    /// Create a new agent
    Create(AgentData),
    /// Update an existing agent
    Update(AgentData),
    /// Grant capability to an agent
    GrantCapability {
        target: AgentId,
        capability: Capability,
    },
    /// Revoke capability from an agent
    RevokeCapability {
        target: AgentId,
        capability: Capability,
    },
}

impl Effect for AgentEffect {
    fn dependencies(&self) -> Vec<FactId> {
        self.dependencies.clone()
    }
    
    fn effect_type(&self) -> EffectType {
        EffectType::Agent
    }
    
    fn agent_context(&self) -> Option<&AgentContext> {
        self.agent_context.as_ref()
    }
    
    fn to_json(&self) -> Result<Value, EffectError> {
        // Serialize the agent effect to JSON
        Ok(json!({
            "agent_id": self.agent_id,
            "operation": match &self.operation {
                AgentOperation::Create(data) => {
                    json!({
                        "type": "create",
                        "data": data,
                    })
                },
                AgentOperation::Update(data) => {
                    json!({
                        "type": "update",
                        "data": data,
                    })
                },
                AgentOperation::GrantCapability { target, capability } => {
                    json!({
                        "type": "grant_capability",
                        "target": target,
                        "capability": capability,
                    })
                },
                AgentOperation::RevokeCapability { target, capability } => {
                    json!({
                        "type": "revoke_capability",
                        "target": target,
                        "capability": capability,
                    })
                },
            },
            "dependencies": self.dependencies,
            "agent_context": self.agent_context,
        }))
    }
}

/// Handler for agent effects
pub struct AgentEffectHandler {
    agent_service: Arc<dyn AgentService>,
}

impl EffectHandler<AgentEffect> for AgentEffectHandler {
    async fn handle(
        &self,
        effect: &AgentEffect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        match &effect.operation {
            AgentOperation::Create(data) => {
                self.agent_service.create_agent(
                    &effect.agent_id,
                    data,
                    context,
                ).await?;
                
                Ok(EffectOutcome::Success(json!({
                    "agent_id": effect.agent_id,
                    "created": true,
                })))
            },
            AgentOperation::Update(data) => {
                self.agent_service.update_agent(
                    &effect.agent_id,
                    data,
                    context,
                ).await?;
                
                Ok(EffectOutcome::Success(json!({
                    "agent_id": effect.agent_id,
                    "updated": true,
                })))
            },
            AgentOperation::GrantCapability { target, capability } => {
                self.agent_service.grant_capability(
                    &effect.agent_id,
                    target,
                    capability,
                    context,
                ).await?;
                
                Ok(EffectOutcome::Success(json!({
                    "granted": true,
                    "from": effect.agent_id,
                    "to": target,
                    "capability": capability,
                })))
            },
            AgentOperation::RevokeCapability { target, capability } => {
                self.agent_service.revoke_capability(
                    &effect.agent_id,
                    target,
                    capability,
                    context,
                ).await?;
                
                Ok(EffectOutcome::Success(json!({
                    "revoked": true,
                    "from": effect.agent_id,
                    "to": target,
                    "capability": capability,
                })))
            },
        }
    }
}
```

## Best Practices

### Effect Creation

1. **Keep Effects Focused**
   ```rust
   // GOOD: Single responsibility
   pub struct UpdateResourceEffect {
       resource_id: ResourceId,
       value: Value,
       agent_context: Option<AgentContext>,
   }
   
   // BAD: Too many responsibilities
   pub struct ComplexEffect {
       resource_id: ResourceId,
       value: Value,
       time_update: TimeUpdate,
       network_request: NetworkRequest,
       agent_context: Option<AgentContext>,
   }
   ```

2. **Validate Early**
   ```rust
   // GOOD: Validate in constructor
   impl UpdateResourceEffect {
       pub fn new(resource_id: ResourceId, value: Value, agent_context: Option<AgentContext>) 
           -> Result<Self, EffectError> {
           if value.is_null() {
               return Err(EffectError::InvalidValue);
           }
           Ok(Self { resource_id, value, agent_context })
       }
   }
   
   // BAD: Validate in handler
   impl UpdateResourceEffect {
       pub fn new(resource_id: ResourceId, value: Value, agent_context: Option<AgentContext>) -> Self {
           Self { resource_id, value, agent_context }
       }
   }
   ```

### Effect Handlers

1. **Handle Errors Gracefully**
   ```rust
   // GOOD: Map errors appropriately
   async fn handle(&self, effect: &UpdateResourceEffect) 
       -> Result<EffectOutcome, EffectError> {
       self.resource_service
           .update_resource(&effect.resource_id, &effect.value)
           .await
           .map_err(|e| EffectError::ResourceError(e.to_string()))?;
       Ok(EffectOutcome::Success(json!({ "success": true })))
   }
   
   // BAD: Propagate raw errors
   async fn handle(&self, effect: &UpdateResourceEffect) 
       -> Result<EffectOutcome, EffectError> {
       self.resource_service
           .update_resource(&effect.resource_id, &effect.value)
           .await?;
       Ok(EffectOutcome::Success(json!({ "success": true })))
   }
   ```

2. **Provide Context in Outcomes**
   ```rust
   // GOOD: Include relevant context
   Ok(EffectOutcome::Success(json!({
       "resource_id": effect.resource_id,
       "updated": true,
       "timestamp": context.wall_time,
       "agent_id": effect.agent_context.as_ref().map(|ctx| &ctx.agent_id),
   })))
   
   // BAD: Minimal context
   Ok(EffectOutcome::Success(json!({ "success": true })))
   ```

### Effect Composition

1. **Compose Effects Safely**
   ```rust
   // GOOD: Handle composition errors
   async fn compose_effects(
       effects: Vec<Box<dyn Effect>>,
       context: &EffectContext,
   ) -> Result<Value, EffectError> {
       let mut result = Value::Null;
       for effect in effects {
           match engine.execute_effect(&*effect, context).await {
               Ok(EffectOutcome::Success(value)) => result = value,
               Ok(EffectOutcome::Error(error)) => return Err(error),
               Ok(EffectOutcome::Skipped) => continue,
           }
       }
       Ok(result)
   }
   
   // BAD: Ignore composition errors
   async fn compose_effects(
       effects: Vec<Box<dyn Effect>>,
       context: &EffectContext,
   ) -> Result<Value, EffectError> {
       let mut result = Value::Null;
       for effect in effects {
           result = engine.execute_effect(&*effect, context).await?;
       }
       Ok(result)
   }
   ```

2. **Use Appropriate Composition Patterns**
   ```rust
   // GOOD: Use parallel composition for independent effects
   let results = compose_effects_parallel(
       independent_effects,
       context,
       engine,
   ).await?;
   
   // BAD: Use sequential composition for independent effects
   let result = compose_effects_sequentially(
       independent_effects,
       context,
       engine,
   ).await?;
   ```

## Testing Effect Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_update_resource_effect() {
        // Set up test environment
        let resource_id = ResourceId::test();
        let value = json!({ "test": "value" });
        let dependencies = vec![FactId::test()];
        let agent_id = AgentId::test();
        let agent_context = AgentContext::new(agent_id.clone());
        
        // Create effect
        let effect = UpdateResourceEffect {
            resource_id: resource_id.clone(),
            value: value.clone(),
            dependencies: dependencies.clone(),
            agent_context: Some(agent_context),
        };
        
        // Create context
        let context = EffectContext::test();
        
        // Set up handler
        let handler = UpdateResourceEffectHandler {
            resource_service: Arc::new(TestResourceService::new()),
        };
        
        // Handle effect
        let outcome = handler.handle(&effect, &context).await.unwrap();
        
        // Verify outcome
        match outcome {
            EffectOutcome::Success(value) => {
                assert_eq!(value["resource_id"], resource_id);
                assert!(value["updated"].as_bool().unwrap());
            }
            _ => panic!("Expected success outcome"),
        }
    }
    
    #[tokio::test]
    async fn test_agent_effect() {
        // Set up test environment
        let agent_id = AgentId::test();
        let target_id = AgentId::test();
        let capability = Capability::ResourceModify;
        let dependencies = vec![FactId::test()];
        let agent_context = AgentContext::new(agent_id.clone());
        
        // Create effect
        let effect = AgentEffect {
            agent_id: agent_id.clone(),
            operation: AgentOperation::GrantCapability {
                target: target_id.clone(),
                capability: capability.clone(),
            },
            dependencies: dependencies.clone(),
            agent_context: Some(agent_context),
        };
        
        // Create context
        let context = EffectContext::test();
        
        // Set up handler
        let handler = AgentEffectHandler {
            agent_service: Arc::new(TestAgentService::new()),
        };
        
        // Handle effect
        let outcome = handler.handle(&effect, &context).await.unwrap();
        
        // Verify outcome
        match outcome {
            EffectOutcome::Success(value) => {
                assert!(value["granted"].as_bool().unwrap());
                assert_eq!(value["from"], agent_id);
                assert_eq!(value["to"], target_id);
            }
            _ => panic!("Expected success outcome"),
        }
    }
    
    #[tokio::test]
    async fn test_effect_composition() {
        // Set up test environment
        let effects = vec![
            Box::new(UpdateResourceEffect::test()),
            Box::new(TimeEffect::test()),
            Box::new(AgentEffect::test()),
        ];
        let context = EffectContext::test();
        let engine = EffectEngine::test();
        
        // Test sequential composition
        let result = compose_effects_sequentially(
            effects.clone(),
            &context,
            &engine,
        ).await.unwrap();
        
        assert!(!result.is_null());
        
        // Test parallel composition
        let results = compose_effects_parallel(
            effects,
            &context,
            &engine,
        ).await.unwrap();
        
        assert_eq!(results.len(), 3);
    }
}
```

## Troubleshooting

| Problem | Possible Cause | Solution |
|---------|---------------|----------|
| Effect execution fails | Missing handler | Register handler for effect type |
| | Invalid dependencies | Check dependency existence and order |
| | Resource not found | Verify resource exists and is accessible |
| | Validation error | Review effect validation rules |
| | Insufficient agent capabilities | Verify agent has required capabilities |
| Effect composition fails | Transaction rollback | Check all effects in transaction |
| | Parallel execution error | Verify effects are independent |
| | Missing context | Ensure all required context is provided |
| | Agent context missing | Add agent context for authorization |
| Handler registration fails | Duplicate handler | Use unique effect types |
| | Invalid handler type | Match handler type to effect type |
| | Missing dependencies | Add required service dependencies |

## References

- [Effect System Architecture](../../architecture/core/effect-system.md)
- [Time System](../../architecture/core/time-system.md)
- [Resource System](../../architecture/core/resource-system.md)
- [Agent-Based Resources](../../architecture/core/agent-based-resources.md)
- [ADR-001: Effect System](../../../spec/adr_001_effects.md)
- [ADR-023: Domain Adapter Effect Handler Unification](../../../spec/adr_023_domain_adapter_effect_handler_unification.md)
- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md) 