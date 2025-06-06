# ML SSZ: Simple Serialize for OCaml

A comprehensive OCaml implementation of the Simple Serialize (SSZ) serialization format used in blockchain systems like Ethereum 2.0. This library provides complete SSZ serialization primitives, Merkle tree operations, and integration utilities for the Causality Resource Model framework.

## Overview

The `ocaml_ssz` library provides a pure OCaml implementation of SSZ serialization with:

- **Complete SSZ Primitives**: Support for all SSZ basic types and collections
- **Type-Safe Serialization**: OCaml type system ensures serialization correctness
- **Merkle Tree Operations**: Full Merkle tree construction and verification
- **Content Addressing**: Utilities for content-addressed identifiers
- **FFI Integration**: Bridge components for Rust interoperability
- **TEL Support**: Specialized serialization for Temporal Effect Language expressions

All implementations maintain compatibility with the SSZ specification and provide deterministic, content-addressed serialization.

## Features

### Core SSZ Support

- **Basic Types**: `bool`, `uint8`, `uint16`, `uint32`, `uint64`, `uint128`, `uint256`
- **Variable Types**: `string`, `bytes` with length prefixes
- **Collections**: Lists, vectors, and fixed-size arrays
- **Containers**: Struct-like composite types with field offsets
- **Union Types**: Tagged union serialization

### Advanced Features

- **Merkle Tree Roots**: Automatic Merkle root calculation for all types
- **Content Addressing**: SHA-256 based content identifiers
- **Streaming Serialization**: Memory-efficient serialization for large data
- **Type Specifications**: Rich type metadata system
- **Mock FFI Stubs**: Testing without Rust dependencies

### Causality Integration

- **TEL Expression Serialization**: Specialized support for Lisp expressions
- **Resource Model Types**: Direct serialization of Causality types
- **Graph Structures**: Serialization for TEL graphs and dataflow blocks

## Installation

```bash
# Install dependencies
opam install dune digestif bytes

# Build from source
git clone https://github.com/timewave-computer/causality.git
cd causality/ocaml_ssz
dune build

# Run tests
dune runtest
```

## Core Modules

### Basic Types

Serialization for primitive SSZ types:

```ocaml
open Ssz.Basic

(* Integer serialization *)
let serialize_uint32 (value : int32) : bytes = ...
let deserialize_uint32 (data : bytes) : int32 = ...

(* Boolean serialization *)
let serialize_bool (value : bool) : bytes = ...
let deserialize_bool (data : bytes) : bool = ...

(* String serialization with length prefix *)
let serialize_string (value : string) : bytes = ...
let deserialize_string (data : bytes) : string = ...
```

### Collections

Serialization for SSZ collections:

```ocaml
open Ssz.Collections

(* List serialization (variable length) *)
let serialize_list (serialize_item : 'a -> bytes) (items : 'a list) : bytes = ...
let deserialize_list (deserialize_item : bytes -> int -> 'a * int) (data : bytes) : 'a list = ...

(* Vector serialization (fixed length) *)
let serialize_vector (serialize_item : 'a -> bytes) (length : int) (items : 'a list) : bytes = ...
let deserialize_vector (deserialize_item : bytes -> int -> 'a * int) (length : int) (data : bytes) : 'a list = ...
```

### Container Types

Serialization for structured data:

```ocaml
open Ssz.Container

(* Define container specification *)
type person_spec = {
  id : int32;
  name : string;
  age : int;
  is_active : bool;
}

(* Container serialization *)
let serialize_person (p : person_spec) : bytes =
  let fields = [
    ("id", Basic.serialize_uint32 p.id);
    ("name", Basic.serialize_string p.name);
    ("age", Basic.serialize_uint8 p.age);
    ("is_active", Basic.serialize_bool p.is_active);
  ] in
  serialize_container fields

let deserialize_person (data : bytes) : person_spec =
  let fields = deserialize_container data in
  {
    id = Basic.deserialize_uint32 (List.assoc "id" fields);
    name = Basic.deserialize_string (List.assoc "name" fields);
    age = Basic.deserialize_uint8 (List.assoc "age" fields);
    is_active = Basic.deserialize_bool (List.assoc "is_active" fields);
  }
```

