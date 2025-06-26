# Causality Runtime

Execution environment providing specialized executors for sequential, zero-knowledge proof generation, and parallel Task Effect Graph execution while maintaining linear resource constraints.

## Core Executors

### Basic Executor
Sequential execution of register machine instructions with linear resource tracking:

```rust
use causality_runtime::{Executor, RuntimeContext};
use causality_core::machine::{Instruction, RegisterId, MachineValue};

// Create basic executor
let mut executor = Executor::new();

// Execute instruction sequence
let instructions = vec![
    Instruction::Witness { out_reg: RegisterId(0) },
    Instruction::Move { src: RegisterId(0), dst: RegisterId(1) },
    Instruction::Return { reg: RegisterId(1) },
];

let result = executor.execute(&instructions)?;
println!("Execution result: {:?}", result);
```

**Basic Executor Features:**
- **Sequential Execution**: Execute instructions in order with proper dependencies
- **State Management**: Maintain machine state across instruction sequences
- **Resource Enforcement**: Enforce linear resource consumption constraints
- **Error Recovery**: Provide detailed error information for debugging

### ZK Executor  
Zero-knowledge proof generation with multiple backend support:

```rust
use causality_runtime::zk_executor::{ZkExecutor, ZkExecutionConfig};

let zk_executor = ZkExecutor::with_config(config);
let (result, proof) = zk_executor.execute_with_proof(&instructions)?;
```

**ZK Backends:**
- **Mock**: Configurable testing backend
- **SP1**: Local proving with SP1 infrastructure  
- **Valence**: Production coprocessor integration

### TEG Executor
Parallel execution using work-stealing scheduler:

```rust
use causality_runtime::teg_executor::{TegExecutor, TegExecutorConfig};
use causality_core::effect::TaskEffectGraph;

let teg_executor = TegExecutor::with_config(config, context);
let result = teg_executor.execute(task_effect_graph)?;
```

**Features:**
- Work-stealing load balancing
- Adaptive scheduling with ML optimization
- Performance metrics and analysis

## Key Features

- **Linear Resource Enforcement**: Runtime validation of linear constraints
- **Multi-Backend ZK**: Unified interface for different proving systems
- **Parallel Execution**: Efficient parallelization of independent effects
- **Error Recovery**: Comprehensive error handling and recovery
- **State Management**: Session-based execution with cleanup

## Integration

- **Layer 0 Instructions**: Direct execution of 5 fundamental instructions
- **ZK Circuits**: Automatic circuit generation and proof verification
- **Resource Tracking**: Content-addressed resource lifecycle management
- **Session Management**: Stateful execution contexts with timeouts

## Purpose

The `causality-runtime` crate serves as the execution environment for the Causality system, providing specialized executors that can handle different execution requirements while maintaining consistency with the resource model's linear constraints and verifiable properties. It bridges the gap between compiled instructions and actual execution, offering optimal performance for various workload types.

### Key Responsibilities

- **Execution Orchestration**: Coordinate execution across multiple execution modes
- **Resource Lifecycle Management**: Enforce linear resource constraints during execution
- **ZK Integration**: Generate and verify zero-knowledge proofs during execution
- **Parallel Execution**: Enable parallel execution of effect graphs through work stealing

## Architecture Overview

The runtime system is designed around specialized executors optimized for different execution patterns:

### Basic Executor
Sequential execution engine for register machine instructions:
- **Simple Interface**: Direct execution of instruction sequences
- **Resource Tracking**: Linear resource consumption enforcement
- **Error Handling**: Comprehensive error reporting and recovery

### ZK Executor
Zero-knowledge proof generation during execution:
- **Proof Generation**: Automatic proof generation for specified instruction sequences
- **Backend Abstraction**: Support for multiple ZK backends (Mock, SP1, Valence)
- **Circuit Caching**: Intelligent caching of compiled circuits for performance

### TEG Executor
Parallel execution for Task Effect Graphs:
- **Work Stealing**: Dynamic load balancing across worker threads
- **Adaptive Scheduling**: Machine learning-based task scheduling optimization
- **Performance Monitoring**: Detailed metrics for parallel execution analysis

## Backend Configuration

### Mock Backend Configuration
```rust
use causality_runtime::zk_executor::{ZkBackendConfig, MockCircuitConfig};

let mock_config = ZkBackendConfig::Mock {
    success_rate: 0.95,        // 95% success rate
    proof_time_ms: 100,        // 100ms simulated proof time
};
```

