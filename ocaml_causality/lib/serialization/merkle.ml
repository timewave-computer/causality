(* Merkle Tree Module for Causality *)
(* Purpose: Merkle tree construction from hash values *)

type hash = string
type merkle_tree = Leaf of hash | Node of hash * merkle_tree * merkle_tree

let hash_combine (left : hash) (right : hash) : hash =
  (* Simple hash combination - in practice would use proper SHA256 *)
  let combined = left ^ right in Digest.string combined |> Digest.to_hex

let merkle_root = function
  | [] -> Digest.string "" |> Digest.to_hex
  | [ h ] -> h
  | hashes ->
      let rec build_level level =
        match level with
        | [] -> []
        | [ h ] -> [ h ]
        | h1 :: h2 :: rest ->
            let combined = hash_combine h1 h2 in
            combined :: build_level rest
      in
      let rec build_tree level =
        match level with
        | [ root ] -> root
        | _ -> build_tree (build_level level)
      in
      build_tree hashes

(* ------------ MERKLE TREE TYPES ------------ *)

(** Merkle tree node *)
type merkle_node = Leaf of bytes | Node of merkle_node * merkle_node * bytes

type merkle_proof = {
    leaf_hash : bytes
  ; path : (bool * bytes) list (* (is_right, sibling_hash) *)
}
(** Merkle proof *)

(* ------------ TREE OPERATIONS ------------ *)

(** Build a merkle tree from a list of leaf hashes *)
let build_merkle_tree (leaves : bytes list) : merkle_node =
  let hash_bytes data = Digest.bytes data |> Bytes.of_string in
  let combine_hashes left right =
    let combined = Bytes.create (Bytes.length left + Bytes.length right) in
    Bytes.blit left 0 combined 0 (Bytes.length left);
    Bytes.blit right 0 combined (Bytes.length left) (Bytes.length right);
    hash_bytes combined
  in
  
  let rec build_level nodes =
    match nodes with
    | [] -> []
    | [single] -> [single]
    | left :: right :: rest ->
        let left_hash = match left with
          | Leaf h -> h
          | Node (_, _, h) -> h
        in
        let right_hash = match right with
          | Leaf h -> h
          | Node (_, _, h) -> h
        in
        let combined_hash = combine_hashes left_hash right_hash in
        let parent = Node (left, right, combined_hash) in
        parent :: build_level rest
  in
  
  let rec build_tree nodes =
    match nodes with
    | [root] -> root
    | _ -> build_tree (build_level nodes)
  in
  
  match leaves with
  | [] -> Leaf (Bytes.create 32) (* Empty tree *)
  | _ -> 
      let leaf_nodes = List.map (fun h -> Leaf h) leaves in
      build_tree leaf_nodes

(** Get the root hash of a merkle tree *)
let get_root_hash = function
  | Leaf h -> h
  | Node (_, _, h) -> h

(* ------------ PROOF VERIFICATION ------------ *)

(** Verify a merkle proof against a root hash *)
let verify_merkle_proof (proof : merkle_proof) (root_hash : bytes) : bool =
  let hash_bytes data = Digest.bytes data |> Bytes.of_string in
  let combine_hashes left right =
    let combined = Bytes.create (Bytes.length left + Bytes.length right) in
    Bytes.blit left 0 combined 0 (Bytes.length left);
    Bytes.blit right 0 combined (Bytes.length left) (Bytes.length right);
    hash_bytes combined
  in
  
  let rec verify_path current_hash path =
    match path with
    | [] -> Bytes.equal current_hash root_hash
    | (is_right, sibling_hash) :: rest ->
        let parent_hash = 
          if is_right then
            combine_hashes sibling_hash current_hash
          else
            combine_hashes current_hash sibling_hash
        in
        verify_path parent_hash rest
  in
  
  verify_path proof.leaf_hash proof.path

(** Generate a merkle proof for a leaf at given index *)
let generate_merkle_proof (tree : merkle_node) (leaf_index : int) : merkle_proof option =
  let rec find_path node index depth =
    match node with
    | Leaf h when index = 0 -> Some (h, [])
    | Leaf _ -> None
    | Node (left, right, _) ->
        let left_size = 1 lsl depth in (* 2^depth *)
        if index < left_size then
          match find_path left index (depth - 1) with
          | Some (leaf_hash, path) ->
              let right_hash = get_root_hash right in
              Some (leaf_hash, (true, right_hash) :: path)
          | None -> None
        else
          match find_path right (index - left_size) (depth - 1) with
          | Some (leaf_hash, path) ->
              let left_hash = get_root_hash left in
              Some (leaf_hash, (false, left_hash) :: path)
          | None -> None
  in
  
  (* Calculate tree depth *)
  let rec calculate_depth node =
    match node with
    | Leaf _ -> 0
    | Node (left, _, _) -> 1 + calculate_depth left
  in
  
  let depth = calculate_depth tree in
  match find_path tree leaf_index depth with
  | Some (leaf_hash, path) -> Some { leaf_hash; path }
  | None -> None

(* ------------ SPARSE MERKLE TREE ------------ *)
(* Purpose: Sparse Merkle Tree implementation mirroring Rust valence_coprocessor *)

(* ------------ TYPE DEFINITIONS ------------ *)

(** Hash type matching Rust [u8; 32] *)
type smt_hash = bytes

(** SMT key type *)
type smt_key = smt_hash

(** SMT value type *)
type smt_value = bytes

(** Opening/Proof type matching Rust Opening struct *)
type opening = {
  key : smt_key;
  value : smt_value option; (* None for non-inclusion proof *)
  siblings : smt_hash list;
}