### Type System

Rich type specifications for SSZ types:

```ocaml
open Ssz.Types

(* Type specification *)
type 'a t = {
  kind : kind;                     (* Basic, Vector, List, Container, Union *)
  size : int option;               (* Fixed size in bytes, None for variable *)
  encode : 'a -> bytes;            (* Encoding function *)
  decode : bytes -> int -> 'a * int; (* Decoding function *)
}

(* Type utilities *)
let is_fixed_size (typ : 'a t) : bool = Option.is_some typ.size
let fixed_size (typ : 'a t) : int = match typ.size with
  | Some size -> size
  | None -> failwith "Type does not have a fixed size"

(* Constants *)
module Constants = struct
  let max_chunk_size = 32
  let bytes_per_length_offset = 4
  let bytes_per_length_prefix = 4
  let default_max_length = 1024 * 1024
end
```

### Merkle Trees

Merkle tree construction and verification:

```ocaml
open Ssz.Merkle

(* Merkle tree operations *)
let merkle_root (chunks : bytes list) : bytes = ...
let merkle_proof (chunks : bytes list) (index : int) : bytes list = ...
let verify_merkle_proof (root : bytes) (leaf : bytes) (proof : bytes list) (index : int) : bool = ...

(* Merkleization for SSZ types *)
let merkleize_chunks (chunks : bytes list) : bytes = ...
let hash_tree_root (data : bytes) : bytes = ...
```

### TEL Integration

Specialized serialization for Temporal Effect Language:

```ocaml
open Ssz.Tel

(* TEL expression serialization *)
type tel_expr = 
  | Atom of string
  | List of tel_expr list
  | Symbol of string
  | Number of int64

let serialize_tel_expr (expr : tel_expr) : bytes = ...
let deserialize_tel_expr (data : bytes) : tel_expr = ...

(* TEL graph serialization *)
type tel_node = {
  id : string;
  expr : tel_expr;
  dependencies : string list;
}

let serialize_tel_graph (nodes : tel_node list) : bytes = ...
let deserialize_tel_graph (data : bytes) : tel_node list = ...
```

### File I/O

Utilities for reading and writing SSZ data:

```ocaml
open Ssz.File_io

(* File operations *)
let write_ssz_file (filename : string) (data : bytes) : unit = ...
let read_ssz_file (filename : string) : bytes = ...

(* Streaming operations *)
let write_ssz_stream (channel : out_channel) (data : bytes) : unit = ...
let read_ssz_stream (channel : in_channel) : bytes = ...
```

## Usage Examples

### Basic Serialization

```ocaml
open Ssz

(* Serialize a simple record *)
type token = {
  id : int32;
  balance : int64;
  owner : string;
  frozen : bool;
}

let serialize_token (t : token) : bytes =
  let fields = [
    Basic.serialize_uint32 t.id;
    Basic.serialize_uint64 t.balance;
    Basic.serialize_string t.owner;
    Basic.serialize_bool t.frozen;
  ] in
  Container.serialize_container_fields fields

let deserialize_token (data : bytes) : token =
  let fields = Container.deserialize_container_fields data in
  match fields with
  | [id_bytes; balance_bytes; owner_bytes; frozen_bytes] ->
      {
        id = Basic.deserialize_uint32 id_bytes;
        balance = Basic.deserialize_uint64 balance_bytes;
        owner = Basic.deserialize_string owner_bytes;
        frozen = Basic.deserialize_bool frozen_bytes;
      }
  | _ -> failwith "Invalid token serialization"
```

### Content Addressing

