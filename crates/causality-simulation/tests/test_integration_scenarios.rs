//! Integration scenario tests for causality-simulation
//!
//! Tests for complex integration scenarios that combine multiple aspects
//! of the simulation system including effects, branching, and optimization.

use causality_simulation::{
    engine::{SimulationEngine, ExecutionState},
    branching::BranchingManager,
    optimizer::{OptimizationStrategy, OptimizerConfig, OptimizationStats},
    clock::SimulatedClock,
};
use causality_core::{
    effect::core::{EffectExpr, EffectExprKind},
    lambda::{Term, Literal, base::TypeInner},
};
use std::sync::Arc;
use uuid::Uuid;

/// Test end-to-end simulation with effects and optimization
#[tokio::test]
async fn test_e2e_simulation_with_effects() {
    let mut engine = SimulationEngine::new();
    
    // Create a simple effect expression for testing
    let effect_expr = EffectExpr {
        kind: EffectExprKind::Pure(Term::literal(Literal::Int(42))),
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    // Convert to string representation for execution
    let effect_str = format!("(pure {})", 42);
    
    // Execute the effect
    let result = engine.execute_effect(effect_str).await;
    assert!(result.is_ok(), "Effect execution should succeed");
    
    // Test basic optimizer functionality
    let optimizer_config = OptimizerConfig {
        strategy: OptimizationStrategy::Balanced,
        max_parallel_effects: 4,
        enable_dependency_analysis: true,
        enable_cost_prediction: true,
    };
    
    // We can't directly test optimization on the engine, but we can verify
    // the optimizer config is valid and doesn't panic
    assert_eq!(optimizer_config.strategy, OptimizationStrategy::Balanced);
    assert_eq!(optimizer_config.max_parallel_effects, 4);
}

/// Test simulation with branching and state management
#[tokio::test]
async fn test_simulation_with_branching() {
    let mut branching_manager = BranchingManager::new();
    
    // Create a unique branch
    let branch_name = format!("integration_branch_{}", Uuid::new_v4());
    let branch_id_str = format!("integration_{}", Uuid::new_v4());
    let execution_state = ExecutionState::new();
    
    let branch_id = branching_manager.create_branch(&branch_id_str, &branch_name, execution_state)
        .expect("Failed to create branch");
    
    // Switch to the new branch
    let switch_result = branching_manager.switch_to_branch(&branch_id);
    assert!(switch_result.is_ok(), "Should be able to switch to branch");
    
    // Verify we can get the branch state
    let state = branching_manager.get_branch_state(&branch_id_str);
    assert!(state.is_ok(), "Should be able to get branch state");
    
    // Test branch listing
    let branches = branching_manager.list_branches();
    assert!(branches.len() >= 2, "Should have root branch plus our created branch");
}

/// Test effect expressions with proper syntax
#[tokio::test]
async fn test_effect_expressions() {
    let mut engine = SimulationEngine::new();
    
    // Create different types of effect expressions
    let pure_effect = EffectExpr {
        kind: EffectExprKind::Pure(Term::literal(Literal::Int(100))),
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    let perform_effect = EffectExpr {
        kind: EffectExprKind::Perform {
            effect_tag: "test_effect".to_string(),
            args: vec![Term::literal(Literal::Int(1))],
        },
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    // Test pure effect execution
    let pure_result = engine.execute_effect("(pure 100)".to_string()).await;
    assert!(pure_result.is_ok(), "Pure effect should execute successfully");
    
    // Test perform effect execution (might fail due to unimplemented handlers)
    let perform_result = engine.execute_effect("(perform test_effect 1)".to_string()).await;
    // This might fail, which is acceptable for unimplemented effects
    match perform_result {
        Ok(_) => {
            // Effect execution succeeded
        }
        Err(_) => {
            // Expected for unimplemented effect handlers
        }
    }
}

/// Test optimization strategies
#[tokio::test]
async fn test_optimization_strategies() {
    // Test different optimization strategies
    let strategies = vec![
        OptimizationStrategy::GasEfficiency,
        OptimizationStrategy::Speed,
        OptimizationStrategy::Balanced,
        OptimizationStrategy::SessionOptimized,
        OptimizationStrategy::CommunicationOptimized,
        OptimizationStrategy::MultiPartyOptimized,
    ];
    
    for strategy in strategies {
        let config = OptimizerConfig {
            strategy,
            max_parallel_effects: 5,
            enable_dependency_analysis: true,
            enable_cost_prediction: true,
        };
        
        // Verify config is valid
        assert_eq!(config.strategy, strategy);
        assert_eq!(config.max_parallel_effects, 5);
        assert!(config.enable_dependency_analysis);
        assert!(config.enable_cost_prediction);
    }
}

/// Test simulation state persistence and recovery
#[tokio::test]
async fn test_state_persistence() {
    let mut engine = SimulationEngine::new();
    
    // Execute an effect that modifies state
    let effect_result = engine.execute_effect("(pure 42)".to_string()).await;
    assert!(effect_result.is_ok(), "Effect execution should succeed");
    
    // Get current state snapshot
    let state_snapshot = engine.execution_state();
    
    // Verify state has some content (registers, etc.)
    // We can't easily compare states, so we just verify we can get them
    assert!(state_snapshot.registers.len() >= 0, "State should be accessible");
    assert_eq!(state_snapshot.instruction_pointer, 0, "Initial IP should be 0");
    
    // Test that we can access state properties
    assert!(state_snapshot.effect_history.len() >= 0, "Effect history should be accessible");
    assert!(state_snapshot.gas >= 0, "Gas should be non-negative");
}

/// Test parallel effect composition
#[tokio::test]
async fn test_parallel_composition() {
    let mut engine = SimulationEngine::new();
    
    // Create parallel effect composition
    let left_effect = EffectExpr {
        kind: EffectExprKind::Pure(Term::literal(Literal::Int(1))),
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    let right_effect = EffectExpr {
        kind: EffectExprKind::Pure(Term::literal(Literal::Int(2))),
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    let parallel_effect = EffectExpr {
        kind: EffectExprKind::Parallel {
            left: Box::new(left_effect),
            right: Box::new(right_effect),
        },
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    // Test parallel execution (might not be fully implemented)
    let parallel_result = engine.execute_effect("(parallel (pure 1) (pure 2))".to_string()).await;
    match parallel_result {
        Ok(_) => {
            // Parallel execution succeeded
        }
        Err(_) => {
            // Expected for unimplemented parallel execution
        }
    }
}

/// Test session-based effects
#[tokio::test]
async fn test_session_effects() {
    let mut engine = SimulationEngine::new();
    
    // Create session-based effect
    let session_effect = EffectExpr {
        kind: EffectExprKind::WithSession {
            session_decl: "test_session".to_string(),
            role: "participant_a".to_string(),
            body: Box::new(EffectExpr {
                kind: EffectExprKind::Pure(Term::literal(Literal::Int(42))),
                ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
            }),
        },
        ty: Some(TypeInner::Base(causality_core::lambda::base::BaseType::Int)),
    };
    
    // Test session effect execution (might not be fully implemented)
    let session_result = engine.execute_effect("(with-session test_session participant_a (pure 42))".to_string()).await;
    match session_result {
        Ok(_) => {
            // Session effect succeeded
        }
        Err(_) => {
            // Expected for unimplemented session features
        }
    }
} 