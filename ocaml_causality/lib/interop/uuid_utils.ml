(* Purpose: UUID utilities for interoperability *)

(* open Uuidm *)

(* Temporarily stub out UUID functions to avoid Uuidm dependency *)

(** Generate a new UUID v4 as bytes *)
let generate_uuid_v4 () : bytes =
  (* Simplified random bytes instead of proper UUID *)
  let random_bytes = Bytes.create 16 in
  for i = 0 to 15 do
    Bytes.set_uint8 random_bytes i (Random.int 256)
  done;
  random_bytes

(** Convert UUID bytes to standard string format *)
let uuid_to_string (uuid_bytes: bytes) : string =
  (* Simple hex representation instead of proper UUID format *)
  let hex_chars = "0123456789abcdef" in
  let len = Bytes.length uuid_bytes in
  let result = Bytes.create (len * 2) in
  for i = 0 to len - 1 do
    let byte = Bytes.get_uint8 uuid_bytes i in
    Bytes.set result (i * 2) hex_chars.[byte lsr 4];
    Bytes.set result (i * 2 + 1) hex_chars.[byte land 0xf];
  done;
  Bytes.to_string result

(** Parse UUID string back to bytes *)
let uuid_from_string (uuid_str: string) : bytes option =
  try
    let len = String.length uuid_str in
    if len mod 2 <> 0 then None
    else
      let result = Bytes.create (len / 2) in
      for i = 0 to (len / 2) - 1 do
        let hex_pair = String.sub uuid_str (i * 2) 2 in
        let byte_val = int_of_string ("0x" ^ hex_pair) in
        Bytes.set_uint8 result i byte_val
      done;
      Some result
  with
  | _ -> None
