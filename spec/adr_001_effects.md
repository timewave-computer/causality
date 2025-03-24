# ADR-001: Rust Algebraic Effects Library optimized for RISC-V ZK VM Compilation

## Status

Accepted

## Implementation Status

This ADR has been fully implemented with an evolution to a three-layer effect architecture as described in ADR-023. The core components implemented include:

1. **Effect System**:
   - Implemented a trait-based approach with `Effect` as the core interface
   - Effects are identified via `EffectId` and produce standardized `EffectOutcome` results
   - Added comprehensive error handling through typed `EffectError` variants

2. **Three-Layer Architecture**:
   - **Algebraic Effect Layer**: Core traits and interfaces in Rust
   - **Effect Constraints Layer**: Type constraints and validation rules
   - **Domain Implementation Layer**: Domain-specific implementations in TEL

3. **Resource-Scoped Concurrency**:
   - Implemented resource locks with RAII-style `ResourceGuard`
   - Added deterministic wait queues for consistent scheduling
   - Integrated with Rust's async/await pattern for concurrency management

4. **Domain Integration**:
   - Created domain adapters for multiple chains (EVM, CosmWasm)
   - Built boundary crossing mechanisms for secure domain interactions
   - Integrated with time map for cross-domain causal consistency

5. **Content-Addressed Storage**:
   - Implemented content-addressed code repository
   - Added support for content-addressed effects and resources
   - Integrated with cryptographic hash interfaces

6. **ZK Integration**:
   - Created ZK-VM integration through domain adapters
   - Added support for proof generation and verification
   - Implemented deterministic execution for consistent proving

7. **TEL Integration**:
   - Built TEL compiler and runtime for domain-specific implementations
   - Added templating system for effect code generation
   - Created bridge between Rust's static types and TEL's domain-specific code

The implementation evolved beyond the initial design to create a more flexible and powerful system while maintaining the core principles of safety, determinism, and ZK compatibility. The architecture now supports unified operation models, cross-domain interactions, and content-addressing throughout the system.

For more details, see [docs/src/unified_effect_model.md](/docs/src/unified_effect_model.md) and [docs/src/effect_system.md](/docs/src/effect_system.md).

## Context

Causaility is the foundational algebraic effects library for a cross-domain program environment while maintaining strong causal consistency. This library needs to support compilation to RISC-V assembly, which will allow our code to run inside zero-knowledge virtual machines (ZK VMs). With this capability, we can generate cryptographic proofs that programs executed correctly.

## Decision

We will implement a Rust-based algebraic effects library with the following core design principles:

1. **Closed Effect Set**: Effects will be represented as a sealed enumeration with explicit continuation types

   We'll define a fixed, known set of operations (like "deposit" or "withdraw") that can interact with external systems, rather than allowing arbitrary extensions. Each operation will explicitly define what happens next with its result.

2. **Static Handler Resolution**: Effect handlers will use static dispatch with composition capabilities

   When deciding how to execute an effect, we'll use compile-time resolution rather than runtime lookups. This makes the code faster and easier to analyze, while still allowing handlers to be combined in flexible ways.

3. **Resource-Scoped Concurrency**: Resource locks will be managed explicitly with deterministic wait queues

   When multiple operations need access to the same resource (like an account balance), we'll use explicit locking with predictable ordering. This ensures operations happen in a consistent way every time.

4. **RISC-V Compilation**: The system will compile to RISC-V code compatible with ZK VM execution

   We'll transform our high-level Rust code into a simpler instruction set (RISC-V for the time being) that can run inside special zero-knowledge virtual machines, enabling cryptographic proofs of execution.

5. **Deterministic Execution**: All non-deterministic operations will be controlled through explicit interfaces

   Things that normally vary between runs (like random numbers or time) will be tightly controlled to ensure our code produces the same result every time it runs with the same inputs.

6. **Content-Addressed Code**: Program components will be stored in a content-addressed repository

   Code will be identified by its content hash (like a fingerprint) rather than its name, ensuring immutability and precise versioning.

7. **Explicit Error Model**: Errors will be represented as typed results with explicit error effects

   We'll use specific error types for each operation rather than generic errors, making it clear exactly what can go wrong and how errors should be handled.

