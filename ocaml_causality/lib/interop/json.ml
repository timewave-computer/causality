(*
 * JSON Conversion Module
 *
 * This module provides functionality for converting Causality types to and from
 * JSON representations. It enables interoperability with web APIs, JavaScript
 * environments, and other systems that use JSON as a data exchange format.
 *)

open Ocaml_causality_core
open Ocaml_causality_lang
open Yojson.Safe

(* ------------ JSON CONVERSION TYPES ------------ *)

(** JSON conversion error *)
type json_error =
  | InvalidJson of string          (* Invalid JSON format *)
  | MissingField of string         (* Required field is missing *)
  | TypeMismatch of string         (* Type doesn't match expected type *)
  | UnsupportedType of string      (* Type not supported for JSON conversion *)
  | InvalidValue of string         (* Value doesn't meet constraints *)

(** Result type for JSON conversion operations *)
type 'a result = ('a, json_error) Result.t

(* ------------ CORE TYPE CONVERSION ------------ *)

(** Convert an entity ID to JSON *)
let entity_id_to_json (id: entity_id) : json =
  `String id

(** Convert JSON to an entity ID *)
let entity_id_of_json (json: json) : entity_id result =
  match json with
  | `String id -> Ok id
  | _ -> Error (TypeMismatch "Expected string for entity_id")

(** Convert a timestamp to JSON *)
let timestamp_to_json (ts: timestamp) : json =
  `Int (Int64.to_int ts)

(** Convert JSON to a timestamp *)
let timestamp_of_json (json: json) : timestamp result =
  match json with
  | `Int i -> Ok (Int64.of_int i)
  | `Intlit s -> 
      (try Ok (Int64.of_string s)
       with _ -> Error (TypeMismatch "Invalid timestamp format"))
  | _ -> Error (TypeMismatch "Expected integer for timestamp")

(** Convert a domain ID to JSON *)
let domain_id_to_json (domain: domain_id) : json =
  `Int (Int32.to_int domain.domain_id)

(** Convert JSON to a domain ID *)
let domain_id_of_json (json: json) : domain_id result =
  match json with
  | `Int i -> Ok { domain_id = Int32.of_int i }
  | `Intlit s -> 
      (try Ok { domain_id = Int32.of_string s }
       with _ -> Error (TypeMismatch "Invalid domain_id format"))
  | _ -> Error (TypeMismatch "Expected integer for domain_id")

(* ------------ AST TYPE CONVERSION ------------ *)

(** Convert an AST atom to JSON *)
let rec atom_to_json (atom: Ast.atom) : json =
  match atom with
  | Ast.Integer i -> `Intlit (Int64.to_string i)
  | Ast.Float f -> `Float f
  | Ast.String s -> `String s
  | Ast.Bool b -> `Bool b
  | Ast.Symbol s -> `Assoc [("symbol", `String s)]

