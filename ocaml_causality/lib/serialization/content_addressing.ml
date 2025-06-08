(* ------------ CONTENT ADDRESSING ------------ *)
(* Purpose: Content-addressable storage and hashing *)

open Ocaml_causality_core

(* ------------ HASH FUNCTIONS ------------ *)

(** Generate content hash using digestif *)
let hash_content (data : bytes) : bytes =
  let hash = Digestif.SHA256.(digest_bytes data |> to_raw_string) in
  Bytes.of_string hash

(** Generate ID from content *)
let content_to_id (content : string) : entity_id =
  hash_content (Bytes.of_string content)

(* ------------ CONTENT ADDRESSING ------------ *)

type content_store = (entity_id, bytes) Hashtbl.t
(** Content store type for mapping IDs to content *)

(** Create a new content store *)
let create_store () : content_store = Hashtbl.create 1024

(** Store content and return its content-addressed ID *)
let store_content (store : content_store) (content : bytes) : entity_id =
  let content_id = hash_content content in
  Hashtbl.replace store content_id content;
  content_id

(** Retrieve content by its ID *)
let retrieve_content (store : content_store) (id : entity_id) : bytes option =
  Hashtbl.find_opt store id

(** Check if content exists in store *)
let content_exists (store : content_store) (id : entity_id) : bool =
  Hashtbl.mem store id

(** Get all stored content IDs *)
let list_content_ids (store : content_store) : entity_id list =
  Hashtbl.fold (fun id _ acc -> id :: acc) store []

(* ------------ VERIFICATION ------------ *)

(** Verify that content matches its claimed ID *)
let verify_content_id (content : bytes) (claimed_id : entity_id) : bool =
  let actual_id = hash_content content in
  Bytes.equal actual_id claimed_id

(** Verify content integrity in store *)
let verify_store_integrity (store : content_store) : (entity_id * bool) list =
  Hashtbl.fold
    (fun id content acc ->
      let is_valid = verify_content_id content id in
      (id, is_valid) :: acc)
    store []

(** Remove invalid content from store *)
let cleanup_invalid_content (store : content_store) : int =
  let invalid_ids = ref [] in
  Hashtbl.iter
    (fun id content ->
      if not (verify_content_id content id) then
        invalid_ids := id :: !invalid_ids)
    store;
  List.iter (Hashtbl.remove store) !invalid_ids;
  List.length !invalid_ids
