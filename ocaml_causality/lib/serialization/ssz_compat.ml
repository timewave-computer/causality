(*
 * SSZ Compatibility Module
 *
 * This module provides compatibility with other Simple Serialize (SSZ) 
 * implementations, particularly those used in the Ethereum ecosystem.
 * It enables interoperability with Ethereum 2.0 and other systems
 * that use SSZ for serialization.
 *)

open Ocaml_causality_core

(* ------------ TYPE DEFINITIONS ------------ *)

(** SSZ basic types *)
type ssz_basic_type = 
  | UInt8        (* 8-bit unsigned integer *)
  | UInt16       (* 16-bit unsigned integer *)
  | UInt32       (* 32-bit unsigned integer *)
  | UInt64       (* 64-bit unsigned integer *)
  | Bool         (* Boolean value *)
  | Bytes32      (* Fixed-length 32-byte array *)
  | Address      (* Ethereum address (20 bytes) *)

(** SSZ schema type *)
type ssz_type =
  | Basic of ssz_basic_type        (* Basic type *)
  | Vector of ssz_type * int       (* Fixed-length array *)
  | List of ssz_type * int         (* Variable-length array with max size *)
  | Container of (string * ssz_type) list (* Struct-like container *)

(** SSZ serialization/deserialization error *)
type ssz_error =
  | InvalidLength of string        (* Invalid length *)
  | InvalidOffset of string        (* Invalid offset *)
  | InvalidType of string          (* Invalid type *)
  | ParseError of string           (* Error parsing SSZ data *)
  | OverflowError of string        (* Value overflow *)

(* ------------ BASIC TYPE ENCODING ------------ *)

(** Encode a uint8 as SSZ bytes *)
let encode_uint8 (v: int) : bytes =
  if v < 0 || v > 255 then
    failwith "Value out of range for uint8"
  else
    Bytes.make 1 (Char.chr v)

(** Encode a uint16 as SSZ bytes (little-endian) *)
let encode_uint16 (v: int) : bytes =
  if v < 0 || v > 65535 then
    failwith "Value out of range for uint16"
  else
    let buf = Bytes.create 2 in
    Bytes.set buf 0 (Char.chr (v land 0xFF));
    Bytes.set buf 1 (Char.chr ((v lsr 8) land 0xFF));
    buf

(** Encode a uint32 as SSZ bytes (little-endian) *)
let encode_uint32 (v: int32) : bytes =
  let buf = Bytes.create 4 in
  Bytes.set_int32_le buf 0 v;
  buf

(** Encode a uint64 as SSZ bytes (little-endian) *)
let encode_uint64 (v: int64) : bytes =
  let buf = Bytes.create 8 in
  Bytes.set_int64_le buf 0 v;
  buf

(** Encode a boolean as SSZ bytes *)
let encode_bool (b: bool) : bytes =
  Bytes.make 1 (if b then '\001' else '\000')

(** Encode bytes32 (fixed 32-byte array) *)
let encode_bytes32 (b: bytes) : bytes =
  if Bytes.length b <> 32 then
    failwith "bytes32 must be exactly 32 bytes"
  else
    Bytes.copy b

(** Encode Ethereum address (20 bytes) *)
let encode_address (addr: bytes) : bytes =
  if Bytes.length addr <> 20 then
    failwith "Ethereum address must be exactly 20 bytes"
  else
    Bytes.copy addr

(* ------------ BASIC TYPE DECODING ------------ *)

(** Decode a uint8 from SSZ bytes *)
let decode_uint8 (data: bytes) (offset: int) : int * int =
  if offset >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    (Char.code (Bytes.get data offset), offset + 1)

(** Decode a uint16 from SSZ bytes (little-endian) *)
let decode_uint16 (data: bytes) (offset: int) : int * int =
  if offset + 1 >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    let low = Char.code (Bytes.get data offset) in
    let high = Char.code (Bytes.get data (offset + 1)) in
    (low lor (high lsl 8), offset + 2)

