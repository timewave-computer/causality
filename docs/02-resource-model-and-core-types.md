# Resource Model and Core Types

The Resource model forms the conceptual foundation of the Causality framework, providing a unified abstraction for representing data, computational processes, and system state. This model treats all entities within the system as Resources that can be created, transformed, and consumed through well-defined operations. At its core, a Resource binds its state (data) and the logic governing its behavior under a single, verifiable, content-addressed identifier.

## Resource Structure and Properties

Resources in the Causality framework encapsulate both data and metadata necessary for content addressing and system operation. Each Resource contains an `EntityId` that serves as its content-addressed identifier, ensuring that Resources with identical content receive identical identifiers regardless of their creation context. The `Resource` structure also includes fields for a human-readable name, domain association, type, quantity, and timestamp.

The core structure of a Resource is as follows:

```rust
pub struct Resource {
    /// Unique identifier for this resource (using EntityId for unified identification)
    pub id: EntityId,
    
    /// Human-readable name or description
    pub name: Str,
    
    /// Domain this resource belongs to
    pub domain_id: DomainId,
    
    /// Resource type identifier (e.g., "token", "compute_credits", "bandwidth")
    pub resource_type: Str,
    
    /// Current quantity/amount of this resource
    pub quantity: u64,
    
    /// When this resource was created or last updated
    pub timestamp: Timestamp,
}
```

- `id`: `EntityId`: The unique, content-addressed identifier of the Resource. It uses `EntityId` for unified identification across different types of entities in the system.
- `name`: `Str`: A human-readable name or description for the Resource. This provides semantic meaning without affecting the content-addressed identifier.
- `domain_id`: `DomainId`: Identifies the `TypedDomain` to which the Resource belongs or is primarily associated. This dictates its execution context and available capabilities.
- `resource_type`: `Str`: A string identifier that categorizes the Resource (e.g., "token", "data_object", "compute_credits"). This allows for type-based operations and filtering.
- `quantity`: `u64`: Represents the current amount or quantity of this Resource, applicable for quantifiable assets.
- `timestamp`: `Timestamp`: Marks when the Resource was created or last updated. While important for temporal ordering and versioning, it typically does not affect the content-addressed `id`.

This structure ensures that a Resource is a well-defined unit, linking its identity and core properties. The `name`, `resource_type`, `quantity`, and `timestamp` fields provide essential metadata, while `domain_id` establishes its operational context.

## Value Expressions

Value Expressions (`ValueExpr`) instances represent all concrete data and state within the Causality framework. They are designed to be SSZ-serialized, allowing their content-addressed identifiers (`ValueExprId`) to be derived from their Merkle roots. This ensures data integrity and verifiability.

The `ValueExpr` enum defines the various types of values that can be represented:

```rust
pub enum ValueExpr {
    /// Represents a unit or void type.
    Unit,
    /// An alias for `Unit`, often used to represent null or empty values.
    Nil,
    /// A boolean true/false value.
    Bool(bool),
    /// A UTF-8 string, typically with a fixed-size representation for SSZ compatibility (e.g., `Str`).
    String(Str),
    /// A numeric value. The `Number` type can represent various forms like integers, fixed-point numbers, or ratios.
    Number(Number),
    /// An ordered list of `ValueExpr` instances.
    List(ValueExprVec), // Wrapper for Vec<ValueExpr>
    /// A key-value map where keys are `Str` and values are `ValueExpr`.
    Map(ValueExprMap),  // Wrapper for BTreeMap<Str, ValueExpr>
    /// A structured record with named fields, essentially a `ValueExprMap` used with specific semantics.
    Record(ValueExprMap),
    /// A reference to another `ValueExpr` or `Expr`, typically via its content-addressed ID.
    Ref(ValueExprRef),
    /// A lambda closure, capturing parameters, the body expression's ID, and its captured environment.
    Lambda {
        params: Vec<Str>,
        body_expr_id: ExprId,
        captured_env: ValueExprMap,
    },
}
```

- `Nil`: Represent empty or null-like values.
- `Bool(bool)`: Standard boolean.
- `String(Str)`: Textual data. `Str` is a specialized string type for efficient SSZ.
- `Number(Number)`: Encapsulates various numeric types (e.g., integers, fixed-point). The specific `Number` type (e.g., `crate::primitive::number::Number`) provides the actual representation.
- `List(ValueExprVec)`: Ordered collections.
- `Map(ValueExprMap)`, `Record(ValueExprMap)`: Key-value stores. `Record` implies a more structured, schema-like usage.
- `Ref(ValueExprRef)`: Enables linking to other content-addressed data or expressions.
- `Lambda`: Represents first-class functions, crucial for the Lisp evaluation model.

