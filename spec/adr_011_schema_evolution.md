# ADR-011: Autonomous Schema Evolution and Hostage Prevention in Causality

Status:

Partially Implemented

## Implementation Status

The Autonomous Schema Evolution and Hostage Prevention systems described in this ADR have been partially implemented. The codebase shows evidence of the core concepts, but with some components still in development.

1. **Schema Definition System**:
   - Basic schema definition structures exist in `/src/schema/definition.rs` including `Schema`, `SchemaField`, and `SchemaType`
   - Type compatibility checking is implemented to ensure schema changes preserve backward compatibility
   - Serialization/deserialization support in JSON and TOML formats

2. **Evolution Rules Engine**:
   - Implementation of the core evolution rules system in `/src/schema/evolution.rs`
   - Support for the change types described in the ADR: AddOptionalField, AddDefaultField, RemoveUnusedField, etc.
   - Rule-based validation of schema changes to ensure they follow allowed evolution patterns

3. **Safe State Management**:
   - Safe state system implemented in `/src/schema/safe_state.rs` with multiple strategies:
     - NoPendingReturns (as recommended in the ADR)
     - NoInFlightOperations
     - NoActiveOperators
   - Transaction model for schema updates that ensures safety and provides rollback capability
   - Timeout handling for long-running operations

4. **Resource Versioning Alternative**:
   - An alternative approach using resource versioning is implemented in `/src/resource/versioning.rs`
   - This provides similar functionality but focused specifically on resources rather than generic schemas
   - Includes migration paths and compatibility checking

The implementation represents a significant portion of the ADR's recommended design, though some components (particularly the migration system) appear to be in early stages of development. The project may be using the resource versioning system as a more targeted alternative to the general schema evolution system.

## Autonomous Schema Evolution

The goal of **autonomous schema evolution** is to allow **program upgrades to happen without requiring User-written migrations** — as long as the schema change falls into **a predictable class of non-breaking changes**.

This fits well with the **default safe upgrade path** where programs automatically evolve along with the protocol, provided they are in a **safe state** at the time of the upgrade.


## What Kinds of Schema Changes Can Be Auto-Evolved?

| Change Type | Example | Automatically Safe? |
|---|---|---|
| Add New Field (Optional) | Add `riskTolerance` to program state | - Yes |
| Add New Field (With Default) | Add `maxSlippage` with default of `0.01` | - Yes |
| Remove Unused Field | Remove deprecated `legacyCounter` | - Yes |
| Change Field Type (Coercible) | Change `ethBalance` from `Int` to `Decimal` | ⚠️ Safe if lossless coercion |
| Rename Field | Rename `price` to `oraclePrice` | ❌ Requires explicit migration |
| Add New Effect Type | Add `ObserveOracle` | - Yes (if old Operators ignore) |
| Modify Effect Payload | Add `timestamp` to `DepositEffect` | ⚠️ Requires careful coordination |


## Recommended Approach: Schema Evolution Rules

Each **program schema version** includes:

```toml
[schema]
version = "1.2.0"
fields = ["balances", "lastPrice", "riskTolerance"]

[schema.evolution]
allowed = ["add-optional-field", "add-default-field", "remove-unused-field"]
```

This allows Operators to:
- Detect when schema changes are **within allowed rules**.
- Auto-apply changes to the serialized program state.
- Validate evolution as part of **effect validation**.


## Evolution Engine Example

```rust
enum EvolutionError {
    IncompatibleSchemaChange,
    // Other error types
}

fn apply_schema_evolution(
    old_state: &ProgramState, 
    new_schema: &NewSchema
) -> Result<ProgramState, EvolutionError> {
    if evolution_is_safe(&old_schema, new_schema) {
        Ok(migrate_fields(old_state, &old_schema, new_schema))
    } else {
        Err(EvolutionError::IncompatibleSchemaChange)
    }
}
```


## Program Declaration Example

```toml
[program]
name = "CrossDomainArb"
version = "1.2.0"
schemaVersion = "1.2.0"
safeStateStrategy = "no-pending-returns"
```


## Example: Add a Field Automatically

Original State (v1.2.0):
```json
{
    "balances": { "ETH": 100 },
    "lastPrice": { "ETH/USDC": 2900 }
}
```

Schema Evolution (v1.3.0 adds `riskTolerance`):
```toml
[schema]
version = "1.3.0"
fields = ["balances", "lastPrice", "riskTolerance"]

[schema.defaultValues]
riskTolerance = 0.05
```

Auto-migrated State:
```json
{
    "balances": { "ETH": 100 },
    "lastPrice": { "ETH/USDC": 2900 },
    "riskTolerance": 0.05
}
```


## Benefits of This Approach

- Users do nothing for common schema changes.  
- Strong typing and schema evolution rules guarantee compatibility.  
- Every schema change is **documented in the program history**.  
- Operators refuse to apply unsafe changes (renames, type changes).


## When Migration Functions Are Still Required

