# Causality: Resource Model and Core Types

This document details the core types and resource model of the Causality system, providing a unified abstraction for representing data, computational processes, and system state. It focuses on how entities are defined, structured using generalized row types, and managed throughout their lifecycle, emphasizing the linear, verifiable nature of operations.

The type system is organized according to the three-Layer Architecture:
- **Layer 0 (Register Machine)**: Defines fundamental machine-level values and base types.
- **Layer 1 (Linear Lambda Calculus)**: Introduces structured types, resources, objects, linearity qualifiers, row types, Lisp data values, and the Lisp Abstract Syntax Tree (AST).
- **Layer 2 (Effect Algebra & Domain Logic)**: Defines types for effects, handlers, intents, and constraints.

## 1. Entity Identification (`EntityId`, `ResourceId`, `ValueExprId`, `ExprId`, `RowTypeId`, `HandlerId`)

Fundamental to the Causality system is **content-addressing**. All significant entities—resources, data values, executable expressions, type schemas, intents, effects, transactions, and handlers—are identified by a unique, deterministic identifier derived from their canonical serialization. This ensures that identical content always yields the same ID, enabling system-wide consistency, deduplication, caching, and verifiable references.

*   **`EntityId`**: The base type for content-addressed identifiers across the system. It is derived from the SSZ serialization of the entity's core data (often the Merkle root for complex structures).
*   **Specific IDs**: Specialized type aliases for `EntityId` are used for clarity, indicating the type of entity being referenced (e.g., `ResourceId` for a `Resource`, `ValueExprId` for a `ValueExpr`, `ExprId` for an `Expr`, `RowTypeId` for a `RowType`, `HandlerId` for a `Handler`).

## 2. Linearity Qualifiers (Layer 1)

Applied at Layer 1, linearity qualifiers control the usage patterns of `Object` instances and other typed data. The system supports four levels of linearity:

```rust
pub enum Linearity {
    /// Must be used exactly once.
    Linear,
    /// May be used at most once (can be dropped).
    Affine,
    /// Must be used at least once (can be copied).
    Relevant,
    /// May be used any number of times (can be dropped and copied).
    Unrestricted,
}
```

These four linearity qualifiers represent the complete set of possibilities when considering resource usage along two orthogonal axes: whether a resource can be dropped without being used (weakening) and whether it can be copied or used multiple times (contraction).

| Linearity Qualifier | Weakening (Can Drop?) | Contraction (Can Copy?) | Typical Use Case |
|---|:---:|:---:|---|
| `Linear` | No | No | Unique resources, critical state | 
| `Affine` | Yes | No | Optional unique resources | 
| `Relevant` | No | Yes | Read-only references needed for an operation | 
| `Unrestricted` | Yes | Yes | Configuration data, freely copyable values | 

This 2×2 matrix captures fundamental resource usage patterns:

- **Linear**: Must be used exactly once.
- **Affine**: May be used at most once.
- **Relevant**: Must be used at least once if part of a computation path that is taken.
- **Unrestricted**: May be used any number of times.

These qualifiers are primarily enforced by the Layer 1 type system and are crucial for ensuring resource safety and enabling formal verification.

## 3. `Resource` and `Object`: Core Linear Entities (Layer 1)

### 3.1 `Resource`: The Strictly Linear Entity

The `Resource` is the fundamental immutable, linear entity in the Causality framework. It represents a single, unique digital asset or state that is consumed exactly once during a state transformation. Resources encapsulate data and metadata, linking their state and behavior under a content-addressed `EntityId` (`ResourceId`).

```rust
pub struct Resource {
    /// Unique identifier for this resource (using EntityId for unified identification)
    pub id: ResourceId,

    /// Human-readable name or description
    pub name: Str,

    /// Domain this resource belongs to
    pub domain_id: DomainId,

    /// Resource type identifier (e.g., "token", "compute_credits", "bandwidth"). This often implicitly refers to associated RowType definitions.
    pub resource_type: Str,

    /// Current quantity/amount of this resource (for quantifiable assets)
    pub quantity: u64,

    /// When this resource was created or last updated
    pub timestamp: Timestamp,

    /// A Product row type instance defining associated permissions or attributes (used for capability patterns).
    pub permissions: Value, // This Value should be a Value::Product conforming to a RowType

    /// A Sum row type instance representing the current state in a state machine.
    pub state: Value, // This Value should be a Value::Sum conforming to a RowType

    /// A Product row type instance holding the intrinsic data of the resource.
    pub data: Value, // This Value should be a Value::Product conforming to a RowType

    /// Cryptographic proof of its origin and transformation history.
    pub causal_chain: CausalProof,

    /// Optional: if this resource also represents a computational budget.
    pub compute_budget: Option<u64>,
}
```

