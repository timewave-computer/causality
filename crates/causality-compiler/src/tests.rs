//! Test Suite for Causality Compiler
//!
//! This module contains tests for the compiler functionality including
//! project management, domain registration, and program generation.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use crate::project::{domain_id, ProgramProject};
use crate::registry::ProgramRegistry;
use causality_types::primitive::string::Str;
use causality_types::primitive::ids::SubgraphId;
use causality_types::AsNode;

// TEL and core types
use causality_types::primitive::ids::{DomainId, EdgeId, IntentId, NodeId};
use causality_types::resource::Resource;
use causality_types::tel::{Edge as TelEdge, Effect};

//-----------------------------------------------------------------------------
// Project Tests
//-----------------------------------------------------------------------------

#[test]
fn test_program_project_creation() {
    // Create a new project
    let project = ProgramProject::with_name("Test Project");

    // Check project properties
    assert_eq!(project.name, Some("Test Project".to_string()));
    assert!(project.domains.is_empty());
}

//-----------------------------------------------------------------------------
// Domain Tests
//-----------------------------------------------------------------------------

#[test]
fn test_domain_registration() {
    // Create a new project
    let mut project = ProgramProject::with_name("Test Project");

    // Register domains
    let domain1 = domain_id([1; 32]);
    let domain2 = domain_id([2; 32]);

    project.register_domain(domain1).unwrap();
    project.register_domain(domain2).unwrap();

    // Check domain registration
    assert_eq!(project.domains.len(), 2);
    assert!(project.get_domain(&domain1).is_some());
    assert!(project.get_domain(&domain2).is_some());
}

//-----------------------------------------------------------------------------
// AST Tests
//-----------------------------------------------------------------------------

#[test]
fn test_ast_registration() {
    // Create a new project
    let mut project = ProgramProject::with_name("Test Project");

    // Register a domain
    let domain = domain_id([1; 32]);
    project.register_domain(domain).unwrap();

    // Register ASTs
    let ast_id1 = "ast1";
    let ast_id2 = "ast2";

    project.register_ast(&domain, ast_id1).unwrap();
    project.register_ast(&domain, ast_id2).unwrap();

    // Check AST registration
    let asts = project.list_asts(&domain).unwrap();
    assert_eq!(asts.len(), 2);
    assert!(asts.contains(&ast_id1.to_string()));
    assert!(asts.contains(&ast_id2.to_string()));
}

//-----------------------------------------------------------------------------
// Isolation Tests
//-----------------------------------------------------------------------------

