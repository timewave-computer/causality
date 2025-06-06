# S-expression Serialization Implementation

This directory contains documentation and implementation details for the hybrid serialization strategy in the Causality project, as described in [serialization.md](serialization.md).

## Implementation Overview

The serialization strategy has been implemented with two main components:

1. **Rust Implementation**:
   - `causality-core/src/sexpr_utils.rs`: Core utilities for S-expression serialization and content addressing
   - `causality-types/src/expr/sexpr.rs`: S-expression serialization for Expression types
   - `causality-core/src/sexpr_ffi.rs`: FFI functions for converting between S-expressions and ssz

2. **OCaml Implementation**:
   - `ml_causality/lib/types/sexpr.ml`: S-expression serialization for OCaml types
   - `ml_causality/lib/types/rust_sexpr_ffi.ml`: OCaml bindings to the Rust FFI functions
   - `ml_causality/bin/test_sexpr.ml`: Test script for S-expression serialization

## Current Status

The implementation now provides:

- Full serialization/deserialization of core types to S-expressions in Rust and OCaml
- Content-addressing capability for all serializable types
- S-expression format compatibility between Rust and OCaml
- FFI function skeletons for ssz serialization (for ZK circuits)

Test scripts in both Rust and OCaml confirm the functionality and compatibility.

## Usage Examples

### Rust Side

```rust
use causality_core::sexpr_utils::{SexprSerializable, SexprContentAddressable};

// Implement SexprSerializable for your type
impl SexprSerializable for MyType {
    fn to_sexpr(&self) -> SexprValue {
        // Convert to S-expression
    }
    
    fn from_sexpr(value: &SexprValue) -> Result<Self> {
        // Parse from S-expression
    }
}

// Automatically implement content addressing
impl SexprContentAddressable for MyType {}

// Usage
let my_object = MyType::new();
let sexpr = my_object.to_sexpr();
let sexpr_string = my_object.to_canonical_sexpr_string();
let content_hash = my_object.sexpr_content_hash_hex();
```

### OCaml Side

```ocaml
open Ml_causality.Lib.Types

(* Convert an expression to S-expression string *)
let expr = ELambda (["x"; "y"], Apply (Combinator Add, [Var "x"; Var "y"]))
let sexpr_str = Sexpr.expr_to_string expr

(* Content addressing *)
let hash = Sexpr.sexpr_content_hash_hex Sexpr.expr_to_sexp expr

(* Round-trip conversion *)
let expr_roundtrip = Sexpr.expr_from_string sexpr_str
```

## FFI Integration

For ZK circuit integration that requires ssz serialization, use the FFI functions:

```ocaml
open Ml_causality.Lib.Types

(* Convert TEL graph to ssz for ZK circuits *)
let graph = create_tel_graph ()
let ssz_bytes = Rust_sexpr_ffi.tel_graph_to_ssz graph

(* Convert back from ssz *)
match Rust_sexpr_ffi.ssz_to_tel_graph ssz_bytes with
| Ok graph -> (* use graph *)
| Error msg -> (* handle error *)
```

## Testing

Tests are included in both the Rust and OCaml implementations to verify:

1. Correct serialization and deserialization
2. Compatibility between Rust and OCaml formats 
3. Content addressing consistency

Run the Rust tests with:

```
cargo test -p causality-core --lib -- sexpr_utils::tests
```

## Implementation Notes

- The serialization format is designed to be human-readable, making debugging easier
- Content addressing works identically in both Rust and OCaml
- The FFI layer allows ssz serialization to be used where required by ZK circuits
- The implementation prioritizes canonical formats to ensure consistent hashing 