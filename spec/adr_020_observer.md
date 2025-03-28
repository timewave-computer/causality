# ADR-020: Program Observability System

## Status

Accepted, Partially Implemented

## Implementation Status

The Program Observability System described in this ADR has been partially implemented with the following components:

### Implemented Components
- **Fact Observer System**: Core fact observation infrastructure (`src/domain/fact/observer.rs`) provides a mechanism for observing and caching facts from various domains
- **Specialized Observers**: Several specialized observers are implemented, including the `ZKProofFactObserver` (`src/domain/fact/zkproof_observer.rs`) which provides specialized observation for ZK proofs
- **Fact Logging**: The `FactLogger` system supports logging observed facts for later retrieval
- **Resource Fact Observer**: A resource-specific observer (`src/resource/fact_observer.rs`) for tracking resource-related facts
- **Fact Replay and Simulation**: The `FactReplay` and `FactSimulator` systems support replaying and simulating observed facts

### Partially Implemented Features
- **Event Storage**: Basic event storage exists but lacks comprehensive materialized views
- **Basic Fact Snapshots**: The `FactSnapshot` system provides basic snapshots but not full visualization support
- **Domain-Specific Observation**: Domain-specific fact observation is partially implemented

### Missing Components
- **Comprehensive Visualization Layer**: The visualization components described in the ADR are not implemented
- **Query Engine**: The SQL-like query capabilities are not implemented
- **Event Materializers**: The advanced materializers for different view types are not implemented
- **Compiler Integration**: Integration with compilation events is not implemented
- **CLI Interface**: The `tb-observe` interface described in the ADR is not implemented
- **Cross-Phase Integration**: The complete cross-phase integration is not implemented

## Context

Causality programs operate across multiple Domains with complex causal relationships, resource flows, and state transitions. Understanding program behavior in such a distributed environment presents significant challenges:

- It's difficult to visualize causal relationships spanning multiple Domains
- Debugging cross-domain issues requires correlated views of distributed state
- Developers need insights into resource flows and program execution paths
- Domain-specific issues require visibility into protocol-level details
- Compilation errors and deployment status need clear, actionable feedback

Our current approach lacks a comprehensive observability system that provides visibility across the full lifecycle from compilation through simulation to production execution.

## Decision

We will implement a dedicated observability system that provides materialized views of compilation, simulation, deployment, and runtime execution. This system will serve as a core pillar of the Causality architecture, complementing the compiler, deployment, and simulation components.

### System Architecture

The observability system will have these core components:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Event Sources  │───▶│ Event Collectors│───▶│  Event Storage  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Visualizers    │◀───│  Query Engine   │◀───│  Materializers  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

The system leverages our existing content-addressable, append-only logs while adding:
1. Specialized collectors to extract and correlate events
2. Materializers that transform raw events into coherent views
3. A query engine for retrieving and filtering observability data
4. Visualization components for rendering different aspects of program behavior

### Event Sources

The observability system will collect events from multiple sources:

#### 1. Compilation Events

The compiler will emit structured events during compilation, capturing:
- Type errors and warnings with source context
- Effect validation results with causality checks
- Resource usage analysis including flow paths
- Dependency resolution and version compatibility issues

```rust
enum CompilationEvent {
    TypeError { source_span: SourceSpan, error: TypeError },
    EffectValidation { effect_id: EffectID, result: ValidationResult },
    ResourceFlow { resource_id: ResourceID, flow_path: FlowPath },
    Dependency { dependency_id: DependencyID, resolution: Resolution },
}
```

#### 2. Deployment Events

The deployment system will emit events tracking:
- Deployment initiation and progress
- Domain-specific deployment status
- Activation progress and coordination events
- Cross-domain synchronization points
- Version compatibility checks

```rust
enum DeploymentEvent {
    DeployInitiated { deployment_id: DeploymentID, program_id: ProgramID },
    DomainDeployment { deployment_id: DeploymentID, domain_id: DomainID, status: DeployStatus },
    Activation { deployment_id: DeploymentID, strategy: ActivationStrategy, status: ActivationStatus },
    Synchronization { deployment_id: DeploymentID, domains: Vec<DomainID>, status: SyncStatus },
}
```

#### 3. Runtime Events

The runtime will generate events capturing program execution:
- Effect application and results
- Resource transfers and transformations
- Domain observations and fact consumption
- Cross-program invocations
- Schema transformations

