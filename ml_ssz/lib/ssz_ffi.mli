(** OCaml bindings for Rust SSZ implementation *)

(** Direct Rust function bindings *)
val rust_serialize_bool : bool -> string
val rust_deserialize_bool : string -> bool
val rust_serialize_u32 : int -> string
val rust_deserialize_u32 : string -> int
val rust_serialize_string : string -> string
val rust_deserialize_string : string -> string
val rust_simple_hash : string -> string
val rust_roundtrip_bool : bool -> bool
val rust_roundtrip_u32 : int -> int
val rust_roundtrip_string : string -> string

(** Mock implementations for testing without the actual Rust library *)
module Mock : sig
  val serialize_bool : bool -> string
  val deserialize_bool : string -> bool
  val serialize_u32 : int -> string
  val deserialize_u32 : string -> int
  val serialize_string : string -> string
  val deserialize_string : string -> string
  val simple_hash : string -> string
  val roundtrip_bool : bool -> bool
  val roundtrip_u32 : int -> int
  val roundtrip_string : string -> string
end

(** Mock round-trip functions for comprehensive testing *)
val ocaml_to_rust_bytes : string -> string
val rust_to_ocaml_bytes : string -> string
val rust_to_ocaml_to_rust_bytes : string -> string

(** Hash tree root compatibility check *)
val check_hash_compatibility : string -> string * string 