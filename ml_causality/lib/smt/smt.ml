(* smt.ml
 * Simplified OCaml SMT implementation - consolidated from multiple modules
 * Provides key-value storage with SHA256 hashing for TEG/Causality system
 *)

open Digestif

(* Core types *)
type hash = bytes
type smt_backend = (string, bytes) Hashtbl.t

(* Hash utilities *)
let hash_len = 32

let empty_hash () = Bytes.create hash_len

let hash_from_bytes b = 
  if Bytes.length b = hash_len then b
  else failwith "Invalid hash length"

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

(* SHA256 hashing *)
let sha256_hash data =
  let digest = SHA256.digest_bytes data in
  let str = SHA256.to_raw_string digest in
  Bytes.of_string str

let sha256_key context data =
  let ctx_bytes = Bytes.of_string context in
  let combined = Bytes.create (Bytes.length ctx_bytes + Bytes.length data) in
  Bytes.blit ctx_bytes 0 combined 0 (Bytes.length ctx_bytes);
  Bytes.blit data 0 combined (Bytes.length ctx_bytes) (Bytes.length data);
  sha256_hash combined

(* Unified SMT implementation *)
type t = {
  backend: smt_backend;
  root: hash;
}

let create () =
  let backend = Hashtbl.create 1024 in
  let empty_root = empty_hash () in
  { backend; root = empty_root }

let store smt key data =
  let new_root = sha256_key key data in
  Hashtbl.replace smt.backend key data;
  { smt with root = new_root }

let get smt key =
  Hashtbl.find_opt smt.backend key

let has smt key =
  Hashtbl.mem smt.backend key

let remove smt key =
  Hashtbl.remove smt.backend key;
  { smt with root = empty_hash () }

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

(* Content-addressable storage *)
let store_content_addressed smt domain_id entity_type data =
  let content_hash = sha256_hash data in
  let content_id = hash_to_hex content_hash in
  let key = teg_key domain_id entity_type content_id in
  let updated_smt = store smt key data in
  (updated_smt, content_id)

let get_content_addressed smt domain_id entity_type content_id =
  let key = teg_key domain_id entity_type content_id in
  get smt key

(* Cross-domain references *)
let store_cross_domain_ref smt source_domain target_domain target_entity_id ref_data =
  let key = Printf.sprintf "%s-cross-domain-%s-%s" source_domain target_domain target_entity_id in
  let updated_smt = store smt key ref_data in
  (updated_smt, key)

let get_cross_domain_ref smt source_domain target_domain target_entity_id =
  let key = Printf.sprintf "%s-cross-domain-%s-%s" source_domain target_domain target_entity_id in
  get smt key

(* Temporal relationships *)
let store_temporal_rel smt domain_id from_entity to_entity rel_type rel_data =
  let key = Printf.sprintf "%s-temporal-%s-%s-%s" domain_id from_entity to_entity rel_type in
  let updated_smt = store smt key rel_data in
  (updated_smt, key)

let get_temporal_rel smt domain_id from_entity to_entity rel_type =
  let key = Printf.sprintf "%s-temporal-%s-%s-%s" domain_id from_entity to_entity rel_type in
  get smt key

(* Batch operations *)
let batch_store smt key_data_pairs =
  List.fold_left (fun acc_smt (key, data) ->
    store acc_smt key data
  ) smt key_data_pairs

(* Utility functions for TEG entity types *)
let store_teg_effect = store_teg_data
let store_teg_handler = store_teg_data  
let store_teg_resource = store_teg_data
let store_teg_intent = store_teg_data
let store_teg_constraint = store_teg_data

let get_teg_effect = get_teg_data
let get_teg_handler = get_teg_data
let get_teg_resource = get_teg_data
let get_teg_intent = get_teg_data
let get_teg_constraint = get_teg_data

(* Compatibility functions *)
let teg_effect_to_smt_key domain_id effect_data =
  let effect_hash = sha256_hash effect_data in
  teg_key domain_id "effect" (hash_to_hex effect_hash)

let teg_handler_to_smt_key domain_id handler_data =
  let handler_hash = sha256_hash handler_data in
  teg_key domain_id "handler" (hash_to_hex handler_hash)

let teg_resource_to_smt_key domain_id resource_data =
  let resource_hash = sha256_hash resource_data in
  teg_key domain_id "resource" (hash_to_hex resource_hash)

let teg_intent_to_smt_key domain_id intent_data =
  let intent_hash = sha256_hash intent_data in
  teg_key domain_id "intent" (hash_to_hex intent_hash)

let teg_constraint_to_smt_key domain_id constraint_data =
  let constraint_hash = sha256_hash constraint_data in
  teg_key domain_id "constraint" (hash_to_hex constraint_hash)

let content_addressable_teg_key domain_id entity_type data =
  let data_hash = sha256_hash data in
  teg_key domain_id entity_type (hash_to_hex data_hash) 