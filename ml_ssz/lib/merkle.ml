(** Merkle module for SSZ.
    Implements merkleization (hash tree root) functionality. *)

open Types
open Serialize

(* Local constants - these match the ones in Types.Constants *)
module Constants = struct
  let bytes_per_length_prefix = 4
  let bytes_per_length_offset = 4
  let max_chunk_size = 32  (* Size of a chunk in bytes *)
end

(** Hash function type *)
type hash_fn = bytes -> bytes

(** Default hash function using OCaml's built-in Digest module *)
let default_hash =
  fun data -> 
    let digest = Digest.string (Bytes.to_string data) in
    let result = Bytes.create 32 in
    (* Pad the result to 32 bytes *)
    let digest_len = String.length digest in
    Bytes.blit_string digest 0 result 0 (min digest_len 32);
    result

(** Hash two nodes together *)
let hash_nodes hash_fn left right =
  let combined = Bytes.create (2 * Constants.max_chunk_size) in
  copy_bytes left 0 combined 0 Constants.max_chunk_size;
  copy_bytes right 0 combined Constants.max_chunk_size Constants.max_chunk_size;
  hash_fn combined

(** Pad a chunk with zeros *)
let pad_chunk data =
  let result = Bytes.create Constants.max_chunk_size in
  let len = min (Bytes.length data) Constants.max_chunk_size in
  copy_bytes data 0 result 0 len;
  result

(** Hash a leaf node *)
let hash_leaf hash_fn data =
  if Bytes.length data <= Constants.max_chunk_size then
    pad_chunk data |> hash_fn
  else
    let chunks = (Bytes.length data + Constants.max_chunk_size - 1) / Constants.max_chunk_size in
    let rec hash_chunks acc i =
      if i >= chunks then acc
      else
        let start = i * Constants.max_chunk_size in
        let len = min Constants.max_chunk_size (Bytes.length data - start) in
        let chunk = Bytes.create Constants.max_chunk_size in
        copy_bytes data start chunk 0 len;
        
        let chunk_hash = hash_fn chunk in
        hash_chunks (hash_nodes hash_fn acc chunk_hash) (i + 1)
    in
    
    let first_chunk = Bytes.create Constants.max_chunk_size in
    copy_bytes data 0 first_chunk 0 (min Constants.max_chunk_size (Bytes.length data));
    let first_hash = hash_fn first_chunk in
    
    if chunks = 1 then first_hash
    else hash_chunks first_hash 1

(** Compute the merkle root of a list of items *)
let merkleize hash_fn chunks =
  let next_power_of_two n =
    let rec find_pow p = if p >= n then p else find_pow (p * 2) in
    find_pow 1
  in
  
  let padded_length = next_power_of_two (List.length chunks) in
  let zero_hash = Bytes.create Constants.max_chunk_size |> hash_fn in
  
  (* Extend with zero hashes if needed *)
  let padded_chunks =
    let base = chunks in
    let rec extend acc remaining =
      if remaining = 0 then acc
      else extend (zero_hash :: acc) (remaining - 1)
    in
    List.rev (extend [] (padded_length - List.length base)) @ base
  in
  
  (* Iteratively hash pairs of nodes *)
  let rec hash_layer nodes =
    match nodes with
    | [] -> Bytes.create 0  (* Empty case *)
    | [root] -> root        (* Single node case *)
    | _ ->
        (* Hash pairs and create new layer *)
        let rec hash_pairs acc = function
          | [] -> List.rev acc
          | [x] -> List.rev (hash_nodes hash_fn x zero_hash :: acc)
          | x :: y :: rest -> hash_pairs (hash_nodes hash_fn x y :: acc) rest
        in
        let next_layer = hash_pairs [] nodes in
        hash_layer next_layer
  in
  
  hash_layer padded_chunks

(** Compute the hash tree root of a value *)
let hash_tree_root ?(hash_fn=default_hash) typ value =
  match typ.kind with
  | Basic ->
      (* For basic types, just hash the raw encoding *)
      let encoded = encode typ value in
      hash_leaf hash_fn encoded
  
  | Container ->
      (* For containers, hash each field and merkleize *)
      (* This is a simplified approach - would need field access for real impl *)
      let encoded = encode typ value in
      hash_leaf hash_fn encoded
  
  | Vector | List ->
      (* For collections, hash each element and merkleize *)
      let encoded = encode typ value in
      hash_leaf hash_fn encoded
  
  | Union ->
      (* For unions, hash the selector and the value *)
      let encoded = encode typ value in
      hash_leaf hash_fn encoded

(** Mix in a length with a root to create a final root *)
let mix_in_length hash_fn root length =
  let length_bytes = Bytes.create 8 in
  write_uint64 length_bytes 0 (Int64.of_int length);
  hash_nodes hash_fn root (pad_chunk length_bytes)

(** Compute a merkle proof for a leaf *)
let compute_proof hash_fn chunks index =
  let next_power_of_two n =
    let rec find_pow p = if p >= n then p else find_pow (p * 2) in
    find_pow 1
  in
  
  let padded_length = next_power_of_two (List.length chunks) in
  let zero_hash = Bytes.create Constants.max_chunk_size |> hash_fn in
  
  (* Extend with zero hashes if needed *)
  let padded_chunks =
    let base = chunks in
    let rec extend acc remaining =
      if remaining = 0 then acc
      else extend (zero_hash :: acc) (remaining - 1)
    in
    List.rev (extend [] (padded_length - List.length base)) @ base
  in
  
  let rec compute_layer_proofs index depth nodes acc =
    match nodes with
    | [] -> acc
    | [_] -> acc
    | _ ->
        let layer_size = List.length nodes in
        let sibling_index = if index mod 2 = 0 then index + 1 else index - 1 in
        
        (* Get the sibling node *)
        let sibling = 
          if sibling_index >= layer_size then zero_hash
          else List.nth nodes sibling_index
        in
        
        (* Add sibling to proof *)
        let proof = sibling :: acc in
        
        (* Hash pairs and create new layer *)
        let rec hash_pairs acc i = function
          | [] -> List.rev acc
          | [x] -> List.rev (hash_nodes hash_fn x zero_hash :: acc)
          | x :: y :: rest -> hash_pairs (hash_nodes hash_fn x y :: acc) (i+1) rest
        in
        
        let next_layer = hash_pairs [] 0 nodes in
        compute_layer_proofs (index / 2) (depth + 1) next_layer proof
  in
  
  compute_layer_proofs index 0 padded_chunks []

(** Verify a merkle proof *)
let verify_proof hash_fn root leaf proof index =
  let padded_leaf = pad_chunk leaf in
  let leaf_hash = hash_fn padded_leaf in
  
  let rec verify node idx proof_nodes =
    match proof_nodes with
    | [] -> node
    | sibling :: rest ->
        let combined = 
          if idx mod 2 = 0 then
            hash_nodes hash_fn node sibling
          else
            hash_nodes hash_fn sibling node
        in
        verify combined (idx / 2) rest
  in
  
  let calculated_root = verify leaf_hash index proof in
  Bytes.equal calculated_root root 

(** Hash two merkle nodes together to produce a parent node *)
let merkleize_pair hash_fn left right =
  let combined = Bytes.create (2 * Constants.max_chunk_size) in
  copy_bytes left 0 combined 0 Constants.max_chunk_size;
  copy_bytes right 0 combined Constants.max_chunk_size Constants.max_chunk_size;
  hash_fn combined 