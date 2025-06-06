# 010: Causality Toolkit and Rust DSL

The `causality-toolkit` crate is the primary Rust interface for interacting with and building upon the Causality framework. It provides type-safe abstractions, Domain Specific Languages (DSLs), and utility functions that map directly to Causality's three-layer architecture. This toolkit empowers Rust developers to construct linear resource applications with confidence, leveraging Rust's performance, safety features, and powerful type system.

## 1. Architectural Alignment in Rust

The toolkit mirrors the core architectural layers of Causality, offering Rust-native representations and mechanisms:

### Layer 0: Register Machine Abstractions

The toolkit provides Rust enums and structs to represent the 9 Layer 0 VM instructions (e.g., `Move`, `Alloc`, `Consume`, `Perform`). A `RegisterMachine` trait might define the execution interface, with implementations leveraging Rust's move semantics to help enforce linearity at a low level. `Value`s and `ResourceId`s held in registers are managed carefully.

*Conceptual Instruction Definition:*
```rust
// From the old 07-causality-toolkit-and-rust-dsl.md
pub enum Instruction {
    Move { src: RegisterId, dst: RegisterId },
    Apply { func_reg: RegisterId, arg_reg: RegisterId, out_reg: RegisterId },
    Alloc { val_reg: RegisterId, out_reg: RegisterId },
    Consume { resource_id_reg: RegisterId, val_out_reg: RegisterId },
    // ... other instructions like Match, Select, Witness, Check, Perform
}
```

### Layer 1: Row Types and Causality Lisp DSL

-   **Compile-Time Row Types**: Rust's type system, particularly traits and phantom types, is used to implement row type operations (projection, restriction) at compile time. This ensures zero runtime overhead for capability tracking.
-   **Causality Lisp (`Expr`) Construction**: The toolkit provides a Rust DSL (macros, builder patterns) to ergonomically construct Layer 1 `Expr` ASTs, which represent Causality Lisp programs based on its 11 core primitives.

### Layer 2: Effects, Handlers, and Intents

-   **Pure `Handler`s**: Handlers are typically represented as Rust traits and their implementations as pure functions or methods that transform input `Effect`s into output `Effect`s.
-   **Stateful `Interpreter`s**: Separate `Interpreter` traits/structs manage the stateful execution of effects and TEGs.
-   **`Effect` and `Intent` Definitions**: Macros and builder patterns facilitate the type-safe definition and construction of `Effect`s (with pre/post conditions) and `Intent`s.

## 2. Rust DSL for Causality Lisp (`Expr` AST)

The toolkit aims to make writing Causality Lisp logic intuitive within Rust. Instead of writing raw Lisp strings, developers can use Rust constructs that generate the corresponding `Expr` AST variants.

*Conceptual DSL for Lisp Primitives (inspired by old `07-causality-toolkit-and-rust-dsl.md`):*

-   **`lambda`**: `lambda!( (x: LispValue, y: LispValue) -> lisp_add(x, y) )`  -> `Expr::Lambda`
-   **`app`**: `app!(my_func_expr, arg1_expr, arg2_expr)` -> `Expr::App`
-   **`let`**: `let_!( (x = val1_expr), (y = val2_expr) ; body_expr )` -> `Expr::Let`
-   **`if`**: `if_!(condition_expr, then_expr, else_expr)` -> `Expr::If`
-   **Resource Operations (as `primitive-op`)**:
    -   `create_resource_expr!(type_val, initial_value_expr)`
    -   `read_field_expr!(resource_locator_expr, "field_name")`
    -   `update_field_expr!(resource_locator_expr, "field_name", new_value_expr)`
    -   `consume_resource_expr!(resource_locator_expr)`

These DSL elements would construct variants of the `Expr` enum (defined in `causality-lisp-ast` or similar), which are then fed into the Causality Lisp compiler to produce Layer 0 VM instructions.

## 3. Linearity and Type Safety in Rust

The toolkit deeply leverages Rust's features to enforce linearity:

