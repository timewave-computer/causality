(* ------------ EXPRESSION TYPES ------------ *)
(* Purpose: Main expression types and operations *)

(* Import types from core module *)
open Ocaml_causality_core

type expr =
  | EAtom of Ast.atom
  | EConst of Value.value_expr
  | EVar of str_t
  | EApply of expr * expr list
  | ECombinator of Ast.atomic_combinator 