## System Architecture

### Core Components

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                          Causality Rust Architecture                           │
│                                                                                │
│  ┌──────────────────┐    ┌────────────────────┐    ┌────────────────────────┐  │
│  │ Effect System    │    │ Domain System      │    │ Content-Addressed Code │  │
│  │                  │    │                    │    │                        │  │
│  │  ┌────────────┐  │    │  ┌──────────────┐  │    │  ┌──────────────────┐  │  │
│  │  │ Effect     │  │    │  │ Domain       │  │    │  │ Code Repository  │  │  │
│  │  │ Definition │  │    │  │ Adapters     │  │    │  │                  │  │  │
│  │  └─────┬──────┘  │    │  └──────┬───────┘  │    │  └────────┬─────────┘  │  │
│  │        │         │    │         │          │    │           │            │  │
│  │  ┌─────▼──────┐  │    │  ┌──────▼───────┐  │    │  ┌────────▼─────────┐  │  │
│  │  │ Handler    │  │    │  │ Time Map     │  │    │  │ Content-Addressed│  │  │
│  │  │ System     │◄─┼────┼──┤ Integration  │  │    │  │ Loader           │  │  │
│  │  └─────┬──────┘  │    │  └──────┬───────┘  │    │  └────────┬─────────┘  │  │
│  │        │         │    │         │          │    │           │            │  │
│  │  ┌─────▼──────┐  │    │  ┌──────▼───────┐  │    │  ┌────────▼─────────┐  │  │
│  │  │ Resource   │  │    │  │ Fact         │  │    │  │ Version          │  │  │
│  │  │ Manager    │◄─┼────┼──┤ Observation  │◄─┼────┼──┤ Resolution       │  │  │
│  │  └─────┬──────┘  │    │  └──────────────┘  │    │  └──────────────────┘  │  │
│  │        │         │    │                    │    │                        │  │
│  └────────┼─────────┘    └────────────────────┘    └────────────────────────┘  │
│           │                                                                    │
│  ┌────────▼─────────┐    ┌────────────────────┐    ┌────────────────────────┐  │
│  │ Runtime System   │    │ RISC-V System      │    │ Unified Log System     │  │
│  │                  │    │                    │    │                        │  │
│  │  ┌────────────┐  │    │  ┌──────────────┐  │    │  ┌──────────────────┐  │  │
│  │  │ Effect     │  │    │  │ RISC-V       │  │    │  │ Log Entry        │  │  │
│  │  │ Interpreter│◄─┼────┼──┤ Compiler     │  │    │  │ Definition       │  │  │
│  │  └─────┬──────┘  │    │  └──────┬───────┘  │    │  └────────┬─────────┘  │  │
│  │        │         │    │         │          │    │           │            │  │
│  │  ┌─────▼──────┐  │    │  ┌──────▼───────┐  │    │  ┌────────▼─────────┐  │  │
│  │  │ Concurrency│  │    │  │ ZK VM        │  │    │  │ Log Storage      │  │  │
│  │  │ Manager    │◄─┼────┼──┤ Integration  │◄─┼────┼──┤ & Retrieval      │  │  │
│  │  └─────┬──────┘  │    │  └──────┬───────┘  │    │  └────────┬─────────┘  │  │
│  │        │         │    │         │          │    │           │            │  │
│  │  ┌─────▼──────┐  │    │  ┌──────▼───────┐  │    │  ┌────────▼─────────┐  │  │
│  │  │ Ecosystem  │  │    │  │ ZK Proof     │  │    │  │ Replay Engine    │  │  │
│  │  │ Integration│◄─┼────┼──┤ Generation   │◄─┼────┼──┤                  │  │  │
│  │  └────────────┘  │    │  └──────────────┘  │    │  └──────────────────┘  │  │
│  │                  │    │                    │    │                        │  │
│  └──────────────────┘    └────────────────────┘    └────────────────────────┘  │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘
```

This diagram shows how our major components fit together. Now let's walk through each one and explain what it does in more approachable terms:

### Effect System
This is the core of our library—it defines what operations our programs can perform and how they're executed. Think of it as the vocabulary and grammar for expressing what our programs can do.

### Domain System
This connects our programs to actual chains. It's like having translators who speak each chain's native language, allowing our programs to talk to Ethereum, Solana, or other domains without needing to know their specific details.

### Content-Addressed Code System
This stores program code by its hash, rather than by name. It's similar to how Git works—each piece of code gets a unique fingerprint based on its content, not an arbitrary name. This makes versioning precise and prevents dependency conflicts.

### Runtime System
This executes our programs, handling the flow of control and coordination between different parts. Think of it like an operating system that runs our effects, manages concurrency, and connects to the broader Rust ecosystem.

### RISC-V System
This translates our high-level effects into simple RISC-V instructions that can run inside a zero-knowledge virtual machine.

#### Unified Log System

The Unified Log System is responsible for:
- Recording all effects, facts, and events in a unified log
- Storing logs in a serializable format
- Providing replay capabilities for program execution
- Supporting audit and verification of program history

## Key Implementation Decisions

### 1. Effect Representation

We will use a **sealed trait with enumeration approach**:

```rust
// Public trait defining the effect interface
pub trait Effect<R>: sealed::SealedEffect {
    fn execute(self, handler: &dyn EffectHandler) -> R;
    // Other methods
}

