# ADR-033: Pure Functional Architecture with Comprehensive Effect System

## Status

Proposed

## Context

The Causality system aims to provide secure, verifiable computation across heterogeneous domains. Currently, the system uses a partial effect model where some side effects are explicitly modeled through the Effect System, but others remain implicit. This creates challenges for:

1. **Deterministic Execution**: Implicit side effects can introduce non-determinism, making execution results inconsistent
2. **Verification**: Implicit effects cannot be easily verified in zero-knowledge proofs
3. **Testing**: Side effects make unit testing more difficult and less reliable
4. **Reasoning**: Implicit effects make it harder to reason about code behavior
5. **Cross-Domain Consistency**: Non-determinism can cause inconsistencies across domains

To achieve full determinism, auditability, and verifiability, we need to transition to a pure functional architecture where all sources of non-determinism are explicitly modeled as effects. This is particularly important for our zero-knowledge proof generation, which requires complete determinism.

## Decision

We will adopt a pure functional architecture where:

1. **All Side Effects Become Explicit**: Every operation with potential non-determinism will be modeled as an effect
2. **Core Logic Remains Pure**: Business logic will be pure functions that only produce effects
3. **Effect System Handles Execution**: The effect system will be responsible for executing all effects
4. **Effect Registry Controls Behavior**: Effects will be registered with handlers that define their behavior
5. **Testing with Mock Handlers**: Testing will use mock effect handlers for deterministic verification

This approach builds on our existing three-layer effect architecture but expands it to be comprehensive, covering all sources of non-determinism.

## Detailed Approach

### 1. Complete Effects Catalog

We will implement the following effect types to cover all sources of non-determinism:

#### External Resource Effects

| Effect Type | Description | Example Operations |
|-------------|-------------|-------------------|
| `FileEffect` | File system operations | Read, write, delete, list |
| `NetworkEffect` | Network communications | HTTP requests, RPC calls, WebSocket |
| `DatabaseEffect` | Database interactions | Query, insert, update, delete |
| `StorageEffect` | Permanent storage operations | Store, retrieve, verify |
| `InputOutputEffect` | Console and user I/O | Read stdin, write stdout/stderr |

#### System Effects

| Effect Type | Description | Example Operations |
|-------------|-------------|-------------------|
| `TimeEffect` | Time-related operations | Get current time, sleep, set timeout |
| `RandomEffect` | Random number generation | Generate random values, secure random |
| `EnvEffect` | Environment access | Read environment variables, config |
| `LogEffect` | Logging operations | Info, warning, error, debug |
| `ErrorEffect` | Error handling | Raise, catch, recover |

#### Concurrency Effects

| Effect Type | Description | Example Operations |
|-------------|-------------|-------------------|
| `ThreadEffect` | Thread operations | Spawn thread, join thread |
| `AsyncEffect` | Async task operations | Spawn task, await task |
| `LockEffect` | Synchronization primitives | Acquire lock, release lock |
| `ChannelEffect` | Inter-thread communication | Send message, receive message |
| `YieldEffect` | Coroutine control | Yield execution, resume |

#### Domain-Specific Effects

| Effect Type | Description | Example Operations |
|-------------|-------------|-------------------|
| `StateEffect` | State access and mutation | Read state, modify state |
| `CapabilityEffect` | Capability operations | Check capability, grant capability |
| `ResourceEffect` | Resource operations | Create, read, update, delete |
| `FactEffect` | Fact observations | Observe fact, verify fact |
| `DomainEffect` | Domain-specific operations | Submit transaction, query state |
| `CircuitEffect` | ZK circuit operations | Generate proof, verify proof |
| `ContentEffect` | Content addressing operations | Hash content, retrieve by hash |
| `AgentEffect` | Agent operations | Authenticate, authorize |
| `MessageEffect` | Messaging operations | Send message, receive message |
| `TelEffect` | TEL operations | Parse TEL, compile TEL, execute TEL |

#### Cross-Cutting Effects

| Effect Type | Description | Example Operations |
|-------------|-------------|-------------------|
| `MetricsEffect` | Metrics and monitoring | Increment counter, record timing |
| `TraceEffect` | Distributed tracing | Start span, end span, add event |
| `ProfilingEffect` | Performance profiling | Start profiling, end profiling |
| `DebugEffect` | Debugging operations | Inspect value, break |
| `UtilityEffect` | Utility operations | Print, format, parse |

### 2. Effect Implementation Details

Each effect will follow this implementation pattern:

