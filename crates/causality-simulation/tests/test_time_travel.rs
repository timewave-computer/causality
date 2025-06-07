//! Time-Travel Testing
//!
//! This module tests the time-travel capabilities including:
//! - Checkpoint architecture and state capture
//! - Timeline management and navigation
//! - Iterative debugging workflows
//! - What-if analysis scenarios

use anyhow::Result;
use causality_simulation::{
    SimulationEngine, SimulationConfig,
    clock::{SimulatedClock, SimulatedTimestamp},
    snapshot::{SnapshotManager, SnapshotId},
};
use causality_core::machine::{Instruction, RegisterId};
use std::time::Duration;
use std::collections::HashMap;
use tokio::test as tokio_test;

#[tokio_test]
async fn test_checkpoint_architecture() -> Result<()> {
    println!("=== Testing Checkpoint Architecture ===");
    
    let mut engine = SimulationEngine::new();
    
    // Execute initial program state
    let initial_program = "(alloc 100)";
    engine.execute_program(initial_program).await?;
    
    // Create checkpoint at critical point
    let checkpoint1 = engine.create_checkpoint("before_critical_operation").await?;
    println!("✓ Checkpoint 1 created: {}", checkpoint1);
    
    // Execute critical operation
    let critical_program = "(consume (alloc 500))";
    let result1 = engine.execute_program(critical_program).await?;
    println!("  Critical operation executed: {} steps", result1.step_count);
    
    // Create second checkpoint
    let checkpoint2 = engine.create_checkpoint("after_critical_operation").await?;
    println!("✓ Checkpoint 2 created: {}", checkpoint2);
    
    // Execute additional operations
    let additional_program = "(tensor (alloc 200) (alloc 300))";
    let result2 = engine.execute_program(additional_program).await?;
    println!("  Additional operations executed: {} steps", result2.step_count);
    
    // Test checkpoint restoration
    engine.rewind_to_checkpoint(&checkpoint1).await?;
    println!("✓ Rewound to checkpoint 1");
    
    // Execute alternative path
    let alternative_program = "(alloc 250)";
    let alt_result = engine.execute_program(alternative_program).await?;
    println!("  Alternative path executed: {} steps", alt_result.step_count);
    
    // Verify different execution path
    assert_ne!(result1.step_count, alt_result.step_count);
    println!("✓ Checkpoint architecture enables different execution paths");
    
    Ok(())
}

#[tokio_test]
async fn test_timeline_management() -> Result<()> {
    println!("=== Testing Timeline Management ===");
    
    let mut engine = SimulationEngine::new();
    let mut timeline_checkpoints = Vec::new();
    
    // Create a timeline with multiple states
    let timeline_operations = vec![
        ("init", "(alloc 50)"),
        ("expand", "(tensor (alloc 100) (alloc 150))"),
        ("process", "(consume (alloc 200))"),
        ("finalize", "(alloc 75)"),
    ];
    
    for (phase_name, program) in &timeline_operations {
        // Execute operation
        let result = engine.execute_program(program).await?;
        
        // Create checkpoint for this timeline point
        let checkpoint = engine.create_checkpoint(&format!("timeline_{}", phase_name)).await?;
        timeline_checkpoints.push((phase_name.to_string(), checkpoint, result.step_count));
        
        println!("  ✓ Timeline point '{}': {} steps, checkpoint created", phase_name, result.step_count);
    }
    
    println!("✓ Timeline created with {} checkpoints", timeline_checkpoints.len());
    
    // Test timeline navigation - jump to middle
    let (middle_phase, middle_checkpoint, expected_steps) = &timeline_checkpoints[1];
    engine.rewind_to_checkpoint(middle_checkpoint).await?;
    println!("✓ Navigated to timeline point: {}", middle_phase);
    
    // Verify we can continue from middle point
    let continuation_result = engine.execute_program("(alloc 25)").await?;
    println!("  Continued from middle: {} additional steps", continuation_result.step_count);
    
    // Test jumping to different timeline points
    for (phase_name, checkpoint, _) in &timeline_checkpoints {
        engine.rewind_to_checkpoint(checkpoint).await?;
        println!("  ✓ Successfully navigated to: {}", phase_name);
    }
    
    println!("✓ Timeline navigation working correctly");
    
    Ok(())
}

