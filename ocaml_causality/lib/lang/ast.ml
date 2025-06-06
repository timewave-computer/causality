(* ------------ ABSTRACT SYNTAX TREE ------------ *)
(* Purpose: Core AST types for expressions and values *)

(* ------------ EXPRESSION AST TYPES ------------ *)
(* Purpose: Abstract syntax tree types for expressions *)

(* Import types from core module *)
open Ocaml_causality_core

type atomic_combinator =
  | List | MakeMap | GetField | Length
  | Eq | Lt | Gt | Add | Sub | Mul | Div
  | And | Or | Not | If | Let | Define
  | Quote | Cons | Car | Cdr

type atom =
  | AInt of int64
  | AString of str_t
  | ABoolean of bool
  | ANil

(* ------------ VALUE TYPES ------------ *)

(* TODO: Extract ValueExpr type from lib/dsl/dsl.ml *)

(* ------------ ATOMIC TYPES ------------ *)

(* TODO: Extract AtomicCombinator type from lib/dsl/dsl.ml *)

(* ------------ AST UTILITIES ------------ *)

(* TODO: Add AST traversal and manipulation functions *)

(* ------------ TYPE CHECKING ------------ *)

(* TODO: Add basic type checking functions *) 