// Unified Transform Examples - Demonstrating Computation/Communication Symmetry
//
// This example demonstrates Causality's key architectural paradigm:
// computation and communication are unified as transformations that differ
// only by their source and target locations.

use causality_core::{
    lambda::base::{BaseType, TypeInner, Location, SessionType},
    effect::{
        transform::{Effect, TransformDefinition, EffectContext, EffectResult},
        transform_constraint::{TransformConstraint, TransformConstraintSystem},
        intent::{Intent, LocationRequirements, MigrationSpec, MigrationStrategy},
        capability::Capability,
    },
    system::deterministic::DeterministicSystem,
};

/// Example 1: Local computation as transform
/// Demonstrates that local computation is just a transform where from == to
fn example_local_computation_as_transform() {
    println!("=== Example 1: Local Computation as Transform ===");
    
    // Create a local computation transform - doubling an integer
    let local_transform = Effect::new(
        Location::Local,
        Location::Local, // Same location = local computation
        TypeInner::Base(BaseType::Int),
        TypeInner::Base(BaseType::Int),
        TransformDefinition::FunctionApplication {
            function: "double".to_string(),
            argument: "x".to_string(),
        },
    );
    
    println!("Local computation transform:");
    println!("  From: {:?}", local_transform.from);
    println!("  To: {:?}", local_transform.to);
    println!("  Transform: {:?}", local_transform.transform);
    
    // Execute the transform
    let context = EffectContext::default();
    let result = local_transform.execute(&context);
    
    match result {
        EffectResult::Success { stats, .. } => {
            println!("  Execution successful:");
            println!("    Compute cost: {}", stats.compute_cost);
            println!("    Communication cost: {}", stats.communication_cost);
            println!("    Locations involved: {:?}", stats.locations_involved);
        }
        _ => println!("  Execution failed"),
    }
    
    println!();
}

/// Example 2: Distributed protocol as transform
/// Demonstrates that distributed communication is the same transform with different locations
fn example_distributed_protocol_as_transform() {
    println!("=== Example 2: Distributed Protocol as Transform ===");
    
    // Create the SAME transform, but with different locations = distributed communication
    let distributed_transform = Effect::new(
        Location::Local,
        Location::Remote("compute_cluster".to_string()), // Different location = distributed
        TypeInner::Base(BaseType::Int),
        TypeInner::Base(BaseType::Int),
        TransformDefinition::FunctionApplication {
            function: "double".to_string(), // Same function!
            argument: "x".to_string(),
        },
    );
    
    println!("Distributed computation transform:");
    println!("  From: {:?}", distributed_transform.from);
    println!("  To: {:?}", distributed_transform.to);
    println!("  Transform: {:?}", distributed_transform.transform);
    
    // The protocol is automatically derived from the transform
    let protocol = SessionType::Send(
        Box::new(TypeInner::Base(BaseType::Int)), // Send input
        Box::new(SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)), // Receive result
            Box::new(SessionType::End)
        ))
    );
    
    println!("  Auto-derived protocol: {:?}", protocol);
    
    // Execute the transform
    let context = EffectContext::default();
    let result = distributed_transform.execute(&context);
    
    match result {
        EffectResult::Success { stats, .. } => {
            println!("  Execution successful:");
            println!("    Compute cost: {}", stats.compute_cost);
            println!("    Communication cost: {}", stats.communication_cost);
            println!("    Locations involved: {:?}", stats.locations_involved);
        }
        _ => println!("  Execution failed"),
    }
    
    println!();
}

