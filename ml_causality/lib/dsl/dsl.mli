(**
 * Domain Specific Language (DSL) for Temporal Effect Language (TEL)
 *
 * Interface for the Lisp DSL, providing functions to construct
 * and manipulate Lisp expressions for the TEL system.
 *)

open Ml_causality_lib_types.Types

(*-----------------------------------------------------------------------------
 * Atom Builders - Basic expression constructors
 *---------------------------------------------------------------------------*)

(** [sym name] creates a variable (symbol) expression. E.g., `sym "my-var"` -> `my-var` *)
val sym : string -> expr

(** [str_lit s] creates a string literal expression. E.g., `str_lit "hello"` -> `"hello"` *)
val str_lit : string -> expr

(** [int_lit i] creates an integer literal expression. E.g., `int_lit 123` -> `123` *)
val int_lit : int64 -> expr

(** [bool_lit b] creates a boolean literal expression. E.g., `bool_lit true` -> `true` *)
val bool_lit : bool -> expr

(** [nil_lit] creates a nil/null literal expression. E.g., `nil_lit` -> `nil` *)
val nil_lit : expr

(** [keyword_lit k_name] creates a keyword literal (often represented as a special string). E.g., `keyword_lit "my-key"` -> `":my-key"` *)
val keyword_lit : string -> expr

(*-----------------------------------------------------------------------------
 * Constant Value Expression
 *---------------------------------------------------------------------------*)

(** [const v_expr] creates a constant value expression, wrapping a `value_expr`. *)
val const : value_expr -> expr

(*-----------------------------------------------------------------------------
 * Core Structure Builders
 *---------------------------------------------------------------------------*)

(** 
 * [lambda params body] creates a lambda expression.
 * E.g., `lambda ["x"] (sym "x")` -> `(lambda (x) x)`
 *)
val lambda : string list -> expr -> expr

(** 
 * [apply func args] creates a function application expression.
 * E.g., `apply (sym "+") [int_lit 1; int_lit 2]` -> `(+ 1 2)`
 *)
val apply : expr -> expr list -> expr

(** 
 * [dynamic steps expr_val] creates a dynamic expression with a step bound.
 *)
val dynamic : int -> expr -> expr

(*-----------------------------------------------------------------------------
 * Combinator Applications
 *---------------------------------------------------------------------------*)

(** 
 * [list_ items] creates a list expression.
 * E.g., `list_ [int_lit 1; sym "a"]` -> `(list 1 a)`
 *)
val list_ : expr list -> expr

(** [if_ cond then_branch else_branch] creates an if expression. E.g., `if_ (bool_lit true) (int_lit 1) (int_lit 0)` -> `(if true 1 0)`*)
val if_ : expr -> expr -> expr -> expr

(** Represents a binding for `let*`. A pair of variable name (string) and its value expression. *)
type binding = string * expr

(** [let_star bindings body_exprs] creates a `let*` expression for sequential bindings.
    Example: `let_star [("x", int_lit 1)] [apply (sym "+") [sym "x"; int_lit 2]]`
    -> `(let* ((x 1)) (+ x 2))` *)
val let_star : binding list -> expr list -> expr

(** [define symbol_name value_expr] creates a define expression.
    Example: `define "my-val" (int_lit 10)` -> `(define my-val 10)` *)
val define : string -> expr -> expr

(** [defun name params body] creates a defun expression (function definition).
    Example: `defun "my-func" ["x"] (sym "x")` -> `(defun my-func (x) x)` *)
val defun : string -> string list -> expr -> expr

(** [quote data_expr] creates a quote expression. E.g., `quote (list_ [sym "a"])` -> `(quote (a))` *)
val quote : expr -> expr

(** [make_map pairs] creates a map construction expression. 
    `pairs` is a list of key-expression/value-expression tuples.
    Example: `make_map [(str_lit "key1", int_lit 1); (str_lit "key2", bool_lit true)]`
    -> `(make-map (list (list "key1" 1) (list "key2" true)))` *)
val make_map : (expr * expr) list -> expr

(* -- Logical Operations -- *)

