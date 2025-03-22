# ADR-007: Content-addressable Code and Execution in Rust

## Status

Accepted

## Context

Our current system identifies code modules and functions by names, leading to several challenges:

1. **Dependency conflicts**: Different versions of the same module may clash when referenced by name
2. **Mutable codebase**: Code can be changed in place, breaking existing references
3. **Complex refactoring**: Renaming or moving functions requires updating all references
4. **Build process overhead**: Traditional compilation requires reprocessing entire dependency trees

To address these issues, we propose adopting a content-addressable approach for both code storage and execution, where:

1. Code is identified by the hash of its abstract syntax tree (AST) or appropriate representation
2. Once defined, code definitions are immutable
3. Names are merely metadata attached to immutable content hashes
4. Code execution is directly tied to content hashes, enabling verification, memoization, and deterministic replay

This approach draws inspiration from systems like the Unison programming language and Git, but will be specifically tailored for our Rust-based ecosystem.

## Decision

We will implement a comprehensive content-addressable code and execution system with the following components:

### 1. Content-Addressable Code Repository

The following code defines the core types and interfaces for our content-addressable code repository. The `CodeHash` struct represents a Blake3 hash (which may be updated to a zk-friendly hash function later) that uniquely identifies a piece of code based on its content. The `CodeDefinition` struct holds the actual code along with metadata like its name and dependencies. The `CodeRepository` trait defines operations for storing, retrieving, and managing code definitions, ensuring that once stored, code definitions cannot be modified.

```rust
/// Represents a content hash for code
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CodeHash([u8; 32]);

/// A code definition with its metadata
pub struct CodeDefinition {
    /// The content hash of this definition
    pub hash: CodeHash,
    /// The human-readable name (if any)
    pub name: Option<String>,
    /// The actual code representation (AST or bytecode)
    pub content: CodeContent,
    /// Dependencies of this code definition
    pub dependencies: Vec<CodeHash>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Interface for the code repository
pub trait CodeRepository {
    /// Store a new code definition
    fn store(&mut self, definition: CodeDefinition) -> Result<CodeHash, RepositoryError>;
    
    /// Retrieve a code definition by its hash
    fn get_by_hash(&self, hash: &CodeHash) -> Result<CodeDefinition, RepositoryError>;
    
    /// Retrieve a code definition by its name
    fn get_by_name(&self, name: &str) -> Result<CodeDefinition, RepositoryError>;
    
    /// Get all transitive dependencies for a code definition
    fn get_dependencies(&self, hash: &CodeHash) -> Result<Vec<CodeDefinition>, RepositoryError>;
}
```

### 2. Execution Context

The execution context code defines the runtime environment for content-addressed code execution. The `ExecutionContext` struct maintains the state of an execution, including variable bindings, call stack, and execution trace. It uses thread-safe structures like `RwLock` to enable concurrent access where appropriate. The `CallFrame` struct represents a single function call in the stack, while the `ExecutionEvent` enum captures all types of events that can occur during execution, enabling comprehensive tracing and replay.

```rust
/// A context for code execution
pub struct ExecutionContext {
    /// Unique identifier for this context
    pub context_id: ContextId,
    /// Parent context, if any
    pub parent: Option<Arc<ExecutionContext>>,
    /// The code repository to use
    pub repository: Arc<dyn CodeRepository>,
    /// Variable bindings in this context
    variables: RwLock<HashMap<String, Value>>,
    /// Current call stack
    call_stack: RwLock<Vec<CallFrame>>,
    /// Execution trace
    execution_trace: RwLock<Vec<ExecutionEvent>>,
    /// Resource allocator
    resource_allocator: Arc<dyn ResourceAllocator>,
}

/// A single frame in the call stack
pub struct CallFrame {
    /// The hash of the code being executed
    pub code_hash: CodeHash,
    /// The name of the function, if known
    pub name: Option<String>,
    /// Arguments to the function
    pub arguments: Vec<Value>,
}

/// Events recorded during execution
pub enum ExecutionEvent {
    /// A function was called
    FunctionCall {
        hash: CodeHash,
        name: Option<String>,
        arguments: Vec<Value>,
    },
    /// A function returned
    FunctionReturn {
        hash: CodeHash,
        result: Value,
    },
    /// An effect was applied
    EffectApplied {
        effect_type: EffectType,
        parameters: HashMap<String, Value>,
        result: Value,
    },
    /// An error occurred
    Error(ExecutionError),
}
```