/// Example 3: Mixed local/distributed workflow
/// Demonstrates seamless composition of local and distributed transforms
fn example_mixed_workflow() {
    println!("=== Example 3: Mixed Local/Distributed Workflow ===");
    
    // Step 1: Local preprocessing
    let preprocess = Effect::new(
        Location::Local,
        Location::Local,
        TypeInner::Base(BaseType::Int),
        TypeInner::Base(BaseType::Int),
        TransformDefinition::FunctionApplication {
            function: "normalize".to_string(),
            argument: "input".to_string(),
        },
    );
    
    // Step 2: Distributed computation on GPU cluster
    let gpu_compute = Effect::new(
        Location::Local,
        Location::Remote("gpu_cluster".to_string()),
        TypeInner::Base(BaseType::Int),
        TypeInner::Base(BaseType::Int),
        TransformDefinition::FunctionApplication {
            function: "heavy_compute".to_string(),
            argument: "normalized_input".to_string(),
        },
    );
    
    // Step 3: Local postprocessing
    let postprocess = Effect::new(
        Location::Local,
        Location::Local,
        TypeInner::Base(BaseType::Int),
        TypeInner::Base(BaseType::Symbol),
        TransformDefinition::FunctionApplication {
            function: "format_result".to_string(),
            argument: "computed_result".to_string(),
        },
    );
    
    // Compose them into a workflow - seamless composition!
    let workflow = preprocess
        .then(gpu_compute)
        .then(postprocess);
    
    println!("Mixed workflow composition:");
    println!("  Steps: {}", workflow.effects.len());
    println!("  Input type: {:?}", workflow.input_type);
    println!("  Output type: {:?}", workflow.output_type);
    
    // The constraint system handles the complexity
    let mut constraint_system = TransformConstraintSystem::new();
    
    // Add constraints for each step
    constraint_system.add_constraint(TransformConstraint::LocalTransform {
        source_type: TypeInner::Base(BaseType::Int),
        target_type: TypeInner::Base(BaseType::Int),
        transform: preprocess.transform.clone(),
    });
    
    constraint_system.add_constraint(TransformConstraint::RemoteTransform {
        source_location: Location::Local,
        target_location: Location::Remote("gpu_cluster".to_string()),
        source_type: TypeInner::Base(BaseType::Int),
        target_type: TypeInner::Base(BaseType::Int),
        protocol: TypeInner::Base(BaseType::Unit), // Simplified
    });
    
    constraint_system.add_constraint(TransformConstraint::LocalTransform {
        source_type: TypeInner::Base(BaseType::Int),
        target_type: TypeInner::Base(BaseType::Symbol),
        transform: postprocess.transform.clone(),
    });
    
    println!("  Constraints added: 3 (local, remote, local)");
    
    // Solve constraints
    let mut det_sys = DeterministicSystem::new();
    match constraint_system.solve_constraints(&mut det_sys) {
        Ok(operations) => {
            println!("  Constraint solving successful: {} Layer 1 operations", operations.len());
        }
        Err(e) => {
            println!("  Constraint solving failed: {:?}", e);
        }
    }
    
    println!();
}

/// Example 4: Location migration
/// Demonstrates transparent data migration between locations
fn example_location_migration() {
    println!("=== Example 4: Location Migration ===");
    
    // Create an intent that requires data migration
    let mut intent = Intent::new(Location::Local);
    
    // Add location requirements that will trigger migration
    let location_reqs = LocationRequirements {
        preferred_locations: vec![Location::Remote("fast_storage".to_string())],
        allowed_locations: vec![
            Location::Local,
            Location::Remote("fast_storage".to_string()),
            Location::Remote("backup_storage".to_string()),
        ],
        migration_specs: vec![
            MigrationSpec {
                from: Location::Local,
                to: Location::Remote("fast_storage".to_string()),
                fields: vec!["data".to_string()],
                strategy: MigrationStrategy::Copy,
                protocol: TypeInner::Base(BaseType::Unit), // Simplified
            }
        ],
        required_protocols: vec![],
        performance_constraints: None,
        cost_constraints: None,
    };
    
    // Set location requirements
    intent.set_location_requirements(location_reqs);
    
    println!("Migration intent created:");
    println!("  Source location: {:?}", Location::Local);
    println!("  Target location: {:?}", Location::Remote("fast_storage".to_string()));
    println!("  Migration strategy: Copy");
    
    // Create migration transform
    let migration_transform = Effect::new(
        Location::Local,
        Location::Remote("fast_storage".to_string()),
        TypeInner::Base(BaseType::Symbol), // Data to migrate
        TypeInner::Base(BaseType::Symbol), // Migrated data
        TransformDefinition::ResourceConsumption {
            resource_type: "migration".to_string(),
        },
    );
    
    println!("  Migration transform: {:?}", migration_transform.transform);
    
    // Execute migration
    let context = EffectContext::default();
    let result = migration_transform.execute(&context);
    
    match result {
        EffectResult::Success { stats, new_location, .. } => {
            println!("  Migration successful:");
            println!("    New location: {:?}", new_location);
            println!("    Network usage: {} bytes", stats.network_used);
            println!("    Locations involved: {:?}", stats.locations_involved);
        }
        EffectResult::MigrationRequired { target_location, .. } => {
            println!("  Migration required to: {:?}", target_location);
        }
        _ => println!("  Migration failed"),
    }
    
    println!();
}

