# Lisp Interpreter and Expression System

The Causality framework incorporates a Lisp-based expression system that serves as the computational foundation for defining transformation logic, validation rules, and complex data manipulations. This system provides a functional programming environment optimized for the framework's resource-based computational model.

## Expression Architecture

The expression system operates on two primary levels: value expressions that represent data and state, and executable expressions that define computational logic. This separation enables clear distinction between data representation and computation while maintaining the functional programming paradigm.

Value expressions (`ValueExpr`) represent all data and state. They are SSZ-serialized, and their Merkle root forms a content-addressed `ValueExprId`. This ensures deterministic and verifiable data representation, crucial for ZK-compatibility. Key `ValueExpr` variants include:

```rust
// Reflects causality-types/src/expression/value.rs
pub enum ValueExpr {
  Nil, // Represents a unit or void type
  Bool(bool),
  String(Str), // Specialized Str type for determinism
  Number(Number), // Numeric type (e.g., integer, fixed-point)
  List(ValueExprVec), // Wrapper for Vec<ValueExpr>
  Map(ValueExprMap), // Wrapper for BTreeMap<Str, ValueExpr> (for maps)
  Record(ValueExprMap), // Wrapper for BTreeMap<Str, ValueExpr> (for structs)
  Ref(ValueExprRef), // Can point to ValueExprId or ExprId
  Lambda { params: Vec<Str>, body_expr_id: ExprId, captured_env: ValueExprMap },
}
```

Executable expressions (`Expr`) define computational logic as Abstract Syntax Trees (ASTs). Like `ValueExpr`, `Expr`s are SSZ-serialized, with their Merkle root forming a content-addressed `ExprId`. This "code-as-data" approach allows logic to be stored, referenced, and verified (e.g., via SMTs). Core `Expr` AST variants include: `Atom` (literals), `Const` (embedded `ValueExpr`), `Var` (variables), `Lambda` (anonymous functions, taking parameters and a boxed body expression), `Apply` (function calls, taking a boxed function and a vector of argument expressions), `Combinator` (predefined operations), and `Dynamic` (for step-bounded evaluation).

## Dataflow Execution Model for Effects

While `Expr` and `ValueExpr` define individual computations and data, the Causality system requires a structured way to execute a collection of `Effect`s within a transaction. This execution must be verifiable, replayable, and efficient, especially in a zero-knowledge context.

The system adopts a bounded dataflow model. This model leverages the fact that each resource in Causality functions like a register in a static single-assignment (SSA) environment, making dataflow a natural fit.

### Dataflow Block Structure

A `DataflowBlock` represents a statically committed subgraph of execution for a concrete set of `Effect`s:

```rust
pub struct DataflowBlock {
    pub layers: Vec<Layer>,
    pub edges: Vec<Edge>,
}

pub struct Layer {
    pub node_ids: Vec<NodeId>, // NodeIds typically reference Effects
}

pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub expression: ExpressionId, // Edge gating condition (a Lisp ExprId)
}
```