// Private sealing trait to prevent external implementation
mod sealed {
    pub trait SealedEffect {}
}

// Core effect enum - exhaustive within our crate, can't be extended externally
pub enum CoreEffect<R> {
    Deposit { 
        timeline: TimelineId, 
        asset: Asset, 
        amount: Amount, 
        continuation: Box<dyn Continuation<DepositResult, R>> 
    },
    Withdraw { /* ... */ },
    Observe { /* ... */ },
    // ... other variants
}

// Implementation of traits for the enum
impl<R> sealed::SealedEffect for CoreEffect<R> {}
impl<R> Effect<R> for CoreEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> R {
        match self {
            CoreEffect::Deposit { timeline, asset, amount, continuation } => {
                let result = handler.handle_deposit(timeline, asset, amount);
                continuation.apply(result)
            },
            // ... other variants
        }
    }
    
    // Other method implementations
}
```

This approach provides:
- Type safety with exhaustive pattern matching
- Optimization opportunities at compile time 
- A clear interface for users of the library
- Compatibility with RISC-V compilation

The tradeoff is reduced extensibility compared to an open trait system, but for a ZK VM target, having a fixed set of effects is actually beneficial for deterministic execution and proof generation.

### 2. Effect Handler Implementation

We will use a **hybrid handler approach** with static dispatch for core effects:

```rust
// Handler trait
pub trait EffectHandler {
    fn handle_deposit(&self, timeline: TimelineId, asset: Asset, amount: Amount) -> DepositResult;
    fn handle_withdraw(&self, timeline: TimelineId, asset: Asset, amount: Amount) -> WithdrawResult;
    fn handle_observe(&self, query: FactQuery) -> ObservedFact;
    // ... other handlers
}

// Handler implementation for timelines
pub struct TimelineHandler {
    Users: HashMap<TimelineId, Arc<TimeCommittee>>,
}

impl EffectHandler for TimelineHandler {
    fn handle_deposit(&self, timeline: TimelineId, asset: Asset, amount: Amount) -> DepositResult {
        if let Some(User) = self.Users.get(&timeline) {
            User.deposit(asset, amount)
        } else {
            DepositResult::TimelineNotFound
        }
    }
    // ... other handler implementations
}

// Composable handler for fallbacks
pub struct CompositeHandler<A, B> {
    primary: A,
    fallback: B,
}

