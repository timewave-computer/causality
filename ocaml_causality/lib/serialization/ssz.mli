(*
 * SSZ Serialization Module Interface
 *
 * This module provides Simple Serialize (SSZ) encoding and decoding
 * functionality for Causality types. SSZ is a deterministic serialization
 * method optimized for minimal encoding/decoding overhead and fixed-width 
 * representation of data.
 *)

open Ocaml_causality_core

(** Serialized data type *)
type serialized = bytes

(** Serialization error type *)
type error =
  | InvalidFormat of string        (** Invalid data format *)
  | MissingField of string         (** Required field missing *)
  | UnsupportedType of string      (** Type not supported for serialization *)
  | LengthMismatch of string       (** Length mismatch during deserialization *)
  | Other of string                (** Other serialization errors *)

(** Result type for serialization operations *)
type 'a result = ('a, error) Result.t

(** Encode an AST value expression to bytes *)
val encode_value : Ocaml_causality_core.Types.value_expr -> serialized

(** Encode an AST expression to bytes *)
val encode : Ocaml_causality_lang.Ast.expr -> serialized

(** Attempt to decode a value expression from bytes *)
val decode_value : serialized -> Ocaml_causality_core.Types.value_expr result

(** Attempt to decode an expression from bytes *)
val decode : serialized -> Ocaml_causality_lang.Ast.expr result

(** Create a string representation of serialized data (for debugging) *)
val to_hex : serialized -> string

(** Encode the content of a resource (excluding its ID) to bytes *)
val encode_resource_content : Ocaml_causality_core.Types.resource -> serialized 