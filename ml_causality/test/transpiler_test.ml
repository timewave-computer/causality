(* lib/transpiler_test.ml *)

[@@@ocaml.warning "-32"] (* Suppress unused value warnings *)
[@@@ocaml.warning "-69"] (* Suppress unused field warnings *)

type simple_record = {
  a : int;
  b : string;
}

type variant_type =
  | NoArg
  | OneArg of int
  | TwoArgs of int * string

let[@tel_static "static_int_val_key"] static_int_val = 42
let[@tel_static "static_string_val_key"] static_string_val = "hello ppx"
let[@tel_static "static_true_val_key"] static_true_val = true
let[@tel_static "static_false_val_key"] static_false_val = false
let[@tel_static "static_unit_val_key"] static_unit_val = ()

let[@tel_static "static_ident_val_key"] static_ident_val = static_int_val

let[@tel_static "static_add_op_key"] static_add_op = 1 + 2
let[@tel_static "static_custom_op_key"] static_custom_op x = x * static_int_val (* Note: function def itself isn't transpiled yet *)

let[@tel_static "static_let_binding_key"] static_let_binding =
  let x = 10 in
  let y = 20 in
  x + y

let[@tel_static "static_if_else_key"] static_if_else =
  if static_true_val then "true branch" else "false branch"

let[@tel_static "static_if_no_else_key"] static_if_no_else =
  if static_false_val then () 

let[@tel_static "static_record_create_key"] static_record_create =
  { a = 100; b = "record string" }

let[@tel_static "static_record_access_key"] static_record_access =
  static_record_create.a

let[@tel_static "static_tuple_val_key"] static_tuple_val = (10, "tuple_string", true)

let[@tel_static "static_variant_no_arg_key"] static_variant_no_arg = NoArg
let[@tel_static "static_variant_one_arg_key"] static_variant_one_arg = OneArg 77
let[@tel_static "static_variant_two_args_key"] static_variant_two_args = TwoArgs (88, "variant")

let[@tel_static "static_match_simple_key"] static_match_simple =
  match static_int_val with
  | 0 -> "zero"
  | 42 -> "forty-two"
  | _ -> "other"

let[@tel_static "static_match_variant_key"] static_match_variant =
  match static_variant_one_arg with
  | NoArg -> 0
  | OneArg x -> x
  | TwoArgs (y, _) -> y

let[@tel_static "static_match_tuple_key"] static_match_tuple =
  match static_tuple_val with
  | (x, "tuple_string", _) -> x
  | _ -> -1
  
let[@tel_static "static_match_with_guard_key"] static_match_with_guard =
  match static_int_val with
  | x when x > 100 -> "greater than 100"
  | x when x > 10 -> "greater than 10"
  | _ -> "ten or less"

(* Example of a dynamic value whose body will be registered *)
let[@tel_dynamic "dynamic_example_op_key"] dynamic_example_op (x:int) (y:int) : int =
  x + y + static_int_val

(* Example of a value with a Lisp body *)
[@@@ocaml.warning "-32"] (* To suppress unused value warning for the string literal *)
let[@tel_lisp "my_lisp_key_1"] _registered_lisp_code_1 = "(define (lisp-add a b) (+ a b))"