```rust
enum RuntimeEvent {
    EffectApplication { effect_id: EffectID, result: Result },
    ResourceTransform { resource_id: ResourceID, transformation_type: TransformationType },
    FactObservation { fact_id: FactID, domain: Domain },
    Invocation { program_id: ProgramID, entry_point: EntryPoint, arguments: Arguments },
    SchemaTransformation { old_schema: OldSchema, new_schema: NewSchema },
}
```

#### 4. Simulation Events

The simulation system will emit events specific to simulation contexts:
- Actor startup and shutdown
- Injected faults and delays
- Domain state manipulations
- Synthetic facts and mocked responses
- Scenario progress markers

```rust
enum SimulationEvent {
    ActorLifecycle { actor_id: ActorID, status: LifecycleStatus },
    FaultInjection { fault_type: FaultType, target: Target },
    DomainManipulation { domain_id: DomainID, manipulation_type: ManipulationType },
    SyntheticFact { fact_id: FactID, fact_type: FactType },
    ScenarioProgress { scenario_id: ScenarioID, milestone: Milestone },
}
```

### Event Collectors

Event collectors will extract and correlate events from multiple sources:
- **Log Collectors**: Process unified logs and factlogs
- **Compiler Collectors**: Extract diagnostics from compiler outputs
- **Simulation Collectors**: Gather events from simulation runs
- **Deployment Collectors**: Track deployment status across Domains

Each collector will normalize events into a consistent format with:
- Unique event ID
- Timestamp (wall clock + logical/Lamport)
- Source identifier
- Event type
- Event payload
- Causal parent references
- Contextual metadata

### Event Storage

Events will be stored in:
1. **Append-only logs**: Content-addressed, immutable event history
2. **Materialized views**: Derived representations optimized for specific queries
3. **Indexed repositories**: Searchable stores with relationship metadata

All storage components will maintain causal consistency, allowing for accurate representation of happens-before relationships across different Domains and execution contexts.

### Materializers

Materializers will transform raw events into meaningful views:

#### 1. Causal Graphs

Materializers will construct causal graphs showing:
- Effect dependencies and execution order
- Cross-program invocation Domains
- Domain observation dependencies
- Resource flow paths

These graphs will be rendered as directed acyclic graphs (DAGs) with:
- Nodes representing effects, facts, and operations
- Edges representing causal dependencies
- Domain boundaries clearly marked
- Resource flows visualized along edges

#### 2. State Transition Views

State transition views will show how program state evolves:
- Schema transformations during upgrades
- Resource balance changes
- Effect application results
- Observed fact integration

#### 3. Domain-Specific Views

Domain-specific views will highlight:
- Block-by-block progress on each Domain
- Confirmation status of transactions
- Fork detection and resolution
- Domain-specific error conditions
- Protocol-level details for debugging

#### 4. Resource Flow Maps

Resource flow maps will visualize:
- Asset movements between actors
- Token flow across Domains
- Balance changes over time
- Resource creation and destruction points

```rust
enum MaterializedView {
    CausalGraph { representation: GraphRepresentation },
    StateTransitionView { program_id: ProgramID, state_sequence: StateSequence },
    DomainView { domain_id: DomainID, block_sequence: BlockSequence },
    ResourceFlowMap { resource_ids: Vec<ResourceID>, flow_representation: FlowRepresentation },
}
```

### Query Engine

The query engine will provide SQL-like capabilities for observability data:

```sql
-- Example queries

-- Find all events related to a specific effect
SELECT * FROM events WHERE effect_id = 'bafy123...' ORDER BY timestamp;

-- Find resource flow path from source to destination
SELECT * FROM resource_flows 
WHERE source_id = 'account1' AND destination_id = 'account2';

-- Find cross-domain causality Domains
SELECT * FROM causal_Domains 
WHERE spans_Domains('ethereum', 'solana') 
ORDER BY start_time;

-- Find all errors during deployment
SELECT * FROM events 
WHERE deployment_id = 'dep-123' AND event_type = 'error' 
ORDER BY timestamp;
```

The engine will support:
- Complex filters and projections
- Aggregations across event streams
- Temporal queries with time windows
- Causal relationship traversal
- Cross-domain correlations

### Visualization Layer

The observability system will include multiple visualization components:

#### 1. Compiler Diagnostic Visualizers

- Type error explainers with source context
- Resource flow diagrams showing linearity violations
- Effect causality graphs highlighting ordering issues
- Dependency graphs showing version constraints