```ocaml
open Ssz

(* Create content-addressed identifier *)
let content_id (data : bytes) : bytes =
  Merkle.hash_tree_root data

(* Verify content addressing *)
let verify_content_id (data : bytes) (expected_id : bytes) : bool =
  let computed_id = content_id data in
  Bytes.equal computed_id expected_id

(* Example usage *)
let token_data = serialize_token my_token in
let token_id = content_id token_data in
Printf.printf "Token ID: %s\n" (Bytes.to_string token_id)
```

### Merkle Proofs

```ocaml
open Ssz

(* Create Merkle proof for a list of items *)
let create_proof (items : bytes list) (index : int) : bytes list =
  Merkle.merkle_proof items index

(* Verify Merkle proof *)
let verify_proof (root : bytes) (leaf : bytes) (proof : bytes list) (index : int) : bool =
  Merkle.verify_merkle_proof root leaf proof index

(* Example: Prove inclusion of a token in a list *)
let token_list = [serialize_token token1; serialize_token token2; serialize_token token3] in
let root = Merkle.merkle_root token_list in
let proof = create_proof token_list 1 in
let verified = verify_proof root (serialize_token token2) proof 1 in
assert verified
```

### Integration with Causality Types

```ocaml
(* Serialize Causality Resource *)
type resource = {
  id : bytes;
  resource_type : string;
  domain_id : bytes;
  quantity : int64;
  timestamp : int64;
}

let serialize_resource (r : resource) : bytes =
  Container.serialize_container [
    ("id", r.id);
    ("resource_type", Basic.serialize_string r.resource_type);
    ("domain_id", r.domain_id);
    ("quantity", Basic.serialize_uint64 r.quantity);
    ("timestamp", Basic.serialize_uint64 r.timestamp);
  ]

(* Content-addressed Resource ID *)
let resource_content_id (r : resource) : bytes =
  let serialized = serialize_resource r in
  Merkle.hash_tree_root serialized
```

## FFI Integration

Mock FFI stubs for testing without Rust dependencies:

```ocaml
open Ssz.Ssz_ffi

(* Mock FFI functions *)
val mock_serialize_to_rust : bytes -> bytes
val mock_deserialize_from_rust : bytes -> bytes
val mock_compute_merkle_root : bytes list -> bytes

(* These functions simulate Rust FFI behavior for testing *)
let test_rust_integration () =
  let data = Basic.serialize_string "test" in
  let rust_result = mock_serialize_to_rust data in
  let ocaml_result = mock_deserialize_from_rust rust_result in
  assert (Bytes.equal data ocaml_result)
```

## Performance Considerations

### Memory Efficiency

- **Streaming Serialization**: Process large data without loading everything into memory
- **Lazy Evaluation**: Defer expensive operations until needed
- **Buffer Reuse**: Minimize memory allocations

### Optimization Tips

```ocaml
(* Use buffer pools for frequent serialization *)
let buffer_pool = Buffer.create 1024

(* Batch operations when possible *)
let serialize_batch (items : 'a list) (serialize_fn : 'a -> bytes) : bytes =
  let chunks = List.map serialize_fn items in
  Collections.serialize_list (fun x -> x) chunks

(* Cache Merkle roots for unchanged data *)
let merkle_cache = Hashtbl.create 1000
```

## Testing

```bash
# Run all tests
dune runtest

# Run specific test suites
dune exec test/test_basic.exe
dune exec test/test_collections.exe
dune exec test/test_merkle.exe

# Run examples
dune exec examples/simple_serialization.exe
dune exec examples/merkle_proof.exe
```

## Dependencies

- **digestif**: Cryptographic hash functions (SHA-256, etc.)
- **bytes**: Byte array operations
- **dune**: Build system

## License

This project is licensed under the MIT License - see the LICENSE file for details.

This library provides a complete, type-safe implementation of SSZ serialization for OCaml, with specialized support for the Causality Resource Model framework and seamless integration with Rust components. 