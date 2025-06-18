# API Reference

This document provides comprehensive documentation for all public interfaces in the Session crate.

## Core Types

### MessageId

Content-addressed identifier for messages using SHA256 hashing.

```rust
pub struct MessageId([u8; 32]);

impl MessageId {
    pub fn new(content: &[u8]) -> Self
    pub fn as_bytes(&self) -> &[u8; 32]
    pub fn to_hex(&self) -> String
}
```

### MessageValue

Linear values that can be stored in messages.

```rust
pub enum MessageValue {
    Unit,
    Bool(bool),
    Int(i64),
    Pair(Box<MessageValue>, Box<MessageValue>),
    Sum(bool, Box<MessageValue>), // Left(false) or Right(true)
}

impl MessageValue {
    pub fn hash(&self) -> MessageId
    pub fn size(&self) -> usize
}
```

### Message

High-level message types for choreographies.

```rust
pub enum Message {
    Text(String),
    Int(i64),
    Bool(bool),
    PaymentRequest { amount: i64, recipient: String },
    Payment { amount: i64, currency: String },
    Receipt,
    TokenDeposit { token_type: String, amount: i64 },
    Ready,
}

impl Message {
    pub fn to_value(&self) -> MessageValue
    pub fn from_value(value: MessageValue) -> Result<Self, ConversionError>
}
```

## Agent System

### AgentId

Unique identifier for agents in the system.

```rust
pub struct AgentId(String);

impl AgentId {
    pub fn new(name: &str) -> Self
    pub fn name(&self) -> &str
}
```

### Agent

Agent with capabilities and state.

```rust
pub struct Agent {
    pub id: AgentId,
    pub capabilities: Vec<EffectRow>,
}

impl Agent {
    pub fn new(id: AgentId, capabilities: Vec<EffectRow>) -> Self
    pub fn can_perform(&self, effect: &EffectRow) -> bool
}
```

## Choreography System

### Choreography

High-level description of multi-party interactions.

```rust
pub struct Choreography {
    steps: Vec<ChoreographyStep>,
}

impl Choreography {
    pub fn new() -> Self
    pub fn add_send(self, from: &AgentId, to: &AgentId, message: Message) -> Self
    pub fn add_parallel(self, choreographies: Vec<Choreography>) -> Self
    pub fn add_sequence(self, choreographies: Vec<Choreography>) -> Self
}
```

### ChoreographyStep

Individual steps in a choreography.

```rust
pub enum ChoreographyStep {
    Send { from: AgentId, to: AgentId, message: Message },
    Parallel(Vec<Choreography>),
    Sequence(Vec<Choreography>),
}
```

## Effect System

### EffectRow

Types of effects that can be performed.

```rust
pub enum EffectRow {
    Comm,    // Communication effects
    State,   // State management effects
    Proof,   // Proof generation effects
    IO,      // Input/output effects
}
```

### Effect

Algebraic effects with row types.

```rust
pub enum Effect<T, Row> {
    Pure(T),
    Send { to: AgentId, message: MessageValue, continuation: Box<dyn FnOnce(MessageId) -> Effect<T, Row>> },
    Receive { from: AgentId, continuation: Box<dyn FnOnce(MessageValue) -> Effect<T, Row>> },
    Read { key: String, continuation: Box<dyn FnOnce(MessageValue) -> Effect<T, Row>> },
    Write { key: String, value: MessageValue, continuation: Box<dyn FnOnce(()) -> Effect<T, Row>> },
}

impl<T, Row> Effect<T, Row> {
    pub fn pure(value: T) -> Self
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Effect<U, Row>
    pub fn bind<U>(self, f: impl FnOnce(T) -> Effect<U, Row>) -> Effect<U, Row>
}
```

## Outcome System

### Outcome

Verifiable outcome of computations.

```rust
pub struct Outcome {
    pub declarations: Vec<StateTransition>,
    pub proof: ProofData,
}

impl Outcome {
    pub fn empty() -> Self
    pub fn compose(self, other: Outcome) -> Self
    pub fn verify(&self) -> bool
    pub fn hash(&self) -> MessageId
}
```

### StateTransition

Types of state changes.

```rust
pub enum StateTransition {
    Transfer { from: AgentId, to: AgentId, amount: i64 },
    Update { agent: AgentId, key: String, value: MessageValue },
    Create { agent: AgentId, key: String, value: MessageValue },
    Delete { agent: AgentId, key: String },
}
```

## Type System

### Type

Layer 1 type system with linear and session types.

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

### SessionType

Session types for communication protocols.

```rust
pub enum SessionType {
    Send(Box<Type>, Box<SessionType>),    // !T.S
    Receive(Box<Type>, Box<SessionType>), // ?T.S
    Choice(Vec<SessionType>),             // &{l1: S1, l2: S2, ...}
    Selection(Vec<SessionType>),          // ⊕{l1: S1, l2: S2, ...}
    End,
}

impl SessionType {
    pub fn dual(&self) -> SessionType
    pub fn is_well_formed(&self) -> bool
}
```

### RowType

Row types for extensible records and effects.

```rust
pub enum RowType {
    Empty,
    Extend(String, Type, Box<RowType>),
    RowVar(String),
}

impl RowType {
    pub fn lookup(&self, label: &str) -> Option<&Type>
    pub fn extend(self, label: String, ty: Type) -> Self
    pub fn restrict(self, labels: &[String]) -> Self
}
```

## Interpreter

### Interpreter

Main execution engine for the session crate.

