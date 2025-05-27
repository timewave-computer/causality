# SSZ Serialization for TEL Graph Types

This module provides SSZ (Simple Serialize) serialization for TEL graph types in the Causality system. 

## Canonical SSZ Schemas

These are the canonical SSZ type schemas used for serialization across languages:

### Core Types

```
# ID types (32 bytes)
class SszId(Container):
    bytes: Vector[byte, 32]

# String type (max 32 bytes)
class SszStr(Vector[byte, 32])
```

### Resource

```
class SszResource(Container):
    id: SszId
    domain: SszId
    ephemeral: boolean
    value: SszId
    type_expr: SszId
    static_expr_present: boolean
    static_expr: Option[SszId]
```

### Effect

```
class SszOutputDefinition(Container):
    id: SszId
    type_expr: SszId

class SszEffect(Container):
    id: SszId
    domain: SszId
    intent_id: SszId
    effect_type: Vector[byte, 32]  # Fixed-size String
    dynamic_expr_present: boolean
    dynamic_expr: Option[SszId]
    inputs: List[SszId, 128]
    outputs: List[SszOutputDefinition, 128]
    constraints: List[SszId, 32]
    scoped_handler_present: boolean
    scoped_handler: Option[SszId]
```

### Handler

```
class SszHandler(Container):
    id: SszId
    domain: SszId
    effect_type: Vector[byte, 32]  # Fixed-size String
    constraints: List[SszId, 32]
    dynamic_expr: SszId
    priority: uint8
    cost: uint64
    ephemeral: boolean
```

### Edge

```
class SszEdge(Container):
    id: SszId
    source: SszId
    target: SszId
    edge_type: uint8
    condition_present: boolean
    condition: Option[SszId]
```

## Binary Layout

SSZ serialization follows these rules for binary layout:

1. **Fixed-size types** are encoded directly
2. **Variable-size types** are encoded using offsets and variable portions
3. **Containers** encode their fields in order, with fixed-size fields first and variable-size fields as offsets

## Memory Management

Care is taken in the FFI layer to ensure proper memory management between Rust and OCaml:

1. Rust allocates memory for objects created during deserialization
2. OCaml is responsible for calling the appropriate `free` functions when done with the objects
3. FFI transfers use explicit memory management to avoid leaks

## Versioning

The SSZ schemas include version information to enable future backwards compatibility:

1. Each schema has an implicit version (the current one is v1)
2. Future versions will maintain backward compatibility when possible
3. Version negotiation happens through detection mechanisms

## Usage

```ocaml
(* Serialize a resource to SSZ bytes *)
let ssz_bytes = Tel.resource_to_ssz my_resource

(* Deserialize SSZ bytes back to a resource *)
let my_resource = Tel.ssz_to_resource ssz_bytes
```

## Features

- Type-directed serialization with static type safety
- Support for basic types (boolean, integers, strings, bytes)
- Collection types (fixed arrays, lists, vectors, dictionaries)
- Container types for composite data structures
- Merkleization support with hash tree roots
- Optimized for both performance and memory usage

## Basic Usage

### Serializing Basic Types

```ocaml
open Ssz

(* Serialize a boolean *)
let bool_value = true
let encoded_bool = Serialize.encode Basic.bool bool_value
let decoded_bool = Serialize.decode Basic.bool encoded_bool

(* Serialize integers *)
let int_value = 42
let encoded_int = Serialize.encode Basic.uint32 int_value
let decoded_int = Serialize.decode Basic.uint32 encoded_int
```

### Working with Collections

```ocaml
(* Fixed-size array *)
let uint16_array = Collections.fixed_array Basic.uint16 3
let arr = [|1; 2; 3|]
let encoded_arr = Serialize.encode uint16_array arr
let decoded_arr = Serialize.decode uint16_array encoded_arr

(* Variable-length list with maximum size *)
let string_list = Collections.list Basic.string 10
let lst = ["hello"; "ssz"; "world"]
let encoded_lst = Serialize.encode string_list lst
let decoded_lst = Serialize.decode string_list encoded_lst
```

### Creating Composite Types

```ocaml
(* Define a simple record type *)
type point = { x: int; y: int }

(* Create fields for the container *)
let point_x = Container.field Basic.uint32 (fun p -> p.x) ~description:"X coordinate"
let point_y = Container.field Basic.uint32 (fun p -> p.y) ~description:"Y coordinate"

(* Create the container type *)
let point_type = Container.create2
  Basic.uint32 Basic.uint32
  ~construct:(fun x y -> { x; y })
  point_x point_y

(* Serialize and deserialize *)
let p = { x = 10; y = 20 }
let encoded_point = Serialize.encode point_type p
let decoded_point = Serialize.decode point_type encoded_point
```

### Merkleization

```ocaml
(* Compute hash tree root of a value *)
let root = Merkle.hash_tree_root Basic.bool true

(* For collections *)
let list_type = Collections.list Basic.uint32 10
let values = [1; 2; 3; 4]
let list_root = Merkle.hash_tree_root list_type values
```

## Advanced Usage

### Nested Containers

```ocaml
type point = { x: int; y: int }
type line = { start: point; end_point: point }

(* Create point type as above *)

(* Create line type using points *)
let line_start = Container.field point_type (fun l -> l.start) ~description:"Start point"
let line_end = Container.field point_type (fun l -> l.end_point) ~description:"End point"

let line_type = Container.create2
  point_type point_type
  ~construct:(fun start end_point -> { start; end_point })
  line_start line_end
```

### Dictionary Types

```ocaml
(* Create a dictionary mapping strings to integers *)
let string_int_dict = Collections.dict Basic.string Basic.uint32 100
let dict_values = [("one", 1); ("two", 2); ("three", 3)]
let encoded_dict = Serialize.encode string_int_dict dict_values
let decoded_dict = Serialize.decode string_int_dict encoded_dict
```

## Performance Considerations

- Fixed-size types are more efficient to serialize than variable-size types
- Consider using fixed-length arrays when possible instead of variable-length lists
- The merkleization process has O(n log n) complexity, where n is the number of chunks

## Integration with Existing Code

To transition from ssz to SSZ:

1. Define SSZ types corresponding to your existing data structures
2. Create serialization/deserialization wrappers if needed
3. Replace ssz serialization calls with SSZ encoding/decoding 