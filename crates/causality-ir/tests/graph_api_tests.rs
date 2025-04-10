//! Tests for the TEG Graph API
//!
//! This module contains tests for the Temporal Effect Graph API,
//! focusing on external consumption and integration points.

use causality_ir::{
    TemporalEffectGraph,
    EffectNode, ResourceNode,
    EffectId, ResourceId,
    builder::TEGBuilder,
    ParameterValue,
};
use std::collections::HashMap;

// Helper function to create a test TEG
fn create_test_teg() -> TemporalEffectGraph {
    let mut builder = TEGBuilder::new();
    
    // Add effect nodes
    let effect1 = builder.add_effect("read")
        .with_parameter("path", ParameterValue::String("file1.txt".to_string()))
        .done();
        
    let effect2 = builder.add_effect("write")
        .with_parameter("path", ParameterValue::String("file2.txt".to_string()))
        .with_parameter("content", ParameterValue::String("Hello, TEG!".to_string()))
        .done();
        
    let effect3 = builder.add_effect("log")
        .with_parameter("message", ParameterValue::String("Operation completed".to_string()))
        .done();
        
    // Add resource nodes
    let resource1 = builder.add_resource("file")
        .with_id("file1.txt")
        .done();
        
    let resource2 = builder.add_resource("file")
        .with_id("file2.txt")
        .done();
        
    // Add relationships
    builder.add_continuation(effect1, effect2);
    builder.add_continuation(effect2, effect3);
    builder.add_resource_access(effect1, resource1, "read");
    builder.add_resource_access(effect2, resource2, "write");
    
    // Add metadata
    builder.add_metadata("version", "1.0".into());
    builder.add_metadata("created_by", "test_suite".into());
    
    // Build the TEG
    builder.build()
}

#[test]
fn test_effect_node_api() {
    let teg = create_test_teg();
    
    // Test effect_nodes iterator
    let effect_nodes: Vec<_> = teg.effect_nodes().collect();
    assert_eq!(effect_nodes.len(), 3, "TEG should have 3 effect nodes");
    
    // Test effect type access
    let read_effects: Vec<_> = teg.effect_nodes()
        .filter(|node| node.effect_type() == "read")
        .collect();
    assert_eq!(read_effects.len(), 1, "TEG should have 1 read effect");
    
    // Test parameter access
    let read_effect = read_effects[0];
    let path_param = read_effect.parameters().get("path");
    assert!(path_param.is_some(), "Read effect should have path parameter");
    assert_eq!(path_param.unwrap().to_string(), "file1.txt", "Path parameter should be file1.txt");
    
    // Test effect ID stability
    let effect_id = read_effect.effect_id();
    let same_effect = teg.effect_nodes()
        .find(|node| node.effect_id() == effect_id);
    assert!(same_effect.is_some(), "Should find effect by ID");
}

#[test]
fn test_resource_node_api() {
    let teg = create_test_teg();
    
    // Test resource_nodes iterator
    let resource_nodes: Vec<_> = teg.resource_nodes().collect();
    assert_eq!(resource_nodes.len(), 2, "TEG should have 2 resource nodes");
    
    // Test resource type access
    let file_resources: Vec<_> = teg.resource_nodes()
        .filter(|node| node.resource_type() == "file")
        .collect();
    assert_eq!(file_resources.len(), 2, "TEG should have 2 file resources");
    
    // Test resource ID access
    let file1 = teg.resource_nodes()
        .find(|node| node.resource_id().to_string().contains("file1.txt"));
    assert!(file1.is_some(), "TEG should contain file1.txt resource");
}

