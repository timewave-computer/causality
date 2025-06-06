# OCaml Causality Framework

A comprehensive OCaml implementation and integration layer for the Causality framework, providing high-level APIs for working with linear resources, expressions, intents, and effects.

## Architecture Overview

The OCaml Causality framework is organized into focused, cohesive modules with clear separation of concerns:

### Core Layer (`lib/core/`)
- **`ocaml_causality_core.ml`**: Unified core types and definitions
  - Basic types and identifiers (resource_id, expr_id, etc.)
  - Result types and error handling (causality_error)
  - LispValue types for FFI integration
  - Domain types (VerifiableDomain, ServiceDomain, ComputeDomain)
  - Resource types (resource, resource_flow, nullifier)
  - Core Causality types (intent, effect, handler, transaction)

### Language Layer (`lib/lang/`)
- **`value.ml`**: Value expressions and LispValue operations
- **`expr.ml`**: Expression AST and operations
- **`ast.ml`**: Core AST types and combinators
- **`builders.ml`**: Builder functions for expressions
- **`combinators.ml`**: Combinator definitions
- **`primitives.ml`**: Primitive operations
- **`validation.ml`**: Expression validation

### Effects Layer (`lib/effects/`)
- **`effects.ml`**: Intent, Effect, and System modules
  - Intent creation, submission, and management
  - Effect monitoring and queries
  - System-level operations and metrics

### Interop Layer (`lib/interop/`)
- **`bindings.ml`**: High-level OCaml API with organized modules:
  - `LispValue`: Constructors, predicates, extractors, list operations
  - `Expr`: Expression AST, compilation, predefined expressions
  - `Intent`: Builder pattern for intent creation and submission
  - `System`: Resource and domain management
- **`ffi.ml`**: Foreign Function Interface layer (placeholder implementations)
- **`external_apis.ml`**: External API integrations
- **`type_conversion.ml`**: Type conversion utilities

### Serialization Layer (`lib/serialization/`)
- **`ssz_compat.ml`**: SSZ serialization compatibility

### System Layer (`lib/system/`)
- **`coordination.ml`**: System coordination
- **`domain_management.ml`**: Domain management
- **`resource_management.ml`**: Resource lifecycle management

## Key Design Principles

### 1. **High Coherence**
Each module has a single, well-defined responsibility:
- **Core**: Type definitions and fundamental abstractions
- **Lang**: Language constructs and expression handling  
- **Effects**: Effect system and intent management
- **Interop**: External integrations and FFI bindings

### 2. **Clear Separation of Concerns**
- **Type definitions** are centralized in the core module
- **Business logic** is separated by domain (language, effects, interop)
- **FFI concerns** are isolated in the interop layer
- **External integrations** have dedicated modules

### 3. **Layered Architecture**
```
┌─────────────────────────────────────┐
│            Applications             │
├─────────────────────────────────────┤
│         Interop (FFI/APIs)          │
├─────────────────────────────────────┤
│    Effects (Intents/System)         │
├─────────────────────────────────────┤
│      Lang (Expressions/AST)         │
├─────────────────────────────────────┤
│       Core (Types/Errors)           │
└─────────────────────────────────────┘
```

### 4. **Modular Organization**
- Each layer depends only on lower layers
- Clear module boundaries with well-defined interfaces
- Re-export patterns for convenient access
- Wrapped libraries for namespace control

## Usage Example

```ocaml
open Ocaml_causality_core
open Ocaml_causality_interop.Bindings

(* Create LispValues *)
let values = [
  LispValue.unit;
  LispValue.bool true;
  LispValue.int 42L;
  LispValue.string "Hello";
  LispValue.list [LispValue.int 1L; LispValue.int 2L];
]

(* Build expressions *)
let expr = Expr.apply 
  (Expr.const (LispValue.symbol "my-function"))
  [Expr.const_string "arg1"; Expr.const_int 42L]

(* Create and submit intents *)
let intent = Intent.create ~name:"MyIntent" ~domain_id:"MyDomain"
let _ = Intent.add_parameter intent (LispValue.string "param")
let _ = Intent.submit intent
```

## Module Dependencies

- **Core**: No dependencies (foundation layer)
- **Lang**: Depends on Core
- **Effects**: Depends on Core 
- **Interop**: Depends on Core
- **Applications**: Can depend on any layer

## Build System

Uses Dune with:
- **Library wrapping** for clean namespaces
- **Explicit module lists** for dependency control
- **Unix/Base64** dependencies for system integration
- **Modular compilation** for fast incremental builds

## Future Extensions

The modular structure easily supports:
- **Additional language frontends** (new lang modules)
- **Different effect systems** (new effects modules)  
- **Multiple FFI backends** (new interop modules)
- **Domain-specific APIs** (new application layers)

## Testing

- **Unit tests** per module in `test/` directory
- **Integration tests** across module boundaries
- **Property-based testing** for core abstractions
- **FFI mocking** for isolated testing
