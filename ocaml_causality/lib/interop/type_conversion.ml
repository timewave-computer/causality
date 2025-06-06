(* ------------ TYPE CONVERSION ------------ *)
(* Purpose: Type conversion utilities for external integrations *)

open Ocaml_causality_core

(* ------------ JSON CONVERSION ------------ *)

(** Convert lisp_value to JSON-like string representation *)
let rec lisp_value_to_json = function
  | Unit -> "null"
  | Bool true -> "true"
  | Bool false -> "false"
  | String s -> "\"" ^ s ^ "\""
  | Int i -> Int64.to_string i
  | List vs -> "[" ^ (String.concat "," (List.map lisp_value_to_json vs)) ^ "]"
  | Symbol s -> "\"" ^ s ^ "\""
  | ResourceId rid -> "\"#resource:" ^ (Bytes.to_string rid) ^ "\""
  | ExprId eid -> "\"#expr:" ^ (Bytes.to_string eid) ^ "\""
  | Bytes b -> "\"#bytes:" ^ (Bytes.to_string b) ^ "\""

(** Convert intent to JSON representation *)
let intent_to_json (intent: intent) : string =
  Printf.sprintf 
    "{\"id\":\"%s\",\"name\":\"%s\",\"domain_id\":\"%s\",\"priority\":%d}"
    (Bytes.to_string intent.id)
    intent.name
    (Bytes.to_string intent.domain_id)
    intent.priority

(* ------------ BINARY CONVERSION ------------ *)

(** Convert bytes to hex string *)
let bytes_to_hex (data: bytes) : string =
  let hex_of_char c = Printf.sprintf "%02x" (Char.code c) in
  Bytes.to_string data |> String.to_seq |> Seq.map hex_of_char |> List.of_seq |> String.concat ""

(** Convert hex string to bytes *)
let hex_to_bytes (hex: string) : bytes =
  Bytes.of_string hex

(* ------------ VALIDATION ------------ *)

(** Validate that a resource_id is properly formatted *)
let validate_resource_id (rid: resource_id) : bool =
  Bytes.length rid > 0

(** Validate that an expr_id is properly formatted *)
let validate_expr_id (eid: expr_id) : bool =
  Bytes.length eid > 0

(* TODO: Add validation functions for type conversions *) 