#[test]
fn test_relationship_api() {
    let teg = create_test_teg();
    
    // Get the read and write effects
    let read_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "read")
        .unwrap();
        
    let write_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "write")
        .unwrap();
        
    // Test continuation relationship
    let continuations: Vec<_> = teg.get_effect_continuations(&read_effect.effect_id()).collect();
    assert_eq!(continuations.len(), 1, "Read effect should have 1 continuation");
    
    // Test that the continuation is the write effect
    let (cont_id, _) = continuations[0];
    assert_eq!(cont_id, &write_effect.effect_id(), "Read should continue to write");
    
    // Test dependencies relationship (should be empty for read effect)
    let dependencies: Vec<_> = teg.get_effect_dependencies(&read_effect.effect_id()).collect();
    assert_eq!(dependencies.len(), 0, "Read effect should have no dependencies");
    
    // Write effect should have read as dependency
    let write_deps: Vec<_> = teg.get_effect_dependencies(&write_effect.effect_id()).collect();
    assert_eq!(write_deps.len(), 1, "Write effect should have 1 dependency");
    assert_eq!(write_deps[0], &read_effect.effect_id(), "Write should depend on read");
}

#[test]
fn test_metadata_api() {
    let teg = create_test_teg();
    
    // Test metadata access
    let metadata = teg.metadata();
    assert!(metadata.contains_key("version"), "TEG should have version metadata");
    assert_eq!(metadata.get("version").unwrap().to_string(), "1.0", "Version should be 1.0");
    
    // Test effect node metadata
    let effect = teg.effect_nodes().next().unwrap();
    let effect_metadata = effect.metadata();
    
    // Content hash should be present
    assert!(!effect.content_hash().is_empty(), "Effect should have content hash");
    
    // Test resource node metadata
    let resource = teg.resource_nodes().next().unwrap();
    let resource_metadata = resource.metadata();
    
    // Content hash should be present
    assert!(!resource.content_hash().is_empty(), "Resource should have content hash");
}

#[test]
fn test_serialization_roundtrip() {
    let teg = create_test_teg();
    
    // Serialize to binary format
    let serialized = teg.serialize().expect("Failed to serialize TEG");
    assert!(!serialized.is_empty(), "Serialized data should not be empty");
    
    // Deserialize and check equality
    let deserialized = TemporalEffectGraph::deserialize(&serialized)
        .expect("Failed to deserialize TEG");
        
    // Check that the graphs are equivalent
    assert_eq!(
        teg.effect_nodes().count(),
        deserialized.effect_nodes().count(),
        "Deserialized graph should have same number of effect nodes"
    );
    
    assert_eq!(
        teg.resource_nodes().count(),
        deserialized.resource_nodes().count(),
        "Deserialized graph should have same number of resource nodes"
    );
    
    // Check content hash preservation
    assert_eq!(
        teg.content_hash(),
        deserialized.content_hash(),
        "Content hash should be preserved through serialization"
    );
}

#[test]
fn test_json_serialization() {
    let teg = create_test_teg();
    
    // Serialize to JSON
    let json = teg.to_json().expect("Failed to convert TEG to JSON");
    let json_str = serde_json::to_string(&json).expect("Failed to serialize JSON");
    assert!(!json_str.is_empty(), "JSON string should not be empty");
    
    // Deserialize from JSON
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Failed to parse JSON string");
    let deserialized = TemporalEffectGraph::from_json(&json_value)
        .expect("Failed to convert JSON to TEG");
        
    // Check content hash preservation
    assert_eq!(
        teg.content_hash(),
        deserialized.content_hash(),
        "Content hash should be preserved through JSON serialization"
    );
}

#[test]
fn test_content_addressing() {
    let teg = create_test_teg();
    
    // Get content hash
    let content_hash = teg.content_hash();
    assert!(!content_hash.is_empty(), "Content hash should not be empty");
    
    // Verify content hash
    assert!(teg.verify_hash(&content_hash), "Content hash verification should succeed");
    
    // Create an identical TEG and check hash equality
    let teg2 = create_test_teg();
    assert_eq!(
        teg.content_hash(),
        teg2.content_hash(),
        "Identical TEGs should have the same content hash"
    );
}

