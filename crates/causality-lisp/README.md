# Causality Lisp

Unified Lisp interpreter for evaluating Resource logic in the Causality framework. This crate implements the core interpreter responsible for executing `Expr` ASTs that define Resource behavior, validation rules, and system interactions.

## Overview

The `causality-lisp` crate provides a deterministic, asynchronous Lisp interpreter that serves as the execution engine for all Lisp expressions in the Causality system. It evaluates `Expr` ASTs (defined in `causality-types`) that represent the logic associated with Resources, including:

- Resource `static_expr` validation logic
- Handler Resource `dynamic_expr` for orchestrating ProcessDataflowBlocks
- Capability system logic for permission management
- Effect and Intent constraint evaluation

## Core Architecture

### Unified Interpreter Design

The interpreter follows a unified design where a single `Interpreter` instance evaluates all types of Lisp expressions, with behavior customized through the execution context:

- **`Interpreter`**: Core evaluation engine that walks `Expr` ASTs
- **`ExprContextual`**: Trait defining the execution environment and available host functions
- **Host Functions**: Rust-implemented functions callable from Lisp (atomic combinators)
- **Evaluation Context**: Provides Resource state, system data, and domain-specific capabilities

### Key Components

#### `Interpreter` Struct
The primary implementation of the `Evaluator` trait that:
- Walks `Expr` ASTs step-by-step
- Handles atoms, variables, lambda abstractions, and function applications
- Supports built-in `AtomicCombinator`s
- Delegates to host functions for system interactions

#### `ExprContextual` Trait
Defines the interface between the interpreter and its execution environment:

```rust
pub trait ExprContextual: AsExprContext + Send + Sync {
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult>;
    async fn try_call_host_function(&self, fn_name: &Str, args: Vec<ValueExpr>) -> Option<Result<ValueExpr, ExprError>>;
    async fn is_effect_completed(&self, effect_id: &ExprId) -> Result<bool, ExprError>;
    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError>;
    async fn define_symbol(&self, name: Str, value: ExprResult) -> Result<(), ExprError>;
}
```

#### Context Binding Support
- **`BindingExprContext`**: Wraps contexts with temporary symbol bindings
- **`LambdaBindingContext`**: Manages lexical scoping for lambda functions
- Support for `let` bindings and parameter passing

## Evaluation Semantics

### Call-by-Value Evaluation
The interpreter follows call-by-value semantics with special handling for:
- **Control Flow**: `if`, `and`, `or` implement conditional evaluation
- **Short-Circuiting**: Logical operators avoid unnecessary computation
- **Special Forms**: `let` extends environments before evaluation

### Stateless Core
The interpreter itself is stateless and side-effect free. All state management and side effects are handled through:
- Host function calls via `ExprContextual`
- Context-provided symbol resolution
- External state management by the runtime

### Error Handling
Comprehensive error handling using `ExprError` variants:
- Execution errors for runtime failures
- Type mismatches for invalid operations
- Missing symbol errors for undefined variables

## Supported Expressions

The interpreter evaluates standard Lisp forms represented as `Expr` AST nodes:

### Basic Forms
- **Atoms**: Integers, strings, booleans, nil (`Expr::Atom`)
- **Constants**: Pre-evaluated `ValueExpr`s (`Expr::Const`)
- **Variables**: Symbol names resolved via context (`Expr::Var`)

### Function Forms
- **Lambda Abstractions**: Anonymous functions (`Expr::Lambda`)
- **Function Application**: Applies functions to arguments (`Expr::Apply`)
- **Atomic Combinators**: Predefined operations (`Expr::Combinator`)

### Special Forms
- **Dynamic Evaluation**: Step-bounded evaluation for ZK circuits (`Expr::Dynamic`)

## Atomic Combinators

The interpreter supports a comprehensive set of atomic combinators organized by category:

### Control Flow
- `if`: Conditional execution
- `and`, `or`: Short-circuiting logical operations
- `not`: Boolean negation

### Arithmetic Operations
- `add` (`+`), `sub` (`-`), `mul` (`*`), `div` (`/`): Basic arithmetic
- Support for both symbolic and named forms

