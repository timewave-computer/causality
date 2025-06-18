# Examples Guide

This guide provides comprehensive examples demonstrating various features of the Session crate, from simple message passing to complex multi-party protocols.

## Basic Examples

### Hello World

The simplest example demonstrating basic message passing between two agents.

```rust
use session::*;

fn hello_world_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Create agents
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    // Register agents with communication capabilities
    interpreter.register_agent(alice.clone(), &[EffectRow::Comm])?;
    interpreter.register_agent(bob.clone(), &[EffectRow::Comm])?;
    
    // Create choreography
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::Text("Hello".to_string()))
        .add_send(&bob, &alice, Message::Text("World".to_string()));
    
    // Execute
    let result = interpreter.execute_choreography(choreography)?;
    
    println!("Hello World executed successfully: {}", result.is_success());
    Ok(())
}
```

**Key Learning Points:**
- Agent registration with capabilities
- Basic choreography construction
- Message passing with `Message::Text`
- Execution and result checking

### Echo Server

A simple echo server demonstrating request-response patterns.

```rust
use session::*;

fn echo_server_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let client = AgentId::new("Client");
    let server = AgentId::new("Server");
    
    interpreter.register_agent(client.clone(), &[EffectRow::Comm])?;
    interpreter.register_agent(server.clone(), &[EffectRow::Comm])?;
    
    let choreography = Choreography::new()
        // Client sends request
        .add_send(&client, &server, Message::Text("Echo this message".to_string()))
        // Server echoes back
        .add_send(&server, &client, Message::Text("Echo this message".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Echo server completed: {}", result.is_success());
    
    Ok(())
}
```

## State Management Examples

### Counter Service

Demonstrates state management with persistent counters.

```rust
use session::*;

fn counter_service_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let client = AgentId::new("Client");
    let counter_service = AgentId::new("CounterService");
    
    // Register with state management capabilities
    interpreter.register_agent(client.clone(), &[EffectRow::Comm])?;
    interpreter.register_agent(counter_service.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    // Initialize counter
    interpreter.set_state(&counter_service, "count", MessageValue::Int(0))?;
    
    let choreography = Choreography::new()
        // Client requests increment
        .add_send(&client, &counter_service, Message::Text("increment".to_string()))
        // Service responds with new value
        .add_send(&counter_service, &client, Message::Int(1));
    
    let result = interpreter.execute_choreography(choreography)?;
    
    // Check final state
    let final_count = interpreter.get_state(&counter_service, "count")?;
    println!("Final count: {:?}", final_count);
    
    Ok(())
}
```

### Bank Account

More complex state management with balance tracking.

```rust
use session::*;

fn bank_account_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let customer = AgentId::new("Customer");
    let bank = AgentId::new("Bank");
    
    interpreter.register_agent(customer.clone(), &[EffectRow::Comm])?;
    interpreter.register_agent(bank.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    // Set initial balance
    interpreter.set_state(&bank, "customer_balance", MessageValue::Int(1000))?;
    
    let choreography = Choreography::new()
        // Customer requests balance
        .add_send(&customer, &bank, Message::Text("get_balance".to_string()))
        // Bank responds with balance
        .add_send(&bank, &customer, Message::Int(1000))
        // Customer requests withdrawal
        .add_send(&customer, &bank, Message::Int(200))
        // Bank confirms withdrawal
        .add_send(&bank, &customer, Message::Text("withdrawal_confirmed".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    
    let final_balance = interpreter.get_state(&bank, "customer_balance")?;
    println!("Final balance: {:?}", final_balance);
    
    Ok(())
}
```

## Payment Protocol Examples

### Simple Payment

Basic payment flow between two parties.

