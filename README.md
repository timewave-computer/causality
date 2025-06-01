# Causality

A framework for verifiable cross-domain computations using Resource-based state management and Zero-Knowledge proofs.

![](./causality.png)

## Overview

Causality is built around a core primitive: the Resource. Resources are content-addressed entities that bind state (data) to the logic that governs their behavior and transformations. This unified model enables deterministic and verifiable computation across domains.

The framework provides a Resource-centric architecture where all state and logic are represented as content-addressed Resources. Executable expressions (`Expr`) define Resource behavior and constraints, while verifiable storage uses SSZ serialization with Sparse Merkle Trees for cryptographic verification. The system supports typed domains with `VerifiableDomain` for ZK-provable operations and `ServiceDomain` for external interactions, complemented by multi-language DSL support through both Rust and OCaml toolkits for constructing Lisp expressions.

## Resource Model

A Resource comprises an `id` as a content-addressed identifier (SSZ Merkle root), a `value` reference to SSZ-encoded state data (`ValueExpr`), an optional `static_expr` for validation logic used in off-chain verification, a `primary_domain_id` indicating the primary execution domain, and `contextual_scope_domain_ids` for additional domains enabling cross-domain interactions.

Resources can represent data, effects, handlers, capabilities, and even system operations themselves, creating a recursive "code-as-data" architecture.

## Algebraic Effects

Causality leverages algebraic effects as a foundational abstraction to separate program logic from domain-specific implementations. The system implements a Rust Algebraic Effect System that allows developers to define effects and their handlers directly in Rust while integrating seamlessly with the Lisp-based execution model.

Effects are defined using Rust traits (`Effect`, `EffectInput`, `EffectOutput`) and handlers implement the `EffectHandler` trait. This separation enables the same program logic to operate in both simulation and production environments with different handler implementations. Effects and handlers are registered at runtime through registry APIs, allowing the system to dynamically dispatch to appropriate handlers based on effect types and execution context.

## OCaml DSL

The OCaml DSL provides a functional approach to constructing Lisp expressions with type safety and pattern matching. The implementation includes `lisp_ast.ml` for abstract syntax tree definitions and S-expression serialization, and `dsl.ml` for builder functions that construct expressions such as `add`, `if_`, and `lambda`. This approach leverages OCaml's type system for well-formed expressions and uses S-expression interop as a canonical format for data exchange between Rust and OCaml components.

Example OCaml DSL usage:
```ocaml
let expr = if_ (gt (sym "x") (int_lit 10)) 
              (str_lit "large") 
              (str_lit "small")
```

Both OCaml and Rust DSLs produce the same canonical `Expr` AST, which is SSZ-serialized for content addressing and verifiable storage.

## Crates

- `causality-core`: Core data structures, traits, and type definitions for Resources, expressions, and IDs
- `causality-lisp`: Lisp interpreter for evaluating Resource logic and expressions
- `causality-runtime`: Executes Resource interactions and manages the evaluation context
- `causality-simulation`: Simulation engine with schema-aware mocking for testing
- `causality-zk`: Zero-Knowledge proof generation and verification using execution traces
- `causality-api`: Traits for external system integration (ZK coprocessors, blockchain connectors)
- `causality-compiler`: *(Future)* Compiles Resource definitions and validates cross-domain logic
- `causality-toolkit`: Standard library of reusable Resources, effects, and Lisp utilities for Rust development

## Environment & Build

This project uses Nix with Flakes for reproducible development. Enter the development environment with `nix develop`, then build all crates using `cargo build --all` and run tests with `cargo test --all`.