-   `Node`s (referenced by `NodeId`) represent `Effect`s to be executed.
-   `Edge`s describe resource and control flow between effects.
-   `Layer`s represent execution stages: all nodes within a single layer are conceptually evaluated in parallel (or their order doesn't affect the outcome of that layer). Layers are executed sequentially.
-   Edges can be conditionally activated using committed Lisp `expression` logic (evaluated in-circuit). This allows for dynamic control flow within the static graph.
-   The full `DataflowBlock` is included in the `ProgramCommitment`, enabling a bounded and reproducible transaction structure. This is crucial for ZK-proofs, as the circuit can be precompiled or templated against this known topology.

### Language-Level Design Goals for a Dataflow DSL

To support intuitive programming that maps to this `DataflowBlock` model, a developer-facing language or DSL (likely Lisp-based) should include primitives that:

#### Encourage Resource Clarity

-   Use named, one-shot resource declarations (e.g., `(let x: Token (...))`).
-   Prevent implicit reuse or mutation of resources, aligning with the SSA nature.
-   Explicitly annotate ephemeral resources to distinguish internal data flow from long-term state.

#### Express Layered Control Flow Declaratively

-   Use `stage` or `layer` keywords (or similar Lisp constructs) to organize parallel evaluation:

    ```lisp
    (stage 1
      (effect withdraw (...))
      (effect log_event (...)))
    (stage 2
      (effect send_output (...) (if (> amount 0))))
    ```

-   Allow conditional edge activation via embedded `if`/`when` modifiers on effects or dataflow connections:

    ```lisp
    (effect unlock (...) (when (> time expiry)))
    ```

#### Make Dataflow Explicit but Intuitive

-   Model effect outputs as bindings that flow to the next stage or to subsequent effects within the same stage if dependencies allow:

    ```lisp
    (let unlocked (effect unlock (vault_id) (when (> time expiry))))
    (effect send_to (unlocked recipient))
    ```

-   Under the hood, these DSL constructs would expand to the appropriate `Layer` and `Edge` structures in the `DataflowBlock`, with conditional logic translating to `expression`s on `Edge`s. This translation leverages the dynamic expression system:
    *   The conditional clause, like `(when (> time expiry))`, is compiled into a reusable Lisp `Expr` (expression tree). This `Expr` is stored, and its unique `ExprId` is referenced.
    *   The `Effect` struct (e.g., for `unlock`) typically stores this `ExprId` in its `expression` field.
    *   At runtime, when the system (e.g., a `Handler` orchestrating a dataflow or the `DataflowBlock` executor) encounters this effect, it uses the Lisp interpreter to evaluate the referenced `Expr`.
    *   Crucially, the interpreter is provided with a dynamic **execution context**. This context supplies the current values for symbols used in the expression (like `time` and `expiry` for that specific effect instance).
    *   The boolean result of this dynamic evaluation then determines if the effect proceeds and, consequently, if associated dataflow edges are activated. This allows for context-sensitive, declarative control over the execution flow.

#### Enable Static Analysis and Optimizations

-   The resource graph derived from the DSL can be statically validated (e.g., no reuse, type matching between effect outputs and inputs, detection of dead edges or unreachable effects).
-   The evaluation order of effects is structurally determined by layers and gating conditions.
-   Developers can be warned at compile-time about potential issues like resource leakage or sources of non-determinism not captured by gating conditions.

## Combinator System

Atomic combinators are predefined host functions provided by the execution environment (e.g., `LispHostEnvironment` in `causality-runtime`). They are the primary means by which `Expr` logic interacts with the system, including Resources and their `ValueExpr` states. The set of available combinators is context-dependent, influenced by factors like the `TypedDomain` of an operation or the purpose of the Lisp evaluation (e.g., `"static_validation_verifiable"`, `"dataflow_orchestration"`).

Key categories of combinators include:
- Core Control Flow: `if`, `and`, `or` (with short-circuiting).
- Logical & Comparison: `not`, `eq`, `gt`, `lt`, etc.
- Integer Arithmetic: `+`, `-`, `*`, `/` (operating on `ValueExpr::Number(Integer(i64))`).
- Data Access & Construction:
  - Contextual: `get-context-value` (e.g., to access `*self-resource*`).
  - Structure: `get-field` (for `ValueExpr::Record`/`Map`), list operations (`list`, `nth`, `len`), map/record operations.
  - Effect Status: `completed` (to check effect states).
- String Operations: `string-concat`, etc. (on `ValueExpr::String(Str)`).
- Type Predicates: `is-string?`, `is-integer?`, etc.
- Dataflow Orchestration (Conceptual): For Handlers managing `ProcessDataflowBlock`s, combinators like `get-dataflow-definition`, `evaluate-gating-condition`, `emit-effect-on-domain` allow structured process execution.

Special forms like `if`, `and`, `or`, and `let` have dedicated handling in the interpreter to ensure correct, non-eager evaluation semantics.

## Interpreter Implementation

The system uses a two-tiered approach for Lisp execution:

1. Core Lisp Interpreter (`causality-lisp`): This crate provides a unified Lisp interpreter responsible for evaluating `Expr` ASTs. It takes an `Expr` and an execution context (implementing the `ExprContextual` trait) and produces a `Result<ValueExpr, InterpreterError>`. The core evaluation is stateless and generally follows call-by-value semantics, with special handling for forms like `if`.

2. Runtime Orchestration (`causality-runtime`):
  - The `TelInterpreter` (Temporal Effect Language Interpreter) orchestrates Lisp evaluation within the runtime. It sets up the execution context, invokes the `causality-lisp` interpreter, and processes results.
  - The `LispHostEnvironment` implements the `ExprContextual` trait (via an adapter like `TelLispAdapter`). It provides concrete implementations for host functions (atomic combinators) and access to runtime services like the `StateManager` (for Resource `ValueExpr` states).
  - `LispContextConfig` allows the `TelInterpreter` to dynamically configure the Lisp environment for each call. This includes:
    - Host Function Profile: Selects the set of available combinators, often based on the `TypedDomain` (e.g., `VerifiableDomain`, `ServiceDomain`) or task (e.g., `"static_validation_verifiable"`, `"dataflow_orchestration"`).
    - Initial Bindings: Pre-defines symbols like `*args*` (input arguments) or `*self-resource*` (the current Resource's `ValueExpr` state for `static_expr` evaluation).

The `ExprContextual` trait is minimal, typically requiring `get_symbol(name: &str)` and `try_call_host_function(fn_name: &str, args: Vec<ValueExpr>)`.

## Expression Evaluation

Expression evaluation primarily uses call-by-value semantics: arguments to functions and combinators are evaluated before application. However, special forms like `if`, `and`, and `or` implement non-eager evaluation (e.g., conditional branch evaluation, short-circuiting).

The core Lisp evaluation is stateless; side effects are managed by the runtime environment based on the Lisp evaluation's outcome, not within the interpreter itself.

Lambda expressions (`Expr::Lambda`) create closures, capturing their lexical environment (as a `ValueExprMap` in their `ValueExpr::Lambda` representation). This supports standard lexical scoping. Function application (`Expr::Apply`) handles user-defined lambdas and combinators (host functions) uniformly.

## Integration with Resource Model

Lisp is central to defining and managing Resource behavior. Key use cases include:

- `static_expr` Evaluation: Resources can have an optional `static_expr: Option<ExprId>`. This Lisp `Expr` defines validation rules and invariants for the Resource's `ValueExpr` state. The `TelInterpreter` evaluates this `Expr` in a restricted context, with the Resource's own state typically bound to `*self-resource*`.
- Capability System Logic: Permissions and authorizations are managed by Lisp logic, often involving "Capability Resources" whose `ValueExpr` state represents grants and whose `ExprId` may point to validation logic.
- Orchestrating `ProcessDataflowBlock`s: Handler Resources use their `dynamic_expr` (Lisp code) to drive `ProcessDataflowBlock`s (declarative multi-step workflows). This Lisp logic uses specialized "Dataflow Orchestration Combinators" to manage the flow, instantiate effects, and interact with different `TypedDomain`s.

The `LispHostEnvironment` provides combinators that allow Lisp `Expr`s to interact with Resource states (via `StateManager`), system information, and domain-specific functionalities.

## S-Expression Syntax

Lisp S-expressions (Symbolic Expressions) provide a human-readable syntax for defining `Expr` logic.

Fundamental Components:
- Atoms: Basic elements like symbols (e.g., `my-var`, `+`, `get-field`) and literals (e.g., `42`, `"hello"`, `true`, `nil`). Literals map directly to `ValueExpr` variants (e.g., `ValueExpr::Number(Integer(i64))`, `ValueExpr::String(Str)`).
- Lists: Parenthesized sequences `(...)`, typically representing function/combinator applications (e.g., `(+ 10 20)`).

S-expressions vs. SSZ (Simple Serialize):
While S-expressions are used for development, readability, and defining logic textually, the canonical representation for `Expr` and `ValueExpr` instances (used for `ExprId`/`ValueExprId` generation, SMT storage, and ZK-proofing) is their SSZ-serialized form.
A parser converts S-expressions into the `Expr` AST. This AST is then SSZ-serialized, and its Merkle root becomes the `ExprId`.

Example S-expressions:

```clojure
(+ 1 (get-field *self-resource* "counter"))
(if (eq? (get-context-value "operation_type") "critical")
  (validate-strict input_data)
  (validate-lenient input_data))
(fn (x y) (+ (* x x) (* y y))) ; Lambda definition
```

## Error Handling and Debugging

The Lisp interpreter returns `Result<ValueExpr, InterpreterError>`. InterpreterError variants provide details on issues like unknown symbols, type mismatches, arity errors, or invalid operations. The runtime environment (e.g., TelInterpreter) handles these errors. Debugging capabilities may include tracing and context inspection.

## Performance Considerations

Performance is addressed through several strategies:
- Optimized Combinators: Host function implementations aim for efficiency.
- Off-Chain First Evaluation: Logic like Resource `static_expr` validation is primarily executed by the off-chain runtime, with results potentially serving as ZK witness data. This minimizes on-circuit computation.
- Ahead-of-Time (AOT) Compilation: For ZK-provable logic or frequently used `Expr`s, AOT compilation to a more constrained, verifiable intermediate representation (like a ZK circuit language) can be employed.
The system balances flexibility of Lisp with the performance and verifiability needs of ZK-proof systems.

## Extensibility and Customization

Extensibility is primarily achieved through:
- Custom Host Functions (Combinators): The `LispHostEnvironment` can be extended with new combinators to provide domain-specific operations or integrate with new services.
- `LispContextConfig`: Dynamically tailoring the set of available host functions and initial variable bindings per evaluation call allows for context-specific capabilities and security profiles.
This approach allows the core interpreter to remain generic while the runtime environment provides specialized features as needed.

## Current Implementation Status

The Lisp system provides a robust foundation for defining and executing logic within the Causality framework. Key features include:
- A unified core Lisp interpreter (`causality-lisp`) for `Expr` AST evaluation.
- Runtime orchestration (`TelInterpreter`, `LispHostEnvironment` in `causality-runtime`) for managing Lisp execution, providing context, and integrating with system services.
- Content-addressable `Expr` and `ValueExpr` types using SSZ-serialization and Merkle-rooted IDs (`ExprId`, `ValueExprId`), suitable for SMT storage and ZK-verification.
- A rich set of host functions (atomic combinators) with context-sensitive availability (e.g., domain-specific, task-specific).
- Support for S-expression syntax for development, with SSZ as the canonical format.
- Integration with the Resource model for `static_expr` validation, capability systems, and `ProcessDataflowBlock` orchestration.

Ongoing work focuses on refining ZK integration, expanding the combinator set, and optimizing performance for diverse workloads.