### 3. Content-Addressable Executor

The `ContentAddressableExecutor` trait defines the main interface for executing content-addressed code. It provides methods to execute code by either its hash or name, supporting both direct hash-based references and human-readable names. The executor is responsible for creating execution contexts, managing their lifecycle, and retrieving execution traces for debugging and verification. This interface abstracts away the actual execution mechanism, which could be an interpreter, JIT compiler, or RISC-V backend.

```rust
/// Main interface for the content-addressable executor
pub trait ContentAddressableExecutor {
    /// Execute code by its hash
    fn execute_by_hash(
        &self,
        hash: &CodeHash,
        arguments: Vec<Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, ExecutionError>;
    
    /// Execute code by its name
    fn execute_by_name(
        &self,
        name: &str,
        arguments: Vec<Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, ExecutionError>;
    
    /// Create a new execution context
    fn create_context(
        &self,
        parent: Option<Arc<ExecutionContext>>,
    ) -> Result<ExecutionContext, ExecutionError>;
    
    /// Get the execution trace from a context
    fn get_execution_trace(
        &self,
        context: &ExecutionContext,
    ) -> Result<Vec<ExecutionEvent>, ExecutionError>;
}
```

### 4. Resource Management and Security

This code defines the resource management and security components of our system. The `ResourceAllocator` trait provides a flexible interface for allocating, tracking, and controlling resource usage, with support for hierarchical resource subdivision. The `ResourceRequest` and `ResourceGrant` structs represent resource requests and allocations, respectively. The `SecuritySandbox` struct defines security boundaries for execution, controlling which effects are allowed and enforcing resource limits.

```rust
/// Interface for resource allocation
pub trait ResourceAllocator: Send + Sync {
    /// Allocate resources
    fn allocate(
        &self,
        request: ResourceRequest,
    ) -> Result<ResourceGrant, AllocationError>;
    
    /// Release resources
    fn release(&self, grant: ResourceGrant);
    
    /// Check current resource usage
    fn check_usage(&self, grant: &ResourceGrant) -> ResourceUsage;
    
    /// Subdivide resources for child contexts
    fn subdivide(
        &self,
        grant: ResourceGrant,
        requests: Vec<ResourceRequest>,
    ) -> Result<Vec<ResourceGrant>, AllocationError>;
}

/// A request for execution resources
pub struct ResourceRequest {
    /// Memory in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_millis: usize,
    /// Number of I/O operations
    pub io_operations: usize,
    /// Number of effects
    pub effect_count: usize,
}

/// A grant of resources
pub struct ResourceGrant {
    /// Unique ID for this grant
    pub grant_id: GrantId,
    /// Memory in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_millis: usize,
    /// Number of I/O operations
    pub io_operations: usize,
    /// Number of effects
    pub effect_count: usize,
}

/// Security sandbox for execution
pub struct SecuritySandbox {
    /// Allowed effect types
    pub allowed_effects: HashSet<EffectType>,
    /// Resource allocator
    pub resource_allocator: Arc<dyn ResourceAllocator>,
    /// Timeout in milliseconds
    pub timeout_millis: usize,
}
```

## Implementation Strategy

### 1. Code Storage and Hashing

1. **Create a robust hashing mechanism**: Implement a Blake3-based hashing system for Rust code that:
   - Ignores comments and formatting
   - Captures the semantic structure (AST) rather than raw text
   - Provides consistent hashes across different machines
   - Allows for potential future migration to a ZK-friendly hash function

2. **Build an immutable repository**: Develop a repository that:
   - Stores code by its content hash
   - Supports efficient lookup by both hash and name
   - Enforces immutability of stored definitions
   - Handles dependency management

3. **Name registry**: Implement a separate registry that:
   - Maps human-readable names to content hashes
   - Supports name changes without breaking references
   - Maintains version history for names

### 2. Execution Engine

1. **Rust-based interpreter**: Develop an interpreter that:
   - Executes code based on its content hash
   - Manages execution context with variable bindings
   - Records comprehensive execution traces
   - Enforces resource limits and security boundaries

2. **Integration with RISC-V for deterministic execution**:
   - Compile content-addressed code to RISC-V instructions
   - Use our ZK co-processor for deterministic execution
   - Ensure cross-platform consistency

3. **JIT compilation for performance**:
   - Identify frequently executed code paths
   - Implement a JIT compilation strategy
   - Maintain the content-addressed guarantees

