# Data Flow Documentation

This document details how data flows through the four-layer Causality-Valence architecture, including message propagation, state transformations, and effect execution.

## Overall Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 3: Choreography                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │    Agent    │ ←→ │Choreography │ ←→ │ Capability  │      │
│  │  Registry   │    │  Engine     │    │   System    │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Layer 2: Effects                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │   Effect    │ ←→ │   Handler   │ ←→ │   Outcome   │      │
│  │ Generation  │    │ Transform   │    │  Algebra    │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                   Layer 1: Sessions                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │    Type     │ ←→ │   Session   │ ←→ │     Row     │      │
│  │  Checking   │    │   Types     │    │    Types    │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Layer 0: Instructions                       │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │   Machine   │ ←→ │   Content   │ ←→ │  Execution  │      │
│  │    State    │    │ Addressing  │    │   Engine    │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Message Lifecycle

### 1. Message Creation (Layer 3 → Layer 0)

```
High-Level Message (Layer 3)
    ↓
Message::Payment { amount: 100, currency: "USD" }
    ↓ Choreography Compiler
Effect::Send { message: MessageValue, ... } (Layer 2)
    ↓ Effect Compiler  
Term::Send(channel, MessageValue, continuation) (Layer 1)
    ↓ Term Compiler
[Create(val_reg, msg_reg), Send(msg_reg, chan_reg)] (Layer 0)
    ↓ Virtual Machine
SHA256 Content-Addressed Message in Machine State
```

### 2. Message Transmission Flow

```
Sender Agent (Layer 3)
    ↓ register_agent()
Agent Registry
    ↓ check_capabilities()  
Effect::Send Generation (Layer 2)
    ↓ handler_transform()
Session Term Send (Layer 1)
    ↓ compile_term()
Instruction::Send (Layer 0)
    ↓ execute_instruction()
Channel Registry
    ↓ channel_send()
Message Queue
    ↓ channel_receive()
Receiver Agent (Layer 3)
```

### 3. Linear Consumption Tracking

```
Message Creation:
  MessageId = SHA256(content)
  RefCount = 1
    ↓
Message Reference:
  Binding { register: R1, message_id: MSG_ID }
    ↓
Message Consumption:
  Instruction::Consume(R1, VAL_REG)
  RefCount = 0 (message destroyed)
    ↓
Linear Property Enforced:
  Future references to MSG_ID → Error
```

## State Flow Patterns

### Agent State Management

```
Agent Registration:
┌──────────────┐
│ Agent Alice  │ ──register_agent()──→ ┌─────────────────┐
│ Capabilities │                      │ Agent Registry  │
│ [Comm,State] │                      │ State Storage   │
└──────────────┘                      └─────────────────┘

State Operations:
┌──────────────┐ ──set_state()──→ ┌─────────────────┐
│   Interpreter│                  │ State HashMap   │
│              │ ←─get_state()──  │ AgentId → KV    │
└──────────────┘                  └─────────────────┘

State Effects:
┌──────────────┐ ──Effect::Read──→ ┌─────────────────┐
│ Effect System│                  │ State Interpreter│
│              │ ←─StateTransition │ Outcome Gen     │
└──────────────┘                  └─────────────────┘
```

### Channel State Evolution

```
Channel Creation:
ChannelRegistry::create_channel("Alice→Bob", capacity: Some(10))
    ↓
Channel { 
  name: "Alice→Bob",
  messages: VecDeque::new(),
  capacity: 10,
  participants: HashSet{Alice, Bob},
  status: Active
}

Message Flow:
Alice ──send──→ Channel ──receive──→ Bob
  ↓                ↓                 ↓
Effect::Send → Queue.push() → Effect::Receive
  ↓                ↓                 ↓  
Outcome     → Channel State → Continuation
```

## Effect Propagation

### Effect Generation and Composition

```
Choreography Step:
ChoreographyStep::Send { from: Alice, to: Bob, message: "Hello" }
    ↓ choreography_compiler
Effect Tree:
Effect::Send {
  to: Bob,
  message: MessageValue::from("Hello"),
  continuation: |msg_id| Effect::Pure(msg_id)
}
    ↓ effect_composition  
Composed Effect:
Effect::Sequence([
  Effect::Send { ... },
  Effect::Pure(MessageId)
])
```

### Handler Transformation Pipeline

```
Source Effect Row:           Target Effect Row:
┌─────────────────┐         ┌─────────────────┐
│ Effect<T, Comm> │ ──────→ │ Effect<T, State>│
│                 │ Handler │                 │
│ Send/Receive    │ ──────→ │ Read/Write      │
│ Operations      │         │ Operations      │
└─────────────────┘         └─────────────────┘

Handler Chain:
Effect<T, CommRow>
    ↓ CommToStateHandler
Effect<T, StateRow>  
    ↓ StateToProofHandler
Effect<T, ProofRow>
    ↓ ProofInterpreter
(Outcome, T)
```

## Interpreter Execution Flow

### Unified Interpreter Coordination

```
execute_choreography(choreography):

1. Choreography Analysis:
   ┌─────────────────┐
   │ extract_agents()│ → Agent List
   │ check_deps()    │ → Dependency Graph  
   │ find_parallel() │ → Parallel Sections
   └─────────────────┘

2. Effect Compilation:
   ┌─────────────────┐
   │ compile_to_fx() │ → Effect Tree
   │ apply_handlers()│ → Transformed Effects
   │ verify_caps()   │ → Capability Check
   └─────────────────┘

3. Session Compilation:
   ┌─────────────────┐
   │ effects_to_terms│ → Session Terms
   │ infer_types()   │ → Session Types
   │ check_duality() │ → Duality Verification
   └─────────────────┘

4. Instruction Generation:
   ┌─────────────────┐
   │ terms_to_instr()│ → Instruction List
   │ alloc_registers │ → Register Allocation
   │ check_linearity │ → Linear Verification  
   └─────────────────┘

5. Virtual Machine Execution:
   ┌─────────────────┐
   │ execute_instrs()│ → State Changes
   │ generate_proof()│ → Proof Data
   │ compose_outcome │ → Final Outcome
   └─────────────────┘
```

