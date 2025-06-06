# ML Causality Codebase Reorganization Proposal

## Overview

This proposal reorganizes the `ml_causality` codebase to improve cohesion, separation of concerns, and maintainability while following idiomatic OCaml practices. The goal is to reduce complexity, eliminate redundancy, and create a more intuitive structure.

## Current Issues

### 1. Overly Large Modules
- `effect_system.ml` (934 lines) - combines multiple responsibilities
- `dsl.ml` (615 lines) - mixes AST, builders, and utilities
- `sexpr.ml` (459 lines) - handles all serialization concerns

### 2. Poor Separation of Concerns
- `dsl/` mixes AST definitions, DSL builders, and domain-specific primitives
- `ssz_bridge/` combines serialization, FFI, and content addressing
- `types/` mixes core types with serialization logic

### 3. Inconsistent Organization
- 8 lib subdirectories create unnecessary fragmentation
- Implementation-focused naming (`ssz_bridge`) rather than domain-focused
- Mixed abstraction levels in same modules

## Proposed New Structure

```
ml_causality/
├── lib/
│   ├── core/           # Core types and fundamental abstractions
│   ├── lang/           # Language constructs (AST, DSL, expressions)
│   ├── effects/        # Effect system components and coordination
│   ├── serialization/  # All serialization and content addressing
│   └── interop/        # FFI, bridges, and external integrations
├── test/               # All tests (reorganized by domain)
├── bin/                # Executables and test runners
├── examples/           # Usage examples and demos
└── docs/               # Additional documentation
```

## Detailed Module Structure

### 1. `lib/core/` - Core Types and Abstractions
*Purpose: Fundamental types, identifiers, and base abstractions*

```ocaml
core/
├── types.ml           # Core Causality types (Intent, Effect, Resource, etc.)
├── types.mli          # Public interface for core types
├── identifiers.ml     # Entity IDs, content addressing primitives
├── domains.ml         # Domain types and domain logic
├── patterns.ml        # Resource patterns and matching
└── dune
```

**Responsibilities:**
- Core Causality types: `Intent`, `Effect`, `Resource`, `Handler`, `Transaction`
- Domain types: `TypedDomain`, `DomainCompatibility`
- ID types and basic content addressing
- Resource patterns and flow specifications
- **Size estimate**: ~300-400 lines per file (vs current 428 lines in types.ml)

### 2. `lib/lang/` - Language Constructs
*Purpose: Expression system, AST, and DSL builders*

```ocaml
lang/
├── ast.ml             # Core AST types (Expr, ValueExpr, Atom)
├── ast.mli            # AST public interface
├── builders.ml        # DSL builder functions (let_, if_, apply, etc.)
├── builders.mli       # DSL public interface  
├── combinators.ml     # Atomic combinators and function primitives
├── primitives.ml      # Domain-specific primitives (tokens, bridges)
├── validation.ml      # Expression validation and type checking
└── dune
```

**Responsibilities:**
- AST definitions (`Expr`, `ValueExpr`, `AtomicCombinator`)
- DSL builder functions (`let_`, `if_`, `apply`, etc.)
- Combinator implementations
- Domain-specific primitives (currently in `token_primitives.ml`, `bridge_primitives.ml`)
- Expression validation
- **Size estimate**: ~200-300 lines per file (vs current 615 lines in dsl.ml)

### 3. `lib/effects/` - Effect System and Execution
*Purpose: Effect system, handlers, execution engine*

```ocaml
effects/
├── effects.ml         # Effect type registration and management
├── effects.mli        # Effect system public interface
├── handlers.ml        # Handler registration and linking
├── execution.ml       # Effect execution and continuation handling
├── registry.ml        # Effect/handler registry and lookup
├── graph.ml           # TEL graph construction and analysis
└── dune
```

**Responsibilities:**
- Effect type registration and configuration
- Handler registration and effect-handler linking
- Effect execution engine
- Continuation handling and validation
- TEL graph construction
- **Size estimate**: ~150-200 lines per file (vs current 934 lines in effect_system.ml)

### 4. `lib/serialization/` - Serialization and Content Addressing
*Purpose: All serialization, content addressing, and data persistence*

```ocaml
serialization/
├── sexpr.ml           # S-expression serialization
├── sexpr.mli          # S-expression public interface
├── ssz.ml             # SSZ serialization implementation (imports ocaml_ssz)
├── content.ml         # Content addressing and hashing
├── merkle.ml          # Sparse Merkle Tree implementation
└── dune
```

