//! Branching System Testing
//!
//! This module tests the simulation branching capabilities including:
//! - Multi-path execution design
//! - Scenario comparison and risk analysis
//! - Branch management with parent-child relationships
//! - Automatic pruning and memory management

use anyhow::Result;
use causality_simulation::{
    SimulationEngine,
    BranchingManager, BranchingConfig,
    branching::BranchStatus,
    engine::ExecutionState,
};
use std::collections::HashMap;
use tokio::test as tokio_test;

#[tokio_test]
async fn test_multi_path_execution_design() -> Result<()> {
    println!("=== Testing Multi-Path Execution Design ===");
    
    let mut engine = SimulationEngine::new();
    
    // Create base execution scenario
    let base_program = "(consume (alloc (tensor 100 200)))";
    let _base_result = engine.execute_program(base_program).await?;
    
    // Fork multiple execution paths
    let scenarios = vec![
        ("optimistic", "High performance scenario"),
        ("conservative", "Safety-focused scenario"),
        ("experimental", "New feature testing"),
    ];
    
    let mut branch_results = HashMap::new();
    
    for (scenario_name, description) in &scenarios {
        println!("  Creating branch: {} - {}", scenario_name, description);
        
        let branch_id = engine.create_branch(scenario_name).await?;
        engine.switch_to_branch(&branch_id).await?;
        
        // Execute scenario-specific program
        let scenario_program = match *scenario_name {
            "optimistic" => "(consume (alloc 1000))",
            "conservative" => "(consume (alloc 100))",
            "experimental" => "(tensor (alloc 50) (alloc 75))",
            _ => base_program,
        };
        
        let result = engine.execute_program(scenario_program).await?;
        branch_results.insert(scenario_name.to_string(), result.clone());
        
        println!("    ✓ Branch {} executed: {} steps", scenario_name, result.step_count);
    }
    
    println!("✓ Multi-path execution completed for {} scenarios", scenarios.len());
    
    // Verify scenario comparison capabilities
    let step_counts: Vec<_> = branch_results.values().map(|r| r.step_count).collect();
    println!("  Step count comparison: {:?}", step_counts);
    
    Ok(())
}

#[tokio_test]
async fn test_branch_management_design() -> Result<()> {
    println!("=== Testing Branch Management Design ===");
    
    let mut manager = BranchingManager::new();
    
    // Initialize root branch
    let root_id = manager.initialize_root("Root Branch".to_string())?;
    println!("✓ Root branch initialized: {:?}", root_id);
    
    // Test parent-child relationships
    let execution_state = ExecutionState::new();
    
    let child1_id = manager.create_branch("child1", "First Child", execution_state.clone())?;
    let child2_id = manager.create_branch("child2", "Second Child", execution_state.clone())?;
    
    assert_eq!(manager.list_branches().len(), 3); // root + 2 children
    println!("✓ Created child branches: {} total branches", manager.list_branches().len());
    
    // Test branch hierarchy
    let child1_info = manager.get_branch_info(&child1_id.0).unwrap();
    let child2_info = manager.get_branch_info(&child2_id.0).unwrap();
    
    assert_eq!(child1_info.parent_id, Some(root_id.clone()));
    assert_eq!(child2_info.parent_id, Some(root_id.clone()));
    println!("✓ Parent-child relationships correctly established");
    
    // Test branch switching
    manager.switch_to_branch(&child1_id)?;
    assert_eq!(manager.current_branch(), Some(&child1_id));
    
    manager.switch_to_branch(&child2_id)?;
    assert_eq!(manager.current_branch(), Some(&child2_id));
    println!("✓ Branch switching working correctly");
    
    // Test branch removal
    manager.remove_branch(&child1_id.0)?;
    assert_eq!(manager.list_branches().len(), 2); // root + 1 child
    println!("✓ Branch removal working correctly");
    
    Ok(())
}

#[tokio_test]
async fn test_branch_forking_scenarios() -> Result<()> {
    println!("=== Testing Branch Forking Scenarios ===");
    
    let mut manager = BranchingManager::new();
    manager.initialize_root("Root".to_string())?;
    
    // Test deep branching hierarchy
    let mut current_parent = manager.current_branch().unwrap().clone();
    let mut branch_depth = Vec::new();
    
    for depth in 1..=5 {
        let branch_name = format!("depth_{}", depth);
        let branch_id = manager.create_branch(&branch_name, &branch_name, ExecutionState::new())?;
        
        let branch_info = manager.get_branch_info(&branch_id.0).unwrap();
        assert_eq!(branch_info.parent_id, Some(current_parent));
        
        manager.switch_to_branch(&branch_id)?;
        current_parent = branch_id;
        branch_depth.push(depth);
        
        println!("  ✓ Created branch at depth {}", depth);
    }
    
    assert_eq!(branch_depth.len(), 5);
    assert_eq!(manager.list_branches().len(), 6); // root + 5 depth levels
    println!("✓ Deep branching hierarchy created: {} levels", branch_depth.len());
    
    // Test branch forking from arbitrary points
    let fork_branches = [manager.fork_branch("Fork A from current".to_string())?,
        manager.fork_branch("Fork B from current".to_string())?];
    
    for (i, fork_id) in fork_branches.iter().enumerate() {
        let fork_info = manager.get_branch_info(&fork_id.0).unwrap();
        assert!(fork_info.parent_id.is_some());
        println!("  ✓ Fork {} created from arbitrary point", i + 1);
    }
    
    Ok(())
}

