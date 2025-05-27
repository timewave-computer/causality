//! Tests for the TEG Executor implementation
//!
//! This module provides comprehensive tests for the TEG executor, including
//! unit tests for individual components and integration tests for complete TEG execution.

use std::sync::Arc;
use std::collections::HashMap;
use futures::executor::block_on;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
};

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager, ResourceId, ResourceResult
};

use causality_ir::{
    TemporalEffectGraph, EffectNode, ResourceNode,
    EffectId, ResourceId as TegResourceId,
    graph::edge::{Condition, TemporalRelation, RelationshipType}
};

use crate::effect::executor::EffectExecutor;
use crate::effect::tel::teg_executor::TegExecutor;
use crate::effect::tel::teg_resource::TegResourceRegistry;

/// Test resource manager implementation for testing
#[derive(Default)]
struct TestResourceManager {
    resources: std::sync::Mutex<HashMap<String, TestResource>>,
}

/// Test resource implementation
#[derive(Clone, Debug)]
struct TestResource {
    id: String,
    resource_type: String,
    data: HashMap<String, String>,
}

impl TestResource {
    fn new(id: impl Into<String>, resource_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            resource_type: resource_type.into(),
            data: HashMap::new(),
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
        
        Ok(serde_json::Value::Object(map))
    }
}

impl ResourceManager for TestResourceManager {
    fn create_resource(
        &self, 
        resource_type: &str, 
        resource_id: Option<&str>,
        params: impl Into<HashMap<String, String>>
    ) -> ResourceResult<Box<dyn CoreResource>> {
        let id = resource_id.unwrap_or(&format!("test-{}", uuid::Uuid::new_v4())).to_string();
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
            // Update the resource
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
        // Custom operation execution
        self.update_resource(resource_type, resource_id, params)
    }
}

