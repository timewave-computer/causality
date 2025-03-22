# Causality

Causality is an integrated toolchain for distributed program development that enables cross-chain operations with causal consistency, resource safety, and verifiable execution. The system compiles high-level algebraic effects into optimized RISC-V instructions for zk-provable execution.

![](./causality.png)

## Core Features

- **Algebraic Effects System**: Compositional approach to distributed computation
- **Causal Time Model**: Unified representation of time across multiple chains
- **Verifiable Execution**: Zero-knowledge proof generation for private computation
- **Content-Addressed Code**: Immutable, verifiable program representation
- **Resource-Safe Concurrency**: Deterministic resource handling with explicit locking
- **Temporal Effect Language (TEL)**: DSL for expressing time-bound effects and causal dependencies
- **Program Account UI Models**: Serializable views for frontend integration
- **Capability-Based Resource API**: Secure, unforgeable access control with delegation and composition
- **Effect Adapter System**: Bridging abstract effects to domain-specific implementations with automatic code generation

## Status

Under heavy development. The core logic is coming together but I'm still refactoring substantially. Once I'm happy with the core data structures and their relationship I plan to implement an extensive test suite. Only when that's finished will I split out node types and implement the P2P system.

Everything in the docs directory is very WIP. Things in the specs folder are more solid, except some of the more advanced, unimplemented concepts. These are more potential ideas I'm considering.

## System Architecture

### Time

The time system ensures causal consistency across disparate execution environments. It provides an abstract representation of time that works across multiple chains through Lamport clocks that track causal relationships between distributed events. Time synchronization enables cross-chain mapping and temporal ordering, while flexible time windows support operations with temporal constraints. The system can identify and compensate for clock drift between domains to maintain consistency.

### Effects

The effect system provides a powerful algebraic approach to distributed computation. Effects are compositional, allowing complex operations to be built from simple, reusable components. Context-specific effect handlers interpret abstract operations based on their execution environment. The system employs continuation-passing style to manage control flow across asynchronous boundaries, automatically infers required capabilities, and isolates side-effects for deterministic execution.

### Effect Adapters and Code Generation

The effect adapter system connects abstract effects to concrete implementations. Domain adapters provide connectivity to external blockchains, APIs, and systems, while schema-driven code generation automatically produces adapter code from high-level schema definitions. The system ensures consistent interfaces across different domains through protocol standardization, verifies adapter implementations against schemas at runtime, and deploys adapters using immutable, content-addressed deployment with hash-based addressing.

### Resources

The resource model provides deterministic concurrency control through explicit resource-scoped locks for acquisition and release of named resources. Deterministic wait queues ensure predictable execution ordering for reproducibility, while structured resource hierarchies prevent deadlocks. The system implements a permission system through resource capabilities and provides high-level abstractions for state management through register management.

### Temporal Effect Language

TEL is a domain-specific language for time-aware, causal computation. It enables time-bound operations with explicit temporal constraints and declarative specification of causal relationships between effects. The language incorporates temporal logic for reasoning about past, present, and future states, supports effect composition with temporal sequencing guarantees, and includes a temporal type system for static verification of temporal properties.

### Verifiable Execution

The system enables verifiable computation by translating high-level effects to RISC-V instructions and providing a zero-knowledge VM optimized for ZK proof generation. Efficient witness generation, on-chain and off-chain proof verification, and automatic circuit optimization minimize proving time while maintaining security guarantees.

### Content-Addressed Code

Content-addressed code ensures immutable, verifiable program representation through cryptographic hashing of code for unique identification. The system supports deterministic builds for reproducible artifacts, cryptographic verification of code integrity, dependency resolution through hash-based linking, and tamper-proof deployment of program logic.

### Fact System

The fact system provides a standardized representation of blockchain state using unified fact types that present common interfaces for different blockchain data. Domain-specific methods enable fact observation from various sources, while cryptographic verification ensures fact integrity. The system explicitly tracks data dependencies to maintain causal consistency.

## Usage Examples

### Working with Effects

```rust
// Define a composable effect
let effect = deposit(account, amount, timestamp)
    .and_then(|_| update_balance(account, amount))
    .map(|result| {
        match result {
            Ok(_) => "Transaction successful",
            Err(_) => "Transaction failed",
        }
    });

// Execute the effect with a handler
let result = effect.execute(&handler).await?;

// Compile to RISC-V for zero-knowledge execution
let riscv_code = compiler.compile(effect);
let proof = prover.generate_proof(riscv_code, inputs);
```

