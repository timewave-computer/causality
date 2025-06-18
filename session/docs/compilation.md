# Compilation Process

This document details the compilation pipeline that transforms high-level choreographies into executable Layer 0 instructions through the four-layer Causality-Valence architecture.

## Compilation Overview

The compilation process follows a multi-stage transformation pipeline:

```
Choreography (Layer 3)
    ↓ Choreography Compiler
Effects (Layer 2)  
    ↓ Effect Compiler
Session Terms (Layer 1)
    ↓ Term Compiler  
Instructions (Layer 0)
    ↓ Virtual Machine
Execution Results
```

Each stage performs specific transformations while preserving correctness properties and enabling optimizations.

## Stage 1: Choreography to Effects

### Choreography Analysis

The compiler first analyzes the choreography structure to understand:

- **Agent Participation**: Which agents are involved in each step
- **Message Dependencies**: Ordering constraints between messages
- **Parallel Opportunities**: Steps that can execute concurrently
- **Capability Requirements**: What effects each agent needs

```rust
fn analyze_choreography(choreo: &Choreography) -> ChoreographyAnalysis {
    ChoreographyAnalysis {
        agents: extract_agents(choreo),
        dependencies: compute_dependencies(choreo),
        parallelism: find_parallel_sections(choreo),
        capabilities: infer_capabilities(choreo),
    }
}
```

### Effect Generation

Each choreography step is transformed into corresponding effects:

#### Send Message Compilation
```rust
// Input: Choreography step
ChoreographyStep::Send { 
    from: alice, 
    to: bob, 
    message: Message::Payment { amount: 100, currency: "USD".to_string() }
}

// Output: Effect sequence  
vec![
    Effect::Send {
        to: bob,
        message: MessageValue::Pair(
            Box::new(MessageValue::Int(100)),
            Box::new(MessageValue::from("USD"))
        ),
        continuation: Box::new(|msg_id| Effect::Pure(msg_id))
    }
]
```

#### Parallel Compilation
```rust
// Input: Parallel choreography
ChoreographyStep::Parallel(vec![choreo1, choreo2, choreo3])

// Output: Parallel effect composition
Effect::Parallel(vec![
    compile_choreography_to_effects(choreo1),
    compile_choreography_to_effects(choreo2), 
    compile_choreography_to_effects(choreo3),
])
```

### Capability Checking

The compiler verifies that each agent has the necessary capabilities:

```rust
fn verify_capabilities(effects: &[Effect], agents: &AgentRegistry) -> Result<(), CompilationError> {
    for effect in effects {
        match effect {
            Effect::Send { to, .. } => {
                if !agents.get(to).unwrap().can_perform(&EffectRow::Comm) {
                    return Err(CompilationError::InsufficientCapabilities {
                        agent: to.clone(),
                        required: EffectRow::Comm,
                    });
                }
            }
            Effect::Write { .. } => {
                // Check state capabilities...
            }
            // ... other effect types
        }
    }
    Ok(())
}
```

## Stage 2: Effects to Session Terms

### Effect Transformation

Effects are transformed into session-typed terms that capture the communication structure:

#### Communication Effects
```rust
// Input: Send effect
Effect::Send { to: bob, message: msg, continuation: k }

// Output: Session term
Term::Send(
    Channel::between(&alice, &bob),  // Channel name: "Alice→Bob"
    msg,
    compile_continuation(k)
)
```

#### State Effects  
```rust
// Input: State read effect
Effect::Read { key: "balance", continuation: k }

// Output: Session protocol for state access
Term::Sequence(vec![
    Term::Send(Channel::new("Agent→StateServer"), MessageValue::from("balance")),
    Term::Receive(Channel::new("StateServer→Agent"), |value| compile_continuation(k(value)))
])
```

### Session Type Inference

The compiler infers session types for each communication channel:

