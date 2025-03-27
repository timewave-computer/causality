# Temporal Effect Language (TEL) System

*This document is derived from [ADR-013](../../spec/adr_013_tel.md) and [ADR-014](../../spec/adr_014_compiler.md), which define the Temporal Effect Language and its compiler architecture.*

*Last updated: 2023-03-27*

*This is subject to change substantially*

## Overview

The Temporal Effect Language (TEL) is a domain-agnostic language designed to express effects across different blockchain platforms and execution environments. It provides a unified programming model for describing complex operations that may involve multiple domains, time constraints, and resource manipulations.

TEL serves as an abstraction layer between the high-level application logic and the low-level domain-specific operations, enabling developers to write portable code that can be executed across different blockchains without changing the core business logic.

## Core Concepts

### Domain-Agnostic Effects

TEL's primary function is to provide a consistent way to express operations across various domains (blockchain platforms). It achieves this through:

- **Abstract Effect Types**: Standard effect representations that aren't tied to specific blockchains
- **Domain Adapters**: Translators that convert abstract effects to domain-specific implementations
- **Unified Resource Model**: Consistent representation of assets and resources across domains

### Temporal Operations

TEL includes built-in support for time-aware operations:

- **Timed Execution**: Effects that should execute at specific times or after delays
- **Time Constraints**: Effects that must complete within certain time windows
- **Sequencing**: Ensuring operations occur in the correct causal order
- **Parallel Execution**: Running independent operations concurrently

### Resource Linearity

TEL enforces resource conservation and prevents double-spending through:

- **Resource Tracking**: Following resource lifecycles through operations
- **Linear Types**: Ensuring resources are used exactly once
- **Conservation Laws**: Maintaining invariants like "value in equals value out"
- **Resource Verification**: Validating resource states and transformations

## Architecture

The TEL system consists of several key components working together:

```
┌─────────────────┐
│ TEL Source Code │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌─────────────────┐
│   TEL Parser    │─────▶│   TEL Script    │
└────────┬────────┘      └────────┬────────┘
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌─────────────────┐
│   TEL Compiler  │◄─────┤ Handler Registry│
└────────┬────────┘      └─────────────────┘
         │                        ▲
         ▼                        │
┌─────────────────┐      ┌─────────────────┐
│ Effect System   │      │ Domain-Specific │
│ Integration     │      │ Handlers        │
└────────┬────────┘      └─────────────────┘
         │
         ▼
┌─────────────────┐
│ Effect Execution│
└─────────────────┘
```

### TEL Script

The script represents a complete TEL program with metadata and operations:

```rust
pub struct TelScript {
    /// Script version
    pub version: String,
    
    /// Source code of the script
    pub source: String,
    
    /// Parsed operations in the script
    pub operations: Vec<TelOperation>,
    
    /// Script metadata
    pub metadata: HashMap<String, String>,
}
```

### TEL Operations

Operations define the actions to be performed:

```rust
pub struct TelOperation {
    /// Operation type
    pub operation_type: TelOperationType,
    
    /// Function name
    pub function_name: String,
    
    /// Parameters for the operation
    pub parameters: Value,
    
    /// Domain ID where the operation should be executed
    pub domain_id: Option<DomainId>,
    
    /// Child operations (for composite operations like sequence)
    pub children: Vec<TelOperation>,
}
```

### TEL Operation Types

TEL supports various operation types:

```rust
pub enum TelOperationType {
    /// Transfer assets between addresses
    Transfer,
    
    /// Store data on chain
    Store,
    
    /// Query data from chain
    Query,
    
    /// Sequence of operations
    Sequence,
    
    /// Parallel operations
    Parallel,
    
    /// Conditional operation
    Conditional,
    
    /// Custom operation type
    Custom(String),
}
```

### TEL Handler Registry

The handler registry manages domain-specific handlers for different operation types:

```rust
pub struct TelHandlerRegistry {
    /// Handlers indexed by (function_name, domain_type)
    handlers: HashMap<(String, String), Arc<dyn TelHandler>>,
    
    /// Domain registry for domain information
    domain_registry: Arc<DomainRegistry>,
}
```

### TEL Handlers

Handlers translate TEL operations into concrete effects for specific domains:

