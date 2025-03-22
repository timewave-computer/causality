# TEL Integration with the Three-Layer Effect Architecture

## Overview

The Transaction Effect Language (TEL) serves as a powerful interface layer in the three-layer effect architecture, providing a declarative way to define complex transactions across multiple domains. This document explains how TEL integrates with the new architecture to provide a consistent programming model while leveraging domain-specific optimizations.

## Three-Layer Architecture

The Causality effect system is structured in three layers:

1. **Abstract Effect Layer**: Core effect interfaces and constraint traits
2. **Domain Adapter Layer**: Domain-specific implementations of effects
3. **TEL Integration Layer**: Declarative language for defining and composing effects

This architecture separates concerns while providing a unified programming model:

- **Abstraction**: Defines what operations are possible
- **Implementation**: Handles how operations are performed on specific chains
- **Composition**: Enables complex multi-domain workflows

## TEL as an Integration Layer

TEL functions as the "glue" between abstract effects and their domain-specific implementations:

```
┌─────────────────────────────┐
│ TEL Program                 │
│ (Declarative Transaction)   │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│ TEL Compiler                │
│ (Effect Graph Generation)   │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│ Effect Constraint System    │
│ (Validation & Optimization) │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│ Domain Adapter Selection    │
│ (Implementation Binding)    │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│ Effect Runtime Execution    │
│ (Orchestration & Execution) │
└─────────────────────────────┘
```

## Key Components

### 1. TEL Compiler

The TEL compiler translates declarative TEL programs into an effect graph:

```rust
// Example TEL program
transfer(
    from: "0x1234...",
    to: "0x5678...",
    amount: 100,
    token: "ETH",
    domain: "ethereum"
)
```

The compiler:
- Parses TEL syntax
- Resolves identifiers
- Type-checks expressions
- Generates an effect graph
- Applies initial optimizations

### 2. Effect Constraint System

The constraint system uses traits to define effect behaviors:

```rust
// Example constraint traits
pub trait TransferEffect: Effect {
    fn source(&self) -> &Address;
    fn destination(&self) -> &Address;
    fn amount(&self) -> &Quantity;
    // ...
}

pub trait StorageEffect: Effect {
    fn register_id(&self) -> &ResourceId;
    fn fields(&self) -> &HashSet<String>;
    // ...
}
```

Constraints enable:
- Type-safe effect composition
- Validation across effect boundaries
- Optimization opportunities
- Documentation and discoverability

### 3. Domain Adapter Binding

The domain adapter binding process:
- Selects the appropriate domain for each effect
- Resolves domain-specific implementations
- Configures domain-specific parameters
- Creates concrete effect instances

```rust
// Example domain adapter binding
let domain_info = domain_registry.get_domain_info(&domain_id)?;
let effect = match domain_info.domain_type {
    DomainType::EVM => create_evm_transfer_effect(...),
    DomainType::CosmWasm => create_cosmwasm_transfer_effect(...),
    // ...
}
```

### 4. Effect Runtime

The effect runtime handles execution:
- Provides execution context
- Manages capabilities and authorizations
- Executes effects in the appropriate boundary
- Collects results and manages state
- Handles errors and rollbacks

## TEL Programming Model

### Declarative Syntax

TEL provides a declarative syntax for defining effects:

```rust
// Transfer example
let tx = tel! {
    transfer(
        from: account,
        to: recipient,
        amount: 100,
        token: "ETH",
        domain: eth_domain
    )
}
```

### Composition

TEL supports composing effects into complex transactions:

```rust
// Composition example
let tx = tel! {
    sequence {
        // First transfer ETH on Ethereum
        transfer(
            from: eth_account,
            to: bridge,
            amount: 100,
            token: "ETH",
            domain: eth_domain
        ),
        
        // Then mint tokens on CosmWasm chain
        mint(
            to: cosmos_account,
            amount: 100,
            token: "wETH",
            domain: cosmos_domain
        )
    }
}
```

### Constraint Validation

TEL programs are validated against effect constraints:

```rust
// Validation example
// This will be rejected if the effect doesn't implement StorageEffect
let tx = tel! {
    store_register(
        id: register_id,
        fields: ["balance", "owner"],
        domain: eth_domain
    )
}
```

## Integration with ResourceRegister

TEL integrates with the unified ResourceRegister model:

```rust
// ResourceRegister integration example
let tx = tel! {
    // Create a new resource register
    let register = create_resource_register(
        id: "resource-123",
        logic: ResourceLogic::Fungible,
        quantity: 100,
        domain: eth_domain,
        storage: StorageStrategy::FullyOnChain { visibility: Public }
    );
    
    // Store it on-chain
    store_on_chain(
        register: register,
        fields: ["id", "quantity", "owner"],
        domain: eth_domain
    )
}
```

## Cross-Domain TEL Operations

TEL shines with cross-domain operations:

```rust
// Cross-domain example
let tx = tel! {
    // Define a complex cross-domain operation
    cross_domain_transfer(
        source_domain: eth_domain,
        source_account: eth_account,
        target_domain: cosmos_domain,
        target_account: cosmos_account,
        amount: 100,
        token: "ETH"
    )
}
```

This translates to:
1. Abstract cross-domain transfer effect
2. Domain-specific implementations via adapters
3. Storage effects for resource registers

## Advanced TEL Features

### Capability Integration

TEL integrates with the capability system:

```rust
// Capability example
let tx = tel! {
    with_capability(capability_id) {
        transfer(
            from: account,
            to: recipient,
            amount: 100,
            token: "ETH",
            domain: eth_domain
        )
    }
}
```

### Conditional Execution

TEL supports conditional effect execution:

```rust
// Conditional example
let tx = tel! {
    if_then_else(
        condition: balance(account, "ETH") > 100,
        then: transfer(from: account, to: recipient, amount: 100, token: "ETH"),
        else: log("Insufficient balance")
    )
}
```

### Effect Templates

TEL can define reusable effect templates:

```rust
// Template example
let template = tel_template! {
    fn swap_tokens(from: Address, amount: u64, token_a: String, token_b: String) {
        sequence {
            transfer(from: from, to: dex, amount: amount, token: token_a),
            receive(to: from, token: token_b)
        }
    }
}

// Usage
let tx = tel! {
    swap_tokens(
        from: account,
        amount: 100,
        token_a: "ETH",
        token_b: "DAI"
    )
}
```

## Implementing TEL Handlers

Domain adapters implement TEL handlers:

```rust
// Example TEL handler for EVM transfer
pub struct EvmTransferHandler {
    // Configuration
}

impl TelHandler<TransferEffect> for EvmTransferHandler {
    fn create_effect(&self, params: TransferParams) -> Result<Arc<dyn Effect>> {
        // Create the EVM-specific transfer effect
        Ok(Arc::new(EvmTransferEffect::new(
            params.from,
            params.to,
            params.amount,
            params.token,
        )))
    }
}
```

## TEL Runtime Integration

The TEL runtime integrates with the effect system:

```rust
// TEL runtime integration
let program = compile_tel(tel_source)?;
let effect_graph = program.to_effect_graph()?;

// Validate constraints
effect_graph.validate()?;

// Bind to domain adapters
let bound_graph = domain_binder.bind_effects(effect_graph)?;

// Execute
let result = effect_runtime.execute_graph(bound_graph).await?;
```

## Best Practices

1. **Use Constraint Traits**: Leverage constraint traits for type safety
2. **Domain Independence**: Write TEL programs that are domain-agnostic when possible
3. **Composition**: Build complex workflows from simple effects
4. **Error Handling**: Use TEL's error handling for robust transactions
5. **Testing**: Test TEL programs with mock domain adapters

## Future Directions

1. **TEL Optimizer**: Advanced optimization passes for TEL programs
2. **Cross-Domain Compiler**: Specialized compiler for cross-domain operations
3. **TEL IDE Support**: Language server for autocomplete and validation
4. **Effect Library**: Standard library of common effect patterns
5. **Domain-Specific Extensions**: Specialized TEL extensions for specific domains 