#[test]
fn test_large_graph_performance() {
    use std::time::Instant;
    
    // Create a large graph
    let mut builder = TEGBuilder::new();
    
    // Add 1000 effect nodes in a chain
    let mut prev_effect = None;
    for i in 0..1000 {
        let effect = builder.add_effect("compute")
            .with_parameter("index", ParameterValue::Int(i))
            .done();
            
        if let Some(prev) = prev_effect {
            builder.add_continuation(prev, effect);
        }
        
        prev_effect = Some(effect);
    }
    
    // Add 100 resource nodes
    for i in 0..100 {
        builder.add_resource("data")
            .with_id(format!("data{}", i))
            .done();
    }
    
    let teg = builder.build();
    
    // Time various operations
    let start = Instant::now();
    let effect_count = teg.effect_nodes().count();
    let effect_time = start.elapsed();
    
    let start = Instant::now();
    let resource_count = teg.resource_nodes().count();
    let resource_time = start.elapsed();
    
    let start = Instant::now();
    let serialized = teg.serialize().expect("Failed to serialize large TEG");
    let serialize_time = start.elapsed();
    
    let start = Instant::now();
    let _ = TemporalEffectGraph::deserialize(&serialized).expect("Failed to deserialize large TEG");
    let deserialize_time = start.elapsed();
    
    // Check results
    assert_eq!(effect_count, 1000, "Should have 1000 effect nodes");
    assert_eq!(resource_count, 100, "Should have 100 resource nodes");
    
    // Performance assertions (adjust thresholds as needed for your environment)
    assert!(effect_time.as_millis() < 100, "Effect node iteration should be fast");
    assert!(resource_time.as_millis() < 50, "Resource node iteration should be fast");
    assert!(serialize_time.as_millis() < 500, "Serialization should be reasonably fast");
    assert!(deserialize_time.as_millis() < 500, "Deserialization should be reasonably fast");
}

#[test]
fn test_api_contract() {
    // This test verifies that the API contract is upheld
    let teg = create_test_teg();
    
    // 1. Effect nodes should always have a valid effect type
    for effect in teg.effect_nodes() {
        assert!(!effect.effect_type().is_empty(), "Effect type should not be empty");
    }
    
    // 2. Resource nodes should always have a valid resource type
    for resource in teg.resource_nodes() {
        assert!(!resource.resource_type().is_empty(), "Resource type should not be empty");
    }
    
    // 3. Effect IDs should be unique
    let effect_ids: Vec<_> = teg.effect_nodes().map(|e| e.effect_id()).collect();
    let unique_ids: std::collections::HashSet<_> = effect_ids.iter().collect();
    assert_eq!(effect_ids.len(), unique_ids.len(), "Effect IDs should be unique");
    
    // 4. Resource IDs should be unique
    let resource_ids: Vec<_> = teg.resource_nodes().map(|r| r.resource_id()).collect();
    let unique_ids: std::collections::HashSet<_> = resource_ids.iter().collect();
    assert_eq!(resource_ids.len(), unique_ids.len(), "Resource IDs should be unique");
    
    // 5. Continuation references should be valid
    for effect in teg.effect_nodes() {
        for (cont_id, _) in teg.get_effect_continuations(&effect.effect_id()) {
            let target = teg.effect_nodes().find(|e| e.effect_id() == *cont_id);
            assert!(target.is_some(), "Continuation target should exist");
        }
    }
    
    // 6. Dependency references should be valid
    for effect in teg.effect_nodes() {
        for dep_id in teg.get_effect_dependencies(&effect.effect_id()) {
            let dep = teg.effect_nodes().find(|e| e.effect_id() == *dep_id);
            assert!(dep.is_some(), "Dependency should exist");
        }
    }
}