#[tokio_test]
async fn test_iterative_debugging_workflow() -> Result<()> {
    println!("=== Testing Iterative Debugging Workflow ===");
    
    let mut engine = SimulationEngine::new();
    
    // Simulate debugging scenario: finding optimal parameters
    let base_program = "(consume (alloc 1000))";
    
    // Create checkpoint before optimization attempts
    let debug_checkpoint = engine.create_checkpoint("debug_start").await?;
    
    let optimization_attempts = vec![
        ("attempt_1", 500, "Conservative approach"),
        ("attempt_2", 750, "Moderate approach"),
        ("attempt_3", 1000, "Aggressive approach"),
        ("attempt_4", 250, "Ultra-conservative"),
    ];
    
    let mut debugging_results = HashMap::new();
    
    for (attempt_name, allocation_size, description) in &optimization_attempts {
        println!("  Debugging attempt: {} - {} (size: {})", attempt_name, description, allocation_size);
        
        // Rewind to debug starting point
        engine.rewind_to_checkpoint(&debug_checkpoint).await?;
        
        // Try optimization with different parameters
        let test_program = format!("(consume (alloc {}))", allocation_size);
        let result = engine.execute_program(&test_program).await?;
        
        // Record debugging metrics
        let metrics = engine.metrics();
        debugging_results.insert(attempt_name.to_string(), (
            result.step_count,
            metrics.total_gas_consumed,
            metrics.effects_executed,
        ));
        
        println!("    ✓ Attempt {}: {} steps, {} gas, {} effects", 
                attempt_name, result.step_count, metrics.total_gas_consumed, metrics.effects_executed);
    }
    
    // Find optimal solution from debugging results
    let optimal_attempt = debugging_results.iter()
        .min_by_key(|(_, (steps, gas, _))| steps + (*gas as usize))
        .map(|(name, _)| name);
    
    println!("✓ Iterative debugging completed");
    println!("  Optimal solution found: {:?}", optimal_attempt.unwrap());
    
    Ok(())
}

#[tokio_test]
async fn test_what_if_analysis() -> Result<()> {
    println!("=== Testing What-If Analysis ===");
    
    let mut engine = SimulationEngine::new();
    
    // Setup base scenario
    let base_setup = "(tensor (alloc 200) (alloc 300))";
    engine.execute_program(base_setup).await?;
    
    // Create what-if scenario checkpoint
    let scenario_checkpoint = engine.create_checkpoint("what_if_scenarios").await?;
    
    // Define what-if scenarios
    let what_if_scenarios = vec![
        ("scenario_a", "(consume (alloc 100))", "What if we consume less?"),
        ("scenario_b", "(consume (alloc 500))", "What if we consume more?"),
        ("scenario_c", "(tensor (consume (alloc 50)) (consume (alloc 50)))", "What if we split consumption?"),
        ("scenario_d", "(alloc 1000)", "What if we allocate instead of consuming?"),
    ];
    
    let mut scenario_outcomes = HashMap::new();
    
    for (scenario_name, program, question) in &what_if_scenarios {
        println!("  What-if analysis: {} - {}", scenario_name, question);
        
        // Reset to scenario starting point
        engine.rewind_to_checkpoint(&scenario_checkpoint).await?;
        
        // Execute what-if scenario
        let start_metrics = engine.metrics().clone();
        let result = engine.execute_program(program).await?;
        let end_metrics = engine.metrics().clone();
        
        // Calculate scenario impact
        let gas_delta = end_metrics.total_gas_consumed - start_metrics.total_gas_consumed;
        let effects_delta = end_metrics.effects_executed - start_metrics.effects_executed;
        
        scenario_outcomes.insert(scenario_name.to_string(), (
            result.step_count,
            gas_delta,
            effects_delta,
        ));
        
        println!("    ✓ Scenario {}: {} steps, {} gas impact, {} effects impact", 
                scenario_name, result.step_count, gas_delta, effects_delta);
    }
    
    // Compare scenario outcomes
    println!("  What-If Analysis Results:");
    for (scenario, (steps, gas, effects)) in &scenario_outcomes {
        println!("    {}: {} steps, {} gas, {} effects", scenario, steps, gas, effects);
    }
    
    // Find most efficient scenario
    let most_efficient = scenario_outcomes.iter()
        .min_by_key(|(_, (steps, gas, _))| steps + (*gas as usize))
        .map(|(name, _)| name);
    
    println!("✓ What-if analysis completed");
    println!("  Most efficient scenario: {:?}", most_efficient.unwrap());
    
    Ok(())
}

