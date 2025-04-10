//! TEG Resource Integration
//!
//! This module provides integration between TEG resource nodes and
//! the causality-core resource system. It handles resource operations
//! with the Temporal Effect Graph (TEG) intermediate representation.

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

use causality_ir::{
    ResourceNode, EffectNode, TemporalEffectGraph, 
    ResourceId as TegResourceId, DomainId,
    graph::edge::{Condition, TemporalRelation, RelationshipType, AccessMode}
};

use anyhow::{Result, anyhow};

/// Registry for TEG resource operations
///
/// This registry manages the integration between TEG resource nodes
/// and the causality-core resource system, respecting the monoidal
/// structure of resources in the TEG.
#[derive(Debug)]
pub struct TegResourceRegistry {
    /// Core resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl TegResourceRegistry {
    /// Create a new TEG resource registry
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            resource_manager,
        }
    }
    
    /// Execute a resource operation from a TEG resource node
    pub async fn execute_resource_operation(
        &self,
        resource_node: &ResourceNode,
        operation: &str,
        params: HashMap<String, serde_json::Value>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        // Get the resource type from the node
        let resource_type = &resource_node.resource_type;
        let resource_id = resource_node.id.clone();
        
        // Execute the operation
        match operation {
            "create" => {
                // Convert parameters to string map for core resource manager
                let string_params = self.convert_params_to_string(&params);
                
                // Create the resource
                self.resource_manager.create_resource(
                    resource_type,
                    &resource_id,
                    string_params
                ).await?;
                
                // Return the created resource
                self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await
            },
            "get" => {
                // Get the resource
                self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await
            },
            "update" => {
                // Convert parameters to string map for core resource manager
                let string_params = self.convert_params_to_string(&params);
                
                // Update the resource
                self.resource_manager.update_resource(
                    resource_type,
                    &resource_id,
                    string_params
                ).await?;
                
                // Return the updated resource
                self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await
            },
            "delete" => {
                // Get the resource before deletion
                let resource = self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await?;
                
                // Delete the resource
                self.resource_manager.delete_resource(
                    resource_type,
                    &resource_id
                ).await?;
                
                Ok(resource)
            },
            _ => {
                // Execute a custom operation
                let string_params = self.convert_params_to_string(&params);
                
                self.resource_manager.execute_operation(
                    resource_type,
                    &resource_id,
                    operation,
                    string_params
                ).await?;
                
                // Return the resource after operation
                self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await
            }
        }
    }
    
    /// Process a state transition
    pub async fn process_state_transition(
        &self,
        resource_node: &ResourceNode,
        target_state: &str,
        fields: HashMap<String, serde_json::Value>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        // Convert fields to parameters
        let mut params = self.convert_params_to_string(&fields);
        params.insert("target_state".to_string(), target_state.to_string());
        
        // Update the resource state
        self.resource_manager.update_resource(
            &resource_node.resource_type,
            &resource_node.id,
            params
        ).await?;
        
        // Return the updated resource
        self.resource_manager.get_resource(
            &resource_node.resource_type,
            &resource_node.id
        ).await
    }
    
    /// Convert a TEG to resources
    pub async fn register_teg_resources(
        &self,
        teg: &TemporalEffectGraph
    ) -> Result<HashMap<TegResourceId, ResourceId>> {
        let mut resource_mapping = HashMap::new();
        
        // Register all resource nodes in the TEG
        for (node_id, node) in &teg.resource_nodes {
            // Create or update the resource
            let resource_result = self.ensure_resource_exists(node).await;
            
            match resource_result {
                Ok(resource) => {
                    // Map the TEG resource ID to the core resource ID
                    resource_mapping.insert(
                        node_id.clone(),
                        resource.id().to_string()
                    );
                },
                Err(e) => {
                    // Log error but continue with other resources
                    eprintln!("Error registering resource {}: {}", node_id, e);
                }
            }
        }
        
        // Process resource relationships to respect monoidal structure
        for (resource_id, relationships) in &teg.resource_relationships {
            if let Some(core_resource_id) = resource_mapping.get(resource_id) {
                for (related_id, relationship_type) in relationships {
                    if let Some(related_core_id) = resource_mapping.get(related_id) {
                        // Register the relationship in the resource manager
                        // This is where we'd respect the monoidal structure
                        self.register_resource_relationship(
                            core_resource_id, 
                            related_core_id, 
                            relationship_type
                        ).await?;
                    }
                }
            }
        }
        
        Ok(resource_mapping)
    }
    
    /// Ensure a resource exists, creating it if necessary
    async fn ensure_resource_exists(&self, node: &ResourceNode) -> ResourceResult<Box<dyn CoreResource>> {
        // Check if the resource already exists
        let existing = self.resource_manager.get_resource(
            &node.resource_type,
            &node.id
        ).await;
        
        match existing {
            Ok(resource) => {
                // Resource exists, return it
                Ok(resource)
            },
            Err(_) => {
                // Resource doesn't exist, create it
                let params = serde_json::to_value(&node.metadata)
                    .map_err(|e| ResourceError::ResourceError(format!("Failed to serialize metadata: {}", e)))?;
                
                // Convert to string params
                let string_params = self.convert_params_to_string(
                    &serde_json::from_value(params)
                    .map_err(|e| ResourceError::ResourceError(format!("Failed to convert params: {}", e)))?
                );
                
                // Create the resource
                self.resource_manager.create_resource(
                    &node.resource_type,
                    &node.id,
                    string_params
                ).await?;
                
                // Return the created resource
                self.resource_manager.get_resource(
                    &node.resource_type,
                    &node.id
                ).await
            }
        }
    }
    
    /// Register a relationship between resources
    async fn register_resource_relationship(
        &self,
        source_id: &str,
        target_id: &str,
        relationship_type: &RelationshipType
    ) -> Result<()> {
        // This would typically update some metadata on the resources
        // or register the relationship in a separate system
        // For now, we'll just create a simple implementation
        
        // Convert relationship type to a string
        let rel_type = match relationship_type {
            RelationshipType::Composition => "composition",
            RelationshipType::Reference => "reference",
            RelationshipType::Dependency => "dependency",
            RelationshipType::Extension => "extension",
            _ => "other",
        };
        
        // Create a parameter map
        let mut params = HashMap::new();
        params.insert("relationship_type".to_string(), rel_type.to_string());
        params.insert("target_id".to_string(), target_id.to_string());
        
        // Update the source resource with the relationship
        self.resource_manager.update_resource(
            "relationship", // Using a special resource type for relationships
            source_id,
            params
        ).await
        .map_err(|e| anyhow!("Failed to register relationship: {}", e))?;
        
        Ok(())
    }
    
    /// Convert JSON parameters to string parameters
    fn convert_params_to_string(&self, params: &HashMap<String, serde_json::Value>) -> HashMap<String, String> {
        let mut string_params = HashMap::new();
        
        for (key, value) in params {
            // Convert value to string representation
            string_params.insert(key.clone(), value.to_string());
        }
        
        string_params
    }
} 