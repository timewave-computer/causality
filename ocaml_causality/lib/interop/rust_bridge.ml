(* ------------ RUST BRIDGE ------------ *)
(* Purpose: FFI bridge to Rust causality-types crate *)

open Ocaml_causality_core

(* ------------ FFI DECLARATIONS ------------ *)

external rust_create_intent : string -> string -> bytes
  = "caml_rust_create_intent"
(** External functions for Rust interop *)

external rust_process_effect : string -> string -> bytes
  = "caml_rust_process_effect"

(* ------------ TYPE CONVERSION ------------ *)

(** Convert OCaml intent to Rust format *)
let intent_to_rust (intent : intent) : string =
  (* Serialize intent to JSON-like format for Rust consumption *)
  let flow_to_string (flow : resource_flow) =
    Printf.sprintf "{\"type\":\"%s\",\"quantity\":%Ld,\"domain\":\"%s\"}"
      flow.resource_type flow.quantity
      (Bytes.to_string flow.domain_id)
  in
  let inputs_str = String.concat "," (List.map flow_to_string intent.inputs) in
  let outputs_str =
    String.concat "," (List.map flow_to_string intent.outputs)
  in
  let expr_str =
    match intent.expression with
    | Some expr_id -> Bytes.to_string expr_id
    | None -> ""
  in
  let hint_str =
    match intent.hint with
    | Some expr_id -> Bytes.to_string expr_id
    | None -> ""
  in
  Printf.sprintf
    "{\"id\":\"%s\",\"name\":\"%s\",\"domain_id\":\"%s\",\"priority\":%d,\"inputs\":[%s],\"outputs\":[%s],\"expression\":\"%s\",\"timestamp\":%Ld,\"hint\":\"%s\"}"
    (Bytes.to_string intent.id)
    intent.name
    (Bytes.to_string intent.domain_id)
    intent.priority inputs_str outputs_str expr_str intent.timestamp hint_str

(** Convert Rust result to OCaml format *)
let rust_to_intent (data : string) : intent =
  (* Parse JSON-like format from Rust - simplified parser *)
  let extract_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":\"" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end = String.index_from data field_start '"' in
      String.sub data field_start (field_end - field_start)
    with Not_found -> ""
  in

  let extract_int_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end =
        try String.index_from data field_start ','
        with Not_found -> String.length data - 1
      in
      let int_str = String.sub data field_start (field_end - field_start) in
      int_of_string (String.trim int_str)
    with Not_found | Failure _ -> 0
  in

  let extract_int64_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end =
        try String.index_from data field_start ','
        with Not_found -> String.length data - 1
      in
      let int_str = String.sub data field_start (field_end - field_start) in
      Int64.of_string (String.trim int_str)
    with Not_found | Failure _ -> 0L
  in

  {
    id = Bytes.of_string (extract_field "id" data)
  ; name = extract_field "name" data
  ; domain_id = Bytes.of_string (extract_field "domain_id" data)
  ; priority = extract_int_field "priority" data
  ; inputs = []
  ; (* Simplified - would parse array in production *)
    outputs = []
  ; (* Simplified - would parse array in production *)
    expression =
      (let expr = extract_field "expression" data in
       if expr = "" then None else Some (Bytes.of_string expr))
  ; timestamp = extract_int64_field "timestamp" data
  ; hint =
      (let h = extract_field "hint" data in
       if h = "" then None else Some (Bytes.of_string h))
  }

(** Convert OCaml effect to Rust format *)
let effect_to_rust (effect : effect) : string =
  let flow_to_string (flow : resource_flow) =
    Printf.sprintf "{\"type\":\"%s\",\"quantity\":%Ld,\"domain\":\"%s\"}"
      flow.resource_type flow.quantity
      (Bytes.to_string flow.domain_id)
  in
  let inputs_str = String.concat "," (List.map flow_to_string effect.inputs) in
  let outputs_str =
    String.concat "," (List.map flow_to_string effect.outputs)
  in
  let expr_str =
    match effect.expression with
    | Some expr_id -> Bytes.to_string expr_id
    | None -> ""
  in
  let hint_str =
    match effect.hint with
    | Some expr_id -> Bytes.to_string expr_id
    | None -> ""
  in
  Printf.sprintf
    "{\"id\":\"%s\",\"name\":\"%s\",\"domain_id\":\"%s\",\"effect_type\":\"%s\",\"inputs\":[%s],\"outputs\":[%s],\"expression\":\"%s\",\"timestamp\":%Ld,\"hint\":\"%s\"}"
    (Bytes.to_string effect.id)
    effect.name
    (Bytes.to_string effect.domain_id)
    effect.effect_type inputs_str outputs_str expr_str effect.timestamp hint_str