```rust
use session::*;

fn simple_payment_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    interpreter.register_agent(alice.clone(), &[EffectRow::Comm, EffectRow::State])?;
    interpreter.register_agent(bob.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    // Set initial balances
    interpreter.set_state(&alice, "balance", MessageValue::Int(1000))?;
    interpreter.set_state(&bob, "balance", MessageValue::Int(500))?;
    
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::PaymentRequest { 
            amount: 100, 
            recipient: "Bob".to_string() 
        })
        .add_send(&bob, &alice, Message::Payment { 
            amount: 100, 
            currency: "USD".to_string() 
        })
        .add_send(&alice, &bob, Message::Receipt);
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Payment completed: {}", result.is_success());
    
    Ok(())
}
```

### Multi-Step Payment with Validation

More complex payment with validation steps.

```rust
use session::*;

fn validated_payment_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    let validator = AgentId::new("Validator");
    
    for agent in [&alice, &bob, &validator] {
        interpreter.register_agent(agent.clone(), &[EffectRow::Comm, EffectRow::State])?;
    }
    
    // Set initial states
    interpreter.set_state(&alice, "balance", MessageValue::Int(1000))?;
    interpreter.set_state(&bob, "balance", MessageValue::Int(500))?;
    
    let choreography = Choreography::new()
        // Alice initiates payment
        .add_send(&alice, &validator, Message::PaymentRequest { 
            amount: 100, 
            recipient: "Bob".to_string() 
        })
        // Validator checks and forwards
        .add_send(&validator, &bob, Message::PaymentRequest { 
            amount: 100, 
            recipient: "Bob".to_string() 
        })
        // Bob accepts
        .add_send(&bob, &validator, Message::Text("accept".to_string()))
        // Validator confirms to Alice
        .add_send(&validator, &alice, Message::Text("validated".to_string()))
        // Alice sends payment
        .add_send(&alice, &bob, Message::Payment { 
            amount: 100, 
            currency: "USD".to_string() 
        })
        // Bob sends receipt
        .add_send(&bob, &alice, Message::Receipt);
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Validated payment completed: {}", result.is_success());
    
    Ok(())
}
```

## Parallel Execution Examples

### Atomic Swap

Two-party atomic swap with escrow.

```rust
use session::*;

fn atomic_swap_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    let escrow = AgentId::new("Escrow");
    
    for agent in [&alice, &bob, &escrow] {
        interpreter.register_agent(agent.clone(), &[EffectRow::Comm, EffectRow::State])?;
    }
    
    // Set initial token balances
    interpreter.set_state(&alice, "tokenA", MessageValue::Int(100))?;
    interpreter.set_state(&bob, "tokenB", MessageValue::Int(150))?;
    
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
        // Both confirm ready in parallel
        .add_parallel(vec![
            Choreography::new().add_send(&alice, &escrow, Message::Ready),
            Choreography::new().add_send(&bob, &escrow, Message::Ready),
        ]);
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Atomic swap completed: {}", result.is_success());
    
    Ok(())
}
```

### Auction Protocol

Multi-party auction with parallel bidding.

```rust
use session::*;

fn auction_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let auctioneer = AgentId::new("Auctioneer");
    let bidder1 = AgentId::new("Bidder1");
    let bidder2 = AgentId::new("Bidder2");
    let bidder3 = AgentId::new("Bidder3");
    
    for agent in [&auctioneer, &bidder1, &bidder2, &bidder3] {
        interpreter.register_agent(agent.clone(), &[EffectRow::Comm, EffectRow::State])?;
    }
    
    let choreography = Choreography::new()
        // Auctioneer announces auction start
        .add_send(&auctioneer, &bidder1, Message::Text("auction_start".to_string()))
        .add_send(&auctioneer, &bidder2, Message::Text("auction_start".to_string()))
        .add_send(&auctioneer, &bidder3, Message::Text("auction_start".to_string()))
        // Parallel bidding round
        .add_parallel(vec![
            Choreography::new().add_send(&bidder1, &auctioneer, Message::Int(100)),
            Choreography::new().add_send(&bidder2, &auctioneer, Message::Int(150)),
            Choreography::new().add_send(&bidder3, &auctioneer, Message::Int(120)),
        ])
        // Auctioneer announces winner
        .add_send(&auctioneer, &bidder2, Message::Text("winner".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Auction completed: {}", result.is_success());
    
    Ok(())
}
```

