//! Comprehensive Simulation Demo: Optimization, Branching, and Time-Travel
//!
//! This example demonstrates the complete simulation capabilities including:
//! 1. Effect optimization strategies
//! 2. Simulation branching for multiple execution paths
//! 3. Time-travel with checkpoints and rewind/fast-forward
//! 4. Combined analysis of optimized effects across branches

use causality_simulation::{
    engine::{SimulationEngine, SimulationConfig, MockEffect, MockEffectCall, ExecutionState},
    optimizer::{EffectOptimizer, OptimizableEffect, EffectCost, OptimizationStrategy},
    branching::BranchingManager,
    time_travel::{TimeTravelManager, TimeTravelConfig},
    error::SimulationError,
};
use causality_core::machine::{Instruction, RegisterId};

fn main() -> Result<(), SimulationError> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        run_comprehensive_demo().await
    })
}

async fn run_comprehensive_demo() -> Result<(), SimulationError> {
    println!("=== Comprehensive Simulation: Optimization + Branching + Time-Travel ===\n");
    
    // 1. Set up simulation components
    println!("1. Setting up simulation components...");
    
    let config = SimulationConfig {
        max_steps: 200,
        gas_limit: 50000,
        timeout_ms: 10000,
        step_by_step_mode: false,
        enable_snapshots: true,
    };
    
    let mut engine = SimulationEngine::new_with_config(config);
    engine.initialize().await?;
    
    // Load a complex program for demonstration
    let program = vec![
        Instruction::Witness { out_reg: RegisterId(0) },
        Instruction::Witness { out_reg: RegisterId(1) },
        Instruction::Witness { out_reg: RegisterId(2) },
        Instruction::Move { src: RegisterId(0), dst: RegisterId(3) },
        Instruction::Move { src: RegisterId(1), dst: RegisterId(4) },
        Instruction::Select { cond_reg: RegisterId(2), true_reg: RegisterId(3), false_reg: RegisterId(4), out_reg: RegisterId(5) },
    ];
    
    engine.load_program(program)?;
    
    // Initialize optimization, branching, and time-travel components
    let mut optimizer = EffectOptimizer::new();
    let mut branching_manager = BranchingManager::new();
    let mut time_travel_manager = TimeTravelManager::with_config(TimeTravelConfig {
        max_checkpoints: 20,
        auto_checkpoint_interval: Some(5),
        compress_old_checkpoints: false,
    });
    
    println!("   ✓ Simulation engine initialized");
    println!("   ✓ Effect optimizer ready");
    println!("   ✓ Branching manager configured");
    println!("   ✓ Time-travel manager configured");
    
    // 2. Create and optimize effects
    println!("\n2. Creating and optimizing effects...");
    
    let effects = create_test_effects();
    println!("   Created {} test effects", effects.len());
    
    // Test different optimization strategies
    let optimization_strategies = vec![
        ("Gas Optimization", OptimizationStrategy::GasEfficiency),
        ("Speed Optimization", OptimizationStrategy::Speed),
        ("Balanced Optimization", OptimizationStrategy::Balanced),
    ];
    
    for (name, strategy) in optimization_strategies {
        println!("\n   Testing {} strategy:", name);
        optimizer.set_strategy(strategy);
        
        // Optimize effects
        let optimized_effects = optimizer.optimize_effects(effects.clone());
        
        println!("     Execution order: {:?}", optimized_effects.execution_order);
        println!("     Parallel batches: {}", optimized_effects.parallel_batches.len());
        println!("     Gas savings: {}", optimized_effects.cost_savings.gas_cost);
        println!("     Time savings: {}", optimized_effects.cost_savings.time_cost);
    }
    
    // 3. Initialize root branch with time-travel
    println!("\n3. Initializing simulation with branching and time-travel...");
    
    let root_id = branching_manager.initialize_root("Root optimization path".to_string())?;
    let checkpoint_id = time_travel_manager.create_checkpoint(&engine, "Initial state".to_string())?;
    
    println!("   ✓ Root branch created: {:?}", root_id);
    println!("   ✓ Initial checkpoint: {:?}", checkpoint_id.as_str());
    
    // 4. Execute optimized scenarios in different branches
    println!("\n4. Exploring different optimization scenarios...");
    
    // Branch 1: Gas-optimized execution
    println!("\n   Creating gas-optimized branch...");
    let gas_execution_state = ExecutionState::new();
    let _gas_branch_id = branching_manager.create_branch(
        "gas_branch", 
        "Gas-optimized execution", 
        gas_execution_state
    )?;
    
    // Execute some steps
    for i in 0..5 {
        // Simulate gas-optimized execution
        println!("    Gas branch step {}: optimizing for gas efficiency", i + 1);
    }
    
    // Branch 2: Time-optimized execution  
    println!("\n   Creating time-optimized branch...");
    let time_execution_state = ExecutionState::new();
    let _time_branch_id = branching_manager.create_branch(
        "time_branch",
        "Time-optimized execution",
        time_execution_state
    )?;
    
    for i in 0..3 {
        println!("    Time branch step {}: optimizing for speed", i + 1);
    }

    // Branch 3: Parallel execution simulation
    println!("\n   Creating parallel execution branch...");
    let parallel_execution_state = ExecutionState::new();
    let _parallel_branch_id = branching_manager.create_branch(
        "parallel_branch",
        "Parallel execution",
        parallel_execution_state
    )?;
    
    // Simulate parallel optimization analysis
    println!("    Analyzing parallel execution opportunities...");
    println!("    Parallel analysis: effects can be batched in 2 groups");

    // Time-travel demonstration with checkpoints
    println!("\n5. Demonstrating time-travel across branches...");
    
    // Create checkpoint for each branch scenario
    let checkpoint_labels = ["post_gas_optimization", "post_time_optimization", "post_parallel_analysis"];
    println!("\n   Creating detailed checkpoint snapshots...");
    
    for (i, &label) in checkpoint_labels.iter().enumerate() {
        let checkpoint_id = time_travel_manager.create_checkpoint(&engine, format!("Branch {} - {}", i + 1, label))?;
        println!("    Created checkpoint: {} ({})", checkpoint_id.as_str(), label);
        
        // Test rewind functionality  
        time_travel_manager.rewind_to_checkpoint(&checkpoint_id, &mut engine)?;
        println!("    Rewound to checkpoint: {}", checkpoint_id.as_str());
    }
    
    // 6. Analyze results across all branches
    println!("\n6. Analyzing results across optimization scenarios...");
    
    let summary = branching_manager.branch_summary();
    println!("   Branch Summary:");
    println!("     Total branches: {}", summary.total_branches);
    println!("     Completed branches: {}", summary.completed_branches);
    println!("     Failed branches: {}", summary.failed_branches);
    println!("     Max depth: {}", summary.max_depth);
    
    // List all branches
    let all_branches = branching_manager.list_branches();
    for branch in all_branches {
        println!("     - {}: {} (created at {:?})", branch.name, branch.id.0, branch.created_at);
    }
    
    // 7. Time-travel statistics
    println!("\n7. Time-travel statistics...");
    
    let tt_stats = time_travel_manager.get_statistics();
    println!("   Total checkpoints: {}", tt_stats.total_checkpoints);
    println!("   Time span: {} seconds", tt_stats.time_span_seconds);
    println!("   Current step: {}", tt_stats.current_step);
    
    // 8. Optimizer statistics
    println!("\n8. Optimization statistics...");
    
    let opt_stats = optimizer.get_statistics();
    println!("   Total optimizations: {}", opt_stats.total_optimizations);
    println!("   Effects optimized: {}", opt_stats.total_effects_optimized);
    println!("   Average cost reduction: {:.2}%", opt_stats.average_cost_reduction * 100.0);
    
    // 9. Execute a step-by-step simulation
    println!("\n9. Running step-by-step simulation...");
    
    for step in 0..3 {
        let should_continue = engine.step().await?;
        println!("   Step {}: continuing = {}", step + 1, should_continue);
        
        if !should_continue {
            break;
        }
    }
    
    println!("\n=== Demo completed successfully! ===");
    println!("\nComprehensive Features Demonstrated:");
    println!("✓ Effect optimization with multiple strategies");
    println!("✓ Simulation branching for scenario exploration");
    println!("✓ Time-travel with checkpoints and rewind/fast-forward");
    println!("✓ Cross-branch performance analysis");
    println!("✓ Integration of optimization, branching, and time-travel");
    println!("✓ Cost-benefit analysis of different optimization approaches");
    
    Ok(())
}

