<!-- Architecture of effect adapters -->
<!-- Original file: docs/src/effect_adapters_architecture.md -->

# Effect Adapters Architecture

## Overview

The Effect Adapters module is a core part of the Causality codebase that provides a unified interface for code generation, content addressing, and execution. This module replaces the previous `src/code` functionality while offering an expanded and more consistent API.

## Key Components

### Hash (`src/effect_adapters/hash.rs`)

This module provides content-addressing functionality through hash implementations. It supports different hashing algorithms, with Blake3 as the primary implementation. Content hashes are used throughout the system to uniquely identify and verify code and other content.

### Repository (`src/effect_adapters/repository.rs`)

The repository component manages storage and retrieval of content-addressed code. It provides APIs for:
- Storing code with metadata
- Retrieving code by hash
- Versioning and tracking code dependencies

### Definition (`src/effect_adapters/definition.rs`)

Defines the fundamental structures for representing code definitions, including:
- Content format (Rust, JavaScript, RISC-V)
- Metadata and dependencies
- Resource requirements

### Name Registry (`src/effect_adapters/name_registry.rs`)

Maps human-readable names to content hashes, supporting:
- Multiple versions of named content
- Latest version lookup
- Name registration and validation

### Compatibility (`src/effect_adapters/compatibility.rs`)

Provides tools for checking compatibility between:
- Code versions
- Runtime environments
- API contracts

### Executor (`src/effect_adapters/executor.rs`)

Handles the execution of content-addressed code:
- Security sandboxing
- Resource allocation and tracking
- Execution context management

### RISC-V Metadata (`src/effect_adapters/riscv_metadata.rs`)

Specialized support for RISC-V execution, including:
- Memory layout
- Instruction set compatibility
- Register allocation

### ZK Module (`src/effect_adapters/zk/`)

Support for zero-knowledge proof generation and verification:
- Circuit compilation
- Witness generation
- Proof verification

## Integration Points

The Effect Adapters module integrates with:

1. The Effect System (`src/effect.rs`) - For handling algebraic effects
2. The Execution Module (`src/execution/`) - For code execution and tracing
3. The Resource System (`src/resource.rs`) - For resource allocation and tracking

## Usage Examples

### Content Addressing

```rust
use crate::effect_adapters::hash::Hash;

// Create a hash from a string
let hash = Hash::from_str("blake3:0123456789abcdef0123456789abcdef").unwrap();

// Get the string representation
let hash_str = hash.to_string();
```

### Code Repository

```rust
use crate::effect_adapters::repository::CodeRepository;
use crate::effect_adapters::definition::CodeDefinition;

// Store a code definition
let hash = repository.store(code_definition)?;

// Retrieve by hash
let definition = repository.get_by_hash(&hash)?;
```

### Execution

```rust
use crate::effect_adapters::executor::ContentAddressableExecutor;

// Execute code by name
let result = executor.execute_by_name(
    "example_function",
    vec![Value::String("input".to_string())],
    &mut context,
)?;
```

## Migration History

This module was created by consolidating and expanding functionality previously found in `src/code/`. The migration was completed in March 2024, with all components successfully transferred and the original `src/code/` directory removed. 