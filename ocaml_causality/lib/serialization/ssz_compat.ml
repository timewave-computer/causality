(* ------------ SSZ COMPATIBILITY ------------ *)
(* Purpose: SSZ serialization compatibility for OCaml Causality types *)

(* ------------ TYPE DEFINITIONS ------------ *)

(** SSZ-compatible type wrapper *)
type 'a ssz_compatible = {
  value : 'a;
  ssz_type : string;
  is_fixed_length : bool;
}

(** SSZ serialization error *)
exception SszError of string

(** SSZ type registry for dynamic type handling *)
type ssz_type_registry = (string, (bytes -> bytes) * (bytes -> bytes)) Hashtbl.t

let global_ssz_registry : ssz_type_registry = Hashtbl.create 64

(* ------------ BASIC SSZ OPERATIONS ------------ *)

(** Register an SSZ type with serialization/deserialization functions *)
let register_ssz_type (type_name : string) (serialize : bytes -> bytes) (deserialize : bytes -> bytes) =
  Hashtbl.replace global_ssz_registry type_name (serialize, deserialize)

(** Serialize a value using registered SSZ type *)
let serialize_with_type (type_name : string) (data : bytes) : bytes =
  match Hashtbl.find_opt global_ssz_registry type_name with
  | Some (serialize, _) -> serialize data
  | None -> raise (SszError ("Unknown SSZ type: " ^ type_name))

(** Deserialize a value using registered SSZ type *)
let deserialize_with_type (type_name : string) (data : bytes) : bytes =
  match Hashtbl.find_opt global_ssz_registry type_name with
  | Some (_, deserialize) -> deserialize data
  | None -> raise (SszError ("Unknown SSZ type: " ^ type_name))

(* ------------ COMPATIBILITY FUNCTIONS ------------ *)

(** Convert OCaml bytes to SSZ-compatible format *)
let to_ssz_bytes (data : bytes) : bytes =
  (* Ensure proper alignment and padding for SSZ *)
  let len = Bytes.length data in
  let padded_len = ((len + 31) / 32) * 32 in (* Align to 32-byte chunks *)
  let result = Bytes.create padded_len in
  Bytes.blit data 0 result 0 len;
  (* Fill remaining bytes with zeros *)
  for i = len to padded_len - 1 do
    Bytes.set_uint8 result i 0;
  done;
  result

(** Convert SSZ bytes back to OCaml format *)
let from_ssz_bytes (data : bytes) : bytes =
  (* Remove padding and return original data *)
  let len = Bytes.length data in
  (* Find the actual length by looking for the last non-zero byte *)
  let rec find_actual_length pos =
    if pos < 0 then 0
    else if Bytes.get_uint8 data pos <> 0 then pos + 1
    else find_actual_length (pos - 1)
  in
  let actual_len = find_actual_length (len - 1) in
  let result = Bytes.create actual_len in
  Bytes.blit data 0 result 0 actual_len;
  result

