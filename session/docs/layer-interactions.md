# Layer Interactions

This document details how the four layers of the Causality-Valence architecture interact with each other, including data flow, compilation processes, and inter-layer communication.

## Overview of Layer Relationships

```
Layer 3 (Choreography) ──compiles to──> Layer 2 (Effects)
Layer 2 (Effects)      ──compiles to──> Layer 1 (Sessions)  
Layer 1 (Sessions)     ──compiles to──> Layer 0 (Instructions)
```

Each layer serves as both a compilation target and a compilation source, creating a clean separation of concerns while maintaining end-to-end functionality.

## Layer 0: Content-Addressed Message Machine

### Layer 0 Instructions

| Instruction | Syntax | Description | Purpose |
|-------------|--------|-------------|----------|
| `create` | `create r_val r_msg` | Creates a new content-addressed message from a value | Fundamental message creation primitive - converts values to content-addressed resources |
| `consume` | `consume r_msg r_val` | Extracts value from message and destroys the message | Enforces linear consumption - ensures messages are used exactly once |
| `send` | `send r_msg r_chan` | Sends a message through a channel | Basic communication primitive - moves messages between contexts |
| `receive` | `receive r_chan r_msg` | Receives a message from a channel | Completes communication pair - enables message-based coordination |
| `match` | `match r_msg branches` | Pattern matches on message structure for branching | Enables conditional execution based on message content |

**Design Rationale**: Layer 0 provides the minimal substrate for all message operations. These 5 instructions are sufficient for universal computation while maintaining linear resource semantics and deterministic execution. No types, no structure - just pure message mechanics.

## Layer 1: Linear Session Calculus

### Layer 1 Primitives

| Primitive | Type Signature | Description | Purpose |
|-----------|----------------|-------------|----------|
| **Type System** |
| `Unit` | `Type` | Unit type (empty value) | Represents computations with no meaningful return value |
| `Bool` | `Type` | Boolean type | Basic logical values for conditionals |
| `Int` | `Type` | Integer type | Numeric computations and quantities |
| `Product` | `Type × Type → Type` | Pair/tuple types | Combines multiple values structurally |
| `Sum` | `Type + Type → Type` | Either/union types | Represents choice between alternatives |
| `Message` | `RowType → Type` | Linear message types | Typed wrapper around Layer 0 messages |
| `Session` | `SessionType → Type` | Session channel types | Typed communication channels |
| **Row Types** |
| `Empty` | `RowType` | Empty row (no fields) | Base case for record types |
| `Extend` | `Label × Type × RowType → RowType` | Add field to row | Build structured record types |
| `RowVar` | `Variable → RowType` | Row type variable | Enable row polymorphism |
| **Session Types** |
| `Send` | `RowType × SessionType → SessionType` | Send message, continue | Output communication protocol |
| `Receive` | `RowType × SessionType → SessionType` | Receive message, continue | Input communication protocol |
| `Choice` | `SessionType ⊕ SessionType` | Offer choice to partner | Internal choice in protocols |
| `Branch` | `SessionType & SessionType` | Accept choice from partner | External choice in protocols |
| `End` | `SessionType` | Terminated session | End of communication protocol |
| **Term Constructors** |
| `Unit` | `Term` | Unit value constructor | Create unit values |
| `Bool` | `Bool → Term` | Boolean value constructor | Create boolean values |
| `Int` | `Int → Term` | Integer value constructor | Create integer values |
| `Pair` | `Term × Term → Term` | Pair constructor | Combine two terms |
| `Fst` | `Term → Term` | First projection | Extract first component of pair |
| `Snd` | `Term → Term` | Second projection | Extract second component of pair |
| `Inl` | `Term → Term` | Left injection | Inject into sum type (left) |
| `Inr` | `Term → Term` | Right injection | Inject into sum type (right) |
| `Case` | `Term × (Term → Term) × (Term → Term) → Term` | Sum elimination | Pattern match on sum types |
| `Record` | `BTreeMap<String, Term> → Term` | Record constructor | Create structured records |
| `Project` | `Term × String → Term` | Field projection | Access record fields |
| `Send` | `Term × Term × Term → Term` | Send term constructor | Send message through session |
| `Receive` | `Term → Term` | Receive term constructor | Receive message from session |
| `Close` | `Term → Term` | Close session | Terminate session protocol |
| **Linear Type Checking** |
| `split_context` | `Context → Context × Context` | Split linear context | Distribute linear resources |
| `check_linearity` | `Context × Term → Bool` | Verify linear usage | Ensure single-use semantics |
| `type_check` | `Context × Term → Type` | Type checking | Verify term well-formedness |
| **Duality** |
| `dual` | `SessionType → SessionType` | Session type duality | Generate compatible session pairs |