(** Decode a uint32 from SSZ bytes (little-endian) *)
let decode_uint32 (data: bytes) (offset: int) : int32 * int =
  if offset + 3 >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    (Bytes.get_int32_le data offset, offset + 4)

(** Decode a uint64 from SSZ bytes (little-endian) *)
let decode_uint64 (data: bytes) (offset: int) : int64 * int =
  if offset + 7 >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    (Bytes.get_int64_le data offset, offset + 8)

(** Decode a boolean from SSZ bytes *)
let decode_bool (data: bytes) (offset: int) : bool * int =
  if offset >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    (Bytes.get data offset <> '\000', offset + 1)

(** Decode bytes32 (fixed 32-byte array) *)
let decode_bytes32 (data: bytes) (offset: int) : bytes * int =
  if offset + 31 >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    let result = Bytes.create 32 in
    Bytes.blit data offset result 0 32;
    (result, offset + 32)

(** Decode Ethereum address (20 bytes) *)
let decode_address (data: bytes) (offset: int) : bytes * int =
  if offset + 19 >= Bytes.length data then
    failwith "Offset out of bounds"
  else
    let result = Bytes.create 20 in
    Bytes.blit data offset result 0 20;
    (result, offset + 20)

(* ------------ CONTAINER TYPE HANDLING ------------ *)

(** Calculate the fixed part size of a container type *)
let rec fixed_part_size (ty: ssz_type) : int =
  match ty with
  | Basic UInt8 -> 1
  | Basic UInt16 -> 2
  | Basic UInt32 -> 4
  | Basic UInt64 -> 8
  | Basic Bool -> 1
  | Basic Bytes32 -> 32
  | Basic Address -> 20
  | Vector (elem_type, length) ->
      let elem_size = fixed_part_size elem_type in
      if elem_size = 0 then 4 * length else elem_size * length
  | List (_, _) -> 4  (* Offset to the actual list data *)
  | Container fields ->
      let fixed_size = ref 0 in
      let has_variable = ref false in
      
      List.iter (fun (_, field_type) ->
        if fixed_part_size field_type = 0 then
          has_variable := true
        else if !has_variable then
          fixed_size := !fixed_size + 4  (* Offset *)
        else
          fixed_size := !fixed_size + fixed_part_size field_type
      ) fields;
      
      !fixed_size

(* ------------ ETH2 COMPATIBILITY ------------ *)

(** Convert OCaml Causality types to Ethereum 2.0 SSZ types *)
let rec to_eth2_ssz_type (ty: string) : ssz_type option =
  match ty with
  | "uint8" -> Some (Basic UInt8)
  | "uint16" -> Some (Basic UInt16)
  | "uint32" -> Some (Basic UInt32)
  | "uint64" -> Some (Basic UInt64)
  | "bool" -> Some (Basic Bool)
  | "bytes32" -> Some (Basic Bytes32)
  | "address" -> Some (Basic Address)
  | _ -> 
      (* Handle container types and other complex types *)
      if String.length ty > 7 && String.sub ty 0 7 = "Vector<" then
        (* Rough parsing of Vector<type, length> *)
        try
          let content = String.sub ty 7 (String.length ty - 8) in
          let comma_pos = String.index content ',' in
          let elem_type_str = String.sub content 0 comma_pos in
          let length_str = String.sub content (comma_pos + 1) 
                             (String.length content - comma_pos - 1) in
          match to_eth2_ssz_type elem_type_str with
          | Some elem_type -> Some (Vector (elem_type, int_of_string length_str))
          | None -> None
        with _ -> None
      else
        None

(** Create an Ethereum 2.0 compatible SSZ hash tree root *)
let eth2_hash_tree_root (data: bytes) (_ty: ssz_type) : string =
  (* For basic implementation, we'll just hash the data *)
  (* A real implementation would build a binary merkle tree according to Eth2 specs *)
  let open Digestif.SHA256 in
  digest_string (Bytes.to_string data)
  |> to_hex