### SP1 Backend Configuration
```rust
let sp1_config = ZkBackendConfig::SP1 {
    use_remote_prover: false,
    timeout_secs: 300,
    recursion_enabled: true,
};
```

### Valence Backend Configuration
```rust
let valence_config = ZkBackendConfig::Valence {
    endpoint: "https://api.valence.network".to_string(),
    api_key: Some("your_api_key".to_string()),
    circuit_deployment_config: ValenceCircuitConfig {
        controller_path: "/path/to/controller.wasm".to_string(),
        circuit_name: "causality_execution".to_string(),
        auto_deploy: true,
    },
};
```

## Error Handling

Comprehensive error types for different failure modes:

```rust
use causality_runtime::{RuntimeError, RuntimeResult};

fn handle_execution_errors(result: RuntimeResult<MachineValue>) {
    match result {
        Ok(value) => println!("Success: {:?}", value),
        Err(RuntimeError::ExecutionFailed { message }) => {
            eprintln!("Execution failed: {}", message);
        }
        Err(RuntimeError::LinearityViolation { message }) => {
            eprintln!("Resource linearity violated: {}", message);
        }
        Err(RuntimeError::MachineError(machine_err)) => {
            eprintln!("Machine-level error: {:?}", machine_err);
        }
        Err(RuntimeError::TypeMismatch(msg)) => {
            eprintln!("Type error: {}", msg);
        }
        Err(err) => eprintln!("Other error: {}", err),
    }
}
```

## Performance Monitoring

### Execution Metrics

Track performance across different execution modes:

```rust
// Basic execution metrics
let start_time = std::time::Instant::now();
let result = executor.execute(&instructions)?;
let execution_time = start_time.elapsed();

println!("Basic execution took: {:?}", execution_time);

// ZK execution metrics  
let zk_metrics = zk_executor.get_performance_metrics();
println!("ZK proof generation stats:");
println!("  Average proof time: {}ms", zk_metrics.avg_proof_generation_time_ms);
println!("  Total proofs: {}", zk_metrics.total_proofs_generated);
println!("  Circuit cache hit rate: {:.2}%", zk_metrics.cache_hit_rate * 100.0);

// TEG execution metrics
let teg_result = teg_executor.execute(teg)?;
println!("TEG execution stats:");
println!("  Nodes executed: {}", teg_result.nodes_executed);
println!("  Parallel efficiency: {:.2}%", teg_result.parallelism_efficiency * 100.0);
println!("  Work stealing events: {}", teg_result.total_work_stealing_events);
```

## Configuration

### Basic Executor Configuration

```rust
use causality_runtime::Executor;

// Basic executor requires no configuration - uses default machine state
let mut executor = Executor::new();
```

### ZK Executor Configuration

```rust
use causality_runtime::zk_executor::{ZkExecutionConfig, ZkBackendConfig};

let config = ZkExecutionConfig {
    enable_circuit_caching: true,     // Cache compiled circuits
    max_circuit_size: 2000,           // Maximum instructions per circuit
    always_generate_proofs: false,    // Generate proofs on demand
    backend_config: ZkBackendConfig::Mock {
        success_rate: 0.95,           // 95% success rate for testing
        proof_time_ms: 250,           // Simulated proof generation time
    },
};
```

### TEG Executor Configuration

```rust
use causality_runtime::teg_executor::TegExecutorConfig;

let config = TegExecutorConfig {
    worker_count: num_cpus::get(),    // Use all available cores
    steal_timeout_ms: 100,            // Wait 100ms before stealing work
    load_balance_threshold: 4,        // Load balance when queue > 4 items
    node_timeout_ms: 30000,           // 30 second timeout per node
    adaptive_scheduling: true,        // Enable learning-based optimization
};
```

## Runtime Architecture

The runtime system is organized into specialized executors for different execution modes:

1. **Basic Executor**: Foundation for sequential instruction execution
2. **ZK Executor**: Extends basic execution with zero-knowledge proof capabilities  
3. **TEG Executor**: Orchestrates parallel execution of effect graphs
4. **Error System**: Unified error handling across all execution modes

This modular design enables optimal performance for different workload types while maintaining consistency across the Causality execution model.
