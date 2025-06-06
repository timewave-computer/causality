(* ------------ FOREIGN FUNCTION INTERFACE ------------ *)
(* Purpose: Rust FFI bindings and C stubs *)

open Ocaml_causality_core

(* Abstract type for Causality values *)
type causality_value

(* Value type enumeration matching Rust FFI *)
type value_type =
  | Unit
  | Bool
  | Int
  | Symbol
  | String
  | Product
  | Sum
  | Record

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
  data: bytes option;
  length: int;
  error_code: int;
  error_message: string option;
}

(* External C function declarations for basic value operations *)
external create_unit : unit -> causality_value = "ocaml_causality_value_unit"
external create_bool : bool -> causality_value = "ocaml_causality_value_bool"
external create_int : int -> causality_value = "ocaml_causality_value_int"
external create_string : string -> causality_value = "ocaml_causality_value_string"
external create_symbol : string -> causality_value = "ocaml_causality_value_symbol"
external free_value : causality_value -> unit = "ocaml_causality_value_free"

(* External C function declarations for value inspection *)
external get_value_type_int : causality_value -> int = "ocaml_causality_value_type"
external extract_bool : causality_value -> int = "ocaml_causality_value_as_bool"
external extract_int : causality_value -> int = "ocaml_causality_value_as_int"
external extract_string_ptr : causality_value -> string option = "ocaml_causality_value_as_string"

(* External C function declarations for SSZ serialization *)
external serialize_value : causality_value -> bytes * int * int * string option = "ocaml_causality_value_serialize"
external deserialize_value : bytes -> int -> causality_value option = "ocaml_causality_value_deserialize"
external free_serialized_data : bytes -> int -> unit = "ocaml_causality_free_serialized_data"

(* External C function declarations for round-trip testing *)
external test_roundtrip : causality_value -> bool = "ocaml_causality_test_roundtrip"
external test_all_roundtrips : unit -> bool = "ocaml_causality_test_all_roundtrips"

(* External C function declarations for diagnostics *)
external get_ffi_version : unit -> string = "ocaml_causality_ffi_version"
external get_debug_info : causality_value -> string = "ocaml_causality_value_debug_info"

(* High-level OCaml interface *)

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

(** Extract integer value (returns None if not an integer, Some 0 could be ambiguous) *)
let as_int value =
  match get_type value with
  | Int -> Some (extract_int value)
  | _ -> None

(** Extract string value (returns None if not a string) *)
let as_string value =
  match get_type value with
  | String | Symbol -> extract_string_ptr value
  | _ -> None

(** SSZ Serialization support *)

(** Serialize a value to SSZ bytes *)
let serialize value =
  let (data, length, error_code, error_message) = serialize_value value in
  if error_code = 0 then
    Result.Ok data
  else
    Result.Error (error_message |> Option.value ~default:"Serialization failed")

(** Deserialize SSZ bytes to a value *)
let deserialize data =
  let length = Bytes.length data in
  match deserialize_value data length with
  | Some value -> Result.Ok value
  | None -> Result.Error "Deserialization failed"

(** Round-trip testing support *)

(** Test round-trip serialization/deserialization for a single value *)
let test_value_roundtrip value = test_roundtrip value

(** Test round-trip for all basic value types *)
let test_comprehensive_roundtrip () = test_all_roundtrips ()

(** Test round-trip for specific value types created in OCaml *)
let test_ocaml_roundtrips () =
  let test_values = [
    ("unit", create_unit ());
    ("bool_true", create_bool true);
    ("bool_false", create_bool false);
    ("int_zero", create_int 0);
    ("int_positive", create_int 42);
    ("int_negative", create_int (-1));
    ("string", create_string "Hello OCaml!");
    ("symbol", create_symbol "test_symbol");
  ] in
  
  let results = List.map (fun (name, value) ->
    let success = test_value_roundtrip value in
    (name, success)
  ) test_values in
  
  let all_passed = List.for_all (fun (_, success) -> success) results in
  
  if all_passed then
    Result.Ok "All OCaml round-trip tests passed"
  else
    let failed = List.filter (fun (_, success) -> not success) results in
    let failed_names = List.map fst failed in
    Result.Error ("Failed tests: " ^ String.concat ", " failed_names)