/// Create a test effect node
fn create_test_effect_node(id: &str, effect_type: &str, params: HashMap<String, serde_json::Value>) -> EffectNode {
    EffectNode {
        id: id.to_string(),
        effect_type: effect_type.to_string(),
        parameters: params,
        required_capabilities: Vec::new(),
        resources_accessed: Vec::new(),
        domain_id: "test_domain".to_string(),
        content_hash: Default::default(),
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

/// Create a simple TEG for testing
fn create_test_teg() -> TemporalEffectGraph {
    let mut teg = TemporalEffectGraph::new();
    
    // Add effect nodes
    let mut params1 = HashMap::new();
    params1.insert("value".to_string(), serde_json::json!(42));
    let effect1 = create_test_effect_node("effect1", "constant", params1);
    
    let mut params2 = HashMap::new();
    params2.insert("resource_type".to_string(), serde_json::json!("test_resource"));
    params2.insert("resource_id".to_string(), serde_json::json!("resource1"));
    let effect2 = create_test_effect_node("effect2", "resource_create", params2);
    
    let mut params3 = HashMap::new();
    params3.insert("resource_type".to_string(), serde_json::json!("test_resource"));
    params3.insert("resource_id".to_string(), serde_json::json!("resource1"));
    let effect3 = create_test_effect_node("effect3", "resource_get", params3);
    
    // Add resource node
    let resource1 = create_test_resource_node("resource1", "test_resource");
    
    // Add nodes to TEG
    teg.add_effect_node(effect1);
    teg.add_effect_node(effect2);
    teg.add_effect_node(effect3);
    teg.add_resource_node(resource1);
    
    // Add dependencies and continuations
    teg.add_effect_continuation("effect1", "effect2", None);
    teg.add_effect_continuation("effect2", "effect3", None);
    
    // Add resource relationships
    teg.add_effect_resource_access("effect2", "resource1", causality_ir::graph::edge::AccessMode::Write);
    teg.add_effect_resource_access("effect3", "resource1", causality_ir::graph::edge::AccessMode::Read);
    
    teg
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test TEG executor creation
    #[test]
    fn test_teg_executor_creation() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let effect_executor = Arc::new(EffectExecutor::new());
        let executor = TegExecutor::new(effect_executor, resource_manager);
        
        assert!(executor.core_executor.is_object());
        assert!(executor.resource_manager.is_object());
    }
    
    /// Test execution of a simple TEG
    #[tokio::test]
    async fn test_simple_teg_execution() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let effect_executor = Arc::new(EffectExecutor::new());
        let executor = TegExecutor::new(effect_executor, resource_manager);
        
        let teg = create_test_teg();
        let result = executor.execute(&teg).await.unwrap();
        
        // Check outputs - we should have outputs for all exit points
        assert_eq!(result.outputs.len(), 1);
        assert!(result.outputs.contains_key("effect3"));
        
        // Check trace - we should have executed all effects
        assert_eq!(result.trace.len(), 3);
        assert_eq!(result.trace[0].effect_id, "effect1");
        assert_eq!(result.trace[1].effect_id, "effect2");
        assert_eq!(result.trace[2].effect_id, "effect3");
    }
    
    /// Test resource operation execution
    #[tokio::test]
    async fn test_resource_operations() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let effect_executor = Arc::new(EffectExecutor::new());
        let executor = TegExecutor::new(effect_executor.clone(), resource_manager.clone());
        
        // Create a TEG with resource operations
        let mut teg = TemporalEffectGraph::new();
        
        // Create resource
        let mut params1 = HashMap::new();
        params1.insert("resource_type".to_string(), serde_json::json!("test_resource"));
        params1.insert("resource_id".to_string(), serde_json::json!("test_id"));
        params1.insert("name".to_string(), serde_json::json!("Test Resource"));
        let create_effect = create_test_effect_node("create", "resource_create", params1);
        
        // Get resource
        let mut params2 = HashMap::new();
        params2.insert("resource_type".to_string(), serde_json::json!("test_resource"));
        params2.insert("resource_id".to_string(), serde_json::json!("test_id"));
        let get_effect = create_test_effect_node("get", "resource_get", params2);
        
        // Update resource
        let mut params3 = HashMap::new();
        params3.insert("resource_type".to_string(), serde_json::json!("test_resource"));
        params3.insert("resource_id".to_string(), serde_json::json!("test_id"));
        params3.insert("name".to_string(), serde_json::json!("Updated Resource"));
        let update_effect = create_test_effect_node("update", "resource_update", params3);
        
        // Delete resource
        let mut params4 = HashMap::new();
        params4.insert("resource_type".to_string(), serde_json::json!("test_resource"));
        params4.insert("resource_id".to_string(), serde_json::json!("test_id"));
        let delete_effect = create_test_effect_node("delete", "resource_delete", params4);
        
        // Add nodes to TEG
        teg.add_effect_node(create_effect);
        teg.add_effect_node(get_effect);
        teg.add_effect_node(update_effect);
        teg.add_effect_node(delete_effect);
        
        // Add continuations
        teg.add_effect_continuation("create", "get", None);
        teg.add_effect_continuation("get", "update", None);
        teg.add_effect_continuation("update", "delete", None);
        
        // Execute
        let result = executor.execute(&teg).await.unwrap();
        
        // Check trace
        assert_eq!(result.trace.len(), 4);
        assert_eq!(result.trace[0].effect_id, "create");
        assert_eq!(result.trace[1].effect_id, "get");
        assert_eq!(result.trace[2].effect_id, "update");
        assert_eq!(result.trace[3].effect_id, "delete");
    }
    
    /// Test error handling in TEG execution
    #[tokio::test]
    async fn test_error_handling() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let effect_executor = Arc::new(EffectExecutor::new());
        let executor = TegExecutor::new(effect_executor.clone(), resource_manager.clone());
        
        // Create a TEG with an error case
        let mut teg = TemporalEffectGraph::new();
        
        // Create an effect that references a non-existent resource
        let mut params = HashMap::new();
        params.insert("resource_type".to_string(), serde_json::json!("test_resource"));
        params.insert("resource_id".to_string(), serde_json::json!("nonexistent"));
        let effect = create_test_effect_node("error_effect", "resource_get", params);
        
        teg.add_effect_node(effect);
        
        // Execute - this should fail with a resource not found error
        let result = executor.execute(&teg).await;
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("not found"));
    }
    
    /// Test conditional execution in TEG
    #[tokio::test]
    async fn test_conditional_execution() {
        let resource_manager = Arc::new(TestResourceManager::default());
        let effect_executor = Arc::new(EffectExecutor::new());
        let executor = TegExecutor::new(effect_executor.clone(), resource_manager.clone());
        
        // Create a TEG with conditional execution
        let mut teg = TemporalEffectGraph::new();
        
        // Create effects
        let effect1 = create_test_effect_node("effect1", "constant", HashMap::new());
        let effect2 = create_test_effect_node("effect2", "constant", HashMap::new());
        let effect3 = create_test_effect_node("effect3", "constant", HashMap::new());
        
        teg.add_effect_node(effect1);
        teg.add_effect_node(effect2);
        teg.add_effect_node(effect3);
        
        // Add conditional continuations
        // Always condition - should execute
        teg.add_effect_continuation_with_condition(
            "effect1", 
            "effect2", 
            Condition::Always
        );
        
        // Never condition - should not execute
        teg.add_effect_continuation_with_condition(
            "effect1", 
            "effect3", 
            Condition::Never
        );
        
        // Execute
        let result = executor.execute(&teg).await.unwrap();
        
        // Check trace - only effect1 and effect2 should have executed
        assert_eq!(result.trace.len(), 2);
        assert_eq!(result.trace[0].effect_id, "effect1");
        assert_eq!(result.trace[1].effect_id, "effect2");
    }
} 