**Design Rationale**: Layer 1 adds types and communication structure while preserving Layer 0's linear semantics. Row types enable extensible records and effect polymorphism. Session types ensure protocol safety through duality. The linear type system guarantees single-use message consumption.

## Layer 2: Verifiable Outcome Algebra

### Layer 2 Primitives

| Primitive | Type Signature | Description | Purpose |
|-----------|----------------|-------------|----------|
| **Outcomes** |
| `empty` | `Outcome` | Identity outcome (no changes) | Monoid identity for outcome composition |
| `compose` | `Outcome × Outcome → Outcome` | Combine outcomes | Algebraic composition of state changes |
| `verify` | `Outcome → Bool` | Verify outcome validity | Check cryptographic proof correctness |
| `commit` | `Outcome → Hash` | Generate outcome commitment | Create unique identifier for outcome |
| **State Transitions** |
| `Transfer` | `Address × Address × Amount → StateTransition` | Move resources between addresses | Model resource transfers |
| `Update` | `Location × Value × Value → StateTransition` | Change state value | Model state mutations |
| `Create` | `Location × Value → StateTransition` | Create new state | Model resource creation |
| `Delete` | `Location → StateTransition` | Remove state | Model resource deletion |
| **Effects with Row Types** |
| `Pure` | `A → Effect<A, ·>` | Pure value (no effects) | Monadic return for effect system |
| `Do` | `Operation × (A → Effect<B, ε>) → Effect<B, ε>` | Perform operation | Monadic bind for effects |
| `Transform` | `Handler<F,G> × Effect<A,F> → Effect<A,G>` | Apply handler transformation | Transform effect algebras |
| **Effect Rows** |
| `State` | `EffectRow` | State read/write operations | Model stateful computations |
| `Comm` | `EffectRow` | Communication operations | Model session-based communication |
| `Proof` | `EffectRow` | Proof generation/verification | Model cryptographic operations |
| `IO` | `EffectRow` | Input/output operations | Model external interactions |
| **Handlers (Natural Transformations)** |
| `identity` | `Handler<F,F>` | Identity handler | Natural transformation identity |
| `compose` | `Handler<F,G> × Handler<G,H> → Handler<F,H>` | Handler composition | Compose natural transformations |
| `state_handler` | `Handler<State+ε, ε>` | Handle state effects | Transform state effects to pure |
| `comm_handler` | `Handler<Comm+ε, ε>` | Handle communication effects | Transform communication to pure |
| `proof_handler` | `Handler<Proof+ε, ε>` | Handle proof effects | Transform proof ops to pure |
| **Interpreters** |
| `interpret_pure` | `Effect<A, ·> → (Outcome, A)` | Execute pure effects | Convert effects to outcomes |
| `interpret_state` | `Effect<A, State> → (Outcome, A)` | Execute state effects | Handle state operations |
| `interpret_comm` | `Effect<A, Comm> → (Outcome, A)` | Execute communication effects | Handle session operations |
| **Proofs** |
| `generate_proof` | `Claim × Witness → Proof` | Generate cryptographic proof | Create ZK proof of correctness |
| `verify_proof` | `Proof × Claim → Bool` | Verify cryptographic proof | Check proof validity |
| `compose_proofs` | `Proof × Proof → Proof` | Combine proofs | Algebraic proof composition |