#[test]
fn test_domain_isolation() {
    // Create a new project
    let mut project = ProgramProject::with_name("Test Project");

    // Register domains
    let domain1 = DomainId::new([1; 32]); // Use direct constructor
    let domain2 = DomainId::new([2; 32]);

    project.register_domain(domain1).unwrap();
    project.register_domain(domain2).unwrap();

    // Create nodes and add them to conceptual subgraphs in each domain
    let node1_d1 = Effect {
        id: NodeId::new([1; 32]),
        domain: domain1,
        intent_id: IntentId::new([0; 32]),
        effect_type: Str::from("effect_type_0"),
        ..Default::default()
    };
    let node2_d1 = Resource {
        id: causality_types::primitive::ids::ResourceId::new([2; 32]),
        domain: domain1,
        ..Default::default()
    };

    project.add_tel_node(&node1_d1).unwrap();
    project.add_tel_node(&node2_d1).unwrap(); // Assuming Resource implements AsNode and is in TelNodeTypes

    let subgraph_id1 = SubgraphId::new("subgraph1_d1");
    project
        .get_domain_mut(&domain1)
        .unwrap()
        .add_node_to_subgraph(subgraph_id1.clone(), node1_d1.id);
    project
        .get_domain_mut(&domain1)
        .unwrap()
        .add_node_to_subgraph(subgraph_id1.clone(), node2_d1.to_node_id());

    let node1_d2 = Effect {
        id: NodeId::new([3; 32]),
        domain: domain2,
        intent_id: IntentId::new([0; 32]),
        effect_type: Str::from("effect_type_0"),
        ..Default::default()
    };
    project.add_tel_node(&node1_d2).unwrap();

    let subgraph_id2 = SubgraphId::new("subgraph1_d2");
    project
        .get_domain_mut(&domain2)
        .unwrap()
        .add_node_to_subgraph(subgraph_id2.clone(), node1_d2.id);

    // Check domain isolation for subgraphs (now conceptual, identified by SubgraphId)
    let subgraphs1_ids: Vec<SubgraphId> = project.list_subgraphs(&domain1).unwrap();
    assert_eq!(subgraphs1_ids.len(), 1);
    assert_eq!(subgraphs1_ids[0], subgraph_id1);

    let subgraphs2_ids: Vec<SubgraphId> = project.list_subgraphs(&domain2).unwrap();
    assert_eq!(subgraphs2_ids.len(), 1);
    assert_eq!(subgraphs2_ids[0], subgraph_id2);

    assert_ne!(subgraphs1_ids[0], subgraphs2_ids[0]);

    // Verify nodes are in correct subgraphs
    let domain1_module = project.get_domain(&domain1).unwrap();
    assert!(domain1_module
        .subgraph_nodes
        .get(&subgraph_id1)
        .unwrap()
        .contains(&node1_d1.id));
    assert!(domain1_module
        .subgraph_nodes
        .get(&subgraph_id1)
        .unwrap()
        .contains(&node2_d1.to_node_id()));
    assert!(domain1_module.subgraph_nodes.get(&subgraph_id2).is_none());

    let domain2_module = project.get_domain(&domain2).unwrap();
    assert!(domain2_module
        .subgraph_nodes
        .get(&subgraph_id2)
        .unwrap()
        .contains(&node1_d2.id));
    assert!(domain2_module.subgraph_nodes.get(&subgraph_id1).is_none());

    // 1. Register ASTs in different domains
    project.register_ast(&domain1, "ast_domain1").unwrap();
    project.register_ast(&domain2, "ast_domain2").unwrap();

    // Verify ASTs are correctly isolated by domain
    let asts_domain1 = project.list_asts(&domain1).unwrap();
    let asts_domain2 = project.list_asts(&domain2).unwrap();

    assert_eq!(asts_domain1.len(), 1);
    assert_eq!(asts_domain2.len(), 1);
    assert!(asts_domain1.contains(&"ast_domain1".to_string()));
    assert!(asts_domain2.contains(&"ast_domain2".to_string()));
    assert!(!asts_domain1.contains(&"ast_domain2".to_string()));
    assert!(!asts_domain2.contains(&"ast_domain1".to_string()));

    // 2. Test domain-specific attributes
    let domain_module1 = project.get_domain_mut(&domain1).unwrap();
    domain_module1.set_attribute("key1", "value1");

    let domain_module2 = project.get_domain_mut(&domain2).unwrap();
    domain_module2.set_attribute("key2", "value2");

    // Verify attributes are correctly isolated
    assert_eq!(
        project.get_domain(&domain1).unwrap().get_attribute("key1"),
        Some("value1")
    );
    assert_eq!(
        project.get_domain(&domain2).unwrap().get_attribute("key2"),
        Some("value2")
    );
    assert_eq!(
        project.get_domain(&domain1).unwrap().get_attribute("key2"),
        None
    );
    assert_eq!(
        project.get_domain(&domain2).unwrap().get_attribute("key1"),
        None
    );
}

//-----------------------------------------------------------------------------
// Reference Mapping Tests
//-----------------------------------------------------------------------------

#[test]
fn test_ast_reference_mapping() {
    // Create a new project
    let mut project = ProgramProject::with_name("Test Project");

    // Register a domain
    let domain = domain_id([1; 32]);
    project.register_domain(domain).unwrap();

    // Register ASTs
    let ast_id = "test_ast";
    project.register_ast(&domain, ast_id).unwrap();

    // Map AST to effects
    let effect_id1 = "effect1";
    let effect_id2 = "effect2";

    project
        .map_domain_ast_to_effect(&domain, ast_id, effect_id1)
        .unwrap();
    project
        .map_domain_ast_to_effect(&domain, ast_id, effect_id2)
        .unwrap();

    // Verify AST to effect mapping
    let effects = project.get_domain_effects_for_ast(&domain, ast_id).unwrap();
    assert_eq!(effects.len(), 2);
    assert!(effects.contains(&effect_id1.to_string()));
    assert!(effects.contains(&effect_id2.to_string()));

    // Test non-existent mappings
    let non_existent_ast = "non_existent_ast";
    let result = project.get_domain_effects_for_ast(&domain, non_existent_ast);
    assert!(result.is_err());

    // Test cross-domain mapping (should fail or return empty)
    let domain2 = domain_id([2; 32]);
    project.register_domain(domain2).unwrap();

    // Register an AST in domain2
    let ast_id2 = "domain2_ast";
    project.register_ast(&domain2, ast_id2).unwrap();

    // Map AST to effect in domain2
    let effect_id3 = "effect3";
    project
        .map_domain_ast_to_effect(&domain2, ast_id2, effect_id3)
        .unwrap();

    // Verify the AST in domain1 doesn't map to the effect in domain2
    let effects1 = project.get_domain_effects_for_ast(&domain, ast_id).unwrap();
    assert!(!effects1.contains(&effect_id3.to_string()));

    // Verify the AST in domain2 doesn't map to effects in domain1
    let effects2 = project
        .get_domain_effects_for_ast(&domain2, ast_id2)
        .unwrap();
    assert!(!effects2.contains(&effect_id1.to_string()));
    assert!(!effects2.contains(&effect_id2.to_string()));
}