(** [and_ exprs] -> `(and expr1 expr2 ...)` *)
val and_ : expr list -> expr

(** [or_ exprs] -> `(or expr1 expr2 ...)` *)
val or_ : expr list -> expr

(** [not_ e] -> `(not e)` *)
val not_ : expr -> expr

(* -- Equality -- *)

(** [eq e1 e2] -> `(eq e1 e2)` *)
val eq : expr -> expr -> expr

(* -- Arithmetic Operations -- *)

(** [add e1 e2] -> `(+ e1 e2)` *)
val add : expr -> expr -> expr

(** [sub e1 e2] -> `(- e1 e2)` *)
val sub : expr -> expr -> expr

(** [mul e1 e2] -> `(multiply e1 e2)` *)
val mul : expr -> expr -> expr

(** [div e1 e2] -> `(/ e1 e2)` *)
val div : expr -> expr -> expr

(* -- Comparison Operations -- *)

(** [gt e1 e2] -> `(> e1 e2)` *)
val gt : expr -> expr -> expr

(** [lt e1 e2] -> `(< e1 e2)` *)
val lt : expr -> expr -> expr

(** [gte e1 e2] -> `(>= e1 e2)` *)
val gte : expr -> expr -> expr

(** [lte e1 e2] -> `(<= e1 e2)` *)
val lte : expr -> expr -> expr

(* -- Data Access & Context -- *)

(** [get_context_value key_expr] -> `(get-context-value key_expr)` *)
val get_context_value : expr -> expr

(** [get_field target field_name] -> `(get-field target_expr field_name_expr)` *)
val get_field : expr -> expr -> expr

(** [completed effect_ref_expr] -> `(completed effect_ref_expr)` *)
val completed : expr -> expr

(* -- List Operations -- *)

(** [nth index list_expr] -> `(nth index_expr list_expr)` *)
val nth : expr -> expr -> expr

(** [length list_expr] -> `(length list_expr)` *)
val length : expr -> expr

(** [cons item list_expr] -> `(cons item_expr list_expr)` *)
val cons : expr -> expr -> expr

(** [car list_expr] -> `(car list_expr)` (first element) *)
val car : expr -> expr

(** [cdr list_expr] -> `(cdr list_expr)` (rest of the list) *)
val cdr : expr -> expr

(* -- Map Operations -- *)

(** [map_get key map_expr] -> `(map-get key_expr map_expr)` *)
val map_get : expr -> expr -> expr

(** [map_has_key key map_expr] -> `(map-has-key? key_expr map_expr)` *)
val map_has_key : expr -> expr -> expr

(* --- SKI C Combinators --- *)

(** [s_ ()] -> `s` (the S combinator symbol) *)
val s_ : unit -> expr

(** [k_ ()] -> `k` (the K combinator symbol) *)
val k_ : unit -> expr

(** [i_ ()] -> `i` (the I combinator symbol) *)
val i_ : unit -> expr

(** [c_ ()] -> `c` (the C combinator symbol) *)
val c_ : unit -> expr

(* --- Helpers for creating value expressions --- *)

(** [vint i] creates an integer value expression. *)
val vint : int64 -> value_expr

(** [vbool b] creates a boolean value expression. *)
val vbool : bool -> value_expr

(** [vstr s] creates a string value expression. *)
val vstr : string -> value_expr

(** [vnil] creates a nil value expression. *)
val vnil : value_expr

(** [vlist items] creates a list value expression. *)
val vlist : value_expr list -> value_expr

(** [vmap entries] creates a map value expression. *)
val vmap : (string * value_expr) list -> value_expr

(*-----------------------------------------------------------------------------
  ID Generation
-----------------------------------------------------------------------------*)

(** [lisp_code_to_expr_id lisp_code_string] generates a content-addressed ID 
    from a Lisp code string. *)
val lisp_code_to_expr_id : string -> expr_id

(** [value_expr_to_id ve] generates a content-addressed ID from a `value_expr`. *)
val value_expr_to_id : value_expr -> value_expr_id

