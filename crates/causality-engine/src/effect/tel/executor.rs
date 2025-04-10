// TEL effect executor 
//
// This module provides the execution engine for TEL effects, 
// integrating with the causality-core effect system.

use std::sync::Arc;
use std::collections::HashMap;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectType as CoreEffectType,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
    EffectResult as CoreEffectResult,
    EffectError as CoreEffectError,
};

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager,
    ResourceId,
    ResourceError,
    ResourceResult
};

use causality_tel::combinators::Combinator;
use causality_tel::types::effect::{TelEffect, EffectError};

use crate::effect::registry::EffectRegistry;
use crate::effect::executor::EffectExecutor;

/// Executor for TEL effects
///
/// This executor is responsible for executing TEL effects through
/// the causality-core effect system. It handles the conversion between
/// TEL combinators and core effects, as well as the execution of those
/// effects.
pub struct TelEffectExecutor {
    /// Core effect executor
    core_executor: Arc<EffectExecutor>,
}

impl TelEffectExecutor {
    /// Create a new TEL effect executor
    pub fn new(core_executor: Arc<EffectExecutor>) -> Self {
        Self {
            core_executor,
        }
    }
    
    /// Execute a TEL effect
    pub async fn execute_effect(
        &self,
        effect: &TelEffect,
        context: &dyn CoreEffectContext
    ) -> Result<serde_json::Value, EffectError> {
        // Instead of using to_core_effect(), directly create a simplified core effect implementation
        // through the TelEffectAdapter which already has the CoreEffect impl
        let adapter = super::adapter::TelEffectAdapter::new(
            &effect.name,
            effect.combinator.clone()
        );
        let core_effect: Box<dyn CoreEffect> = super::adapter::adapter_to_core_effect(adapter);
        
        // Execute through the core effect implementation
        match core_effect.execute(context).await {
            Ok(outcome) => {
                // Convert HashMap<String, String> to serde_json::Value
                let mut map = serde_json::Map::new();
                for (key, value) in outcome.data {
                    map.insert(key, serde_json::Value::String(value));
                }
                Ok(serde_json::Value::Object(map))
            },
            Err(e) => {
                Err(EffectError::from(e))
            }
        }
    }
    
