//! Demonstration of Simulation Branching for Multi-Path Exploration
//!
//! This example shows how to:
//! 1. Create a simulation with branching capabilities
//! 2. Fork simulation states to explore different execution paths
//! 3. Execute different scenarios in parallel branches
//! 4. Compare results across branches

use causality_simulation::{
    engine::{SimulationEngine, SimulationConfig},
    branching::{BranchingManager, BranchingConfig, BranchStatus},
    error::SimulationError,
};
use causality_core::machine::instruction::{Instruction, RegisterId};

fn main() -> Result<(), SimulationError> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        run_branching_demo().await
    })
}

async fn run_branching_demo() -> Result<(), SimulationError> {
    println!("=== Causality Simulation Branching Demo ===\n");
    
    // 1. Set up simulation engine and branching manager
    println!("1. Setting up simulation with branching capabilities...");
    
    let config = SimulationConfig {
        max_steps: 100,
        gas_limit: 10000,
        timeout_ms: 5000,
        step_by_step_mode: false,
        enable_snapshots: true,
    };
    
    let branching_config = BranchingConfig {
        max_branches: 5,
        max_depth: 3,
        auto_prune: true,
    };
    
    let mut engine = SimulationEngine::new_with_config(config);
    engine.initialize().await?;
    
    // Load a simple program for demonstration
    let program = vec![
        Instruction::Witness { out_reg: RegisterId(0) },
        Instruction::Witness { out_reg: RegisterId(1) },
        Instruction::Witness { out_reg: RegisterId(2) },
        Instruction::Move { src: RegisterId(0), dst: RegisterId(3) },
        Instruction::Move { src: RegisterId(1), dst: RegisterId(4) },
        Instruction::Select { cond_reg: RegisterId(2), true_reg: RegisterId(3), false_reg: RegisterId(4), out_reg: RegisterId(5) },
    ];
    
    engine.load_program(program)?;
    
    let mut branching_manager = BranchingManager::with_config(branching_config);
    
    // Initialize root branch
    let root_id = branching_manager.initialize_root(engine, "Root execution path".to_string())?;
    println!("   ✓ Root branch created: {:?}", root_id);
    
    // 2. Execute some steps in the root branch
    println!("\n2. Executing initial steps in root branch...");
    
    if let Some(root_branch) = branching_manager.active_branch_mut() {
        // Execute one step
        let step_result = root_branch.engine.step().await?;
        println!("   ✓ Root step executed: {}", step_result);
    }
    
    // 3. Fork branches for different scenarios
    println!("\n3. Creating branches for different execution scenarios...");
    
    // Fork for "fast execution" scenario
    let fast_branch_id = branching_manager.fork_branch("Fast execution scenario".to_string())?;
    println!("   ✓ Fast execution branch created: {:?}", fast_branch_id);
    
    // Fork for "resource-constrained" scenario  
    let constrained_branch_id = branching_manager.fork_branch("Resource-constrained scenario".to_string())?;
    println!("   ✓ Resource-constrained branch created: {:?}", constrained_branch_id);
    
    // Fork for "fault injection" scenario
    let fault_branch_id = branching_manager.fork_branch("Fault injection scenario".to_string())?;
    println!("   ✓ Fault injection branch created: {:?}", fault_branch_id);
    
    // 4. Execute different scenarios in each branch
    println!("\n4. Executing different scenarios in parallel branches...");
    
    // Execute fast scenario
    branching_manager.switch_to_branch(&fast_branch_id)?;
    if let Some(fast_branch) = branching_manager.active_branch_mut() {
        println!("   Executing fast scenario...");
        for i in 0..3 {
            let step_result = fast_branch.engine.step().await?;
            println!("     Fast step {}: {}", i + 1, step_result);
            if !step_result {
                break;
            }
        }
        fast_branch.metadata.status = BranchStatus::Completed;
    }
    
    // Execute resource-constrained scenario
    branching_manager.switch_to_branch(&constrained_branch_id)?;
    if let Some(constrained_branch) = branching_manager.active_branch_mut() {
        println!("   Executing resource-constrained scenario...");
        // Simulate resource constraints by limiting gas
        constrained_branch.engine.machine.gas = 50; // Reduce available gas
        
        for i in 0..2 {
            let step_result = constrained_branch.engine.step().await?;
            println!("     Constrained step {}: {}", i + 1, step_result);
            if !step_result {
                break;
            }
        }
        constrained_branch.metadata.status = BranchStatus::Completed;
    }
    
    // Execute fault injection scenario
    branching_manager.switch_to_branch(&fault_branch_id)?;
    if let Some(fault_branch) = branching_manager.active_branch_mut() {
        println!("   Executing fault injection scenario...");
        // Simulate a fault by setting error state
        fault_branch.metadata.status = BranchStatus::Failed("Simulated network failure".to_string());
        println!("     Fault injected: Simulated network failure");
    }
    
    // 5. Analyze results across branches
    println!("\n5. Analyzing results across all branches...");
    
    let summary = branching_manager.branch_summary();
    println!("   Branch Summary:");
    println!("     Total branches: {}", summary.total_branches);
    println!("     Active branches: {}", summary.active_branches);
    println!("     Completed branches: {}", summary.completed_branches);
    println!("     Failed branches: {}", summary.failed_branches);
    println!("     Maximum depth: {}", summary.max_depth);
    
    // 6. Examine individual branch results
    println!("\n6. Examining individual branch results...");
    
    for branch_id in branching_manager.all_branch_ids() {
        if let Some(branch) = branching_manager.get_branch(&branch_id) {
            println!("   Branch: {}", branch.metadata.description);
            println!("     Status: {:?}", branch.metadata.status);
            println!("     Depth: {}", branch.metadata.depth);
            println!("     Steps executed: {}", branch.metadata.steps_executed);
            println!("     Gas remaining: {}", branch.engine.machine.gas);
            
            // Show parent-child relationships
            if let Some(parent_id) = &branch.parent_id {
                println!("     Parent: {:?}", parent_id);
            }
            if !branch.children.is_empty() {
                println!("     Children: {:?}", branch.children);
            }
            println!();
        }
    }
    
    // 7. Demonstrate branch pruning
    println!("7. Demonstrating automatic branch pruning...");
    
    let pruned_count = branching_manager.prune_inactive_branches();
    println!("   ✓ Pruned {} inactive branches", pruned_count);
    
    let final_summary = branching_manager.branch_summary();
    println!("   Final branch count: {}", final_summary.total_branches);
    
    // 8. Switch back to root for final analysis
    println!("\n8. Switching back to root branch for final analysis...");
    
    branching_manager.switch_to_branch(&root_id)?;
    if let Some(root_branch) = branching_manager.active_branch() {
        println!("   Root branch status: {:?}", root_branch.metadata.status);
        println!("   Root branch metrics: {:?}", root_branch.engine.metrics());
    }
    
    println!("\n=== Demo completed successfully! ===");
    println!("\nBranching Features Demonstrated:");
    println!("✓ Simulation state forking and branching");
    println!("✓ Parallel execution of different scenarios");
    println!("✓ Branch status tracking and management");
    println!("✓ Parent-child relationship tracking");
    println!("✓ Automatic pruning of inactive branches");
    println!("✓ Cross-branch analysis and comparison");
    println!("✓ Branch switching and navigation");
    
    Ok(())
} 