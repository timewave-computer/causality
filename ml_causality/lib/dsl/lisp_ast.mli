(** lisp_ast.mli - Interface for Lisp Abstract Syntax Tree operations,
    focusing on S-expression serialization. *)

(* Purpose: Interface for Lisp Abstract Syntax Tree (AST) and S-expression serialization. Uses the new shared AST from Types module. *)

open Ml_causality_lib_types.Types

(* The primary AST type is now taken from the Types module *)
type t = expr

(** [sexp_to_string sexp] converts an S-expression to its string representation. *)
val sexp_to_string : Sexplib0.Sexp.t -> string

(** [expr_to_sexp e] converts an expression of type [Types.expr] to an S-expression.
    This is the core serialization function.
*)
val expr_to_sexp : expr -> Sexplib0.Sexp.t

(** [value_expr_to_sexp v] converts a [Types.value_expr] to an S-expression. *)
val value_expr_to_sexp : value_expr -> Sexplib0.Sexp.t

(** [atom_to_sexp a] converts an [Types.atom] to an S-expression. *)
val atom_to_sexp : atom -> Sexplib0.Sexp.t

(** [atomic_combinator_to_symbol_string ac] converts an [Types.atomic_combinator] to its Lisp symbol string. *)
val atomic_combinator_to_symbol_string : atomic_combinator -> string

(** Converts a Lisp expression (from the canonical AST) into its S-expression string representation.
    This is used for generating Lisp code that can be parsed by a Lisp interpreter.
*)
val to_sexpr_string : expr -> string

(** [string_of_expr expr] is an alias for to_sexpr_string for backward compatibility. *)
val string_of_expr : expr -> string 