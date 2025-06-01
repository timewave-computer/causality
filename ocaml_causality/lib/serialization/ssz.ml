(*
 * SSZ Serialization Module
 *
 * This module provides Simple Serialize (SSZ) encoding and decoding
 * functionality for Causality types. SSZ is a deterministic serialization
 * method optimized for minimal encoding/decoding overhead and fixed-width 
 * representation of data.
 *)

open Ocaml_causality_core
open Ocaml_causality_lang
open Types (* Explicitly open Types from Ocaml_causality_core for resource type AND value_expr *)
open Batteries (* For BatMap and other utilities *)

(* ------------ SERIALIZATION TYPES ------------ *)

(** Serialized data type *)
type serialized = bytes

(** Serialization error type *)
type error =
  | InvalidFormat of string        (* Invalid data format *)
  | MissingField of string         (* Required field missing *)
  | UnsupportedType of string      (* Type not supported for serialization *)
  | LengthMismatch of string       (* Length mismatch during deserialization *)
  | Other of string                (* Other serialization errors *)

(** Result type for serialization operations *)
type 'a result = ('a, error) Result.t

(* ------------ PRIMITIVE ENCODING FUNCTIONS ------------ *)

(** Encode a 64-bit integer as bytes (little-endian) *)
let encode_int64 (i: int64) : bytes =
  let buf = Bytes.create 8 in
  Bytes.set_int64_le buf 0 i;
  buf

(** Encode a 32-bit integer as bytes (little-endian) *)
let encode_int32 (i: int32) : bytes =
  let buf = Bytes.create 4 in
  Bytes.set_int32_le buf 0 i;
  buf

(** Encode a boolean as a byte *)
let encode_bool (b: bool) : bytes =
  Bytes.make 1 (if b then '\001' else '\000')

(** Encode a string with length prefix *)
let encode_string (s: string) : bytes =
  let len = String.length s in
  let len_bytes = encode_int32 (Int32.of_int len) in
  let buf = Bytes.create (4 + len) in
  Bytes.blit len_bytes 0 buf 0 4;
  Bytes.blit_string s 0 buf 4 len;
  buf

(** Encode bytes with length prefix *)
let encode_bytes (b: bytes) : bytes =
  let len = Bytes.length b in
  let len_bytes = encode_int32 (Int32.of_int len) in
  let buf = Bytes.create (4 + len) in
  Bytes.blit len_bytes 0 buf 0 4;
  Bytes.blit b 0 buf 4 len;
  buf

(* ------------ PRIMITIVE DECODING FUNCTIONS ------------ *)

(** Decode a 64-bit integer from bytes (little-endian) *)
let decode_int64 (buf: bytes) (offset: int) : int64 =
  Bytes.get_int64_le buf offset

(** Decode a 32-bit integer from bytes (little-endian) *)
let decode_int32 (buf: bytes) (offset: int) : int32 =
  Bytes.get_int32_le buf offset

(** Decode a boolean from a byte *)
let decode_bool (buf: bytes) (offset: int) : bool * int = (* Return new offset *)
  if offset >= Bytes.length buf then raise (Invalid_argument "decode_bool: offset out of bounds");
  (Bytes.get buf offset <> '\000', offset + 1)

(** Decode a string with length prefix *)
let decode_string (buf: bytes) (offset: int) : string * int =
  if offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_string: offset for length out of bounds");
  let len = Int32.to_int (decode_int32 buf offset) in
  if offset + 4 + len > Bytes.length buf then raise (Invalid_argument "decode_string: offset for data out of bounds");
  let str = Bytes.sub_string buf (offset + 4) len in
  (str, offset + 4 + len)

(* ------------ CAUSALITY TYPE ENCODING ------------ *)