## Advanced Examples

### Capability-Based Access Control

Demonstrating the capability system.

```rust
use session::*;

fn capability_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let admin = AgentId::new("Admin");
    let user = AgentId::new("User");
    let database = AgentId::new("Database");
    
    // Admin has full capabilities, user has limited
    interpreter.register_agent(admin.clone(), &[EffectRow::Comm, EffectRow::State, EffectRow::IO])?;
    interpreter.register_agent(user.clone(), &[EffectRow::Comm])?;  // No state access
    interpreter.register_agent(database.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    let choreography = Choreography::new()
        // Admin can modify database
        .add_send(&admin, &database, Message::Text("create_table".to_string()))
        .add_send(&database, &admin, Message::Text("table_created".to_string()))
        // User can only read
        .add_send(&user, &database, Message::Text("read_data".to_string()))
        .add_send(&database, &user, Message::Text("data_response".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    println!("Capability-based access completed: {}", result.is_success());
    
    Ok(())
}
```

### Error Handling and Recovery

Demonstrating error handling patterns.

```rust
use session::*;

fn error_handling_example() {
    let mut interpreter = Interpreter::new();
    
    // Intentionally create an invalid scenario
    let unknown_agent = AgentId::new("Unknown");
    let valid_agent = AgentId::new("Valid");
    
    interpreter.register_agent(valid_agent.clone(), &[EffectRow::Comm]).unwrap();
    
    let invalid_choreography = Choreography::new()
        .add_send(&unknown_agent, &valid_agent, Message::Text("This will fail".to_string()));
    
    match interpreter.execute_choreography(invalid_choreography) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("Expected error: {}", e);
            
            // Access diagnostic information
            if let Some(context) = e.context() {
                println!("Error context: {:?}", context);
            }
            
            // Get suggestions for fixing the error
            for suggestion in e.suggestions() {
                println!("Suggestion: {}", suggestion);
            }
        }
    }
}
```

### Debugging and Introspection

Using the debugging features.

```rust
use session::*;

fn debugging_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Enable debugging
    interpreter.enable_debug();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    interpreter.register_agent(alice.clone(), &[EffectRow::Comm, EffectRow::State])?;
    interpreter.register_agent(bob.clone(), &[EffectRow::Comm, EffectRow::State])?;
    
    // Set some initial state
    interpreter.set_state(&alice, "debug_value", MessageValue::Int(42))?;
    
    let choreography = Choreography::new()
        .add_send(&alice, &bob, Message::Text("Debug message".to_string()));
    
    let result = interpreter.execute_choreography(choreography)?;
    
    // Examine the execution log
    println!("=== Execution Log ===");
    for (i, log_entry) in interpreter.get_effect_log().iter().enumerate() {
        println!("Step {}: {:?}", i, log_entry);
    }
    
    // Get final state snapshot
    println!("=== Final State ===");
    let snapshot = interpreter.get_state_snapshot();
    println!("{:?}", snapshot);
    
    Ok(())
}
```

## Running the Examples

To run these examples in your own project:

1. Add the session crate dependency
2. Copy the example code
3. Call the example function from your main function

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    hello_world_example()?;
    simple_payment_example()?;
    atomic_swap_example()?;
    error_handling_example();
    debugging_example()?;
    
    Ok(())
}
```

## Best Practices

### Agent Registration
- Always register agents before using them in choreographies
- Grant minimal necessary capabilities
- Use descriptive agent names

### State Management
- Initialize state before use
- Use typed state keys consistently
- Handle state access errors gracefully

### Choreography Design
- Keep individual steps simple
- Use parallel execution for independent operations
- Design for error recovery

### Debugging
- Enable debug mode during development
- Use effect logs to trace execution
- Check state snapshots for debugging 