| Case | Example |
|---|---|
| Semantic Changes | Change `riskTolerance` from being per-asset to global. |
| Field Renames | Rename `price` to `oraclePrice`. |
| Field Splits | Replace `spread` with `{askPrice, bidPrice}`. |
| Aggregation Logic | Add a rolling average that needs to bootstrap from history. |

For these, Users provide a **MigrationFunction** that is:
- Pure.
- Content-addressed.
- Logged in the **UpgradeProgram** effect.


# Hostage Situation Prevention


## Context

In cross-program workflows, **Program A** may call **Program B**.  
If Program A can only upgrade when it is in a **safe state**, this creates a risk:  
- **Program B could intentionally stall the return value**.
- **Program A becomes stuck — unable to upgrade until B cooperates**.


## Options to Mitigate Hostage Risk

### Option 1: Strict No Pending Returns (Hard Rule)

- Programs **cannot make cross-program calls** unless they explicitly allow deferring their own upgrade.
- This means programs either:
    - Operate **fully standalone**.
    - Or explicitly handle the risk of depending on others.

- Simple and predictable.  
❌ Severely limits composability.  


### Option 2: Timeout and Auto-Abort (Graceful Degrade)

- If a call to Program B doesn't return within a deadline, Program A enters a **degraded state**.
- This state logs the timeout and:
    - Either ignores the result permanently.
    - Or substitutes a default result.
- This allows upgrades to proceed from the degraded state.

- Keeps composability.  
- Avoids permanent deadlock.  
❌ Some loss of flexibility in program logic.


### Option 3: Optional Lock Contracts

- Programs could register **lock contracts** with Operators.
- A lock contract governs **under what conditions a program can hold another hostage**.
- Example: A program can only delay another program if both were deployed together (composable pair).

- Highly flexible.  
- Explicit opt-in.  
❌ Complex to implement.


### Option 4: Explicit Safe State Handshakes

- When Program A calls Program B, it sends a **safe state intent**.
- This declares:
    - When Program A expects to upgrade.
    - What state it expects to be in at that time.
- Program B either:
    - Acknowledges this (promising to return before then).
    - Or refuses the interaction.

- Formalizes expectations up front.  
❌ Adds messaging overhead.


## Recommended Approach

| Component | Approach |
|---|---|
| Default | Option 2: Timeout and Auto-Abort |
| Programs That Never Upgrade Mid-Run | Option 1: Strict No Pending Returns |
| Highly Coupled Programs | Option 3: Lock Contracts |
| Programs with High-Value State Transitions | Option 4: Safe State Handshake |


## Example Timeout Rule

```toml
[safeStateStrategy]
type = "no-pending-returns"
timeout = "15 minutes"
degradedState = { status = "timeout", reason = "awaiting ProgramB" }
```


## Example Safe State Handshake

When A calls B:
```json
{
    "call": "doRiskCheck",
    "args": { "balance": 100 },
    "safeStateIntent": {
        "expectedReturnTime": "2025-06-01T12:00:00Z",
        "expectedSafeState": "no-pending-returns"
    }
}
```

Program B either acknowledges:
```json
{
    "ack": true
}
```
Or refuses:
```json
{
    "ack": false,
    "reason": "Program B does not support guaranteed return times"
}
```


# Key Takeaways

| Mechanism | Goal |
|---|---|
| Schema Evolution Rules | Allow no-touch upgrades for Users. |
| Migration Functions | Only required for semantic shifts. |
| Safe State Policies | Define how and when programs can upgrade. |
| Timeout Handling | Default safeguard against hostage risk. |
| Safe State Handshake | Optional stricter commitment mechanism. |


# Summary Recommendations

- Standardize **schema evolution rules** in the protocol.  
- Make **Users opt into a safe state policy** at program deploy time.  
- Use **timeouts by default** for pending return mitigation.  
- Allow optional safe state handshakes for Users who want stronger guarantees.  
- Document all safe state policies and program version history in the effect log.


# Addendum to ADR 010: Compiler and Deployment Architecture

## Context

This addendum extends ADR 010 to address compiler implementation, deployment workflows, and extension mechanisms within the Causality system. Following our core principle of User sovereignty, we need a cohesive architecture that enables Users to compile, deploy, and extend programs while maintaining the system's integrity guarantees.

## Key Decisions

### 1. Account Programs as Deployment Gateways

All deployment operations must flow through account programs, maintaining our "programs talk to programs" invariant:

```rust
enum AccountProgramEffect {
    Deposit { asset: Asset, amount: Amount },
    Withdraw { asset: Asset, amount: Amount },
    DeployProgram { compiled_program: CompiledProgram, deployment_config: DeploymentConfig },
    DeployEffect { custom_effect: CustomEffect },
    DeployDomainAdapter { domain_adapter: DomainAdapter },
    ActivateProgram { program_id: ProgramID, activation_strategy: ActivationStrategy },
}
```

This unified approach enables both User-initiated and program-initiated deployments to follow the same pipeline, preserving causality and audit trails.