```rust
/// Effect for file operations
pub enum FileEffect<R> {
    /// Read a file
    Read {
        path: PathBuf,
        continuation: Box<dyn Continuation<Result<Vec<u8>, FileError>, R>>,
    },
    /// Write to a file
    Write {
        path: PathBuf,
        content: Vec<u8>,
        continuation: Box<dyn Continuation<Result<(), FileError>, R>>,
    },
    /// Delete a file
    Delete {
        path: PathBuf,
        continuation: Box<dyn Continuation<Result<(), FileError>, R>>,
    },
    /// List files in a directory
    List {
        directory: PathBuf,
        continuation: Box<dyn Continuation<Result<Vec<PathBuf>, FileError>, R>>,
    },
}

impl<R> Effect<R> for FileEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        // Implementation delegates to handler
    }
    
    fn effect_id(&self) -> EffectId {
        // Generate and return effect ID
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        // Return resources this effect requires
    }
    
    fn required_capabilities(&self) -> Vec<Capability> {
        // Return capabilities required for this effect
    }
}

// Factory functions for convenience
pub fn read_file<R>(path: PathBuf) -> impl Effect<Result<Vec<u8>, FileError>, Output = R> {
    FileEffect::Read {
        path,
        continuation: Box::new(FnContinuation::new(|result| result)),
    }
}
```

### 3. Components That Need Modification

We will modify the following components to use explicit effects:

#### Core System Components

| Component | Location | Modifications Needed |
|-----------|----------|---------------------|
| Effect System | `causality-core::effect` | Extend with new effect types, handlers |
| Resource System | `causality-core::resource` | Convert direct operations to effects |
| Agent System | `causality-core::agent` | Make agent operations use effects |
| Capability System | `causality-core::capability` | Convert capability checks to effects |
| Content Addressing | `causality-core::content` | Make hash calculations use effects |
| Time System | `causality-core::time` | Expand time operations as effects |
| Domain System | `causality-domain` | Convert domain operations to effects |
| RISC-V Compiler | `causality-riscv` | Support compilation of new effects |
| ZK Integration | `causality-zkvm` | Ensure ZK proof generation uses effects |

#### Domain Adapters

| Component | Location | Modifications Needed |
|-----------|----------|---------------------|
| Ethereum Adapter | `causality-domain-ethereum` | Convert to effect-based operations |
| CosmWasm Adapter | `causality-domain-cosmwasm` | Convert to effect-based operations |
| Local Adapter | `causality-domain-local` | Convert to effect-based operations |
| Database Adapter | `causality-domain-database` | Convert to effect-based operations |

#### Specialized Components

| Component | Location | Modifications Needed |
|-----------|----------|---------------------|
| TEL Compiler | `causality-tel` | Use effects for all operations |
| Effect Interpreter | `causality-core::effect::interpreter` | Support all new effect types |
| Resource Accessor | `causality-core::resource::accessor` | Convert to effect-based access |
| Storage Manager | `causality-core::storage` | Use storage effects |
| Log System | `causality-core::log` | Use log effects |
| Time Map | `causality-core::time::map` | Use time effects |
| Fact Observer | `causality-core::fact` | Use fact effects |

#### Testing Infrastructure

| Component | Location | Modifications Needed |
|-----------|----------|---------------------|
| Test Framework | `causality-testing` | Add mock handlers for all effect types |
| Test Runners | `causality-testing::runner` | Support effect-based test execution |
| Test Assertions | `causality-testing::assert` | Add effect-specific assertions |

### 4. Integration Strategy

We will implement this transition in phases:

#### Phase 1: Core Effect Extensions

1. Define all new effect types in the core effect system
2. Implement basic handlers for each effect type
3. Extend the effect registry to support the new types
4. Create a pure execution context for running effects

#### Phase 2: System Component Migration

1. Identify all impure operations in each system component
2. Convert these operations to use the appropriate effects
3. Update interfaces to return effects rather than performing actions
4. Create adapters for backward compatibility during transition

#### Phase 3: Domain Integration

1. Refactor domain adapters to use effects for all operations
2. Update the domain registry to support effect-based adapters
3. Implement domain-specific effect handlers
4. Create migration guides for domain adapter implementers

#### Phase 4: Testing & Optimization

1. Create mock handlers for all effect types to support testing
2. Build test utilities for effect-based testing
3. Optimize effect execution for performance
4. Add instrumentation for monitoring effect execution

#### Phase 5: RISC-V & ZK Integration

1. Extend RISC-V compiler to support all effect types
2. Update ZK proof generation to handle all effects
3. Implement deterministic handlers for ZK execution
4. Add verification for effect-based ZK proofs

## Consequences

### Benefits

1. **Complete Determinism**: All sources of non-determinism become explicit and controllable
2. **Improved Verification**: Effects can be verified in zero-knowledge proofs
3. **Better Testing**: Pure functions with explicit effects are easier to test
4. **Enhanced Reasoning**: Developers can reason about code behavior more easily
5. **Cross-Domain Consistency**: Deterministic execution ensures consistent results across domains
6. **Auditability**: All operations are explicitly tracked through the effect system
7. **Composability**: Effects can be composed in a type-safe manner
8. **Separation of Concerns**: Business logic is separated from effect execution
9. **Mockability**: All effects can be mocked for testing
10. **ZK Integration**: Deterministic execution is essential for ZK proof generation

### Challenges

