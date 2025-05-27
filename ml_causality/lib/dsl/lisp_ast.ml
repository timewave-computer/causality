(** 
 * Lisp Abstract Syntax Tree operations and S-expression serialization
 * 
 * This module provides functionality for working with Lisp-style expressions,
 * primarily focusing on serializing AST nodes to S-expression strings for
 * debugging, persistence, and interoperation with the Lisp runtime.
 *)

open Ml_causality_lib_types.Types  (* Core type definitions *)
open Sexplib0.Sexp                 (* S-expression utilities *)

(** Type alias for expr from Types module *)
type t = expr

(** 
 * Convert an S-expression to a human-readable string
 *
 * @param s The S-expression to convert
 * @return A string representation of the S-expression
 *)
let sexp_to_string s = to_string_hum s

(** 
 * Map atomic combinators to their Lisp symbol strings
 *
 * @param combinator The atomic combinator to convert
 * @return The string representation of the combinator in Lisp syntax
 *)
let atomic_combinator_to_symbol_string combinator =
  match combinator with
  (* Data structure operations *)
  | Ml_causality_lib_types.Types.List -> "list"
  | MakeMap -> "make-map"
  | GetField -> "get-field"
  | Length -> "length"
  
  (* Comparison operations *)
  | Eq -> "eq"
  | Lt -> "lt"
  | Gt -> "gt"
  | Gte -> ">="
  | Lte -> "<="
  
  (* Arithmetic operations *)
  | Add -> "+"
  | Sub -> "-"
  | Mul -> "*"
  | Div -> "/"
  
  (* Logic operations *)
  | And -> "and"
  | Or -> "or"
  | Not -> "not"
  
  (* Control flow *)
  | If -> "if"
  | Let -> "let*"  (* OCaml uses let*, Rust uses Let *)
  | Define -> "define"
  | Defun -> "defun"
  | Quote -> "quote"
  
  (* SKI calculus combinators *)
  | S -> "s"
  | K -> "k"
  | I -> "i"
  | C -> "c"
  
  (* Context and effect operations *)
  | GetContextValue -> "get-context-value"
  | Completed -> "completed"
  
  (* List operations *)
  | Nth -> "nth"
  | Cons -> "cons"
  | Car -> "car"
  | Cdr -> "cdr"
  
  (* Map operations *)
  | MapGet -> "map-get"
  | MapHasKey -> "map-has-key?"

(**
 * Convert an atom to an S-expression
 *
 * @param atom The atom to convert
 * @return The S-expression representation of the atom
 *)
let atom_to_sexp = function
  | Integer i -> Atom (Int64.to_string i)
  | String s -> Atom s        (* String literals are represented directly *)
  | Boolean b -> Atom (Bool.to_string b)
  | Nil -> Atom "nil"         (* nil represents absence of a value *)

(**
 * Convert a value expression to an S-expression
 *
 * This function handles all types of values that can appear in the Lisp 
 * environment, including complex structures like maps and lambdas.
 *
 * @param v_expr The value expression to convert
 * @return The S-expression representation of the value
 *)
let rec value_expr_to_sexp v_expr =
  match v_expr with
  | Unit -> Atom "nil"            (* Unit is represented as nil *)
  | VNil -> Atom "nil"            (* Explicit nil value *)
  | Bool b -> Atom (Bool.to_string b)
  | VString s -> Atom s
  
  (* Numeric value formatting *)
  | VNumber num -> 
      (match num with
      | NInteger i -> 
          (* Integer representation *)
          Atom (Int64.to_string i)
      | NFixed (value, scale) -> 
          (* Fixed-point decimal representation 
             e.g., (123, 2) -> 1.23 *)
          let scale_factor = float_of_int (int_of_float (10.0 ** float_of_int scale)) in
          let decimal_value = (Int64.to_float value) /. scale_factor in
          Atom (string_of_float decimal_value)
      | NRatio { numerator; denominator } ->
          (* Rational number representation as (/ num denom) *)
          List [Atom "/"; 
                Atom (Int64.to_string numerator); 
                Atom (Int64.to_string denominator)])
  
  (* Container types *)
  | VList vs -> 
      (* List of values *)
      List (List.map value_expr_to_sexp vs)
  | VMap kvs ->
      (* Map represented as (make-map (key1 val1) (key2 val2) ...) *)
      List (Atom "make-map" :: 
            List.map (fun (k, v) -> List [Atom k; value_expr_to_sexp v]) kvs)
  | VRecord kvs -> 
      (* Record represented as (make-record (field1 val1) (field2 val2) ...) *)
      List (Atom "make-record" :: 
            List.map (fun (k, v) -> List [Atom k; value_expr_to_sexp v]) kvs)
  
  (* Reference types *)
  | Ref (Value value_id) -> 
      (* Reference to another value *)
      List [Atom "ref-value"; Atom value_id]
  | Ref (Expr expr_id) -> 
      (* Reference to an expression *)
      List [Atom "ref-expr"; Atom expr_id]
  
  (* Lambda with captured environment *)
  | Lambda { params; body_expr_id; captured_env } ->
      List [
        Atom "lambda";
        List (List.map (fun p -> Atom p) params);  (* Parameter list *)
        Atom body_expr_id;                         (* Body reference *)
        (* Captured environment variables *)
        List (Atom "env" :: 
              List.map (fun (k,v) -> List [Atom k; value_expr_to_sexp v]) 
                       captured_env)
      ]

(**
 * Convert an expression to an S-expression
 *
 * This handles all expression types that can appear in Lisp code.
 *
 * @param e The expression to convert
 * @return The S-expression representation of the expression
 *)
let rec expr_to_sexp e =
  match e with
  | EAtom a -> 
      (* Atomic literals (numbers, strings, booleans, nil) *)
      atom_to_sexp a
  | Const v -> 
      (* Embedded value expressions *)
      value_expr_to_sexp v
  | Var s -> 
      (* Variable references *)
      Atom s
  | ELambda (params, body) ->
      (* Lambda expressions: (lambda (param1 param2...) body) *)
      List [Atom "lambda"; 
            List (List.map (fun s -> Atom s) params); 
            expr_to_sexp body]
  | Apply (func, args) ->
      (* Function application: (func arg1 arg2...) *)
      List (expr_to_sexp func :: List.map expr_to_sexp args)
  | Combinator c -> 
      (* Built-in combinators (like +, -, if, etc.) *)
      Atom (atomic_combinator_to_symbol_string c)
  | Dynamic (n, expr_val) ->
      (* Expressions with step limits: (dynamic 100 expr) *)
      List [Atom "dynamic"; Atom (Int.to_string n); expr_to_sexp expr_val]

(**
 * Convert an expression to an S-expression string
 *
 * This is the main entry point for serializing expressions to strings.
 *
 * @param expr The expression to convert
 * @return A string representation of the expression in S-expression format
 *)
let to_sexpr_string expr =
  sexp_to_string (expr_to_sexp expr)

(**
 * Alias for to_sexpr_string for backward compatibility
 *
 * @param expr The expression to convert
 * @return A string representation of the expression
 *)
let string_of_expr expr = to_sexpr_string expr 