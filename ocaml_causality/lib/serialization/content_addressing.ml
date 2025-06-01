(*
 * Content Addressing Module
 *
 * This module provides functionality for content-based addressing and 
 * identification of data using cryptographic hashing. It enables deterministic
 * referencing of resources, expressions, and other entities based on their content.
 *)

open Ocaml_causality_core
open Digestif

(* ------------ HASH TYPES ------------ *)

(** Hash algorithm identifier *)
type hash_algorithm =
  | SHA256          (* SHA-256 algorithm *)
  | BLAKE2B         (* BLAKE2b algorithm *)
  | KECCAK256       (* Keccak-256 algorithm (Ethereum) *)

(** Content hash with algorithm information *)
type content_hash = {
  algorithm: hash_algorithm;  (* Hash algorithm used *)
  digest: string;             (* Hash digest as hex string *)
}

(* ------------ HASHING FUNCTIONS ------------ *)

(** Compute SHA-256 hash of byte data *)
let sha256_hash (data: bytes) : string =
  SHA256.digest_string (Bytes.to_string data)
  |> SHA256.to_hex

(** Compute BLAKE2b hash of byte data *)
let blake2b_hash (data: bytes) : string =
  BLAKE2B.digest_string (Bytes.to_string data)
  |> BLAKE2B.to_hex

(** Compute Keccak-256 hash of byte data *)
let keccak256_hash (data: bytes) : string =
  let open Digestif.SHA3_256 in
  digest_string (Bytes.to_string data)
  |> to_hex

(** Compute hash using specified algorithm *)
let hash_bytes (algorithm: hash_algorithm) (data: bytes) : content_hash =
  let digest = match algorithm with
    | SHA256 -> sha256_hash data
    | BLAKE2B -> blake2b_hash data
    | KECCAK256 -> keccak256_hash data
  in
  { algorithm; digest }

(* ------------ CONTENT ID GENERATION ------------ *)

(** Generate a content ID for entity identification *)
let generate_content_id (data: bytes) : entity_id =
  (* Use SHA-256 by default for content IDs *)
  let hash = sha256_hash data in
  (* Prefix with "c:" to indicate content-addressed ID *)
  Bytes.of_string ("c:" ^ hash)

(** Generate a multi-hash content ID with algorithm prefix *)
let generate_multi_hash (algorithm: hash_algorithm) (data: bytes) : entity_id =
  let prefix = match algorithm with
    | SHA256 -> "sha256:"
    | BLAKE2B -> "blake2b:"
    | KECCAK256 -> "keccak256:"
  in
  let hash = match algorithm with
    | SHA256 -> sha256_hash data
    | BLAKE2B -> blake2b_hash data
    | KECCAK256 -> keccak256_hash data
  in
  Bytes.of_string (prefix ^ hash)

(* ------------ CONTENT VERIFICATION ------------ *)

