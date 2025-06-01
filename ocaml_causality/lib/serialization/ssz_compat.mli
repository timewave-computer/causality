(*
 * SSZ Compatibility Module Interface
 *
 * This module provides compatibility with other Simple Serialize (SSZ) 
 * implementations, particularly those used in the Ethereum ecosystem.
 * It enables interoperability with Ethereum 2.0 and other systems
 * that use SSZ for serialization.
 *)

open Ocaml_causality_core

(** SSZ basic types *)
type ssz_basic_type = 
  | UInt8        (** 8-bit unsigned integer *)
  | UInt16       (** 16-bit unsigned integer *)
  | UInt32       (** 32-bit unsigned integer *)
  | UInt64       (** 64-bit unsigned integer *)
  | Bool         (** Boolean value *)
  | Bytes32      (** Fixed-length 32-byte array *)
  | Address      (** Ethereum address (20 bytes) *)

(** SSZ schema type *)
type ssz_type =
  | Basic of ssz_basic_type        (** Basic type *)
  | Vector of ssz_type * int       (** Fixed-length array *)
  | List of ssz_type * int         (** Variable-length array with max size *)
  | Container of (string * ssz_type) list (** Struct-like container *)

(** SSZ serialization/deserialization error *)
type ssz_error =
  | InvalidLength of string        (** Invalid length *)
  | InvalidOffset of string        (** Invalid offset *)
  | InvalidType of string          (** Invalid type *)
  | ParseError of string           (** Error parsing SSZ data *)
  | OverflowError of string        (** Value overflow *)

(** Encode a uint8 as SSZ bytes *)
val encode_uint8 : int -> bytes

(** Encode a uint16 as SSZ bytes (little-endian) *)
val encode_uint16 : int -> bytes

(** Encode a uint32 as SSZ bytes (little-endian) *)
val encode_uint32 : int32 -> bytes

(** Encode a uint64 as SSZ bytes (little-endian) *)
val encode_uint64 : int64 -> bytes

(** Encode a boolean as SSZ bytes *)
val encode_bool : bool -> bytes

(** Encode bytes32 (fixed 32-byte array) *)
val encode_bytes32 : bytes -> bytes

(** Encode Ethereum address (20 bytes) *)
val encode_address : bytes -> bytes

(** Decode a uint8 from SSZ bytes *)
val decode_uint8 : bytes -> int -> int * int

(** Decode a uint16 from SSZ bytes (little-endian) *)
val decode_uint16 : bytes -> int -> int * int

(** Decode a uint32 from SSZ bytes (little-endian) *)
val decode_uint32 : bytes -> int -> int32 * int

(** Decode a uint64 from SSZ bytes (little-endian) *)
val decode_uint64 : bytes -> int -> int64 * int

(** Decode a boolean from SSZ bytes *)
val decode_bool : bytes -> int -> bool * int

(** Decode bytes32 (fixed 32-byte array) *)
val decode_bytes32 : bytes -> int -> bytes * int

(** Decode Ethereum address (20 bytes) *)
val decode_address : bytes -> int -> bytes * int

(** Calculate the fixed part size of a container type *)
val fixed_part_size : ssz_type -> int

(** Convert OCaml Causality types to Ethereum 2.0 SSZ types *)
val to_eth2_ssz_type : string -> ssz_type option

(** Create an Ethereum 2.0 compatible SSZ hash tree root *)
val eth2_hash_tree_root : bytes -> ssz_type -> string