```rust
pub struct Interpreter {
    // Private fields
}

impl Interpreter {
    pub fn new() -> Self
    
    // Agent management
    pub fn register_agent(&mut self, agent: AgentId, capabilities: &[EffectRow]) -> Result<(), InterpreterError>
    pub fn get_agent(&self, id: &AgentId) -> Option<&Agent>
    
    // State management
    pub fn set_state(&mut self, agent: &AgentId, key: &str, value: MessageValue) -> Result<(), InterpreterError>
    pub fn get_state(&self, agent: &AgentId, key: &str) -> Result<MessageValue, InterpreterError>
    
    // Execution
    pub fn execute_choreography(&mut self, choreography: Choreography) -> Result<ExecutionResult, InterpreterError>
    pub fn execute_effect<T, Row>(&mut self, effect: Effect<T, Row>) -> Result<(T, Outcome), InterpreterError>
    
    // Debugging
    pub fn enable_debug(&mut self)
    pub fn get_effect_log(&self) -> &[EffectLogEntry]
    pub fn get_state_snapshot(&self) -> StateSnapshot
}
```

### ExecutionResult

Result of choreography execution.

```rust
pub struct ExecutionResult {
    pub outcome: Outcome,
    pub final_state: StateSnapshot,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool
    pub fn get_proof(&self) -> &ProofData
}
```

## Channel Management

### ChannelRegistry

Manages communication channels between agents.

```rust
pub struct ChannelRegistry {
    // Private fields
}

impl ChannelRegistry {
    pub fn new() -> Self
    pub fn create_channel(&mut self, name: String, capacity: Option<usize>) -> Result<(), ChannelError>
    pub fn send_message(&mut self, channel: &str, message: MessageValue) -> Result<(), ChannelError>
    pub fn receive_message(&mut self, channel: &str) -> Result<MessageValue, ChannelError>
    pub fn get_channel_status(&self, channel: &str) -> Option<ChannelStatus>
}
```

## Error Types

### InterpreterError

Main error type for interpreter operations.

```rust
pub enum InterpreterError {
    AgentNotFound(AgentId),
    ChannelError(ChannelError),
    TypeError(String),
    RuntimeError(String),
    InvalidOperation(String),
    CompilationError(String),
}

impl InterpreterError {
    pub fn context(&self) -> Option<&ErrorContext>
    pub fn suggestions(&self) -> Vec<&str>
}
```

### ErrorContext

Diagnostic information for errors.

```rust
pub struct ErrorContext {
    pub operation_stack: Vec<String>,
    pub current_agent: Option<AgentId>,
    pub current_choreography_step: Option<usize>,
    pub state_snapshot: StateSnapshot,
}
```

## Capability System

### Capability

Type-level capability constraints.

```rust
pub struct Capability<Row> {
    pub effect_row: Row,
    pub constraints: Vec<CapabilityConstraint>,
}

impl<Row> Capability<Row> {
    pub fn new(effect_row: Row) -> Self
    pub fn with_constraint(self, constraint: CapabilityConstraint) -> Self
    pub fn can_perform(&self, effect: &Effect<(), Row>) -> bool
}
```

### CapabilityConstraint

Constraints on capability usage.

```rust
pub enum CapabilityConstraint {
    RateLimit { max_operations: usize, time_window: std::time::Duration },
    DataAccess { allowed_keys: Vec<String> },
    NetworkAccess { allowed_hosts: Vec<String> },
}
```

## Compilation System

### Layer0Compiler

Compiles to Layer 0 instructions.

```rust
pub struct Layer0Compiler;

impl Layer0Compiler {
    pub fn compile_term(term: &Term) -> Result<Vec<Instruction>, CompilationError>
    pub fn compile_choreography(choreography: &Choreography) -> Result<Vec<Instruction>, CompilationError>
}
```

### Layer1Compiler

Compiles Layer 2 effects to Layer 1 terms.

```rust
pub struct Layer1Compiler;

impl Layer1Compiler {
    pub fn compile_effect<T, Row>(effect: &Effect<T, Row>) -> Result<Term, CompilationError>
    pub fn compile_outcome(outcome: &Outcome) -> Result<Term, CompilationError>
}
```

## Usage Examples

### Basic Message Passing

```rust
use session::*;

let mut interpreter = Interpreter::new();
let alice = AgentId::new("Alice");
let bob = AgentId::new("Bob");

interpreter.register_agent(alice.clone(), &[EffectRow::Comm])?;
interpreter.register_agent(bob.clone(), &[EffectRow::Comm])?;

let choreography = Choreography::new()
    .add_send(&alice, &bob, Message::Text("Hello".to_string()));

let result = interpreter.execute_choreography(choreography)?;
```

### State Management

```rust
use session::*;

let mut interpreter = Interpreter::new();
let alice = AgentId::new("Alice");

interpreter.register_agent(alice.clone(), &[EffectRow::State])?;
interpreter.set_state(&alice, "balance", MessageValue::Int(1000))?;

let balance = interpreter.get_state(&alice, "balance")?;
```

### Effect Composition

```rust
use session::*;

let effect1 = Effect::pure(42);
let effect2 = effect1.map(|x| x * 2);
let effect3 = effect2.bind(|x| Effect::pure(x + 1));
```

### Parallel Execution

```rust
use session::*;

let choreography = Choreography::new()
    .add_parallel(vec![
        Choreography::new().add_send(&alice, &bob, Message::Text("Hello".to_string())),
        Choreography::new().add_send(&bob, &alice, Message::Text("World".to_string())),
    ]);
``` 