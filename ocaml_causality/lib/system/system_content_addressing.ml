(** Content Addressing System
    
    This module provides the core content addressing functionality for the Causality framework.
    All entities in the system are identified by deterministic hashes of their SSZ-serialized
    content, ensuring global uniqueness, deduplication, and verifiable references.
*)

(** {1 Core Entity Identifiers} *)

(** Universal content-addressed identifier *)
module EntityId = struct
  type t = string  (* 32-byte SHA256 hash as hex string for now *)
  
  (** Create an EntityId from raw bytes *)
  let from_bytes (bytes : bytes) : t = 
    bytes |> Bytes.to_string |> Digest.to_hex
  
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
    let hex_to_int c = match c with
      | '0'..'9' -> Char.code c - Char.code '0'
      | 'a'..'f' -> Char.code c - Char.code 'a' + 10
      | 'A'..'F' -> Char.code c - Char.code 'A' + 10
      | _ -> invalid_arg "EntityId.to_bytes: invalid hex character"
    in
    for i = 0 to len - 1 do
      let high = hex_to_int id.[i * 2] in
      let low = hex_to_int id.[i * 2 + 1] in
      Bytes.set_uint8 result i ((high lsl 4) lor low);
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
  let zero = String.make 64 '0'  (* 32 bytes = 64 hex chars *)
end

(** {1 Type Aliases} *)

(** Content-addressed identifier for a Resource *)
type resource_id = EntityId.t

(** Content-addressed identifier for a ValueExpr *)  
type value_expr_id = EntityId.t

(** Content-addressed identifier for an Expr (executable expression) *)
type expr_id = EntityId.t

(** Content-addressed identifier for a RowType schema *)
type row_type_id = EntityId.t

(** Content-addressed identifier for a Handler *)
type handler_id = EntityId.t

(** Content-addressed identifier for a Transaction *)
type transaction_id = EntityId.t

(** Content-addressed identifier for an Intent *)
type intent_id = EntityId.t

(** Content-addressed identifier for a Domain *)
type domain_id = EntityId.t

(** Content-addressed identifier for a Nullifier (for preventing double-spending) *)
type nullifier_id = EntityId.t

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
    Unix.time () |> ( *. ) 1000.0 |> Int64.of_float
  
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

(* TODO: Replace with proper SSZ serialization + SHA256 hashing *)
let compute_content_hash content = 
  EntityId.from_content content 