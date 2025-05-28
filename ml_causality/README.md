# ML Causality

OCaml DSL and toolkit for working with the Causality Resource Model framework. This project provides a comprehensive OCaml implementation of the Causality Lisp ecosystem, including type definitions, DSL construction, effect systems, and integration bridges.

## Overview

The `ml_causality` project serves as the OCaml counterpart to the Rust-based Causality framework, providing:

- **Type System**: Complete OCaml type definitions corresponding to `causality_types` crate
- **Lisp DSL**: Domain-specific language for constructing Temporal Effect Language (TEL) expressions
- **Effect System**: OCaml implementation of the Causality effect system
- **SSZ Bridge**: Integration with Simple Serialize format for content addressing
- **Capability System**: Authorization and permission management
- **Content Addressing**: Utilities for content-addressed identifiers

All components maintain compatibility with the Resource Model's content-addressed, SSZ-serialized architecture.

## Project Structure

```
ml_causality/
├── lib/                         # Core library modules
│   ├── types/                   # Type definitions
│   ├── dsl/                     # Lisp DSL for TEL expressions
│   ├── effect_system/           # Effect system implementation
│   ├── content_addressing/      # Content addressing utilities
│   ├── ssz_bridge/              # SSZ serialization bridge
│   ├── smt/                     # Sparse Merkle Tree implementation
│   ├── capability_system/       # Capability and authorization system
│   └── ppx_registry/            # PPX preprocessor registry
├── bin/                         # Executable tests and examples
├── test/                        # Test suite
├── ppx/                         # PPX preprocessors
└── generated_lisp/              # Generated Lisp code
```

## Core Components

### Type System

Complete OCaml type definitions corresponding to the Rust `causality_types` crate:

```ocaml
open Ml_causality_lib_types.Types

(* Core Resource Model types *)
type resource = {
  id: entity_id;
  name: str_t;
  domain_id: domain_id;
  resource_type: str_t;
  quantity: int64;
  timestamp: timestamp;
}

type intent = {
  id: entity_id;
  name: str_t;
  domain_id: domain_id;
  priority: int;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option;
  timestamp: timestamp;
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(* Expression AST types *)
type expr = 
  | EAtom of atom
  | EConst of value_expr 
  | EVar of str_t 
  | ELambda of str_t list * expr 
  | EApply of expr * expr list 
  | ECombinator of atomic_combinator
  | EDynamic of int * expr

type value_expr =  
  | VNil 
  | VBool of bool 
  | VString of str_t 
  | VInt of int64 
  | VList of value_expr list 
  | VMap of (str_t, value_expr) BatMap.t 
  | VStruct of (str_t, value_expr) BatMap.t 
  | VRef of value_expr_ref_target 
  | VLambda of {
      params: str_t list;
      body_expr_id: expr_id;
      captured_env: (str_t, value_expr) BatMap.t;
    }
```

### Lisp DSL

Domain-specific language for constructing TEL expressions:

```ocaml
open Ml_causality_lib_dsl.Dsl

(* Basic expression construction *)
let my_expr = 
  let_star [
    ("balance", get_field (sym "resource") (str_lit "balance"));
    ("amount", int_lit 100L);
  ] [
    if_ (gte (sym "balance") (sym "amount"))
        (bool_lit true)
        (bool_lit false)
  ]

(* Function definition *)
let transfer_logic = 
  defun "transfer-tokens" ["from"; "to"; "amount"] (
    and_ [
      gte (get_field (sym "from") (str_lit "balance")) (sym "amount");
      gt (sym "amount") (int_lit 0L);
    ]
  )

(* Map operations *)
let token_state = 
  make_map [
    (str_lit "balance", int_lit 1000L);
    (str_lit "owner", str_lit "alice");
    (str_lit "frozen", bool_lit false);
  ]
```

### Effect System

OCaml implementation of the Causality effect system:

```ocaml
open Ml_causality_lib_effect_system.Effect_system

(* Define custom effects *)
type token_effect = 
  | Transfer of { from: entity_id; to: entity_id; amount: int64 }
  | Mint of { to: entity_id; amount: int64 }
  | Burn of { from: entity_id; amount: int64 }

(* Effect handlers *)
let handle_token_effect = function
  | Transfer { from; to; amount } ->
      (* Implementation for token transfer *)
      validate_transfer from to amount
  | Mint { to; amount } ->
      (* Implementation for token minting *)
      validate_mint to amount
  | Burn { from; amount } ->
      (* Implementation for token burning *)
      validate_burn from amount
```

### ProcessDataflowBlock Support

Support for complex dataflow orchestrations:

