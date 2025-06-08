(* ------------ SSZ COMPATIBILITY ------------ *)
(* Purpose: SSZ serialization compatibility layer *)

open Ocaml_causality_core

(* ------------ BASIC SSZ OPERATIONS ------------ *)

(** Serialize bytes to string *)
let serialize_bytes (data : bytes) : string = Bytes.to_string data

(** Deserialize string to bytes *)
let deserialize_bytes (data : string) : bytes = Bytes.of_string data

(* ------------ TYPE SERIALIZATION ------------ *)

(** Serialize entity_id *)
let serialize_entity_id (id : entity_id) : string = serialize_bytes id

(** Deserialize entity_id *)
let deserialize_entity_id (data : string) : entity_id = deserialize_bytes data

(* ------------ COMPATIBILITY LAYER ------------ *)

(* TODO: Add compatibility functions for ocaml_ssz integration *)

(* ------------ VALIDATION ------------ *)

(* TODO: Add serialization validation functions *)
