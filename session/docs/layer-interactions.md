# Layer Interactions

This document details how the four layers of the Causality-Valence architecture interact with each other, including data flow, compilation processes, and inter-layer communication.

## Overview of Layer Relationships

```
Layer 3 (Choreography) ──compiles to──> Layer 2 (Effects)
Layer 2 (Effects)      ──compiles to──> Layer 1 (Sessions)  
Layer 1 (Sessions)     ──compiles to──> Layer 0 (Instructions)
```

Each layer serves as both a compilation target and a compilation source, creating a clean separation of concerns while maintaining end-to-end functionality.

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