impl<A, B> EffectHandler for CompositeHandler<A, B>
where
    A: EffectHandler,
    B: EffectHandler,
{
    fn handle_deposit(&self, timeline: TimelineId, asset: Asset, amount: Amount) -> DepositResult {
        match self.primary.handle_deposit(timeline, asset, amount) {
            DepositResult::TimelineNotFound => self.fallback.handle_deposit(timeline, asset, amount),
            result => result,
        }
    }
    // ... other handler implementations
}
```

This approach:
- Uses static dispatch for core handlers, which translates well to RISC-V
- Allows for handler composition without dynamic dispatch
- Provides a clear interface for implementing handlers
- Makes testing easier with mock handlers
- Is deterministic for ZK proving

The downside is that adding new effect types requires modifying the handler trait, but this aligns with our sealed effect approach. For the ZK VM target, having a fixed set of effect types that can be statically analyzed and optimized is more important than unlimited extensibility.

### 3. Continuation Representation

We will use **explicit continuation objects with a factory pattern**:

```rust
// Continuation trait
pub trait Continuation<I, O>: Send + 'static {
    fn apply(self: Box<Self>, input: I) -> O;
    fn to_risc_v<W: RiscVWriter>(&self, writer: &mut W) -> Result<(), RiscVError>;
    fn content_hash(&self) -> Hash;
}

// Simple function continuation
pub struct FnContinuation<I, O, F: FnOnce(I) -> O + Send + 'static> {
    f: F,
}

impl<I, O, F: FnOnce(I) -> O + Send + 'static> Continuation<I, O> for FnContinuation<I, O, F> {
    fn apply(self: Box<Self>, input: I) -> O {
        (self.f)(input)
    }
    
    fn to_risc_v<W: RiscVWriter>(&self, writer: &mut W) -> Result<(), RiscVError> {
        // Implementation for RISC-V code generation
        unimplemented!()
    }
    
    fn content_hash(&self) -> Hash {
        // Implementation for content hash calculation
        unimplemented!()
    }
}

// Continuation factory
pub struct ContinuationFactory;

impl ContinuationFactory {
    pub fn from_fn<I, O, F: FnOnce(I) -> O + Send + 'static>(f: F) -> Box<dyn Continuation<I, O>> {
        Box::new(FnContinuation { f })
    }
    
    // More factory methods for other continuation types
}

// Example usage
let cont = ContinuationFactory::from_fn(|result: DepositResult| {
    // Process result
    format!("Deposit result: {:?}", result)
});
```

This approach:
- Separates continuation logic from effect definition
- Allows for different types of continuations (functions, complex state machines, etc.)
- Makes continuation composition explicit and manageable
- Is more RISC-V friendly than closure captures
- Can be optimized for the ZK VM context

The explicit approach is more verbose than using closures directly, but it gives us better control over memory layout and allocation, which is crucial for efficient RISC-V compilation and ZK proving. It also makes it easier to serialize and deserialize continuations for the content-addressed code system.

### 4. Resource-Scoped Concurrency

We will implement a **resource lock table with explicit wait queues**:

```rust
// Resource ID type
pub struct ResourceId(Uuid);

// Resource lock manager
pub struct ResourceLockManager {
    locks: Mutex<HashMap<ResourceId, LockEntry>>,
}

struct LockEntry {
    holder: Option<TaskId>,
    wait_queue: VecDeque<Waker>,
}

impl ResourceLockManager {
    // Try to acquire a lock immediately
    pub fn try_acquire(&self, resource: ResourceId) -> Option<ResourceGuard> {
        let mut locks = self.locks.lock().unwrap();
        let entry = locks.entry(resource.clone()).or_insert_with(|| LockEntry {
            holder: None,
            wait_queue: VecDeque::new(),
        });
        
        if entry.holder.is_none() {
            entry.holder = Some(current_task_id());
            Some(ResourceGuard {
                manager: self.clone(),
                resource,
            })
        } else {
            None
        }
    }
    
    // Async lock acquisition
    pub async fn acquire(&self, resource: ResourceId) -> ResourceGuard {
        // Implementation using futures and wakers
        unimplemented!()
    }
    
    // Release a lock
    fn release(&self, resource: ResourceId) {
        let mut locks = self.locks.lock().unwrap();
        if let Some(entry) = locks.get_mut(&resource) {
            entry.holder = None;
            if let Some(waker) = entry.wait_queue.pop_front() {
                waker.wake();
            }
        }
    }
}

