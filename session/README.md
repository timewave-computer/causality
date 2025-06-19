# Session Crate

Welcome to the Session crate! This crate implements a minimal prototype of the unified Causality-Valence architecture, demonstrating all four layers of verifiable message-passing computation.

## Quick Start

```rust
use session::*;

// Create a simple hello world choreography
let choreography = Choreography::sequence([
    ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Hello".to_string()),
    },
    ChoreographyStep::Send {
        from: AgentId::new("Bob"),
        to: AgentId::new("Alice"),
        message: Message::Text("World".to_string()),
    },
]);

// Execute with interpreter
let mut interpreter = Interpreter::new();
let result = interpreter.execute_choreography(&choreography)?;
```

## Documentation Structure

### API Documentation
- **[Getting Started Guide](./docs/getting-started.md)** - Your first steps with the session crate
- **[API Reference](./docs/api-reference.md)** - Complete public interface documentation
- **[Examples Guide](./docs/examples.md)** - Comprehensive usage examples

### Architecture Documentation
- **[Architecture Overview](./docs/architecture.md)** - Four-layer system design
- **[Layer Interactions](./docs/layer-interactions.md)** - How layers communicate
- **[Compilation Process](./docs/compilation.md)** - Code transformation pipeline
- **[Data Flow](./docs/data-flow.md)** - Message and effect propagation

### Implementation Details
- **[Implementation Summary](./docs/implementation-summary.md)** - Current status and completion
- **[Theory](./docs/theory.md)** - Mathematical foundations and formal semantics

## Architecture Layers

1. **Layer 0**: Content-addressed message machine (5 core instructions)
2. **Layer 1**: Linear session calculus with row types
3. **Layer 2**: Verifiable outcome algebra with effects
4. **Layer 3**: Agent orchestration and choreography

## Key Features

- **Linear Message Consumption**: Messages are consumed exactly once
- **Content Addressing**: All messages are cryptographically identified
- **Row Type Polymorphism**: Extensible records and effects
- **Natural Transformation Handlers**: Composable effect transformations
- **Verifiable Outcomes**: Algebraic composition with proof generation
- **Multi-Party Choreography**: Complex communication patterns
- **Capability-Based Security**: Type-level access control

## Examples

The crate includes several complete examples:

- **Hello World**: Basic message passing between two agents
- **Payment Protocol**: Four-step financial transaction with validation
- **Atomic Swap**: Parallel execution with three-party coordination
- **Choreography Demo**: Multi-party payment protocol with capability checking
- **Effect Transform**: Algebraic effect composition and transformation

## Running Examples

```bash
# Run the choreography demo
cargo run --example choreography_demo

# Run the payment example
cargo run --example payment

# Run the atomic swap example
cargo run --example atomic_swap
```

## Testing

```bash
# Run all library tests
cargo test --lib

# Run integration tests
cargo test --tests

# Run all tests and examples
cargo test && cargo run --example choreography_demo
```

## Dependencies

- `sha2`: For content addressing and cryptographic integrity
- `thiserror`: For structured error handling
- `serde`: For serialization support
