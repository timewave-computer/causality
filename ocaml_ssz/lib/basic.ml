(** * Basic SSZ types implementation module * * Implements serialization for
    primitive types like booleans, integers, * and other basic data types used
    in the SSZ serialization scheme. *)

open Types
open Serialize

(*-----------------------------------------------------------------------------
 * Basic Type Serializers
 *---------------------------------------------------------------------------*)

(* Local constants - these match the ones in Types.Constants *)
module Constants = struct
  let bytes_per_length_prefix = 4
  let bytes_per_length_offset = 4
end

(** Boolean type *)
let bool =
  {
    kind = Basic
  ; size = Some 1
  ; encode =
      (fun b ->
        let result = Bytes.create 1 in
        write_uint8 result 0 (if b then 1 else 0);
        result)
  ; decode =
      (fun bytes offset ->
        let value = read_uint8 bytes offset > 0 in
        (value, offset + 1))
  }

(** Unsigned 8-bit integer *)
let uint8 =
  {
    kind = Basic
  ; size = Some 1
  ; encode =
      (fun n ->
        let result = Bytes.create 1 in
        write_uint8 result 0 (n land 0xFF);
        result)
  ; decode =
      (fun bytes offset ->
        let value = read_uint8 bytes offset in
        (value, offset + 1))
  }

(** Unsigned 16-bit integer *)
let uint16 =
  {
    kind = Basic
  ; size = Some 2
  ; encode =
      (fun n ->
        let result = Bytes.create 2 in
        write_uint16 result 0 (n land 0xFFFF);
        result)
  ; decode =
      (fun bytes offset ->
        let value = read_uint16 bytes offset in
        (value, offset + 2))
  }

(** Unsigned 32-bit integer *)
let uint32 =
  {
    kind = Basic
  ; size = Some 4
  ; encode =
      (fun n ->
        let result = Bytes.create 4 in
        write_uint32 result 0 n;
        result)
  ; decode =
      (fun bytes offset ->
        let value = read_uint32 bytes offset in
        (value, offset + 4))
  }

(** Unsigned 64-bit integer *)
let uint64 =
  {
    kind = Basic
  ; size = Some 8
  ; encode =
      (fun n ->
        let result = Bytes.create 8 in
        write_uint64 result 0 n;
        result)
  ; decode =
      (fun bytes offset ->
        let value = read_uint64 bytes offset in
        (value, offset + 8))
  }

(** String type (variable length) *)
let string =
  {
    kind = List
  ; size = None
  ; encode =
      (fun s ->
        let bytes_len = String.length s in
        let result =
          Bytes.create (Constants.bytes_per_length_prefix + bytes_len)
        in
        write_uint32 result 0 bytes_len;

        (* Copy string content *)
        String.iteri
          (fun i c ->
            Bytes.set result (Constants.bytes_per_length_prefix + i) c)
          s;

        result)
  ; decode =
      (fun bytes offset ->
        let length = read_uint32 bytes offset in
        let offset = offset + Constants.bytes_per_length_prefix in

        (* Extract string *)
        let result = Bytes.create length in
        for i = 0 to length - 1 do
          Bytes.set result i (Bytes.get bytes (offset + i))
        done;

        (Bytes.to_string result, offset + length))
  }

(** Bytes type (variable length) *)
let bytes =
  {
    kind = List
  ; size = None
  ; encode =
      (fun b ->
        let bytes_len = Bytes.length b in
        let result =
          Bytes.create (Constants.bytes_per_length_prefix + bytes_len)
        in
        write_uint32 result 0 bytes_len;

        (* Copy bytes content *)
        copy_bytes b 0 result Constants.bytes_per_length_prefix bytes_len;

        result)
  ; decode =
      (fun bytes offset ->
        let length = read_uint32 bytes offset in
        let offset = offset + Constants.bytes_per_length_prefix in

        (* Extract bytes *)
        let result = Bytes.create length in
        copy_bytes bytes offset result 0 length;

        (result, offset + length))
  }

(** String serialization with length prefix *)
let serialize_string_with_length s =
  let bytes_len = String.length s in
  let result = Bytes.create (Constants.bytes_per_length_prefix + bytes_len) in

  write_uint32 result 0 bytes_len;

  (* Copy string content *)
  String.iteri
    (fun i c -> Bytes.set result (Constants.bytes_per_length_prefix + i) c)
    s;

  result

(** Simple string serialization helper *)
let serialize_string s =
  let encoded = string.encode s in
  Bytes.to_string encoded

(** Simple string deserialization helper *)
let deserialize_string s =
  let bytes = Bytes.of_string s in
  let decoded, _ = string.decode bytes 0 in
  decoded

(** Simple uint32 serialization helper *)
let serialize_uint32 n =
  let encoded = uint32.encode n in
  Bytes.to_string encoded

(** Simple uint32 deserialization helper *)
let deserialize_uint32 s =
  let bytes = Bytes.of_string s in
  let decoded, _ = uint32.decode bytes 0 in
  decoded

(** Simple bool serialization helper *)
let serialize_bool b =
  let encoded = bool.encode b in
  Bytes.to_string encoded

(** Simple bool deserialization helper *)
let deserialize_bool s =
  let bytes = Bytes.of_string s in
  let decoded, _ = bool.decode bytes 0 in
  decoded

(** Simple uint8 serialization helper *)
let serialize_uint8 n =
  let encoded = uint8.encode n in
  Bytes.to_string encoded

(** Simple uint8 deserialization helper *)
let deserialize_uint8 s =
  let bytes = Bytes.of_string s in
  let decoded, _ = uint8.decode bytes 0 in
  decoded
