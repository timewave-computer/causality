use causality_ir::{
    builder::GraphBuilder,
    graph::edge::{AccessMode, EdgeType, NodeId},
};

fn main() {
    println!("Testing connect_effect_to_resource...");
    
    let mut builder = GraphBuilder::new();
    
    // Add an effect and a resource
    let effect_id = builder.add_effect("effect1", "test_domain");
    let resource_id = builder.add_resource("resource1", "test_resource");
    
    // Connect the effect to the resource
    builder.connect_effect_to_resource(&effect_id, &resource_id, AccessMode::Read);
    
    // Build the graph
    let teg = builder.build().unwrap();
    
    // Verify the connection exists in the edges collection
    let edge_found = teg.edges.values().any(|edge| {
        match (&edge.source, &edge.target, &edge.edge_type) {
            (
                NodeId::Effect(e), 
                NodeId::Resource(r), 
                EdgeType::ResourceAccess { mode }
            ) => *e == effect_id && *r == resource_id && *mode == AccessMode::Read,
            _ => false
        }
    });
    
    if !edge_found {
        panic!("Expected to find an edge connecting effect1 to resource1 with AccessMode::Read");
    }
    
    println!("Test passed successfully!");
} 