These types form the building blocks for all data manipulated and stored by the system.

## Executable Expressions (`Expr`)

`Expr` (Expression) instances define the executable logic, behavior, validation rules, and transformations within the Causality framework. They are represented as Lisp Abstract Syntax Trees (ASTs). This "code-as-data" approach allows logic itself to be treated as data, serialized using SSZ, and content-addressed via an `ExprId` (the Merkle root of the SSZ-serialized AST).

The `Expr` enum outlines the structure of these executable expressions:

```rust
pub enum Expr {
    /// Atomic value (e.g., number, string, boolean, nil). `Atom` is a type that wraps these primitives.
    Atom(Atom),
    /// Constant `ValueExpr`. Useful for embedding literal data directly within an expression tree.
    Const(ValueExpr),
    /// Variable reference, identified by a `Str` (string name).
    Var(Str),
    /// Lambda abstraction (anonymous function).
    /// Takes a vector of parameter names (`Vec<Str>`) and a boxed expression (`ExprBox`) for the body.
    Lambda(Vec<Str>, ExprBox),
    /// Function application.
    /// Applies a function (an `ExprBox`) to a list of arguments (an `ExprVec`).
    Apply(ExprBox, ExprVec), // ExprBox is Box<Expr>, ExprVec is Vec<Expr>
    /// An atomic, predefined combinator or primitive operation (e.g., arithmetic, list operations).
    Combinator(AtomicCombinator),
    /// Dynamic expression for step-bounded evaluation, often used in ZK coprocessor contexts.
    /// Takes a step bound (`u32`) and a boxed expression (`ExprBox`) to evaluate.
    Dynamic(u32, ExprBox),
}
```

- `Atom(Atom)`: Basic literal values.
- `Const(ValueExpr)`: Embeds a `ValueExpr` directly as a constant in the AST.
- `Var(Str)`: Represents a named variable to be looked up in the current evaluation environment.
- `Lambda(Vec<Str>, ExprBox)`: Defines an anonymous function. `ExprBox` is a wrapper for `Box<Expr>`, representing the function's body.
- `Apply(ExprBox, ExprVec)`: Represents the application of a function to arguments. `ExprVec` is a wrapper for `Vec<Expr>`.
- `Combinator(AtomicCombinator)`: Core, non-decomposable operations provided by the runtime.
- `Dynamic(u32, ExprBox)`: Allows for expressions whose evaluation is bounded by a certain number of computational steps, relevant for gas-constrained or verifiable computation environments.

These expression types form a Lisp-like language, enabling the definition of complex logic that can be associated with Resources and evaluated in various `TypedDomain` contexts.

## Intent Model and Transformation Logic

Intents represent desired state transformations within the system, expressing what should happen rather than how it should be accomplished. This declarative approach enables the framework to optimize execution strategies while maintaining clear separation between specification and implementation.

The Intent structure captures transformation requirements through input and output ResourceFlow specifications. These flows describe the types and quantities of Resources required for the transformation and the expected outputs. The framework uses this information for validation, optimization, and resource allocation.

```rust
pub struct Intent {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub priority: u32,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
    pub optimization_hint: Option<ExprId>,
    pub target_typed_domain: Option<TypedDomain>,
    pub process_dataflow_hint: Option<ProcessDataflowInitiationHint>,
}
```

Priority levels enable ordering of Intent execution when multiple Intents compete for resources or execution capacity. The optional expression field allows complex transformation logic to be specified through the framework's Lisp-based expression system.

Optimization hints provide guidance to the execution engine about preferred strategies or constraints. These hints can influence domain selection, resource allocation, and execution ordering without mandating specific implementation approaches.

## Effect Model and State Changes

Effects represent the actual state changes that occur when Intents are processed. They capture both the transformation logic and the resulting system state modifications, providing a complete record of system evolution.

The Effect structure includes comprehensive information about the transformation context, including source and target domains, the Handler responsible for execution, and detailed resource flows. This information enables auditing, debugging, and system analysis.

```rust
pub struct Effect {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effect_type: Str,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
    pub resources: Vec<ResourceFlow>,
    pub nullifiers: Vec<ResourceFlow>,
    pub scoped_by: HandlerId,
    pub intent_id: Option<ExprId>,
    pub source_typed_domain: TypedDomain,
    pub target_typed_domain: TypedDomain,
    pub cost_model: Option<EffectCostModel>,
    pub resource_usage_estimate: Option<ResourceUsageEstimate>,
    pub originating_dataflow_instance: Option<ProcessDataflowInstanceId>,
}
```

