# Architecture Overview

The Session crate implements a minimal prototype of the unified Causality-Valence architecture, a four-layer system for verifiable message-passing computation.

## Four-Layer Architecture

The system consists of four layers that compile down through a transformation pipeline:

1. **Layer 3**: Agent Orchestration - High-level choreographic programming
2. **Layer 2**: Verifiable Outcome Algebra - Algebraic effects and proofs  
3. **Layer 1**: Linear Session Calculus - Type-safe communication protocols
4. **Layer 0**: Content-Addressed Message Machine - Minimal execution engine

Each layer compiles to the layer below, creating a complete execution pipeline from high-level choreographies to low-level machine instructions.

## Architectural Principles

### 1. Linear Message Consumption
Every message is consumed exactly once, ensuring resource accountability and preventing double-spending or replay attacks.

### 2. Content Addressing
All messages are cryptographically identified using SHA256 hashing, enabling verifiable references and tamper detection.

### 3. Row Type Polymorphism
Extensible record and effect types allow for composable and modular system design without sacrificing type safety.

### 4. Natural Transformation Handlers
Effect handlers are mathematical natural transformations that compose cleanly, maintaining algebraic properties.

### 5. Verifiable Outcomes
All computations produce cryptographically verifiable outcomes with algebraic composition properties.

## Four-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Agent Orchestration                                │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │
│ │Choreography │ │ Capability  │ │   Agent     │             │
│ │   System    │ │   System    │ │ Management  │             │
│ └─────────────┘ └─────────────┘ └─────────────┘             │
└─────────────────────────────────────────────────────────────┘
                              │
                      Compiles to Effects
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Verifiable Outcome Algebra                         │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │
│ │   Effect    │ │   Handler   │ │  Outcome    │             │
│ │   System    │ │  Transform  │ │  Algebra    │             │
│ └─────────────┘ └─────────────┘ └─────────────┘             │
└─────────────────────────────────────────────────────────────┘
                              │
                      Compiles to Sessions
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Linear Session Calculus                           │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│ │    Type     │ │   Session   │ │     Row     │           │
│ │   System    │ │    Types    │ │    Types    │           │
│ └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
                              │
                      Compiles to Instructions
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: Content-Addressed Message Machine                 │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│ │   Machine   │ │   Content   │ │    5 Core   │           │
│ │    State    │ │ Addressing  │ │Instructions │           │
│ └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

## Layer 0: Content-Addressed Message Machine

The foundation layer provides a minimal virtual machine with deterministic execution semantics.

### Core Components

#### Machine State
```rust
pub struct MachineState {
    pub messages: HashMap<MessageId, MessageValue>,
    pub bindings: HashMap<Register, Binding>,
    pub instruction_pointer: usize,
}
```

#### Message Values
```rust
pub enum MessageValue {
    Unit,
    Bool(bool),
    Int(i64),
    Pair(Box<MessageValue>, Box<MessageValue>),
    Sum(bool, Box<MessageValue>), // Left(false) or Right(true)
}
```

#### Five Core Instructions
1. **`create r_val r_msg`**: Create content-addressed message from value
2. **`consume r_msg r_val`**: Extract value from message, destroying it (linear consumption)
3. **`send r_msg r_chan`**: Send message through channel
4. **`receive r_chan r_msg`**: Receive message from channel
5. **`match r_msg branches`**: Pattern match on message structure

### Key Properties
- **Deterministic Execution**: Same inputs always produce same outputs
- **Linear Resource Management**: Messages consumed exactly once
- **Content Addressing**: All messages have SHA256-based identifiers
- **Minimal Instruction Set**: Only 5 instructions needed for full expressivity

## Layer 1: Linear Session Calculus

Provides type safety and session-based communication protocols.

### Type System
```rust
pub enum Type {
    Unit,
    Bool,
    Int,
    Product(Box<Type>, Box<Type>),
    Sum(Box<Type>, Box<Type>),
    Message(Box<Type>),
    Session(SessionType),
    Record(RowType),
}
```

### Session Types
```rust
pub enum SessionType {
    Send(Box<Type>, Box<SessionType>),    // !T.S
    Receive(Box<Type>, Box<SessionType>), // ?T.S
    Choice(Vec<SessionType>),             // &{l1: S1, l2: S2, ...}
    Selection(Vec<SessionType>),          // ⊕{l1: S1, l2: S2, ...}
    End,
}
```

### Row Types
```rust
pub enum RowType {
    Empty,
    Extend(String, Type, Box<RowType>),
    RowVar(String),
}
```

### Key Properties
- **Session Type Duality**: Dual session types ensure protocol compatibility
- **Linear Type Checking**: Variables used exactly once
- **Row Polymorphism**: Extensible records and effects
- **Protocol Verification**: Session types guarantee communication safety

## Layer 2: Verifiable Outcome Algebra

Implements algebraic effects with natural transformation handlers and verifiable outcomes.

### Effect System
```rust
pub enum Effect<T, Row> {
    Pure(T),
    Send { to: AgentId, message: MessageValue, continuation: Box<dyn FnOnce(MessageId) -> Effect<T, Row>> },
    Receive { from: AgentId, continuation: Box<dyn FnOnce(MessageValue) -> Effect<T, Row>> },
    Read { key: String, continuation: Box<dyn FnOnce(MessageValue) -> Effect<T, Row>> },
    Write { key: String, value: MessageValue, continuation: Box<dyn FnOnce(()) -> Effect<T, Row>> },
}
```

### Handler System
```rust
pub trait Handler<F, G> {
    fn handle<T>(&self, effect: Effect<T, F>) -> Effect<T, G>;
}
```