(** Encode an AST atom value *)
let encode_atom (atom: Ast.atom) : bytes =
  match atom with
  | Ast.Integer i -> 
      let tag = Bytes.make 1 '\000' in
      Bytes.cat tag (encode_int64 i)
  | Ast.Float f -> 
      let tag = Bytes.make 1 '\001' in
      let buf = Bytes.create 8 in
      Bytes.set_int64_le buf 0 (Int64.bits_of_float f);
      Bytes.cat tag buf
  | Ast.String s -> 
      let tag = Bytes.make 1 '\002' in
      Bytes.cat tag (encode_string s)
  | Ast.Boolean b -> 
      let tag = Bytes.make 1 '\003' in
      Bytes.cat tag (encode_bool b)
  | Ast.Symbol s -> 
      let tag = Bytes.make 1 '\004' in
      Bytes.cat tag (encode_string s)

(** Encode an AST value expression *)
let rec encode_value_expr (value: Ast.value_expr) : bytes =
  match value with
  | Ast.VAtom atom ->
      let tag = Bytes.make 1 '\000' in
      Bytes.cat tag (encode_atom atom)
  | Ast.VList values ->
      let tag = Bytes.make 1 '\001' in
      let count = List.length values in
      let count_bytes = encode_int32 (Int32.of_int count) in
      let encoded_values = List.map encode_value_expr values in
      let total_len = List.fold_left (fun acc v -> acc + Bytes.length v) 0 encoded_values in
      let buf = Bytes.create (1 + 4 + total_len) in
      Bytes.set buf 0 (Bytes.get tag 0);
      Bytes.blit count_bytes 0 buf 1 4;
      let offset = ref 5 in
      List.iter (fun v ->
        let len = Bytes.length v in
        Bytes.blit v 0 buf !offset len;
        offset := !offset + len
      ) encoded_values;
      buf
  | Ast.VMap kvs ->
      let tag = Bytes.make 1 '\002' in
      let count = List.length kvs in
      let count_bytes = encode_int32 (Int32.of_int count) in
      let encoded_kvs = List.map (fun (k, v) -> 
        Bytes.cat (encode_string k) (encode_value_expr v)
      ) kvs in
      let total_len = List.fold_left (fun acc kv -> acc + Bytes.length kv) 0 encoded_kvs in
      let buf = Bytes.create (1 + 4 + total_len) in
      Bytes.set buf 0 (Bytes.get tag 0);
      Bytes.blit count_bytes 0 buf 1 4;
      let offset = ref 5 in
      List.iter (fun kv ->
        let len = Bytes.length kv in
        Bytes.blit kv 0 buf !offset len;
        offset := !offset + len
      ) encoded_kvs;
      buf
  | Ast.VClosure _ ->
      (* Closures are not serializable *)
      let tag = Bytes.make 1 '\003' in
      tag
  | Ast.VUnit ->
      let tag = Bytes.make 1 '\004' in
      tag
  | Ast.VNative _ ->
      (* Native functions are not serializable *)
      let tag = Bytes.make 1 '\005' in
      tag

(** Temporary stub for encoding core value_expr until proper implementation *)
let encode_core_value_expr (value: Types.value_expr) : bytes =
  (* Simplified encoding - would need proper implementation for different value types *)
  match value with
  | Types.VNil -> Bytes.of_string "nil"
  | Types.VBool true -> Bytes.of_string "true"  
  | Types.VBool false -> Bytes.of_string "false"
  | Types.VString s -> encode_string s
  | Types.VInt i -> encode_int64 i
  | Types.VList _l -> Bytes.of_string "list" (* stub *)
  | Types.VMap _m -> Bytes.of_string "map" (* stub *)
  | Types.VStruct _s -> Bytes.of_string "struct" (* stub *)
  | Types.VRef _r -> Bytes.of_string "ref" (* stub *)
  | Types.VLambda _lambda -> Bytes.of_string "lambda" (* stub *)