    /// Execute a combinator as an effect
    pub async fn execute_combinator(
        &self,
        combinator: &Combinator,
        context: &dyn CoreEffectContext
    ) -> Result<CoreEffectOutcome, EffectError> {
        match combinator {
            Combinator::Effect { effect_name, args, core_effect } => {
                match core_effect {
                    Some(core_effect) => {
                        // If we already have a core effect, use it
                        core_effect.execute(context).await
                            .map_err(|e| EffectError::from(e))
                    },
                    None => {
                        // Create a serialized representation for basic effects
                        let mut params = serde_json::Map::new();
                        for (i, arg) in args.iter().enumerate() {
                            params.insert(
                                format!("arg{}", i),
                                serde_json::Value::String(format!("{:?}", arg))
                            );
                        }
                        
                        // Convert params to a HashMap<String, String> for CoreEffectOutcome
                        let mut outcome_data = std::collections::HashMap::new();
                        for (key, value) in params.iter() {
                            outcome_data.insert(key.clone(), value.to_string());
                        }
                        
                        // Return a simple outcome
                        Ok(CoreEffectOutcome::success(outcome_data))
                    }
                }
            },
            Combinator::Resource { operation, resource_type, resource_id, params } => {
                // Convert parameters to a string map
                let mut string_params = HashMap::new();
                for (key, value) in params {
                    // For simplicity, just use debug representation
                    string_params.insert(key.clone(), format!("{:?}", value));
                }
                
                // Create a simple outcome with the resource operation information
                let mut outcome_data = HashMap::new();
                outcome_data.insert("operation".to_string(), operation.clone());
                outcome_data.insert("resource_type".to_string(), resource_type.clone());
                if let Some(id) = resource_id {
                    outcome_data.insert("resource_id".to_string(), id.to_string());
                }
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            _ => {
                // Convert to a HashMap<String, String> for CoreEffectOutcome
                let mut outcome_data = std::collections::HashMap::new();
                outcome_data.insert("combinator".to_string(), format!("{:?}", combinator));
                
                // Return a simple serialized representation
                Ok(CoreEffectOutcome::success(outcome_data))
            }
        }
    }
    
    /// Execute a resource operation from a combinator
    pub async fn execute_resource_operation(
        &self,
        combinator: &Combinator,
        resource_manager: Arc<dyn ResourceManager>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        match combinator {
            Combinator::Resource { operation, resource_type, resource_id, params } => {
                // Convert parameters to a string map
                let mut string_params = HashMap::new();
                for (key, value) in params {
                    // For simplicity, just use debug representation
                    string_params.insert(key.clone(), format!("{:?}", value));
                }
                
                // Execute the operation
                match operation.as_str() {
                    "create" => {
                        // Create a resource
                        resource_manager.create_resource(
                            resource_type,
                            resource_id.as_ref().map(|id| id.to_string()).unwrap_or_default().as_str(),
                            string_params
                        ).await?;
                        
                        // Return the created resource
                        resource_manager.get_resource(
                            resource_type,
                            resource_id.as_ref().map(|id| id.to_string()).unwrap_or_default().as_str()
                        ).await
                    },
                    "get" => {
                        // Get a resource
                        if let Some(id) = resource_id {
                            resource_manager.get_resource(
                                resource_type,
                                &id.to_string()
                            ).await
                        } else {
                            Err(ResourceError::ResourceError(
                                "Resource ID required for get operation".to_string()
                            ))
                        }
                    },
                    "update" => {
                        // Update a resource
                        if let Some(id) = resource_id {
                            resource_manager.update_resource(
                                resource_type,
                                &id.to_string(),
                                string_params
                            ).await?;
                            
                            // Return the updated resource
                            resource_manager.get_resource(
                                resource_type,
                                &id.to_string()
                            ).await
                        } else {
                            Err(ResourceError::ResourceError(
                                "Resource ID required for update operation".to_string()
                            ))
                        }
                    },
                    "delete" => {
                        // Delete a resource
                        if let Some(id) = resource_id {
                            // Get the resource before deletion
                            let resource = resource_manager.get_resource(
                                resource_type,
                                &id.to_string()
                            ).await?;
                            
                            // Delete it
                            resource_manager.delete_resource(
                                resource_type,
                                &id.to_string()
                            ).await?;
                            
                            Ok(resource)
                        } else {
                            Err(ResourceError::ResourceError(
                                "Resource ID required for delete operation".to_string()
                            ))
                        }
                    },
                    _ => {
                        // Custom operation
                        if let Some(id) = resource_id {
                            // Execute the operation
                            resource_manager.execute_operation(
                                resource_type,
                                &id.to_string(),
                                operation,
                                string_params
                            ).await?;
                            
                            // Return the resource
                            resource_manager.get_resource(
                                resource_type,
                                &id.to_string()
                            ).await
                        } else {
                            Err(ResourceError::ResourceError(
                                "Resource ID required for custom operation".to_string()
                            ))
                        }
                    }
                }
            },
            Combinator::StateTransition { target_state, fields, resource_id } => {
                // Convert state transition to a resource update
                if let Some(id) = resource_id {
                    // Convert fields to parameters
                    let mut string_params = HashMap::new();
                    string_params.insert("target_state".to_string(), target_state.clone());
                    
                    for (key, value) in fields {
                        // For simplicity, just use debug representation
                        string_params.insert(key.clone(), format!("{:?}", value));
                    }
                    
                    // Update the resource state
                    resource_manager.update_resource(
                        "state",  // Using "state" as the resource type for state transitions
                        &id.to_string(),
                        string_params
                    ).await?;
                    
                    // Return the updated resource
                    resource_manager.get_resource(
                        "state",
                        &id.to_string()
                    ).await
                } else {
                    // For state transitions without a resource ID, create a new state resource
                    let mut string_params = HashMap::new();
                    string_params.insert("target_state".to_string(), target_state.clone());
                    
                    for (key, value) in fields {
                        // For simplicity, just use debug representation
                        string_params.insert(key.clone(), format!("{:?}", value));
                    }
                    
                    // Create a new state resource
                    resource_manager.create_resource(
                        "state",
                        "",  // Let the resource manager assign an ID
                        string_params
                    ).await?;
                    
                    // This is a bit of a hack - we don't know the assigned ID
                    // In a real implementation, we'd get the ID from the create operation
                    Err(ResourceError::ResourceError(
                        "State transition without resource ID not fully implemented".to_string()
                    ))
                }
            },
            _ => {
                Err(ResourceError::ResourceError(
                    format!("Cannot execute resource operation for combinator: {:?}", combinator)
                ))
            }
        }
    }
} 