**Responsibilities:**
- S-expression serialization (currently in `types/sexpr.ml`)
- SSZ serialization (currently scattered across `ssz_bridge/`) - imports separate `ocaml_ssz` module
- Content addressing and hashing (currently in `content_addressing/`)
- SMT implementation (currently in `smt/`)
- **Size estimate**: ~150-250 lines per file

### 5. `lib/interop/` - External Integrations
*Purpose: FFI, bridges, and external system integration*

```ocaml
interop/
├── ffi.ml             # Rust FFI bindings and C stubs
├── ffi.mli            # FFI public interface
├── bridges.ml         # Cross-domain bridge workflows
├── capabilities.ml    # Capability system and authorization
└── dune
```

**Responsibilities:**
- Rust FFI bindings (currently in `ssz_bridge/rust_ffi.ml`)
- Bridge workflows (currently in `dsl/bridge_workflow.ml`)
- Capability system (currently in `capability_system/`)
- PPX registry (currently in `ppx_registry/`)
- **Size estimate**: ~200-300 lines per file

## Reorganized Test Structure

```
test/
├── unit/              # Unit tests by domain
│   ├── test_core.ml   # Core types tests
│   ├── test_lang.ml   # Language/DSL tests
│   ├── test_effects.ml # Effect system tests
│   ├── test_serialization.ml # Serialization tests
│   └── test_interop.ml # Integration tests
├── integration/       # End-to-end integration tests
│   ├── test_e2e_workflow.ml
│   └── test_cross_language.ml
└── property/          # Property-based tests (if any)
```

## Migration Strategy

### Phase 1: Create New Structure
1. Create new directory structure
2. Set up dune files for each module
3. Create empty .ml/.mli files with proper module documentation

### Phase 2: Extract and Reorganize Core Types
1. Split `types/types.ml` into domain-focused modules in `core/`
2. Move ID and content addressing logic to `core/identifiers.ml`
3. Extract domain logic to `core/domains.ml`

### Phase 3: Reorganize Language Components
1. Extract AST from `dsl/dsl.ml` to `lang/ast.ml`
2. Move DSL builders to `lang/builders.ml`
3. Consolidate primitives from multiple files to `lang/primitives.ml`

### Phase 4: Break Down Effect System
1. Split `effect_system.ml` into focused modules in `effects/`
2. Extract registry logic to `effects/registry.ml`
3. Separate execution engine to `effects/execution.ml`

### Phase 5: Consolidate Serialization
1. Move all serialization to `serialization/`
2. Consolidate SSZ logic from `ssz_bridge/` (importing separate `ocaml_ssz` module)
3. Move SMT implementation from `smt/`

### Phase 6: Organize External Integrations
1. Move FFI to `interop/ffi.ml`
2. Consolidate bridge workflows
3. Move capability system

### Phase 7: Update Dependencies and Tests
1. Update all import statements
2. Reorganize tests by domain
3. Update dune configuration
4. Verify all builds and tests pass

## Benefits of This Reorganization

### 1. Improved Cohesion
- Each module has a single, clear responsibility
- Related functionality is grouped together
- Clear boundaries between domains

### 2. Better Separation of Concerns
- Core types separated from serialization logic
- Language constructs separated from runtime execution
- External integrations isolated from core logic

### 3. Enhanced Maintainability
- Smaller, focused modules (~150-300 lines each)
- Clear dependency relationships
- Easier to understand and modify

### 4. Idiomatic OCaml Structure
- Follows OCaml best practices for module organization
- Clear public interfaces (.mli files)
- Logical grouping of related functionality

### 5. Improved Discoverability
- Domain-focused naming makes it easy to find relevant code
- Logical structure matches mental model of the system
- Clear entry points for different use cases

## Implementation Notes

### Module Naming Conventions
- Use clear, domain-focused names
- Avoid implementation details in module names
- Keep names concise but descriptive

### Interface Design
- Every module should have a corresponding .mli file
- Expose only necessary types and functions
- Document all public interfaces

### Dependency Management
- Minimize cross-module dependencies
- Use dependency injection where appropriate
- Keep core modules dependency-free

## Expected Outcomes

1. **Reduced Complexity**: Smaller, focused modules are easier to understand
2. **Improved Navigation**: Clear structure makes it easy to find relevant code
3. **Better Testing**: Domain-focused tests are more comprehensive
4. **Enhanced Extensibility**: Clean boundaries make it easier to add features
5. **Simplified Onboarding**: New developers can understand the system faster

This reorganization maintains all existing functionality while providing a much cleaner, more maintainable codebase structure. 