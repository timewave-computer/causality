(* ------------ FOREIGN FUNCTION INTERFACE ------------ *)
(* Purpose: Rust FFI bindings and C stubs *)

open Ocaml_causality_core

(* Abstract types for FFI handles *)
type causality_value
type causality_resource
type causality_expr

(* Value type enumeration matching Rust FFI *)
type value_type = Unit | Bool | Int | Symbol | String | Product | Sum | Record

(* Convert value_type to int for FFI *)
let value_type_to_int = function
  | Unit -> 0
  | Bool -> 1
  | Int -> 2
  | Symbol -> 3
  | String -> 4
  | Product -> 5
  | Sum -> 6
  | Record -> 7

(* Convert int from FFI to value_type *)
let int_to_value_type = function
  | 0 -> Unit
  | 1 -> Bool
  | 2 -> Int
  | 3 -> Symbol
  | 4 -> String
  | 5 -> Product
  | 6 -> Sum
  | 7 -> Record
  | _ -> Unit (* fallback *)

(* Serialization result from Rust FFI *)
type serialization_result = {
    data : bytes option
  ; length : int
  ; error_code : int
  ; error_message : string option
}

(* --------------------------------------------------------------------------- *)
(* External C function declarations for basic value operations *)
(* --------------------------------------------------------------------------- *)

external create_unit : unit -> causality_value = "ocaml_causality_value_unit"
external create_bool : bool -> causality_value = "ocaml_causality_value_bool"
external create_int : int -> causality_value = "ocaml_causality_value_int"

external create_string : string -> causality_value
  = "ocaml_causality_value_string"

external create_symbol : string -> causality_value
  = "ocaml_causality_value_symbol"

external free_value : causality_value -> unit = "ocaml_causality_value_free"

external get_value_type_int : causality_value -> int
  = "ocaml_causality_value_type"

external extract_bool : causality_value -> int = "ocaml_causality_value_as_bool"
external extract_int : causality_value -> int = "ocaml_causality_value_as_int"

external extract_string_ptr : causality_value -> string option
  = "ocaml_causality_value_as_string"

external serialize_value : causality_value -> bytes * int * int * string option
  = "ocaml_causality_value_serialize"

external deserialize_value : bytes -> int -> causality_value option
  = "ocaml_causality_value_deserialize"

external free_serialized_data : bytes -> int -> unit
  = "ocaml_causality_free_serialized_data"

external test_roundtrip : causality_value -> bool
  = "ocaml_causality_test_roundtrip"

external test_all_roundtrips : unit -> bool
  = "ocaml_causality_test_all_roundtrips"

external get_ffi_version : unit -> string = "ocaml_causality_ffi_version"

external get_debug_info : causality_value -> string
  = "ocaml_causality_value_debug_info"

(* --------------------------------------------------------------------------- *)
(* External C function declarations for resource management *)
(* --------------------------------------------------------------------------- *)

external create_resource_ffi : string -> bytes -> int64 -> causality_resource
  = "ocaml_causality_create_resource"

external consume_resource_ffi : causality_resource -> bool
  = "ocaml_causality_consume_resource"

external is_resource_valid_ffi : causality_resource -> bool
  = "ocaml_causality_is_resource_valid"

external get_resource_id_ffi : causality_resource -> bytes
  = "ocaml_causality_resource_id"

(* --------------------------------------------------------------------------- *)
(* External C function declarations for expression management *)
(* --------------------------------------------------------------------------- *)

external compile_expr_ffi : string -> causality_expr
  = "ocaml_causality_compile_expr"

external get_expr_id_ffi : causality_expr -> bytes = "ocaml_causality_expr_id"

external submit_intent_ffi : string -> bytes -> string -> bool
  = "ocaml_causality_submit_intent"

external get_system_metrics_ffi : unit -> string
  = "ocaml_causality_get_system_metrics"

(* --------------------------------------------------------------------------- *)
(* High-level OCaml interface *)
(* --------------------------------------------------------------------------- *)

(** Create a unit value *)
let create_unit () = create_unit ()

(** Create a boolean value *)
let create_bool b = create_bool b

(** Create an integer value *)
let create_int i = create_int i

(** Create a string value *)
let create_string s = create_string s

(** Create a symbol value *)
let create_symbol s = create_symbol s

(** Get the type of a value *)
let get_type value = int_to_value_type (get_value_type_int value)

(** Extract boolean value (returns None if not a boolean) *)
let as_bool value =
  match extract_bool value with
  | -1 -> None
  | 0 -> Some false
  | 1 -> Some true
  | _ -> None

(** Extract integer value (returns None if not an integer, Some 0 could be
    ambiguous) *)
let as_int value =
  match get_type value with Int -> Some (extract_int value) | _ -> None

(** Extract string value (returns None if not a string) *)
let as_string value =
  match get_type value with
  | String | Symbol -> extract_string_ptr value
  | _ -> None

(* --------------------------------------------------------------------------- *)
(* SSZ Serialization support *)
(* --------------------------------------------------------------------------- *)

