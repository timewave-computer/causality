(** Content Addressing System Interface *)

module EntityId : sig
  type t = string
  val from_bytes : bytes -> t
  val from_content : 'a -> t
  val to_bytes : t -> bytes
  val to_hex : t -> string
  val from_hex : string -> t
  val compare : t -> t -> int
  val equal : t -> t -> bool
  val zero : t
end

type resource_id = EntityId.t
type value_expr_id = EntityId.t
type expr_id = EntityId.t
type row_type_id = EntityId.t
type handler_id = EntityId.t
type transaction_id = EntityId.t
type intent_id = EntityId.t
type domain_id = EntityId.t
type nullifier_id = EntityId.t

module Timestamp : sig
  type t = int64
  val from_millis : int64 -> t
  val to_millis : t -> int64
  val now : unit -> t
  val zero : t
  val compare : t -> t -> int
end

module Str : sig
  type t = string
  val create : string -> t
  val to_string : t -> string
  val compare : t -> t -> int
  val equal : t -> t -> bool
end

module type ContentAddressable = sig
  type t
  val content_id : t -> EntityId.t
end

val content_id : 'a -> EntityId.t

module SSZ : sig
  val serialize_string : string -> bytes
  val serialize_int64 : int64 -> bytes
  val serialize_bool : bool -> bytes
  val serialize_bytes : bytes -> bytes
  val concat_serialized : bytes list -> bytes
end

module Hash : sig
  val hash_bytes : bytes -> bytes
  val hash_to_hex : bytes -> string
end

val compute_content_hash : 'a -> EntityId.t
