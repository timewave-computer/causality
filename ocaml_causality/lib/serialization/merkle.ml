(*
 * Merkle Tree Module
 *
 * This module provides functionality for creating and verifying Merkle trees
 * and proofs. Merkle trees are used for efficient and secure verification
 * of data integrity in distributed systems.
 *)

open Ocaml_causality_core
open Content_addressing

(* Re-export hash_algorithm type from Content_addressing *)
type hash_algorithm = Content_addressing.hash_algorithm

(* ------------ MERKLE TREE TYPES ------------ *)

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

(** Merkle tree node type *)
type merkle_node =
  | Leaf of { hash: string; data: bytes option }
  | Node of { hash: string; left: merkle_node; right: merkle_node }

(** Complete Merkle tree *)
type merkle_tree = {
  root: merkle_node;       (* Root node of the tree *)
  leaf_count: int;         (* Number of leaf nodes *)
  height: int;             (* Height of the tree *)
  hash_algorithm: hash_algorithm; (* Hash algorithm used *)
}

(* ------------ MERKLE TREE CONSTRUCTION ------------ *)

(** Create a Merkle tree from a list of data items *)
let create_tree (items: bytes list) (algo: hash_algorithm) : merkle_tree =
  (* If no items, create an empty tree *)
  if items = [] then
    { root = Leaf { hash = ""; data = None };
      leaf_count = 0;
      height = 0;
      hash_algorithm = algo }
  else
    (* First, hash all leaf nodes *)
    let leaf_nodes = List.map (fun data ->
      let hash_obj = hash_bytes algo data in
      Leaf { hash = hash_obj.digest; data = Some data }
    ) items in
    
    (* Ensure leaf count is a power of 2 by padding with empty leaves *)
    let rec next_power_of_2 n =
      if n <= 1 then 1
      else 2 * next_power_of_2 ((n + 1) / 2)
    in
    
    let target_count = next_power_of_2 (List.length leaf_nodes) in
    let empty_hash = (hash_bytes algo (Bytes.of_string "")).digest in
    
    let padded_leaf_nodes = 
      let pad_count = target_count - List.length leaf_nodes in
      leaf_nodes @ List.init pad_count (fun _ -> 
        Leaf { hash = empty_hash; data = None }
      )
    in
    
    (* Build the tree from the bottom up *)
    let rec build_level nodes =
      match nodes with
      | [] -> []  (* Empty level *)
      | [single] -> [single]  (* Odd node at the end *)
      | left_node :: right_node :: rest ->
          (* Combine each pair of nodes *)
          let get_node_hash n =
            match n with
            | Leaf { hash; _ } -> hash
            | Node { hash; _ } -> hash
          in
          let combined_hash_str =
            let lh = get_node_hash left_node in
            let rh = get_node_hash right_node in
            lh ^ rh
          in
          let combined_hash = 
            (hash_bytes algo (Bytes.of_string combined_hash_str)).digest
          in
          let new_merkle_node = Node { 
            hash = combined_hash; 
            left = left_node; 
            right = right_node 
          } in
          new_merkle_node :: build_level rest
    in
    
    (* Build the tree until we reach a single root node *)
    let rec build_tree level height =
      match level with
      | [root] -> (root, height)  (* We've reached the root *)
      | _ -> 
          let new_level = build_level level in
          build_tree new_level (height + 1)
    in
    
    let (root, height) = build_tree padded_leaf_nodes 0 in
    
    { root; 
      leaf_count = List.length items;
      height; 
      hash_algorithm = algo }

(* ------------ MERKLE PROOF GENERATION ------------ *)

(** Generate a Merkle proof for a specific data item *)
let generate_proof (tree: merkle_tree) (data: bytes) : merkle_proof option =
  (* First, hash the data to get the leaf hash *)
  let leaf_hash = (hash_bytes tree.hash_algorithm data).digest in
  
  (* Find the path from root to the leaf *)
  let rec find_proof_path node target_hash proof =
    match node with
    | Leaf { hash; _ } ->
        if hash = target_hash then Some (List.rev proof)
        else None
    | Node { hash = _node_hash; left; right } -> (* Changed hash to _node_hash to avoid confusion *)
        (* Try left branch *)
        let get_node_hash n =
          match n with
          | Leaf { hash; _ } -> hash
          | Node { hash; _ } -> hash
        in
        let left_result = find_proof_path left target_hash 
          ((Leaf { hash = get_node_hash right; (* Use helper *)
                  data = None }, 
            Right) :: proof) in (* Note: Storing Leaf for proof, this might need to be just hash *)
        
        match left_result with
        | Some p -> Some p
        | None ->
            (* Try right branch *)
            find_proof_path right target_hash 
              ((Leaf { hash = get_node_hash left; (* Use helper *)
                     data = None }, 
                Left) :: proof)
  in
  
  match find_proof_path tree.root leaf_hash [] with
  | Some path_nodes ->
      (* Convert the path nodes to proof nodes *)
      let proof_nodes = List.map (fun (node, dir) ->
        match node with
        | Leaf { hash; _ } -> { direction = dir; hash }
        | Node { hash; _ } -> { direction = dir; hash }
      ) path_nodes in
      
      (* Get the root hash *)
      let root_hash = match tree.root with
        | Leaf { hash; _ } -> hash
        | Node { hash; _ } -> hash
      in
      
      Some { leaf_hash; proof_nodes; root_hash }
  | None -> None

(* ------------ MERKLE PROOF VERIFICATION ------------ *)

(** Verify a Merkle proof against a root hash *)
let verify_proof (proof: merkle_proof) (algo: hash_algorithm) : bool =
  let rec verify current_hash nodes =
    match nodes with
    | [] -> current_hash = proof.root_hash
    | node :: rest ->
        (* Combine the current hash with the proof node hash *)
        let combined = match node.direction with
          | Left -> node.hash ^ current_hash   (* Node hash is on the left *)
          | Right -> current_hash ^ node.hash  (* Node hash is on the right *)
        in
        let new_hash = (hash_bytes algo (Bytes.of_string combined)).digest in
        verify new_hash rest
  in
  
  verify proof.leaf_hash proof.proof_nodes

(* ------------ MERKLE TREE UTILITIES ------------ *)

(** Get the root hash of a Merkle tree *)
let root_hash (tree: merkle_tree) : string =
  match tree.root with
  | Leaf { hash; _ } -> hash
  | Node { hash; _ } -> hash

(** Check if a data item is in the Merkle tree *)
let contains (tree: merkle_tree) (data: bytes) : bool =
  let target_hash = (hash_bytes tree.hash_algorithm data).digest in
  
  let rec search node =
    match node with
    | Leaf { hash; _ } -> hash = target_hash
    | Node { left; right; _ } -> search left || search right
  in
  
  search tree.root

(** Convert a Merkle tree to a list of all leaf data *)
let leaf_data (tree: merkle_tree) : bytes list =
  let result = ref [] in
  
  let rec collect node =
    match node with
    | Leaf { data = Some d; _ } -> result := d :: !result
    | Leaf { data = None; _ } -> ()
    | Node { left; right; _ } -> 
        collect left;
        collect right
  in
  
  collect tree.root;
  List.rev !result
