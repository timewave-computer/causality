(** OCaml to Rust Compiler Bridge
    
    This module provides OCaml functions to call into the Rust causality-compiler
    via C FFI to compile lambda terms to register machine instructions.
*)

(** {1 C Types and External Functions} *)

(** Term type constants matching Rust *)
let term_unit = 0
let term_symbol = 1
let term_int = 2
let term_bool = 3
let term_lambda = 4
let term_apply = 5
let term_alloc = 6
let term_consume = 7

(** C compilation result structure *)
type c_compilation_result = {
  success : int;
  instructions : string array;
  instruction_count : int;
  registers_used : int;
  resource_allocations : int;
  resource_consumptions : int;
  error_message : string option;
}

(** External C functions - these call into the Rust compiler *)
external rust_compile_lambda_term_raw : 
  int -> (* term_type *)
  string -> (* data (encoded as string) *)
  c_compilation_result = "rust_compile_lambda_term_stub"

external rust_compiler_version : unit -> string = "rust_compiler_version"
external rust_test_compilation : string -> string = "rust_test_compilation"
external rust_free_string : string -> unit = "rust_free_string"

(** {1 OCaml Lambda Term Types} *)

(** OCaml representation of lambda terms *)
type lambda_term = 
  | Unit
  | Symbol of string
  | Int of int
  | Bool of bool
  | Lambda of string * lambda_term
  | Apply of lambda_term * lambda_term
  | Alloc of lambda_term
  | Consume of lambda_term

(** Compilation result *)
type compilation_result = {
  success : bool;
  instructions : string list;
  registers_used : int;
  resource_allocations : int;
  resource_consumptions : int;
  error_message : string option;
}

(** {1 Term Encoding for C FFI} *)

(** Encode a lambda term as a string for C transfer *)
let rec encode_term (term : lambda_term) : (int * string) =
  match term with
  | Unit -> (term_unit, "")
  | Symbol s -> (term_symbol, s)
  | Int i -> (term_int, string_of_int i)
  | Bool b -> (term_bool, if b then "1" else "0")
  | Lambda (var, body) ->
    let (body_type, body_data) = encode_term body in
    (term_lambda, Printf.sprintf "%s|%d|%s" var body_type body_data)
  | Apply (func, arg) ->
    let (func_type, func_data) = encode_term func in
    let (arg_type, arg_data) = encode_term arg in
    (term_apply, Printf.sprintf "%d|%s|%d|%s" func_type func_data arg_type arg_data)
  | Alloc expr ->
    let (expr_type, expr_data) = encode_term expr in
    (term_alloc, Printf.sprintf "%d|%s" expr_type expr_data)
  | Consume expr ->
    let (expr_type, expr_data) = encode_term expr in
    (term_consume, Printf.sprintf "%d|%s" expr_type expr_data)

(** {1 High-Level Interface} *)

(** Convert C result to OCaml result *)
let convert_c_result (c_result : c_compilation_result) : compilation_result =
  {
    success = (c_result.success <> 0);
    instructions = Array.to_list c_result.instructions;
    registers_used = c_result.registers_used;
    resource_allocations = c_result.resource_allocations;
    resource_consumptions = c_result.resource_consumptions;
    error_message = c_result.error_message;
  }

(** Compile a lambda term using the Rust compiler *)
let compile_term (term : lambda_term) : compilation_result =
  try
    let (term_type, term_data) = encode_term term in
    let c_result = rust_compile_lambda_term_raw term_type term_data in
    convert_c_result c_result
  with
  | e -> {
      success = false;
      instructions = [];
      registers_used = 0;
      resource_allocations = 0;
      resource_consumptions = 0;
      error_message = Some (Printexc.to_string e);
    }

(** {1 Convenience Functions} *)

(** Term builder functions *)
module TermBuilder = struct
  let unit () = Unit
  let symbol name = Symbol name
  let int value = Int value
  let bool value = Bool value
  let lambda var body = Lambda (var, body)
  let apply func arg = Apply (func, arg)
  let alloc expr = Alloc expr
  let consume expr = Consume expr
  
  (** Build a simple lambda that returns a constant *)
  let const_lambda var value = 
    Lambda (var, Int value)
  
  (** Build an allocation of a constant *)
  let alloc_int value = 
    Alloc (Int value)
  
  (** Build function application with integer argument *)
  let apply_int func_name arg_value =
    Apply (Symbol func_name, Int arg_value)
    
  (** Build a simple identity function *)
  let identity () =
    Lambda ("x", Symbol "x")
    
  (** Build a constant function *)
  let const_func value =
    Lambda ("_", Int value)
end

(** {1 Test Interface} *)

(** Test compiler connection *)
let test_compiler_connection () : bool =
  try
    let version = rust_compiler_version () in
    Printf.printf "Connected to Rust compiler: %s\n" version;
    true
  with
  | _ -> 
    Printf.printf "Failed to connect to Rust compiler\n";
    false

(** Test simple compilation *)
let test_simple_compilation () : compilation_result =
  let term = TermBuilder.alloc_int 42 in
  compile_term term

(** Test compilation with error handling *)
let test_compilation_with_error_handling () : unit =
  let test_cases = [
    ("Unit", TermBuilder.unit ());
    ("Integer", TermBuilder.int 123);
    ("Symbol", TermBuilder.symbol "test");
    ("Alloc", TermBuilder.alloc_int 42);
    ("Lambda", TermBuilder.identity ());
    ("Application", TermBuilder.apply_int "f" 10);
  ] in
  
  List.iter (fun (name, term) ->
    Printf.printf "Testing %s: " name;
    let result = compile_term term in
    if result.success then
      Printf.printf "SUCCESS (%d instructions, %d registers)\n"
        (List.length result.instructions)
        result.registers_used
    else
      Printf.printf "FAILED: %s\n"
        (match result.error_message with
         | Some msg -> msg
         | None -> "Unknown error")
  ) test_cases

(** {1 Advanced Examples} *)

(** Example: compile a simple program *)
let example_simple_program () : compilation_result =
  (* (alloc (lambda (x) x)) *)
  let identity = TermBuilder.identity () in
  let program = TermBuilder.alloc identity in
  compile_term program

(** Example: compile nested application *)
let example_nested_application () : compilation_result =
  (* ((lambda (f) (f 42)) (lambda (x) x)) *)
  let identity = TermBuilder.identity () in
  let apply_to_42 = Lambda ("f", Apply (Symbol "f", Int 42)) in
  let program = Apply (apply_to_42, identity) in
  compile_term program 