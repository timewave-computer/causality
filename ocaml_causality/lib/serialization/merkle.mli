(* Purpose: Merkle Tree Module Interface *)

open Ocaml_causality_core
open Content_addressing (* To get hash_algorithm type *)

(** Direction in a Merkle proof *)
type direction = Left | Right

(** Single node in a Merkle proof *)
type proof_node = {
  direction: direction;    (* Which side this hash is on *)
  hash: string;            (* The hash value *)
}

(** Merkle proof, a path from leaf to root *)
type merkle_proof = {
  leaf_hash: string;       (* Hash of the leaf node *)
  proof_nodes: proof_node list; (* Nodes in the proof path *)
  root_hash: string;       (* Hash of the root node *)
}

(** Merkle tree node type - Abstracted in the interface *)
type merkle_node

(** Complete Merkle tree - Abstracted in the interface *)
type merkle_tree

(* ------------ MERKLE TREE CONSTRUCTION ------------ *)

(** [create_tree items algo] creates a Merkle tree from a list of byte [items]
    using the specified hash [algo]. *)
val create_tree : bytes list -> hash_algorithm -> merkle_tree

(* ------------ MERKLE PROOF GENERATION ------------ *)

(** [generate_proof tree data] generates a Merkle proof for the given [data]
    item within the [tree]. Returns [None] if the data is not in the tree. *)
val generate_proof : merkle_tree -> bytes -> merkle_proof option

(* ------------ MERKLE PROOF VERIFICATION ------------ *)

(** [verify_proof proof algo] verifies a [merkle_proof] against the expected
    [root_hash] (contained within the proof) using the specified hash [algo]. *)
val verify_proof : merkle_proof -> hash_algorithm -> bool

(* ------------ MERKLE TREE UTILITIES ------------ *)

(** [root_hash tree] returns the root hash of the Merkle [tree]. *)
val root_hash : merkle_tree -> string

(** [contains tree data] checks if the given [data] item is present in the
    Merkle [tree] by verifying its hash. *)
val contains : merkle_tree -> bytes -> bool

(** [leaf_data tree] extracts all leaf data (as bytes) from the Merkle [tree].
    The order might not be preserved depending on tree construction. *)
val leaf_data : merkle_tree -> bytes list

(* Note: find_path is an internal helper and not exposed. *)
