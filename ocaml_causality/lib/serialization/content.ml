(** Content module for SSZ serialization *)

open Causality_system.System_content_addressing

(** Hash content using SHA256 *)
let hash_content (content : bytes) : bytes =
  (* Use OCaml's Digest module for hashing *)
  Bytes.of_string (Digest.bytes content)
  
(** Get content ID from bytes *)  
let content_id (content : bytes) : EntityId.t =
  EntityId.from_bytes (hash_content content)