// Resource guard that auto-releases on drop
pub struct ResourceGuard {
    manager: Arc<ResourceLockManager>,
    resource: ResourceId,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.manager.release(self.resource.clone());
    }
}
```

This approach:
- Explicitly models resource ownership with async/await
- Provides deadlock prevention through ordered lock acquisition
- Is compatible with standard Rust async runtimes
- Can be made deterministic for the ZK VM by controlling the wait queue order
- Supports both cooperative and preemptive concurrency models

The complexity in managing the wait queues and ensuring fairness is a trade-off, but this gives us fine-grained control over resource access patterns. For the ZK VM, we'll need to ensure deterministic scheduling, which this approach supports by making the wait queue order explicit and controllable.

### 5. RISC-V ZK VM Integration

We will implement a **focused RISC-V compilation pipeline** with the following components:

```rust
// RISC-V program representation
pub struct RiscVProgram {
    sections: Vec<RiscVSection>,
    entry_point: Label,
    symbols: HashMap<String, usize>,
}

// RISC-V code generator
pub struct RiscVGenerator {
    target: RiscVTarget,
    optimizations: Vec<Box<dyn RiscVOptimization>>,
}

impl RiscVGenerator {
    pub fn generate_code(&self, effect: &dyn Effect<()>) -> Result<RiscVProgram, RiscVError> {
        let mut program = RiscVProgram::new();
        effect.to_risc_v(&mut program)?;
        
        for opt in &self.optimizations {
            opt.optimize(&mut program)?;
        }
        
        Ok(program)
    }
}

// ZK VM integration
pub struct ZkVmIntegration {
    vm: ZkVm,
}

impl ZkVmIntegration {
    pub fn execute(&self, program: &RiscVProgram) -> Result<ZkExecutionResult, ZkError> {
        // Execute the program in the ZK VM
        unimplemented!()
    }
    
    pub fn generate_proof(&self, program: &RiscVProgram, result: &ZkExecutionResult) -> Result<ZkProof, ZkError> {
        // Generate a ZK proof for the execution
        unimplemented!()
    }
}
```

This approach:
- Provides a clean separation between effect definition and RISC-V compilation
- Allows for optimization passes on the generated RISC-V code
- Integrates with the ZK VM for execution and proof generation
- Supports different ZK VM targets through abstraction

The tradeoff is increased complexity in the compilation pipeline, but this is necessary for efficient execution in the ZK VM and optimal proof generation. By focusing on a specific RISC-V subset supported by ZK VMs, we can ensure that all effects have efficient implementations.

### 6. Deterministic Execution

We will implement a **controlled execution environment** for determinism:

```rust
// Deterministic execution context
pub struct DeterministicContext {
    random_seed: [u8; 32],
    time_source: Box<dyn TimeSource>,
    memory_allocator: Box<dyn MemoryAllocator>,
}

pub trait TimeSource: Send + Sync {
    fn now(&self) -> SystemTime;
}

// Mock time source with deterministic behavior
pub struct MockTimeSource {
    current_time: Mutex<SystemTime>,
    increment: Duration,
}

impl TimeSource for MockTimeSource {
    fn now(&self) -> SystemTime {
        let mut time = self.current_time.lock().unwrap();
        *time += self.increment;
        *time
    }
}

// Deterministic memory allocator
pub struct DeterministicAllocator;

unsafe impl GlobalAlloc for DeterministicAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Deterministic allocation algorithm
        unimplemented!()
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Deterministic deallocation
        unimplemented!()
    }
}
```

This approach:
- Controls all sources of non-determinism (time, randomness, memory, etc.)
- Provides reproducible execution for consistent proving
- Makes debugging easier with reproducible behavior
- Is explicit about non-deterministic operations
- Can be switched out for native implementations in non-ZK contexts

The overhead of wrapping all potentially non-deterministic operations is a tradeoff, but it's necessary for consistent execution in the ZK VM. The mock implementations also make testing easier since we can control the behavior of time and randomness in tests.

### 7. Error Handling

We will use a **typed result approach** with explicit error effects:

```rust
// Result type for each effect
pub enum DepositResult {
    Success { tx_id: TransactionId },
    InsufficientFunds,
    TimelineNotFound,
    NetworkError(String),
    // ... other error cases
}