(* TODO: The following sections reference types that don't exist yet in Types module
   Commenting out until the types are properly defined

(*-----------------------------------------------------------------------------
  TEL Graph Construction Entry Points
-----------------------------------------------------------------------------*)

(**
 * Defines an effect and translates it into a TEL effect resource.
 * This function will likely interact with the PPX to capture
 * the OCaml effect definition and any associated static logic.
 *
 * @param name The name of the OCaml effect.
 * @param payload_value_id The ID of the ValueExpr representing the effect's payload structure.
 * @param static_logic_expr_id Optional ID of the Expr for static validation logic.
 * @param domain_id The domain this effect belongs to.
 * @return The generated TelEffectResource or an error.
 *)
val define_tel_effect_resource :
  name:Ml_causality_lib_types.Types.str ->
  payload_value_id:Ml_causality_lib_types.Types.value_expr_id ->
  static_logic_expr_id:Ml_causality_lib_types.Types.expr_id option ->
  domain_id:Ml_causality_lib_types.Types.domain_id ->
  (Ml_causality_lib_types.Types.tel_effect_resource, string) result

(**
 * Defines an effect handler and translates it into a TEL handler resource.
 * This function will interact with the PPX to capture the OCaml handler
 * definition, its configuration, static validation, and dynamic logic.
 *
 * @param name The name of the OCaml handler.
 * @param config_value_id The ID of the ValueExpr representing the handler's configuration.
 * @param static_logic_expr_id Optional ID of the Expr for static config validation.
 * @param dynamic_logic_expr_id ID of the Expr for the handler's core dynamic logic.
 * @param domain_id The domain this handler belongs to.
 * @return The generated TelHandlerResource or an error.
 *)
val define_tel_handler_resource :
  name:Ml_causality_lib_types.Types.str ->
  config_value_id:Ml_causality_lib_types.Types.value_expr_id ->
  static_logic_expr_id:Ml_causality_lib_types.Types.expr_id option ->
  dynamic_logic_expr_id:Ml_causality_lib_types.Types.expr_id ->
  domain_id:Ml_causality_lib_types.Types.domain_id ->
  (Ml_causality_lib_types.Types.tel_handler_resource, string) result

*)

(*-----------------------------------------------------------------------------
  Generic AST Compilation Framework
-----------------------------------------------------------------------------*)

(** Generic argument types for function compilation *)
type argument = 
  | String of string
  | Variable of string
  | Value of value_expr
  | Call of function_call
  | List of argument list

(** Structured function call representation *)
and function_call = {
  name: string;
  args: argument list;
}

(** Structured function definition *)
type function_def = {
  name: string;
  params: string list;
  body: function_call;
}

(** Structured let binding *)
type let_binding = {
  var_name: string;
  value: argument;
}

(** Structured conditional *)
type conditional = {
  condition: argument;
  then_branch: argument;
  else_branch: argument;
}

(** Generic compilation from structured representation to AST *)
val compile_argument : argument -> expr

val compile_function_call : function_call -> expr

(** Compile function definition to AST *)
val compile_function_def : function_def -> expr

(** Compile let bindings to AST *)
val compile_let_bindings : let_binding list -> argument -> expr

(** Compile conditional to AST *)
val compile_conditional : conditional -> expr

(** Helper to create function calls *)
val call : string -> argument list -> function_call

(** Helper to create string arguments *)
val arg_str : string -> argument

(** Helper to create variable arguments *)
val arg_var : string -> argument

(** Helper to create value arguments *)
val arg_val : value_expr -> argument

(** Helper to create call arguments *)
val arg_call : function_call -> argument

(** Helper to create list arguments *)
val arg_list : argument list -> argument

(* Additional functions might be needed for defining generic resources,
   linking nodes with edges, and serializing the graph.
   These can be added as we progress through the ocaml_effect_system.md plan. *)

(*-----------------------------------------------------------------------------
 * Content-Addressed Effect Mapping Functions
 *---------------------------------------------------------------------------*)

(** Store mapping from OCaml effect name to TEL effect ID using content-addressed storage *)
val store_effect_mapping : string -> string -> unit

(** Retrieve TEL effect ID by OCaml effect name from content-addressed storage *)
val get_effect_id_by_name : string -> string option