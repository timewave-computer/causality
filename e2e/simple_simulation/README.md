# Simple Simulation Test

This test provides a basic example of using the Causality simulation engine for simple effect execution and testing scenarios.

## What is Tested

### Basic Simulation Features
- **Effect Creation**: Creating simple effects for testing
- **Simulation Execution**: Running effects in simulation mode  
- **State Tracking**: Monitoring state changes during simulation
- **Resource Management**: Basic linear resource handling

### Simple Workflow
- **Effect Definition**: Defining basic transfer and state change effects
- **Simulation Setup**: Configuring the simulation environment
- **Execution**: Running effects and capturing results
- **Validation**: Verifying expected outcomes

## How to Run

### Run the Simple Simulation Test
```bash
cargo test --test simple_simulation_test
```

### Run with Verbose Output
```bash
cargo test --test simple_simulation_test -- --nocapture
```

## Test Structure

This test demonstrates:
1. **Basic Effect Creation**: Creating simple effects for testing
2. **Simulation Environment Setup**: Configuring a minimal simulation
3. **Effect Execution**: Running effects in the simulation
4. **Result Validation**: Checking that effects produce expected outcomes

## Use Cases

This test is useful for:
- **Learning**: Understanding basic simulation concepts
- **Development**: Quick testing of simple effects
- **Debugging**: Isolating basic simulation functionality
- **Examples**: Demonstrating minimal simulation usage

## Expected Results

The test should pass, demonstrating:
- ✅ Basic simulation engine functionality
- ✅ Simple effect execution
- ✅ State change tracking
- ✅ Resource management basics

## Note

This is a simpler alternative to the full `simulation_zk_integration_e2e` test suite, focusing only on core simulation functionality without ZK integration complexity. 