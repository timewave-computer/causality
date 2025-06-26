# Causality FFI

Foreign Function Interface providing OCaml bindings for Causality core types and operations with type-safe marshalling across language boundaries.

## Purpose

Enables seamless integration between Rust and OCaml codebases while maintaining type safety and linear resource semantics for the Causality framework.

## Core Components

- **Value Marshalling** (`value.rs`): Safe Rust-OCaml type conversion
- **Error Handling** (`error.rs`): Cross-language error propagation  
- **Serialization** (`serialization.rs`): SSZ-compatible data exchange
- **OCaml Bindings** (`ocaml/`): Core types, error handling, and Layer 1 integration

## Key Features

- Type-safe cross-language calls with compile-time guarantees
- Linear resource lifecycle preservation across FFI boundary
- Comprehensive error handling with source location tracking
- Zero-copy data transfer where possible

## Integration

Primary consumer is the `ocaml_causality` project which provides OCaml interfaces to Causality's three-layer architecture. 