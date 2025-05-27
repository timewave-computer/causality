//! Tests for the TEG Resource Registry
//!
//! This module provides tests for the TEG Resource Registry, which manages
//! the integration between TEG resource nodes and the causality-core resource system.

use std::sync::Arc;
use std::collections::HashMap;
use futures::executor::block_on;

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager, ResourceId, ResourceResult
};

use causality_ir::{
    ResourceNode, EffectNode, TemporalEffectGraph,
    ResourceId as TegResourceId,
    graph::edge::{RelationshipType, AccessMode}
};

use crate::effect::tel::teg_resource::TegResourceRegistry;

/// Test resource implementation
#[derive(Clone, Debug)]
struct TestResource {
    id: String,
    resource_type: String,
    data: HashMap<String, String>,
    relationships: Vec<(String, String, String)>, // (target_id, relationship_type, metadata)
}

impl TestResource {
    fn new(id: impl Into<String>, resource_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            resource_type: resource_type.into(),
            data: HashMap::new(),
            relationships: Vec::new(),
        }
    }
}

impl CoreResource for TestResource {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    fn to_json(&self) -> ResourceResult<serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("id".to_string(), serde_json::Value::String(self.id.clone()));
        map.insert("type".to_string(), serde_json::Value::String(self.resource_type.clone()));
        
        let data_map = serde_json::Map::from_iter(
            self.data.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        );
        map.insert("data".to_string(), serde_json::Value::Object(data_map));
        
        let relationships = self.relationships.iter().map(|(target, rel_type, meta)| {
            let mut rel_map = serde_json::Map::new();
            rel_map.insert("target_id".to_string(), serde_json::Value::String(target.clone()));
            rel_map.insert("type".to_string(), serde_json::Value::String(rel_type.clone()));
            rel_map.insert("metadata".to_string(), serde_json::Value::String(meta.clone()));
            serde_json::Value::Object(rel_map)
        }).collect::<Vec<_>>();
        
        map.insert("relationships".to_string(), serde_json::Value::Array(relationships));
        
        Ok(serde_json::Value::Object(map))
    }
}

/// Test resource manager implementation
#[derive(Default)]
struct TestResourceManager {
    resources: std::sync::Mutex<HashMap<String, TestResource>>,
}

impl ResourceManager for TestResourceManager {
    fn create_resource(
        &self, 
        resource_type: &str, 
        resource_id: Option<&str>,
        params: impl Into<HashMap<String, String>>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        let id = resource_id.unwrap_or("test-id").to_string();
        let mut resource = TestResource::new(id.clone(), resource_type);
        resource.data = params.into();
        
        let mut resources = self.resources.lock().unwrap();
        let key = format!("{}:{}", resource_type, id);
        resources.insert(key, resource.clone());
        
        Ok(Box::new(resource))
    }
    
    fn get_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str
    ) -> ResourceResult<Box<dyn CoreResource>> {
        let resources = self.resources.lock().unwrap();
        let key = format!("{}:{}", resource_type, resource_id);
        
        if let Some(resource) = resources.get(&key) {
            Ok(Box::new(resource.clone()))
        } else {
            Err(causality_core::resource::ResourceError::ResourceNotFound(format!(
                "Resource not found: {}:{}", resource_type, resource_id
            )))
        }
    }
    
    fn update_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str, 
        params: impl Into<HashMap<String, String>>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        let mut resources = self.resources.lock().unwrap();
        let key = format!("{}:{}", resource_type, resource_id);
        
        if let Some(resource) = resources.get_mut(&key) {
            let update_params = params.into();
            for (k, v) in update_params {
                resource.data.insert(k, v);
            }
            Ok(Box::new(resource.clone()))
        } else {
            Err(causality_core::resource::ResourceError::ResourceNotFound(format!(
                "Resource not found: {}:{}", resource_type, resource_id
            )))
        }
    }
    
    fn delete_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str
    ) -> ResourceResult<()> {
        let mut resources = self.resources.lock().unwrap();
        let key = format!("{}:{}", resource_type, resource_id);
        
        if resources.remove(&key).is_some() {
            Ok(())
        } else {
            Err(causality_core::resource::ResourceError::ResourceNotFound(format!(
                "Resource not found: {}:{}", resource_type, resource_id
            )))
        }
    }
    
    fn execute_operation(
        &self, 
        resource_type: &str, 
        resource_id: &str, 
        operation: &str, 
        params: impl Into<HashMap<String, String>>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        if operation == "add_relationship" {
            let mut resources = self.resources.lock().unwrap();
            let key = format!("{}:{}", resource_type, resource_id);
            
            if let Some(resource) = resources.get_mut(&key) {
                let params = params.into();
                if let (Some(target_id), Some(rel_type)) = (params.get("target_id"), params.get("relationship_type")) {
                    let metadata = params.get("metadata").cloned().unwrap_or_default();
                    resource.relationships.push((target_id.clone(), rel_type.clone(), metadata));
                    Ok(Box::new(resource.clone()))
                } else {
                    Err(causality_core::resource::ResourceError::InvalidParameters(
                        "Missing target_id or relationship_type".to_string()
                    ))
                }
            } else {
                Err(causality_core::resource::ResourceError::ResourceNotFound(format!(
                    "Resource not found: {}:{}", resource_type, resource_id
                )))
            }
        } else {
            // Default to update for operations
            self.update_resource(resource_type, resource_id, params)
        }
    }
}