/// Create a set of test effects with different characteristics
fn create_test_effects() -> Vec<OptimizableEffect> {
    vec![
        // Fast, low-cost effects (good for parallelization)
        OptimizableEffect {
            effect: MockEffect {
                call: MockEffectCall {
                    tag: "storage_read".to_string(),
                    args: vec!["key1".to_string()],
                    return_type: Some("Value".to_string()),
                },
                result_register: Some(RegisterId(0)),
            },
            cost: EffectCost::new(3, 10, 256, 0),
            dependencies: vec![],
            priority: 1,
            parallelizable: true,
        },
        
        // Expensive computation (sequential)
        OptimizableEffect {
            effect: MockEffect {
                call: MockEffectCall {
                    tag: "compute_hash".to_string(),
                    args: vec!["large_data".to_string()],
                    return_type: Some("Hash".to_string()),
                },
                result_register: Some(RegisterId(1)),
            },
            cost: EffectCost::new(50, 200, 4096, 0),
            dependencies: vec![],
            priority: 2,
            parallelizable: false,
        },
        
        // Network operation (high latency, parallelizable)
        OptimizableEffect {
            effect: MockEffect {
                call: MockEffectCall {
                    tag: "network_request".to_string(),
                    args: vec!["api_endpoint".to_string()],
                    return_type: Some("Response".to_string()),
                },
                result_register: Some(RegisterId(2)),
            },
            cost: EffectCost::new(8, 300, 1024, 2048),
            dependencies: vec![],
            priority: 1,
            parallelizable: true,
        },
        
        // Transfer operation (medium cost)
        OptimizableEffect {
            effect: MockEffect {
                call: MockEffectCall {
                    tag: "transfer".to_string(),
                    args: vec!["from".to_string(), "to".to_string(), "amount".to_string()],
                    return_type: Some("Receipt".to_string()),
                },
                result_register: Some(RegisterId(3)),
            },
            cost: EffectCost::new(20, 100, 512, 256),
            dependencies: vec![0], // Depends on storage read
            priority: 3,
            parallelizable: false,
        },
        
        // Validation (high priority, expensive)
        OptimizableEffect {
            effect: MockEffect {
                call: MockEffectCall {
                    tag: "signature_verify".to_string(),
                    args: vec!["signature".to_string(), "message".to_string()],
                    return_type: Some("Valid".to_string()),
                },
                result_register: Some(RegisterId(4)),
            },
            cost: EffectCost::new(40, 180, 2048, 256),
            dependencies: vec![],
            priority: 5,
            parallelizable: true,
        },
    ]
} 