//! Simple simulation test without FFI dependencies
//!
//! This test verifies that the basic simulation functionality works
//! without requiring the problematic causality-ffi crate.

use anyhow::Result;
use causality_simulation::{
    SimulationEngine, 
    SimulationConfig,
    BranchingManager,
};

#[tokio::test]
async fn test_basic_simulation_workflow() -> Result<()> {
    println!("=== Basic Simulation Workflow Test ===\n");
    
    // Create simulation engine
    let mut engine = SimulationEngine::new();
    println!("✓ Created simulation engine");
    
    // Test program execution
    let program = "(consume (alloc (tensor 100 200)))";
    let result = engine.execute_program(program).await?;
    println!("✓ Executed program: {} steps", result.step_count);
    
    // Test branching
    let branch_id = engine.create_branch("test_branch").await?;
    println!("✓ Created branch: {}", branch_id);
    
    engine.switch_to_branch(&branch_id).await?;
    println!("✓ Switched to branch");
    
    // Test checkpoints
    let checkpoint_id = engine.create_checkpoint("test_checkpoint").await?;
    println!("✓ Created checkpoint: {}", checkpoint_id);
    
    engine.rewind_to_checkpoint(&checkpoint_id).await?;
    println!("✓ Rewound to checkpoint");
    
    println!("\n=== All basic simulation features working! ===");
    Ok(())
}

#[tokio::test]
async fn test_branching_manager() -> Result<()> {
    println!("=== Branching Manager Test ===\n");
    
    let mut manager = BranchingManager::new();
    println!("✓ Created branching manager");
    
    // Initialize root
    let root_id = manager.initialize_root("Root Branch".to_string())?;
    println!("✓ Initialized root branch: {:?}", root_id);
    
    // Create branches
    let branch1 = manager.create_branch("branch1", "Test Branch 1", 
        causality_simulation::ExecutionState::new())?;
    let branch2 = manager.create_branch("branch2", "Test Branch 2", 
        causality_simulation::ExecutionState::new())?;
    
    println!("✓ Created branches: {:?}, {:?}", branch1, branch2);
    
    // Test branch switching
    manager.switch_to_branch(&branch1)?;
    assert_eq!(manager.current_branch(), Some(&branch1));
    println!("✓ Switched to branch1");
    
    manager.switch_to_branch(&branch2)?;
    assert_eq!(manager.current_branch(), Some(&branch2));
    println!("✓ Switched to branch2");
    
    // Test branch listing
    let branches = manager.list_branches();
    assert_eq!(branches.len(), 3); // root + 2 branches
    println!("✓ Listed {} branches", branches.len());
    
    println!("\n=== Branching manager working correctly! ===");
    Ok(())
}

#[tokio::test]
async fn test_simulation_with_config() -> Result<()> {
    println!("=== Simulation with Configuration Test ===\n");
    
    let config = SimulationConfig {
        max_steps: 100,
        gas_limit: 50000,
        timeout_ms: 5000,
        step_by_step_mode: true,
        enable_snapshots: true,
    };
    
    let mut engine = SimulationEngine::new_with_config(config);
    println!("✓ Created simulation engine with custom config");
    
    // Test step-by-step execution
    let program = vec![
        causality_core::machine::Instruction::Alloc { type_reg: causality_core::machine::RegisterId::new(0), init_reg: causality_core::machine::RegisterId::new(1), 
            output_reg: causality_core::machine::RegisterId::new(0) 
        },
        causality_core::machine::Instruction::Alloc { type_reg: causality_core::machine::RegisterId::new(0), init_reg: causality_core::machine::RegisterId::new(1), 
            output_reg: causality_core::machine::RegisterId::new(1) 
        },
    ];
    
    engine.load_program(program)?;
    println!("✓ Loaded program");
    
    // Execute step by step
    let step1 = engine.step().await?;
    println!("✓ Executed step 1, continue: {}", step1);
    
    let step2 = engine.step().await?;
    println!("✓ Executed step 2, continue: {}", step2);
    
    // Check state progression
    let progression = engine.state_progression();
    assert_eq!(progression.steps.len(), 2);
    println!("✓ State progression tracked: {} steps", progression.steps.len());
    
    println!("\n=== Configuration and step-by-step execution working! ===");
    Ok(())
} 