Nullifiers within Effects prove that specific Resources were consumed during the transformation. This mechanism enables privacy-preserving resource management by proving consumption without revealing the consumed Resource content.

Cost models and resource usage estimates support optimization and resource planning. These fields enable the framework to make informed decisions about execution strategies and resource allocation.

## Resource Flow and Transformation Patterns

ResourceFlow structures describe the movement of Resources through the system during transformations. They specify resource types, quantities, and domain contexts, enabling precise specification of transformation requirements and outputs.

```rust
pub struct ResourceFlow {
    pub resource_type: Str,
    pub quantity: u64,
    pub domain_id: DomainId,
}
```

Resource flows enable the framework to validate that Intents have sufficient inputs and that Effects produce the expected outputs. This validation occurs at both the type level and the quantity level, ensuring system consistency.

The domain context within resource flows enables cross-domain transformations, where Resources move between different execution environments. This capability supports complex workflows that span multiple computational contexts.

## Handler Model and Execution Context

Handlers define the execution context and capabilities available during Effect processing. They encapsulate the logic necessary to transform Intents into Effects, providing the bridge between declarative specifications and actual computation.

```rust
pub struct Handler {
    pub id: HandlerId,
    pub name: Str,
    pub domain_id: DomainId,
    pub handler_type: Str,
    pub priority: u32,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
}
```

Handler types enable specialization for different kinds of transformations. Some handlers might focus on data transformation, others on external service integration, and still others on complex computational workflows.

Priority levels among handlers enable conflict resolution when multiple handlers could process the same Intent. The framework can select the most appropriate handler based on priority, capabilities, and current system state.

## Transaction Model and Atomicity

Transactions group multiple Effects into atomic units, ensuring that either all Effects in a transaction succeed or none do. This mechanism provides consistency guarantees for complex multi-step operations.

```rust
pub struct Transaction {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effects: Vec<EntityId>,
    pub timestamp: Timestamp,
}
```

Transaction boundaries ensure system consistency and allow for sophisticated error handling and recovery strategies.

## Serialization and Content Addressing

All core types, including `Resource`, `ValueExpr`, and `Expr`, implement Simple Serialize (SSZ) encoding. SSZ ensures deterministic and efficient serialization, a critical property for a content-addressed system. The canonical SSZ representation of an entity is hashed (typically producing a Merkle root if the structure is complex) to generate its unique content-addressed identifier: `ResourceId`, `ValueExprId`, or `ExprId`.

This content addressing mechanism ensures that identical data or logic will always produce the identical identifier, regardless of when or where it is created. This enables powerful system-wide deduplication, caching, and verification capabilities.

To provide authenticated and verifiable storage for these SSZ-identified entities, the system employs Sparse Merkle Trees (SMTs). Each entity's content-addressed ID serves as its unique key within a global or domain-specific SMT. The SMT maps this ID to the entity's full SSZ-serialized data (or a commitment to it). SMTs are authenticated data structures, meaning they provide cryptographic proof (a Merkle proof) of an entity's inclusion, exclusion, or current state relative to the SMT's root hash. This is vital for data integrity, partial state disclosure in ZK proofs, and overall system verifiability.

## Integration with Expression System

The core types integrate seamlessly with the framework's Lisp-based expression system primarily through `ExprId` references. As seen in the `Resource` struct (`static_expr`) and various model definitions like `Intent` and `Handler`, complex transformation logic, validation rules, or behavioral definitions can be stored as `Expr` instances (identified by their `ExprId`) and referenced where needed.

This integration enables code reuse, versioning (as changing logic changes the `ExprId`), and sophisticated dependency management. Expressions can be treated as first-class entities, enabling meta-programming and dynamic system behavior. The evaluation of these expressions is performed by a Lisp interpreter, and for critical operations, the system may commit to a specific `InterpreterId` for system-wide consistency.

## Type Safety and Validation

The framework's type system provides compile-time and runtime validation of Resource operations. Type constraints ensure that only valid transformations can be specified and executed, reducing the likelihood of runtime errors.

Trait implementations provide common interfaces for Resource manipulation, enabling generic algorithms while maintaining type safety. The `AsResource` trait, for example, enables uniform handling of Resource-like entities regardless of their specific implementation.

```rust
pub trait AsResource {
    fn resource_type(&self) -> &Str;
    fn quantity(&self) -> u64;
    fn matches_pattern(&self, pattern: &ResourcePattern) -> bool;
}
```

Pattern matching capabilities enable sophisticated Resource selection and filtering operations. ResourcePattern structures can specify constraints on resource types, domains, quantities, and other properties.