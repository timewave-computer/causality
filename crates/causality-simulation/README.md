# Causality Simulation

Simulation and testing framework for the Causality Resource Model. This crate provides tools for simulating Resource interactions, testing ProcessDataflowBlock orchestrations, and validating system behavior under various conditions.

## Overview

The `causality-simulation` crate enables comprehensive testing and simulation of Causality systems, providing:

- **Resource Interaction Simulation**: Simulate complex Resource state transitions and interactions
- **ProcessDataflowBlock Testing**: Test dataflow orchestrations with various input scenarios
- **Property-Based Testing**: Generate and test Resource behaviors with property-based approaches
- **Performance Analysis**: Analyze system performance under different load conditions
- **Mock Generation**: Generate mock Resources and effects for testing

All simulations maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### Simulation Engine

The core simulation engine orchestrates Resource interactions:

```rust
use causality_simulation::{SimulationEngine, SimulationConfig};

let config = SimulationConfig {
    max_steps: 1000,
    random_seed: Some(42),
    enable_tracing: true,
};

let mut engine = SimulationEngine::new(config);
engine.add_resource(initial_resource);
engine.add_effect_generator(transfer_generator);

let results = engine.run_simulation().await?;
```

### Resource State Evolution

Simulate Resource state changes over time:

```rust
use causality_simulation::resource::{ResourceSimulator, StateTransition};

let simulator = ResourceSimulator::new();
let transitions = simulator.simulate_state_evolution(
    &initial_resource,
    &effect_sequence,
    100 // steps
)?;

for transition in transitions {
    println!("Step {}: {} -> {}", 
        transition.step, 
        transition.old_state, 
        transition.new_state
    );
}
```

### ProcessDataflowBlock Testing

Test complex dataflow orchestrations:

```rust
use causality_simulation::dataflow::{DataflowTester, TestScenario};

let tester = DataflowTester::new();
let scenario = TestScenario::new()
    .with_input_resources(input_resources)
    .with_expected_outputs(expected_outputs)
    .with_timeout(Duration::from_secs(30));

let result = tester.test_dataflow_block(&dataflow_block, scenario).await?;
assert!(result.success);
```

### Property-Based Testing

Generate and test Resource properties:

```rust
use causality_simulation::property::{PropertyTester, ResourceProperty};

let property = ResourceProperty::new("balance_non_negative")
    .with_invariant(|resource| {
        resource.get_field("balance").unwrap().as_integer() >= 0
    });

let tester = PropertyTester::new();
let result = tester.test_property(&property, 1000).await?;
assert!(result.all_passed());
```

### Mock Generation

Generate mock Resources and effects for testing:

```rust
use causality_simulation::mock::{MockGenerator, MockConfig};

let generator = MockGenerator::new();
let mock_config = MockConfig::new()
    .with_resource_type("TokenResource")
    .with_field_range("balance", 0..1000)
    .with_field_values("owner", vec!["alice", "bob", "charlie"]);

let mock_resources = generator.generate_resources(&mock_config, 100)?;
```

## Testing Utilities

### Assertion Helpers

Specialized assertions for Resource testing:

```rust
use causality_simulation::assertions::*;

// Assert Resource state
assert_resource_state(&resource, "balance", 1000);

// Assert effect execution
assert_effect_success(&effect_result);

// Assert dataflow completion
assert_dataflow_completed(&dataflow_instance);

// Assert property holds
assert_property_holds(&resource, &balance_positive_property);
```

### Test Fixtures

Pre-built test scenarios and data:

```rust
use causality_simulation::fixtures::*;

// Standard test scenarios
let token_scenario = create_token_transfer_scenario();
let nft_scenario = create_nft_marketplace_scenario();
let defi_scenario = create_defi_protocol_scenario();

// Test data generators
let random_resources = generate_random_resources(100);
let stress_test_effects = generate_stress_test_effects(1000);
```

### Trace Analysis

Analyze simulation execution traces:

```rust
use causality_simulation::trace::{TraceAnalyzer, TraceQuery};

let analyzer = TraceAnalyzer::new();
let query = TraceQuery::new()
    .filter_by_resource_type("TokenResource")
    .filter_by_effect_type("TransferEffect")
    .time_range(start_time..end_time);

let analysis = analyzer.analyze_trace(&simulation_trace, query)?;
println!("Resource interactions: {}", analysis.interaction_count());
```

## Configuration

Simulation configuration options:

```toml
[simulation]
max_steps = 1000
random_seed = 42
enable_tracing = true
trace_level = "debug"

[simulation.performance]
max_concurrent_effects = 100
timeout_seconds = 30
memory_limit_mb = 512

[simulation.mock]
default_resource_count = 100
field_value_distribution = "uniform"
enable_property_validation = true

[simulation.output]
format = "json"
include_traces = true
include_metrics = true
output_dir = "simulation_results"
```

## Feature Flags

- **default**: Standard simulation features
- **property-testing**: Property-based testing support
- **performance**: Performance testing utilities
- **mock-generation**: Mock data generation
- **trace-analysis**: Execution trace analysis
- **visualization**: Simulation result visualization

## Module Structure

```
src/
├── lib.rs                    # Main library interface
├── engine.rs                 # Core simulation engine
├── resource.rs               # Resource simulation utilities
├── dataflow.rs               # ProcessDataflowBlock testing
├── property.rs               # Property-based testing
├── mock.rs                   # Mock generation
├── performance.rs            # Performance testing
├── trace.rs                  # Trace analysis
├── assertions.rs             # Testing assertions
├── fixtures.rs               # Test fixtures and scenarios
└── config.rs                 # Configuration management
```

This crate enables comprehensive testing and validation of Resource-based systems, ensuring reliability and correctness while maintaining the deterministic properties of the Causality framework.