(** Check if a content ID matches the content it's supposed to identify *)
let verify_content_id (id: entity_id) (data: bytes) : bool =
  (* Parse the ID to determine the algorithm *)
  let id_str = Bytes.to_string id in
  if String.length id_str < 3 then
    false
  else
    let expected_hash = 
      if String.sub id_str 0 2 = "c:" then
        (* Legacy format with implicit SHA-256 *)
        String.sub id_str 2 (String.length id_str - 2)
      else if String.sub id_str 0 7 = "sha256:" then
        String.sub id_str 7 (String.length id_str - 7)
      else if String.sub id_str 0 8 = "blake2b:" then
        String.sub id_str 8 (String.length id_str - 8)
      else if String.sub id_str 0 9 = "keccak256:" then
        String.sub id_str 9 (String.length id_str - 9)
      else
        "" (* Unrecognized format *)
    in
    
    if expected_hash = "" then
      false
    else
      (* Compute the actual hash of the data *)
      let actual_hash = 
        if String.sub id_str 0 2 = "c:" || String.sub id_str 0 7 = "sha256:" then
          sha256_hash data
        else if String.sub id_str 0 8 = "blake2b:" then
          blake2b_hash data
        else (* keccak256: *)
          keccak256_hash data
      in
      
      (* Compare expected and actual hashes *)
      String.lowercase_ascii expected_hash = String.lowercase_ascii actual_hash

(** Parse a content ID to extract algorithm and hash *)
let parse_content_id (id: entity_id) : (hash_algorithm * string) option =
  let id_str = Bytes.to_string id in
  if String.length id_str >= 10 && String.sub id_str 0 9 = "keccak256:" then
    Some (KECCAK256, String.sub id_str 9 (String.length id_str - 9))
  else if String.length id_str >= 8 && String.sub id_str 0 7 = "sha256:" then
    Some (SHA256, String.sub id_str 7 (String.length id_str - 7))
  else if String.length id_str >= 9 && String.sub id_str 0 8 = "blake2b:" then
    Some (BLAKE2B, String.sub id_str 8 (String.length id_str - 8))
  else if String.length id_str >= 3 && String.sub id_str 0 2 = "c:" then (* legacy *)
    Some (SHA256, String.sub id_str 2 (String.length id_str - 2))
  else
    None

(* ------------ RESOURCE CONTENT ADDRESSING ------------ *)

(** Generate content ID for a resource based on its serialized form *)
let content_address_resource (resource : Types.resource) : entity_id =
  let ssz_bytes = Ssz.encode_resource_content resource in
  let content_hash_obj = hash_bytes SHA256 ssz_bytes in
  Bytes.of_string content_hash_obj.digest (* Using the hex digest as entity_id for now *)

(** Generate content ID for an expression *)
let content_address_expr (expr : Ocaml_causality_lang.Ast.expr) : expr_id =
  let ssz_bytes = Ssz.encode expr in
  let content_hash_obj = hash_bytes SHA256 ssz_bytes in
  Bytes.of_string content_hash_obj.digest

(** Generate content ID for a value expression *)
let content_address_value (value : Ocaml_causality_core.Types.value_expr) : value_expr_id =
  let ssz_bytes = Ssz.encode_value value in
  let content_hash_obj = hash_bytes SHA256 ssz_bytes in
  Bytes.of_string content_hash_obj.digest

(* ------------ NULLIFIER GENERATION ------------ *)

(** Generate a nullifier for a consumed resource *)
let generate_nullifier (resource_content_id: entity_id) (secret: bytes) : entity_id (* entity_id is used for the nullifier hash itself *)=
  let combined = Bytes.cat secret resource_content_id in (* Order matters: secret first *)
  let content_hash_obj = hash_bytes SHA256 combined in
  Bytes.of_string content_hash_obj.digest

(** Verify a nullifier against a resource *)
let verify_nullifier (claimed_nullifier_hash: entity_id) (resource_content_id: entity_id) (secret: bytes) : bool =
  let expected_hash_obj = hash_bytes SHA256 (Bytes.cat secret resource_content_id) in
  let expected_nullifier_hash = Bytes.of_string expected_hash_obj.digest in
  Bytes.equal claimed_nullifier_hash expected_nullifier_hash

(* ------------ UTILITIES ------------ *)
(*
(** Convert a hash to a short representation (first 8 chars) *)
let short_hash (hash: content_hash) : string =
  if String.length hash.digest >= 8 then
    String.sub hash.digest 0 8
  else
    hash.digest

(** Determine if a string is a valid content ID *)
let is_content_id (id: string) : bool =
  (* Check if the ID starts with a recognized prefix *)
  String.length id >= 3 &&
  (String.sub id 0 2 = "c:" ||
   (String.length id >= 8 && String.sub id 0 7 = "sha256:") ||
   (String.length id >= 9 && String.sub id 0 8 = "blake2b:") ||
   (String.length id >= 10 && String.sub id 0 9 = "keccak256:"))

(** Extract the hash algorithm from a content ID *)
let algorithm_from_id (id: entity_id) : hash_algorithm option =
  let id_str = Bytes.to_string id in
  if String.length id_str < 3 then
    None
  else if String.sub id_str 0 2 = "c:" then
    Some SHA256
  else if String.length id_str >= 8 && String.sub id_str 0 7 = "sha256:" then
    Some SHA256
  else if String.length id_str >= 9 && String.sub id_str 0 8 = "blake2b:" then
    Some BLAKE2B
  else if String.length id_str >= 10 && String.sub id_str 0 9 = "keccak256:" then
    Some KECCAK256
  else
    None
*)

(** Convert content hash to string representation *)
let content_hash_to_string (ch: content_hash) : string =
  let algo_str = match ch.algorithm with
    | SHA256 -> "sha256"
    | BLAKE2B -> "blake2b"
    | KECCAK256 -> "keccak256"
  in
  algo_str ^ ":" ^ ch.digest

(** Compare two content hashes for equality *)
let content_hash_equal (ch1: content_hash) (ch2: content_hash) : bool =
  ch1.algorithm = ch2.algorithm && String.equal ch1.digest ch2.digest

(** Get the default hash algorithm used by the system *)
let default_algorithm () : hash_algorithm =
  SHA256 (* Or could be configurable *)
