# Getting Started Guide

This guide will walk you through your first steps with the Session crate, demonstrating how to create and execute simple choreographies using the unified Causality-Valence architecture.

## Installation

Add the session crate to your `Cargo.toml`:

```toml
[dependencies]
session = { path = "../session" }
```

## Basic Concepts

The Session crate implements a four-layer architecture for verifiable message-passing:

- **Messages**: Content-addressed linear values that are consumed exactly once
- **Agents**: Participants in communication protocols
- **Choreographies**: High-level descriptions of multi-party interactions
- **Effects**: Algebraic descriptions of computational side effects

## Your First Program

Let's start with a simple two-party message exchange:

```rust
use session::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agents
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");

    // Create a choreography
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::Text("Hello".to_string()))
        .add_send(&bob, &alice, Message::Text("World".to_string()));

    // Create interpreter and execute
    let mut interpreter = Interpreter::new();
    
    // Register agents with capabilities
    interpreter.register_agent(alice, &[EffectRow::Comm, EffectRow::State])?;
    interpreter.register_agent(bob, &[EffectRow::Comm, EffectRow::State])?;

    // Execute the choreography
    let result = interpreter.execute_choreography(choreography)?;
    
    println!("Execution successful: {}", result.is_success());
    Ok(())
}
```

## Understanding the Flow

1. **Agent Registration**: Agents must be registered with their capabilities before participating
2. **Choreography Creation**: Define the sequence of message exchanges
3. **Execution**: The interpreter compiles and executes the choreography through all layers
4. **Verification**: Results include proof of correct execution

## Working with Different Message Types

The session crate supports various message types:

```rust
use session::*;

// Text messages
let text_msg = Message::Text("Hello".to_string());

// Numeric values
let int_msg = Message::Int(42);

// Boolean values
let bool_msg = Message::Bool(true);

// Payment-specific messages
let payment_msg = Message::Payment { amount: 100, currency: "USD".to_string() };

// Structured data
let request_msg = Message::PaymentRequest { 
    amount: 100, 
    recipient: "Bob".to_string() 
};
```

## Adding State Management

Agents can maintain state that persists across message exchanges:

```rust
use session::*;

fn payment_with_balance() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    // Register agents
    interpreter.register_agent(alice.clone(), &[EffectRow::Comm, EffectRow::State])?;
    interpreter.register_agent(bob.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    // Set initial balances
    interpreter.set_state(&alice, "balance", MessageValue::Int(1000))?;
    interpreter.set_state(&bob, "balance", MessageValue::Int(500))?;
    
    // Create payment choreography
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::PaymentRequest { 
            amount: 100, 
            recipient: "Bob".to_string() 
        });
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Payment completed: {}", result.is_success());
    
    Ok(())
}
```

## Parallel Execution

The session crate supports parallel choreographies for complex protocols:

```rust
use session::*;

fn atomic_swap() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    let escrow = AgentId::new("Escrow");
    
    // Register all agents
    for agent in [&alice, &bob, &escrow] {
        interpreter.register_agent(agent.clone(), &[EffectRow::Comm, EffectRow::State])?;
    }
    
    // Set initial token balances
    interpreter.set_state(&alice, "tokenA", MessageValue::Int(100))?;
    interpreter.set_state(&bob, "tokenB", MessageValue::Int(150))?;
    
    // Create parallel choreography
    let choreography = Choreography::new()
        // Both parties deposit tokens in parallel
        .add_parallel(vec![
            Choreography::new().add_send(&alice, &escrow, Message::TokenDeposit {
                token_type: "TokenA".to_string(),
                amount: 50,
            }),
            Choreography::new().add_send(&bob, &escrow, Message::TokenDeposit {
                token_type: "TokenB".to_string(),
                amount: 75,
            }),
        ])
        // Both confirm ready
        .add_parallel(vec![
            Choreography::new().add_send(&alice, &escrow, Message::Ready),
            Choreography::new().add_send(&bob, &escrow, Message::Ready),
        ]);
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Atomic swap completed: {}", result.is_success());
    
    Ok(())
}
```

## Error Handling

The session crate provides structured error handling with detailed diagnostics:

```rust
use session::*;

fn error_handling_example() {
    let mut interpreter = Interpreter::new();
    
    // This will fail because the agent isn't registered
    let unregistered_agent = AgentId::new("Unknown");
    let choreography = Choreography::new()
        .add_send(&unregistered_agent, &AgentId::new("Bob"), Message::Text("Hello".to_string()));
    
    match interpreter.execute_choreography(choreography) {
        Ok(_) => println!("Success"),
        Err(e) => {
            println!("Error: {}", e);
            
            // Access diagnostic information
            if let Some(context) = e.context() {
                println!("Context: {}", context);
                for suggestion in e.suggestions() {
                    println!("Suggestion: {}", suggestion);
                }
            }
        }
    }
}
```

## Debugging Support

The interpreter provides debugging capabilities for development:

```rust
use session::*;

fn debugging_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Enable debug mode
    interpreter.enable_debug();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    interpreter.register_agent(alice.clone(), &[EffectRow::Comm])?;
    interpreter.register_agent(bob.clone(), &[EffectRow::Comm])?;
    
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::Text("Debug test".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    
    // Access execution logs
    for log in interpreter.get_effect_log() {
        println!("Effect executed: {:?}", log);
    }
    
    // Get state snapshot
    let snapshot = interpreter.get_state_snapshot();
    println!("Final state: {:?}", snapshot);
    
    Ok(())
}
```

## Next Steps

Now that you've learned the basics:

1. **Read the [API Reference](./api-reference.md)** for complete interface documentation
2. **Explore [Examples](./examples.md)** for more complex use cases
3. **Study [Architecture](./architecture.md)** to understand the four-layer design
4. **Learn about [Linear Types](./linear-types.md)** for advanced type safety

## Common Patterns

### Agent Registration
Always register agents with appropriate capabilities before use:

```rust
interpreter.register_agent(agent_id, &[EffectRow::Comm, EffectRow::State])?;
```

### Channel Naming
Use Unicode arrows for clear channel identification:

```rust
let channel_name = "Alice→Bob";
```

### Effect Composition
Combine effects using the algebraic operations:

```rust
let combined_effect = effect1.compose(effect2);
```

### State Management
Use typed state operations:

```rust
interpreter.set_state(&agent, "key", MessageValue::Int(42))?;
let value = interpreter.get_state(&agent, "key")?;
``` 