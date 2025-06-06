(* ------------ RUST BRIDGE ------------ *)
(* Purpose: FFI bridge to Rust causality-types crate *)

open Ocaml_causality_core

(* ------------ FFI DECLARATIONS ------------ *)

(** External functions for Rust interop *)
external rust_create_intent : string -> string -> bytes
  = "caml_rust_create_intent"

external rust_process_effect : string -> string -> bytes  
  = "caml_rust_process_effect"

(* ------------ TYPE CONVERSION ------------ *)

(** Convert OCaml intent to Rust format *)
let intent_to_rust (intent: intent) : string =
  (* TODO: Implement serialization to Rust-compatible format *)
  intent.name

(** Convert Rust result to OCaml format *)
let rust_to_intent (data: string) : intent =
  (* TODO: Implement deserialization from Rust format *)
  {
    id = Bytes.of_string "placeholder";
    name = data;
    domain_id = Bytes.of_string "default";
    priority = 0;
    inputs = [];
    outputs = [];
    expression = None;
    timestamp = 0L;
    hint = None;
  }

(* ------------ BRIDGE OPERATIONS ------------ *)

(* TODO: Add more bridge functions for full Rust interop *) 