#### 2. Runtime Flow Visualizers

- Live program execution traces
- Domains showing effect application sequence
- Resource movement animations
- Cross-domain causality maps
- State transition diagrams

#### 3. Deployment Status Dashboards

- Cross-domain deployment progress
- Activation status and synchronization points
- Version compatibility matrices
- Domain-specific deployment details

#### 4. Simulation Visualizers

- Actor interaction diagrams
- Injected fault impacts
- Domain manipulations
- Scenario progression visualization

All visualizations will be available through:
- CLI-based ASCII representations
- Web dashboards with interactive elements
- Exportable SVG/PNG formats for documentation

### Cross-Phase Integration

The observability system will integrate information across different phases of the program lifecycle:

- **Compilation ↔ Simulation**: Link compiler warnings to runtime issues
- **Simulation ↔ Deployment**: Compare simulated vs. actual behavior
- **Deployment ↔ Runtime**: Track how deployment choices affect execution
- **Runtime ↔ Compilation**: Identify patterns that could be caught statically

This cross-phase integration provides a comprehensive view of program behavior from source code to execution.

### CLI Interface

The observability system will provide a CLI interface:

```bash
# Get real-time visualization of program execution
tb-observe trace program-123

# Show resource flow for a specific asset
tb-observe resource-flow USDC --program program-123

# Compare simulation vs. production behavior
tb-observe compare --sim sim-456 --prod program-123

# Generate a causal graph for a deployment
tb-observe causal-graph dep-789 --output graph.svg

# Generate a full execution report
tb-observe report program-123 --format markdown --output report.md
```

### Integration with Content-Addressable System

The observability system will leverage our content-addressable storage:
- Each event is content-addressed for immutability and verifiability
- Materializers produce content-addressed views
- Visualization outputs are content-addressed for reproducibility
- Queries reference content-addressed data via hash

This ensures that observability data maintains the same trust and verification properties as the core system.

## Consequences

### Positive

- **Comprehensive Visibility**: Unified view across the entire program lifecycle
- **Improved Debugging**: Rich context for identifying and resolving issues
- **Cross-domain Clarity**: Clear visualization of complex causal relationships
- **Resource Transparency**: Explicit tracking of all resource movements
- **Developer Experience**: Faster feedback cycles and clearer error resolution
- **Auditability**: Improved validation of correctness properties

### Challenges

- **Storage Overhead**: Comprehensive observability generates substantial data
- **Performance Implications**: Collection may impact runtime performance
- **Complexity Management**: Users may be overwhelmed by information
- **Privacy Considerations**: Some programs may not want all details visible
- **Scaling Visualization**: Representing complex distributed programs is difficult

### Visualization Challenges

The observability system faces several visualization challenges:

- **Causal Complexity**: Representing happens-before relationships across many Domains
- **Resource Flow Clarity**: Showing resource movement without overwhelming detail
- **Time Representation**: Balancing wall clock, logical, and block-based time
- **Error Context**: Providing sufficient context for effective debugging
- **Scale Management**: Handling very large programs with many effects

These challenges require careful UI/UX design and progressive disclosure of information.

## Implementation Plan

We will implement the observability system in phases:

1. **Foundation**: Event collection and storage infrastructure
2. **Compiler Integration**: Link compiler diagnostics to observability system
3. **Runtime Tracing**: Implement runtime event collection
4. **Basic Visualizations**: Core visualizers for common patterns
5. **Query Engine**: Advanced querying capabilities
6. **Cross-Phase Integration**: Link information across compilation/simulation/runtime
7. **Advanced Visualizations**: Complex visualizations for specialized use cases

Each phase will focus on delivering immediate developer value while building toward the complete system.

## Additional Considerations

### Privacy and Control

Programs may have different observability requirements. The system will support:
- **Selective Disclosure**: Control over which events are published
- **Permissioned Access**: Fine-grained control over who can access observability data
- **Local-Only Mode**: Option to keep observability data entirely local

### Integration with Time Map

The observability system will integrate deeply with the Map of Time, providing:
- Domain-specific context for each observation
- Causal ordering information across Domains
- Fact verification for observed external state

### Artifact Relationship Tracking

The system will track relationships between artifacts:
- Compiled programs linked to their source code
- Deployed programs linked to their compiled artifacts
- Runtime instances linked to their deployed versions

This enables tracing issues from runtime back to source code and vice versa.