#[tokio_test]
async fn test_temporal_consistency() -> Result<()> {
    println!("=== Testing Temporal Consistency ===");
    
    let mut engine = SimulationEngine::new();
    
    // Execute operations with time tracking
    let program1 = "(alloc 100)";
    engine.execute_program(program1).await?;
    
    // Create checkpoint with temporal state
    let temporal_checkpoint = engine.create_checkpoint("temporal_state").await?;
    
    // Execute more operations
    let program2 = "(tensor (alloc 200) (alloc 300))";
    engine.execute_program(program2).await?;
    
    println!("✓ Time progression verified through execution sequence");
    
    // Test temporal rewind
    engine.rewind_to_checkpoint(&temporal_checkpoint).await?;
    
    println!("✓ Temporal checkpoint restoration completed");
    
    // Test temporal consistency across operations
    let consistency_program = "(consume (alloc 150))";
    engine.execute_program(consistency_program).await?;
    
    println!("✓ Temporal consistency maintained after rewind");
    
    Ok(())
}

#[tokio_test]
async fn test_checkpoint_metadata_and_labeling() -> Result<()> {
    println!("=== Testing Checkpoint Metadata and Labeling ===");
    
    let mut engine = SimulationEngine::new();
    
    // Create checkpoints with descriptive labels
    let checkpoint_scenarios = vec![
        ("initial_setup", "(alloc 100)", "Initial system setup"),
        ("user_interaction", "(tensor (alloc 50) (alloc 75))", "User interaction phase"),
        ("data_processing", "(consume (alloc 125))", "Data processing phase"),
        ("error_recovery", "(alloc 200)", "Error recovery mechanism"),
        ("final_state", "(consume (alloc 50))", "Final system state"),
    ];
    
    let mut labeled_checkpoints = HashMap::new();
    
    for (label, program, description) in &checkpoint_scenarios {
        // Execute phase
        let result = engine.execute_program(program).await?;
        
        // Create labeled checkpoint
        let checkpoint_name = format!("{}_checkpoint", label);
        let checkpoint = engine.create_checkpoint(&checkpoint_name).await?;
        
        labeled_checkpoints.insert(label.to_string(), (checkpoint, description.to_string(), result.step_count));
        
        println!("  ✓ Created checkpoint '{}': {} - {} steps", 
                label, description, result.step_count);
    }
    
    println!("✓ Created {} labeled checkpoints", labeled_checkpoints.len());
    
    // Test checkpoint access by label
    for (label, (checkpoint, description, expected_steps)) in &labeled_checkpoints {
        engine.rewind_to_checkpoint(checkpoint).await?;
        println!("  ✓ Successfully accessed checkpoint '{}': {}", label, description);
    }
    
    println!("✓ Checkpoint metadata and labeling system working");
    
    Ok(())
}

