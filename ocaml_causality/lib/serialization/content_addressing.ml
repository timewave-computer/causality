(** Content Addressing System *)

open Causality_system.System_content_addressing

(** Content addressing for expressions and values *)
let content_to_id (content : string) : EntityId.t =
  EntityId.from_content content

(** Content store for caching *)
type content_store = (string, bytes) Hashtbl.t

(** Create a new content store *)
let create_content_store () : content_store =
  Hashtbl.create 1024

(** Store content and return its ID *)
let store_content (store : content_store) (content : bytes) : EntityId.t =
  let content_id = EntityId.from_bytes content in
  Hashtbl.replace store content_id (content);
  EntityId.from_content content_id

(** Retrieve content by ID *)
let retrieve_content (store : content_store) (id : EntityId.t) : bytes option =
  Hashtbl.find_opt store id