(** Serialize a value to SSZ bytes *)
let serialize value =
  let data, _length, error_code, error_message = serialize_value value in
  if error_code = 0 then Result.Ok data
  else
    Result.Error (error_message |> Option.value ~default:"Serialization failed")

(** Deserialize SSZ bytes to a value *)
let deserialize data =
  let length = Bytes.length data in
  match deserialize_value data length with
  | Some value -> Result.Ok value
  | None -> Result.Error "Deserialization failed"

(* --------------------------------------------------------------------------- *)
(* Round-trip testing support *)
(* --------------------------------------------------------------------------- *)

(** Test round-trip serialization/deserialization for a single value *)
let test_value_roundtrip value = test_roundtrip value

(** Test round-trip for all basic value types *)
let test_comprehensive_roundtrip () = test_all_roundtrips ()

(** Test round-trip for specific value types created in OCaml *)
let test_ocaml_roundtrips () =
  let test_values =
    [
      ("unit", create_unit ())
    ; ("bool_true", create_bool true)
    ; ("bool_false", create_bool false)
    ; ("int_zero", create_int 0)
    ; ("int_positive", create_int 42)
    ; ("int_negative", create_int (-1))
    ; ("string", create_string "Hello OCaml!")
    ; ("symbol", create_symbol "test_symbol")
    ]
  in

  let results =
    List.map
      (fun (name, value) ->
        let success = test_value_roundtrip value in
        (name, success))
      test_values
  in

  let all_passed = List.for_all (fun (_, success) -> success) results in

  if all_passed then Result.Ok "All OCaml round-trip tests passed"
  else
    let failed = List.filter (fun (_, success) -> not success) results in
    let failed_names = List.map fst failed in
    Result.Error ("Failed tests: " ^ String.concat ", " failed_names)

(* --------------------------------------------------------------------------- *)
(* Resource Management API *)
(* --------------------------------------------------------------------------- *)

(** Safe wrapper for resource creation that handles exceptions *)
let safe_create_resource resource_type domain_id quantity :
    (resource_id option, causality_error) result =
  try
    let resource_handle =
      create_resource_ffi resource_type domain_id quantity
    in
    let resource_id = get_resource_id_ffi resource_handle in
    Ok (Some resource_id)
  with exn ->
    Error
      (FFIError
         ("Unexpected error in resource creation: " ^ Printexc.to_string exn))

(** Safe wrapper for resource consumption *)
let safe_consume_resource_by_id _resource_id : (bool, causality_error) result =
  (* Note: This is a simplified version. In practice, we'd need to maintain a registry
     of resource handles to look up by ID *)
  Error
    (FFIError
       "Direct resource consumption by ID not implemented - use resource handle")

(** Safe wrapper for expression compilation *)
let safe_compile_expr expr_string : (expr_id option, causality_error) result =
  try
    let expr_handle = compile_expr_ffi expr_string in
    let expr_id = get_expr_id_ffi expr_handle in
    Ok (Some expr_id)
  with exn ->
    Error
      (FFIError
         ("Unexpected error in expression compilation: "
        ^ Printexc.to_string exn))

(** Safe wrapper for intent submission *)
let safe_submit_intent name domain_id expr_string :
    (bool, causality_error) result =
  try
    let success = submit_intent_ffi name domain_id expr_string in
    Ok success
  with exn ->
    Error
      (FFIError
         ("Unexpected error in intent submission: " ^ Printexc.to_string exn))

(** Safe wrapper for getting system metrics *)
let safe_get_system_metrics () : (str_t, causality_error) result =
  try
    let metrics = get_system_metrics_ffi () in
    Ok metrics
  with exn ->
    Error
      (FFIError
         ("Unexpected error getting system metrics: " ^ Printexc.to_string exn))

(* --------------------------------------------------------------------------- *)
(* Diagnostic and utility functions *)
(* --------------------------------------------------------------------------- *)

(** Get FFI version information *)
let get_version () = get_ffi_version ()

(** Get debug information about a value *)
let debug_value value = get_debug_info value

(** Pretty print a value for debugging *)
let pretty_print value =
  let type_name =
    match get_type value with
    | Unit -> "Unit"
    | Bool -> "Bool"
    | Int -> "Int"
    | Symbol -> "Symbol"
    | String -> "String"
    | Product -> "Product"
    | Sum -> "Sum"
    | Record -> "Record"
  in
  let debug_info = debug_value value in
  Printf.sprintf "%s: %s" type_name debug_info

(* --------------------------------------------------------------------------- *)
(* Type conversion utilities *)
(* --------------------------------------------------------------------------- *)

