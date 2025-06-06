# Causality Simulation

Comprehensive simulation framework for the Causality system that enables testing, debugging, and visualization of effects, resources, and distributed computation across all three architectural layers.

## Purpose

The `causality-simulation` crate provides a **complete simulation environment** for the Causality system, enabling developers to test complex distributed scenarios, debug effect interactions, and visualize system behavior without requiring full distributed infrastructure. It supports testing across all three layers of the architecture while maintaining full compatibility with production execution.

### Key Responsibilities

- **Effect Simulation**: Simulate complex effect sequences and interactions
- **State Management**: Provide snapshot and rollback capabilities for debugging
- **Performance Testing**: Measure and analyze performance characteristics
- **Fault Injection**: Test system resilience under adverse conditions
- **Visualization**: Generate visual representations of effect execution flows

## Core Features

### Simulation Engine

Central orchestration for simulation execution:

```rust
use causality_simulation::{SimulationEngine, SimulationConfig};

let config = SimulationConfig {
    max_effects: 1000,
    effect_timeout_ms: 5000,
    enable_fault_injection: true,
    enable_visualization: true,
    enable_snapshots: true,
};

let mut engine = SimulationEngine::new(config);

// Install effect handlers
engine.install_effect_handler::<TransferEffect>()?;
engine.install_effect_handler::<MintEffect>()?;

// Execute simulation
let result = engine.run_simulation().await?;
println!("Simulation completed: {:?}", result.summary);
```

### Mock Effect Handlers

Configurable mock implementations for rapid testing:

```rust
use causality_simulation::{MockStrategy, EffectSchema};

// Install mock handler with specific behavior
engine.install_mock_handler(
    "test_effect".to_string(),
    effect_schema,
    MockStrategy::AlwaysSucceed
)?;

// Install handler with failure simulation
engine.install_mock_handler(
    "flaky_effect".to_string(),
    effect_schema,
    MockStrategy::FailWithProbability { probability: 0.1 }
)?;
```

**Mock Strategies:**
- **AlwaysSucceed**: Always return success for testing happy paths
- **AlwaysFail**: Always fail for testing error handling
- **FailWithProbability**: Random failures for resilience testing
- **SimulateSuccess**: Add controllable delays and outcomes

### Resource State Snapshots

State snapshot and rollback for debugging:

```rust
use causality_simulation::SimulationEngine;

let mut engine = SimulationEngine::new(config);

// Create a snapshot before risky operation
let snapshot_id = engine.create_snapshot("before_complex_effect".to_string()).await?;

// Execute some operations
engine.execute_effect("complex_transfer_effect".to_string()).await?;

// Rollback if needed
engine.restore_snapshot(&snapshot_id).await?;
```

### Effect Testing

Comprehensive effect execution testing with various strategies:

```rust
use causality_simulation::{SimulationEngine, TestScenario, FailureRecoveryStrategy};

let mut engine = SimulationEngine::new(config);

// Create test scenario
let scenario = TestScenario {
    id: "transfer_test".to_string(),
    description: "Test token transfer effects".to_string(),
    test_suites: vec![transfer_test_suite],
    expected_outcomes: vec![ScenarioOutcome::AllTestsPass],
    recovery_strategies: vec![FailureRecoveryStrategy::Retry { 
        max_attempts: 3, 
        backoff: Duration::from_millis(100) 
    }],
    timeout: Duration::from_secs(30),
};

let result = engine.execute_test_scenario(scenario).await?;
assert_eq!(result.status, ScenarioStatus::Success);
```

## Testing Strategies by Layer

### Layer 0: Register Machine Testing

Direct testing of the 11-instruction typed register machine:

