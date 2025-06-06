(** SSZ serialization for TEL (Transaction Expression Language) types *)

(** Vector size constants *)
let id_byte_len = 32
let str_byte_len = 32
let max_inputs = 128
let max_outputs = 128
let max_constraints = 32

(** SSZ ID representation *)
type ssz_id = {
  bytes : string; (** 32-byte ID *)
}

(** SSZ Output Definition for Effect outputs *)
type ssz_output_definition = {
  id : ssz_id;
  type_expr : ssz_id;
}

(** SSZ Effect container *)
type ssz_effect = {
  id : ssz_id;
  domain : ssz_id;
  intent_id : ssz_id;
  effect_type : string; (** Fixed-size Vector for Str, limit to str_byte_len *)
  dynamic_expr_present : bool;
  dynamic_expr : ssz_id option;
  inputs : ssz_id list; (** Limited to max_inputs *)
  outputs : ssz_output_definition list; (** Limited to max_outputs *)
  constraints : ssz_id list; (** Limited to max_constraints *)
  scoped_handler_present : bool;
  scoped_handler : ssz_id option;
}

(** SSZ Handler container *)
type ssz_handler = {
  id : ssz_id;
  domain : ssz_id;
  effect_type : string; (** Fixed-size Vector for Str, limit to str_byte_len *)
  constraints : ssz_id list; (** Limited to max_constraints *)
  dynamic_expr : ssz_id;
  priority : int; (** uint8 *)
  cost : int64; (** uint64 *)
  ephemeral : bool;
}

(** SSZ Edge container *)
type ssz_edge = {
  id : ssz_id;
  source : ssz_id;
  target : ssz_id;
  edge_type : int; (** uint8 *)
  condition_present : bool;
  condition : ssz_id option;
}

(** Helper functions *)

(** Convert string to fixed-length string, padding or truncating as needed *)
let to_fixed_len_string str len =
  let result = Bytes.create len in
  let str_len = String.length str in
  let copy_len = min str_len len in
  String.blit str 0 result 0 copy_len;
  if copy_len < len then
    for i = copy_len to len - 1 do
      Bytes.set result i '\000'
    done;
  Bytes.to_string result

(** Serialize a 32-byte ID *)
let serialize_id (id : string) : string =
  (* Ensure the ID is exactly 32 bytes *)
  to_fixed_len_string id id_byte_len

(** Deserialize a 32-byte ID *)
let deserialize_id (bytes : string) : string =
  (* Simple validation *)
  if String.length bytes <> id_byte_len then
    failwith (Printf.sprintf "Invalid ID length: expected %d, got %d" 
                id_byte_len (String.length bytes));
  bytes

(** Serialize/deserialize strings and IDs *)

let serialize_str (s : string) : string =
  to_fixed_len_string s str_byte_len

let deserialize_str (bytes : string) : string =
  (* Remove trailing zero bytes *)
  let rec find_nul_pos i =
    if i < 0 then String.length bytes
    else if bytes.[i] = '\000' then i
    else find_nul_pos (i - 1)
  in
  let end_pos = find_nul_pos (String.length bytes - 1) in
  String.sub bytes 0 end_pos

(** Serialize an ssz_id *)
let serialize_ssz_id (id : ssz_id) : string =
  id.bytes

(** Deserialize an ssz_id *)
let deserialize_ssz_id (bytes : string) : ssz_id =
  { bytes = deserialize_id bytes }

(** Serialize an output definition *)
let serialize_output_definition (od : ssz_output_definition) : string =
  serialize_ssz_id od.id ^ serialize_ssz_id od.type_expr

(** Deserialize an output definition *)
let deserialize_output_definition (bytes : string) : ssz_output_definition =
  if String.length bytes < id_byte_len * 2 then
    failwith "Invalid output definition bytes";
  {
    id = deserialize_ssz_id (String.sub bytes 0 id_byte_len);
    type_expr = deserialize_ssz_id (String.sub bytes id_byte_len id_byte_len);
  }

(** Create a new identifier *)
let create_id (s : string) : ssz_id =
  { bytes = serialize_id s }

(** Get the string representation of an ID *)
let id_to_string (id : ssz_id) : string =
  id.bytes

(** Serialize an effect *)
let serialize_effect (effect : ssz_effect) : string =
  let dynamic_expr_bytes = match effect.dynamic_expr with
    | Some expr -> serialize_ssz_id expr
    | None -> String.make id_byte_len '\000'
  in
  
  let scoped_handler_bytes = match effect.scoped_handler with
    | Some handler -> serialize_ssz_id handler
    | None -> String.make id_byte_len '\000'
  in
  
  (* Truncate lists to their maximum lengths *)
  let inputs = if List.length effect.inputs > max_inputs 
               then List.filteri (fun i _ -> i < max_inputs) effect.inputs
               else effect.inputs in
               
  let outputs = if List.length effect.outputs > max_outputs
                then List.filteri (fun i _ -> i < max_outputs) effect.outputs
                else effect.outputs in
                
  let constraints = if List.length effect.constraints > max_constraints
                    then List.filteri (fun i _ -> i < max_constraints) effect.constraints
                    else effect.constraints in
  
  (* Combine all fields *)
  serialize_ssz_id effect.id ^
  serialize_ssz_id effect.domain ^
  serialize_ssz_id effect.intent_id ^
  serialize_str effect.effect_type ^
  (if effect.dynamic_expr_present then "\001" else "\000") ^
  dynamic_expr_bytes ^
  String.concat "" (List.map serialize_ssz_id inputs) ^
  String.concat "" (List.map serialize_output_definition outputs) ^
  String.concat "" (List.map serialize_ssz_id constraints) ^
  (if effect.scoped_handler_present then "\001" else "\000") ^
  scoped_handler_bytes

(** Deserialize an effect - simplified implementation *)
let deserialize_effect (_bytes : string) : ssz_effect =
  (* Placeholder implementation *)
  {
    id = { bytes = String.make id_byte_len '\000' };
    domain = { bytes = String.make id_byte_len '\000' };
    intent_id = { bytes = String.make id_byte_len '\000' };
    effect_type = "";
    dynamic_expr_present = false;
    dynamic_expr = None;
    inputs = [];
    outputs = [];
    constraints = [];
    scoped_handler_present = false;
    scoped_handler = None;
  }

(** Serialize a handler - simplified implementation *)
let serialize_handler (handler : ssz_handler) : string =
  (* Simplified implementation *)
  serialize_ssz_id handler.id ^
  serialize_ssz_id handler.domain ^
  serialize_str handler.effect_type ^
  String.concat "" (List.map serialize_ssz_id handler.constraints) ^
  serialize_ssz_id handler.dynamic_expr ^
  String.make 1 (Char.chr handler.priority) ^
  (* Int64 serialization would go here *) 
  String.make 8 '\000' ^
  (if handler.ephemeral then "\001" else "\000")

(** Deserialize a handler - simplified implementation *)
let deserialize_handler (_bytes : string) : ssz_handler =
  (* Placeholder implementation *)
  {
    id = { bytes = String.make id_byte_len '\000' };
    domain = { bytes = String.make id_byte_len '\000' };
    effect_type = "";
    constraints = [];
    dynamic_expr = { bytes = String.make id_byte_len '\000' };
    priority = 0;
    cost = 0L;
    ephemeral = false;
  } 