```rust
#[async_trait]
pub trait TelHandler: Send + Sync + Debug {
    /// Get the effect type this handler creates
    fn effect_type(&self) -> &'static str;
    
    /// Get the TEL function name this handler processes
    fn tel_function_name(&self) -> &'static str;
    
    /// Get the domain type this handler supports
    fn domain_type(&self) -> &'static str;
    
    /// Parse TEL parameters and create an effect
    async fn create_effect(&self, params: Value, context: &EffectContext) 
        -> Result<Arc<dyn Effect>, anyhow::Error>;
}
```

### Specialized Handler Types

TEL includes specialized handlers for common operations:

```rust
// For transfer operations
pub trait TransferTelHandler: ConstraintTelHandler<dyn TransferEffect> {
    fn supported_tokens(&self) -> Vec<String>;
}

// For storage operations
pub trait StorageTelHandler: ConstraintTelHandler<dyn StorageEffect> {
    fn supported_storage_strategies(&self) -> Vec<String>;
}

// For query operations
pub trait QueryTelHandler: ConstraintTelHandler<dyn QueryEffect> {
    fn supported_query_types(&self) -> Vec<String>;
}
```

### TEL Compiler

The compiler transforms TEL scripts into executable effects:

```rust
pub trait TelCompiler {
    /// Compile a TEL script into effects
    async fn compile(&self, script: &TelScript, context: &EffectContext) 
        -> Result<Vec<Arc<dyn Effect>>, anyhow::Error>;
    
    /// Execute a TEL script
    async fn execute(&self, script: &TelScript, context: EffectContext) 
        -> Result<Vec<EffectOutcome>, anyhow::Error>;
}
```

## Integration with Other Systems

### Effect System Integration

TEL integrates with the Effect System through:

1. **Effect Creation**: TEL operations are compiled into Effects
2. **Effect Execution**: The compiled Effects are executed through the Effect Engine
3. **Continuation Chaining**: TEL supports combining multiple Effects with continuations

```rust
/// Shorthand function to compile a TEL script into effects
pub async fn compile_tel(
    source: &str,
    compiler: &dyn TelCompiler,
    context: &EffectContext,
) -> Result<Vec<Arc<dyn Effect>>, anyhow::Error> {
    let script = parse_tel(source)?;
    compiler.compile(&script, context).await
}

/// Shorthand function to execute a TEL script
pub async fn execute_tel(
    source: &str,
    compiler: &dyn TelCompiler,
    context: EffectContext,
) -> Result<Vec<EffectOutcome>, anyhow::Error> {
    let script = parse_tel(source)?;
    compiler.execute(&script, context).await
}
```

### Domain System Integration

TEL integrates with the Domain System through:

1. **Domain-Specific Handlers**: Specialized handlers for each supported domain
2. **Domain Registry Lookup**: Finding appropriate domains for operations
3. **Domain Capability Checks**: Verifying domains support required operations

### Resource System Integration

TEL integrates with the Resource System through:

1. **Resource Effects**: Creating, updating, and transferring resources
2. **Resource Verification**: Validating resource states and transformations
3. **Resource Adapters**: Converting between domain-specific and universal resource formats

## Resource Effects

TEL defines several core resource effects:

```rust
pub struct ResourceEffect {
    /// ID of the effect
    pub id: ContentId,
    /// The operation this effect will perform
    pub operation: ResourceOperation,
    /// The proof associated with this effect (if any)
    pub proof: Option<Proof>,
    /// Whether this effect requires verification
    pub requires_verification: bool,
}
```

### Resource Operations

Operations that can be performed on resources:

```rust
pub enum ResourceOperationType {
    Create {
        owner: Address,
        domain: Domain,
        initial_data: RegisterContents,
    },
    Update {
        resource_id: ResourceId,
        new_data: RegisterContents,
    },
    Delete {
        resource_id: ResourceId,
    },
    Transfer {
        resource_id: ResourceId,
        from: Address,
        to: Address,
    },
    // And other operation types...
}
```

## Builder Pattern

TEL implements a builder pattern for constructing complex effects:

```rust
// Create a basic transfer effect
let effect = Effect::transfer(from, to, "eth", 1_000_000_000);

// Add authorization
let authorized = effect.with_auth(Authorization::Signature {
    address: vec![1, 2, 3, 4],
    signature: vec![9, 8, 7, 6],
});

// Add a time condition
let conditional = authorized.with_condition(
    Condition::Time(TimeCondition::After(1679305200000))
);

// Add a timeout
let timed = conditional.with_timeout(1679391600000);
```

## Repeating Effects

TEL supports scheduled and repeating effects:

```rust
pub struct RepeatConfig {
    /// The schedule for repetition
    pub schedule: RepeatSchedule,
    /// Maximum number of iterations (safety limit)
    pub max_iterations: usize,
    /// Whether to retry on failure
    pub retry_on_failure: bool,
    /// Maximum number of retries for failed attempts
    pub max_retries: usize,
    /// Delay between retries
    pub retry_delay: Duration,
}

pub enum RepeatSchedule {
    /// Fixed number of repetitions
    Count(usize),
    /// Regular interval
    Interval(Duration),
    /// Repeat until a specific time
    Until(SystemTime),
    /// Indefinitely (limited by max_iterations)
    Indefinite,
}
```

## TelBuilder

TEL provides a builder to configure and create the TEL system with its dependencies:

```rust
pub struct TelBuilder {
    /// Unique ID for this TEL instance
    instance_id: ContentId,
    /// Config for resource verification
    verifier_config: Option<VerifierConfig>,
    /// Storage for snapshots
    snapshot_storage: Option<Box<dyn SnapshotStorage>>,
}

pub struct TelSystem {
    /// Unique ID for this TEL instance
    pub instance_id: ContentId,
    /// Resource manager for ResourceRegister management
    pub resource_manager: Arc<ResourceManager>,
    /// ZK verifier for validating operations on resources
    pub verifier: Arc<ZkVerifier>,
    /// Snapshot manager for ResourceRegister state persistence
    pub snapshot_manager: Arc<SnapshotManager>,
    /// Version manager for tracking resource versions
    pub version_manager: Arc<VersionManager>,
    /// Effect adapter for applying operations to ResourceRegisters
    pub effect_adapter: Arc<ResourceEffectAdapter>,
}
```

## Examples

### Basic Transfer Example

```rust
// TEL script for a basic transfer
let tel_source = r#"
{
    "operation": "transfer",
    "domain": "ethereum",
    "parameters": {
        "from": "0x1234...5678",
        "to": "0xabcd...ef01",
        "asset": "eth",
        "amount": "1000000000000000000"
    }
}
"#;

// Parse and execute
let results = execute_tel(tel_source, compiler, context).await?;
```

### Cross-Domain Transfer Example

```rust
// TEL script for a cross-domain transfer
let tel_source = r#"
{
    "operation": "sequence",
    "children": [
        {
            "operation": "transfer",
            "domain": "ethereum",
            "parameters": {
                "from": "0x1234...5678",
                "to": "bridge-address",
                "asset": "eth",
                "amount": "1000000000000000000"
            }
        },
        {
            "operation": "transfer",
            "domain": "cosmos",
            "parameters": {
                "from": "bridge-address",
                "to": "cosmos1abc...def",
                "asset": "atom",
                "amount": "10000000"
            }
        }
    ]
}
"#;

// Parse and execute
let results = execute_tel(tel_source, compiler, context).await?;
```

### Conditional Execution Example

```rust
// TEL script with conditional execution
let tel_source = r#"
{
    "operation": "conditional",
    "children": [
        {
            "operation": "query",
            "domain": "ethereum",
            "parameters": {
                "function": "balanceOf",
                "arguments": {
                    "account": "0x1234...5678",
                    "token": "0xdcba...9876"
                }
            }
        },
        {
            "operation": "transfer",
            "domain": "ethereum",
            "parameters": {
                "from": "0x1234...5678",
                "to": "0xabcd...ef01",
                "asset": "eth",
                "amount": "1000000000000000000"
            }
        },
        {
            "operation": "noop"
        }
    ]
}
"#;

// Parse and execute
let results = execute_tel(tel_source, compiler, context).await?;
```

## Implementation in Causality

The TEL system is implemented in Causality across several crates:

1. **causality-tel**: Core TEL implementation, including script parsing, operation handling, and effect creation
2. **causality-effects**: Effect system integration for executing TEL operations
3. **causality-domain**: Domain-specific adapters for integrating with different blockchains
4. **causality-resource**: Resource system for managing assets across domains

## Conclusion

The Temporal Effect Language (TEL) provides a powerful, domain-agnostic way to express operations across multiple blockchain platforms. By abstracting away the details of specific domains, TEL enables developers to write portable code that can be executed anywhere while maintaining strong guarantees about resource conservation and temporal ordering.

TEL's integration with the Effect System, Domain System, and Resource System makes it a central component in the Causality architecture, serving as the bridge between high-level application logic and low-level blockchain operations. 