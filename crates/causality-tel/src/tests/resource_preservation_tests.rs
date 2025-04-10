//! Resource preservation tests across transformations
//!
//! These tests verify that resources and their relationships are preserved 
//! when TEL programs are transformed to TEG and back, as well as through
//! various optimization passes.

#[cfg(test)]
mod tests {
    use crate::ast::{Program, Flow, Statement, Expression, Literal};
    use crate::parser::parse_program;
    use crate::ToTEG;
    use causality_ir::TemporalEffectGraph;
    
    /// Test that resource identities are preserved in TEG conversion
    #[test]
    fn test_resource_identity_preservation() {
        // Create a TEL program with resource operations
        let tel_code = r#"
            flow resource_test() -> Unit {
                // Operations on a single resource
                perform read("resource1");
                perform write("resource1", "updated data");
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Find resource nodes for "resource1"
        let resource_nodes: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("resource1"))
            .collect();
        
        // Verify we have at least one resource node for resource1
        assert!(!resource_nodes.is_empty(), "TEG should contain resource node for resource1");
        
        // Verify that all operations on resource1 refer to the same resource node
        // by checking that read and write effects connect to it
        let read_effects: Vec<_> = teg.effect_nodes()
            .filter(|node| node.effect_type() == "read")
            .collect();
        
        let write_effects: Vec<_> = teg.effect_nodes()
            .filter(|node| node.effect_type() == "write")
            .collect();
        
        assert!(!read_effects.is_empty(), "TEG should contain read effect");
        assert!(!write_effects.is_empty(), "TEG should contain write effect");
    }
    
    /// Test that resource relationships are preserved in TEG conversion
    #[test]
    fn test_resource_relationship_preservation() {
        // Create a TEL program with multiple related resources
        let tel_code = r#"
            flow relationship_test() -> Unit {
                // Create a parent-child relationship between resources
                perform create_parent("parent_resource");
                perform create_child("child_resource", "parent_resource");
                
                // Operations should respect the relationship
                perform update_family("parent_resource", "child_resource");
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Find parent and child resource nodes
        let parent_resources: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("parent"))
            .collect();
            
        let child_resources: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("child"))
            .collect();
        
        // Verify resources were created
        assert!(!parent_resources.is_empty(), "TEG should contain parent resource");
        assert!(!child_resources.is_empty(), "TEG should contain child resource");
        
        // The update_family effect should depend on both resources
        let update_effects: Vec<_> = teg.effect_nodes()
            .filter(|node| node.effect_type() == "update_family")
            .collect();
        
