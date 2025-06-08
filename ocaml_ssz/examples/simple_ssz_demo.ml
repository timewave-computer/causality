(* Simple_ssz usage demonstration *)

(* Mock hash_tree_root_basic since it's not available in the standalone version *)
let hash_tree_root_basic data =
  (* Simple mock hash implementation *)
  let hash = Bytes.create 32 in
  for i = 0 to 31 do
    let b = if i < String.length data then Char.code data.[i] else 0 in
    Bytes.set hash i (Char.chr (b lxor (i * 7)))
  done;
  Bytes.to_string hash

(* Define a simple Person record *)
type person = { id : int; name : string; age : int; is_active : bool }

(* Create serialization functions for the Person type *)
let serialize_person (p : person) : string =
  (* Directly use Basic serialization functions *)
  let serialized_id = Ssz.Basic.serialize_uint32 p.id in
  let serialized_name = Ssz.Basic.serialize_string p.name in
  let serialized_age = Ssz.Basic.serialize_uint8 p.age in
  let serialized_active = Ssz.Basic.serialize_bool p.is_active in

  (* Combine all serialized fields *)
  serialized_id ^ serialized_name ^ serialized_age ^ serialized_active

(* Deserialize a Person record *)
let deserialize_person (s : string) : person =
  (* First extract the ID (fixed size 4 bytes) *)
  let id = Ssz.Basic.deserialize_uint32 (String.sub s 0 4) in

  (* Next extract the name (variable size with length prefix) *)
  let name =
    Ssz.Basic.deserialize_string (String.sub s 4 (String.length s - 4))
  in
  let name_size = 4 + String.length name in
  (* 4 bytes for length prefix *)

  (* Extract the age (fixed size 1 byte) *)
  let age_offset = 4 + name_size in
  let age = Ssz.Basic.deserialize_uint8 (String.sub s age_offset 1) in

  (* Extract the active status (fixed size 1 byte) *)
  let active_offset = age_offset + 1 in
  let is_active = Ssz.Basic.deserialize_bool (String.sub s active_offset 1) in

  (* Construct and return the person *)
  { id; name; age; is_active }

(* Calculate the Merkle root of a person *)
let person_merkle_root (p : person) : string =
  let serialized = serialize_person p in
  hash_tree_root_basic serialized

(* Demo main function *)
let () =
  (* Create a person *)
  let alice = { id = 42; name = "Alice"; age = 30; is_active = true } in

  Printf.printf "Original person: ID=%d, Name=%s, Age=%d, Active=%b\n" alice.id
    alice.name alice.age alice.is_active;

  (* Serialize the person *)
  let serialized = serialize_person alice in
  Printf.printf "Serialized to %d bytes\n" (String.length serialized);

  (* Calculate Merkle root *)
  let root = person_merkle_root alice in
  Printf.printf "Merkle root (hex): ";
  String.iter (fun c -> Printf.printf "%02x" (Char.code c)) root;
  Printf.printf "\n";

  (* Deserialize back to a person *)
  let deserialized = deserialize_person serialized in
  Printf.printf "Deserialized person: ID=%d, Name=%s, Age=%d, Active=%b\n"
    deserialized.id deserialized.name deserialized.age deserialized.is_active;

  (* Verify the round trip worked *)
  assert (alice.id = deserialized.id);
  assert (alice.name = deserialized.name);
  assert (alice.age = deserialized.age);
  assert (alice.is_active = deserialized.is_active);

  Printf.printf "Round trip serialization successful!\n"