```rust
fn infer_session_type(terms: &[Term]) -> HashMap<ChannelName, SessionType> {
    let mut types = HashMap::new();
    
    for term in terms {
        match term {
            Term::Send(channel, msg_type, continuation) => {
                let session_type = SessionType::Send(
                    msg_type.clone(), 
                    Box::new(infer_continuation_type(continuation))
                );
                types.insert(channel.clone(), session_type);
            }
            Term::Receive(channel, continuation) => {
                let session_type = SessionType::Receive(
                    infer_message_type(channel),
                    Box::new(infer_continuation_type(continuation))  
                );
                types.insert(channel.clone(), session_type);
            }
            // ... other term types
        }
    }
    
    types
}
```

### Duality Verification

The compiler checks that dual session types are compatible:

```rust
fn verify_session_duality(types: &HashMap<ChannelName, SessionType>) -> Result<(), CompilationError> {
    for (channel, session_type) in types {
        if let Some(dual_channel) = channel.reverse() {
            if let Some(dual_type) = types.get(&dual_channel) {
                if !session_type.is_dual_of(dual_type) {
                    return Err(CompilationError::SessionTypeMismatch {
                        channel: channel.clone(),
                        expected: session_type.dual(),
                        found: dual_type.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}
```

## Stage 3: Session Terms to Instructions

### Register Allocation

The compiler allocates registers for values and channels:

```rust
struct RegisterAllocator {
    next_register: u32,
    register_map: HashMap<Variable, Register>,
    type_map: HashMap<Register, Type>,
}

impl RegisterAllocator {
    fn allocate_for_value(&mut self, value: &MessageValue) -> Register {
        let reg = Register(self.next_register);
        self.next_register += 1;
        self.type_map.insert(reg, value.infer_type());
        reg
    }
    
    fn allocate_for_channel(&mut self, channel: &ChannelName) -> Register {
        let reg = Register(self.next_register);
        self.next_register += 1;
        self.type_map.insert(reg, Type::Channel);
        reg
    }
}
```

### Instruction Generation

Session terms are compiled to instruction sequences:

#### Send Term Compilation
```rust
// Input: Send term
Term::Send(channel, message, continuation)

// Output: Instruction sequence
vec![
    // Create message from value
    Instruction::Create(value_reg, msg_reg),
    // Send message through channel  
    Instruction::Send(msg_reg, channel_reg),
    // Compile continuation
    ...compile_term(continuation)
]
```

#### Receive Term Compilation
```rust
// Input: Receive term  
Term::Receive(channel, continuation)

// Output: Instruction sequence
vec![
    // Receive message from channel
    Instruction::Receive(channel_reg, msg_reg),
    // Extract value from message
    Instruction::Consume(msg_reg, value_reg),
    // Compile continuation with received value
    ...compile_continuation(continuation, value_reg)
]
```

### Linear Resource Tracking

The compiler ensures linear usage of messages and channels:

```rust
struct LinearityChecker {
    used_registers: HashSet<Register>,
    available_registers: HashSet<Register>,
}

impl LinearityChecker {
    fn check_instruction(&mut self, instr: &Instruction) -> Result<(), CompilationError> {
        match instr {
            Instruction::Consume(msg_reg, _) => {
                if self.used_registers.contains(msg_reg) {
                    return Err(CompilationError::DoubleUse { register: *msg_reg });
                }
                self.used_registers.insert(*msg_reg);
                self.available_registers.remove(msg_reg);
                Ok(())
            }
            // ... other instructions
        }
    }
}
```

## Optimization Passes

### Cross-Layer Optimizations

#### Dead Effect Elimination
```rust
fn eliminate_dead_effects(effects: Vec<Effect>) -> Vec<Effect> {
    effects.into_iter()
        .filter(|effect| !is_dead_effect(effect))
        .collect()
}

fn is_dead_effect(effect: &Effect) -> bool {
    match effect {
        Effect::Pure(_) => true,  // Pure effects with unused results
        Effect::Write { key, .. } if is_unused_key(key) => true,
        _ => false,
    }
}
```

