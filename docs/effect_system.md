# Effect System

The Effect System in Causality is a powerful abstraction for representing state changes and operations across system boundaries. This document explains how the effect system works and how it can be used with program accounts.

## Core Concepts

### Effects

An effect is a discrete operation that may cause state changes within the system or across system boundaries. Effects:

- Have a name and description
- Require specific capabilities to execute
- Can execute at specific system boundaries
- Produce an outcome with results or errors
- May cause resource changes

### Execution Boundaries

The system defines two primary execution boundaries:

- **Inside**: Effects execute within the Causality system
- **Outside**: Effects execute outside the system (e.g., on a blockchain or external service)

Additionally, effects can cross chain boundaries, which include:
- EVM (Ethereum)
- SVM (Solana)
- MoveVM
- CosmWasm
- Local
- Custom

### Effect Context

Each effect executes within a context that provides:

- Execution ID for tracing
- Boundary information
- Invoker address
- Capabilities for authorization
- Parameters for configuration

### Boundary Crossing

When effects cross system boundaries, they are wrapped in a `BoundaryCrossing` that includes:

- Context information
- Payload data
- Authentication
- Timestamp
- Origin/destination information

## Program Account Effects Integration

Program accounts can leverage the effect system through the `ProgramAccountEffectAdapter`, which:

1. Maps program accounts to effects
2. Manages effect capabilities for accounts
3. Filters available effects based on account types
4. Executes effects with the appropriate context
5. Provides a consistent interface for UI integration

### Using Program Account Effects

To use effects with program accounts:

1. Implement the `ProgramAccountEffect` trait for your effects
2. Create a `ProgramAccountEffectAdapterImpl` instance
3. Register accounts and their capabilities
4. Get available effects for accounts
5. Execute effects with parameters

### Example: Token Transfer Effect

The example demonstrates a token transfer effect that:

1. Transfers tokens between resources
2. Verifies capabilities for authorization
3. Can execute across different boundaries
4. Provides rich metadata for UI display
5. Integrates with program accounts

## Security Considerations

- Effects require appropriate capabilities to execute
- Boundary crossings include authentication
- Effects validate their inputs and execution environment
- Each effect specifies its required capabilities
- The effect system maintains an audit trail of boundary crossings

## Creating Custom Effects

To create a custom effect for program accounts:

1. Implement the `Effect` trait
2. Implement the `ProgramAccountEffect` trait
3. Define execution logic with appropriate capability checks
4. Register the effect with the `EffectRegistry`
5. Use the effect adapter to expose it to program accounts

## Example Code

See the following examples for detailed implementations:

- `examples/program_account_effect.rs` - Basic effect usage with program accounts
- `examples/privacy_preserving_effect.rs` - Privacy-preserving computation with ZK proofs

### Basic Effect Execution

```rust
// Example of executing an effect on a program account
let mut params = HashMap::new();
params.insert("source_resource_id".to_string(), source_id.to_string());
params.insert("destination_resource_id".to_string(), dest_id.to_string());
params.insert("amount".to_string(), "50".to_string());

let outcome = effect_adapter.execute_effect(
    &account_id,
    "transfer",
    params,
).await?;
```

### Privacy-Preserving Effect

```rust
// Example of executing a privacy-preserving effect
let mut params = HashMap::new();
params.insert("private_resource_id".to_string(), private_resource_id.to_string());
params.insert("encrypted_inputs".to_string(), encrypted_inputs);
params.insert("computation_type".to_string(), "zero_knowledge".to_string());

let outcome = effect_adapter.execute_effect(
    &account_id,
    "private_computation",
    params,
).await?;
``` 