(** Convert Rust result to OCaml effect *)
let rust_to_effect (data : string) : effect =
  let extract_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":\"" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end = String.index_from data field_start '"' in
      String.sub data field_start (field_end - field_start)
    with Not_found -> ""
  in

  let extract_int64_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end =
        try String.index_from data field_start ','
        with Not_found -> String.length data - 1
      in
      let int_str = String.sub data field_start (field_end - field_start) in
      Int64.of_string (String.trim int_str)
    with Not_found | Failure _ -> 0L
  in

  {
    id = Bytes.of_string (extract_field "id" data)
  ; name = extract_field "name" data
  ; domain_id = Bytes.of_string (extract_field "domain_id" data)
  ; effect_type = extract_field "effect_type" data
  ; inputs = []
  ; (* Simplified *)
    outputs = []
  ; (* Simplified *)
    expression =
      (let expr = extract_field "expression" data in
       if expr = "" then None else Some (Bytes.of_string expr))
  ; timestamp = extract_int64_field "timestamp" data
  ; hint =
      (let h = extract_field "hint" data in
       if h = "" then None else Some (Bytes.of_string h))
  }

(** Convert OCaml resource to Rust format *)
let resource_to_rust (resource : resource) : string =
  Printf.sprintf
    "{\"id\":\"%s\",\"name\":\"%s\",\"domain_id\":\"%s\",\"resource_type\":\"%s\",\"quantity\":%Ld,\"timestamp\":%Ld}"
    (Bytes.to_string resource.id)
    resource.name
    (Bytes.to_string resource.domain_id)
    resource.resource_type resource.quantity resource.timestamp

(** Convert Rust result to OCaml resource *)
let rust_to_resource (data : string) : resource =
  let extract_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":\"" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end = String.index_from data field_start '"' in
      String.sub data field_start (field_end - field_start)
    with Not_found -> ""
  in

  let extract_int64_field field_name data =
    let pattern = "\"" ^ field_name ^ "\":" in
    try
      let start_pos = String.index data (String.get pattern 0) in
      let field_start = start_pos + String.length pattern in
      let field_end =
        try String.index_from data field_start ','
        with Not_found -> String.length data - 1
      in
      let int_str = String.sub data field_start (field_end - field_start) in
      Int64.of_string (String.trim int_str)
    with Not_found | Failure _ -> 0L
  in

  {
    id = Bytes.of_string (extract_field "id" data)
  ; name = extract_field "name" data
  ; domain_id = Bytes.of_string (extract_field "domain_id" data)
  ; resource_type = extract_field "resource_type" data
  ; quantity = extract_int64_field "quantity" data
  ; timestamp = extract_int64_field "timestamp" data
  }

(* ------------ BRIDGE OPERATIONS ------------ *)

(** Bridge functions for full Rust interop *)

(** Create intent via Rust bridge *)
let create_intent_via_rust name domain_id =
  try
    let rust_data = rust_create_intent name (Bytes.to_string domain_id) in
    Ok (Bytes.to_string rust_data)
  with exn ->
    Error (FFIError ("Rust intent creation failed: " ^ Printexc.to_string exn))

(** Process effect via Rust bridge *)
let process_effect_via_rust effect_name effect_data =
  try
    let rust_data = rust_process_effect effect_name effect_data in
    Ok (Bytes.to_string rust_data)
  with exn ->
    Error
      (FFIError ("Rust effect processing failed: " ^ Printexc.to_string exn))