-   **Ownership and Borrowing**: Linear resources are often wrapped in Rust structs that do not implement `Clone` or `Copy`. Consuming such a resource involves taking ownership, naturally preventing double-use.
    ```rust
    // Conceptual LinearResource wrapper
    #[must_use] // Warns if a linear resource is not explicitly consumed
    pub struct LinearResource<T> {
        value: Option<T>, // Option to allow taking the value, leaving None (consumed state)
    }

    impl<T> LinearResource<T> {
        pub fn new(data: T) -> Self {
            LinearResource { value: Some(data) }
        }

        // Consumes the resource, returning its inner data.
        // Panics if already consumed.
        pub fn consume(mut self) -> T {
            self.value.take().expect("LinearResource already consumed")
        }
    }
    ```
-   **Marker Traits for Linearity Qualifiers**: Traits like `Linear`, `Affine`, `Relevant`, `Unrestricted` can be used to mark types and constrain their usage via generic bounds, enforced at compile time.
-   **`Object<T, L: Linearity>`**: A generic type to represent data `T` with a specific `Linearity` qualifier `L`.

## 4. Defining Effects and Intents in Rust

### Declarative Effect Macros
Macros can simplify `Effect` definition, automatically generating boilerplate for trait implementations, pre/post condition checks, and hint processing.

*Conceptual Macro Usage:*
```rust
// define_effect! macro (hypothetical)
// define_effect! {
//     name: "TransferTokens",
//     domain: "Finance",
//     inputs: { from_account: AccountId, to_account: AccountId, amount: u64 },
//     outputs: { receipt: TransferReceipt },
//     preconditions: |inputs| inputs.amount > 0 && account_exists(inputs.from_account),
//     postconditions: |_, outputs| outputs.receipt.status == "SUCCESS",
//     expression_id: Some(ExprId::from_hash(...)) // Optional Lisp logic
// }
```

### Fluent Intent Builder DSL
A builder pattern provides a clear and type-safe way to construct `Intent`s incrementally.

*Conceptual Builder Usage:*
```rust
// let transfer_intent = Intent::build("TokenTransferIntent")
//     .domain(finance_domain_id)
//     .with_input(ResourceFlow::new(alice_token_resource_id, ...))
//     .with_output(ResourceFlow::new_placeholder(bob_address(), ...))
//     .with_priority(10)
//     .with_lisp_logic(transfer_lisp_expr_id) // Optional Lisp expression for the intent
//     .with_hint(optimization_hint_expr_id)   // Optional hint
//     .finalize();
```

## 5. Key Features and Utilities of the Toolkit

-   **Compile-Time Row Type Operations**: As mentioned, using Rust's type system (phantom types, type-level lists/operations) to manage capabilities without runtime cost.
-   **Temporal Effect Graph (TEG) Construction**: Builder APIs (`TEGBuilder`, `NodeBuilder`) to declaratively construct TEGs, ensuring causal consistency and resource safety.
-   **Zero-Knowledge Proof (ZKP) Integration**: Utilities and traits (e.g., `ZKCompilable`) for effects that need to be compiled into ZK circuits. This includes support for specifying witness schemas and leveraging content-addressed optimizations for ZK proofs.
-   **Testing Framework Integration**: Specialized assertions, property-based testing capabilities (e.g., with `quickcheck`), and utilities for testing each architectural layer (VM instructions, Lisp functions, Handlers, TEG execution).
-   **Performance Optimizations**: Features like content-addressed caching for verified effects and potentially lazy resource loading.

## 6. Benefits of the Causality Toolkit

-   **Compile-Time Safety**: Leverages Rust's strengths to catch many errors (linearity violations, type mismatches, capability misuse) at compile time.
-   **Performance**: Aims for zero-cost abstractions where possible, especially for type-level operations like row types.
-   **Developer Ergonomics**: Provides DSLs and builder patterns to make working with Causality's concepts more natural and less error-prone for Rust developers.
-   **Direct Architectural Mapping**: Offers a clear and direct implementation of Causality's formal three-layer architecture.

The Causality Toolkit is the cornerstone for building robust, verifiable, and efficient linear resource applications in Rust, providing the necessary abstractions while upholding the core principles of the Causality framework.
