# Causality Simulation

Comprehensive simulation framework for testing, debugging, and visualizing effects, resources, and distributed computation across all three architectural layers with session-driven capabilities.

## Core Features

### Simulation Engine
Central orchestration for simulation execution:

```rust
use causality_simulation::{SimulationEngine, SessionSimulationEnvironment};

// Basic simulation
let mut engine = SimulationEngine::new();
engine.initialize().await?;

// Session-driven simulation  
let env = SessionSimulationEnvironment::default();
```

### Session-Driven Simulation
Advanced simulation with session choreography:
- **Protocol Compliance**: Automatic session protocol checking
- **Deadlock Detection**: Timeout-based execution monitoring
- **Session Visualization**: Protocol flow diagrams and state tracking
- **Cross-Chain Coordination**: Multi-chain session orchestration

### Testing Strategies

**Layer 0: Register Machine Testing**
```rust
let instructions = vec![
    Instruction::Transform { morph, input, output },
    Instruction::Alloc { type_init, output },
    Instruction::Consume { resource, output },
];
```

**Layer 1: Lisp Expression Testing**
```rust
let lisp_expr = "(lambda (x) (alloc (tensor x (symbol transfer))))";
let result = engine.execute_effect(lisp_expr).await?;
```

**Layer 2: Effect Orchestration Testing**
```rust
let scenario = TestScenario::new("multi_party_escrow")
    .with_timeout(Duration::from_secs(60));
```

### Core Components

- **SimulationEngine**: Main orchestration engine with effect handlers
- **EffectTestRunner**: Effect testing with mock generation
- **FaultInjector**: Session-aware fault injection for resilience testing
- **SnapshotManager**: State snapshot and rollback for debugging
- **VisualizationHooks**: TEG visualization and execution tracing
- **CrossChainTestExecutor**: Multi-chain scenario testing
- **SessionEnvironmentGenerator**: Session participant generation

## Advanced Capabilities

### Performance Testing
- **Optimization Analysis**: Session protocol performance optimization
- **Resource Usage**: Memory and computation profiling
- **Bottleneck Detection**: Identify performance constraints

### Fault Injection
- **Session-Aware Faults**: Protocol-semantic fault injection
- **Resilience Testing**: Test recovery mechanisms
- **Error Propagation**: Verify error handling across layers

### Visualization
- **Session Flow Diagrams**: Real-time protocol visualization
- **Effect Execution Graphs**: Temporal Effect Graph rendering
- **State Transitions**: Resource lifecycle visualization

## Migration Support

Utilities for upgrading from mock-based to session-driven simulation:
- Migration validation and reporting
- Backward compatibility with existing tests
- Progressive adoption of session features
