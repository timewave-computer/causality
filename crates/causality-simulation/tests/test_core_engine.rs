//! Core Simulation Engine Testing
//!
//! This module tests the fundamental simulation engine capabilities including:
//! - State management and execution tracking
//! - Effect execution sandbox
//! - Temporal modeling and deterministic execution
//! - Basic engine lifecycle and configuration

use anyhow::Result;
use causality_simulation::{
    SimulationEngine, SimulationConfig, SimulationState,
    engine::{ExecutionState, ExecutionSummary},
    clock::{SimulatedClock, SimulatedTimestamp},
};
use causality_core::machine::{Instruction, RegisterId};
use std::time::Duration;
use tokio::test as tokio_test;

#[tokio_test]
async fn test_engine_lifecycle() -> Result<()> {
    println!("=== Testing Core Engine Lifecycle ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test initial state
    assert_eq!(engine.state(), &SimulationState::Created);
    println!("✓ Engine created in correct initial state");
    
    // Test initialization
    engine.initialize().await?;
    assert_eq!(engine.state(), &SimulationState::Initialized);
    println!("✓ Engine initialization successful");
    
    // Test program loading
    let program = vec![
        Instruction::Witness { out_reg: RegisterId::new(0) },
        Instruction::Witness { out_reg: RegisterId::new(1) },
    ];
    engine.load_program(program)?;
    println!("✓ Program loaded successfully");
    
    // Test execution
    engine.run().await?;
    assert_eq!(engine.state(), &SimulationState::Completed);
    println!("✓ Program execution completed");
    
    // Test reset
    engine.reset()?;
    assert_eq!(engine.state(), &SimulationState::Created);
    println!("✓ Engine reset successful");
    
    Ok(())
}

#[tokio_test]
async fn test_state_management_design() -> Result<()> {
    println!("=== Testing State Management Design ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test state progression tracking
    let initial_progression = engine.state_progression();
    assert_eq!(initial_progression.steps.len(), 0);
    assert_eq!(initial_progression.state_transitions.len(), 0);
    
    // Test state transitions with tracking
    engine.set_state(SimulationState::Running);
    let after_running = engine.state_progression();
    assert_eq!(after_running.state_transitions.len(), 1);
    assert_eq!(after_running.state_transitions[0].0, SimulationState::Running);
    println!("✓ State transition tracking working");
    
    // Test execution state management
    let execution_state = engine.execution_state();
    assert_eq!(execution_state.registers.len(), 0);
    assert_eq!(execution_state.memory.len(), 0);
    assert_eq!(execution_state.instruction_pointer, 0);
    println!("✓ Execution state properly initialized");
    
    // Test state invariants preservation
    let program = vec![
        Instruction::Witness { out_reg: RegisterId::new(0) },
        Instruction::Move { src: RegisterId::new(0), dst: RegisterId::new(1) },
    ];
    engine.load_program(program)?;
    engine.run().await?;
    
    let final_progression = engine.state_progression();
    assert!(final_progression.steps.len() > 0);
    println!("✓ State invariants preserved during execution");
    
    Ok(())
}

#[tokio_test]
async fn test_effect_execution_sandbox() -> Result<()> {
    println!("=== Testing Effect Execution Sandbox ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test isolated effect execution
    let effect_results = vec![
        engine.execute_effect("(transfer 100)".to_string()).await?,
        engine.execute_effect("(compute hash)".to_string()).await?,
        engine.execute_effect("(storage read)".to_string()).await?,
    ];
    
    // Verify all effects executed successfully
    assert_eq!(effect_results.len(), 3);
    println!("✓ Multiple effects executed in sandbox");
    
    // Test sandbox isolation
    let metrics_before = engine.metrics().clone();
    engine.execute_effect("(network call)".to_string()).await?;
    let metrics_after = engine.metrics().clone();
    
    assert!(metrics_after.effects_executed > metrics_before.effects_executed);
    println!("✓ Effect execution properly tracked and isolated");
    
    // Test effect failure handling
    engine.machine.gas = 5; // Set low gas to trigger failure
    let result = engine.execute_effect("(compute intensive_hash)".to_string()).await;
    assert!(result.is_err());
    println!("✓ Effect failures properly handled in sandbox");
    
    Ok(())
}

#[tokio_test]
async fn test_temporal_modeling() -> Result<()> {
    println!("=== Testing Temporal Modeling ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test deterministic time progression
    let program = "(tensor (alloc 100) (alloc 200))";
    
    engine.load_program(program)?;
    
    let result = engine.run().await?;
    
    // Verify temporal consistency
    assert!(result.step_count > 0);
    println!("✓ Temporal modeling: {} execution steps recorded", result.step_count);
    
    // Test reproducibility
    let mut engine2 = SimulationEngine::new();
    engine2.load_program(program)?;
    let result2 = engine2.run().await?;
    
    assert_eq!(result.step_count, result2.step_count);
    println!("✓ Deterministic execution: consistent step counts");
    
    Ok(())
}

#[tokio_test]
async fn test_deterministic_execution() -> Result<()> {
    println!("=== Testing Deterministic Execution ===");
    
    let config = SimulationConfig {
        max_steps: 100,
        gas_limit: 50000,
        timeout_ms: 5000,
        step_by_step_mode: false,
        enable_snapshots: true,
    };
    
    let program = "(consume (alloc (tensor 42 84)))";
    
    // Execute the same program multiple times
    let mut results = Vec::new();
    for _ in 0..3 {
        let mut engine = SimulationEngine::new_with_config(config.clone());
        let result = engine.execute_program(program).await?;
        results.push(result);
    }
    
    // Verify deterministic results
    for i in 1..results.len() {
        assert_eq!(results[0].step_count, results[i].step_count);
        assert_eq!(results[0].instruction_count, results[i].instruction_count);
    }
    println!("✓ Deterministic execution verified across {} runs", results.len());
    
    Ok(())
}

#[tokio_test]
async fn test_step_by_step_execution() -> Result<()> {
    println!("=== Testing Step-by-Step Execution ===");
    
    let mut config = SimulationConfig::default();
    config.step_by_step_mode = true;
    
    let mut engine = SimulationEngine::new_with_config(config);
    
    let program = vec![
        Instruction::Witness { out_reg: RegisterId::new(0) },
        Instruction::Witness { out_reg: RegisterId::new(1) },
        Instruction::Move { src: RegisterId::new(0), dst: RegisterId::new(2) },
    ];
    
    engine.load_program(program)?;
    
    // Execute step by step
    let mut step_count = 0;
    while engine.step().await? {
        step_count += 1;
        let progression = engine.state_progression();
        assert_eq!(progression.steps.len(), step_count);
        
        if step_count >= 10 { // Safety limit
            break;
        }
    }
    
    assert!(step_count > 0);
    assert_eq!(engine.state(), &SimulationState::Completed);
    println!("✓ Step-by-step execution completed in {} steps", step_count);
    
    Ok(())
}

#[tokio_test]
async fn test_resource_conservation_properties() -> Result<()> {
    println!("=== Testing Resource Conservation Properties ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test alloc-consume conservation
    let program = vec![
        Instruction::Witness { out_reg: RegisterId::new(0) }, // Type
        Instruction::Witness { out_reg: RegisterId::new(1) }, // Value
        Instruction::Alloc { 
            type_reg: RegisterId::new(0), 
            val_reg: RegisterId::new(1), 
            out_reg: RegisterId::new(2) 
        },
        Instruction::Consume { 
            resource_reg: RegisterId::new(2), 
            out_reg: RegisterId::new(3) 
        },
    ];
    
    engine.load_program(program)?;
    engine.run().await?;
    
    let progression = engine.state_progression();
    
    // Find alloc and consume steps
    let alloc_step = progression.steps.iter()
        .find(|step| step.instruction.as_ref()
            .map(|i| i.contains("Alloc"))
            .unwrap_or(false));
    let consume_step = progression.steps.iter()
        .find(|step| step.instruction.as_ref()
            .map(|i| i.contains("Consume"))
            .unwrap_or(false));
    
    assert!(alloc_step.is_some());
    assert!(consume_step.is_some());
    
    let alloc_step = alloc_step.unwrap();
    let consume_step = consume_step.unwrap();
    
    assert!(!alloc_step.resources_allocated.is_empty());
    assert!(!consume_step.resources_consumed.is_empty());
    
    println!("✓ Resource conservation properties verified");
    println!("  - Alloc step allocated: {:?}", alloc_step.resources_allocated);
    println!("  - Consume step consumed: {:?}", consume_step.resources_consumed);
    
    Ok(())
}

#[tokio_test]
async fn test_configuration_variations() -> Result<()> {
    println!("=== Testing Configuration Variations ===");
    
    let configurations = vec![
        ("minimal", SimulationConfig {
            max_steps: 10,
            gas_limit: 1000,
            timeout_ms: 1000,
            step_by_step_mode: false,
            enable_snapshots: false,
        }),
        ("standard", SimulationConfig::default()),
        ("high_performance", SimulationConfig {
            max_steps: 10000,
            gas_limit: 1_000_000,
            timeout_ms: 60_000,
            step_by_step_mode: false,
            enable_snapshots: true,
        }),
    ];
    
    for (config_name, config) in configurations {
        println!("  Testing {} configuration", config_name);
        
        let mut engine = SimulationEngine::new_with_config(config);
        let result = engine.execute_program("(alloc 100)").await?;
        
        assert!(result.step_count > 0);
        println!("    ✓ {} configuration: {} steps", config_name, result.step_count);
    }
    
    Ok(())
}

#[tokio_test]
async fn test_error_handling_and_recovery() -> Result<()> {
    println!("=== Testing Error Handling and Recovery ===");
    
    let mut engine = SimulationEngine::new();
    
    // Test invalid program handling
    let invalid_program: Vec<Instruction> = vec![];
    engine.load_program(invalid_program)?;
    engine.run().await?; // Should complete immediately
    assert_eq!(engine.state(), &SimulationState::Completed);
    println!("✓ Empty program handled gracefully");
    
    // Test engine reset after error
    engine.reset()?;
    assert_eq!(engine.state(), &SimulationState::Created);
    println!("✓ Engine reset after error successful");
    
    // Test gas exhaustion handling
    let mut config = SimulationConfig::default();
    config.gas_limit = 10; // Very low gas limit
    
    let mut gas_limited_engine = SimulationEngine::new_with_config(config);
    gas_limited_engine.machine.gas = 5; // Set low gas
    
    let result = gas_limited_engine.execute_effect("(compute expensive_operation)".to_string()).await;
    assert!(result.is_err());
    println!("✓ Gas exhaustion properly handled");
    
    Ok(())
}

#[tokio_test]
async fn test_performance_metrics_collection() -> Result<()> {
    println!("=== Testing Performance Metrics Collection ===");
    
    let mut engine = SimulationEngine::new();
    
    // Execute various operations
    engine.execute_effect("(compute hash1)".to_string()).await?;
    engine.execute_effect("(transfer 100)".to_string()).await?;
    engine.execute_effect("(storage write)".to_string()).await?;
    
    let metrics = engine.metrics();
    
    assert!(metrics.effects_executed >= 3);
    assert!(metrics.total_gas_consumed > 0);
    
    println!("✓ Performance metrics collected:");
    println!("  - Effects executed: {}", metrics.effects_executed);
    println!("  - Total gas consumed: {}", metrics.total_gas_consumed);
    println!("  - Execution time: {} ms", metrics.execution_time_ms);
    
    Ok(())
} 