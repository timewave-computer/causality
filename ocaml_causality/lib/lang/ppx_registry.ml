(* Purpose: PPX Registry for storing Lisp S-expressions *)

(** The type of the key used in the registry. Typically a fully qualified OCaml function name. *)
type key = string

(** The type of the value stored: a Lisp S-expression string. *)
type lisp_code = string

(** Content-addressed ID for Lisp code *)
type lisp_code_id = string

(** Global registry mapping keys to lisp code *)
let code_registry : (key, lisp_code) Hashtbl.t = Hashtbl.create 128

(** Content-addressed storage for deduplication *)
let content_store : (lisp_code_id, lisp_code) Hashtbl.t = Hashtbl.create 128

(** Mapping from key to content-addressed ID *)
let key_to_id : (key, lisp_code_id) Hashtbl.t = Hashtbl.create 128

(** Generate content-addressed ID from Lisp code *)
let generate_content_id (code: lisp_code) : lisp_code_id =
  let hash = Digestif.SHA256.digest_string code in
  Digestif.SHA256.to_hex hash

(** Store Lisp code in content-addressed storage.
    Returns the content-addressed ID for the code. *)
let store_lisp_code (code: lisp_code) : lisp_code_id =
  let id = generate_content_id code in
  Hashtbl.replace content_store id code;
  id

(** Register a Lisp S-expression string under a given key.
    Returns the content-addressed ID of the stored code. *)
let register_logic (key: key) (code: lisp_code) : lisp_code_id =
  let id = store_lisp_code code in
  Hashtbl.replace code_registry key code;
  Hashtbl.replace key_to_id key id;
  id

(** Retrieve the Lisp S-expression string associated with a key, if any. *)
let get_logic (key: key) : lisp_code option =
  Hashtbl.find_opt code_registry key

(** Retrieve the content-addressed ID for a function's Lisp code. *)
let get_logic_id (key: key) : lisp_code_id option =
  Hashtbl.find_opt key_to_id key

(** Retrieve Lisp code by its content-addressed ID. *)
let get_lisp_code_by_id (id: lisp_code_id) : lisp_code option =
  Hashtbl.find_opt content_store id

(** Get statistics about code deduplication. *)
let get_deduplication_stats () : (int * int) =
  let unique_functions = Hashtbl.length code_registry in
  let unique_code_segments = Hashtbl.length content_store in
  (unique_functions, unique_code_segments) 