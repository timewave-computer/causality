use std::collections::HashMap;
use causality_types::test_content_hash;
use causality_ir::{
    EffectNode, ResourceNode, TemporalEffectGraph, TEGFragment,
    EffectId, ResourceId, DomainId, 
    builder::TEGBuilder,
};

fn main() {
    println!("Testing that builders use test_content_hash...");
    
    // Create a new TEG builder
    let mut builder = TEGBuilder::new();
    
    // Add an effect node
    builder.effect("test-effect")
        .effect_type("test")
        .domain("test-domain")
        .add();
        
    // Add a resource node
    builder.resource("test-resource")
        .resource_type("test")
        .domain("test-domain")
        .add();
        
    // Build the graph
    let teg = builder.build().unwrap();
    
    // Get the effect and resource nodes
    let effect = teg.effect_nodes.get("test-effect").unwrap();
    let resource = teg.resource_nodes.get("test-resource").unwrap();
    
    // Verify they use test_content_hash
    let expected_hash = test_content_hash();
    
    if effect.content_hash.algorithm != expected_hash.algorithm {
        panic!("Effect node content hash algorithm doesn't match expected: {} vs {}", 
               effect.content_hash.algorithm, expected_hash.algorithm);
    }
    
    if effect.content_hash.bytes != expected_hash.bytes {
        panic!("Effect node content hash bytes don't match expected");
    }
    
    if resource.content_hash.algorithm != expected_hash.algorithm {
        panic!("Resource node content hash algorithm doesn't match expected: {} vs {}", 
               resource.content_hash.algorithm, expected_hash.algorithm);
    }
    
    if resource.content_hash.bytes != expected_hash.bytes {
        panic!("Resource node content hash bytes don't match expected");
    }
    
    println!("Test passed successfully!");
} 