```rust
use causality_simulation::SimulationEngine;
use causality_core::machine::instruction::Instruction;

let mut engine = SimulationEngine::new(config);

// Test individual instructions
let instructions = vec![
    Instruction::Move { src: RegisterId(0), dest: RegisterId(1) },
    Instruction::Alloc { value_reg: RegisterId(1), dest: RegisterId(2) },
    Instruction::Consume { resource_reg: RegisterId(2), dest: RegisterId(3) },
];

engine.load_program(instructions)?;

// Execute with detailed state tracking
while engine.execute_step().await? {
    let step = engine.state_progression().current_step().unwrap();
    println!("PC: {}, Resources: {}, Gas: {}", 
        step.machine_state.program_counter,
        step.machine_state.resource_count,
        step.machine_state.gas_remaining
    );
}
```

### Layer 1: Causality Lisp Testing

Testing Lisp compilation and execution:

```rust
// Test Lisp function execution
let lisp_expr = "(lambda (x) (alloc (tensor x (symbol transfer))))";
let result = engine.execute_effect(lisp_expr.to_string()).await?;

// Verify type safety and linearity
assert!(matches!(result, LispValue::ResourceRef(_)));
```

### Layer 2: Effects and TEG Testing

High-level effect orchestration testing:

```rust
// Test complex effect sequences
let complex_scenario = TestScenario {
    id: "multi_party_escrow".to_string(),
    description: "Test multi-party escrow with timeouts".to_string(),
    test_suites: vec![escrow_test_suite],
    expected_outcomes: vec![
        ScenarioOutcome::AllTestsPass,
        ScenarioOutcome::PerformanceThreshold { 
            metric: "execution_time".to_string(), 
            threshold: 1.0 
        }
    ],
    recovery_strategies: vec![FailureRecoveryStrategy::FailFast],
    timeout: Duration::from_secs(60),
};

let result = engine.execute_test_scenario(complex_scenario).await?;
```

## Zero-Knowledge Proof Integration Testing

Test ZK-proof integration with mock provers for rapid iteration:

```rust
use causality_simulation::{MockStrategy, EffectSchema};

// Install mock ZK prover
engine.install_mock_handler(
    "zk_proof_effect".to_string(),
    zk_proof_schema,
    MockStrategy::SimulateSuccess { delay_ms: 100 }
)?;

// Test with controlled ZK proof timing
let zk_test = TestCase {
    name: "zk_proof_generation".to_string(),
    description: "Test ZK proof generation timing".to_string(),
    inputs: zk_test_inputs,
    expected_outputs: zk_expected_outputs,
    timeout: Duration::from_secs(5),
};
```

## Visualization and Analysis

Generate TEG visualizations and execution traces:

```rust
// Generate TEG visualization
let teg_graph = engine.generate_teg_visualization()?;
println!("TEG Structure:\n{}", teg_graph);

// Analyze performance metrics
let metrics = engine.metrics();
println!("Total effects executed: {}", metrics.total_effects);
println!("Average effect duration: {:?}", metrics.average_effect_duration);

// Examine effect execution log
for effect_execution in engine.effects_log() {
    println!("Effect: {}, Status: {:?}, Duration: {:?}",
        effect_execution.effect_id,
        effect_execution.result,
        effect_execution.duration
    );
}
```

## Configuration

Comprehensive simulation configuration options:

```rust
use causality_simulation::SimulationConfig;

let config = SimulationConfig {
    max_effects: 10000,           // Maximum effects to execute
    effect_timeout_ms: 30000,     // Individual effect timeout
    enable_fault_injection: true, // Enable resilience testing
    enable_visualization: true,   // Enable TEG visualization
    enable_snapshots: true,       // Enable state snapshots
    snapshot_interval: 50,        // Snapshot every 50 effects
    max_steps: 100000,           // Maximum simulation steps
    step_by_step_mode: false,    // Enable single-step debugging
};
```

## Testing Best Practices

1. **Use Deterministic Seeds**: Always use fixed seeds for random number generators to ensure reproducible tests
2. **Leverage Snapshots**: Create snapshots before complex operations for easy debugging
3. **Test Failure Modes**: Use fault injection to verify resilience under adverse conditions
4. **Monitor Resources**: Track resource allocation/consumption to verify conservation laws
5. **Validate Performance**: Establish performance baselines and monitor regressions
6. **Test Incrementally**: Build complex scenarios from well-tested simple components