### 3. Resource Management

1. **Hierarchical resource allocation**:
   - Implement the `ResourceAllocator` trait
   - Create a static allocator for initial implementation
   - Support resource subdivision for child contexts

2. **Security sandboxing**:
   - Implement the security sandbox
   - Control access to effects and external resources
   - Enforce timeouts for execution

### 4. Determinism and Replay

1. **Execution tracing**:
   - Record all function calls, returns, and effects
   - Create a serializable trace format
   - Support efficient storage and query of traces

2. **Replay capability**:
   - Implement replay from execution traces
   - Verify deterministic execution
   - Support debugging based on traces

## Time-Travel Debugging and State Persistence

### Snapshot-Based Persistence

We will implement a snapshot-based persistence model inspired by CRIU (Checkpoint/Restore In Userspace) but adapted for our Rust-based content-addressable system:

The following code defines our snapshot-based persistence system. The `ExecutionSnapshot` struct captures the complete state of an execution context at a point in time, including variable bindings, call stack, and resource usage. The `SnapshotManager` trait provides operations for creating, restoring, listing, and deleting snapshots. This system enables saving and resuming long-running computations, supports failure recovery, and provides the foundation for time-travel debugging.

```rust
/// A snapshot of an execution context
pub struct ExecutionSnapshot {
    /// Unique identifier for this snapshot
    pub snapshot_id: SnapshotId,
    /// The context ID this snapshot belongs to
    pub context_id: ContextId,
    /// The timestamp when this snapshot was created
    pub created_at: std::time::SystemTime,
    /// The execution position (event index) in the trace
    pub execution_position: usize,
    /// Variable bindings at the time of snapshot
    pub variables: HashMap<String, Value>,
    /// Call stack at the time of snapshot
    pub call_stack: Vec<CallFrame>,
    /// Resource usage at the time of snapshot
    pub resource_usage: ResourceUsage,
}

/// Interface for snapshot management
pub trait SnapshotManager: Send + Sync {
    /// Create a snapshot of the current execution context
    fn create_snapshot(
        &self, 
        context: &ExecutionContext
    ) -> Result<SnapshotId, SnapshotError>;
    
    /// Restore execution from a snapshot
    fn restore_snapshot(
        &self,
        snapshot_id: &SnapshotId
    ) -> Result<ExecutionContext, SnapshotError>;
    
    /// List available snapshots for a context
    fn list_snapshots(
        &self,
        context_id: &ContextId
    ) -> Result<Vec<ExecutionSnapshot>, SnapshotError>;
    
    /// Delete a snapshot
    fn delete_snapshot(
        &self,
        snapshot_id: &SnapshotId
    ) -> Result<(), SnapshotError>;
}
```

Key features of our snapshot system:

1. **Effect Boundary Checkpointing**: Snapshots are automatically created at effect boundaries, providing natural points for resumption.

2. **Incremental Snapshots**: Only state changes are stored between snapshots, minimizing storage requirements while maintaining complete state history.

3. **Deterministic Replay**: Snapshots include all necessary state to deterministically resume execution.

4. **Long-Running Execution Support**: Enables long-running computations to be paused, saved, and resumed later, even across machine restarts.

5. **Failure Recovery**: Execution can be resumed from the last stable snapshot after a failure.

### Time-Travel Debugging

Building on our snapshot-based persistence and execution tracing, we will implement a time-travel debugging system:

The `TimeTravel` trait defines our time-travel debugging interface. It provides methods for navigating execution history bidirectionally, jumping to specific execution points or effects, inspecting program state, and comparing states between different execution points. This powerful debugging capability leverages the immutability of content-addressed code and the comprehensive execution traces to enable developers to explore program behavior in ways not possible with traditional debuggers.

```rust
/// Interface for time-travel debugging
pub trait TimeTravel {
    /// Step forward by one event
    fn step_forward(
        &self,
        context: &mut ExecutionContext
    ) -> Result<ExecutionEvent, DebugError>;
    
    /// Step backward by one event
    fn step_backward(
        &self,
        context: &mut ExecutionContext
    ) -> Result<ExecutionEvent, DebugError>;
    
    /// Jump to a specific point in the execution trace
    fn jump_to_position(
        &self,
        context: &mut ExecutionContext,
        position: usize
    ) -> Result<ExecutionEvent, DebugError>;
    
    /// Jump to a specific effect application
    fn jump_to_effect(
        &self,
        context: &mut ExecutionContext,
        effect_type: EffectType,
        occurrence: usize
    ) -> Result<ExecutionEvent, DebugError>;
    
    /// Inspect the state at the current position
    fn inspect_state(
        &self,
        context: &ExecutionContext
    ) -> Result<HashMap<String, Value>, DebugError>;
    
    /// Compare state between two execution points
    fn compare_states(
        &self,
        context: &ExecutionContext,
        position1: usize,
        position2: usize
    ) -> Result<HashMap<String, (Option<Value>, Option<Value>)>, DebugError>;
}
```