(** Convert JSON to an AST atom *)
let atom_of_json (json: json) : Ast.atom result =
  match json with
  | `Intlit s -> 
      (try Ok (Ast.Integer (Int64.of_string s))
       with _ -> Error (TypeMismatch "Invalid integer format"))
  | `Int i -> Ok (Ast.Integer (Int64.of_int i))
  | `Float f -> Ok (Ast.Float f)
  | `String s -> Ok (Ast.String s)
  | `Bool b -> Ok (Ast.Bool b)
  | `Assoc [("symbol", `String s)] -> Ok (Ast.Symbol s)
  | _ -> Error (TypeMismatch "Invalid atom format")

(** Convert an AST value expression to JSON *)
let rec value_expr_to_json (value: Ast.value_expr) : json =
  match value with
  | Ast.VAtom atom -> 
      `Assoc [("type", `String "atom"); ("value", atom_to_json atom)]
  | Ast.VList values ->
      `Assoc [
        ("type", `String "list");
        ("elements", `List (List.map value_expr_to_json values))
      ]
  | Ast.VMap kvs ->
      let json_kvs = List.map (fun (k, v) ->
        `Assoc [
          ("key", value_expr_to_json k);
          ("value", value_expr_to_json v)
        ]
      ) kvs in
      `Assoc [
        ("type", `String "map");
        ("entries", `List json_kvs)
      ]
  | Ast.VClosure _ ->
      `Assoc [("type", `String "closure"); ("value", `Null)]
  | Ast.VUnit ->
      `Assoc [("type", `String "unit"); ("value", `Null)]
  | Ast.VNative _ ->
      `Assoc [("type", `String "native"); ("value", `Null)]

(** Convert JSON to an AST value expression *)
let rec value_expr_of_json (json: json) : Ast.value_expr result =
  match json with
  | `Assoc fields ->
      (match List.assoc_opt "type" fields with
       | Some (`String "atom") ->
           (match List.assoc_opt "value" fields with
            | Some value -> 
                Result.map (fun a -> Ast.VAtom a) (atom_of_json value)
            | None -> Error (MissingField "value"))
       | Some (`String "list") ->
           (match List.assoc_opt "elements" fields with
            | Some (`List elements) ->
                let rec process_elements = function
                  | [] -> Ok []
                  | e :: rest ->
                      match value_expr_of_json e with
                      | Ok value ->
                          (match process_elements rest with
                           | Ok values -> Ok (value :: values)
                           | Error e -> Error e)
                      | Error e -> Error e
                in
                Result.map (fun elements -> Ast.VList elements) (process_elements elements)
            | Some _ -> Error (TypeMismatch "elements should be a list")
            | None -> Error (MissingField "elements"))
       | Some (`String "map") ->
           (match List.assoc_opt "entries" fields with
            | Some (`List entries) ->
                let process_entry entry =
                  match entry with
                  | `Assoc kv_fields ->
                      (match List.assoc_opt "key" kv_fields, List.assoc_opt "value" kv_fields with
                       | Some k, Some v ->
                           (match value_expr_of_json k, value_expr_of_json v with
                            | Ok key, Ok value -> Ok (key, value)
                            | Error e, _ -> Error e
                            | _, Error e -> Error e)
                       | None, _ -> Error (MissingField "key")
                       | _, None -> Error (MissingField "value"))
                  | _ -> Error (TypeMismatch "map entry should be an object")
                in
                let rec process_entries = function
                  | [] -> Ok []
                  | e :: rest ->
                      match process_entry e with
                      | Ok kv ->
                          (match process_entries rest with
                           | Ok kvs -> Ok (kv :: kvs)
                           | Error e -> Error e)
                      | Error e -> Error e
                in
                Result.map (fun entries -> Ast.VMap entries) (process_entries entries)
            | Some _ -> Error (TypeMismatch "entries should be a list")
            | None -> Error (MissingField "entries"))
       | Some (`String "unit") -> Ok Ast.VUnit
       | Some (`String "closure") -> Error (UnsupportedType "Cannot convert closure from JSON")
       | Some (`String "native") -> Error (UnsupportedType "Cannot convert native function from JSON")
       | Some _ -> Error (TypeMismatch "Invalid type field")
       | None -> Error (MissingField "type"))
  | _ -> Error (TypeMismatch "Expected object for value expression")

(** Convert an AST expression to JSON *)
let rec expr_to_json (expr: Ast.expr) : json =
  match expr with
  | Ast.EConst value ->
      `Assoc [
        ("type", `String "const");
        ("value", value_expr_to_json value)
      ]
  | Ast.EVar name ->
      `Assoc [
        ("type", `String "var");
        ("name", `String name)
      ]
  | Ast.ELambda (params, body) ->
      `Assoc [
        ("type", `String "lambda");
        ("params", `List (List.map (fun p -> `String p) params));
        ("body", expr_to_json body)
      ]
  | Ast.EApply (func, args) ->
      `Assoc [
        ("type", `String "apply");
        ("func", expr_to_json func);
        ("args", `List (List.map expr_to_json args))
      ]
  | Ast.ECombinator comb ->
      `Assoc [
        ("type", `String "combinator");
        ("name", `String (Ast.combinator_to_string comb))
      ]
  | Ast.EAtom atom ->
      `Assoc [
        ("type", `String "atom");
        ("value", atom_to_json atom)
      ]
  | Ast.EDynamic (id, expr) ->
      `Assoc [
        ("type", `String "dynamic");
        ("id", `Int id);
        ("expr", expr_to_json expr)
      ]

(** Convert JSON to an AST expression *)
let rec expr_of_json (json: json) : Ast.expr result =
  match json with
  | `Assoc fields ->
      (match List.assoc_opt "type" fields with
       | Some (`String "const") ->
           (match List.assoc_opt "value" fields with
            | Some value -> 
                Result.map (fun v -> Ast.EConst v) (value_expr_of_json value)
            | None -> Error (MissingField "value"))
       | Some (`String "var") ->
           (match List.assoc_opt "name" fields with
            | Some (`String name) -> Ok (Ast.EVar name)
            | Some _ -> Error (TypeMismatch "name should be a string")
            | None -> Error (MissingField "name"))
       | Some (`String "lambda") ->
           let params = match List.assoc_opt "params" fields with
             | Some (`List ps) ->
                 let rec extract_params = function
                   | [] -> Ok []
                   | (`String p) :: rest ->
                       (match extract_params rest with
                        | Ok params -> Ok (p :: params)
                        | Error e -> Error e)
                   | _ :: _ -> Error (TypeMismatch "params should be strings")
                 in
                 extract_params ps
             | Some _ -> Error (TypeMismatch "params should be a list")
             | None -> Error (MissingField "params")
           in
           let body = match List.assoc_opt "body" fields with
             | Some body -> expr_of_json body
             | None -> Error (MissingField "body")
           in
           (match params, body with
            | Ok ps, Ok b -> Ok (Ast.ELambda (ps, b))
            | Error e, _ -> Error e
            | _, Error e -> Error e)
       | Some (`String "apply") ->
           let func = match List.assoc_opt "func" fields with
             | Some f -> expr_of_json f
             | None -> Error (MissingField "func")
           in
           let args = match List.assoc_opt "args" fields with
             | Some (`List as_json) ->
                 let rec process_args = function
                   | [] -> Ok []
                   | a :: rest ->
                       match expr_of_json a with
                       | Ok arg ->
                           (match process_args rest with
                            | Ok args -> Ok (arg :: args)
                            | Error e -> Error e)
                       | Error e -> Error e
                 in
                 process_args as_json
             | Some _ -> Error (TypeMismatch "args should be a list")
             | None -> Error (MissingField "args")
           in
           (match func, args with
            | Ok f, Ok as_ -> Ok (Ast.EApply (f, as_))
            | Error e, _ -> Error e
            | _, Error e -> Error e)
       | Some (`String "combinator") ->
           (match List.assoc_opt "name" fields with
            | Some (`String name) ->
                (match Ast.combinator_of_string name with
                 | Some comb -> Ok (Ast.ECombinator comb)
                 | None -> Error (InvalidValue ("Unknown combinator: " ^ name)))
            | Some _ -> Error (TypeMismatch "name should be a string")
            | None -> Error (MissingField "name"))
       | Some (`String "atom") ->
           (match List.assoc_opt "value" fields with
            | Some value -> 
                Result.map (fun a -> Ast.EAtom a) (atom_of_json value)
            | None -> Error (MissingField "value"))
       | Some (`String "dynamic") ->
           let id = match List.assoc_opt "id" fields with
             | Some (`Int id) -> Ok id
             | Some _ -> Error (TypeMismatch "id should be an integer")
             | None -> Error (MissingField "id")
           in
           let expr = match List.assoc_opt "expr" fields with
             | Some e -> expr_of_json e
             | None -> Error (MissingField "expr")
           in
           (match id, expr with
            | Ok i, Ok e -> Ok (Ast.EDynamic (i, e))
            | Error e, _ -> Error e
            | _, Error e -> Error e)
       | Some _ -> Error (TypeMismatch "Invalid type field")
       | None -> Error (MissingField "type"))
  | _ -> Error (TypeMismatch "Expected object for expression")

(* ------------ RESOURCE TYPE CONVERSION ------------ *)

(** Convert a resource to JSON *)
let resource_to_json (resource: resource) : json =
  `Assoc [
    ("id", entity_id_to_json resource.id);
    ("name", `String resource.name);
    ("domain_id", domain_id_to_json resource.domain_id);
    ("resource_type", `String resource.resource_type);
    ("quantity", `Intlit (Int64.to_string resource.quantity));
    ("timestamp", timestamp_to_json resource.timestamp);
  ]

(** Convert JSON to a resource *)
let resource_of_json (json: json) : resource result =
  match json with
  | `Assoc fields ->
      let id = match List.assoc_opt "id" fields with
        | Some id_json -> entity_id_of_json id_json
        | None -> Error (MissingField "id")
      in
      let name = match List.assoc_opt "name" fields with
        | Some (`String n) -> Ok n
        | Some _ -> Error (TypeMismatch "name should be a string")
        | None -> Error (MissingField "name")
      in
      let domain_id = match List.assoc_opt "domain_id" fields with
        | Some domain_json -> domain_id_of_json domain_json
        | None -> Error (MissingField "domain_id")
      in
      let resource_type = match List.assoc_opt "resource_type" fields with
        | Some (`String rt) -> Ok rt
        | Some _ -> Error (TypeMismatch "resource_type should be a string")
        | None -> Error (MissingField "resource_type")
      in
      let quantity = match List.assoc_opt "quantity" fields with
        | Some (`Intlit s) -> 
            (try Ok (Int64.of_string s)
             with _ -> Error (TypeMismatch "Invalid quantity format"))
        | Some (`Int i) -> Ok (Int64.of_int i)
        | Some _ -> Error (TypeMismatch "quantity should be an integer")
        | None -> Error (MissingField "quantity")
      in
      let timestamp = match List.assoc_opt "timestamp" fields with
        | Some ts_json -> timestamp_of_json ts_json
        | None -> Error (MissingField "timestamp")
      in
      
      (* Combine all results *)
      match id, name, domain_id, resource_type, quantity, timestamp with
      | Ok i, Ok n, Ok d, Ok rt, Ok q, Ok ts ->
          Ok { id = i; name = n; domain_id = d; resource_type = rt; quantity = q; timestamp = ts }
      | Error e, _, _, _, _, _ -> Error e
      | _, Error e, _, _, _, _ -> Error e
      | _, _, Error e, _, _, _ -> Error e
      | _, _, _, Error e, _, _ -> Error e
      | _, _, _, _, Error e, _ -> Error e
      | _, _, _, _, _, Error e -> Error e
  | _ -> Error (TypeMismatch "Expected object for resource")

(* ------------ EFFECT TYPE CONVERSION ------------ *)

(** Convert an effect instance to JSON *)
let effect_to_json (effect: Ocaml_causality_effects.Effects.effect_instance) : json =
  `Assoc [
    ("tag", `String effect.tag);
    ("payload", value_expr_to_json effect.payload);
    ("id", match effect.id with Some id -> entity_id_to_json id | None -> `Null);
    ("context", `Assoc (List.map (fun (k, v) -> (k, `String v)) effect.context))
  ]

(** Convert JSON to an effect instance *)
let effect_of_json (json: json) : Ocaml_causality_effects.Effects.effect_instance result =
  match json with
  | `Assoc fields ->
      let tag = match List.assoc_opt "tag" fields with
        | Some (`String t) -> Ok t
        | Some _ -> Error (TypeMismatch "tag should be a string")
        | None -> Error (MissingField "tag")
      in
      let payload = match List.assoc_opt "payload" fields with
        | Some p_json -> value_expr_of_json p_json
        | None -> Error (MissingField "payload")
      in
      let id = match List.assoc_opt "id" fields with
        | Some (`Null) -> Ok None
        | Some id_json -> Result.map Option.some (entity_id_of_json id_json)
        | None -> Ok None  (* Optional field *)
      in
      let context = match List.assoc_opt "context" fields with
        | Some (`Assoc ctx) ->
            let rec process_context = function
              | [] -> Ok []
              | (k, `String v) :: rest ->
                  (match process_context rest with
                   | Ok items -> Ok ((k, v) :: items)
                   | Error e -> Error e)
              | (_, _) :: _ -> Error (TypeMismatch "context values should be strings")
            in
            process_context ctx
        | Some _ -> Error (TypeMismatch "context should be an object")
        | None -> Ok []  (* Default to empty context *)
      in
      
      (* Combine all results *)
      match tag, payload, id, context with
      | Ok t, Ok p, Ok i, Ok c ->
          Ok { Ocaml_causality_effects.Effects.tag = t; 
               payload = p; 
               id = i; 
               context = c }
      | Error e, _, _, _ -> Error e
      | _, Error e, _, _ -> Error e
      | _, _, Error e, _ -> Error e
      | _, _, _, Error e -> Error e
  | _ -> Error (TypeMismatch "Expected object for effect")

(* ------------ PUBLIC API ------------ *)

(** Convert any Causality value to JSON *)
let to_json (value: 'a) (value_type: string) : json =
  match value_type with
  | "expr" -> expr_to_json (Obj.magic value)
  | "value_expr" -> value_expr_to_json (Obj.magic value)
  | "atom" -> atom_to_json (Obj.magic value)
  | "resource" -> resource_to_json (Obj.magic value)
  | "effect" -> effect_to_json (Obj.magic value)
  | "entity_id" -> entity_id_to_json (Obj.magic value)
  | "timestamp" -> timestamp_to_json (Obj.magic value)
  | "domain_id" -> domain_id_to_json (Obj.magic value)
  | _ -> `Null  (* Unsupported type *)

(** Convert JSON to a specific Causality type *)
let of_json (json: json) (value_type: string) : 'a result =
  match value_type with
  | "expr" -> Result.map Obj.magic (expr_of_json json)
  | "value_expr" -> Result.map Obj.magic (value_expr_of_json json)
  | "atom" -> Result.map Obj.magic (atom_of_json json)
  | "resource" -> Result.map Obj.magic (resource_of_json json)
  | "effect" -> Result.map Obj.magic (effect_of_json json)
  | "entity_id" -> Result.map Obj.magic (entity_id_of_json json)
  | "timestamp" -> Result.map Obj.magic (timestamp_of_json json)
  | "domain_id" -> Result.map Obj.magic (domain_id_of_json json)
  | _ -> Error (UnsupportedType ("Conversion not supported for type: " ^ value_type))

(** Convert a value to a JSON string *)
let to_string (value: 'a) (value_type: string) : string =
  to_json value value_type |> to_string

(** Parse a JSON string to a specific Causality type *)
let parse_string (json_str: string) (value_type: string) : 'a result =
  try
    let json = from_string json_str in
    of_json json value_type
  with
  | Yojson.Json_error msg -> Error (InvalidJson msg)
  | exn -> Error (InvalidJson (Printexc.to_string exn)) 