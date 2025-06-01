(*
 * Binary Serialization Module Interface
 *
 * This module provides functionality for binary serialization of Causality types.
 * It enables efficient data exchange with other Causality implementations,
 * particularly the Rust implementation.
 *)

open Ocaml_causality_core

(** Binary format identifier *)
type binary_format =
  | SSZ       (** Simple Serialize format *)
  | Protobuf  (** Protocol Buffers format *)
  | Msgpack   (** MessagePack format *)
  | CBOR      (** Concise Binary Object Representation format *)

(** Binary serialization error *)
type binary_error =
  | EncodingError of string    (** Error during encoding *)
  | DecodingError of string    (** Error during decoding *)
  | UnsupportedFormat of string (** Unsupported binary format *)
  | UnsupportedType of string   (** Unsupported type for serialization *)

(** Result type for binary operations *)
type 'a result = ('a, binary_error) Result.t

(** Convert a binary format to string *)
val format_to_string : binary_format -> string

(** Parse a string to a binary format *)
val format_of_string : string -> binary_format option

(** Encode a value to binary format *)
val encode : 'a -> string -> binary_format -> bytes result

(** Decode a binary value *)
val decode : bytes -> string -> binary_format -> 'a result

(** Encode a value to binary format using string format name *)
val encode_with_format_name : 'a -> string -> string -> bytes result

(** Decode a binary value using string format name *)
val decode_with_format_name : bytes -> string -> string -> 'a result

(** Convert binary data to a hex string for debugging *)
val to_hex : bytes -> string 