### 2. Explicit Compiler Architecture

The TEL compiler transforms source programs into deployable artifacts through a series of validated transformations:

1. **Parse** TEL source into AST
2. **Type Check** program structure and resource usage
3. **Effect Validate** causal relationships and temporal properties
4. **Transform** to an Effect-Based IR
5. **Optimize** for targeted execution environment
6. **Generate** content-addressed deployment artifacts

Each stage enforces guarantees that ensure program correctness before deployment.

### 3. Configurable Deployment Activation Strategies

Programs may specify their preferred activation strategy:

```rust
enum ActivationStrategy {
    ManualStepwise,                // Activate each component separately after verification
    FloodActivation,               // Last component triggers parallel activation messages
    DomainedActivation(OrderSpec), // Activation proceeds in specified order
    ConditionBased(Vec<Condition>), // Activate when specific conditions are met
}
```

This enables complex cross-domain coordination patterns while preserving User control over the activation sequence.

### 4. System Extensibility via Custom Effects and Domain Adapters

Users may extend the system by deploying:
- **Custom Effects** that add new capabilities to the effect system
- **Domain Adapters** that enable interaction with new chains

These extensions are deployed through account programs and tracked using the content-addressable code system, enabling composition and reuse.

### 5. Version Compatibility Management for Custom Extensions

Custom effects and adapters must declare their compatibility with protocol versions:

```toml
[effect]
name = "ObserveCustomOracle"
version = "1.2.0"

[effect.compatibility]
protocol_versions = ["2.x", "3.x"]
evolution_rules = ["add-optional-parameter", "refine-return-type"]

[effect.handler]
hash = "bafy123..."

[effect.fallbacks]
protocol = "2.x"
strategy = "use-alternative-implementation"
alternative_handler = "bafy456..."
```

This ensures that extensions can evolve alongside the protocol without breaking existing programs.

## Implementation Details

### Compiler Pipeline

The compiler will output artifacts in a uniform format that includes:

```rust
struct CompiledProgram {
    program_hash: ContentHash,           // Content hash of the entire program
    effect_dag: EffectDAG,               // The core effect structure
    dependencies: std::collections::HashSet<ContentHash>, // All code/effect dependencies
    schema: Schema,                      // Program state schema
    schema_evolution_rules: Vec<EvolutionRule>, // Allowed schema changes
    compatible_protocol_versions: VersionRange, // Compatible protocol versions
}
```

The content-addressable nature of this representation ensures that programs are immutable and can be precisely referenced.

### Deployment Flow

The deployment sequence will follow this pattern:

1. User **compiles** program locally
2. User **initiates deployment** through their account program
3. Account program **distributes** program to Operators
4. Operators **register** program components on relevant Domains
5. Committees **observe and verify** successful deployment
6. User or program **activates** deployment according to chosen strategy

### Custom Effect Registration

Custom effects will be registered in a global effect registry that tracks:

```rust
struct EffectRegistry {
    effect_definitions: std::collections::HashMap<EffectID, EffectDefinition>,
    effect_handlers: std::collections::HashMap<(EffectID, Version), ContentHash>,
    effect_compatibility: std::collections::HashMap<EffectID, Vec<ProtocolCompatibility>>,
}
```

This registry ensures that effects are consistently available across the Operator network.

## Consequences

### Positive

- **Uniform Deployment Model**: Same mechanism for all deployments maintains simplicity
- **Extensibility**: Users can extend the system with new effects and Domain adapters
- **Sovereignty Preservation**: All deployments remain under User control
- **Upgrade Safety**: Version compatibility declarations prevent breaking changes
- **Composability**: Content-addressed components enable safe composition and reuse

### Challenges

- **Partial Deployment Handling**: We need robust recovery mechanisms for partial deployment failures
- **Effect Compatibility Matrix**: Managing compatibility between protocol versions and custom effects will grow in complexity
- **Testing Requirements**: Comprehensive testing of custom components requires sophisticated simulation capabilities

## Implementation Plan

We recommend implementing this architecture in phases:

1. **Compiler Foundation**: Basic TEL compiler with effect validation
2. **Deployment Pipeline**: Core deployment flow through account programs
3. **Activation Strategies**: Support for different activation patterns
4. **Custom Effects**: Extension mechanism for effect system
5. **Domain Adapters**: Extension mechanism for chain support

Each phase should be implemented with full testing and documentation before proceeding to the next.

## Additional Considerations

The interaction between custom effects and schema evolution requires careful consideration. While schema evolution applies to program state, effect evolution applies to the protocol's extension points. We should develop clear guidelines for when to evolve an effect versus creating a new one.

When extending the system with new Domain adapters, we should consider a vetting or reputation system to ensure that adapters correctly implement chain interfaces, particularly for security-critical operations like signature verification.

As the ecosystem grows, we may need a discovery mechanism for custom components, potentially including a registry or marketplace for Users to find and evaluate extensions.