(** Encode an AST expression *)
let rec encode_expr (expr: Ast.expr) : bytes =
  match expr with
  | Ast.EConst core_value -> (* core_value is now Ocaml_causality_core.Types.value_expr *)
      let tag = Bytes.make 1 '\000' in
      Bytes.cat tag (encode_core_value_expr core_value)
  | Ast.EVar name ->
      let tag = Bytes.make 1 '\001' in
      Bytes.cat tag (encode_string name)
  | Ast.ELambda (params, body) ->
      let tag = Bytes.make 1 '\002' in
      let param_count = List.length params in
      let count_bytes = encode_int32 (Int32.of_int param_count) in
      let param_bytes = List.fold_left (fun acc param ->
        Bytes.cat acc (encode_string param)
      ) (Bytes.empty) params in
      let body_bytes = encode_expr body in
      Bytes.cat (Bytes.cat (Bytes.cat tag count_bytes) param_bytes) body_bytes
  | Ast.EApply (func, args) ->
      let tag = Bytes.make 1 '\003' in
      let arg_count = List.length args in
      let count_bytes = encode_int32 (Int32.of_int arg_count) in
      let func_bytes = encode_expr func in
      let args_bytes = List.fold_left (fun acc arg ->
        Bytes.cat acc (encode_expr arg)
      ) (Bytes.empty) args in
      Bytes.cat (Bytes.cat (Bytes.cat tag count_bytes) func_bytes) args_bytes
  | Ast.ECombinator comb ->
      let tag = Bytes.make 1 '\004' in
      let comb_id = match comb with
        | Ast.S -> 0 | Ast.K -> 1 | Ast.I -> 2 | Ast.C -> 3
        | Ast.If -> 4 | Ast.Let -> 5 | Ast.LetStar -> 6
        | Ast.And -> 7 | Ast.Or -> 8 | Ast.Not -> 9
        | Ast.Eq -> 10 | Ast.Gt -> 11 | Ast.Lt -> 12 | Ast.Gte -> 13 | Ast.Lte -> 14
        | Ast.Add -> 15 | Ast.Sub -> 16 | Ast.Mul -> 17 | Ast.Div -> 18
        | Ast.GetContextValue -> 19 | Ast.GetField -> 20 | Ast.Completed -> 21
        | Ast.List -> 22 | Ast.Nth -> 23 | Ast.Length -> 24
        | Ast.Cons -> 25 | Ast.Car -> 26 | Ast.Cdr -> 27
        | Ast.MakeMap -> 28 | Ast.MapGet -> 29 | Ast.MapHasKey -> 30
        | Ast.Define -> 31 | Ast.Defun -> 32 | Ast.Quote -> 33
      in
      Bytes.cat tag (encode_int32 (Int32.of_int comb_id))
  | Ast.EAtom atom ->
      let tag = Bytes.make 1 '\005' in
      Bytes.cat tag (encode_atom atom)
  | Ast.EDynamic (id, expr) ->
      let tag = Bytes.make 1 '\006' in
      let id_bytes = encode_int32 (Int32.of_int id) in
      let expr_bytes = encode_expr expr in
      Bytes.cat (Bytes.cat tag id_bytes) expr_bytes

(* ------------ CAUSALITY TYPE DECODING ------------ *)

(** Decode bytes with length prefix *)
let decode_bytes (buf: bytes) (offset: int) : bytes * int =
  if offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_bytes: offset for length out of bounds");
  let len = Int32.to_int (decode_int32 buf offset) in
  if offset + 4 + len > Bytes.length buf then raise (Invalid_argument "decode_bytes: offset for data out of bounds");
  let b = Bytes.sub buf (offset + 4) len in
  (b, offset + 4 + len)