(** Create SSZ-compatible wrapper *)
let make_ssz_compatible (value : 'a) (type_name : string) (is_fixed : bool) : 'a ssz_compatible =
  { value; ssz_type = type_name; is_fixed_length = is_fixed }

(** Extract value from SSZ wrapper *)
let extract_ssz_value (wrapper : 'a ssz_compatible) : 'a =
  wrapper.value

(** Serialize entity ID to SSZ format *)
let serialize_entity_id (id : bytes) : bytes =
  if Bytes.length id <> 32 then
    raise (SszError "Entity ID must be exactly 32 bytes");
  to_ssz_bytes id

(** Deserialize entity ID from SSZ format *)
let deserialize_entity_id (data : bytes) : bytes =
  let unpadded = from_ssz_bytes data in
  if Bytes.length unpadded <> 32 then
    raise (SszError "Invalid entity ID length after deserialization");
  unpadded

(** Serialize string to SSZ format *)
let serialize_string (s : string) : bytes =
  let data = Bytes.of_string s in
  let len = Bytes.length data in
  let len_bytes = Bytes.create 4 in
  Bytes.set_int32_le len_bytes 0 (Int32.of_int len);
  let result = Bytes.create (4 + len) in
  Bytes.blit len_bytes 0 result 0 4;
  Bytes.blit data 0 result 4 len;
  to_ssz_bytes result

(** Deserialize string from SSZ format *)
let deserialize_string (data : bytes) : string =
  let unpadded = from_ssz_bytes data in
  if Bytes.length unpadded < 4 then
    raise (SszError "Invalid string data: too short");
  let len = Int32.to_int (Bytes.get_int32_le unpadded 0) in
  if Bytes.length unpadded < 4 + len then
    raise (SszError "Invalid string data: length mismatch");
  let str_data = Bytes.create len in
  Bytes.blit unpadded 4 str_data 0 len;
  Bytes.to_string str_data

(** Serialize integer to SSZ format *)
let serialize_int (i : int) : bytes =
  let data = Bytes.create 8 in
  Bytes.set_int64_le data 0 (Int64.of_int i);
  to_ssz_bytes data

(** Deserialize integer from SSZ format *)
let deserialize_int (data : bytes) : int =
  let unpadded = from_ssz_bytes data in
  if Bytes.length unpadded < 8 then
    raise (SszError "Invalid integer data");
  Int64.to_int (Bytes.get_int64_le unpadded 0)

(** Serialize boolean to SSZ format *)
let serialize_bool (b : bool) : bytes =
  let data = Bytes.create 1 in
  Bytes.set_uint8 data 0 (if b then 1 else 0);
  to_ssz_bytes data

(** Deserialize boolean from SSZ format *)
let deserialize_bool (data : bytes) : bool =
  let unpadded = from_ssz_bytes data in
  if Bytes.length unpadded < 1 then
    raise (SszError "Invalid boolean data");
  Bytes.get_uint8 unpadded 0 <> 0

(* ------------ VALIDATION FUNCTIONS ------------ *)

(** Validate SSZ data format *)
let validate_ssz_format (data : bytes) : bool =
  try
    let len = Bytes.length data in
    (* Check if length is aligned to 32-byte chunks *)
    len mod 32 = 0
  with
  | _ -> false

(** Validate SSZ type compatibility *)
let validate_ssz_type (type_name : string) : bool =
  Hashtbl.mem global_ssz_registry type_name

(** Check if data is properly SSZ-encoded *)
let is_valid_ssz_encoding (data : bytes) (expected_type : string) : bool =
  validate_ssz_format data && validate_ssz_type expected_type

(** Verify SSZ data integrity *)
let verify_ssz_integrity (data : bytes) : bool =
  try
    let _ = from_ssz_bytes data in
    true
  with
  | SszError _ -> false
  | _ -> false

(* ------------ INTEGRATION HELPERS ------------ *)

(** Convert Causality types to SSZ-compatible format *)
module CausalityToSsz = struct
  (** Convert entity ID *)
  let entity_id (id : bytes) : bytes =
    serialize_entity_id id

  (** Convert domain ID *)
  let domain_id (id : bytes) : bytes =
    serialize_entity_id id

  (** Convert handler ID *)
  let handler_id (id : bytes) : bytes =
    serialize_entity_id id

  (** Convert timestamp *)
  let timestamp (ts : int64) : bytes =
    let data = Bytes.create 8 in
    Bytes.set_int64_le data 0 ts;
    to_ssz_bytes data

  (** Convert string *)
  let string (s : string) : bytes =
    serialize_string s
end

(** Convert SSZ data back to Causality types *)
module SszToCausality = struct
  (** Convert to entity ID *)
  let entity_id (data : bytes) : bytes =
    deserialize_entity_id data

  (** Convert to domain ID *)
  let domain_id (data : bytes) : bytes =
    deserialize_entity_id data

  (** Convert to handler ID *)
  let handler_id (data : bytes) : bytes =
    deserialize_entity_id data

  (** Convert to timestamp *)
  let timestamp (data : bytes) : int64 =
    let unpadded = from_ssz_bytes data in
    if Bytes.length unpadded < 8 then
      raise (SszError "Invalid timestamp data");
    Bytes.get_int64_le unpadded 0

  (** Convert to string *)
  let string (data : bytes) : string =
    deserialize_string data
end

(* ------------ INITIALIZATION ------------ *)

(** Initialize SSZ compatibility with standard types *)
let initialize_ssz_compatibility () =
  (* Wrapper functions to match bytes -> bytes signature *)
  let string_serialize_wrapper (data : bytes) : bytes =
    serialize_string (Bytes.to_string data)
  in
  let string_deserialize_wrapper (data : bytes) : bytes =
    Bytes.of_string (deserialize_string data)
  in
  let int_serialize_wrapper (data : bytes) : bytes =
    if Bytes.length data >= 8 then
      serialize_int (Int64.to_int (Bytes.get_int64_le data 0))
    else
      serialize_int 0
  in
  let int_deserialize_wrapper (data : bytes) : bytes =
    let result = Bytes.create 8 in
    Bytes.set_int64_le result 0 (Int64.of_int (deserialize_int data));
    result
  in
  let bool_serialize_wrapper (data : bytes) : bytes =
    if Bytes.length data >= 1 then
      serialize_bool (Bytes.get_uint8 data 0 <> 0)
    else
      serialize_bool false
  in
  let bool_deserialize_wrapper (data : bytes) : bytes =
    let result = Bytes.create 1 in
    Bytes.set_uint8 result 0 (if deserialize_bool data then 1 else 0);
    result
  in
  
  register_ssz_type "entity_id" serialize_entity_id deserialize_entity_id;
  register_ssz_type "string" string_serialize_wrapper string_deserialize_wrapper;
  register_ssz_type "int" int_serialize_wrapper int_deserialize_wrapper;
  register_ssz_type "bool" bool_serialize_wrapper bool_deserialize_wrapper;
  Printf.printf "SSZ compatibility initialized with %d types\n" 
    (Hashtbl.length global_ssz_registry)

(* Initialize on module load *)
let () = initialize_ssz_compatibility ()
