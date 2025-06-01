# Deployment and Infrastructure

The Causality framework utilizes a development-focused infrastructure centered on Nix for reproducible builds and Cargo for build orchestration. This approach ensures consistent development environments and efficient workflows, prioritizing developer productivity and build reproducibility while supporting the three-layer architecture that defines the system.

## Development Environment Architecture

The project leverages Nix flakes to create reproducible development environments for both Rust and OCaml. This eliminates dependency conflicts and ensures all developers work with identical toolsets, including the Rust toolchain for the register machine and core types, the OCaml ecosystem for `ml_causality`, and various utility tools for testing and optimization.

Key Nix flake inputs illustrate this:
```nix
{
  description = "Causality Linear Resource Framework";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
}
```
Nix provides immediate access to all versioned tools and dependencies, including specialized tools for ZK circuit generation and register machine simulation. Integration with `direnv` enables automatic environment activation upon entering the project directory, streamlining the development workflow.

## Build System Organization

The framework employs a Cargo workspace to manage its Rust components, organized according to the three-layer architecture. This structure promotes efficient incremental builds and a clear separation of concerns.

A typical workspace structure might include:
```toml
[workspace]
members = [
    # Foundational Types (All Layers)
    "crates/causality-types",            # Defines types for Layer 0, Layer 1, and Layer 2 (internally structured)

    # Layer 0: Core Computational Substrate
    "crates/causality-vm",               # Layer 0 Typed Register Machine implementation (executes 9 instructions)

    # Layer 1: Causality Lisp & Compilation
    "crates/causality-lisp-parser",      # Parser for Layer 1 Causality Lisp S-expressions
    "crates/causality-lisp-ast",         # Defines Layer 1 Expr (AST) for the 11 core Lisp primitives
    "crates/causality-lisp-compiler",    # Compiles Layer 1 Lisp AST to Layer 0 VM instructions

    # Layer 2: Effect System & Orchestration
    "crates/causality-effects-engine",   # Manages Layer 2 Effects, Intents, Handlers, TEG construction

    # Runtime & Execution
    "crates/causality-runtime",          # Orchestrates execution across layers, manages state

    # Specialized Components
    "crates/causality-zk",               # ZK proof generation and verification utilities (SSZ-based)
    "crates/causality-simulation",       # Multi-layer simulation engine

    # Developer Support
    "crates/causality-toolkit",          # CLI tools, DSL helpers, and other developer utilities
    # "examples/"                        # Example applications and usage scenarios (optional)
]
```
This organization centralizes all type definitions in `causality-types` (which internally separates Layer 0, 1, and 2 types), provides a dedicated Virtual Machine (`causality-vm`) for the 9-instruction Layer 0, distinct components for Layer 1 Lisp processing (parser, AST for the 11 primitives, compiler), and a Layer 2 effects engine. This setup facilitates shared dependency management via SSZ for serialization and leverages Cargo's incremental compilation for optimized build times, crucial for the Layer 1 Lisp to Layer 0 instruction compilation pipeline.

## Testing and Quality Assurance

Comprehensive testing is integral to the framework, with dedicated test suites for each architectural layer:

- **Layer 0 Testing**: Verification of the 9-instruction register machine, memory model validation, and conservation law checking.
- **Layer 1 Testing**: Row type operations (as defined in `causality-types`), linearity enforcement, type inference for Causality Lisp, and compilation of the 11 Lisp primitives to Layer 0 instructions.
- **Layer 2 Testing**: Effect handler composition, TEG construction, and intent resolution, ensuring correct interaction with Layer 1 expressions and Layer 0 resources.

Testing infrastructure includes property-based testing for the type system, simulation-based testing for the register machine, and integration tests that verify the complete pipeline from Lisp source to register machine execution.

## OCaml Integration Infrastructure

The `ml_causality` OCaml project uses the `dune` build system and provides alternative implementations of key components:
- Row type inference engine
- Effect handler optimization
- Property-based testing with QCheck

This is integrated into the overarching Nix-based development environment, ensuring that OCaml components are built and tested consistently. Build coordination between Rust and OCaml components maintains interoperability, particularly for shared interfaces like row type representations and effect definitions.

## Development Performance Optimization

Build and test execution performance is optimized for the three-layer architecture:

- **Static Analysis Cache**: Row type operations and linearity checks are cached between builds
- **Register Machine Optimization**: Instruction sequences are optimized during compilation
- **Parallel Testing**: Independent test suites for each layer run in parallel

The infrastructure leverages Cargo's caching mechanisms and parallel compilation capabilities while the Nix environment provides consistently versioned and optimized tooling. Special attention is given to the compilation pipeline performance, as transforming Lisp expressions through the three layers to register machine langauge is a critical path for development iteration speed.