#[test]
fn test_incremental_updates() {
    // Create initial TEG
    let mut teg = create_test_teg();
    let initial_hash = teg.content_hash();
    
    // Make incremental update
    let mut transaction = teg.begin_transaction();
    
    // Add a new effect
    let new_effect = transaction.add_effect("delete")
        .with_parameter("path", ParameterValue::String("file2.txt".to_string()))
        .done();
        
    // Connect to existing graph
    let write_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "write")
        .unwrap();
    
    transaction.add_continuation(write_effect.effect_id(), new_effect);
    
    // Commit transaction
    transaction.commit().expect("Failed to commit transaction");
    
    // Verify changes
    assert_ne!(
        initial_hash,
        teg.content_hash(),
        "Content hash should change after update"
    );
    
    let delete_effects: Vec<_> = teg.effect_nodes()
        .filter(|node| node.effect_type() == "delete")
        .collect();
    assert_eq!(delete_effects.len(), 1, "TEG should now have a delete effect");
    
    // Test diffing capability
    let original_teg = create_test_teg();
    let diff = teg.diff(&original_teg).expect("Failed to compute diff");
    
    assert_eq!(diff.added_effects.len(), 1, "Diff should show 1 added effect");
    assert_eq!(diff.added_continuations.len(), 1, "Diff should show 1 added continuation");
}

#[test]
fn test_effect_resource_relations() {
    let teg = create_test_teg();
    
    // Find read effect and the file1 resource
    let read_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "read")
        .unwrap();
        
    let file1 = teg.resource_nodes()
        .find(|node| node.resource_id().to_string().contains("file1.txt"))
        .unwrap();
        
    // Check resource access
    let accessed_resources = teg.get_resources_accessed_by(&read_effect.effect_id())
        .collect::<Vec<_>>();
    assert_eq!(accessed_resources.len(), 1, "Read effect should access 1 resource");
    assert_eq!(accessed_resources[0], &file1.resource_id(), "Read effect should access file1");
    
    // Check effect access
    let accessing_effects = teg.get_effects_accessing(&file1.resource_id())
        .collect::<Vec<_>>();
    assert_eq!(accessing_effects.len(), 1, "File1 should be accessed by 1 effect");
    assert_eq!(accessing_effects[0], &read_effect.effect_id(), "File1 should be accessed by read effect");
    
    // Check access mode
    let access_mode = teg.get_resource_access_mode(&read_effect.effect_id(), &file1.resource_id());
    assert_eq!(access_mode, Some("read"), "Access mode should be read");
}

#[test]
fn test_query_capabilities() {
    let teg = create_test_teg();
    
    // Test filtering by effect type
    let log_effects = teg.filter_effects(|e| e.effect_type() == "log")
        .collect::<Vec<_>>();
    assert_eq!(log_effects.len(), 1, "Filter should find 1 log effect");
    
    // Test filtering by resource type
    let file_resources = teg.filter_resources(|r| r.resource_type() == "file")
        .collect::<Vec<_>>();
    assert_eq!(file_resources.len(), 2, "Filter should find 2 file resources");
    
    // Test pagination
    let all_effects = teg.effect_nodes().collect::<Vec<_>>();
    let paginated_effects = teg.paginate_effects(0, 2).collect::<Vec<_>>();
    assert_eq!(paginated_effects.len(), 2, "Pagination should return 2 effects");
    
    // Test searching by parameter value
    let effects_with_file1 = teg.search_effects_by_parameter("path", |v| v.to_string().contains("file1"))
        .collect::<Vec<_>>();
    assert_eq!(effects_with_file1.len(), 1, "Search should find 1 effect with file1 parameter");
}

#[test]
fn test_subgraph_extraction() {
    let teg = create_test_teg();
    
    // Find read and log effects
    let read_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "read")
        .unwrap();
        
    let log_effect = teg.effect_nodes()
        .find(|node| node.effect_type() == "log")
        .unwrap();
        
    // Extract subgraph between read and log
    let subgraph = teg.extract_subgraph(read_effect.effect_id(), log_effect.effect_id())
        .expect("Failed to extract subgraph");
        
    // Verify subgraph structure
    assert_eq!(subgraph.effect_nodes().count(), 3, "Subgraph should have 3 effects");
    assert_eq!(subgraph.resource_nodes().count(), 2, "Subgraph should have 2 resources");
} 