**Design Rationale**: Layer 2 adds declarative outcomes and algebraic effects while preserving Layer 1's type safety. Handlers as natural transformations enable modular effect composition. Interpreters separate transformation from execution. The outcome algebra provides mathematical foundation for verification.

## Layer 3: Agent Orchestration

### Layer 3 Primitives

| Primitive | Type Signature | Description | Purpose |
|-----------|----------------|-------------|----------|
| **Agents** |
| `Agent::new` | `String → Agent` | Create new agent | Basic agent constructor |
| `add_capability` | `Agent × Capability → Agent` | Grant capability to agent | Extend agent permissions |
| `get_capabilities` | `Agent → Set<Capability>` | List agent capabilities | Inspect agent permissions |
| `set_supervisor` | `Agent × AgentId → Agent` | Set agent supervisor | Establish hierarchy |
| **Agent Registry** |
| `register` | `AgentRegistry × Agent → Result<(), Error>` | Register agent in system | Add agent to active set |
| `lookup` | `AgentRegistry × AgentId → Option<Agent>` | Find agent by ID | Agent discovery |
| `list_agents` | `AgentRegistry → Vec<AgentId>` | List all registered agents | System introspection |
| **Choreography Language** |
| `Send` | `AgentId × AgentId × Message → ChoreographyStep` | Agent sends message to agent | Basic communication step |
| `Spawn` | `AgentId × Agent → ChoreographyStep` | Agent spawns new agent | Dynamic agent creation |
| `Sequence` | `Vec<Choreography> → Choreography` | Sequential composition | Execute steps in order |
| `Parallel` | `Vec<Choreography> → Choreography` | Parallel composition | Execute steps concurrently |
| `Choice` | `Vec<Choreography> → Choreography` | Conditional execution | Branching choreography |
| **Capabilities** |
| `Communication` | `EffectRow → Capability` | Communication permission | Allow message sending/receiving |
| `StateAccess` | `Set<Location> → Capability` | State access permission | Allow reading/writing specific state |
| `ProofGeneration` | `Set<ClaimType> → Capability` | Proof generation permission | Allow creating proofs |
| `AgentSpawning` | `Set<AgentType> → Capability` | Agent creation permission | Allow spawning new agents |
| **Choreography Compilation** |
| `compile_choreography` | `Choreography × AgentRegistry → Result<Vec<Effect>, Error>` | Compile to Layer 2 effects | Convert high-level choreography |
| `validate_capabilities` | `ChoreographyStep × Agent → Result<(), Error>` | Check agent permissions | Capability-based access control |
| `optimize_parallel` | `Vec<ChoreographyStep> → Vec<ChoreographyStep>` | Optimize parallel execution | Performance optimization |
| **Message Types** |
| `Text` | `String → Message` | Text message | Simple string communication |
| `Data` | `Value → Message` | Structured data message | Rich data communication |
| `Request` | `RequestId × Value → Message` | Request message | Request-response pattern |
| `Response` | `RequestId × Value → Message` | Response message | Complete request-response |
| **Error Handling** |
| `AgentNotFound` | `AgentId → Error` | Agent lookup failure | Missing agent error |
| `MissingCapability` | `AgentId × Capability → Error` | Insufficient permissions | Capability error |
| `InvalidChoreography` | `String → Error` | Malformed choreography | Compilation error |
| `ExecutionFailure` | `String → Error` | Runtime execution error | Execution error |

**Design Rationale**: Layer 3 provides developer-friendly abstractions while preserving all lower-layer guarantees. Agents encapsulate computation with capabilities for security. Choreographies enable intuitive multi-party protocol specification. The capability system provides fine-grained access control based on effect rows.

