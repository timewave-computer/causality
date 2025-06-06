(* ------------ CONTENT ADDRESSING ------------ *)
(* Purpose: Content-addressable storage and hashing *)

open Ocaml_causality_core

(* ------------ HASH FUNCTIONS ------------ *)

(** Generate content hash using digestif *)
let hash_content (data: bytes) : bytes =
  let hash = Digestif.SHA256.(digest_bytes data |> to_raw_string) in
  Bytes.of_string hash

(** Generate ID from content *)
let content_to_id (content: string) : entity_id =
  hash_content (Bytes.of_string content)

(* ------------ CONTENT ADDRESSING ------------ *)

(* TODO: Add content addressing operations *)

(* ------------ VERIFICATION ------------ *)

(* TODO: Add content verification functions *) 