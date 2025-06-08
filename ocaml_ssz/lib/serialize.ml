(** * Serialization module for SSZ * * Provides core functions for encoding and
    decoding Simple Serialize (SSZ) data * with utility functions for reading
    and writing various integer types. *)

(* Use Types from the external types.ml *)
open Types

(* Local constants - these match the ones in Types.Constants *)
module Constants = struct
  let bytes_per_length_prefix = 4
  let bytes_per_length_offset = 4
end

(*-----------------------------------------------------------------------------
 * Core Serialization Functions
 *---------------------------------------------------------------------------*)

(** * Encode a value using its type specification *)
let encode typ value = typ.encode value

(** Decode a value using its type specification.

    @param typ The type specification to use for decoding
    @param bytes The bytes buffer containing the encoded data
    @return The decoded value *)
let decode typ bytes =
  let value, _ = typ.decode bytes 0 in
  value

(*-----------------------------------------------------------------------------
 * Integer Serialization Primitives
 *---------------------------------------------------------------------------*)

(** * Helper to write a uint8 at an offset in a bytes buffer *)
let write_uint8 buf offset value = Bytes.set buf offset (Char.chr value)

(** * Helper to write a uint16 at an offset in a bytes buffer *)
let write_uint16 buf offset value =
  write_uint8 buf offset (value land 0xFF);
  write_uint8 buf (offset + 1) ((value lsr 8) land 0xFF)

(** Helper to write a uint32 at an offset in a bytes buffer *)
let write_uint32 buf offset value =
  write_uint16 buf offset (value land 0xFFFF);
  write_uint16 buf (offset + 2) ((value lsr 16) land 0xFFFF)

(** Helper to write a uint64 at an offset in a bytes buffer *)
let write_uint64 buf offset value =
  write_uint32 buf offset (Int64.to_int (Int64.logand value 0xFFFFFFFFL));
  write_uint32 buf (offset + 4)
    (Int64.to_int (Int64.shift_right_logical value 32))

(** * Helper to read a uint8 from an offset in a bytes buffer *)
let read_uint8 buf offset = Char.code (Bytes.get buf offset)

(** * Helper to read a uint16 from an offset in a bytes buffer *)
let read_uint16 buf offset =
  let lo = read_uint8 buf offset in
  let hi = read_uint8 buf (offset + 1) in
  lo lor (hi lsl 8)

(** Helper to read a uint32 from an offset in a bytes buffer *)
let read_uint32 buf offset =
  let lo = read_uint16 buf offset in
  let hi = read_uint16 buf (offset + 2) in
  lo lor (hi lsl 16)

(** Helper to read a uint64 from an offset in a bytes buffer *)
let read_uint64 buf offset =
  let lo = Int64.of_int (read_uint32 buf offset) in
  let hi = Int64.of_int (read_uint32 buf (offset + 4)) in
  Int64.logor lo (Int64.shift_left hi 32)

(*-----------------------------------------------------------------------------
 * Buffer Operations
 *---------------------------------------------------------------------------*)

(** Copy bytes from one buffer to another.

    @param src Source buffer
    @param src_offset Starting offset in the source buffer
    @param dst Destination buffer
    @param dst_offset Starting offset in the destination buffer
    @param len Number of bytes to copy *)
let copy_bytes src src_offset dst dst_offset len =
  for i = 0 to len - 1 do
    Bytes.set dst (dst_offset + i) (Bytes.get src (src_offset + i))
  done

(*-----------------------------------------------------------------------------
 * Collection Serialization
 *---------------------------------------------------------------------------*)

(** Helper to encode a list with length prefix.

    @param typ Type specification for list elements
    @param values List of values to encode
    @return Bytes buffer containing length-prefixed encoded data *)
let encode_list_with_length typ values =
  let count = List.length values in
  let items_bytes = List.map (encode typ) values in
  let total_size =
    List.fold_left (fun acc bytes -> acc + Bytes.length bytes) 0 items_bytes
  in
  let result = Bytes.create (Constants.bytes_per_length_prefix + total_size) in

  (* Write length prefix *)
  write_uint32 result 0 count;

  (* Copy each item *)
  let offset = ref Constants.bytes_per_length_prefix in
  List.iter
    (fun item_bytes ->
      let len = Bytes.length item_bytes in
      copy_bytes item_bytes 0 result !offset len;
      offset := !offset + len)
    items_bytes;

  result

(** Helper to decode a list with length prefix *)
let decode_list_with_length typ bytes offset =
  let length = read_uint32 bytes offset in
  let offset = offset + Constants.bytes_per_length_prefix in

  let rec read_items acc remaining new_offset =
    if remaining = 0 then (List.rev acc, new_offset)
    else
      let item, next_offset = typ.decode bytes new_offset in
      read_items (item :: acc) (remaining - 1) next_offset
  in

  read_items [] length offset