(** Diagnostic and utility functions *)

(** Get FFI version information *)
let get_version () = get_ffi_version ()

(** Get debug information about a value *)
let debug_value value = get_debug_info value

(** Pretty print a value for debugging *)
let pretty_print value =
  let type_name = match get_type value with
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

(** Comprehensive FFI test suite *)
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
    | Result.Ok data ->
      Printf.printf "  %s: serialized to %d bytes\n" name (Bytes.length data);
      (* Test round-trip *)
      (match deserialize data with
       | Result.Ok _ -> Printf.printf "  %s: deserialization successful\n" name
       | Result.Error err -> Printf.printf "  %s: deserialization failed: %s\n" name err)
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
  | Result.Error err -> Printf.printf "  OCaml comprehensive test: FAILED - %s\n" err;
  
  Printf.printf "\nFFI tests completed.\n"

(* ------------ LEGACY PLACEHOLDER FFI FUNCTIONS ------------ *)
(* TODO: Replace with actual external declarations when C bindings are available *)

(* ------------ SAFETY WRAPPERS ------------ *)

(** Safe wrapper for resource creation that handles exceptions *)
let safe_create_resource resource_type domain_id quantity : (Ocaml_causality_core.resource_id option, Ocaml_causality_core.causality_error) result =
  try
    (* TODO: Replace with actual FFI call *)
    let _ = (resource_type, domain_id, quantity) in
    Ok (Some (Bytes.of_string "mock_resource_id"))
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("Unexpected error in resource creation: " ^ Printexc.to_string exn))

(** Safe wrapper for resource consumption *)
let safe_consume_resource resource_id : (bool, Ocaml_causality_core.causality_error) result =
  try
    (* TODO: Replace with actual FFI call *)
    let _ = resource_id in
    Ok true
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("Unexpected error in resource consumption: " ^ Printexc.to_string exn))

(** Safe wrapper for expression compilation *)
let safe_compile_expr expr_string : (Ocaml_causality_core.expr_id option, Ocaml_causality_core.causality_error) result =
  try
    (* TODO: Replace with actual FFI call *)
    let expr_bytes = Bytes.of_string expr_string in
    Ok (Some expr_bytes)
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("Unexpected error in expression compilation: " ^ Printexc.to_string exn))

(** Safe wrapper for intent submission *)
let safe_submit_intent name domain_id expr_string : (bool, Ocaml_causality_core.causality_error) result =
  try
    (* TODO: Replace with actual FFI call *)
    let _ = (name, domain_id, expr_string) in
    Ok true
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("Unexpected error in intent submission: " ^ Printexc.to_string exn))

(** Safe wrapper for getting system metrics *)
let safe_get_system_metrics () : (Ocaml_causality_core.str_t, Ocaml_causality_core.causality_error) result =
  try
    (* TODO: Replace with actual FFI call *)
    Ok "Mock system metrics"
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("Unexpected error getting system metrics: " ^ Printexc.to_string exn))

(* ------------ TYPE CONVERSION UTILITIES ------------ *)

(** Convert OCaml lisp_value to string for FFI *)
let lisp_value_to_ffi_string value =
  let rec to_sexp = function
    | Ocaml_causality_core.Unit -> "()"
    | Ocaml_causality_core.Bool true -> "true"
    | Ocaml_causality_core.Bool false -> "false"
    | Ocaml_causality_core.Int i -> Int64.to_string i
    | Ocaml_causality_core.String s -> "\"" ^ String.escaped s ^ "\""
    | Ocaml_causality_core.Symbol s -> s
    | Ocaml_causality_core.List l -> "(" ^ String.concat " " (List.map to_sexp l) ^ ")"
    | Ocaml_causality_core.ResourceId rid -> "#<resource:" ^ Bytes.to_string rid ^ ">"
    | Ocaml_causality_core.ExprId eid -> "#<expr:" ^ Bytes.to_string eid ^ ">"
    | Ocaml_causality_core.Bytes b -> "#<bytes:" ^ Bytes.to_string b ^ ">"
  in
  to_sexp value

