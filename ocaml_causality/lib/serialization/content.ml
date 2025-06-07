(* ------------ CONTENT ADDRESSING ------------ *)
(* Purpose: Content addressing and hashing *)

open Ocaml_causality_core

(* ------------ HASHING ------------ *)

(* Simple hash function using OCaml's built-in Digest module *)
let hash_bytes (data: bytes) : bytes =
  let digest = Digest.bytes data in
  Bytes.of_string digest

let hash_string (data: string) : bytes =
  let digest = Digest.string data in
  Bytes.of_string digest

let hash_lisp_value (value: lisp_value) : bytes =
  let rec serialize_value = function
    | Unit -> "unit"
    | Bool true -> "true"
    | Bool false -> "false"
    | Int i -> "int:" ^ Int64.to_string i
    | String s -> "string:" ^ s
    | Symbol s -> "symbol:" ^ s
    | List l -> "list:[" ^ String.concat ";" (List.map serialize_value l) ^ "]"
    | ResourceId rid -> "resource:" ^ Bytes.to_string rid
    | ExprId eid -> "expr:" ^ Bytes.to_string eid
    | Bytes b -> "bytes:" ^ Bytes.to_string b
  in
  hash_string (serialize_value value)

(* ------------ CONTENT ADDRESSING ------------ *)

(* Content addressing functions *)
let content_address_of_value (value: lisp_value) : bytes =
  hash_lisp_value value

let content_address_of_resource (resource: resource) : bytes =
  let resource_data = Printf.sprintf "resource:%s:%s:%s:%Ld:%Ld"
    (Bytes.to_string resource.id)
    resource.name
    resource.resource_type
    resource.quantity
    resource.timestamp
  in
  hash_string resource_data

let content_address_of_intent (intent: intent) : bytes =
  let intent_data = Printf.sprintf "intent:%s:%s:%s:%d:%Ld"
    (Bytes.to_string intent.id)
    intent.name
    (Bytes.to_string intent.domain_id)
    intent.priority
    intent.timestamp
  in
  hash_string intent_data

let content_address_of_effect (effect: effect) : bytes =
  let effect_data = Printf.sprintf "effect:%s:%s:%s:%s:%Ld"
    (Bytes.to_string effect.id)
    effect.name
    (Bytes.to_string effect.domain_id)
    effect.effect_type
    effect.timestamp
  in
  hash_string effect_data

(* ------------ VERIFICATION ------------ *)

(* Content verification functions *)
let verify_content_address (data: bytes) (expected_address: bytes) : bool =
  let computed_address = hash_bytes data in
  Bytes.equal computed_address expected_address

let verify_value_address (value: lisp_value) (expected_address: bytes) : bool =
  let computed_address = content_address_of_value value in
  Bytes.equal computed_address expected_address

let verify_resource_address (resource: resource) (expected_address: bytes) : bool =
  let computed_address = content_address_of_resource resource in
  Bytes.equal computed_address expected_address

(* ------------ UTILITIES ------------ *)

(* Content addressing utilities *)
let address_to_string (address: bytes) : string =
  let hex_chars = "0123456789abcdef" in
  let len = Bytes.length address in
  let result = Bytes.create (len * 2) in
  for i = 0 to len - 1 do
    let byte = Bytes.get_uint8 address i in
    let high = byte lsr 4 in
    let low = byte land 0x0f in
    Bytes.set_uint8 result (i * 2) (Char.code hex_chars.[high]);
    Bytes.set_uint8 result (i * 2 + 1) (Char.code hex_chars.[low])
  done;
  Bytes.to_string result

let address_from_string (hex_string: string) : bytes option =
  let len = String.length hex_string in
  if len mod 2 <> 0 then None
  else
    try
      let result = Bytes.create (len / 2) in
      for i = 0 to (len / 2) - 1 do
        let high_char = hex_string.[i * 2] in
        let low_char = hex_string.[i * 2 + 1] in
        let high_val = match high_char with
          | '0'..'9' -> Char.code high_char - Char.code '0'
          | 'a'..'f' -> Char.code high_char - Char.code 'a' + 10
          | 'A'..'F' -> Char.code high_char - Char.code 'A' + 10
          | _ -> failwith "Invalid hex character"
        in
        let low_val = match low_char with
          | '0'..'9' -> Char.code low_char - Char.code '0'
          | 'a'..'f' -> Char.code low_char - Char.code 'a' + 10
          | 'A'..'F' -> Char.code low_char - Char.code 'A' + 10
          | _ -> failwith "Invalid hex character"
        in
        Bytes.set_uint8 result i ((high_val lsl 4) lor low_val)
      done;
      Some result
    with
    | _ -> None

let compare_addresses (addr1: bytes) (addr2: bytes) : int =
  Bytes.compare addr1 addr2

let is_valid_address (address: bytes) : bool =
  Bytes.length address = 16  (* MD5 hash length *)

(* Content addressing registry *)
module ContentRegistry = struct
  type t = {
    mutable value_addresses: (bytes * lisp_value) list;
    mutable resource_addresses: (bytes * resource) list;
    mutable intent_addresses: (bytes * intent) list;
    mutable effect_addresses: (bytes * effect) list;
  }

  let create () = {
    value_addresses = [];
    resource_addresses = [];
    intent_addresses = [];
    effect_addresses = [];
  }

  let register_value registry value =
    let address = content_address_of_value value in
    registry.value_addresses <- (address, value) :: registry.value_addresses;
    address

  let register_resource registry resource =
    let address = content_address_of_resource resource in
    registry.resource_addresses <- (address, resource) :: registry.resource_addresses;
    address

  let register_intent registry intent =
    let address = content_address_of_intent intent in
    registry.intent_addresses <- (address, intent) :: registry.intent_addresses;
    address

  let register_effect registry effect =
    let address = content_address_of_effect effect in
    registry.effect_addresses <- (address, effect) :: registry.effect_addresses;
    address

  let lookup_value registry address =
    List.assoc_opt address registry.value_addresses

  let lookup_resource registry address =
    List.assoc_opt address registry.resource_addresses

  let lookup_intent registry address =
    List.assoc_opt address registry.intent_addresses

  let lookup_effect registry address =
    List.assoc_opt address registry.effect_addresses
end

(* Default content registry *)
let default_registry = ContentRegistry.create () 