#### Message Fusion
```rust
fn fuse_messages(instructions: Vec<Instruction>) -> Vec<Instruction> {
    // Combine adjacent create/send into single operation
    let mut optimized = Vec::new();
    let mut i = 0;
    
    while i < instructions.len() {
        if let (Instruction::Create(val_reg, msg_reg), Instruction::Send(send_msg_reg, chan_reg)) = 
            (&instructions[i], instructions.get(i + 1).unwrap_or(&Instruction::Nop)) {
                if msg_reg == send_msg_reg {
                    optimized.push(Instruction::CreateAndSend(*val_reg, *chan_reg));
                    i += 2;
                    continue;
                }
            }
        optimized.push(instructions[i].clone());
        i += 1;
    }
    
    optimized
}
```

#### Channel Coalescing
```rust
fn coalesce_channels(terms: Vec<Term>) -> Vec<Term> {
    // Merge channels with identical usage patterns
    let usage_patterns = analyze_channel_usage(&terms);
    let channel_mapping = compute_channel_coalescing(&usage_patterns);
    
    terms.into_iter()
        .map(|term| apply_channel_mapping(term, &channel_mapping))
        .collect()
}
```

## Error Handling During Compilation

### Compilation Error Types

```rust
#[derive(Debug, Clone)]
pub enum CompilationError {
    // Layer 3 → 2 errors
    InvalidChoreography { reason: String },
    InsufficientCapabilities { agent: AgentId, required: EffectRow },
    
    // Layer 2 → 1 errors  
    EffectTypeError { effect: String, expected_row: String },
    HandlerCompositionError { source: String, target: String },
    
    // Layer 1 → 0 errors
    SessionTypeMismatch { channel: ChannelName, expected: SessionType, found: SessionType },
    LinearityViolation { variable: String, usage_count: usize },
    RegisterAllocationError { reason: String },
    
    // Cross-layer errors
    ResourceExhaustion { resource_type: String },
    InternalCompilerError { message: String },
}
```

### Error Recovery Strategies

```rust
impl Compiler {
    fn try_recover_from_error(&mut self, error: CompilationError) -> Result<(), CompilationError> {
        match error {
            CompilationError::InsufficientCapabilities { agent, required } => {
                // Try to infer minimal capability set
                if self.can_infer_capability(&agent, &required) {
                    self.add_inferred_capability(agent, required);
                    Ok(())
                } else {
                    Err(error)
                }
            }
            CompilationError::SessionTypeMismatch { channel, expected, found } => {
                // Try session type coercion
                if self.can_coerce_session_type(&found, &expected) {
                    self.add_session_coercion(channel, found, expected);
                    Ok(())
                } else {
                    Err(error)
                }
            }
            _ => Err(error), // No recovery possible
        }
    }
}
```

## Compilation Metrics and Profiling

### Performance Metrics

```rust
#[derive(Debug)]
pub struct CompilationMetrics {
    pub choreography_size: usize,
    pub effect_count: usize,
    pub session_term_count: usize,
    pub instruction_count: usize,
    
    pub compile_time_ms: u64,
    pub optimization_time_ms: u64,
    
    pub register_usage: usize,
    pub channel_count: usize,
    pub message_count: usize,
}
```

### Compilation Phases Timing

```rust
fn compile_with_timing(choreography: Choreography) -> (Vec<Instruction>, CompilationMetrics) {
    let start_time = Instant::now();
    
    let (effects, choreo_time) = time_phase(|| compile_choreography_to_effects(choreography));
    let (terms, effect_time) = time_phase(|| compile_effects_to_terms(effects));  
    let (instructions, term_time) = time_phase(|| compile_terms_to_instructions(terms));
    let (optimized, opt_time) = time_phase(|| optimize_instructions(instructions));
    
    let total_time = start_time.elapsed();
    
    let metrics = CompilationMetrics {
        compile_time_ms: total_time.as_millis() as u64,
        optimization_time_ms: opt_time.as_millis() as u64,
        // ... other metrics
    };
    
    (optimized, metrics)
}
```

This compilation process ensures that high-level choreographies are transformed into efficient, verifiable, and type-safe executable code while preserving all correctness properties throughout the pipeline. 