// Dedicated error effect for cross-cutting concerns
pub enum ErrorEffect<R> {
    Timeout { 
        duration: Duration, 
        continuation: Box<dyn Continuation<(), R>> 
    },
    Retry { 
        attempts: u32, 
        delay: Duration, 
        continuation: Box<dyn Continuation<(), R>> 
    },
    Recover { 
        fallback: R, 
        continuation: Box<dyn Continuation<bool, R>> 
    },
    // ... other error handling effects
}

// Handler for error effects
pub struct ErrorHandler {
    // Handler state
}

impl EffectHandler for ErrorHandler {
    // Implementation of error handling
}

// Extension methods for working with results
pub trait ResultExt<T, E> {
    fn and_then<U, F>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>;
        
    fn or_else<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(E) -> Result<T, E>;
        
    fn with_timeout(self, duration: Duration) -> Result<T, TimeoutError<E>>;
}
```

This approach:
- Types each error explicitly rather than using a generic error type
- Makes error handling part of the effect system
- Provides compositional error handling
- Is deterministic for ZK proving
- Gives fine-grained control over error recovery strategies

The downside is more verbose error types, but this explicitness is beneficial for a ZK VM target where we need to know exactly what errors can occur and how they're handled. It also makes error handling more predictable and testable, which is important for reliability.

## Trade-offs and Consequences

### Key Trade-offs

1. **Static vs. Dynamic Type System**
   - We choose a more static approach with sealed traits and enumerations
   - This sacrifices some extensibility for better static analysis and optimization
   - The ZK VM compilation requires knowing effect types at compile time

2. **Explicit vs. Implicit Continuations**
   - We use explicit continuation objects rather than implicit closures
   - This gives us more control over memory layout and allocation
   - It makes the system more complex but easier to optimize for ZK proving

3. **Resource Contention Model**
   - We use explicit lock acquisition with wait queues
   - This adds complexity but gives us deterministic scheduling
   - The approach works well with the ZK VM requirement for determinism

4. **Error Handling Strategy**
   - We use typed results rather than a generic Result type
   - This increases verbosity but improves type safety and explicitness
   - It helps catch errors at compile time that might otherwise cause issues in ZK proving

5. **Runtime Compatibility**
   - We implement our own runtime with adapters to mainstream Rust async runtimes
   - This increases maintenance burden but gives us more control
   - It allows optimizing specifically for the ZK VM target

### Positive Consequences

1. **Better Performance**
   - Static dispatch and specialized code paths enable better optimization
   - Control over memory allocation patterns reduces overhead
   - Batched operations improve ZK proving efficiency

2. **Enhanced Safety**
   - Strong typing catches more errors at compile time
   - Explicit error handling improves reliability
   - Resource safety through RAII guards prevents leaks

3. **Deterministic Execution**
   - Control over non-deterministic operations ensures reproducible execution
   - Explicit time and randomness sources make testing easier
   - Consistent execution is critical for ZK proving

4. **Improved Auditability**
   - Content-addressed code improves traceability
   - Unified logging captures the complete execution history
   - Every effect is linked to its time map and dependencies

5. **ZK VM Compatibility**
   - The design is built around RISC-V compilation from the start
   - All components consider ZK proving requirements
   - The system can generate and verify ZK proofs of execution

### Negative Consequences

1. **Increased Complexity**
   - The explicit approach is more verbose than implicit alternatives
   - Managing continuations manually requires more care
   - The RISC-V compilation pipeline adds complexity

2. **Limited Extensibility**
   - The sealed trait approach restricts adding new effect types
   - Handler implementation requires modifying core traits
   - This is a deliberate tradeoff for ZK VM compatibility

3. **Learning Curve**
   - Developers need to understand the effect system and continuation model
   - The explicit approach differs from typical Rust patterns
   - Resource-scoped concurrency requires careful design

4. **Performance Overhead in Native Execution**
   - Controls for determinism add overhead in non-ZK contexts
   - The explicit continuation model may be slower than native closures
   - These overheads are acceptable given the ZK VM target

5. **Maintenance Burden**
   - Supporting both native and RISC-V execution requires maintaining two paths
   - Adapting to changes in ZK VM implementations requires ongoing work
   - The complex system has more potential points of failure