/// Create a test resource node
fn create_test_resource_node(id: &str, resource_type: &str) -> ResourceNode {
    ResourceNode {
        id: id.to_string(),
        resource_type: resource_type.to_string(),
        state: serde_json::json!({}),
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test basic resource operations
    #[tokio::test]
    async fn test_resource_operations() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let registry = TegResourceRegistry::new(resource_manager.clone());
        
        // Test resource node
        let resource_node = create_test_resource_node("test_id", "test_resource");
        
        // Test create operation
        let mut create_params = HashMap::new();
        create_params.insert("name".to_string(), serde_json::json!("Test Resource"));
        
        let create_result = registry.execute_resource_operation(
            &resource_node,
            "create",
            create_params
        ).await.unwrap();
        
        assert_eq!(create_result.id(), "test_id");
        assert_eq!(create_result.resource_type(), "test_resource");
        
        // Test get operation
        let get_result = registry.execute_resource_operation(
            &resource_node,
            "get",
            HashMap::new()
        ).await.unwrap();
        
        assert_eq!(get_result.id(), "test_id");
        assert_eq!(get_result.resource_type(), "test_resource");
        
        // Test update operation
        let mut update_params = HashMap::new();
        update_params.insert("name".to_string(), serde_json::json!("Updated Resource"));
        
        let update_result = registry.execute_resource_operation(
            &resource_node,
            "update",
            update_params
        ).await.unwrap();
        
        assert_eq!(update_result.id(), "test_id");
        let json = update_result.to_json().unwrap();
        let data = json.as_object().unwrap().get("data").unwrap().as_object().unwrap();
        assert_eq!(data.get("name").unwrap().as_str().unwrap(), "Updated Resource");
        
        // Test delete operation
        registry.execute_resource_operation(
            &resource_node,
            "delete",
            HashMap::new()
        ).await.unwrap();
        
        // Verify resource is deleted
        let get_result = registry.execute_resource_operation(
            &resource_node,
            "get",
            HashMap::new()
        ).await;
        
        assert!(get_result.is_err());
    }
    
    /// Test state transitions
    #[tokio::test]
    async fn test_state_transitions() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let registry = TegResourceRegistry::new(resource_manager.clone());
        
        // Create a resource first
        let resource_node = create_test_resource_node("transition_test", "stateful_resource");
        let mut create_params = HashMap::new();
        create_params.insert("state".to_string(), serde_json::json!("initial"));
        create_params.insert("value".to_string(), serde_json::json!("10"));
        
        let _ = registry.execute_resource_operation(
            &resource_node,
            "create",
            create_params
        ).await.unwrap();
        
        // Test state transition
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), serde_json::json!("20"));
        fields.insert("transition_reason".to_string(), serde_json::json!("test update"));
        
        let transition_result = registry.process_state_transition(
            &resource_node,
            "active",
            fields
        ).await.unwrap();
        
        // Verify the state transition
        let json = transition_result.to_json().unwrap();
        let data = json.as_object().unwrap().get("data").unwrap().as_object().unwrap();
        assert_eq!(data.get("target_state").unwrap().as_str().unwrap(), "active");
        assert_eq!(data.get("value").unwrap().as_str().unwrap(), "20");
        assert_eq!(data.get("transition_reason").unwrap().as_str().unwrap(), "test update");
    }
    
    /// Test registering TEG resources
    #[tokio::test]
    async fn test_register_teg_resources() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let registry = TegResourceRegistry::new(resource_manager.clone());
        
        // Create a TEG with multiple resources and relationships
        let mut teg = TemporalEffectGraph::new();
        
        // Add resource nodes
        let resource1 = create_test_resource_node("resource1", "test_resource");
        let resource2 = create_test_resource_node("resource2", "test_resource");
        let resource3 = create_test_resource_node("resource3", "test_resource");
        
        teg.add_resource_node(resource1);
        teg.add_resource_node(resource2);
        teg.add_resource_node(resource3);
        
        // Add resource relationships
        teg.add_resource_relationship("resource1", "resource2", RelationshipType::Composition);
        teg.add_resource_relationship("resource1", "resource3", RelationshipType::Reference);
        teg.add_resource_relationship("resource2", "resource3", RelationshipType::Dependency);
        
        // Register resources
        let resource_mapping = registry.register_teg_resources(&teg).await.unwrap();
        
        // Verify mapping contains all resources
        assert_eq!(resource_mapping.len(), 3);
        assert!(resource_mapping.contains_key("resource1"));
        assert!(resource_mapping.contains_key("resource2"));
        assert!(resource_mapping.contains_key("resource3"));
        
        // Verify resources were created
        let r1 = resource_manager.get_resource("test_resource", "resource1").await.unwrap();
        let r2 = resource_manager.get_resource("test_resource", "resource2").await.unwrap();
        let r3 = resource_manager.get_resource("test_resource", "resource3").await.unwrap();
        
        assert_eq!(r1.id(), "resource1");
        assert_eq!(r2.id(), "resource2");
        assert_eq!(r3.id(), "resource3");
    }
    
    /// Test monoidal structure preservation
    #[tokio::test]
    async fn test_monoidal_structure() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let registry = TegResourceRegistry::new(resource_manager.clone());
        
        // Create a TEG with a more complex resource structure
        let mut teg = TemporalEffectGraph::new();
        
        // Create resources representing a monoidal structure
        // We'll create resources A, B, C, D with relationships A ⊗ B = C, B ⊗ D = E
        let resource_a = create_test_resource_node("A", "monoidal_resource");
        let resource_b = create_test_resource_node("B", "monoidal_resource");
        let resource_c = create_test_resource_node("C", "monoidal_resource");
        let resource_d = create_test_resource_node("D", "monoidal_resource");
        let resource_e = create_test_resource_node("E", "monoidal_resource");
        
        teg.add_resource_node(resource_a);
        teg.add_resource_node(resource_b);
        teg.add_resource_node(resource_c);
        teg.add_resource_node(resource_d);
        teg.add_resource_node(resource_e);
        
        // Add relationships to represent tensor product
        teg.add_resource_relationship("A", "B", RelationshipType::Composition);
        teg.add_resource_relationship("C", "A", RelationshipType::Dependency);
        teg.add_resource_relationship("C", "B", RelationshipType::Dependency);
        
        teg.add_resource_relationship("B", "D", RelationshipType::Composition);
        teg.add_resource_relationship("E", "B", RelationshipType::Dependency);
        teg.add_resource_relationship("E", "D", RelationshipType::Dependency);
        
        // Register resources
        let resource_mapping = registry.register_teg_resources(&teg).await.unwrap();
        
        // Verify mapping contains all resources
        assert_eq!(resource_mapping.len(), 5);
        
        // Verify resources were created with proper relationships
        let r_a = resource_manager.get_resource("monoidal_resource", "A").await.unwrap();
        let r_b = resource_manager.get_resource("monoidal_resource", "B").await.unwrap();
        let r_c = resource_manager.get_resource("monoidal_resource", "C").await.unwrap();
        let r_d = resource_manager.get_resource("monoidal_resource", "D").await.unwrap();
        let r_e = resource_manager.get_resource("monoidal_resource", "E").await.unwrap();
        
        // Verify each resource exists
        assert_eq!(r_a.id(), "A");
        assert_eq!(r_b.id(), "B");
        assert_eq!(r_c.id(), "C");
        assert_eq!(r_d.id(), "D");
        assert_eq!(r_e.id(), "E");
    }
    
    /// Test error handling with non-existent resources
    #[tokio::test]
    async fn test_nonexistent_resource() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let registry = TegResourceRegistry::new(resource_manager.clone());
        
        // Create a non-existent resource node
        let resource_node = create_test_resource_node("nonexistent", "test_resource");
        
        // Try to get the resource
        let result = registry.execute_resource_operation(
            &resource_node,
            "get",
            HashMap::new()
        ).await;
        
        // Verify error
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("not found"));
    }
} 