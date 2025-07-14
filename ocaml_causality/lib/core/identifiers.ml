(* ------------ ENTITY IDENTIFIERS ------------ *)
(* Purpose: Entity IDs, content addressing primitives *)

(* ------------ BASIC TYPE ALIASES ------------ *)

type bytes = Bytes.t
(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's
    [u8; N] for IDs. *)

type str_t = string
(** Represents a string. Corresponds to Rust's Str or String. *)

type timestamp = int64
(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to
    Rust's Timestamp. *)

(* ------------ IDENTIFIER TYPES ------------ *)

type expr_id = bytes
(** Unique identifier for an expression. Corresponds to Rust's ExprId. *)

type value_expr_id = bytes
(** Unique identifier for a value expression. (Note: In Rust, ExprId often
    serves for content-addressed values too) *)

type entity_id = bytes
(** Generic unique identifier for an entity. Corresponds to Rust's EntityId. *)

type domain_id = bytes
(** Unique identifier for a domain. Corresponds to Rust's DomainId. *)

type handler_id = bytes
(** Unique identifier for a handler. Corresponds to Rust's HandlerId. *)

type edge_id = bytes
(** Unique identifier for an edge. Corresponds to Rust's EdgeId. *)

type node_id = bytes
(** Unique identifier for a node. Corresponds to Rust's NodeId. *)

(* ------------ CONTENT ADDRESSING ------------ *)

(** Content addressing trait for creating deterministic IDs *)
module ContentAddressing = struct
  (** Hash function for content addressing *)
  let hash_content (content : bytes) : bytes =
    let digest = Digest.bytes content in
    let result = Bytes.create 32 in
    let digest_bytes = Bytes.of_string digest in
    let copy_len = min (Bytes.length digest_bytes) 32 in
    Bytes.blit digest_bytes 0 result 0 copy_len;
    result

  (** Create content-addressed ID from string *)
  let from_string (content : string) : entity_id =
    hash_content (Bytes.of_string content)

  (** Create content-addressed ID from bytes *)
  let from_bytes (content : bytes) : entity_id =
    hash_content content

  (** Create content-addressed ID with domain prefix *)
  let from_domain_content (domain : string) (content : string) : entity_id =
    let combined = domain ^ ":" ^ content in
    from_string combined

  (** Create deterministic ID from multiple components *)
  let from_components (components : string list) : entity_id =
    let combined = String.concat "|" components in
    from_string combined
end

(* ------------ ID GENERATION ------------ *)

(** Generate a new expression ID from content *)
let generate_expr_id (content : string) : expr_id =
  ContentAddressing.from_domain_content "expr" content

(** Generate a new value expression ID *)
let generate_value_expr_id (value_repr : string) : value_expr_id =
  ContentAddressing.from_domain_content "value" value_repr

(** Generate a new entity ID *)
let generate_entity_id (entity_type : string) (entity_data : string) : entity_id =
  ContentAddressing.from_components [entity_type; entity_data]

(** Generate a new domain ID *)
let generate_domain_id (domain_name : string) : domain_id =
  ContentAddressing.from_domain_content "domain" domain_name

(** Generate a new handler ID *)
let generate_handler_id (handler_name : string) (handler_signature : string) : handler_id =
  ContentAddressing.from_components ["handler"; handler_name; handler_signature]

(** Generate a new edge ID *)
let generate_edge_id (from_node : string) (to_node : string) (edge_type : string) : edge_id =
  ContentAddressing.from_components ["edge"; from_node; to_node; edge_type]

(** Generate a new node ID *)
let generate_node_id (node_type : string) (node_data : string) : node_id =
  ContentAddressing.from_components ["node"; node_type; node_data]

(** Generate ID from timestamp and content *)
let generate_timestamped_id (ts : timestamp) (content : string) : entity_id =
  let timestamp_str = Int64.to_string ts in
  ContentAddressing.from_components [timestamp_str; content]

(** Generate random-looking but deterministic ID *)
let generate_deterministic_id (seed : string) : entity_id =
  ContentAddressing.from_domain_content "deterministic" seed

(* ------------ COMPARISON AND EQUALITY ------------ *)

(** Compare two IDs for equality *)
let equal_id (id1 : bytes) (id2 : bytes) : bool =
  Bytes.equal id1 id2

(** Compare two IDs lexicographically *)
let compare_id (id1 : bytes) (id2 : bytes) : int =
  Bytes.compare id1 id2

(** Check if an ID is the zero/empty ID *)
let is_zero_id (id : bytes) : bool =
  let zero_id = Bytes.create (Bytes.length id) in
  Bytes.equal id zero_id

(** Create a zero ID of specified length *)
let zero_id (length : int) : bytes =
  Bytes.create length

(** Standard 32-byte zero ID *)
let zero_entity_id () : entity_id =
  zero_id 32

(* ------------ SERIALIZATION ------------ *)

(** Convert ID to hex string *)
let id_to_hex (id : bytes) : string =
  let hex_chars = "0123456789abcdef" in
  let len = Bytes.length id in
  let result = Bytes.create (len * 2) in
  for i = 0 to len - 1 do
    let byte_val = Bytes.get_uint8 id i in
    let high = byte_val lsr 4 in
    let low = byte_val land 0x0f in
    Bytes.set result (i * 2) hex_chars.[high];
    Bytes.set result (i * 2 + 1) hex_chars.[low];
  done;
  Bytes.to_string result

(** Convert hex string to ID *)
let hex_to_id (hex : string) : bytes option =
  let len = String.length hex in
  if len mod 2 <> 0 then None
  else
    try
      let result = Bytes.create (len / 2) in
      for i = 0 to (len / 2) - 1 do
        let high_char = hex.[i * 2] in
        let low_char = hex.[i * 2 + 1] in
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
        Bytes.set_uint8 result i ((high_val lsl 4) lor low_val);
      done;
      Some result
    with
    | _ -> None

(** Convert ID to base64 string *)
let id_to_base64 (id : bytes) : string =
  (* Simple base64 encoding - in practice would use a proper library *)
  let base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" in
  let len = Bytes.length id in
  let output_len = ((len + 2) / 3) * 4 in
  let result = Bytes.create output_len in
  let rec encode_chunk input_pos output_pos =
    if input_pos >= len then ()
    else
      let chunk_size = min 3 (len - input_pos) in
      let chunk = Array.make 3 0 in
      for i = 0 to chunk_size - 1 do
        chunk.(i) <- Bytes.get_uint8 id (input_pos + i);
      done;
      
      let combined = (chunk.(0) lsl 16) lor (chunk.(1) lsl 8) lor chunk.(2) in
      for i = 0 to 3 do
        let char_index = (combined lsr (6 * (3 - i))) land 0x3f in
        if i < ((chunk_size * 8 + 5) / 6) then
          Bytes.set result (output_pos + i) base64_chars.[char_index]
        else
          Bytes.set result (output_pos + i) '=';
      done;
      encode_chunk (input_pos + 3) (output_pos + 4)
  in
  encode_chunk 0 0;
  Bytes.to_string result

(* ------------ UTILITIES ------------ *)

(** Get the first N bytes of an ID for short representation *)
let id_prefix (id : bytes) (n : int) : bytes =
  let len = min n (Bytes.length id) in
  let result = Bytes.create len in
  Bytes.blit id 0 result 0 len;
  result

(** Get short hex representation (first 8 characters) *)
let id_short_hex (id : bytes) : string =
  let prefix = id_prefix id 4 in
  id_to_hex prefix

(** Check if an ID has the expected length *)
let validate_id_length (id : bytes) (expected_length : int) : bool =
  Bytes.length id = expected_length

(** Validate that an ID is a proper 32-byte entity ID *)
let validate_entity_id (id : entity_id) : bool =
  validate_id_length id 32

(** Create ID from current timestamp *)
let current_timestamp_id () : entity_id =
  let current_time = Time_utils.current_time_int64_ms () in
  generate_timestamped_id current_time "current"

(** Hash multiple IDs together *)
let combine_ids (ids : bytes list) : entity_id =
  let combined_content = List.map id_to_hex ids |> String.concat "|" in
  ContentAddressing.from_string combined_content