### Parallel Execution Coordination

```
Parallel Choreography:
Choreography::Parallel([choreo1, choreo2, choreo3])
    ↓
Effect Parallelization:
Effect::Parallel([
  compile(choreo1), 
  compile(choreo2), 
  compile(choreo3)
])
    ↓
Concurrent Session Execution:
┌─────────┐  ┌─────────┐  ┌─────────┐
│Session 1│  │Session 2│  │Session 3│
│Terms    │  │Terms    │  │Terms    │  
└─────────┘  └─────────┘  └─────────┘
     ↓            ↓            ↓
┌─────────────────────────────────────┐
│      Synchronized Instruction       │
│         Execution Engine            │
└─────────────────────────────────────┘
     ↓            ↓            ↓
┌─────────┐  ┌─────────┐  ┌─────────┐
│Outcome 1│  │Outcome 2│  │Outcome 3│
└─────────┘  └─────────┘  └─────────┘
     ↓            ↓            ↓
      ┌─────────────────────────┐
      │   Composed Outcome      │
      │   Algebraic Merge       │
      └─────────────────────────┘
```

## Content Addressing Flow

### Hash-Based Message Identity

```
Message Content:
┌─────────────────────────────────────┐
│ MessageValue::Pair(                 │
│   Box::new(MessageValue::Int(100)), │
│   Box::new(MessageValue::from("USD"))│
│ )                                   │
└─────────────────────────────────────┘
    ↓ serialize_canonical()
Canonical Bytes: [0x42, 0x64, 0x55, 0x53, 0x44, ...]
    ↓ SHA256::digest()
MessageId: [0xa7, 0x2f, 0x8b, ..., 0x3d] (32 bytes)
    ↓ to_hex()
"a72f8b...3d" (64 char hex string)

Content Store:
HashMap<MessageId, MessageValue> {
  "a72f8b...3d": MessageValue::Pair(...),
  "b83e9c...7a": MessageValue::Int(42),
  ...
}
```

### Reference Tracking and Cleanup

```
Message Reference Creation:
create_message(value) → (MessageId, Register)
    ↓
Register Table:
HashMap<Register, MessageId> {
  Register(1): MessageId("a72f8b...3d"),
  Register(2): MessageId("b83e9c...7a"),
}

Message Consumption:
consume_message(Register(1)) → MessageValue
    ↓
Reference Removal:
Register(1) removed from table
MessageId("a72f8b...3d") marked for cleanup
    ↓
Garbage Collection:
If no other references exist → remove from content store
```

## Error Propagation Flow

### Cross-Layer Error Context

```
Layer 0 Error:
InvalidRegister(Register(5))
    ↓ add_context()
Layer 1 Error:  
TypeError {
  expected: Type::Int,
  found: Type::String, 
  source: InvalidRegister(Register(5))
}
    ↓ add_context()
Layer 2 Error:
EffectError {
  effect: "Send to Bob",
  effect_row: CommRow,
  source: TypeError { ... }  
}
    ↓ add_context()
Layer 3 Error:
ChoreographyError {
  step: 3,
  agents: [Alice, Bob],
  choreography: "Payment Protocol",
  source: EffectError { ... }
}
```

### Error Recovery and Diagnostics

```
Error Detection:
┌─────────────────┐
│ InterpreterError│ ──analyze()──→ ┌─────────────────┐
│ with Context    │              │ Error Analyzer  │
└─────────────────┘              └─────────────────┘
                                          ↓
Diagnostic Generation:                    ↓
┌─────────────────┐ ←──generate()───  ┌─────────────────┐
│ Error Context:  │                 │ Diagnostic Info │
│ - Stack Trace   │                 │ - Suggestions   │  
│ - State Snapshot│                 │ - Fix Hints     │
│ - Agent Context │                 │ - Related Errors│
└─────────────────┘                 └─────────────────┘
```

## Performance and Optimization Flow

### Compilation Pipeline Optimization

```
Optimization Passes:

1. Choreography Level:
   ┌─────────────────────┐
   │ Dead Agent Removal  │ → Reduced Agent Set
   │ Parallel Detection  │ → Concurrency Map
   │ Message Coalescing  │ → Batched Operations
   └─────────────────────┘

2. Effect Level:  
   ┌─────────────────────┐
   │ Effect Fusion       │ → Reduced Effect Count
   │ Handler Composition │ → Optimized Transforms
   │ Dead Effect Elim    │ → Pruned Effect Tree
   └─────────────────────┘

3. Session Level:
   ┌─────────────────────┐
   │ Type Inference Opt  │ → Faster Type Checking
   │ Channel Coalescing  │ → Reduced Channel Count
   │ Session Merging     │ → Simplified Protocols
   └─────────────────────┘

4. Instruction Level:
   ┌─────────────────────┐
   │ Register Allocation │ → Optimal Register Use
   │ Instruction Fusion  │ → Reduced Instruction Count
   │ Control Flow Opt    │ → Efficient Execution Path
   └─────────────────────┘
```

This data flow architecture ensures efficient, verifiable, and type-safe execution while maintaining clear separation of concerns across all four layers. 