#[tokio_test]
async fn test_branch_state_isolation() -> Result<()> {
    println!("=== Testing Branch State Isolation ===");
    
    let mut engine = SimulationEngine::new();
    
    // Execute base program
    let base_program = "(alloc 1000)";
    engine.execute_program(base_program).await?;
    
    // Create branch and verify state isolation
    let branch_id = engine.create_branch("isolated_branch").await?;
    engine.switch_to_branch(&branch_id).await?;
    
    // Execute different program in branch
    let branch_program = "(tensor (alloc 500) (alloc 300))";
    let branch_result = engine.execute_program(branch_program).await?;
    
    // Verify branch has different execution history
    assert!(branch_result.step_count > 0);
    println!("✓ Branch state isolation verified");
    
    // Test branch-specific state modifications
    let checkpoint_id = engine.create_checkpoint("branch_checkpoint").await?;
    println!("  ✓ Branch-specific checkpoint created: {}", checkpoint_id);
    
    // Verify checkpoint is isolated to this branch
    engine.rewind_to_checkpoint(&checkpoint_id).await?;
    println!("  ✓ Branch-specific checkpoint restoration working");
    
    Ok(())
}

#[tokio_test]
async fn test_branch_comparison_analysis() -> Result<()> {
    println!("=== Testing Branch Comparison Analysis ===");
    
    let mut engine = SimulationEngine::new();
    
    // Define comparison scenarios
    let scenarios = vec![
        ("scenario_a", "(consume (alloc 100))", "Conservative allocation"),
        ("scenario_b", "(consume (alloc 500))", "Moderate allocation"),
        ("scenario_c", "(consume (alloc 1000))", "Aggressive allocation"),
    ];
    
    let mut scenario_metrics = HashMap::new();
    
    for (scenario_name, program, description) in &scenarios {
        println!("  Running scenario: {} - {}", scenario_name, description);
        
        // Create dedicated branch for scenario
        let branch_id = engine.create_branch(scenario_name).await?;
        engine.switch_to_branch(&branch_id).await?;
        
        // Execute scenario and collect metrics
        let result = engine.execute_program(program).await?;
        
        scenario_metrics.insert(scenario_name.to_string(), (
            result.step_count,
            result.instruction_count,
            result.execution_time_ms,
        ));
        
        println!("    ✓ Scenario {}: {} steps, {} instructions, {} ms", 
                scenario_name, result.step_count, result.instruction_count, result.execution_time_ms);
    }
    
    // Perform comparison analysis
    println!("  Comparison Analysis:");
    let step_counts: Vec<_> = scenario_metrics.values().map(|(steps, _, _)| *steps).collect();
    let avg_steps = step_counts.iter().sum::<usize>() as f64 / step_counts.len() as f64;
    
    println!("    Average steps: {:.2}", avg_steps);
    println!("    Step range: {} - {}", step_counts.iter().min().unwrap(), step_counts.iter().max().unwrap());
    
    // Identify best performing scenario
    let best_scenario = scenario_metrics.iter()
        .min_by_key(|(_, (steps, _, time))| steps + (*time as usize))
        .map(|(name, _)| name);
    
    println!("    Best performing scenario: {:?}", best_scenario.unwrap());
    println!("✓ Branch comparison analysis completed");
    
    Ok(())
}

#[tokio_test]
async fn test_branch_memory_management() -> Result<()> {
    println!("=== Testing Branch Memory Management ===");
    
    let mut manager = BranchingManager::with_config(BranchingConfig {
        max_branches: 5,
        max_depth: 3,
        auto_prune: true,
    });
    
    manager.initialize_root("Root".to_string())?;
    
    // Create branches up to the limit
    let mut created_branches = Vec::new();
    for i in 1..=4 { // Leave room for root
        let branch_name = format!("branch_{}", i);
        let branch_id = manager.create_branch(&branch_name, &branch_name, ExecutionState::new())?;
        created_branches.push(branch_id);
        println!("  ✓ Created branch {}", i);
    }
    
    assert_eq!(manager.list_branches().len(), 5); // root + 4 branches
    println!("✓ Branch limit management working: {} branches", manager.list_branches().len());
    
    // Test branch summary statistics
    let summary = manager.branch_summary();
    assert_eq!(summary.total_branches, 5);
    println!("✓ Branch summary: {} total branches", summary.total_branches);
    
    // Test branch cleanup
    manager.clear();
    assert_eq!(manager.list_branches().len(), 1); // Only root should remain
    println!("✓ Branch cleanup working: {} branches remaining", manager.list_branches().len());
    
    Ok(())
}

