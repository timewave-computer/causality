# Effect

*This document provides reference information for the `Effect` data structure and the effect system.*

*Last updated: 2023-09-05*

## Overview

An `Effect` represents a side effect or interaction with external systems in the Causality framework. The effect system provides a structured way to handle operations that have side effects, such as I/O, state changes, or resource access, while maintaining functional purity and composability. As defined in ADR-032, effects are now integrated directly into the `causality-core` crate, providing a unified approach to system design.

## Type Definition

Effects are typically defined as trait bounds rather than concrete structures:

```rust
/// Trait for effects that can be performed
pub trait Effect: Clone + Send + Sync + 'static {
    /// The type returned when this effect is performed
    type Output;
    
    /// The error type that can occur when performing this effect
    type Error;
    
    /// Description of the effect
    fn description(&self) -> String;
}
```

## Effect Handlers

Effect handlers implement the logic for executing effects:

```rust
/// Handler for a specific effect type
pub trait EffectHandler<E: Effect>: Send + Sync + 'static {
    /// Handle the effect
    fn handle(&self, effect: E, ctx: &mut dyn EffectContext) -> Result<E::Output, E::Error>;
    
    /// Get the effect type this handler can handle
    fn effect_type(&self) -> &'static str;
}
```

## Effectful Computations

The `Effectful` type represents a computation that can have effects:

```rust
/// A computation that can have effects
pub struct Effectful<T, E> {
    /// The computation function
    computation: Box<dyn FnOnce(&mut dyn EffectContext) -> Result<T, EffectError<E>>>,
    
    /// Metadata about the computation
    metadata: EffectfulMetadata,
}
```

## Effect Context

The context in which effects are executed:

```rust
/// Context for effect execution
pub trait EffectContext: Send + Sync + 'static {
    /// Perform an effect
    fn perform<E: Effect>(&mut self, effect: E) -> Result<E::Output, EffectError<E>>;
    
    /// Get the current agent
    fn current_agent(&self) -> Option<&AgentId>;
    
    /// Get the current transaction (if any)
    fn current_transaction(&self) -> Option<&TransactionId>;
    
    /// Get the current timestamp
    fn current_time(&self) -> TimeStamp;
}
```

## Common Effect Types

### Resource Effects

Effects related to resource access and manipulation:

```rust
/// Effect for getting a resource
pub struct GetResource {
    /// Resource ID
    pub id: ResourceId,
    
    /// Lock mode
    pub lock_mode: LockMode,
}

impl Effect for GetResource {
    type Output = ResourceGuard<dyn Resource>;
    type Error = ResourceError;
    
    fn description(&self) -> String {
        format!("Get resource {} with lock mode {:?}", self.id, self.lock_mode)
    }
}
```

### Capability Effects

Effects related to capability verification:

```rust
/// Effect for verifying a capability
pub struct VerifyCapability {
    /// Capability to verify
    pub capability: Capability,
    
    /// Resource ID
    pub resource_id: ResourceId,
}

impl Effect for VerifyCapability {
    type Output = ();
    type Error = CapabilityError;
    
    fn description(&self) -> String {
        format!("Verify capability for resource {}", self.resource_id)
    }
}
```

### IO Effects

Effects related to input/output operations:

```rust
/// Effect for reading a file
pub struct ReadFile {
    /// Path to the file
    pub path: String,
}

impl Effect for ReadFile {
    type Output = Vec<u8>;
    type Error = IOError;
    
    fn description(&self) -> String {
        format!("Read file {}", self.path)
    }
}

/// Effect for writing a file
pub struct WriteFile {
    /// Path to the file
    pub path: String,
    
    /// Content to write
    pub content: Vec<u8>,
}

impl Effect for WriteFile {
    type Output = ();
    type Error = IOError;
    
    fn description(&self) -> String {
        format!("Write file {}", self.path)
    }
}
```

### State Effects

Effects related to state management:

```rust
/// Effect for getting state
pub struct GetState<T: Clone + Send + Sync + 'static> {
    /// State key
    pub key: String,
    
    /// Phantom data for the state type
    pub _phantom: PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> Effect for GetState<T> {
    type Output = Option<T>;
    type Error = StateError;
    
    fn description(&self) -> String {
        format!("Get state for key {}", self.key)
    }
}

/// Effect for setting state
pub struct SetState<T: Clone + Send + Sync + 'static> {
    /// State key
    pub key: String,
    
    /// State value
    pub value: T,
}

impl<T: Clone + Send + Sync + 'static> Effect for SetState<T> {
    type Output = ();
    type Error = StateError;
    
    fn description(&self) -> String {
        format!("Set state for key {}", self.key)
    }
}
```

## Effect System

The `EffectSystem` manages effect handlers and executes effectful computations. As per ADR-032, the effect system is now integrated directly into the `causality-core` crate:

```rust
pub struct EffectSystem {
    /// Registered effect handlers
    handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    
    /// Effect middleware
    middleware: Vec<Box<dyn EffectMiddleware>>,
}

impl EffectSystem {
    /// Create a new effect system
    pub fn new() -> Self;
    
    /// Register an effect handler
    pub fn register_handler<E: Effect, H: EffectHandler<E>>(&mut self, handler: H) -> Result<(), EffectSystemError>;
    
    /// Execute an effectful computation
    pub fn execute<T, E>(&self, computation: impl Effectful<T, E>, context: EffectContext) -> Result<T, EffectError<E>>;
    
    /// Add middleware
    pub fn add_middleware(&mut self, middleware: impl EffectMiddleware);
}
```

## Effect Composition

Effects can be composed to build complex operations:

```rust
/// Combine two effectful computations sequentially
pub fn and_then<T, U, E>(
    first: impl Effectful<T, E>,
    f: impl FnOnce(T) -> impl Effectful<U, E>
) -> impl Effectful<U, E> {
    Effectful::new(move |ctx| {
        let t = first.run(ctx)?;
        f(t).run(ctx)
    })
}

/// Combine two effectful computations in parallel
pub fn par<T1, T2, E>(
    first: impl Effectful<T1, E>,
    second: impl Effectful<T2, E>
) -> impl Effectful<(T1, T2), E> {
    Effectful::new(move |ctx| {
        let t1 = first.run(ctx)?;
        let t2 = second.run(ctx)?;
        Ok((t1, t2))
    })
}
```

## Three-Layer Effect Architecture

As described in ADR-032, the effect system follows a three-layer architecture and is now integrated directly into the `causality-core` crate:

### 1. Algebraic Effect Layer

The core layer defining the algebraic effect abstractions:

```rust
/// Algebraic effect operator
pub enum EffectOp<E, A> {
    /// Return a value
    Return(A),
    
    /// Perform an effect and continue
    Perform(E, Box<dyn FnOnce(Result<E::Output, E::Error>) -> EffectOp<E, A>>),
}

impl<E: Effect, A> EffectOp<E, A> {
    /// Bind a continuation
    pub fn bind<B, F>(self, f: F) -> EffectOp<E, B>
    where
        F: FnOnce(A) -> EffectOp<E, B>,
    {
        match self {
            EffectOp::Return(a) => f(a),
            EffectOp::Perform(e, k) => {
                EffectOp::Perform(e, Box::new(move |r| {
                    k(r).bind(f)
                }))
            }
        }
    }
}
```

### 2. Effect Constraints Layer

The middle layer linking effects to capabilities:

```rust
/// Constraint on an effect based on capabilities
pub struct EffectConstraint<E: Effect> {
    /// The effect being constrained
    effect: E,
    
    /// Required capabilities
    required_capabilities: Vec<Capability>,
}

impl<E: Effect> EffectConstraint<E> {
    /// Create a new effect constraint
    pub fn new(effect: E, required_capabilities: Vec<Capability>) -> Self;
    
    /// Verify the constraint
    pub fn verify(&self, agent: &Agent) -> Result<(), CapabilityError>;
}
```

### 3. Domain Implementation Layer

The layer integrating effects with specific domains:

```rust
/// Domain-specific effect handler
pub trait DomainEffectHandler<E: Effect>: EffectHandler<E> {
    /// The domain this handler belongs to
    fn domain(&self) -> &DomainId;
    
    /// Get domain-specific resources needed for handling the effect
    fn domain_resources(&self) -> Vec<ResourceId>;
}
```

## Usage Example

```rust
use causality_core::{
    effect::{EffectSystem, Effectful, EffectContext},
    resource::{ResourceManager, ResourceEffect, GetResource, LockMode},
};

// Define a custom effect
#[derive(Clone)]
struct QueryDatabase {
    query: String,
}

impl Effect for QueryDatabase {
    type Output = Vec<Row>;
    type Error = DatabaseError;
    
    fn description(&self) -> String {
        format!("Query database: {}", self.query)
    }
}

// Define an effectful computation
fn get_user_data(user_id: String) -> impl Effectful<UserData, ResourceEffect + QueryDatabase> {
    Effectful::new(move |ctx| {
        // Get the database resource
        let db = ctx.perform(GetResource { 
            id: "users_db".into(), 
            lock_mode: LockMode::Read 
        })?;
        
        // Query the database
        let rows = ctx.perform(QueryDatabase { 
            query: format!("SELECT * FROM users WHERE id = '{}'", user_id)
        })?;
        
        // Process the data
        let user_data = UserData::from_rows(rows)?;
        
        Ok(user_data)
    })
}

// Create the effect system
let mut effect_system = EffectSystem::new();

// Register handlers
effect_system.register_handler(ResourceEffectHandler::new(resource_manager.clone()))?;
effect_system.register_handler(DatabaseEffectHandler::new(database_connection))?;

// Execute the computation
let user_data = effect_system.execute(
    get_user_data("alice".to_string()),
    EffectContext::new()
)?;
```

## Effect Middleware

Effect middleware intercepts effects before they are handled:

```rust
pub trait EffectMiddleware: Send + Sync + 'static {
    /// Process an effect before it's handled
    fn process<E: Effect>(
        &self, 
        effect: E, 
        ctx: &mut dyn EffectContext,
        next: &dyn Fn(E, &mut dyn EffectContext) -> Result<E::Output, EffectError<E>>
    ) -> Result<E::Output, EffectError<E>>;
}
```

Example middleware for logging:

```rust
struct LoggingMiddleware {
    logger: Logger,
}

impl EffectMiddleware for LoggingMiddleware {
    fn process<E: Effect>(
        &self, 
        effect: E, 
        ctx: &mut dyn EffectContext,
        next: &dyn Fn(E, &mut dyn EffectContext) -> Result<E::Output, EffectError<E>>
    ) -> Result<E::Output, EffectError<E>> {
        self.logger.log(format!("Effect: {}", effect.description()));
        let start = Instant::now();
        let result = next(effect, ctx);
        let duration = start.elapsed();
        match &result {
            Ok(_) => self.logger.log(format!("Effect completed in {:?}", duration)),
            Err(e) => self.logger.log(format!("Effect failed in {:?}: {}", duration, e)),
        }
        result
    }
}
```

## References

- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md)
- [System Contract](../../../spec/system_contract.md)
- [Effect System Architecture](../../architecture/core/effect-system.md)
- [Causality Core Library](../../reference/libraries/causality-core.md) 