## Layer 3 → Layer 2: Choreography to Effects

### Transformation Process

Choreographies are high-level descriptions of multi-party interactions that compile to algebraic effects representing the computational actions.

#### Send Operations
```rust
// Layer 3: Choreography step
ChoreographyStep::Send { 
    from: alice, 
    to: bob, 
    message: Message::Text("Hello".to_string()) 
}

// Compiles to Layer 2: Effect
Effect::Send { 
    to: bob,
    message: MessageValue::from("Hello"),
    continuation: Box::new(|msg_id| Effect::Pure(msg_id))
}
```

#### Parallel Composition
```rust
// Layer 3: Parallel choreography
ChoreographyStep::Parallel(vec![choreo1, choreo2])

// Compiles to Layer 2: Composed effects
let effect1 = compile_choreography(choreo1);
let effect2 = compile_choreography(choreo2);
Effect::Parallel(vec![effect1, effect2])
```

### Capability Translation

Agent capabilities are translated into effect row constraints:

```rust
// Layer 3: Agent with capabilities
Agent { 
    id: alice, 
    capabilities: vec![EffectRow::Comm, EffectRow::State] 
}

// Layer 2: Effect row constraint
Effect<T, Row> where Row: Contains<Comm> + Contains<State>
```

## Layer 2 → Layer 1: Effects to Sessions

### Effect-to-Session Mapping

Effects are compiled to session-typed terms that describe communication protocols.

#### Communication Effects
```rust
// Layer 2: Send effect
Effect::Send { to: bob, message: msg, continuation: k }

// Compiles to Layer 1: Send term
Term::Send(
    Channel::new("Alice→Bob"),
    msg,
    compile_continuation(k)
)
```

#### State Effects
```rust
// Layer 2: Read effect
Effect::Read { key: "balance", continuation: k }

// Compiles to Layer 1: Session protocol
Term::Session(
    SessionType::Send(Type::Message(Type::String), 
                     SessionType::Receive(Type::Message(Type::Int), 
                                        SessionType::End))
)
```

### Handler Compilation

Natural transformation handlers are compiled to session type transformations:

```rust
// Layer 2: Handler
impl Handler<StateRow, CommRow> for StateToComm {
    fn handle<T>(&self, effect: Effect<T, StateRow>) -> Effect<T, CommRow>;
}

// Layer 1: Session type transformation
fn compile_handler(session: SessionType<StateRow>) -> SessionType<CommRow>
```

## Layer 1 → Layer 0: Sessions to Instructions

### Term Compilation

Session-typed terms are compiled to sequences of Layer 0 instructions.

#### Message Operations
```rust
// Layer 1: Send term
Term::Send(channel, message, continuation)

// Compiles to Layer 0: Instructions
vec![
    Instruction::Create(value_register, message_register),
    Instruction::Send(message_register, channel_register),
    // ... continuation instructions
]
```

#### Type-Guided Compilation

Linear types guide register allocation and ensure single-use semantics:

```rust
// Layer 1: Linear context
Context { x: !Int.End, y: ?String.End }

// Layer 0: Register allocation
RegisterMap { 
    x: Register(1),  // Send channel
    y: Register(2),  // Receive channel
}
```

### Row Type Monomorphization

Row types are compiled to fixed record layouts:

```rust
// Layer 1: Row type
RowType::Extend("name", Type::String, 
          RowType::Extend("age", Type::Int, 
                          RowType::Empty))

// Layer 0: Pair encoding
MessageValue::Pair(
    Box::new(MessageValue::from("Alice")),
    Box::new(MessageValue::Pair(
        Box::new(MessageValue::Int(30)),
        Box::new(MessageValue::Unit)
    ))
)
```

## Cross-Layer Data Flow

### Message Lifecycle