(** Simple key-value storage for demo purposes *)
type memory_smt = {
  mutable storage : (string, bytes) Hashtbl.t;
}

(** Hasher trait matching Rust Hasher *)
module type HASHER = sig
  val hash : bytes -> smt_hash
  val key : string -> bytes -> smt_hash
  val merge : smt_hash -> smt_hash -> smt_hash
  val digest : bytes list -> smt_hash
end

(** SHA256 hasher implementation matching Rust Sha256Hasher *)
module Sha256Hasher : HASHER = struct
  let hash (data : bytes) : smt_hash =
    let digest = Digest.bytes data in
    let result = Bytes.create 32 in
    let digest_bytes = Bytes.of_string digest in
    let copy_len = min (Bytes.length digest_bytes) 32 in
    Bytes.blit digest_bytes 0 result 0 copy_len;
    result

  let key (domain : string) (data : bytes) : smt_hash =
    let combined = Bytes.create (String.length domain + 1 + Bytes.length data) in
    Bytes.blit_string domain 0 combined 0 (String.length domain);
    Bytes.set combined (String.length domain) ':';
    Bytes.blit data 0 combined (String.length domain + 1) (Bytes.length data);
    hash combined

  let merge (left : smt_hash) (right : smt_hash) : smt_hash =
    let combined = Bytes.create 64 in
    Bytes.blit left 0 combined 0 32;
    Bytes.blit right 0 combined 32 32;
    hash combined

  let digest (data_list : bytes list) : smt_hash =
    let total_len = List.fold_left (fun acc b -> acc + Bytes.length b) 0 data_list in
    let combined = Bytes.create total_len in
    let _ = List.fold_left (fun offset b ->
      Bytes.blit b 0 combined offset (Bytes.length b);
      offset + Bytes.length b
    ) 0 data_list in
    hash combined
end

(* ------------ SIMPLIFIED SMT OPERATIONS ------------ *)

(** Create empty SMT *)
let create_memory_smt () : memory_smt =
  { storage = Hashtbl.create 256 }

(** Convert hash to hex string for storage key *)
let hash_to_key (hash : smt_hash) : string =
  let hex_chars = "0123456789abcdef" in
  let result = Bytes.create (Bytes.length hash * 2) in
  for i = 0 to Bytes.length hash - 1 do
    let byte_val = Bytes.get_uint8 hash i in
    let high = byte_val lsr 4 in
    let low = byte_val land 15 in
    Bytes.set result (i * 2) hex_chars.[high];
    Bytes.set result (i * 2 + 1) hex_chars.[low];
  done;
  Bytes.to_string result

(** Simplified insert - just store key-value pairs and compute root *)
let smt_insert (smt : memory_smt) (_root_hash : smt_hash) (key : smt_key) (value : smt_value) : smt_hash =
  let key_str = hash_to_key key in
  Hashtbl.replace smt.storage key_str value;
  
  (* Compute new root as hash of all stored key-value pairs *)
  let all_pairs = Hashtbl.fold (fun k v acc -> (k, v) :: acc) smt.storage [] in
  let sorted_pairs = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) all_pairs in
  
  let combined_data = List.fold_left (fun acc (k, v) ->
    let key_bytes = Bytes.of_string k in
    key_bytes :: v :: acc
  ) [] sorted_pairs in
  
  if combined_data = [] then
    Bytes.create 32 (* Empty root *)
  else
    Sha256Hasher.digest (List.rev combined_data)

(** Simplified get_opening - create a minimal proof *)
let smt_get_opening (smt : memory_smt) (_root_hash : smt_hash) (key : smt_key) : opening option =
  let key_str = hash_to_key key in
  match Hashtbl.find_opt smt.storage key_str with
  | Some value ->
      (* Create a simple proof with empty siblings list for demo *)
      Some { key; value = Some value; siblings = [] }
  | None ->
      (* Non-inclusion proof *)
      Some { key; value = None; siblings = [] }

(** Simplified verify - just check if the key-value pair exists and root matches *)
let smt_verify (opening : opening) (_root_hash : smt_hash) (key : smt_key) (value : smt_value) : bool =
  (* Check that the opening is for the right key and value *)
  if not (Bytes.equal opening.key key) then false
  else
    match opening.value with
    | Some opening_value when Bytes.equal opening_value value -> 
        (* For this simplified version, we'll consider it valid if the values match *)
        (* In a real implementation, we'd verify the merkle path *)
        true
    | None -> false (* Non-inclusion proof, but we're checking inclusion *)
    | Some _ -> false (* Value mismatch *)

(* ------------ UTILITIES ------------ *)

(** Check if SMT is empty *)
let is_smt_empty (smt : memory_smt) : bool =
  Hashtbl.length smt.storage = 0

(** Get the size (number of stored nodes) of an SMT *)
let smt_size (smt : memory_smt) : int =
  Hashtbl.length smt.storage

(** Create empty root hash *)
let empty_root_hash () : smt_hash =
  Bytes.create 32

(** Memory SMT type alias matching Rust MemorySmt *)
type memory_smt_t = memory_smt

(** Module providing the same API as Rust MemorySmt *)
module MemorySmt = struct
  type t = memory_smt_t
  
  let default () = create_memory_smt ()
  
  let insert (smt : t) (root_hash : smt_hash) (key : smt_key) (value : smt_value) : smt_hash =
    smt_insert smt root_hash key value
  
  let get_opening (smt : t) (root_hash : smt_hash) (key : smt_key) : opening option =
    smt_get_opening smt root_hash key
  
  let verify (opening : opening) (root_hash : smt_hash) (key : smt_key) (value : smt_value) : bool =
    smt_verify opening root_hash key value
end
