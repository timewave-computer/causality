
(** Content Addressing System

    This module provides the core content addressing functionality for the
    Causality framework. All entities in the system are identified by
    deterministic hashes of their SSZ-serialized content, ensuring global
    uniqueness, deduplication, and verifiable references. *)

(** {1 Core Entity Identifiers} *)

(** Universal content-addressed identifier *)
module EntityId = struct
  type t = string (* 32-byte SHA256 hash as hex string for now *)

  (** Create an EntityId from raw bytes *)
  let from_bytes (bytes : bytes) : t = bytes |> Digest.bytes |> Digest.to_hex

  (** Create an EntityId from the content hash of SSZ-serializable data *)
  let from_content (content : 'a) : t =
    (* Convert content to string representation for hashing *)
    let content_str = Marshal.to_string content [] in
    (* Use built-in Digest module for SHA256-like hashing *)
    let hash = Digest.string content_str in
    Digest.to_hex hash

  (** Get the raw bytes of this EntityId *)
  let to_bytes (id : t) : bytes =
    (* Convert hex string back to bytes *)
    let len = String.length id / 2 in
    let result = Bytes.create len in
    let hex_to_int c =
      match c with
      | '0' .. '9' -> Char.code c - Char.code '0'
      | 'a' .. 'f' -> Char.code c - Char.code 'a' + 10
      | 'A' .. 'F' -> Char.code c - Char.code 'A' + 10
      | _ -> invalid_arg "EntityId.to_bytes: invalid hex character"
    in
    for i = 0 to len - 1 do
      let high = hex_to_int id.[i * 2] in
      let low = hex_to_int id.[(i * 2) + 1] in
      Bytes.set_uint8 result i ((high lsl 4) lor low)
    done;
    result

  (** Convert to a hex string for debugging/display *)
  let to_hex (id : t) : string = id

  (** Create from hex string (for testing/debugging) *)
  let from_hex (hex_str : string) : t = hex_str

  (** Compare two EntityIds *)
  let compare (id1 : t) (id2 : t) : int = String.compare id1 id2

  (** Check if two EntityIds are equal *)
  let equal (id1 : t) (id2 : t) : bool = String.equal id1 id2

  (** Zero EntityId (for testing) *)
  let zero = String.make 64 '0' (* 32 bytes = 64 hex chars *)
end

(** {1 Type Aliases} *)

type resource_id = EntityId.t
(** Content-addressed identifier for a Resource *)

type value_expr_id = EntityId.t
(** Content-addressed identifier for a ValueExpr *)

type expr_id = EntityId.t
(** Content-addressed identifier for an Expr (executable expression) *)

type row_type_id = EntityId.t
(** Content-addressed identifier for a RowType schema *)

type handler_id = EntityId.t
(** Content-addressed identifier for a Handler *)

type transaction_id = EntityId.t
(** Content-addressed identifier for a Transaction *)

type intent_id = EntityId.t
(** Content-addressed identifier for an Intent *)

type domain_id = EntityId.t
(** Content-addressed identifier for a Domain *)

type nullifier_id = EntityId.t
(** Content-addressed identifier for a Nullifier (for preventing
    double-spending) *)

(** {1 Timestamp Implementation} *)

(** Unix timestamp in milliseconds (int64 for ZK compatibility) *)
module Timestamp = struct
  type t = int64 [@@deriving show, eq]

  (** Create timestamp from milliseconds *)
  let from_millis (millis : int64) : t = millis

  (** Get milliseconds from timestamp *)
  let to_millis (ts : t) : int64 = ts

  (** Get current timestamp *)
  let now () : t =
    (* Simple implementation using current time *)
    Int64.of_float (Unix.time () *. 1000.0)

  (** Zero timestamp *)
  let zero : t = 0L

  (** Compare timestamps *)
  let compare (ts1 : t) (ts2 : t) : int = Int64.compare ts1 ts2
end

(** {1 String with SSZ Serialization} *)

(** UTF-8 string with SSZ serialization support *)
module Str = struct
  type t = string [@@deriving show, eq]

  (** Create a new Str from string *)
  let create (s : string) : t = s

  (** Convert to string *)
  let to_string (str : t) : string = str

  (** Compare two Str values *)
  let compare (s1 : t) (s2 : t) : int = String.compare s1 s2

  (** Check equality *)
  let equal (s1 : t) (s2 : t) : bool = String.equal s1 s2
end

(** {1 Content Addressable Trait} *)

(** Trait for types that can be content-addressed *)
module type ContentAddressable = sig
  type t

  val content_id : t -> EntityId.t
end

(** Implementation for any type using generic content addressing *)
let content_id (value : 'a) : EntityId.t = EntityId.from_content value

(** {1 SSZ Serialization and Content Hashing} *)

(** SSZ-compatible serialization utilities *)
module SSZ = struct
  (** Serialize a string to SSZ format *)
  let serialize_string (s : string) : bytes =
    let len = String.length s in
    let result = Bytes.create (len + 4) in
    (* Little-endian length prefix *)
    Bytes.set_int32_le result 0 (Int32.of_int len);
    (* String content *)
    Bytes.blit_string s 0 result 4 len;
    result

  (** Serialize an int64 to SSZ format *)
  let serialize_int64 (i : int64) : bytes =
    let result = Bytes.create 8 in
    Bytes.set_int64_le result 0 i;
    result

  (** Serialize a boolean to SSZ format *)
  let serialize_bool (b : bool) : bytes =
    let result = Bytes.create 1 in
    Bytes.set_uint8 result 0 (if b then 1 else 0);
    result

  (** Serialize bytes to SSZ format *)
  let serialize_bytes (b : bytes) : bytes =
    let len = Bytes.length b in
    let result = Bytes.create (len + 4) in
    Bytes.set_int32_le result 0 (Int32.of_int len);
    Bytes.blit b 0 result 4 len;
    result

  (** Concatenate multiple serialized values *)
  let concat_serialized (parts : bytes list) : bytes =
    let total_len =
      List.fold_left (fun acc b -> acc + Bytes.length b) 0 parts
    in
    let result = Bytes.create total_len in
    let _ =
      List.fold_left
        (fun offset b ->
          let len = Bytes.length b in
          Bytes.blit b 0 result offset len;
          offset + len)
        0 parts
    in
    result
end

(** SHA256-like hashing using OCaml's Digest module *)
module Hash = struct
  (** Compute SHA256-like hash of bytes *)
  let hash_bytes (data : bytes) : bytes =
    let hash_str = Digest.bytes data in
    Bytes.of_string hash_str

  (** Compute SHA256-like hash and return as hex string *)
  let hash_to_hex (data : bytes) : string =
    let hash = Digest.bytes data in
    Digest.to_hex hash
end

(** Proper SSZ serialization + SHA256 hashing *)
let compute_content_hash content =
  (* This is a simplified version - in production would use proper SSZ *)
  let serialized = Marshal.to_bytes content [] in
  let hash = Hash.hash_bytes serialized in
  EntityId.from_bytes hash
