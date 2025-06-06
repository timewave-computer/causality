# SSZ File I/O Examples

This document provides examples of how to use the SSZ file I/O functionality in both Rust and OCaml to serialize and deserialize TEL graph objects to disk.

## Rust Examples

### Writing a Resource to a File

```rust
use causality_ssz::{write_resource, read_resource};
use causality_types::{
    core::id::{ResourceId, DomainId, TypeExprId, ValueExprId, ExprId},
    resource::Resource,
};
use std::path::Path;

fn write_resource_example() -> Result<(), causality_ssz::FileError> {
    // Create a test resource
    let resource = Resource {
        id: ResourceId::new([1u8; 32]),
        domain: DomainId::new([2u8; 32]),
        ephemeral: true,
        value: ValueExprId::new([3u8; 32]),
        type_expr: TypeExprId::new([4u8; 32]),
        static_expr: Some(ExprId::new([5u8; 32])),
    };

    // Write the resource to a file
    write_resource("my_resource.telg", &resource)?;
    
    println!("Resource written to my_resource.telg");
    Ok(())
}

fn read_resource_example() -> Result<Resource, causality_ssz::FileError> {
    // Read the resource from a file
    let resource: Resource = read_resource("my_resource.telg")?;
    
    println!("Resource ID: {:?}", resource.id);
    Ok(resource)
}
```

### Writing Multiple Resources to a File

```rust
use causality_ssz::{ObjectType, write_objects, read_objects};
use causality_types::resource::Resource;
use std::path::Path;

fn write_multiple_resources() -> Result<(), causality_ssz::FileError> {
    // Create a vector of resources
    let resources = vec![
        Resource { /* ... */ },
        Resource { /* ... */ },
        Resource { /* ... */ },
    ];

    // Write the resources to a file
    write_objects("resources.telg", ObjectType::Resource, &resources)?;
    
    println!("Wrote {} resources to resources.telg", resources.len());
    Ok(())
}

fn read_multiple_resources() -> Result<Vec<Resource>, causality_ssz::FileError> {
    // Read the resources from a file
    let resources: Vec<Resource> = read_objects("resources.telg", ObjectType::Resource)?;
    
    println!("Read {} resources from resources.telg", resources.len());
    Ok(resources)
}
```

### Working with Effects, Handlers, and Edges

```rust
use causality_ssz::{write_effect, read_effect, write_handler, read_handler, write_edge, read_edge};
use causality_types::tel::{effect::Effect, handler::Handler, Edge};

// Write an effect to a file
fn save_effect(effect: &Effect) -> Result<(), causality_ssz::FileError> {
    write_effect("my_effect.telg", effect)
}

// Read an effect from a file
fn load_effect() -> Result<Effect, causality_ssz::FileError> {
    read_effect("my_effect.telg")
}

// Similarly for handlers
fn save_handler(handler: &Handler) -> Result<(), causality_ssz::FileError> {
    write_handler("my_handler.telg", handler)
}

// And for edges
fn save_edge(edge: &Edge) -> Result<(), causality_ssz::FileError> {
    write_edge("my_edge.telg", edge)
}
```

## OCaml Examples

### Writing a Resource to a File

```ocaml
open Ml_causality.Ssz.File_io
open Ml_causality.Ssz.Tel
open Ml_causality.Types

(* Create a resource serialization function *)
let serialize_resource resource =
  (* Convert the resource to SSZ bytes *)
  Tel.serialize_resource resource

(* Function to save a resource *)
let save_resource path resource =
  match write_object path Resource serialize_resource resource with
  | Ok () -> 
      Printf.printf "Resource saved to %s\n" path;
      true
  | Error e ->
      Printf.printf "Error: %s\n" (string_of_file_error e);
      false

(* Function to load a resource *)
let load_resource path =
  match read_object path Resource Tel.deserialize_resource with
  | Ok resource -> 
      Printf.printf "Resource loaded from %s\n" path;
      Some resource
  | Error e ->
      Printf.printf "Error: %s\n" (string_of_file_error e);
      None
```

### Working with Multiple Objects

```ocaml
open Ml_causality.Ssz.File_io

(* Save multiple resources to a file *)
let save_resources path resources =
  match write_objects path Resource Tel.serialize_resource resources with
  | Ok () -> 
      Printf.printf "Saved %d resources to %s\n" (List.length resources) path;
      true
  | Error e ->
      Printf.printf "Error: %s\n" (string_of_file_error e);
      false

(* Load multiple resources from a file *)
let load_resources path =
  match read_objects path Resource Tel.deserialize_resource with
  | Ok resources -> 
      Printf.printf "Loaded %d resources from %s\n" (List.length resources) path;
      Some resources
  | Error e ->
      Printf.printf "Error: %s\n" (string_of_file_error e);
      None
```

### Working with Different TEL Object Types

```ocaml
open Ml_causality.Ssz.File_io
open Ml_causality.Ssz.Tel

(* Save an effect to a file *)
let save_effect path effect =
  match write_object path Effect Tel.serialize_effect effect with
  | Ok () -> Printf.printf "Effect saved to %s\n" path; true
  | Error e -> Printf.printf "Error: %s\n" (string_of_file_error e); false

(* Load an effect from a file *)
let load_effect path =
  match read_object path Effect Tel.deserialize_effect with
  | Ok effect -> Printf.printf "Effect loaded from %s\n" path; Some effect
  | Error e -> Printf.printf "Error: %s\n" (string_of_file_error e); None

(* Save a handler to a file *)
let save_handler path handler =
  match write_object path Handler Tel.serialize_handler handler with
  | Ok () -> Printf.printf "Handler saved to %s\n" path; true
  | Error e -> Printf.printf "Error: %s\n" (string_of_file_error e); false

(* Save an edge to a file *)
let save_edge path edge =
  match write_object path Edge Tel.serialize_edge edge with
  | Ok () -> Printf.printf "Edge saved to %s\n" path; true
  | Error e -> Printf.printf "Error: %s\n" (string_of_file_error e); false
```

## File Format Details

The SSZ file format for TEL graph objects has the following structure:

### Single Object File Format

```
+----------------+------------------+
| Header (10 B)  | Object Data      |
+----------------+------------------+
```

Header Structure:
- Magic bytes "TELG" (4 bytes)
- Version (1 byte)
- Object type (1 byte)
- Object count (4 bytes) - always 1 for single objects

### Multiple Object File Format

```
+----------------+----------------+----------------+-------+----------------+
| Header (10 B)  | Length1 (4 B)  | Object1 Data   | ...   | ObjectN Data   |
+----------------+----------------+----------------+-------+----------------+
```

Header Structure:
- Magic bytes "TELG" (4 bytes)
- Version (1 byte)
- Object type (1 byte)
- Object count (4 bytes) - number of objects in the file

Each object is prefixed with a 4-byte length field indicating the size of the object data in bytes.

## Compatibility

The file format is designed to be compatible between Rust and OCaml implementations. Files created by one language can be read by the other, providing a seamless way to exchange TEL graph objects between the two runtime environments. 