### Using Effect Adapters

```rust
// Define an adapter schema for Ethereum
let schema = AdapterSchema::new()
    .with_domain("ethereum")
    .with_effect("transfer", transfer_schema)
    .with_fact("balance", balance_schema);

// Save the schema to a file for code generation
let schema_path = std::path::Path::new("schemas/ethereum.toml");
std::fs::write(schema_path, schema.to_toml()?)?;

// Generate code from schema
effect_adapters::compile_schema(schema_path, "src/adapters", "rust")?;

// Create and use the adapter
let adapter = EthereumAdapter::new(config);
let receipt = adapter.apply_effect(transfer_effect).await?;
```

### Using TEL (Temporal Effect Language)

```rust
// Define a TEL program with temporal constraints
let program = tel! {
    // Operation must occur within specified time window
    within(time_window) {
        // Operations must happen in causal sequence
        sequence {
            // Verify previous balance fact
            observe(balance(account)).
            // Perform deposit with temporal constraint
            then(deposit(account, amount)).
            // Update state only after deposit confirmation
            then(update_balance(account, amount))
        }
    }
};

// Compile and execute the TEL program
let executor = TelExecutor::new();
let result = executor.execute(program).await?;
```

### Working with Time

```rust
// Create a time window for cross-chain operations
let time_window = TimeWindow::new()
    .with_start(Timestamp::now())
    .with_duration(Duration::from_secs(600))
    .with_domains(&[ethereum_domain, solana_domain]);

// Synchronize time across domains
let time_map = time_synchronizer.synchronize(&domains).await?;

// Perform time-bounded operation
let operation = WithinTime::new(transfer_effect, time_window);
let result = executor.execute(operation).await?;
```

### Content-Addressed Code

```rust
// Generate content address for code
let code_hash = content_hasher.hash(program_code);

// Deploy using content address
let deployment = Deployment::new(code_hash)
    .with_runtime("risc-v")
    .with_permissions(["read_balance", "update_register"]);

// Execute code by content address
let execution = runtime.execute_by_hash(code_hash, inputs).await?;
```

### Resource Management

```rust
// Acquire multiple resources with deterministic ordering
let resources = resource_manager
    .acquire(["account:alice", "register:token1", "time:window1"])
    .with_timeout(Duration::from_secs(5))
    .await?;

// Resources automatically released when guard goes out of scope
let result = perform_atomic_operation(resources);
```

### Program Account UI Models

```rust
// Transform a program account to a UI view
let transformer = ProgramAccountViewTransformer::new();
let view = transformer.to_view(account);

// Serialize to JSON for frontend use
let json = to_json(&view)?;

// Create API response
HttpResponse::Ok()
    .content_type("application/json")
    .body(json)
```

## Development

### Building with Nix

Causality is built using Nix and managed with a flake:

```bash
# Enter development shell
nix develop

# Build the project
cargo build --release

# Run examples
cargo run --example program_account_serialization
cargo run --example program_account_api
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test tel_tests
cargo test --test zk_vm_tests
cargo test --test time_tests
cargo test --test effect_tests
cargo test program_account::tests::ui_tests
```

## Documentation

### Core Concepts & Guides
- [System Boundaries](docs/SYSTEM_BOUNDARIES.md)
- [Effect System](docs/EFFECT_SYSTEM.md)
- [Effect Adapters](docs/adr_002_effect_adapters.md)
- [Domain Adapters](docs/domain_adapters.md)
- [Migration Guides](docs/migrations/)

### Technical Specifications
- [Time Module](docs/time_module.md) 
- [Content Addressing](docs/adr_007_content_addressing.md)
- [Fact Management](docs/adr_008_fact_management.md)
- [TEL Integration](docs/tel_integration.md)
- [ZK VM Integration](docs/zk_vm_integration.md)

### Implementation & Development
- [Building and Running](docs/BUILD.md)
- [Nix Environment](docs/nix-environment.md)
- [Resource Capability Guide](docs/resource_capability_guide.md)
- [TEL API Reference](docs/tel_api_reference.md)
- [Glossary](docs/glossary.md)