(** Decode an AST atom value *)
let decode_ast_atom (buf: bytes) (initial_offset: int) : Ast.atom * int = (* Returns atom and new offset *)
  if initial_offset >= Bytes.length buf then raise (Invalid_argument "decode_atom: offset out of bounds for tag");
  let tag = Bytes.get buf initial_offset in
  let current_offset = initial_offset + 1 in
  match tag with
  | '\000' (* Integer *) ->
    if current_offset + 8 > Bytes.length buf then raise (Invalid_argument "decode_atom: Integer data out of bounds");
    let i = decode_int64 buf current_offset in
    (Ast.Integer i, current_offset + 8)
  | '\001' (* Float *) ->
    if current_offset + 8 > Bytes.length buf then raise (Invalid_argument "decode_atom: Float data out of bounds");
    let i64_bits = decode_int64 buf current_offset in
    (Ast.Float (Int64.float_of_bits i64_bits), current_offset + 8)
  | '\002' (* String *) ->
    let s, next_offset = decode_string buf current_offset in
    (Ast.String s, next_offset)
  | '\003' (* Boolean *) ->
    if current_offset >= Bytes.length buf then raise (Invalid_argument "decode_atom: Boolean data out of bounds");
    let b, next_offset = decode_bool buf current_offset in
    (Ast.Boolean b, next_offset)
  | '\004' (* Symbol *) ->
    let s, next_offset = decode_string buf current_offset in
    (Ast.Symbol s, next_offset)
  | _ -> raise (Invalid_argument (Printf.sprintf "decode_atom: unknown tag %C" tag))

