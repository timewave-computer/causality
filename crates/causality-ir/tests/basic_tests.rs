// Basic tests for the Temporal Effect Graph implementation

use std::collections::HashMap;
use causality_ir::{
    TemporalEffectGraph, EffectNode, ResourceNode, TEGFragment,
    builder::build_graph,
    graph::edge::{Condition, TemporalRelation, RelationshipType, AccessMode},
};
use causality_types::ContentHash;

#[test]
fn test_teg_creation() {
    let teg = TemporalEffectGraph::new();
    assert_eq!(teg.effect_nodes.len(), 0);
    assert_eq!(teg.resource_nodes.len(), 0);
    assert_eq!(teg.edges.len(), 0);
}

#[test]
fn test_effect_node_addition() {
    let mut teg = TemporalEffectGraph::new();
    
    let effect = EffectNode {
        id: "test_effect".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    let effect_id = teg.add_effect_node(effect.clone());
    
    assert_eq!(teg.effect_nodes.len(), 1);
    assert!(teg.effect_nodes.contains_key(effect_id));
    assert_eq!(teg.effect_nodes[effect_id].effect_type, "test_type");
}

#[test]
fn test_resource_node_addition() {
    let mut teg = TemporalEffectGraph::new();
    
    let resource = ResourceNode {
        id: "test_resource".to_string(),
        resource_type: "test_type".to_string(),
        state: serde_json::json!({}),
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    let resource_id = teg.add_resource_node(resource.clone());
    
    assert_eq!(teg.resource_nodes.len(), 1);
    assert!(teg.resource_nodes.contains_key(resource_id));
    assert_eq!(teg.resource_nodes[resource_id].resource_type, "test_type");
}

#[test]
fn test_fragment_creation_and_composition() {
    // Create first fragment
    let mut fragment1 = TEGFragment::new();
    
    let effect1 = EffectNode {
        id: "effect1".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    fragment1.effect_nodes.insert(effect1.id.clone(), effect1);
    fragment1.entry_points.push("effect1".to_string());
    fragment1.exit_points.push("effect1".to_string());
    
    // Create second fragment
    let mut fragment2 = TEGFragment::new();
    
    let effect2 = EffectNode {
        id: "effect2".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    fragment2.effect_nodes.insert(effect2.id.clone(), effect2);
    fragment2.entry_points.push("effect2".to_string());
    fragment2.exit_points.push("effect2".to_string());
    
    // Compose fragments
    let combined = fragment1.sequence(fragment2);
    
    assert_eq!(combined.effect_nodes.len(), 2);
    assert!(combined.effect_nodes.contains_key("effect1"));
    assert!(combined.effect_nodes.contains_key("effect2"));
    
    // Check that the sequence created a continuation
    assert!(combined.effect_continuations.contains_key("effect1"));
    assert_eq!(combined.effect_continuations["effect1"].len(), 1);
    assert_eq!(combined.effect_continuations["effect1"][0].0, "effect2");
}

#[test]
fn test_parallel_composition() {
    // Create first fragment
    let mut fragment1 = TEGFragment::new();
    
    let effect1 = EffectNode {
        id: "effect1".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    fragment1.effect_nodes.insert(effect1.id.clone(), effect1);
    fragment1.entry_points.push("effect1".to_string());
    fragment1.exit_points.push("effect1".to_string());
    
    // Create second fragment
    let mut fragment2 = TEGFragment::new();
    
    let effect2 = EffectNode {
        id: "effect2".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    fragment2.effect_nodes.insert(effect2.id.clone(), effect2);
    fragment2.entry_points.push("effect2".to_string());
    fragment2.exit_points.push("effect2".to_string());
    
    // Compose fragments in parallel
    let combined = fragment1.parallel(fragment2);
    
    assert_eq!(combined.effect_nodes.len(), 2);
    assert!(combined.effect_nodes.contains_key("effect1"));
    assert!(combined.effect_nodes.contains_key("effect2"));
    
    // Check that entry and exit points are combined
    assert_eq!(combined.entry_points.len(), 2);
    assert_eq!(combined.exit_points.len(), 2);
    assert!(combined.entry_points.contains(&"effect1".to_string()));
    assert!(combined.entry_points.contains(&"effect2".to_string()));
}

#[test]
fn test_builder_api() {
    let teg = build_graph()
        .effect("effect1")
            .effect_type("test_effect")
            .parameter("param1", "value1")
            .domain("test_domain")
            .add()
        .effect("effect2")
            .effect_type("test_effect")
            .parameter("param2", "value2")
            .domain("test_domain")
            .add()
        .resource("resource1")
            .resource_type("test_resource")
            .state(serde_json::json!({"key": "value"}))
            .domain("test_domain")
            .add()
        .relationship()
            .effect_depends_on("effect2", "effect1")
            .effect_continues_to("effect1", "effect2", Some(Condition::Success))
            .done()
        .build();
    
    assert_eq!(teg.effect_nodes.len(), 2);
    assert_eq!(teg.resource_nodes.len(), 1);
    assert!(teg.effect_dependencies.contains_key("effect2"));
    assert!(teg.effect_continuations.contains_key("effect1"));
}

#[test]
fn test_content_addressing() {
    // This test would verify that the content addressing works correctly
    // For now, this is a placeholder until we implement actual content hash computation
    
    let teg = build_graph()
        .effect("effect1")
            .effect_type("test_effect")
            .add()
        .build();
    
    // This just verifies that we have a content hash field
    // The actual content addressing logic would need more complete tests
    assert!(teg.content_hash != ContentHash::default());
}

#[test]
fn test_branching() {
    // Create main fragment
    let mut main_fragment = TEGFragment::new();
    
    let effect1 = EffectNode {
        id: "effect1".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    main_fragment.effect_nodes.insert(effect1.id.clone(), effect1);
    main_fragment.entry_points.push("effect1".to_string());
    main_fragment.exit_points.push("effect1".to_string());
    
    // Create then fragment
    let mut then_fragment = TEGFragment::new();
    
    let effect2 = EffectNode {
        id: "effect2".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    then_fragment.effect_nodes.insert(effect2.id.clone(), effect2);
    then_fragment.entry_points.push("effect2".to_string());
    then_fragment.exit_points.push("effect2".to_string());
    
    // Create else fragment
    let mut else_fragment = TEGFragment::new();
    
    let effect3 = EffectNode {
        id: "effect3".to_string(),
        effect_type: "test_type".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec![],
        resources_accessed: vec![],
        fact_dependencies: vec![],
        domain_id: "test_domain".to_string(),
        metadata: HashMap::new(),
        content_hash: ContentHash::default(),
    };
    
    else_fragment.effect_nodes.insert(effect3.id.clone(), effect3);
    else_fragment.entry_points.push("effect3".to_string());
    else_fragment.exit_points.push("effect3".to_string());
    
    // Create a branch
    let branched = main_fragment.branch(Condition::Success, then_fragment, Some(else_fragment));
    
    assert_eq!(branched.effect_nodes.len(), 3);
    assert!(branched.effect_nodes.contains_key("effect1"));
    assert!(branched.effect_nodes.contains_key("effect2"));
    assert!(branched.effect_nodes.contains_key("effect3"));
} 