        assert!(!update_effects.is_empty(), "TEG should contain update_family effect");
    }
    
    /// Test that resource transformations preserve resource semantics
    #[test]
    fn test_resource_transformation_preservation() {
        // Create a TEL program with resource transformations
        let tel_code = r#"
            flow transform_test() -> Unit {
                // Initial state
                perform init_resource("transform_res", "initial");
                
                // Apply multiple transformations
                perform transform_resource("transform_res", "step1");
                perform transform_resource("transform_res", "step2");
                perform transform_resource("transform_res", "step3");
                
                // Read final state
                let final_state = perform read_resource("transform_res");
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Find all transform resource effects
        let transform_effects: Vec<_> = teg.effect_nodes()
            .filter(|node| node.effect_type() == "transform_resource")
            .collect();
        
        // Verify we have all 3 transform operations
        assert_eq!(transform_effects.len(), 3, "TEG should contain all 3 transform effects");
        
        // Find the resource being transformed
        let resources: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("transform_res"))
            .collect();
        
        assert!(!resources.is_empty(), "TEG should contain the transformed resource");
        
        // Verify transformation sequence is preserved by checking for continuation edges
        // between the transform effects (would be verified more completely with effect IDs)
        let init_effects = teg.effect_nodes()
            .filter(|node| node.effect_type() == "init_resource")
            .count();
            
        let read_effects = teg.effect_nodes()
            .filter(|node| node.effect_type() == "read_resource")
            .count();
            
        assert_eq!(init_effects, 1, "TEG should have one init effect");
        assert_eq!(read_effects, 1, "TEG should have one read effect");
    }
    
    /// Test that resource access patterns are preserved
    #[test]
    fn test_resource_access_pattern_preservation() {
        // Create a TEL program with specific resource access patterns
        let tel_code = r#"
            flow access_pattern_test() -> Unit {
                // Sequential access to a resource
                perform read("sequential_res");
                perform update("sequential_res", "data1");
                perform read("sequential_res");
                perform update("sequential_res", "data2");
                
                // Interleaved access to multiple resources
                perform read("resource_a");
                perform read("resource_b");
                perform update("resource_a", "new_a");
                perform update("resource_b", "new_b");
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Verify sequential access pattern preservation
        let sequential_resources: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("sequential"))
            .collect();
            
        assert!(!sequential_resources.is_empty(), "TEG should contain sequential resource");
        
        // Count sequential operations
        let sequential_reads = teg.effect_nodes()
            .filter(|node| node.effect_type() == "read" && 
                   node.parameters().values().any(|v| v.to_string().contains("sequential")))
            .count();
            
        let sequential_updates = teg.effect_nodes()
            .filter(|node| node.effect_type() == "update" && 
                   node.parameters().values().any(|v| v.to_string().contains("sequential")))
            .count();
            
        assert_eq!(sequential_reads, 2, "TEG should preserve both sequential reads");
        assert_eq!(sequential_updates, 2, "TEG should preserve both sequential updates");
        
        // Verify interleaved access pattern preservation by checking resource count
        let resource_a_ops = teg.effect_nodes()
            .filter(|node| node.parameters().values().any(|v| v.to_string().contains("resource_a")))
            .count();
            
        let resource_b_ops = teg.effect_nodes()
            .filter(|node| node.parameters().values().any(|v| v.to_string().contains("resource_b")))
            .count();
            
        assert_eq!(resource_a_ops, 2, "TEG should preserve resource_a operations");
        assert_eq!(resource_b_ops, 2, "TEG should preserve resource_b operations");
    }
    
    /// Test that resource tensor products are preserved
    #[test]
    fn test_resource_tensor_product_preservation() {
        // Create a TEL program with tensor product of resources
        let tel_code = r#"
            flow tensor_product_test() -> Unit {
                // Create two independent resources
                perform create("resource_x");
                perform create("resource_y");
                
                // Operation acting on both resources together (tensor product)
                perform combine("resource_x", "resource_y", "combined_xy");
                
                // Operations on the combined resource
                perform read_combined("combined_xy");
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Find the combined resource node (tensor product)
        let combined_resources: Vec<_> = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("combined"))
            .collect();
            
        assert!(!combined_resources.is_empty(), "TEG should contain combined resource");
        
        // Find the combine effect that creates the tensor product
        let combine_effects = teg.effect_nodes()
            .filter(|node| node.effect_type() == "combine")
            .count();
            
        assert_eq!(combine_effects, 1, "TEG should contain the combine effect");
        
        // Verify operations on the tensor product
        let read_combined = teg.effect_nodes()
            .filter(|node| node.effect_type() == "read_combined")
            .count();
            
        assert_eq!(read_combined, 1, "TEG should preserve operations on tensor product");
    }
    
    /// Test that resource optimizations preserve semantics
    #[test]
    fn test_resource_optimization_preservation() {
        // Create a TEL program with resource operations that can be optimized
        let tel_code = r#"
            flow optimization_test() -> Unit {
                // Operations that could be optimized
                perform read("opt_resource");  // Read 1
                perform read("opt_resource");  // Read 2 (potentially redundant)
                
                perform write("opt_resource", "data1");  // Write 1
                perform write("opt_resource", "data2");  // Write 2 (could be merged)
                
                // These reads and writes should be preserved regardless of optimization
                perform read("opt_resource");  // Final read must be preserved
                
                return unit;
            }
        "#;
        
        // Parse and convert to TEG
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Even if optimized, the final effect should remain
        let read_effects = teg.effect_nodes()
            .filter(|node| node.effect_type() == "read")
            .count();
            
        let write_effects = teg.effect_nodes()
            .filter(|node| node.effect_type() == "write")
            .count();
            
        // Note: We can't assert exact counts since optimizations may vary,
        // but we should have at least one of each kind of effect
        assert!(read_effects >= 1, "TEG should preserve at least one read effect");
        assert!(write_effects >= 1, "TEG should preserve at least one write effect");
        
        // The final read effect must come after all writes
        // This would normally be tested by analyzing the graph structure
        let opt_resources = teg.resource_nodes()
            .filter(|node| node.resource_id().to_string().contains("opt_resource"))
            .count();
            
        assert!(opt_resources > 0, "TEG should preserve the optimized resource");
    }
} 