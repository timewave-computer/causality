(* SSZ demo *)

(* Import the module *)
open Ssz

(* Define a record to serialize *)
type person = {
  id: int;
  name: string;
  age: int;
  active: bool;
}

(* Serialize a person *)
let serialize_person (p: person) =
  (* Concatenate all fields *)
  let id_bytes = Basic.serialize_uint32 p.id in
  let name_bytes = Basic.serialize_string p.name in
  let age_bytes = Basic.serialize_uint8 p.age in
  let active_bytes = Basic.serialize_bool p.active in
  
  id_bytes ^ name_bytes ^ age_bytes ^ active_bytes

(* Deserialize a person *)
let deserialize_person (data: string) =
  (* Read id (fixed size 4 bytes) *)
  let id = Basic.deserialize_uint32 (String.sub data 0 4) in
  
  (* Read name (variable size with length prefix) *)
  let name = Basic.deserialize_string (String.sub data 4 (String.length data - 4)) in
  let name_size = 4 + String.length name in  (* 4 bytes for length prefix *)
  
  (* Read age (fixed size 1 byte) *)
  let age_offset = 4 + name_size in
  let age = Basic.deserialize_uint8 (String.sub data age_offset 1) in
  
  (* Read active flag (fixed size 1 byte) *)
  let active_offset = age_offset + 1 in
  let active = Basic.deserialize_bool (String.sub data active_offset 1) in
  
  { id; name; age; active }

(* Main function *)
let () =
  (* Create a person *)
  let alice = { id = 42; name = "Alice"; age = 30; active = true } in
  Printf.printf "Original: id=%d, name=%s, age=%d, active=%b\n" 
    alice.id alice.name alice.age alice.active;
  
  (* Serialize *)
  let serialized = serialize_person alice in
  Printf.printf "Serialized to %d bytes\n" (String.length serialized);
  
  (* Calculate Merkle root *)
  let root = hash_tree_root_basic serialized in
  Printf.printf "Merkle root: ";
  String.iter (fun c -> Printf.printf "%02x" (Char.code c)) (String.sub root 0 8);
  Printf.printf "...\n";
  
  (* Deserialize *)
  let deserialized = deserialize_person serialized in
  Printf.printf "Deserialized: id=%d, name=%s, age=%d, active=%b\n" 
    deserialized.id deserialized.name deserialized.age deserialized.active;
  
  (* Verify roundtrip *)
  assert (alice.id = deserialized.id);
  assert (alice.name = deserialized.name);
  assert (alice.age = deserialized.age);
  assert (alice.active = deserialized.active);
  
  Printf.printf "Roundtrip successful!\n" 