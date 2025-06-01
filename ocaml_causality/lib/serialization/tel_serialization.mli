(* Purpose: TEL (Temporal Effect Logic) serialization functions using SSZ *)

(** Basic serialization types *)
type resource_id = string  (* 32-byte identifier *)
type effect_id = string    (* 32-byte identifier *)  
type handler_id = string   (* 32-byte identifier *)

(** TEL graph components *)
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

(** Serialize a TEL resource to bytes *)
val serialize_tel_resource : tel_resource -> string

(** Serialize a TEL effect to bytes *)
val serialize_tel_effect : tel_effect -> string

(** Serialize a TEL handler to bytes *)
val serialize_tel_handler : tel_handler -> string

(** Serialize a TEL edge to bytes *)
val serialize_tel_edge : tel_edge -> string

(** Serialize a complete TEL graph to bytes *)
val serialize_tel_graph : tel_graph -> string

(** Serialize a TEL graph to hexadecimal string *)
val serialize_tel_graph_to_hex : tel_graph -> string

(** Generate content hash for a TEL graph *)
val tel_graph_content_hash : tel_graph -> string 