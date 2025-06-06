(** OCaml bindings for Rust SSZ implementation 
    This module provides the FFI bindings to call Rust SSZ functions from OCaml.
*)

(** Mock implementations for testing without the actual Rust library *)
module Mock = struct
  (** Mock implementation of boolean serialization *)
  let serialize_bool b = if b then "\001" else "\000"
  
  (** Mock implementation of boolean deserialization *)
  let deserialize_bool s = 
    if String.length s = 0 then false
    else s.[0] <> '\000'
    
  (** Mock implementation of uint32 serialization *)
  let serialize_u32 n =
    let b = Bytes.create 4 in
    for i = 0 to 3 do
      Bytes.set b i (Char.chr ((n lsr (i * 8)) land 0xff))
    done;
    Bytes.to_string b
    
  (** Mock implementation of uint32 deserialization *)
  let deserialize_u32 s =
    if String.length s < 4 then 0 else
    let n = ref 0 in
    for i = 0 to 3 do
      n := !n lor ((Char.code (String.get s i)) lsl (i * 8))
    done;
    !n
    
  (** Mock implementation of string serialization *)
  let serialize_string s =
    let len = String.length s in
    let len_bytes = Bytes.create 4 in
    for i = 0 to 3 do
      Bytes.set len_bytes i (Char.chr ((len lsr (i * 8)) land 0xff))
    done;
    (Bytes.to_string len_bytes) ^ s
    
  (** Mock implementation of string deserialization *)
  let deserialize_string s =
    if String.length s < 4 then "" else
    let len = ref 0 in
    for i = 0 to 3 do
      len := !len lor ((Char.code (String.get s i)) lsl (i * 8))
    done;
    if String.length s < 4 + !len then "" else
    String.sub s 4 !len
    
  (** Mock implementation of simple hash function *)
  let simple_hash s =
    let hash = ref 0 in
    String.iter (fun c -> hash := (!hash * 31 + Char.code c) land 0xFFFFFFFF) s;
    let result = Bytes.create 32 in
    for i = 0 to 7 do
      let value = (!hash lsr (i * 4)) land 0xF in
      for j = 0 to 3 do
        Bytes.set result (i * 4 + j) (Char.chr value)
      done
    done;
    Bytes.to_string result
  
  (** Mock roundtrip functions *)
  let roundtrip_bool b = deserialize_bool (serialize_bool b)
  let roundtrip_u32 n = deserialize_u32 (serialize_u32 n)
  let roundtrip_string s = deserialize_string (serialize_string s)
end

(** 
 * In production, these would call the actual Rust functions via FFI.
 * For now, we're using the mock implementations in tests.
 *)
let rust_serialize_bool = Mock.serialize_bool
let rust_deserialize_bool = Mock.deserialize_bool
let rust_serialize_u32 = Mock.serialize_u32
let rust_deserialize_u32 = Mock.deserialize_u32
let rust_serialize_string = Mock.serialize_string
let rust_deserialize_string = Mock.deserialize_string
let rust_simple_hash = Mock.simple_hash
let rust_roundtrip_bool = Mock.roundtrip_bool
let rust_roundtrip_u32 = Mock.roundtrip_u32
let rust_roundtrip_string = Mock.roundtrip_string

(** Mock round-trip functions for comprehensive testing *)
let ocaml_to_rust_bytes data = data  (* Mock: just pass through *)
let rust_to_ocaml_bytes data = data  (* Mock: just pass through *)
let rust_to_ocaml_to_rust_bytes data = data  (* Mock: just pass through *)

(** Hash tree root compatibility check *)
let check_hash_compatibility data =
  let ocaml_hash = "placeholder_for_ocaml_hash" in (* To be implemented *)
  let rust_hash = rust_simple_hash data in
  (ocaml_hash, rust_hash) 