1. **Layer 3**: Messages defined in choreography (high-level types)
2. **Layer 2**: Messages become effect continuations (algebraic structure)
3. **Layer 1**: Messages typed with session types (protocol structure)
4. **Layer 0**: Messages as content-addressed values (execution representation)

### Type Information Flow

```
Layer 3: Agent capabilities
    ↓
Layer 2: Effect row constraints  
    ↓
Layer 1: Session type annotations
    ↓
Layer 0: Runtime type checking
```

### Error Propagation

Errors propagate upward through the layers with increasing context:

```rust
// Layer 0: Basic execution error
Error::InvalidRegister(r1)

// Layer 1: Type context added
Error::TypeError { 
    expected: Type::Int, 
    found: Type::String,
    source: InvalidRegister(r1)
}

// Layer 2: Effect context added  
Error::EffectError {
    effect: "Send to Bob",
    effect_row: CommRow,
    source: TypeError { ... }
}

// Layer 3: Choreography context added
Error::ChoreographyError {
    step: 3,
    agents: [alice, bob],
    source: EffectError { ... }
}
```

## Inter-Layer Communication Patterns

### Compilation Pipeline

Each layer implements a compiler to the layer below:

```rust
// Layer 3 → Layer 2
pub trait ChoreographyCompiler {
    fn compile(choreography: &Choreography) -> Vec<Effect<(), Row>>;
}

// Layer 2 → Layer 1  
pub trait EffectCompiler {
    fn compile<T, R>(effect: &Effect<T, R>) -> Term;
}

// Layer 1 → Layer 0
pub trait TermCompiler {
    fn compile(term: &Term) -> Vec<Instruction>;
}
```

### Interpreter Coordination

The unified interpreter coordinates execution across layers:

```rust
impl Interpreter {
    pub fn execute_choreography(&mut self, choreo: Choreography) -> Result<Outcome, Error> {
        // Layer 3 → Layer 2
        let effects = self.compile_choreography(choreo)?;
        
        // Layer 2 → Layer 1
        let terms = self.compile_effects(effects)?;
        
        // Layer 1 → Layer 0  
        let instructions = self.compile_terms(terms)?;
        
        // Execute at Layer 0
        self.execute_instructions(instructions)
    }
}
```

### Capability Propagation

Capabilities flow downward through the compilation process:

1. **Layer 3**: Agents registered with capability sets
2. **Layer 2**: Effects constrained by capability rows
3. **Layer 1**: Session types include capability annotations
4. **Layer 0**: Runtime capability checking during execution

## Verification and Proof Generation

### Cross-Layer Verification

Verification properties are maintained across all layers:

```rust
// Layer 3: Choreography well-formedness
verify_choreography_safety(&choreography)

// Layer 2: Effect composition laws  
verify_effect_algebra(&composed_effect)

// Layer 1: Session type duality
verify_session_duality(&session_type)

// Layer 0: Linear consumption
verify_linear_usage(&instruction_sequence)
```

### Proof Composition

Proofs generated at each layer compose into a final verification artifact:

```
Layer 3 Proof: Choreography safety
    ∧
Layer 2 Proof: Effect correctness  
    ∧
Layer 1 Proof: Session compliance
    ∧
Layer 0 Proof: Execution trace
    ‖
Final Outcome Proof
```

## Performance Considerations

### Compilation Overhead

Each layer adds compilation overhead but enables optimization:

- **Layer 3**: High-level optimizations (choreography analysis)
- **Layer 2**: Effect fusion and handler optimization
- **Layer 1**: Type-directed optimization and register allocation
- **Layer 0**: Instruction-level optimization and caching

### Runtime Efficiency

The layered approach enables efficient execution:

1. Compile-time verification reduces runtime checks
2. Type erasure at lower layers improves performance
3. Layer-specific optimizations compound
4. Content addressing enables caching and deduplication

This layered architecture provides both mathematical rigor and practical efficiency while maintaining clear separation of concerns. 