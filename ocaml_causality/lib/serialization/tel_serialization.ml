(* Purpose: TEL (Temporal Effect Logic) serialization functions using SSZ *)

(* Basic serialization functions - simplified version *)
module Basic = struct
  let serialize_uint32 n =
    let b = Bytes.create 4 in
    for i = 0 to 3 do
      Bytes.set b i (Char.chr ((n lsr (i * 8)) land 0xff))
    done;
    Bytes.to_string b
    
  let serialize_string s =
    let len = String.length s in
    let len_bytes = serialize_uint32 len in
    len_bytes ^ s

  let bytes_to_hex bytes =
    let hex_chars = "0123456789abcdef" in
    let len = String.length bytes in
    let hex = Bytes.create (len * 2) in
    for i = 0 to len - 1 do
      let byte = Char.code (String.get bytes i) in
      Bytes.set hex (i * 2) hex_chars.[byte lsr 4];
      Bytes.set hex (i * 2 + 1) hex_chars.[byte land 15]
    done;
    Bytes.to_string hex
end

(* TEL basic types for serialization *)
type resource_id = string  (* 32-byte identifier *)
type effect_id = string    (* 32-byte identifier *)  
type handler_id = string   (* 32-byte identifier *)

(* TEL graph components *)
type tel_resource = {
  id: resource_id;
  name: string;
  properties: (string * string) list;
}

type tel_effect = {
  id: effect_id;
  name: string;
  resource_id: resource_id;
  parameters: string list;
}

type tel_handler = {
  id: handler_id;
  name: string;
  effect_id: effect_id;
  implementation: string;
}

type tel_edge = {
  from_id: string;
  to_id: string;
  edge_type: string;
}

type tel_graph = {
  resources: tel_resource list;
  effects: tel_effect list;
  handlers: tel_handler list;
  edges: tel_edge list;
}

(* Serialization functions using basic SSZ *)
module Serialize = struct
  let serialize_string_list strings =
    let count = List.length strings in
    let count_bytes = Basic.serialize_uint32 count in
    let strings_bytes = String.concat "" (List.map Basic.serialize_string strings) in
    count_bytes ^ strings_bytes
  
  let serialize_property_list props =
    let count = List.length props in
    let count_bytes = Basic.serialize_uint32 count in
    let props_bytes = String.concat "" (List.map (fun (k, v) -> 
      Basic.serialize_string k ^ Basic.serialize_string v
    ) props) in
    count_bytes ^ props_bytes
  
  let serialize_tel_resource (resource : tel_resource) : string =
    Basic.serialize_string resource.id ^
    Basic.serialize_string resource.name ^
    serialize_property_list resource.properties
    
  let serialize_tel_effect (effect : tel_effect) : string =
    Basic.serialize_string effect.id ^
    Basic.serialize_string effect.name ^
    Basic.serialize_string effect.resource_id ^
    serialize_string_list effect.parameters
    
  let serialize_tel_handler (handler : tel_handler) : string =
    Basic.serialize_string handler.id ^
    Basic.serialize_string handler.name ^
    Basic.serialize_string handler.effect_id ^
    Basic.serialize_string handler.implementation
    
  let serialize_tel_edge edge =
    Basic.serialize_string edge.from_id ^
    Basic.serialize_string edge.to_id ^
    Basic.serialize_string edge.edge_type
    
  let serialize_tel_graph graph =
    let resources_count = List.length graph.resources in
    let effects_count = List.length graph.effects in
    let handlers_count = List.length graph.handlers in
    let edges_count = List.length graph.edges in
    
    let header = 
      Basic.serialize_uint32 resources_count ^
      Basic.serialize_uint32 effects_count ^
      Basic.serialize_uint32 handlers_count ^
      Basic.serialize_uint32 edges_count in
      
    let resources_bytes = String.concat "" (List.map serialize_tel_resource graph.resources) in
    let effects_bytes = String.concat "" (List.map serialize_tel_effect graph.effects) in
    let handlers_bytes = String.concat "" (List.map serialize_tel_handler graph.handlers) in
    let edges_bytes = String.concat "" (List.map serialize_tel_edge graph.edges) in
    
    header ^ resources_bytes ^ effects_bytes ^ handlers_bytes ^ edges_bytes
end

(* FFI functions for Rust interop *)
module Ffi = struct
  let serialize_tel_graph_to_hex graph =
    let bytes = Serialize.serialize_tel_graph graph in
    Basic.bytes_to_hex bytes
    
  let tel_graph_content_hash graph =
    let bytes = Serialize.serialize_tel_graph graph in
    let hash = Digest.string bytes in
    Digest.to_hex hash
end

(* Public API functions *)
let serialize_tel_resource = Serialize.serialize_tel_resource
let serialize_tel_effect = Serialize.serialize_tel_effect  
let serialize_tel_handler = Serialize.serialize_tel_handler
let serialize_tel_edge = Serialize.serialize_tel_edge
let serialize_tel_graph = Serialize.serialize_tel_graph
let serialize_tel_graph_to_hex = Ffi.serialize_tel_graph_to_hex
let tel_graph_content_hash = Ffi.tel_graph_content_hash 