# Causality Simulation

A Rust crate for simulating Causality agents and scenarios. This library is designed to work in both standalone mode and integrated with the Causality engine.

## Operation Modes

The crate supports two operation modes:

1. **Standalone** (default) - A lightweight simulation environment that doesn't require the full Causality engine. This mode is useful for testing and development.

2. **Engine** - Integration with the full Causality engine, which provides more advanced features. This mode is currently conditionally compiled and available but has issues due to ongoing API changes in the engine.

## Features

- Agent simulation - Create and manage simulated agents with custom behaviors
- Scenario management - Define and run scenarios with multiple agents
- Logging - Record and analyze events during simulation runs
- Runner system - Multiple execution environments (in-memory, local process, engine)
- Controller interface - High-level API for managing simulations

## Usage

### Standalone Mode (Default)

```rust
use causality_simulation::{
    agent::SimulationAgentConfig,
    scenario::Scenario,
    runner::{RunnerFactory, RunnerType},
    controller::SimulationController,
};
use std::sync::Arc;
use anyhow::Result;

// Create a scenario with some agents
let mut scenario = Scenario::new("test_scenario");
scenario.add_agent_config("agent1", "test_agent");
scenario.add_agent_config("agent2", "test_agent");

// Create a runner factory
let factory = RunnerFactory::new();

// Create a runner
let runner = factory.create(RunnerType::InMemory)?;

// Create a controller with the runner
let controller = SimulationController::new(runner);

// Run the scenario
let result = controller.run_scenario(Arc::new(scenario))?;

// Stop the scenario when done
controller.stop_scenario("test_scenario")?;
```

### Engine Integration

To use the engine integration, add the engine feature to your dependencies:

```toml
[dependencies]
causality-simulation = { version = "0.1.0", features = ["engine"] }
```

Then use the engine runner in your code:

```rust
use causality_simulation::{
    agent::SimulationAgentConfig,
    scenario::Scenario,
    runner::{RunnerFactory, RunnerType},
    controller::SimulationController,
};
use std::sync::Arc;
use anyhow::Result;

// Create a scenario with some agents
let mut scenario = Scenario::new("test_scenario");
scenario.add_agent_config("agent1", "test_agent");
scenario.add_agent_config("agent2", "test_agent");

// Create a runner factory
let factory = RunnerFactory::new();

// Create an engine runner
let runner = factory.create(RunnerType::Engine)?;

// Create a controller with the engine runner
let controller = SimulationController::new(runner);

// Run the scenario
let result = controller.run_scenario(Arc::new(scenario))?;

// Stop the scenario when done
controller.stop_scenario("test_scenario")?;
```

**Note:** The engine integration currently has issues with the `causality-engine` crate that need to be addressed before it can be fully functional. These issues include API mismatches and missing type definitions that prevent the engine runner from compiling correctly.

## Current Status

- [x] Standalone mode is fully functional
- [x] In-memory runner implementation
- [x] Local process runner implementation
- [ ] Engine integration (conditionally compiled but has issues with the engine crate)
- [x] Logging system
- [x] Scenario management
- [x] Agent simulation infrastructure

## Development Roadmap

- [x] Basic simulation framework
- [x] Standalone mode implementation
- [ ] Fix issues in the engine crate to enable proper integration
- [ ] Enhanced agent capabilities
- [ ] More sophisticated simulation scenarios
- [ ] Better error handling and reporting
- [ ] Performance optimizations

## License

This crate is available under MIT or Apache-2.0 licenses.
