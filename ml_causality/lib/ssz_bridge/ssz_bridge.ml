(* Purpose: Bridge module integrating ml_ssz with ml_causality for FFI interactions *)

(* Enhanced error handling for FFI context *)
type serialization_result = 
  | Ok of string
  | Error of string

(* Basic SSZ serialization functions *)
module Basic = struct
  let serialize_bool b =
    if b then "\001" else "\000"
    
  let deserialize_bool s =
    match String.get s 0 with
    | '\001' -> true
    | '\000' -> false
    | _ -> failwith "Invalid bool encoding"
    
  let serialize_uint32 n =
    let b = Bytes.create 4 in
    for i = 0 to 3 do
      Bytes.set b i (Char.chr ((n lsr (i * 8)) land 0xff))
    done;
    Bytes.to_string b
    
  let deserialize_uint32 s =
    let n = ref 0 in
    for i = 0 to 3 do
      n := !n lor ((Char.code (String.get s i)) lsl (i * 8))
    done;
    !n
    
  let serialize_string s =
    let len = String.length s in
    let len_bytes = serialize_uint32 len in
    len_bytes ^ s
    
  let deserialize_string s =
    let len = deserialize_uint32 (String.sub s 0 4) in
    String.sub s 4 len
end

(* Re-export core SSZ functionality *)
module Core = struct
  include Basic
  
  let serialize_safe fn value =
    try Ok (fn value)
    with e -> Error (Printexc.to_string e)
    
  let deserialize_safe fn bytes =
    try Ok (fn bytes)
    with e -> Error (Printexc.to_string e)
end

(* FFI-compatible serialization functions *)
module Ffi = struct
  (* Simple hex conversion helpers *)
  let bytes_to_hex bytes =
    let hex_chars = "0123456789abcdef" in
    let len = String.length bytes in
    let hex = Bytes.create (len * 2) in
    for i = 0 to len - 1 do
      let byte = Char.code (String.get bytes i) in
      Bytes.set hex (i * 2) hex_chars.[byte lsr 4];
      Bytes.set hex (i * 2 + 1) hex_chars.[byte land 15]
    done;
    Bytes.to_string hex
  
  let hex_to_bytes hex_str =
    let len = String.length hex_str in
    if len mod 2 <> 0 then failwith "Invalid hex string length";
    let bytes = Bytes.create (len / 2) in
    for i = 0 to (len / 2) - 1 do
      let high = match String.get hex_str (i * 2) with
        | '0'..'9' as c -> Char.code c - Char.code '0'
        | 'a'..'f' as c -> Char.code c - Char.code 'a' + 10
        | 'A'..'F' as c -> Char.code c - Char.code 'A' + 10
        | _ -> failwith "Invalid hex character" in
      let low = match String.get hex_str (i * 2 + 1) with
        | '0'..'9' as c -> Char.code c - Char.code '0'
        | 'a'..'f' as c -> Char.code c - Char.code 'a' + 10
        | 'A'..'F' as c -> Char.code c - Char.code 'A' + 10
        | _ -> failwith "Invalid hex character" in
      Bytes.set bytes i (Char.chr ((high lsl 4) lor low))
    done;
    Bytes.to_string bytes

  (* Serialize to hex string for safe FFI transfer *)
  let serialize_to_hex serialize_fn value =
    try
      let bytes = serialize_fn value in
      let hex = bytes_to_hex bytes in
      Ok hex
    with e -> Error (Printexc.to_string e)
  
  (* Deserialize from hex string *)
  let deserialize_from_hex deserialize_fn hex_str =
    try
      let bytes = hex_to_bytes hex_str in
      let result = deserialize_fn bytes in
      Ok result
    with e -> Error (Printexc.to_string e)
  
  (* Round-trip test for validation *)
  let test_roundtrip serialize_fn deserialize_fn value =
    match serialize_to_hex serialize_fn value with
    | Ok hex ->
        (match deserialize_from_hex deserialize_fn hex with
         | Ok result -> 
             let success = String.equal (serialize_fn value) (serialize_fn result) in
             Ok (if success then "success" else "failure")
         | Error err -> Error ("Deserialize failed: " ^ err))
    | Error err -> Error ("Serialize failed: " ^ err)
end

(* Content addressing using SSZ *)
module ContentAddressing = struct
  (* Simple SHA-256 hash computation *)
  let compute_sha256 data =
    let ctx = Digestif.SHA256.empty in
    let ctx = Digestif.SHA256.feed_string ctx data in
    Digestif.SHA256.to_raw_string (Digestif.SHA256.get ctx)
    
  let compute_content_hash serialize_fn value =
    try
      let bytes = serialize_fn value in
      let hash = compute_sha256 bytes in
      Ok hash
    with e -> Error (Printexc.to_string e)
    
  let compute_content_hash_hex serialize_fn value =
    match compute_content_hash serialize_fn value with
    | Ok hash -> Ok (Ffi.bytes_to_hex hash)
    | Error err -> Error err
end

(* Version and feature information *)
let version = "0.2.0"
let ssz_enabled = true
let ffi_enabled = true

(* Bridge utilities *)
module Utils = struct
  (* Check if two serialized representations are identical *)
  let bytes_equal a b = String.equal a b
  
  (* Get serialization size *)
  let serialized_size serialize_fn value =
    try
      let bytes = serialize_fn value in
      Ok (string_of_int (String.length bytes))
    with e -> Error (Printexc.to_string e)
end 