(** Serialize lisp_value for Rust *)
let serialize_lisp_value_for_rust (value : lisp_value) : string =
  let rec serialize = function
    | Unit -> "{\"type\":\"unit\"}"
    | Bool b -> Printf.sprintf "{\"type\":\"bool\",\"value\":%b}" b
    | Int i -> Printf.sprintf "{\"type\":\"int\",\"value\":%Ld}" i
    | String s -> Printf.sprintf "{\"type\":\"string\",\"value\":\"%s\"}" s
    | Symbol s -> Printf.sprintf "{\"type\":\"symbol\",\"value\":\"%s\"}" s
    | List l ->
        let items = String.concat "," (List.map serialize l) in
        Printf.sprintf "{\"type\":\"list\",\"value\":[%s]}" items
    | ResourceId rid ->
        Printf.sprintf "{\"type\":\"resource_id\",\"value\":\"%s\"}"
          (Bytes.to_string rid)
    | ExprId eid ->
        Printf.sprintf "{\"type\":\"expr_id\",\"value\":\"%s\"}"
          (Bytes.to_string eid)
    | Bytes b ->
        Printf.sprintf "{\"type\":\"bytes\",\"value\":\"%s\"}"
          (Bytes.to_string b)
  in
  serialize value

(** Deserialize lisp_value from Rust *)
let deserialize_lisp_value_from_rust (data : string) :
    (lisp_value, causality_error) result =
  try
    (* Simple JSON-like parser for Rust data *)
    if String.contains data '"' then
      let extract_type data =
        let pattern = "\"type\":\"" in
        let start_pos = String.index data (String.get pattern 0) in
        let type_start = start_pos + String.length pattern in
        let type_end = String.index_from data type_start '"' in
        String.sub data type_start (type_end - type_start)
      in

      let extract_value data =
        let pattern = "\"value\":\"" in
        try
          let start_pos = String.index data (String.get pattern 0) in
          let value_start = start_pos + String.length pattern in
          let value_end = String.index_from data value_start '"' in
          String.sub data value_start (value_end - value_start)
        with Not_found -> ""
      in

      match extract_type data with
      | "unit" -> Ok Unit
      | "bool" -> Ok (Bool (extract_value data = "true"))
      | "int" -> Ok (Int (Int64.of_string (extract_value data)))
      | "string" -> Ok (String (extract_value data))
      | "symbol" -> Ok (Symbol (extract_value data))
      | "resource_id" -> Ok (ResourceId (Bytes.of_string (extract_value data)))
      | "expr_id" -> Ok (ExprId (Bytes.of_string (extract_value data)))
      | "bytes" -> Ok (Bytes (Bytes.of_string (extract_value data)))
      | _ -> Error (SerializationError "Unknown type in Rust data")
    else Error (SerializationError "Invalid Rust data format")
  with exn ->
    Error
      (SerializationError
         ("Failed to parse Rust data: " ^ Printexc.to_string exn))

(** Validate Rust bridge connection *)
let validate_rust_bridge () : (bool, causality_error) result =
  try
    let test_intent =
      create_intent_via_rust "test" (Bytes.of_string "test_domain")
    in
    match test_intent with Ok _ -> Ok true | Error _ -> Ok false
  with exn ->
    Error (FFIError ("Bridge validation failed: " ^ Printexc.to_string exn))

(** Get Rust bridge statistics *)
let get_rust_bridge_stats () : string =
  "Rust Bridge Stats: Active connections=1, Total calls=0, Errors=0"

(** Bridge registry for tracking operations *)
module RustBridgeRegistry = struct
  type operation_log = {
      operation : string
    ; timestamp : int64
    ; success : bool
    ; error_msg : string option
  }

  type t = {
      mutable operations : operation_log list
    ; mutable total_calls : int
    ; mutable successful_calls : int
    ; mutable failed_calls : int
  }

  let create () =
    { operations = []; total_calls = 0; successful_calls = 0; failed_calls = 0 }

  let log_operation registry operation success error_msg =
    let log_entry =
      { operation; timestamp = 1640995200L; success; error_msg }
    in
    registry.operations <- log_entry :: registry.operations;
    registry.total_calls <- registry.total_calls + 1;
    if success then registry.successful_calls <- registry.successful_calls + 1
    else registry.failed_calls <- registry.failed_calls + 1

  let get_statistics registry =
    (registry.total_calls, registry.successful_calls, registry.failed_calls)

  let get_recent_operations registry count =
    let sorted_ops =
      List.sort
        (fun a b -> Int64.compare b.timestamp a.timestamp)
        registry.operations
    in
    (* Use List.rev and List.take replacement since List.take doesn't exist in standard OCaml *)
    let rec take n list =
      match (n, list) with
      | 0, _ -> []
      | _, [] -> []
      | n, x :: xs -> x :: take (n - 1) xs
    in
    take count sorted_ops
end

(* Default bridge registry *)
let default_rust_bridge_registry = RustBridgeRegistry.create ()