### Comparison Operations
- `eq`: Equality testing
- `gt`, `lt`, `gte`, `lte`: Numeric comparisons

### Data Access and Construction
- `get-field`: Access fields from maps/records
- `get-context-value`: Access execution context values
- `list`: Construct lists
- `record`: Construct records
- `nth`, `len`: List operations

### Type Predicates
- `is-string?`, `is-integer?`, `is-list?`: Type checking functions

### String Operations
- `string-concat`: String concatenation
- `string-to-upper`: Case conversion

## Dataflow Orchestration

Special support for ProcessDataflowBlock orchestration through dataflow combinators:

### Dataflow Combinators
- `get-dataflow-definition`: Retrieve dataflow block definitions
- `evaluate-gating-condition`: Evaluate process edge conditions
- `instantiate-effect-from-node`: Create effects from dataflow nodes
- `emit-effect-on-domain`: Emit effects to specific domains
- `update-dataflow-instance-state`: Update dataflow execution state

### ZK Compatibility
- `is-zk-compatible-operation`: Check ZK circuit compatibility
- `validate-dataflow-step-constraints`: Validate step constraints

## Integration with Runtime

### Host Function Bridge
The interpreter integrates with the runtime through host functions:

1. **Symbol Resolution**: Context provides variable bindings and function references
2. **Host Function Calls**: Rust functions exposed as Lisp combinators
3. **Resource Access**: Context provides access to Resource state via `*self-resource*`
4. **System Integration**: Host functions bridge to broader system capabilities

### Execution Flow
```
Runtime Request → Context Setup → Interpreter.evaluate_expr() → Host Function Calls → Result
```

### Context Configuration
Different execution contexts provide different capabilities:
- **Static Validation**: Restricted context for `static_expr` evaluation
- **Handler Execution**: Full context with dataflow combinators
- **Capability Checks**: Security-focused context for permission logic

## Usage Examples

### Basic Evaluation
```rust
use causality_lisp::{Interpreter, Evaluator};
use causality_types::expr::ast::Expr;

async fn evaluate_expression(expr: &Expr, ctx: &dyn ExprContextual) -> Result<ExprResult, ExprError> {
    let interpreter = Interpreter::new();
    interpreter.evaluate_expr(expr, ctx).await
}
```

### Resource Validation
```rust
// Evaluating a Resource's static_expr with its own state bound to *self-resource*
let result = interpreter.evaluate_expr(&resource_static_expr, &validation_context).await?;
```

### Handler Orchestration
```rust
// Handler evaluating dataflow logic with specialized combinators
let result = interpreter.evaluate_expr(&handler_dynamic_expr, &dataflow_context).await?;
```

## Feature Flags

- **default**: Standard library features
- **std**: Enables standard library dependencies
- **async**: Enables asynchronous evaluation (default)
- **wasm**: WebAssembly optimization
- **zk**: Zero-Knowledge circuit optimization

## Compatibility

### Environment Support
- **std/no_std**: Compatible with both environments
- **WebAssembly**: Optimized for WASM targets
- **ZK Circuits**: Designed for zero-knowledge proof systems

### Serialization
- All types use SSZ serialization for deterministic behavior
- Content-addressed evaluation results
- ZK-compatible data structures

## Integration Points

### With `causality-types`
- Consumes `Expr` AST definitions
- Produces `ExprResult` and `ValueExpr` outputs
- Uses content-addressed identifiers (`ExprId`, `ValueExprId`)

### With `causality-runtime`
- Runtime provides `ExprContextual` implementations
- Host functions bridge to system capabilities
- Context configuration based on execution mode

### With Resource System
- Evaluates Resource `static_expr` for validation
- Processes Handler `dynamic_expr` for orchestration
- Supports capability system logic evaluation

## Deterministic Execution

The interpreter ensures deterministic execution through:
- **Pure Evaluation**: No side effects within interpreter logic
- **Consistent Host Functions**: Deterministic host function implementations
- **Content-Addressed Results**: Reproducible evaluation outcomes
- **Bounded Execution**: Step limits for dynamic evaluation

This design enables verifiable computation where the same `Expr` with the same context always produces the same result, crucial for zero-knowledge proof systems and distributed verification.