(** Convert OCaml lisp_value to string for FFI *)
let lisp_value_to_ffi_string value =
  let rec to_sexp = function
    | Ocaml_causality_core.Unit -> "()"
    | Ocaml_causality_core.Bool true -> "true"
    | Ocaml_causality_core.Bool false -> "false"
    | Ocaml_causality_core.Int i -> Int64.to_string i
    | Ocaml_causality_core.String s -> "\"" ^ String.escaped s ^ "\""
    | Ocaml_causality_core.Symbol s -> s
    | Ocaml_causality_core.List l ->
        "(" ^ String.concat " " (List.map to_sexp l) ^ ")"
    | Ocaml_causality_core.ResourceId rid ->
        "#<resource:" ^ Bytes.to_string rid ^ ">"
    | Ocaml_causality_core.ExprId eid -> "#<expr:" ^ Bytes.to_string eid ^ ">"
    | Ocaml_causality_core.Bytes b -> "#<bytes:" ^ Bytes.to_string b ^ ">"
  in
  to_sexp value

(** Convert string from FFI back to lisp_value *)
let ffi_string_to_lisp_value s : (lisp_value, causality_error) result =
  try
    (* Simple parser for basic types - in production this would be more robust *)
    if s = "()" then Ok Ocaml_causality_core.Unit
    else if s = "true" then Ok (Ocaml_causality_core.Bool true)
    else if s = "false" then Ok (Ocaml_causality_core.Bool false)
    else if String.contains s '"' then
      let unescaped = String.sub s 1 (String.length s - 2) in
      Ok (Ocaml_causality_core.String unescaped)
    else if
      String.for_all
        (fun c -> (Char.code c >= 48 && Char.code c <= 57) || c = '-')
        s
    then Ok (Ocaml_causality_core.Int (Int64.of_string s))
    else Ok (Ocaml_causality_core.Symbol s)
  with exn ->
    Error
      (SerializationError
         ("Failed to parse lisp value: " ^ Printexc.to_string exn))

(* --------------------------------------------------------------------------- *)
(* Comprehensive FFI test suite *)
(* --------------------------------------------------------------------------- *)

let run_comprehensive_ffi_tests () =
  let version = get_version () in
  Printf.printf "FFI Version: %s\n" version;

  (* Test basic value creation and inspection *)
  let unit_val = create_unit () in
  let bool_val = create_bool true in
  let int_val = create_int 42 in
  let string_val = create_string "test" in
  let symbol_val = create_symbol "sym" in

  Printf.printf "Created values:\n";
  Printf.printf "  Unit: %s\n" (pretty_print unit_val);
  Printf.printf "  Bool: %s\n" (pretty_print bool_val);
  Printf.printf "  Int: %s\n" (pretty_print int_val);
  Printf.printf "  String: %s\n" (pretty_print string_val);
  Printf.printf "  Symbol: %s\n" (pretty_print symbol_val);

  (* Test serialization *)
  Printf.printf "\nTesting serialization:\n";
  let test_serialize name value =
    match serialize value with
    | Result.Ok data -> (
        Printf.printf "  %s: serialized to %d bytes\n" name (Bytes.length data);
        (* Test round-trip *)
        match deserialize data with
        | Result.Ok _ -> Printf.printf "  %s: deserialization successful\n" name
        | Result.Error err ->
            Printf.printf "  %s: deserialization failed: %s\n" name err)
    | Result.Error err ->
        Printf.printf "  %s: serialization failed: %s\n" name err
  in

  test_serialize "unit" unit_val;
  test_serialize "bool" bool_val;
  test_serialize "int" int_val;
  test_serialize "string" string_val;
  test_serialize "symbol" symbol_val;

  (* Test comprehensive round-trips *)
  Printf.printf "\nTesting comprehensive round-trips:\n";
  let rust_roundtrip_result = test_comprehensive_roundtrip () in
  Printf.printf "  Rust comprehensive test: %s\n"
    (if rust_roundtrip_result then "PASSED" else "FAILED");

  match test_ocaml_roundtrips () with
  | Result.Ok msg -> Printf.printf "  OCaml comprehensive test: %s\n" msg
  | Result.Error err ->
      Printf.printf "  OCaml comprehensive test: FAILED - %s\n" err;

      Printf.printf "\nFFI tests completed.\n"

(* --------------------------------------------------------------------------- *)
(* Error handling *)
(* --------------------------------------------------------------------------- *)

(** Convert Rust error codes to OCaml causality_error *)
let interpret_rust_error error_code =
  match error_code with
  | 1 -> LinearityViolation "Resource already consumed"
  | 2 -> InvalidResource (Bytes.of_string "unknown")
  | 3 -> InvalidExpression (Bytes.of_string "unknown")
  | 4 -> DomainError "Domain not found"
  | 5 -> SerializationError "Invalid data format"
  | _ -> FFIError ("Unknown Rust error code: " ^ string_of_int error_code)

(* --------------------------------------------------------------------------- *)
(* Initialization *)
(* --------------------------------------------------------------------------- *)

(** Initialize the FFI subsystem *)
let initialize_ffi () : (unit, causality_error) result =
  try Ok ()
  with exn ->
    Error (FFIError ("FFI initialization failed: " ^ Printexc.to_string exn))

(** Cleanup FFI resources *)
let cleanup_ffi () = ()
