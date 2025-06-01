(* Purpose: PPX Registry for storing Lisp S-expressions *)

(** The type of the key used in the registry. Typically a fully qualified OCaml function name. *)
type key = string

(** The type of the value stored: a Lisp S-expression string. *)
type lisp_code = string

(** Content-addressed ID for Lisp code *)
type lisp_code_id = string

(** Register a Lisp S-expression string under a given key.
    This is intended to be called by the PPX rewriter during compilation.
    If a key is already present, its value will be replaced.
    Returns the content-addressed ID of the stored code. *)
val register_logic : key -> lisp_code -> lisp_code_id

(** Retrieve the Lisp S-expression string associated with a key, if any.
    This is intended to be called by the DSL functions when constructing TEL resources. *)
val get_logic : key -> lisp_code option

(** Retrieve the content-addressed ID for a function's Lisp code.
    Returns the ID if found, None otherwise. *)
val get_logic_id : key -> lisp_code_id option

(** Retrieve Lisp code by its content-addressed ID.
    Returns the code if found, None otherwise. *)
val get_lisp_code_by_id : lisp_code_id -> lisp_code option

(** Store Lisp code in content-addressed storage.
    Returns the content-addressed ID for the code. *)
val store_lisp_code : lisp_code -> lisp_code_id

(** Get statistics about code deduplication.
    Returns (unique_functions, unique_code_segments) tuple. *)
val get_deduplication_stats : unit -> (int * int) 