```ocaml
(* ProcessDataflowBlock definition *)
type process_dataflow_definition = {
  definition_id: expr_id;
  name: str_t;
  input_schema: (str_t, str_t) BatMap.t;
  output_schema: (str_t, str_t) BatMap.t;
  state_schema: (str_t, str_t) BatMap.t;
  nodes: pdb_node list;
  edges: pdb_edge list;
  default_typed_domain: typed_domain;
}

(* Typed domain support *)
type typed_domain =
  | VerifiableDomain of {
      domain_id: domain_id;
      zk_constraints: bool;
      deterministic_only: bool;
    }
  | ServiceDomain of {
      domain_id: domain_id;
      external_apis: str_t list;
      non_deterministic_allowed: bool;
    }
  | ComputeDomain of {
      domain_id: domain_id;
      compute_intensive: bool;
      parallel_execution: bool;
    }
```

### SSZ Bridge

Integration with Simple Serialize format:

```ocaml
open Ml_causality_lib_ssz_bridge.Ssz_bridge

(* Serialize expressions to SSZ format *)
let serialize_expr expr =
  expr_to_ssz expr

(* Deserialize from SSZ format *)
let deserialize_expr ssz_bytes =
  ssz_to_expr ssz_bytes

(* Content addressing *)
let content_id = compute_content_id expr
```

### Capability System

Authorization and permission management:

```ocaml
open Ml_causality_lib_capability_system.Capability_system

(* Define capabilities *)
type capability = {
  name: string;
  resource_type: string option;
  domain_scope: domain_id option;
  constraints: expr_id list;
}

(* Check capabilities *)
let check_capability user_caps required_cap context =
  validate_capability_access user_caps required_cap context
```

## Usage Examples

### Basic Expression Construction

```ocaml
open Ml_causality_lib_dsl.Dsl

(* Create a simple validation expression *)
let balance_check = 
  gte (get_field (sym "*self-resource*") (str_lit "balance")) (int_lit 0L)

(* Create a transfer validation *)
let transfer_validation from_resource to_resource amount =
  and_ [
    gte (get_field from_resource (str_lit "balance")) amount;
    gt amount (int_lit 0L);
    eq (get_field from_resource (str_lit "owner")) (get_context_value (str_lit "caller"));
  ]
```

### Resource Definition

```ocaml
(* Define a token resource type *)
let token_resource_type = {
  id = generate_entity_id ();
  name = "TokenResource";
  domain_id = verifiable_domain_id;
  resource_type = "token";
  quantity = 1000L;
  timestamp = current_timestamp ();
}

(* Define validation logic *)
let token_validation_expr = 
  and_ [
    gte (get_field (sym "*self-resource*") (str_lit "balance")) (int_lit 0L);
    not_ (get_field (sym "*self-resource*") (str_lit "frozen"));
  ]
```

### Cross-Domain Operations

```ocaml
(* Define cross-domain transfer *)
let cross_domain_transfer source_domain target_domain resource_id amount =
  let transfer_intent = {
    id = generate_entity_id ();
    name = "CrossDomainTransfer";
    domain_id = source_domain;
    priority = 1;
    inputs = [{ resource_type = "token"; quantity = amount; domain_id = source_domain }];
    outputs = [{ resource_type = "token"; quantity = amount; domain_id = target_domain }];
    expression = Some (serialize_expr transfer_validation_expr);
    timestamp = current_timestamp ();
    hint = None;  (* Optional optimization hints *)
  } in
  submit_intent transfer_intent
```

## Building and Testing

### Prerequisites

- OCaml 5.0.0 or later
- Dune 3.8 or later
- Required dependencies: `base`, `zarith`, `sexplib0`, `digestif`, `ml_ssz`

### Build

```bash
# Build the entire project
dune build

# Build specific components
dune build lib/
dune build bin/

# Build with tests
dune build @runtest
```

### Testing

```bash
# Run all tests
dune runtest

# Run specific test executables
dune exec bin/test_sexpr_simple.exe
dune exec bin/test_ssz_bridge.exe
dune exec bin/test_sexpr_basic.exe
```

### Development

```bash
# Watch mode for development
dune build --watch

# Generate documentation
dune build @doc

# Format code
dune fmt
```

## Integration with Rust

The OCaml implementation is designed to work seamlessly with the Rust Causality framework:

- **Type Compatibility**: All types correspond directly to Rust types
- **SSZ Serialization**: Shared serialization format for interoperability
- **Content Addressing**: Compatible content-addressed identifiers
- **Expression Evaluation**: OCaml can generate expressions for Rust runtime

## Feature Flags and Configuration

The project supports various build configurations:

- **Default**: Standard library features
- **Testing**: Additional testing utilities and mock implementations
- **PPX**: Preprocessor extensions for DSL syntax
- **Bridge**: FFI bridge components for Rust integration

## Dependencies

- **ml_ssz**: Simple Serialize implementation for OCaml
- **digestif**: Cryptographic hash functions
- **zarith**: Arbitrary precision integers
- **sexplib0**: S-expression support
- **base**: Jane Street's alternative standard library

This OCaml implementation provides a complete toolkit for working with the Causality Resource Model, enabling developers to build Resource-based applications using functional programming paradigms while maintaining full compatibility with the Rust ecosystem.