(*
 * Content Addressing Module Interface
 *
 * This module provides functionality for content-based addressing and 
 * identification of data using cryptographic hashing.
 *)

open Ocaml_causality_core

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
val sha256_hash : bytes -> string

(** Compute BLAKE2b hash of byte data *)
val blake2b_hash : bytes -> string

(** Compute Keccak-256 hash of byte data *)
val keccak256_hash : bytes -> string

(** Compute hash using specified algorithm *)
val hash_bytes : hash_algorithm -> bytes -> content_hash

(* ------------ CONTENT ID GENERATION ------------ *)

(** Generate a content ID for entity identification *)
val generate_content_id : bytes -> entity_id

(** Generate a multi-hash content ID with algorithm prefix *)
val generate_multi_hash : hash_algorithm -> bytes -> entity_id

(* ------------ CONTENT VERIFICATION ------------ *)

(** Check if a content ID matches the content it's supposed to identify *)
val verify_content_id : entity_id -> bytes -> bool

(** Parse a content ID to extract algorithm and hash *)
val parse_content_id : entity_id -> (hash_algorithm * string) option

(* ------------ RESOURCE CONTENT ADDRESSING ------------ *)

(** Generate content ID for a resource based on its serialized form *)
val content_address_resource : Types.resource -> entity_id

(** Generate content ID for an expression *)
val content_address_expr : Ocaml_causality_lang.Ast.expr -> expr_id

(** Generate content ID for a value expression *)
val content_address_value : Ocaml_causality_core.Types.value_expr -> value_expr_id

(* ------------ NULLIFIER GENERATION ------------ *)

(** Generate a nullifier for a consumed resource *)
val generate_nullifier : entity_id -> bytes -> entity_id

(** Verify a nullifier against a resource *)
val verify_nullifier : entity_id -> entity_id -> bytes -> bool

(* ------------ UTILITIES ------------ *)

(** Convert content hash to string representation *)
val content_hash_to_string : content_hash -> string

(** Compare two content hashes for equality *)
val content_hash_equal : content_hash -> content_hash -> bool

(** Get the default hash algorithm used by the system *)
val default_algorithm : unit -> hash_algorithm