*   **`id` (`ResourceId`)**: The unique, content-addressed identifier of this specific `Resource` instance, derived from its SSZ serialization.
*   **`name` (`Str`)**: A human-readable identifier (does not affect `id`).
*   **`domain_id` (`DomainId`)**: Associates the resource with a specific `TypedDomain` for context and capabilities.
*   **`resource_type` (`Str`)**: Categorizes the resource, often linking to `RowType` schemas.
*   **`quantity` (`u64`)**: For quantifiable resources.
*   **`timestamp` (`Timestamp`)**: Creation/update time.
*   **`permissions` (`Value` - Product Type)**: A `Value::Product` instance whose structure (defined by a `RowType`) can be used to implement permission/capability patterns.
*   **`state` (`Value` - Sum Type)**: A `Value::Sum` instance representing lifecycle state, structured by a `RowType` for state machines.
*   **`data` (`Value` - Product Type)**: A `Value::Product` instance holding the resource's core data, structured by a `RowType`.
*   **`causal_chain` (`CausalProof`)**: Tracks provenance and transformation history.
*   **`compute_budget` (`Option<u64>`)**: For resources representing computation.

Upon creation, a `Resource` resides in a unique register. Transformations consume the original and produce new `Resource` instances in new registers.

### 3.2 `Object`: Generalized Resource with Configurable Linearity

Objects generalize resources with configurable linearity, enabling more flexible resource patterns:

```rust
pub struct Object<T> {
    /// The encapsulated data
    pub data: T,
    
    /// Linearity qualifier controlling usage patterns
    pub linearity: Linearity,
    
    /// Set of capabilities associated with this object
    pub capabilities: Set<Capability>,
}
```

Type relationships illustrate how `Object` generalizes other concepts:
- A `Resource` can be seen as an `Object` with `Linear` linearity, specific data fields, and capabilities.
- A `Capability` itself can be modeled as an `Object` (often `Linear` or `Affine`) whose data field describes the permission.
- A `Message` or freely copyable data could be an `Object` with `Unrestricted` linearity.

## 4. Layer 0 Machine Values (`Value`)

Layer 0 defines the most fundamental values that the register machine operates on. These are simple, unboxed types directly manipulated by the 9 core machine instructions. All higher-level data structures are ultimately compiled down to these representations for execution.

```rust
// Represents the types of values the Layer 0 machine can handle.
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),      // Or a platform-specific integer type
    Symbol(Str),   // For symbolic atoms

    // Machine-level identifiers
    RegisterId(u32), // Identifies a machine register
    ResourceId(u64), // Identifies a heap-allocated resource
    Label(Str),      // Identifies a code location for jumps (e.g., in `match`)
    EffectTag(Str),  // Opaque tag representing a type of effect for `perform`

    // Basic structural forms at Layer 0
    Product(Box<Value>, Box<Value>), // Layer 0's way to pair two values
    Sum(SumVariant),                 // Layer 0's way to represent a choice (e.g., Inl(Box<Value>), Inr(Box<Value>))
}

pub enum SumVariant {
    Inl(Box<Value>),
    Inr(Box<Value>),
}
```

- **Base Types**: `Unit`, `Bool`, `Int`, `Symbol` are the primitive data types.
- **Machine-Level Identifiers**: `RegisterId`, `ResourceId`, `Label`, `EffectTag` are used by the machine for its internal operations.
- **Structural Forms**: `Product` and `Sum` represent the simplest forms of data aggregation and choice, directly corresponding to the SMCC with coproducts structure of Layer 0.

These Layer 0 values are distinct from the richer `LispValue` types used at Layer 1 for programming convenience.

## 5. Layer 1 Lisp Data Values (`LispValue`)

Layer 1 introduces a richer set of data values, `LispValue`, suitable for programming in Causality Lisp. These values are what programmers typically manipulate and are used as constants within Layer 1 `Expr` ASTs. They build upon Layer 0 values but include more complex structures.

