//! Comprehensive Simulation Demo: Optimization, Branching, and Time-Travel
//!
//! This example demonstrates the complete simulation capabilities including:
//! 1. Effect optimization strategies
//! 2. Simulation branching for multiple execution paths
//! 3. Time-travel with checkpoints and rewind/fast-forward
//! 4. Combined analysis of optimized effects across branches

use causality_simulation::{
    engine::{SimulationEngine, SimulationConfig, MockEffect, MockEffectCall},
    optimizer::{EffectOptimizer, OptimizableEffect, EffectCost, OptimizationStrategy},
    branching::{BranchingManager, BranchingConfig, BranchStatus},
    time_travel::{TimeTravelManager, TimeTravelConfig},
    error::SimulationError,
};
use causality_core::machine::instruction::{Instruction, RegisterId};

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
    let mut branching_manager = BranchingManager::with_config(BranchingConfig {
        max_branches: 4,
        max_depth: 2,
        auto_prune: true,
    });
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
    let strategies = vec![
        ("Gas Optimization", OptimizationStrategy::MinimizeGasCost),
        ("Time Optimization", OptimizationStrategy::MinimizeTime),
        ("Parallelization", OptimizationStrategy::MaximizeParallelism),
        ("Balanced", OptimizationStrategy::Balanced),
    ];
    
    for (name, strategy) in strategies {
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
    
    let root_id = branching_manager.initialize_root(engine.clone(), "Root optimization path".to_string())?;
    let checkpoint_id = time_travel_manager.create_checkpoint(&engine, "Initial state".to_string())?;
    
    println!("   ✓ Root branch created: {:?}", root_id);
    println!("   ✓ Initial checkpoint: {:?}", checkpoint_id.as_str());
    
    // 4. Execute optimized scenarios in different branches
    println!("\n4. Exploring different optimization scenarios...");
    
    // Branch 1: Gas-optimized execution
    let gas_branch_id = branching_manager.fork_branch("Gas-optimized execution".to_string())?;
    branching_manager.switch_to_branch(&gas_branch_id)?;
    
    if let Some(gas_branch) = branching_manager.active_branch_mut() {
        println!("   Executing gas-optimized scenario...");
        
        // Set gas-optimized configuration
        gas_branch.engine.machine.gas = 30000; // Moderate gas limit
        
        // Execute some steps with gas optimization focus
        for i in 0..3 {
            let step_result = gas_branch.engine.step().await?;
            println!("     Gas-optimized step {}: {} (Gas: {})", 
                    i + 1, step_result, gas_branch.engine.machine.gas);
            
            if i == 1 {
                let checkpoint_id = time_travel_manager.create_checkpoint(
                    &gas_branch.engine, 
                    format!("Gas branch step {}", i + 1)
                )?;
                println!("     Created checkpoint: {}", checkpoint_id.as_str());
            }
            
            if !step_result {
                break;
            }
        }
        
        gas_branch.metadata.status = BranchStatus::Completed;
    }
    
    // Branch 2: Time-optimized execution  
    let time_branch_id = branching_manager.fork_branch("Time-optimized execution".to_string())?;
    branching_manager.switch_to_branch(&time_branch_id)?;
    
    if let Some(time_branch) = branching_manager.active_branch_mut() {
        println!("   Executing time-optimized scenario...");
        
        // Set time-optimized configuration (more gas, parallel focus)
        time_branch.engine.machine.gas = 50000; // High gas limit for speed
        
        // Execute with time optimization focus
        for i in 0..4 {
            let step_result = time_branch.engine.step().await?;
            println!("     Time-optimized step {}: {} (Gas: {})", 
                    i + 1, step_result, time_branch.engine.machine.gas);
            
            if !step_result {
                break;
            }
        }
        
        time_branch.metadata.status = BranchStatus::Completed;
    }
    
    // Branch 3: Parallel-optimized execution
    let parallel_branch_id = branching_manager.fork_branch("Parallel-optimized execution".to_string())?;
    branching_manager.switch_to_branch(&parallel_branch_id)?;
    
    if let Some(parallel_branch) = branching_manager.active_branch_mut() {
        println!("   Executing parallel-optimized scenario...");
        
        // Simulate parallel execution with multiple effect batches
        let checkpoint_before = time_travel_manager.create_checkpoint(
            &parallel_branch.engine, 
            "Before parallel execution".to_string()
        )?;
        
        println!("     ✓ Created checkpoint: {}", checkpoint_before.as_str());
        
        parallel_branch.metadata.status = BranchStatus::Completed;
    }
    
    // Fast-forward simulation to demonstrate time travel (on root branch)
    branching_manager.switch_to_branch(&root_id)?;
    if let Some(root_branch) = branching_manager.active_branch_mut() {
        let target_timestamp = root_branch.engine.clock().now().add_duration(std::time::Duration::from_secs(10));
        let steps_executed = time_travel_manager.fast_forward_to_timestamp(
            target_timestamp, 
            &mut root_branch.engine
        ).await?;
        println!("     ✓ Fast-forwarded {} steps", steps_executed);
    }
    
    // 5. Demonstrate time-travel capabilities
    println!("\n5. Demonstrating time-travel capabilities...");
    
    // List all checkpoints
    let checkpoints = time_travel_manager.list_checkpoints();
    println!("   Available checkpoints:");
    for checkpoint in &checkpoints {
        println!("     - {}: {} (Step {})", 
                checkpoint.id.as_str(), 
                checkpoint.description,
                checkpoint.step_number);
    }
    
    // Rewind to an earlier checkpoint
    if let Some(checkpoint) = checkpoints.first() {
        let checkpoint_id = checkpoint.id.clone();
        println!("\n   Rewinding to checkpoint: {}", checkpoint_id.as_str());
        
        // Switch to a branch for time-travel demo
        branching_manager.switch_to_branch(&root_id)?;
        if let Some(root_branch) = branching_manager.active_branch_mut() {
            time_travel_manager.rewind_to_checkpoint(&checkpoint_id, &mut root_branch.engine)?;
            println!("     ✓ Successfully rewound to earlier state");
            
            // Fast-forward with some steps
            let target_timestamp = root_branch.engine.clock().now().add_duration(std::time::Duration::from_secs(10));
            let steps_executed = time_travel_manager.fast_forward_to_timestamp(
                target_timestamp, 
                &mut root_branch.engine
            ).await?;
            println!("     ✓ Fast-forwarded {} steps", steps_executed);
        }
    }
    
    // 6. Analyze results across all branches
    println!("\n6. Analyzing results across optimization scenarios...");
    
    let summary = branching_manager.branch_summary();
    println!("   Branch Summary:");
    println!("     Total branches: {}", summary.total_branches);
    println!("     Completed branches: {}", summary.completed_branches);
    println!("     Failed branches: {}", summary.failed_branches);
    println!("     Maximum depth: {}", summary.max_depth);
    
    // Compare performance across branches
    println!("\n   Performance Comparison:");
    for branch_id in branching_manager.all_branch_ids() {
        if let Some(branch) = branching_manager.get_branch(&branch_id) {
            let metrics = branch.engine.metrics();
            println!("     {}: {} effects, {} gas consumed", 
                    branch.metadata.description,
                    metrics.effects_executed,
                    metrics.total_gas_consumed);
        }
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