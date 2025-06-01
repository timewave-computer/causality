(* Purpose: Simplified Sparse Merkle Tree (SMT) implementation. *)

open Digestif

(* Core types *)
type hash = bytes
type smt_backend = (string, bytes) Hashtbl.t

(* Hash utilities *)
let hash_len = 32

let empty_hash () = Bytes.make hash_len '\000' (* Initialize with null bytes *)

(* SHA256 hashing *)
let sha256_hash data :
  hash =
  let digest = SHA256.digest_bytes data in
  Bytes.of_string (SHA256.to_raw_string digest)

let hash_to_hex h = 
  let hex_char n = 
    if n < 10 then char_of_int (int_of_char '0' + n)
    else char_of_int (int_of_char 'a' + n - 10)
  in
  let result = Bytes.create (hash_len * 2) in
  for i = 0 to hash_len - 1 do
    let byte = Bytes.get_uint8 h i in
    let hi = byte lsr 4 in
    let lo = byte land 0x0f in
    Bytes.set result (i * 2) (hex_char hi);
    Bytes.set result (i * 2 + 1) (hex_char lo)
  done;
  Bytes.to_string result


(* Unified SMT implementation *)
type t = {
  backend: smt_backend;
  mutable root: hash; (* Root is now mutable as in typical SMTs *)
}

let create () =
  let backend = Hashtbl.create 1024 in
  let empty_root = empty_hash () in
  { backend; root = empty_root }

(* Internal helper to update root; simplified for this version.
   A proper SMT would recompute the Merkle path. *)
let update_root_hash key data =
  let key_bytes = Bytes.of_string key in
  let combined = Bytes.cat key_bytes data in (* Simplistic combination for root update *)
  sha256_hash combined

let store smt key data =
  Hashtbl.replace smt.backend key data;
  let new_root = update_root_hash key data in
  smt.root <- new_root;
  smt (* Return the modified SMT *)

let get smt key =
  Hashtbl.find_opt smt.backend key

let has smt key =
  Hashtbl.mem smt.backend key

let remove smt key =
  Hashtbl.remove smt.backend key;
  (* Per legacy behavior, reset root on remove. This is not standard SMT. *)
  smt.root <- empty_hash (); 
  smt

let get_root smt = smt.root

(* TEG-specific operations *)
let teg_key domain_id entity_type entity_id =
  Printf.sprintf "%s-%s-%s" domain_id entity_type entity_id

let store_teg_data smt domain_id entity_type entity_id data =
  let key = teg_key domain_id entity_type entity_id in
  let updated_smt = store smt key data in
  (updated_smt, key)

let get_teg_data smt domain_id entity_type entity_id =
  let key = teg_key domain_id entity_type entity_id in
  get smt key

let has_teg_data smt domain_id entity_type entity_id =
  let key = teg_key domain_id entity_type entity_id in
  has smt key

let store_content_addressed smt domain_id entity_type data =
  let content_hash = sha256_hash data in
  let content_id = hash_to_hex content_hash in
  let key = teg_key domain_id entity_type content_id in
  let updated_smt = store smt key data in
  (updated_smt, content_id)

let get_content_addressed smt domain_id entity_type content_id =
  let key = teg_key domain_id entity_type content_id in
  get smt key

(* Batch operations *)
let batch_store smt key_data_pairs =
  List.fold_left (fun acc_smt (key, data) ->
    store acc_smt key data
  ) smt key_data_pairs

let content_addressable_teg_key domain_id entity_type data =
  let data_hash = sha256_hash data in
  teg_key domain_id entity_type (hash_to_hex data_hash) 