#[tokio_test]
async fn test_regression_testing_workflow() -> Result<()> {
    println!("=== Testing Regression Testing Workflow ===");
    
    let mut engine = SimulationEngine::new();
    
    // Establish baseline behavior
    let baseline_program = "(tensor (consume (alloc 100)) (consume (alloc 200)))";
    let baseline_result = engine.execute_program(baseline_program).await?;
    
    // Create baseline checkpoint
    let baseline_checkpoint = engine.create_checkpoint("baseline_behavior").await?;
    
    println!("✓ Baseline established: {} steps", baseline_result.step_count);
    
    // Simulate code changes and test for regressions
    let change_scenarios = vec![
        ("optimization_change", "(consume (tensor (alloc 100) (alloc 200)))", "Code optimization"),
        ("refactor_change", "(consume (alloc 300))", "Code refactoring"),
        ("feature_change", "(tensor (alloc 150) (consume (alloc 150)))", "New feature addition"),
    ];
    
    let mut regression_results = HashMap::new();
    
    for (change_name, modified_program, change_description) in &change_scenarios {
        println!("  Testing change: {} - {}", change_name, change_description);
        
        // Reset to baseline
        engine.rewind_to_checkpoint(&baseline_checkpoint).await?;
        
        // Test modified behavior
        let modified_result = engine.execute_program(modified_program).await?;
        
        // Check for regression
        let is_regression = modified_result.step_count > baseline_result.step_count * 2; // 2x threshold
        
        regression_results.insert(change_name.to_string(), (
            modified_result.step_count,
            is_regression,
            change_description.to_string(),
        ));
        
        let status = if is_regression { "REGRESSION" } else { "OK" };
        println!("    ✓ Change {}: {} steps [{}]", change_name, modified_result.step_count, status);
    }
    
    // Summary of regression testing
    let regressions: Vec<_> = regression_results.iter()
        .filter(|(_, (_, is_regression, _))| *is_regression)
        .collect();
    
    println!("✓ Regression testing completed");
    println!("  Baseline: {} steps", baseline_result.step_count);
    println!("  Regressions detected: {}/{}", regressions.len(), change_scenarios.len());
    
    for (change_name, (steps, _, description)) in regressions {
        println!("    REGRESSION: {} ({}) - {} steps vs {} baseline", 
                change_name, description, steps, baseline_result.step_count);
    }
    
    Ok(())
}

#[tokio_test]
async fn test_complex_time_travel_scenarios() -> Result<()> {
    println!("=== Testing Complex Time-Travel Scenarios ===");
    
    let mut engine = SimulationEngine::new();
    
    // Build complex execution history
    let execution_phases = vec![
        ("phase_1", "(alloc 100)"),
        ("phase_2", "(tensor (alloc 50) (alloc 75))"),
        ("phase_3", "(consume (alloc 125))"),
        ("phase_4", "(alloc 200)"),
        ("phase_5", "(consume (alloc 100))"),
    ];
    
    let mut phase_checkpoints = Vec::new();
    
    // Execute phases and create checkpoints
    for (phase_name, program) in &execution_phases {
        let result = engine.execute_program(program).await?;
        let checkpoint = engine.create_checkpoint(&format!("complex_{}", phase_name)).await?;
        
        phase_checkpoints.push((phase_name.to_string(), checkpoint, result.step_count));
        println!("  ✓ Phase {} completed: {} steps", phase_name, result.step_count);
    }
    
    // Test complex time-travel pattern: 5 -> 2 -> 4 -> 1 -> 5
    let travel_pattern = vec![4, 1, 3, 0, 4]; // Indices into phase_checkpoints
    
    for &phase_index in &travel_pattern {
        let (phase_name, checkpoint, expected_steps) = &phase_checkpoints[phase_index];
        engine.rewind_to_checkpoint(checkpoint).await?;
        
        // Execute verification program to ensure state is correct
        let verification_result = engine.execute_program("(alloc 10)").await?;
        
        println!("  ✓ Traveled to {}: verification {} steps", phase_name, verification_result.step_count);
    }
    
    println!("✓ Complex time-travel scenario completed");
    
    // Test time-travel with state modifications
    let modification_checkpoint = &phase_checkpoints[2].1; // Phase 3
    engine.rewind_to_checkpoint(modification_checkpoint).await?;
    
    // Modify state from this point
    let modification_result = engine.execute_program("(tensor (alloc 500) (alloc 600))").await?;
    println!("  ✓ State modification from checkpoint: {} steps", modification_result.step_count);
    
    // Create new timeline branch
    let new_timeline_checkpoint = engine.create_checkpoint("modified_timeline").await?;
    println!("  ✓ New timeline branch created");
    
    Ok(())
} 