# Causality Compiler

Compilation and transformation tools for the Causality Linear Resource Language. This crate is responsible for parsing Causality Lisp code, type checking with linear resource semantics and row types, and compiling to the Register Machine Intermediate Representation (IR).

## Overview

The `causality-compiler` crate provides the core compilation pipeline for the new Causality Linear Resource Language, including:

- **Lisp Parsing**: Converting S-expression syntax into a structured Abstract Syntax Tree (AST).
- **Type Checking**: Static analysis to ensure type correctness, linear resource safety, and compliance with row type constraints.
- **Machine Representation Generation**: Compiling the typed AST into Register Machine Langauge.
- **Macro Expansion**: Compile-time transformation of Lisp expressions.
- **ZK Circuit Generation**: (Future) Generating zero-knowledge circuits from the Register Machine IR.

The compiler ensures that programs adhere to the principles of the linear resource model and are suitable for deterministic execution on the Register Machine.

## Core Components

### Parser

Parses Causality Lisp source code into an Abstract Syntax Tree (AST).

```rust
use causality_compiler::parser::parse_program_str;

let source_code = "(defprogram my-program ...)";
let ast = parse_program_str(source_code)?;
// AST represents the parsed Lisp code
```

### Type Checker

Performs static analysis, including type inference, type checking, and verification of linear resource constraints and row type operations.

```rust
use causality_compiler::type_checker::{TypeChecker, TypeCheckResult};
use causality_types::expr::effect::Expr; // Assuming new AST type

let type_checker = TypeChecker::new();
let typed_ast = type_checker.check_program(&ast)?;
// typed_ast contains type annotations and verified linear resource flows
```

### Machine Language Generator

Compiles the typed AST into the Register Machine Intermediate Representation (IR).

```rust
use causality_compiler::machine_generator::{IrGenerator, RegisterMachineIR};
use causality_types::expr::effect::Expr; // Assuming typed AST type

let machine_generator = IrGenerator::new();
let register_machine = machine_generator.generate_machine(&typed_ast)?;
// register_machine is a sequence of Register Machine instructions
```

### Macro Expansion

Handles compile-time macro expansion as a transformation of expression resources.

```rust
use causality_compiler::macro_expansion::MacroExpander;
use causality_types::expr::effect::Expr; // Assuming AST type

let expander = MacroExpander::new();
let expanded_ast = expander.expand(&initial_ast)?;
// expanded_ast has macros replaced with their definitions
```

### ZK Compiler (Future)

Responsible for generating zero-knowledge circuits from the Register Machine IR.

```rust
// This component is planned for a future phase.
// It will take Register Machine and produce ZK circuits
// suitable for proof generation.
// use causality_compiler::zk::ZkCompiler;
// let zk_circuit = ZkCompiler::compile_to_zk(&register_machine)?;
```

## Compilation Pipeline

The compilation process is structured as a pipeline:

1.  **Parsing**: Source code -> AST.
2.  **Macro Expansion**: AST -> Expanded AST.
3.  **Type Checking**: Expanded AST -> Typed AST (with linear resource flow analysis and row type resolution).
4.  **Machine Representation Generation**: Typed AST -> Register Machine IR.
5.  **Optimization**: Register Machine Langauge -> Optimized Register Machine Langauge (various passes).
6.  **ZK Compilation (Future)**: Optimized Machine Langauge -> ZK Circuits.

## Content-Addressed Artifacts

Compilation outputs (like the Register Machine IR) will be content-addressed to ensure deterministic builds and enable caching.

## Configuration

(Configuration details will be added as the implementation progresses)

```toml
# Example placeholder structure
[compiler]
# settings for the parser, type checker, Machine language generation, etc.

[compiler.optimization]
# settings for optimization passes

[compiler.zk]
# settings for ZK compilation
```

## Error Handling

Comprehensive error reporting for parsing, type checking, and compilation errors.

## Feature Flags

(Feature flags will be defined as specific components are implemented)
