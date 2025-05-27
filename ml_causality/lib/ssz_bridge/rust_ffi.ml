(* Purpose: FFI interface for calling Rust SSZ functions from OCaml *)

(* External functions that will be implemented in C stubs to call Rust *)
external rust_serialize_tel_graph : string -> string = "rust_serialize_tel_graph_stub"
external rust_deserialize_tel_graph : string -> string = "rust_deserialize_tel_graph_stub"
external rust_compute_content_hash : string -> string = "rust_compute_content_hash_stub"

(* Safe wrapper functions *)
let serialize_tel_graph_via_rust data =
  try
    Some (rust_serialize_tel_graph data)
  with
  | _ -> None

let deserialize_tel_graph_via_rust data =
  try
    Some (rust_deserialize_tel_graph data)
  with
  | _ -> None

let compute_content_hash_via_rust data =
  try
    Some (rust_compute_content_hash data)
  with
  | _ -> None 