```rust
pub enum LispValue {
    Unit,
    Bool(bool),
    Int(i64),
    String(Str),      // UTF-8 String
    Symbol(Str),
    List(Vec<LispValue>), // Ordered list
    Map(std::collections::HashMap<Str, LispValue>), // Key-value map
    Record(std::collections::HashMap<Str, LispValue>), // Structured record with named fields
    
    ResourceId(u64),  // Reference to a Layer 0 resource
    ExprId(u64),      // Reference to a persisted Expr AST
    // Other EntityId variants can also be represented if needed.

    // Note: Lambdas/closures are part of the Expr AST, not typically direct LispValues themselves,
    // unless representing a first-class function value after closure conversion.
}
```
- `LispValue` includes common data types like strings, lists, and maps, providing a more convenient programming model than raw Layer 0 values.
- `Record` provides a way to represent structured data with named fields, often conforming to `RowType` schemas at compile time.
- References like `ResourceId` and `ExprId` allow Lisp programs to refer to other significant entities.

## 6. Layer 1 Lisp AST (`Expr`)

`Expr` instances define the executable logic of Causality Lisp programs as Abstract Syntax Trees (ASTs). These ASTs are processed by the Layer 1 compiler and type checker. `Expr`s are SSZ-serializable and can be content-addressed by an `ExprId`.

The structure of `Expr` directly reflects the 11 core primitives of the Layer 1 Linear Lambda Calculus, along with general programming constructs:

```rust
// Represents a parameter in a lambda or let binding.
pub struct Param { pub name: Str, pub type_annot: Option<Str> }

pub enum Expr {
    // Core Values & Variables
    Const(LispValue),         // Constant LispValue
    Var(Str),                 // Variable reference by name

    // General Programming Constructs
    Let(Str, Option<Str>, Box<Expr>, Box<Expr>), // let name: type = val_expr in body_expr

    // Layer 1 Primitives (Linear Lambda Calculus)
    UnitVal,                                 // The 'unit' primitive value
    LetUnit(Box<Expr>, Box<Expr>),           // 'letunit' u = e1 in e2 (e1 must be unit)
    Tensor(Box<Expr>, Box<Expr>),            // 'tensor e1 e2'
    LetTensor(Box<Expr>, Str, Str, Box<Expr>), // 'lettensor (x,y) = e_pair in e_body'
    Inl(Box<Expr>),                          // 'inl e'
    Inr(Box<Expr>),                          // 'inr e'
    Case(Box<Expr>,                         // 'case e_sum of inl x => e_left | inr y => e_right'
         Str, Box<Expr>,                  // x, e_left
         Str, Box<Expr>),                 // y, e_right
    Lambda(Vec<Param>, Box<Expr>),           // 'lambda (p1:t1, ...) => body'
    Apply(Box<Expr>, Vec<Expr>),             // 'apply fn_expr arg_exprs'
    Alloc(Box<Expr>),                        // 'alloc e' (allocates the value of e as a resource)
    Consume(Box<Expr>),                      // 'consume e' (consumes the resource e)
}
```

- The `Expr` enum provides direct syntactic forms for each of the 11 Layer 1 primitives.
- `Let` provides local bindings.
- `Const` allows embedding `LispValue`s directly into the code.
- This AST is type-checked against Layer 1 types (including linearity and row types) and then compiled down to Layer 0 register machine instructions.

## 7. Generalized Row Types: Schemas for Structure and Validation (`RowType`) (Layer 1)

Generalized `RowType`s are a **compile-time-only** mechanism for defining the structure of data and validating their conformity. They are not runtime values but content-addressed schemas (`RowTypeId`).

Two kinds of `RowType`s exist conceptually:

1.  **Product Row Types**: Define named fields with types (for records like `Resource.data`, `Resource.permissions`).
2.  **Sum Row Types**: Define tagged variants with types (for unions like `Resource.state`, effect signatures).

`RowType`s support compile-time operations (Projection, Restriction, Merge, Diff) used by the type checker and compiler to manipulate and reason about structured data without runtime cost.

## 8. Constraint Language (Layer 2)

Constraints express logical conditions that must hold during program execution:

```rust
pub enum Constraint {
    /// Truth values
    True,
    False,
    
    /// Logical connectives
    And(Box<Constraint>, Box<Constraint>),
    Or(Box<Constraint>, Box<Constraint>),
    Not(Box<Constraint>),
    
    /// Capability and ownership checks
    HasCapability(Symbol),
    IsOwner(ResourceId),
    
    /// Type predicates
    Satisfies(TypeExpr, Predicate),
    
    /// Effect membership
    EffectIn(Effect, Set<Effect>),
}
```