Key features of our time-travel debugging:

1. **Bidirectional Navigation**: Navigate forward and backward through the execution history with precise control.

2. **Effect-Based Navigation**: Jump directly to specific effect applications to quickly identify issues.

3. **State Inspection**: Examine the complete program state at any point in the execution history.

4. **State Differencing**: Compare program state between different execution points to understand how values changed.

5. **Query Capabilities**: Search the execution history for specific events or state conditions.

6. **Causality Tracking**: Trace the causal relationships between effects and state changes.

7. **Execution Forks**: Create alternative execution paths from any point in history to explore "what if" scenarios.

The immutable nature of content-addressed code makes this time-travel capability particularly powerful, as the relationship between code versions is explicitly tracked through their content hashes, enabling navigation not just through a single execution, but across different versions of the same function.

## Integration with Existing System

Our content-addressable execution system will integrate with the existing codebase as follows:

1. **Effect System Integration**:
   - Effect definitions will be content-addressed
   - Effect handlers will be registered with the executor
   - Effect application will be recorded in execution traces

2. **Integration with Temporal Effect Language (TEL)**:
   - TEL expressions will compile to content-addressed fragments
   - References between fragments will use content hashes
   - TEL interpreter will use the content-addressable executor

3. **RISC-V Integration**:
   - Content-addressed code will compile to RISC-V instructions
   - Execution will leverage our RISC-V toolchain
   - Proofs will be generated for verification

## Benefits

This content-addressable approach provides several key benefits:

1. **Deterministic Execution**: Behavior is directly tied to content hash, enabling verification
2. **Safe Evolution**: New versions can be deployed without breaking existing references
3. **Dependency Precision**: Dependencies are resolved exactly as specified
4. **Auditability**: Every execution can be traced and replayed
5. **Distributed Execution**: Code can be executed on any node with the repository
6. **Memoization**: Results can be cached based on function hash and input hashes
7. **Time-Travel Debugging**: Execution can be stepped forward and backward

## Challenges and Mitigations

1. **Performance Overhead**:
   - **Challenge**: Content-addressed execution may introduce overhead
   - **Mitigation**: Implement JIT compilation and memoization for frequently used paths

2. **Complexity**:
   - **Challenge**: The system is more complex than direct function calls
   - **Mitigation**: Create clear abstractions and comprehensive documentation

3. **Rust Integration**:
   - **Challenge**: Making content-addressing work well with Rust's ownership model
   - **Mitigation**: Create proper abstractions that work with Rust's type system

4. **Tooling**:
   - **Challenge**: Developers need tools to work with content-addressed code
   - **Mitigation**: Build IDE integrations and developer-friendly tools

## Implementation Plan

1. **Phase 1**: Core Repository and Hashing (2 weeks)
   - Implement the CodeHash and CodeDefinition structures
   - Build the basic CodeRepository implementation
   - Create the name registry

2. **Phase 2**: Execution Context and Interpreter (3 weeks)
   - Implement the ExecutionContext
   - Build the interpreter for content-addressed code
   - Create the execution trace system

3. **Phase 3**: Resource Management and Security (2 weeks)
   - Implement the ResourceAllocator trait
   - Create the initial StaticAllocator implementation
   - Build the security sandbox

4. **Phase 4**: Integration with Existing Systems (3 weeks)
   - Integrate with the effect system
   - Connect to the Temporal Effect Language
   - Link with the RISC-V execution environment

5. **Phase 5**: Tooling and Documentation (2 weeks)
   - Build developer tools for working with content-addressed code
   - Create comprehensive documentation
   - Develop examples and best practices

## Conclusion

Implementing a content-addressable code and execution system in Rust will provide significant benefits for our project. By treating code as immutable, content-addressed values, we can simplify dependency management, enable powerful verification and auditing capabilities, and create a foundation for deterministic, reproducible execution. 