//-----------------------------------------------------------------------------
// Program Generation Tests
//-----------------------------------------------------------------------------

#[test]
fn test_program_generation() {
    // Create a new project
    let mut project = ProgramProject::with_name("Test Program Gen");
    let mut registry = ProgramRegistry::new();

    // Register a domain
    let domain = DomainId::new([1; 32]);
    project.register_domain(domain).unwrap();

    let subgraph_id_test = SubgraphId::new("test_prog_subgraph");

    // Add TEL nodes
    let effect_node1 = Effect {
        id: NodeId::new([10; 32]),
        domain,
        intent_id: IntentId::new([1; 32]),
        effect_type: Str::from("effect_type_1"),
        ..Default::default()
    };
    let resource_node2 = Resource {
        id: causality_types::primitive::ids::ResourceId::new([11; 32]),
        domain,
        ..Default::default()
    };

    project.add_tel_node(&effect_node1).unwrap();
    project.add_tel_node(&resource_node2).unwrap();

    // Add nodes to conceptual subgraph
    project
        .get_domain_mut(&domain)
        .unwrap()
        .add_node_to_subgraph(subgraph_id_test.clone(), effect_node1.id);
    project
        .get_domain_mut(&domain)
        .unwrap()
        .add_node_to_subgraph(subgraph_id_test.clone(), resource_node2.to_node_id());

    // Create edge between effect and resource
    let edge1 = TelEdge {
        id: EdgeId::new([10; 32]),
        source: effect_node1.id,
        target: resource_node2.to_node_id(),
        kind: "DataOutput".to_string(), // Use string instead of enum variant
    };
    
    project.add_tel_edge(&edge1).unwrap();

    // Generate the program
    let program_id = project.generate_program(&mut registry).unwrap();

    // Verify program registration and ID
    // The program_id will be ProgramId::default() because generate_program uses project.id
    // which is initialized to default and not currently recalculated based on content.
    //assert_eq!(program_id, crate::ids::ProgramId::default());
    assert!(registry.contains(&program_id));

    let registered_program = registry.get(&program_id).unwrap();
    assert_eq!(registered_program.id, program_id);
    assert!(
        registered_program.circuit_ids.is_empty(),
        "Circuit IDs should be empty for now"
    );
}

//-----------------------------------------------------------------------------
// Concurrency Tests
//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_concurrent_access_to_registry() {
    // This is a placeholder for a proper concurrent registry access test
    // In a real test, we would use Arc<Mutex<ProgramRegistry>> and spawn multiple tasks
    let _registry = ProgramRegistry::new();

    // Example of how a real concurrent test would be structured:
    // let registry = Arc::new(tokio::sync::Mutex::new(ProgramRegistry::new()));
    // let mut tasks = vec![];
    //
    // for i in 0..5 {
    //     let registry_clone = Arc::clone(&registry);
    //     tasks.push(tokio::spawn(async move {
    //         let mut reg = registry_clone.lock().await;
    //
    //         // Simulate registration operation
    //         let prog_id = crate::ids::ProgramId::new([i as u8; 32]);
    //         let dummy_program = crate::program::Program {
    //             id: prog_id,
    //             circuit_ids: std::collections::HashSet::new()
    //         };
    //         reg.insert(prog_id, dummy_program).expect("Insert failed");
    //     }));
    // }
    //
    // futures::future::join_all(tasks).await;
    //
    // // Verify results
    // let final_registry = registry.lock().await;
    // assert_eq!(final_registry.list_program_ids().len(), 5);
}
