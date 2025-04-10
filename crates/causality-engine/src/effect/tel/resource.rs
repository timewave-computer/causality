//! TEL Resource Integration
//!
//! This module provides integration between TEL combinators and
//! the causality-core resource system. It handles the conversion
//! of TEL resource operations to core resource operations.

use std::sync::Arc;
use std::collections::HashMap;

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager,
    ResourceId,
    ResourceType,
    ResourceError,
    ResourceResult
};

use causality_tel::combinators::Combinator;

/// Registry for TEL resource operations
///
/// This registry manages the integration between TEL resource operations
/// and the causality-core resource system.
#[derive(Debug)]
pub struct TelResourceRegistry {
    /// Core resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl TelResourceRegistry {
    /// Create a new TEL resource registry
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            resource_manager,
        }
    }
    
    /// Execute a resource operation from a combinator
    pub async fn execute_resource_operation(
        &self,
        combinator: &Combinator
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
                        self.resource_manager.create_resource(
                            resource_type,
                            resource_id.as_ref().map(|id| id.to_string()).unwrap_or_default().as_str(),
                            string_params
                        ).await?;
                        
                        // Return the created resource
                        self.resource_manager.get_resource(
                            resource_type,
                            resource_id.as_ref().map(|id| id.to_string()).unwrap_or_default().as_str()
                        ).await
                    },
                    "get" => {
                        // Get a resource
                        if let Some(id) = resource_id {
                            self.resource_manager.get_resource(
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
                            self.resource_manager.update_resource(
                                resource_type,
                                &id.to_string(),
                                string_params
                            ).await?;
                            
                            // Return the updated resource
                            self.resource_manager.get_resource(
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
                            let resource = self.resource_manager.get_resource(
                                resource_type,
                                &id.to_string()
                            ).await?;
                            
                            // Delete it
                            self.resource_manager.delete_resource(
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
                            self.resource_manager.execute_operation(
                                resource_type,
                                &id.to_string(),
                                operation,
                                string_params
                            ).await?;
                            
                            // Return the resource
                            self.resource_manager.get_resource(
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
                    self.resource_manager.update_resource(
                        "state",  // Using "state" as the resource type for state transitions
                        &id.to_string(),
                        string_params
                    ).await?;
                    
                    // Return the updated resource
                    self.resource_manager.get_resource(
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
                    self.resource_manager.create_resource(
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