## 8. Hint Language (Layer 2)

Hints are structured optimization directives that guide runtime execution without affecting correctness:

```rust
pub enum Hint {
    /// Batch effects with matching selector
    BatchWith(Selector),
    
    /// Optimize for specific metrics
    Minimize(Metric),
    Maximize(Metric),
    
    /// Domain preferences
    PreferDomain(DomainId),  // Soft preference
    RequireDomain(DomainId), // Hard requirement
    
    /// Timing preferences
    Deadline(Timestamp),
    
    /// Hint combinations
    HintAll(Vec<Hint>),  // Conjunction
    HintAny(Vec<Hint>),  // Disjunction
}

pub enum Selector {
    SameType,       // Effects with same type tag
    SameTarget,     // Effects with same target address
    Custom(ExprId), // User-defined selection predicate
}

pub enum Metric {
    Price,
    Latency,
}
```

## 9. Effect Model (Layer 2)

Effects are tagged operations with structured metadata. They define *what* should happen, not *how*:

```rust
pub struct Effect<T> {
    /// Unique effect type identifier
    pub tag: Symbol,
    
    /// Effect parameters
    pub params: T,
    
    /// Precondition that must hold before execution
    pub pre: Constraint,
    
    /// Postcondition that will hold after execution
    pub post: Constraint,
    
    /// Optimization hints
    pub hints: Vec<Hint>,
}
```

Effects are processed through a two-stage system:
1. **Pure handlers** transform effects (effect-to-effect transformations)
2. **Stateful interpreter** executes the transformed effects

## 10. Handler Model (Layer 2)

Handlers are pure functions that transform effects. They form an effect algebra where composition is just function composition:

```rust
/// Handler type: pure effect-to-effect transformation
pub type Handler<E1, E2> = fn(E1) -> E2;

/// Handler composition is function composition
pub fn compose<E1, E2, E3>(h2: Handler<E2, E3>, h1: Handler<E1, E2>) -> Handler<E1, E3> {
    |e1| h2(h1(e1))
}

/// Identity handler
pub fn id<E>() -> Handler<E, E> {
    |e| e
}
```

This avoids the traditional monad transformer composition problem—no lifting, no transformer stacks, just plain function composition.

## 11. Intent Model (Layer 2)

Intents represent desired state transformations declaratively:

```rust
pub struct Intent {
    /// Resources to be consumed
    pub resources: Vec<Resource>,
    
    /// Constraint that must be satisfied
    pub constraint: Constraint,
    
    /// Effects to be performed
    pub effects: Vec<Effect<dyn Any>>,
    
    /// Optimization hints
    pub hints: Vec<Hint>,
}
```

## 12. Transaction Model

A `Transaction` is an atomic, immutable record of intent execution:

```rust
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: EntityId,

    /// Human-readable name or description
    pub name: Str,

    /// Domain this transaction belongs to
    pub domain_id: DomainId,

    /// All effects included in this transaction (by EntityId)
    pub effects: Vec<EntityId>,

    /// All intents satisfied by this transaction (by EntityId)
    pub intents: Vec<EntityId>,

    /// Aggregated resources consumed by all effects
    pub inputs: Vec<ResourceFlow>,

    /// Aggregated resources produced by all effects
    pub outputs: Vec<ResourceFlow>,

    /// When this transaction was created or executed
    pub timestamp: Timestamp,
}
```

Transaction semantics ensure atomicity: either all effects complete successfully or none are applied.

## 13. Resource Flow

`ResourceFlow` structures describe resource movement through transformations:

```rust
pub struct ResourceFlow {
    pub resource_type: Str,
    pub quantity: u64,
    pub domain_id: DomainId,
}
```

## 14. Serialization and Content Addressing (`SSZ`, `SMT`)

All core types implement SSZ for deterministic serialization. The hash (Merkle root) of the SSZ data is the `EntityId`. These IDs are keys in SMTs, providing authenticated storage and verifiable proofs of inclusion/exclusion.

## 15. Type Safety and Validation

The type system, linear types, and `RowType`s enforce valid resource operations statically. The combination of:
- Linearity qualifiers (preventing misuse)
- Row types (ensuring structural correctness)
- Constraints (expressing logical requirements)
- Pure handlers (composable transformations)
- Stateful interpreter (controlled execution)

provides a comprehensive framework for safe, verifiable resource management.

This comprehensive set of core types, organized by layers and validated by the type system, forms the foundation for the Causality framework's linear, verifiable resource management.