### Outcome Algebra
```rust
pub struct Outcome {
    pub declarations: Vec<StateTransition>,
    pub proof: ProofData,
}

impl Outcome {
    pub fn compose(self, other: Outcome) -> Self;
    pub fn verify(&self) -> bool;
}
```

### Key Properties
- **Natural Transformations**: Handlers transform effects between row types
- **Algebraic Composition**: Effects and outcomes compose algebraically
- **Verifiable Results**: All outcomes include cryptographic proofs
- **Row-Polymorphic Effects**: Extensible effect system

## Layer 3: Agent Orchestration

High-level choreography language for multi-party protocols.

### Agent System
```rust
pub struct Agent {
    pub id: AgentId,
    pub capabilities: Vec<EffectRow>,
}
```

### Choreography Language
```rust
pub enum ChoreographyStep {
    Send { from: AgentId, to: AgentId, message: Message },
    Parallel(Vec<Choreography>),
    Sequence(Vec<Choreography>),
}
```

### Capability System
```rust
pub struct Capability<Row> {
    pub effect_row: Row,
    pub constraints: Vec<CapabilityConstraint>,
}
```

### Key Properties
- **Multi-Party Coordination**: Complex protocols with multiple participants
- **Capability-Based Security**: Type-level access control
- **Parallel Execution**: Concurrent operations with synchronization
- **Choreographic Programming**: High-level protocol specification

## Compilation Pipeline

The four layers form a compilation pipeline:

```
Choreography → Effects → Sessions → Instructions
```

### Layer 3 → Layer 2 Compilation
Choreographies are compiled to algebraic effects:
```rust
// Choreography step
Send { from: alice, to: bob, message: "Hello" }

// Compiles to effect
Effect::Send { 
    to: bob, 
    message: MessageValue::from("Hello"),
    continuation: ... 
}
```

### Layer 2 → Layer 1 Compilation
Effects are compiled to session-typed terms:
```rust
// Effect
Effect::Send { to: bob, message: msg, continuation: k }

// Compiles to term
Term::Send(bob, msg, k_term)
```

### Layer 1 → Layer 0 Compilation
Session terms are compiled to instruction sequences:
```rust
// Term
Term::Send(channel, message, continuation)

// Compiles to instructions
[
    Instruction::Create(val_reg, msg_reg),
    Instruction::Send(msg_reg, channel_reg),
    ...
]
```

## Data Flow

### Message Flow
1. **Creation**: Messages created from values at Layer 0
2. **Addressing**: Content-addressed with SHA256 hash
3. **Transmission**: Sent through typed channels
4. **Consumption**: Linearly consumed exactly once
5. **Verification**: Outcomes verified cryptographically

### Effect Flow
1. **Generation**: Effects generated from choreographies
2. **Transformation**: Handlers transform between effect rows
3. **Interpretation**: Interpreters execute effects to outcomes
4. **Composition**: Outcomes composed algebraically
5. **Verification**: Final outcome verified

### Type Flow
1. **Inference**: Types inferred from choreography structure
2. **Checking**: Linear and session types checked
3. **Compilation**: Types guide compilation to lower layers
4. **Runtime**: Type information used for verification

## Key Design Decisions

### Messages as Linear Records
Every message is a content-addressed linear record with row type, unifying the message and record concepts.

### Sessions as Effects
Session types are communication effects rather than separate abstractions, enabling uniform treatment.

### Handlers as Natural Transformations
Effect handlers are mathematical natural transformations, ensuring compositional properties.

### Capabilities as Session Types
Capabilities are session types parameterized by effect rows, leveraging existing type machinery.

## Performance Characteristics

### Space Complexity
- **Messages**: O(content size) with hash-based addressing
- **Types**: O(structure depth) for inference and checking
- **Effects**: O(composition depth) for handler chains
- **State**: O(agent count × state size) for global state

### Time Complexity
- **Message Creation**: O(content size) for hashing
- **Type Checking**: O(term size × type complexity)
- **Effect Execution**: O(effect count × handler complexity)
- **Verification**: O(outcome size) for proof checking

## Security Properties

### Linearity Guarantees
- Messages cannot be replayed or duplicated
- Resources consumed exactly once
- Double-spending prevention

### Content Integrity
- Messages cryptographically identified
- Tampering detection through hash verification
- Immutable content addressing

### Capability Isolation
- Agents restricted to granted capabilities
- Type-level access control
- Fine-grained permission system

### Verifiable Execution
- All outcomes cryptographically verifiable
- Algebraic composition preserves verification
- Proof generation for accountability

## Extensibility Points

### Custom Effect Rows
Add new effect types by extending the `EffectRow` enum:
```rust
pub enum EffectRow {
    // Existing effects
    Comm, State, Proof, IO,
    // Custom effects
    Database, Network, Cryptography,
}
```

### Handler Extensions
Implement custom handlers for effect transformations:
```rust
impl Handler<DatabaseRow, CommRow> for DatabaseToComm {
    fn handle<T>(&self, effect: Effect<T, DatabaseRow>) -> Effect<T, CommRow> {
        // Transform database effects to communication
    }
}
```

### Message Types
Extend the `Message` enum for domain-specific protocols:
```rust
pub enum Message {
    // Existing messages
    Text(String), Int(i64), ...
    // Custom messages
    HttpRequest { url: String, headers: Vec<(String, String)> },
    DatabaseQuery { sql: String, params: Vec<Value> },
}
```

### Capability Constraints
Add custom capability constraints:
```rust
pub enum CapabilityConstraint {
    // Existing constraints
    RateLimit { ... }, DataAccess { ... },
    // Custom constraints
    GeographicRestriction { allowed_regions: Vec<String> },
    TimeWindow { start: DateTime, end: DateTime },
}
```

This architecture provides a solid foundation for verifiable message-passing systems while maintaining mathematical rigor and practical usability. 