1. **Migration Effort**: Significant effort required to convert all impure operations
2. **Performance Overhead**: Effect-based operations may have some overhead
3. **Learning Curve**: Developers need to learn the pure functional approach
4. **API Changes**: Interfaces will change to return effects rather than results
5. **Documentation**: Comprehensive documentation needed for the new approach
6. **Compatibility**: Maintaining compatibility with existing code during transition

## Implementation Plan

We will implement this transition over the course of six months:

### Month 1: Design and Planning

1. Define the complete effect catalog with detailed specifications
2. Create architectural diagrams for the pure functional approach
3. Identify all components that need modification
4. Develop a detailed migration plan
5. Create documentation and training materials

### Month 2: Core Effect System Extensions

1. Implement all new effect types
2. Create basic handlers for each effect type
3. Extend the effect registry
4. Build the pure execution context
5. Write unit tests for all new effects

### Month 3: System Component Migration

1. Convert core system components to use effects
2. Update interfaces to return effects
3. Create compatibility adapters
4. Add comprehensive tests

### Month 4: Domain Integration

1. Refactor domain adapters
2. Implement domain-specific effect handlers
3. Update the domain registry
4. Create migration guides

### Month 5: Testing Infrastructure

1. Build mock handlers for all effect types
2. Create test utilities
3. Add effect-specific assertions
4. Write integration tests

### Month 6: RISC-V & ZK Integration

1. Extend RISC-V compiler
2. Update ZK proof generation
3. Implement deterministic handlers
4. Add verification for ZK proofs

## Examples

### Example 1: Reading a File (Before)

```rust
fn read_config_file(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}
```

### Example 1: Reading a File (After)

```rust
fn read_config_file(path: PathBuf) -> impl Effect<Result<Config, ConfigError>> {
    read_file(path).and_then(|content_result| {
        match content_result {
            Ok(content) => {
                match serde_json::from_str(&String::from_utf8_lossy(&content)) {
                    Ok(config) => Effect::pure(Ok(config)),
                    Err(e) => Effect::pure(Err(ConfigError::ParseError(e.to_string()))),
                }
            },
            Err(e) => Effect::pure(Err(ConfigError::FileError(e))),
        }
    })
}
```

### Example 2: Querying a Domain (Before)

```rust
async fn get_balance(
    domain: &DomainAdapter,
    account: Address,
) -> Result<Balance, DomainError> {
    let query = FactQuery::new()
        .with_fact_type("balance")
        .with_parameter("account", account.to_string());
    
    let result = domain.query_state(&query).await?;
    let balance = result.parse_balance()?;
    Ok(balance)
}
```

### Example 2: Querying a Domain (After)

```rust
fn get_balance(
    domain_id: DomainId,
    account: Address,
) -> impl Effect<Result<Balance, DomainError>> {
    let query = FactQuery::new()
        .with_fact_type("balance")
        .with_parameter("account", account.to_string());
    
    query_domain(domain_id, query).and_then(|result| {
        match result {
            Ok(data) => {
                match data.parse_balance() {
                    Ok(balance) => Effect::pure(Ok(balance)),
                    Err(e) => Effect::pure(Err(DomainError::ParseError(e))),
                }
            },
            Err(e) => Effect::pure(Err(e)),
        }
    })
}
```

## Conclusion

Adopting a pure functional architecture with a comprehensive effect system represents a significant evolution of the Causality platform. By making all sources of non-determinism explicit through effects, we will achieve greater determinism, verifiability, and testability.

This architectural change aligns perfectly with Causality's goals of providing secure, verifiable computation across heterogeneous domains. The transition will require substantial effort but will result in a more robust, deterministic, and verifiable system.

I've created an Architecture Decision Record (ADR) for implementing a pure functional architecture with a comprehensive effect system in the Causality platform. This ADR outlines a systematic approach to make the system fully deterministic by converting all sources of non-determinism into explicit effects.

The key points of this approach include:

1. **Comprehensive Effect Catalog**: A complete enumeration of all effect types needed to represent every source of non-determinism, organized into categories:
   - External resource effects (file, network, database)
   - System effects (time, random, environment, logging)
   - Concurrency effects (threads, async, locks, channels)
   - Domain-specific effects (state, capability, resource, domain operations)
   - Cross-cutting effects (metrics, tracing, debugging)

2. **Implementation Pattern**: A consistent pattern for implementing each effect type, with explicit continuations and handler delegation.

3. **Component Modifications**: Detailed list of all components that need to be modified to use explicit effects, including core systems, domain adapters, and specialized components.

4. **Phased Integration Strategy**: A six-month plan broken into phases:
   - Core effect extensions
   - System component migration
   - Domain integration
   - Testing and optimization
   - RISC-V and ZK integration

5. **Before/After Examples**: Code examples showing how typical operations transform from direct side effects to explicit effect-based implementations.

This transition offers significant benefits including complete determinism, improved verification (especially for ZK proofs), better testing, enhanced reasoning about code behavior, and cross-domain consistency. The primary challenges involve migration effort, potential performance overhead, and the learning curve for developers.

Would you like me to expand on any specific area of this ADR, such as the implementation details of a particular effect type, the migration strategy, or the testing approach?