(** Decode an AST value expression *)
let rec decode_ast_value_expr (buf: bytes) (initial_offset: int) : Ast.value_expr * int =
  if initial_offset >= Bytes.length buf then raise (Invalid_argument "decode_ast_value_expr: offset out of bounds for tag");
  let tag = Bytes.get buf initial_offset in
  let current_offset = initial_offset + 1 in
  match tag with
  | '\000' (* VAtom from Ast.value_expr *) ->
    let atom, next_offset = decode_ast_atom buf current_offset in
    (Ast.VAtom atom, next_offset)
  | '\001' (* VList from Ast.value_expr *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_ast_value_expr: VList count out of bounds");
    let count = Int32.to_int (decode_int32 buf current_offset) in
    let mut_offset = ref (current_offset + 4) in
    let items = ref [] in
    for _ = 1 to count do
      let item, next_item_offset = decode_ast_value_expr buf !mut_offset in
      items := item :: !items;
      mut_offset := next_item_offset
    done;
    (Ast.VList (List.rev !items), !mut_offset)
  | '\002' (* VMap from Ast.value_expr (list of pairs) *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_ast_value_expr: VMap count out of bounds");
    let count = Int32.to_int (decode_int32 buf current_offset) in
    let mut_offset = ref (current_offset + 4) in
    let kvs = ref [] in
    for _ = 1 to count do
      let key, next_key_offset = decode_string buf !mut_offset in
      let value, next_value_offset = decode_ast_value_expr buf next_key_offset in
      kvs := (key, value) :: !kvs;
      mut_offset := next_value_offset
    done;
    (Ast.VMap (List.rev !kvs), !mut_offset)
  | '\003' (* VClosure from Ast.value_expr *) ->
    raise (Invalid_argument "decode_ast_value_expr: VClosure tag encountered; Ast closures are not directly serializable this way")
  | '\004' (* VUnit from Ast.value_expr *) ->
    (Ast.VUnit, current_offset)
  | '\005' (* VNative from Ast.value_expr *) ->
    raise (Invalid_argument "decode_ast_value_expr: VNative tag encountered; native functions are not serializable")
  | _ -> raise (Invalid_argument (Printf.sprintf "decode_ast_value_expr: unknown tag %C" tag))

(*
(** Decode an entity ID *)
let decode_entity_id (buf: bytes) (offset: int) : entity_id * int =
  decode_bytes buf offset

(** Decode a timestamp *)
let decode_timestamp (buf: bytes) (offset: int) : timestamp * int =
  (decode_int64 buf offset, offset + 8)

(** Decode a domain ID *)
let decode_domain_id (buf: bytes) (offset: int) : domain_id * int =
  decode_bytes buf offset
*)

(* ------------ PUBLIC API ------------ *)

(** Encode an AST value expression to bytes *)
let encode_value (value: Types.value_expr) : serialized =
  encode_core_value_expr value

(** Encode an AST expression to bytes *)
let encode (expr: Ast.expr) : serialized =
  encode_expr expr

(** Encode the content of a resource (excluding its ID) to bytes *)
let encode_resource_content (res: resource) : serialized =
  let name_bytes = encode_string res.resource_name in
  let domain_id_bytes = encode_bytes res.resource_domain_id in
  let type_bytes = encode_string res.resource_type in
  let quantity_bytes = encode_int64 res.resource_quantity in
  let timestamp_bytes = encode_int64 res.resource_timestamp in
  Bytes.concat Bytes.empty [
    name_bytes;
    domain_id_bytes;
    type_bytes;
    quantity_bytes;
    timestamp_bytes;
  ]

(** Decode a Core.Types.value_expr - stub implementation *)
let decode_core_value_expr (buf: bytes) (offset: int) : Types.value_expr * int =
  (* Temporary stub - would need proper implementation *)
  (Types.VNil, offset + 1)

(** Attempt to decode a Core.Types.value_expr from bytes *)
let decode_value (data: serialized) : value_expr result = (* value_expr is Types.value_expr *)
  try
    let ve, consumed = decode_core_value_expr data 0 in
    if consumed <> Bytes.length data then
      Error (LengthMismatch (Printf.sprintf "decode_value: Expected to consume %d bytes, but consumed %d" (Bytes.length data) consumed))
    else
      Ok ve
  with
  | Invalid_argument msg -> Error (InvalidFormat msg)
  (* Add other specific exceptions if decode_core_value_expr raises them *)
  | ex -> Error (Other (Printexc.to_string ex))

(** Decode an AST.expr *)
let rec decode_expr (buf: bytes) (initial_offset: int) : Ast.expr * int =
  if initial_offset >= Bytes.length buf then raise (Invalid_argument "decode_expr: offset out of bounds for tag");
  let tag = Bytes.get buf initial_offset in
  let current_offset = initial_offset + 1 in
  match tag with
  | '\000' (* EConst *) ->
    let core_value, next_offset = decode_core_value_expr buf current_offset in (* EConst now holds Types.value_expr *)
    (Ast.EConst core_value, next_offset)
  | '\001' (* EVar *) ->
    let name, next_offset = decode_string buf current_offset in
    (Ast.EVar name, next_offset)
  | '\002' (* ELambda *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_expr: ELambda param_count out of bounds");
    let param_count = Int32.to_int (decode_int32 buf current_offset) in
    let mut_offset = ref (current_offset + 4) in
    let params_list = ref [] in
    for _ = 1 to param_count do
      let p_name, next_p_offset = decode_string buf !mut_offset in
      params_list := p_name :: !params_list;
      mut_offset := next_p_offset
    done;
    let body_expr, next_body_offset = decode_expr buf !mut_offset in
    (Ast.ELambda (List.rev !params_list, body_expr), next_body_offset)
  | '\003' (* EApply *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_expr: EApply arg_count out of bounds");
    let arg_count = Int32.to_int (decode_int32 buf current_offset) in
    let func_offset = current_offset + 4 in
    let func_expr, args_start_offset = decode_expr buf func_offset in
    let mut_offset = ref args_start_offset in
    let args_list = ref [] in
    for _ = 1 to arg_count do
      let arg_expr, next_arg_offset = decode_expr buf !mut_offset in
      args_list := arg_expr :: !args_list;
      mut_offset := next_arg_offset
    done;
    (Ast.EApply (func_expr, List.rev !args_list), !mut_offset)
  | '\004' (* ECombinator *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_expr: ECombinator id out of bounds");
    let comb_id_int = Int32.to_int (decode_int32 buf current_offset) in
    let comb = match comb_id_int with (* This matches Ast.atomic_combinator *)
      | 0 -> Ast.S | 1 -> Ast.K | 2 -> Ast.I | 3 -> Ast.C
      | 4 -> Ast.If | 5 -> Ast.Let | 6 -> Ast.LetStar
      | 7 -> Ast.And | 8 -> Ast.Or | 9 -> Ast.Not
      | 10 -> Ast.Eq | 11 -> Ast.Gt | 12 -> Ast.Lt | 13 -> Ast.Gte | 14 -> Ast.Lte
      | 15 -> Ast.Add | 16 -> Ast.Sub | 17 -> Ast.Mul | 18 -> Ast.Div
      | 19 -> Ast.GetContextValue | 20 -> Ast.GetField | 21 -> Ast.Completed
      | 22 -> Ast.List | 23 -> Ast.Nth | 24 -> Ast.Length
      | 25 -> Ast.Cons | 26 -> Ast.Car | 27 -> Ast.Cdr
      | 28 -> Ast.MakeMap | 29 -> Ast.MapGet | 30 -> Ast.MapHasKey
      | 31 -> Ast.Define | 32 -> Ast.Defun | 33 -> Ast.Quote
      | _ -> raise (Invalid_argument (Printf.sprintf "decode_expr: ECombinator unknown id %d" comb_id_int))
    in
    (Ast.ECombinator comb, current_offset + 4)
  | '\005' (* EAtom *) ->
    let atom, next_offset = decode_ast_atom buf current_offset in (* Assuming EAtom holds Ast.atom *)
    (Ast.EAtom atom, next_offset)
  | '\006' (* EDynamic *) ->
    if current_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_expr: EDynamic id out of bounds");
    let id_int = Int32.to_int (decode_int32 buf current_offset) in
    let expr_body, next_offset = decode_expr buf (current_offset + 4) in
    (Ast.EDynamic (id_int, expr_body), next_offset)
  | _ -> raise (Invalid_argument (Printf.sprintf "decode_expr: unknown tag %C at offset %d" tag initial_offset))

(** Attempt to decode an Ast.expr from bytes *)
let decode (data: serialized) : Ast.expr result =
  try
    let expr, consumed = decode_expr data 0 in
    if consumed <> Bytes.length data then
      Error (LengthMismatch (Printf.sprintf "decode (Ast.expr): Expected to consume %d bytes, but consumed %d" (Bytes.length data) consumed))
    else
      Ok expr
  with
  | Invalid_argument msg -> Error (InvalidFormat msg)
  | ex -> Error (Other (Printexc.to_string ex))

(** Create a string representation of serialized data (for debugging) *)
let to_hex (data: serialized) : string =
  let len = Bytes.length data in
  let hex = Bytes.create (len * 2) in
  
  for i = 0 to len - 1 do
    let byte = Char.code (Bytes.get data i) in
    let hi = byte lsr 4 in
    let lo = byte land 0xF in
    let to_hex_char n = Char.chr (if n < 10 then n + 48 else n + 87) in
    Bytes.set hex (i * 2) (to_hex_char hi);
    Bytes.set hex (i * 2 + 1) (to_hex_char lo);
  done;
  
  Bytes.to_string hex 

(* Helper for encoding BatMap for Types.value_expr VMap and VStruct *)
let encode_batmap_value_expr (map: (str_t, value_expr) BatMap.t) (encoder_func: value_expr -> bytes) : bytes =
  let sorted_bindings = BatMap.bindings map in (* BatMap.bindings are already sorted by key *)
  let count = List.length sorted_bindings in
  let count_bytes = encode_int32 (Int32.of_int count) in
  let encoded_kvs = List.map (fun (k, v) ->
    Bytes.cat (encode_string k) (encoder_func v)
  ) sorted_bindings in
  let content_bytes = Bytes.concat Bytes.empty encoded_kvs in
  Bytes.cat count_bytes content_bytes

(* Helper for decoding BatMap for Types.value_expr VMap and VStruct *)
let decode_batmap_value_expr (buf: bytes) (initial_offset: int) (decoder_func: bytes -> int -> value_expr * int) : ((str_t, value_expr) BatMap.t * int) =
  if initial_offset + 4 > Bytes.length buf then raise (Invalid_argument "decode_batmap_value_expr: count out of bounds");
  let count = Int32.to_int (decode_int32 buf initial_offset) in
  let mut_offset = ref (initial_offset + 4) in
  let kvs = ref [] in
  for _ = 1 to count do
    let key, next_key_offset = decode_string buf !mut_offset in
    let value, next_value_offset = decoder_func buf next_key_offset in (* Recursive call via decoder_func *)
    kvs := (key, value) :: !kvs;
    mut_offset := next_value_offset
  done;
  (* Build the map properly using fold_left *)
  let empty_map = BatMap.empty in
  let final_map = List.fold_left (fun acc (k, v) -> BatMap.add k v acc) empty_map (List.rev !kvs) in
  (final_map, !mut_offset)

(* --- Functions for Ast.value_expr and Ast.atom --- *)

(** Encode an AST atom value (for Ast.atom) *)
let encode_ast_atom (atom: Ast.atom) : bytes =
  match atom with
  | Ast.Integer i -> 
      let tag_byte = Bytes.make 1 '\000' in (* Tag for Ast.Integer *)
      Bytes.cat tag_byte (encode_int64 i)
  | Ast.Float f -> 
      let tag_byte = Bytes.make 1 '\001' in (* Tag for Ast.Float *)
      let buf = Bytes.create 8 in
      Bytes.set_int64_le buf 0 (Int64.bits_of_float f);
      Bytes.cat tag_byte buf
  | Ast.String s -> 
      let tag_byte = Bytes.make 1 '\002' in (* Tag for Ast.String *)
      Bytes.cat tag_byte (encode_string s)
  | Ast.Boolean b -> 
      let tag_byte = Bytes.make 1 '\003' in (* Tag for Ast.Boolean *)
      Bytes.cat tag_byte (encode_bool b)
  | Ast.Symbol s -> 
      let tag_byte = Bytes.make 1 '\004' in (* Tag for Ast.Symbol *)
      Bytes.cat tag_byte (encode_string s)

(** Encode an AST value expression (for Ast.value_expr) *)
let rec encode_ast_value_expr (value: Ast.value_expr) : bytes =
  match value with
  | Ast.VAtom atom ->
      let tag_byte = Bytes.make 1 '\000' in (* Tag for Ast.VAtom *)
      Bytes.cat tag_byte (encode_ast_atom atom)
  | Ast.VList values ->
      let tag_byte = Bytes.make 1 '\001' in (* Tag for Ast.VList *)
      let count = List.length values in
      let count_bytes = encode_int32 (Int32.of_int count) in
      let encoded_values = List.map encode_ast_value_expr values in
      let total_len = List.fold_left (fun acc v -> acc + Bytes.length v) 0 encoded_values in
      let buf = Bytes.create (1 + 4 + total_len) in
      Bytes.set buf 0 (Bytes.get tag_byte 0);
      Bytes.blit count_bytes 0 buf 1 4;
      let offset = ref 5 in
      List.iter (fun v ->
        let len = Bytes.length v in
        Bytes.blit v 0 buf !offset len;
        offset := !offset + len
      ) encoded_values;
      buf
  | Ast.VMap kvs -> (* kvs is (string * Ast.value_expr) list *)
      let tag_byte = Bytes.make 1 '\002' in (* Tag for Ast.VMap *)
      let count = List.length kvs in
      let count_bytes = encode_int32 (Int32.of_int count) in
      let encoded_kvs = List.map (fun (k, v) -> 
        Bytes.cat (encode_string k) (encode_ast_value_expr v)
      ) kvs in (* No sorting needed for list-based VMap *)
      let total_len = List.fold_left (fun acc kv -> acc + Bytes.length kv) 0 encoded_kvs in
      let buf = Bytes.create (1 + 4 + total_len) in
      Bytes.set buf 0 (Bytes.get tag_byte 0);
      Bytes.blit count_bytes 0 buf 1 4;
      let offset = ref 5 in
      List.iter (fun kv ->
        let len = Bytes.length kv in
        Bytes.blit kv 0 buf !offset len;
        offset := !offset + len
      ) encoded_kvs;
      buf
  | Ast.VClosure _ ->
      Bytes.make 1 '\003' (* Tag for Ast.VClosure - not serializable *)
  | Ast.VUnit ->
      Bytes.make 1 '\004' (* Tag for Ast.VUnit *)
  | Ast.VNative _ ->
      Bytes.make 1 '\005' (* Tag for Ast.VNative - not serializable *)


(** Encode an AST expression *)
let rec encode_expr (expr: Ast.expr) : bytes =
  match expr with
  | Ast.EConst core_value -> (* core_value is now Ocaml_causality_core.Types.value_expr *)
      let tag = Bytes.make 1 '\000' in
      Bytes.cat tag (encode_core_value_expr core_value)
  | Ast.EVar name ->
      let tag = Bytes.make 1 '\001' in
      Bytes.cat tag (encode_string name)
  | Ast.ELambda (params, body) ->
      let tag = Bytes.make 1 '\002' in
      let param_count = List.length params in
      let count_bytes = encode_int32 (Int32.of_int param_count) in
      let param_bytes = List.fold_left (fun acc param ->
        Bytes.cat acc (encode_string param)
      ) (Bytes.empty) params in
      let body_bytes = encode_expr body in
      Bytes.cat (Bytes.cat (Bytes.cat tag count_bytes) param_bytes) body_bytes
  | Ast.EApply (func, args) ->
      let tag = Bytes.make 1 '\003' in
      let arg_count = List.length args in
      let count_bytes = encode_int32 (Int32.of_int arg_count) in
      let func_bytes = encode_expr func in
      let args_bytes = List.fold_left (fun acc arg ->
        Bytes.cat acc (encode_expr arg)
      ) (Bytes.empty) args in
      Bytes.cat (Bytes.cat (Bytes.cat tag count_bytes) func_bytes) args_bytes
  | Ast.ECombinator comb ->
      let tag = Bytes.make 1 '\004' in
      let comb_id = match comb with
        | Ast.S -> 0 | Ast.K -> 1 | Ast.I -> 2 | Ast.C -> 3
        | Ast.If -> 4 | Ast.Let -> 5 | Ast.LetStar -> 6
        | Ast.And -> 7 | Ast.Or -> 8 | Ast.Not -> 9
        | Ast.Eq -> 10 | Ast.Gt -> 11 | Ast.Lt -> 12 | Ast.Gte -> 13 | Ast.Lte -> 14
        | Ast.Add -> 15 | Ast.Sub -> 16 | Ast.Mul -> 17 | Ast.Div -> 18
        | Ast.GetContextValue -> 19 | Ast.GetField -> 20 | Ast.Completed -> 21
        | Ast.List -> 22 | Ast.Nth -> 23 | Ast.Length -> 24
        | Ast.Cons -> 25 | Ast.Car -> 26 | Ast.Cdr -> 27
        | Ast.MakeMap -> 28 | Ast.MapGet -> 29 | Ast.MapHasKey -> 30
        | Ast.Define -> 31 | Ast.Defun -> 32 | Ast.Quote -> 33
      in
      Bytes.cat tag (encode_int32 (Int32.of_int comb_id))
  | Ast.EAtom atom ->
      let tag = Bytes.make 1 '\005' in
      Bytes.cat tag (encode_atom atom)
  | Ast.EDynamic (id, expr) ->
      let tag = Bytes.make 1 '\006' in
      let id_bytes = encode_int32 (Int32.of_int id) in
      let expr_bytes = encode_expr expr in
      Bytes.cat (Bytes.cat tag id_bytes) expr_bytes 