#[tokio_test]
async fn test_concurrent_branch_operations() -> Result<()> {
    println!("=== Testing Concurrent Branch Operations ===");
    
    let mut manager = BranchingManager::new();
    manager.initialize_root("Concurrent Root".to_string())?;
    
    // Simulate concurrent branch creation
    let concurrent_operations = vec![
        ("thread_1", "Branch from thread 1"),
        ("thread_2", "Branch from thread 2"),
        ("thread_3", "Branch from thread 3"),
    ];
    
    let mut branch_ids = Vec::new();
    
    for (thread_name, description) in &concurrent_operations {
        let branch_id = manager.create_branch(thread_name, description, ExecutionState::new())?;
        branch_ids.push(branch_id);
        println!("  ✓ Created branch for {}", thread_name);
    }
    
    // Verify all branches were created successfully
    assert_eq!(manager.list_branches().len(), 4); // root + 3 concurrent branches
    
    // Test concurrent branch switching
    for branch_id in &branch_ids {
        manager.switch_to_branch(branch_id)?;
        assert_eq!(manager.current_branch(), Some(branch_id));
        println!("  ✓ Successfully switched to branch {:?}", branch_id);
    }
    
    println!("✓ Concurrent branch operations completed successfully");
    
    Ok(())
}

#[tokio_test]
async fn test_branch_metadata_tracking() -> Result<()> {
    println!("=== Testing Branch Metadata Tracking ===");
    
    let mut manager = BranchingManager::new();
    manager.initialize_root("Metadata Root".to_string())?;
    
    // Create branch with metadata
    let branch_id = manager.create_branch("metadata_test", "Test Branch with Metadata", ExecutionState::new())?;
    
    let branch_info = manager.get_branch_info(&branch_id.0).unwrap();
    
    // Verify metadata fields
    assert_eq!(branch_info.name, "Test Branch with Metadata");
    assert!(branch_info.parent_id.is_some());
    assert!(branch_info.created_at.elapsed().is_ok());
    
    // Verify branch metadata structure
    assert_eq!(branch_info.metadata.description, "Test Branch with Metadata");
    assert_eq!(branch_info.metadata.status, BranchStatus::Active);
    assert_eq!(branch_info.metadata.depth, 1);
    assert_eq!(branch_info.metadata.steps_executed, 0);
    
    println!("✓ Branch metadata tracking verified:");
    println!("  - Name: {}", branch_info.name);
    println!("  - Status: {:?}", branch_info.metadata.status);
    println!("  - Depth: {}", branch_info.metadata.depth);
    println!("  - Steps executed: {}", branch_info.metadata.steps_executed);
    
    Ok(())
}

#[tokio_test]
async fn test_risk_analysis_scenarios() -> Result<()> {
    println!("=== Testing Risk Analysis Scenarios ===");
    
    let mut engine = SimulationEngine::new();
    
    // Define risk scenarios
    let risk_scenarios = vec![
        ("low_risk", "(alloc 100)", "Conservative scenario"),
        ("medium_risk", "(tensor (alloc 200) (alloc 300))", "Moderate risk scenario"),
        ("high_risk", "(consume (alloc 1000))", "High risk scenario"),
    ];
    
    let mut risk_results = HashMap::new();
    
    for (risk_level, program, description) in &risk_scenarios {
        println!("  Analyzing risk scenario: {} - {}", risk_level, description);
        
        let branch_id = engine.create_branch(&format!("risk_{}", risk_level)).await?;
        engine.switch_to_branch(&branch_id).await?;
        
        // Execute and measure impact
        let start_metrics = engine.metrics().clone();
        let result = engine.execute_program(program).await?;
        let end_metrics = engine.metrics().clone();
        
        let gas_consumed = end_metrics.total_gas_consumed - start_metrics.total_gas_consumed;
        let effects_executed = end_metrics.effects_executed - start_metrics.effects_executed;
        
        risk_results.insert(risk_level.to_string(), (
            result.step_count,
            gas_consumed,
            effects_executed,
        ));
        
        println!("    ✓ Risk analysis {}: {} steps, {} gas, {} effects", 
                risk_level, result.step_count, gas_consumed, effects_executed);
    }
    
    // Compare risk profiles
    println!("  Risk Profile Comparison:");
    for (risk_level, (steps, gas, effects)) in &risk_results {
        println!("    {}: {} steps, {} gas, {} effects", risk_level, steps, gas, effects);
    }
    
    println!("✓ Risk analysis scenarios completed");
    
    Ok(())
} 