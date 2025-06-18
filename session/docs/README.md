# Session Crate Documentation

Welcome to the Session crate documentation! This crate implements a minimal prototype of the unified Causality-Valence architecture, demonstrating all four layers of verifiable message-passing computation.

## Quick Start

```rust
use session::*;

// Create a simple hello world choreography
let choreography = Choreography::new()
    .add_send("Alice", "Bob", Message::Text("Hello".to_string()))
    .add_send("Bob", "Alice", Message::Text("World".to_string()));

// Execute with interpreter
let mut interpreter = Interpreter::new();
let result = interpreter.execute_choreography(choreography)?;
```

## Documentation Structure

### API Documentation
- **[Getting Started Guide](./getting-started.md)** - Your first steps with the session crate
- **[API Reference](./api-reference.md)** - Complete public interface documentation
- **[Examples Guide](./examples.md)** - Comprehensive usage examples
- **[Error Handling](./error-handling.md)** - Error types and debugging

### Architecture Documentation
- **[Architecture Overview](./architecture.md)** - Four-layer system design
- **[Layer Interactions](./layer-interactions.md)** - How layers communicate
- **[Compilation Process](./compilation.md)** - Code transformation pipeline
- **[Data Flow](./data-flow.md)** - Message and effect propagation

### Advanced Topics
- **[Linear Types](./linear-types.md)** - Linear type system and checking
- **[Row Types](./row-types.md)** - Extensible record and effect types
- **[Effect System](./effects.md)** - Algebraic effects and handlers
- **[Session Types](./session-types.md)** - Communication protocols

## Architecture Layers

1. **Layer 0**: Content-addressed message machine (5 core instructions)
2. **Layer 1**: Linear session calculus with row types
3. **Layer 2**: Verifiable outcome algebra with effects
4. **Layer 3**: Agent orchestration and choreography

## Key Features

- ✅ **Linear Message Consumption**: Messages are consumed exactly once
- ✅ **Content Addressing**: All messages are cryptographically identified
- ✅ **Row Type Polymorphism**: Extensible records and effects
- ✅ **Natural Transformation Handlers**: Composable effect transformations
- ✅ **Verifiable Outcomes**: Algebraic composition with proof generation
- ✅ **Multi-Party Choreography**: Complex communication patterns
- ✅ **Capability-Based Security**: Type-level access control

## Examples

The crate includes several complete examples:

- **Hello World**: Basic message passing between two agents
- **Payment Protocol**: Four-step financial transaction with validation
- **Atomic Swap**: Parallel execution with three-party coordination

## Dependencies

- `sha2`: For content addressing
- `thiserror`: For structured error handling

## License

This crate is part of the Causality project. 