(** Convert string from FFI back to lisp_value *)
let ffi_string_to_lisp_value s : (Ocaml_causality_core.lisp_value, Ocaml_causality_core.causality_error) result =
  try
    (* Simple parser for basic types - in production this would be more robust *)
    if s = "()" then Ok Ocaml_causality_core.Unit
    else if s = "true" then Ok (Ocaml_causality_core.Bool true)
    else if s = "false" then Ok (Ocaml_causality_core.Bool false)
    else if String.contains s '"' then
      let unescaped = String.sub s 1 (String.length s - 2) in
      Ok (Ocaml_causality_core.String unescaped)
    else if String.for_all (fun c -> Char.code c >= 48 && Char.code c <= 57 || c = '-') s then
      Ok (Ocaml_causality_core.Int (Int64.of_string s))
    else
      Ok (Ocaml_causality_core.Symbol s)
  with
  | exn -> Error (Ocaml_causality_core.SerializationError ("Failed to parse lisp value: " ^ Printexc.to_string exn))

(* ------------ MEMORY MANAGEMENT ------------ *)

(** Resource ID handle registry to track valid resource IDs *)
module ResourceRegistry = struct
  let valid_resources = Hashtbl.create 1024

  let register_resource (id: Ocaml_causality_core.resource_id) =
    Hashtbl.replace valid_resources id true

  let invalidate_resource (id: Ocaml_causality_core.resource_id) =
    Hashtbl.remove valid_resources id

  let is_valid_resource (id: Ocaml_causality_core.resource_id) =
    Hashtbl.mem valid_resources id

  let get_valid_resources () =
    Hashtbl.fold (fun id _ acc -> id :: acc) valid_resources []
end

(** Expression ID registry *)
module ExprRegistry = struct
  let registered_exprs = Hashtbl.create 1024

  let register_expr (id: Ocaml_causality_core.expr_id) (name: Ocaml_causality_core.str_t) =
    Hashtbl.replace registered_exprs name id

  let lookup_expr (name: Ocaml_causality_core.str_t) : Ocaml_causality_core.expr_id option =
    Hashtbl.find_opt registered_exprs name

  let get_registered_exprs () =
    Hashtbl.fold (fun name id acc -> (name, id) :: acc) registered_exprs []
end

(* ------------ ERROR HANDLING ------------ *)

(** Convert Rust error codes to OCaml causality_error *)
let interpret_rust_error error_code =
  match error_code with
  | 1 -> Ocaml_causality_core.LinearityViolation "Resource already consumed"
  | 2 -> Ocaml_causality_core.InvalidResource (Bytes.of_string "unknown")
  | 3 -> Ocaml_causality_core.InvalidExpression (Bytes.of_string "unknown")
  | 4 -> Ocaml_causality_core.DomainError "Domain not found"
  | 5 -> Ocaml_causality_core.SerializationError "Invalid data format"
  | _ -> Ocaml_causality_core.FFIError ("Unknown Rust error code: " ^ string_of_int error_code)

(** Log FFI operations for debugging *)
let log_ffi_call operation result =
  if !Sys.interactive then
    Printf.printf "[FFI] %s -> %s\n" operation result

(* ------------ INITIALIZATION ------------ *)

(** Initialize the FFI subsystem *)
let initialize_ffi () : (unit, Ocaml_causality_core.causality_error) result =
  try
    (* Initialize resource and expression registries *)
    Hashtbl.clear ResourceRegistry.valid_resources;
    Hashtbl.clear ExprRegistry.registered_exprs;
    Ok ()
  with
  | exn -> Error (Ocaml_causality_core.FFIError ("FFI initialization failed: " ^ Printexc.to_string exn))

(** Cleanup FFI resources *)
let cleanup_ffi () =
  Hashtbl.clear ResourceRegistry.valid_resources;
  Hashtbl.clear ExprRegistry.registered_exprs 