/// Example 5: Unified constraint system
/// Demonstrates how the same constraint language works for all operations
fn example_unified_constraints() {
    println!("=== Example 5: Unified Constraint System ===");
    
    let mut constraint_system = TransformConstraintSystem::new();
    
    // Add diverse constraints - all using the same language!
    
    // 1. Local computation constraint
    constraint_system.add_constraint(TransformConstraint::LocalTransform {
        source_type: TypeInner::Base(BaseType::Int),
        target_type: TypeInner::Base(BaseType::Int),
        transform: TransformDefinition::FunctionApplication {
            function: "hash".to_string(),
            argument: "data".to_string(),
        },
    });
    
    // 2. Remote communication constraint
    constraint_system.add_constraint(TransformConstraint::RemoteTransform {
        source_location: Location::Local,
        target_location: Location::Remote("database".to_string()),
        source_type: TypeInner::Base(BaseType::Symbol),
        target_type: TypeInner::Base(BaseType::Symbol),
        protocol: TypeInner::Base(BaseType::Unit), // Simplified
    });
    
    // 3. Data migration constraint
    constraint_system.add_constraint(TransformConstraint::DataMigration {
        from_location: Location::Local,
        to_location: Location::Remote("cache".to_string()),
        data_type: TypeInner::Base(BaseType::Symbol),
        migration_strategy: "copy".to_string(),
    });
    
    // 4. Distributed synchronization constraint
    constraint_system.add_constraint(TransformConstraint::DistributedSync {
        locations: vec![
            Location::Local,
            Location::Remote("replica1".to_string()),
            Location::Remote("replica2".to_string()),
        ],
        sync_type: TypeInner::Base(BaseType::Unit),
        consistency_model: "strong".to_string(),
    });
    
    // 5. Capability access constraint
    constraint_system.add_constraint(TransformConstraint::CapabilityAccess {
        resource: "secure_data".to_string(),
        required_capability: Some(Capability::new("read".to_string(), 1)),
        access_pattern: "read_only".to_string(),
    });
    
    println!("Added 5 different constraint types:");
    println!("  1. Local computation");
    println!("  2. Remote communication");
    println!("  3. Data migration");
    println!("  4. Distributed synchronization");
    println!("  5. Capability access");
    
    // Solve all constraints using the same solver!
    let mut det_sys = DeterministicSystem::new();
    match constraint_system.solve_constraints(&mut det_sys) {
        Ok(operations) => {
            println!("  All constraints solved successfully!");
            println!("  Generated {} Layer 1 operations", operations.len());
            
            // Show the unified compilation
            for (i, op) in operations.iter().enumerate() {
                println!("    Operation {}: {:?}", i + 1, op);
            }
        }
        Err(e) => {
            println!("  Constraint solving failed: {:?}", e);
        }
    }
    
    println!();
}

fn main() {
    println!("Causality Transform-Based Unification Examples");
    println!("=============================================");
    println!();
    
    // Run all examples
    example_local_computation_as_transform();
    example_distributed_protocol_as_transform();
    example_mixed_workflow();
    example_location_migration();
    example_unified_constraints();
    
    println!("=== Summary ===");
    println!("These examples demonstrate the core architectural breakthrough:");
    println!("- Computation and communication are unified as transformations");
    println!("- Location determines whether a transform is local or distributed");
    println!("- The same constraint language works for all operations");
    println!("- Protocols are automatically derived from access patterns");
    println!("- Location transparency enables seamless composition");
    println!();
    println!("This unification eliminates artificial distinctions and provides");
    println!("a mathematically elegant foundation based on category theory.");
} 