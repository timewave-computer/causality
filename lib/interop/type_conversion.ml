(* ------------ TYPE CONVERSION ------------ *)
(* Purpose: Type conversion utilities for external integrations *)

open Ocaml_causality_core
open Ocaml_causality_lang.Value
open Ocaml_causality_serialization.Ssz_compat

(* ------------ JSON CONVERSION ------------ *)

(** Convert value_expr to JSON-like string representation *)
let rec value_expr_to_json = function
  | VNil -> "null"
  | VBool true -> "true"
  | VBool false -> "false"
  | VString s -> "\"" ^ s ^ "\""
  | VInt i -> Int64.to_string i
  | VList vs -> "[" ^ (String.concat "," (List.map value_expr_to_json vs)) ^ "]"
  | VRef _ -> "\"ref\""

(** Convert intent to JSON representation *)
let intent_to_json (intent: intent) : string =
  Printf.sprintf 
    "{\"id\":\"%s\",\"name\":\"%s\",\"domain_id\":\"%s\",\"priority\":%d}"
    (serialize_entity_id intent.id)
    intent.name
    (serialize_entity_id intent.domain_id)
    intent.priority

(* ------------ BINARY CONVERSION ------------ *)

(** Convert bytes to hex string *)
let bytes_to_hex (data: bytes) : string =
  let hex_of_char c = Printf.sprintf "%02x" (Char.code c) in
  Bytes.to_string data |> String.to_seq |> Seq.map hex_of_char |> List.of_seq |> String.concat ""

(** Convert hex string to bytes *)
let hex_to_bytes (hex: string) : bytes =
  (* TODO: Implement proper hex decoding *)
  Bytes.of_string hex

(* ------------ VALIDATION ------------ *)

(* TODO: Add validation functions for type conversions *) 