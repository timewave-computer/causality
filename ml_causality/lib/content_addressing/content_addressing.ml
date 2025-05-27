(**
 * Unified content addressing using SSZ serialization + SHA256 hashing
 * 
 * This provides the canonical method for generating content-addressed IDs
 * across both Rust and OCaml systems, ensuring identical results for
 * identical content.
 *)

open Ml_causality_lib_types.Types

(**
 * Convert value_expr to SSZ bytes for content addressing (mirrors Rust implementation)
 *)
let value_expr_to_ssz_bytes (ve: value_expr) : bytes =
  (* Deterministic binary representation that mirrors Rust SSZ serialization *)
  let rec serialize_value_expr ve =
    match ve with
    | VNil -> "\001"
    | VBool b -> "\002" ^ (if b then "\001" else "\000")
    | VString s -> "\003" ^ (Printf.sprintf "%04d%s" (String.length s) s)
    | VInt i -> "\004" ^ (Printf.sprintf "%016Lx" i)
    | VList items -> 
        let item_bytes = List.map serialize_value_expr items in
        "\007" ^ (Printf.sprintf "%04d" (List.length items)) ^ (String.concat "" item_bytes)
    | VMap m ->
        let entries = BatMap.bindings m in
        let sorted_entries = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) entries in
        let entry_bytes = List.map (fun (k, v) -> 
          (Printf.sprintf "%04d%s" (String.length k) k) ^ (serialize_value_expr v)
        ) sorted_entries in
        "\008" ^ (Printf.sprintf "%04d" (List.length entries)) ^ (String.concat "" entry_bytes)
    | VStruct s_map -> 
        let fields = BatMap.bindings s_map in
        let sorted_fields = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) fields in
        let field_bytes = List.map (fun (k, v) -> 
          (Printf.sprintf "%04d%s" (String.length k) k) ^ (serialize_value_expr v)
        ) sorted_fields in
        "\009" ^ (Printf.sprintf "%04d" (List.length fields)) ^ (String.concat "" field_bytes)
    | VRef (VERValue id) -> "\010" ^ (Printf.sprintf "%04d%s" (Bytes.length id) (Bytes.to_string id))
    | VRef (VERExpr id) -> "\011" ^ (Printf.sprintf "%04d%s" (Bytes.length id) (Bytes.to_string id))
    | VLambda { params; body_expr_id; captured_env } ->
        let param_bytes = String.concat "\000" params in
        let env_entries = BatMap.bindings captured_env in
        let env_bytes = List.map (fun (k, v) -> 
          (Printf.sprintf "%04d%s" (String.length k) k) ^ (serialize_value_expr v)
        ) (List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) env_entries) in
        "\012" ^ 
        (Printf.sprintf "%04d%s" (String.length param_bytes) param_bytes) ^
        (Printf.sprintf "%04d%s" (Bytes.length body_expr_id) (Bytes.to_string body_expr_id)) ^
        (Printf.sprintf "%04d" (List.length env_entries)) ^ (String.concat "" env_bytes)
  in
  Bytes.of_string (serialize_value_expr ve)

(**
 * Generate content-addressed ID from SSZ bytes using SHA256 (mirrors Rust implementation)
 *)
let content_id_from_bytes (ssz_bytes: bytes) : string =
  Digestif.SHA256.to_hex (Digestif.SHA256.digest_bytes ssz_bytes)

(**
 * Generate content-addressed ID from value_expr using SSZ + SHA256 (mirrors Rust implementation)
 *)
let content_id_from_value_expr (ve: value_expr) : value_expr_id =
  let ssz_bytes = value_expr_to_ssz_bytes ve in
  let hex_string = content_id_from_bytes ssz_bytes in
  Bytes.of_string hex_string

(**
 * Generic function to generate content-addressed ID from any data that can be converted to bytes
 *)
let content_id_from_string (data: string) : string =
  let bytes = Bytes.of_string data in
  content_id_from_bytes bytes

(**
 * Content addressing trait-like functions for common types
 *)

(** Generate content-addressed ID for effect type configuration *)
let content_id_for_effect_config (effect_name: string) (config_data: string) : string =
  let combined_data = effect_name ^ "|" ^ config_data in
  content_id_from_string combined_data

(** Generate content-addressed ID for handler definition *)
let content_id_for_handler (handler_name: string) (handles_effects: string list) (config: value_expr) : string =
  let effects_str = String.concat "," (List.sort String.compare handles_effects) in
  let config_bytes = value_expr_to_ssz_bytes config in
  let combined_data = handler_name ^ "|" ^ effects_str ^ "|" ^ (Bytes.to_string config_bytes) in
  content_id_from_string combined_data

(** Generate content-addressed ID for lisp code (for PPX registry) *)
let content_id_for_lisp_code (code: string) : string =
  let normalized_code = String.trim code in
  content_id_from_string normalized_code

(**
 * Debug helper to convert value_expr to S-expression string (for debugging only)
 *)
let rec value_expr_to_s_expression_debug (ve: value_expr) : string =
  match ve with
  | VNil -> "nil"
  | VBool b -> string_of_bool b
  | VString s -> Printf.sprintf "\"%s\"" (String.escaped s)
  | VInt i -> Int64.to_string i
  | VList items -> 
    let item_strs = List.map value_expr_to_s_expression_debug items in
    Printf.sprintf "(%s)" (String.concat " " item_strs)
  | VMap m ->
    let entries = BatMap.bindings m in
    let sorted_entries = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) entries in
    let entry_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression_debug v)
    ) sorted_entries in
    Printf.sprintf "(map (%s))" (String.concat " " entry_strs)
  | VStruct s_map -> 
    let fields = BatMap.bindings s_map in
    let sorted_fields = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) fields in
    let field_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression_debug v)
    ) sorted_fields in
    Printf.sprintf "(struct (%s))" (String.concat " " field_strs)
  | VRef (VERValue id) -> Printf.sprintf "(ref:value %s)" (Bytes.to_string id)
  | VRef (VERExpr id) -> Printf.sprintf "(ref:expr %s)" (Bytes.to_string id)
  | VLambda { params; body_expr_id; captured_env } ->
    let param_str = String.concat " " params in
    let env_entries = BatMap.bindings captured_env in
    let sorted_env = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) env_entries in
    let env_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression_debug v)
    ) sorted_env in
    Printf.sprintf "(lambda (%s) %s (env %s))" param_str (Bytes.to_string body_expr_id) (String.concat " " env_strs) 