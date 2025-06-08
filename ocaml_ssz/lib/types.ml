(** * Types module for SSZ serialization * * Defines the core type
    specifications for Simple Serialize (SSZ) format, * including type kinds,
    size calculations, and serialization interfaces. *)

(*-----------------------------------------------------------------------------
 * Type Kind Definitions
 *---------------------------------------------------------------------------*)

(** * The kind of serializable type * * Used for determining the appropriate
    serialization strategy *)
type kind =
  | Basic  (** Fixed-size basic types (bool, integers) *)
  | Vector  (** Fixed-length homogeneous collection *)
  | List  (** Variable-length homogeneous collection *)
  | Container  (** Struct-like composite type *)
  | Union  (** Tagged union/variant type *)

(*-----------------------------------------------------------------------------
 * Type Specification
 *---------------------------------------------------------------------------*)

type 'a t = {
    kind : kind  (** The kind of type *)
  ; size : int option  (** Fixed size in bytes, None for variable-sized types *)
  ; encode : 'a -> bytes  (** Encoding function *)
  ; decode : bytes -> int -> 'a * int
        (** Decoding function, returns value and new offset *)
}
(** * Type specification for an SSZ type * * Provides the metadata and functions
    needed for serialization *)

(*-----------------------------------------------------------------------------
 * Size Utilities
 *---------------------------------------------------------------------------*)

(** Indicates if a type has a fixed size.

    @param typ The type to check
    @return true if the type has a fixed size, false otherwise *)
let is_fixed_size typ = Option.is_some typ.size

(** Get the size of a type, or raise an exception if variable-sized.

    @param typ The type to get the size of
    @return The size in bytes
    @raise Failure if the type does not have a fixed size *)
let fixed_size typ =
  match typ.size with
  | Some size -> size
  | None -> failwith "Type does not have a fixed size"

(*-----------------------------------------------------------------------------
 * Specification Constants
 *---------------------------------------------------------------------------*)

(** * Sizes and limits from the SSZ specification * * Defines constants used
    throughout the serialization process *)
module Constants = struct
  (** Size of a chunk in bytes (32) *)
  let max_chunk_size = 32

  (** Size of offset entries (uint32) *)
  let bytes_per_length_offset = 4

  (** Size of length prefixes (uint32) *)
  let bytes_per_length_prefix = 4

  (** Default maximum length for lists *)
  let default_max_length = 1024 * 1024
  (** 1M elements *)
end
