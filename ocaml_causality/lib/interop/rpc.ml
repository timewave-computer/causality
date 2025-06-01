(*
 * RPC Module
 *
 * This module provides Remote Procedure Call interfaces for the Causality
 * framework. It enables client-server communication and cross-process
 * integration of Causality components.
 *)

open Ocaml_causality_core
open Ocaml_causality_lang
open Ocaml_causality_effects

(* ------------ RPC TYPES ------------ *)

(** RPC service ID *)
type service_id = string

(** RPC method ID *)
type method_id = string

(** RPC error type *)
type rpc_error =
  | ConnectionError of string    (* Connection-related error *)
  | RequestError of string       (* Error in request format/encoding *)
  | ResponseError of string      (* Error in response format/decoding *)
  | TimeoutError of string       (* Timeout while waiting for response *)
  | ServiceError of string       (* Error from the service *)
  | UnavailableError of string   (* Service or method unavailable *)

(** Result type for RPC operations *)
type 'a result = ('a, rpc_error) Result.t

(** RPC request payload *)
type request_payload = {
  service: service_id;           (* Target service *)
  method_name: method_id;        (* Target method *)
  params: Ast.value_expr;        (* Parameters as a value expression *)
  request_id: string;            (* Unique request ID *)
  timeout_ms: int option;        (* Optional timeout in milliseconds *)
  context: (string * string) list; (* Request context *)
}

(** RPC response payload *)
type response_payload = {
  request_id: string;            (* Corresponding request ID *)
  result: response_result;       (* Response result *)
  execution_time_ms: int option; (* Execution time in milliseconds *)
  context: (string * string) list; (* Response context *)
}

(** Response result *)
and response_result =
  | Success of Ast.value_expr    (* Successful result value *)
  | Error of {                   (* Error information *)
      code: int;                 (* Error code *)
      message: string;           (* Error message *)
      details: Ast.value_expr option; (* Optional error details *)
    }

(** RPC service definition *)
type service_definition = {
  service_id: service_id;        (* Unique service identifier *)
  methods: method_definition list; (* Available methods *)
  description: string option;    (* Optional service description *)
}

(** RPC method definition *)
and method_definition = {
  method_id: method_id;          (* Unique method identifier *)
  handler: request_payload -> response_result; (* Method implementation *)
  param_schema: string option;   (* Optional JSON schema for parameters *)
  return_schema: string option;  (* Optional JSON schema for return value *)
  description: string option;    (* Optional method description *)
}

(** RPC client configuration *)
type client_config = {
  default_timeout_ms: int;       (* Default timeout in milliseconds *)
  retry_count: int;              (* Number of retries on failure *)
  retry_delay_ms: int;           (* Delay between retries *)
}

(** RPC server configuration *)
type server_config = {
  max_concurrent_requests: int;  (* Maximum concurrent requests *)
  request_timeout_ms: int;       (* Request timeout in milliseconds *)
  max_request_size_bytes: int;   (* Maximum request size *)
}

(* ------------ RPC REGISTRY ------------ *)

(** Registry of available services *)
let service_registry : (service_id, service_definition) Hashtbl.t =
  Hashtbl.create 16

(** Register a service with the registry *)
let register_service (service: service_definition) : unit =
  Hashtbl.replace service_registry service.service_id service

(** Get a service definition from the registry *)
let get_service (service_id: service_id) : service_definition option =
  Hashtbl.find_opt service_registry service_id

(** Get a method definition from a service *)
let get_method (service_id: service_id) (method_id: method_id) : method_definition option =
  match get_service service_id with
  | Some service ->
      List.find_opt (fun m -> m.method_id = method_id) service.methods
  | None -> None

(** List all registered services *)
let list_services () : service_definition list =
  Hashtbl.fold (fun _id service acc -> service :: acc) service_registry []

(* ------------ RPC CLIENT ------------ *)

(** Default client configuration *)
let default_client_config = {
  default_timeout_ms = 30000;  (* 30 seconds *)
  retry_count = 3;
  retry_delay_ms = 1000;       (* 1 second *)
}

(** Create a new RPC client *)
let create_client ?(config=default_client_config) () =
  (* For MVP, we'll just return the config *)
  config

(** Create a request payload *)
let create_request
    ~service
    ~method_name
    ~params
    ?(request_id=Uuidm.v4_gen (Random.State.make_self_init ()) () |> Uuidm.to_string)
    ?(timeout_ms=None)
    ?(context=[])
    () : request_payload =
  { service; method_name; params; request_id; timeout_ms; context }

(** Send an RPC request (placeholder implementation) *)
let send_request (_client: client_config) (request: request_payload) : response_payload result =
  (* For MVP, we'll check if the service and method exist in the registry *)
  match get_method request.service request.method_name with
  | Some method_def ->
      (* Execute the method handler *)
      let result = method_def.handler request in
      let response = {
        request_id = request.request_id;
        result;
        execution_time_ms = Some 0;  (* Placeholder *)
        context = [];  (* Placeholder *)
      } in
      Ok response
  | None ->
      Error (UnavailableError (
        Printf.sprintf "Service '%s' or method '%s' not available"
          request.service request.method_name))

(* ------------ RPC SERVER ------------ *)

(** Default server configuration *)
let default_server_config = {
  max_concurrent_requests = 100;
  request_timeout_ms = 30000;  (* 30 seconds *)
  max_request_size_bytes = 1024 * 1024;  (* 1MB *)
}

(** Create a new RPC server *)
let create_server ?(config=default_server_config) () =
  (* For MVP, we'll just return the config *)
  config

(** Start the server (placeholder implementation) *)
let start_server (_server: server_config) (_port: int) : unit =
  (* Placeholder: Would start a server listening on the given port *)
  ()

(** Stop the server (placeholder implementation) *)
let stop_server (_server: server_config) : unit =
  (* Placeholder: Would stop the server *)
  ()

(* ------------ METHOD BUILDERS ------------ *)

(** Create a method that executes a pure function *)
let create_pure_method
    (method_id: method_id)
    (f: Ast.value_expr -> Ast.value_expr)
    ?(param_schema=None)
    ?(return_schema=None)
    ?(description=None)
    () : method_definition =
  {
    method_id;
    handler = (fun request -> Success (f request.params));
    param_schema;
    return_schema;
    description;
  }

(** Create a method that executes an effect *)
let create_effect_method
    (method_id: method_id)
    (effect_tag: Effects.effect_tag)
    ?(param_schema=None)
    ?(return_schema=None)
    ?(description=None)
    () : method_definition =
  {
    method_id;
    handler = (fun request ->
      (* Create an effect instance *)
      let effect = Effects.create_effect effect_tag request.params () in
      
      (* Execute the effect *)
      match Registry.execute_effect effect with
      | Effects.Success value -> Success value
      | Effects.Failure msg -> Error { code = 500; message = msg; details = None }
      | Effects.Pending -> Error { code = 202; message = "Effect execution pending"; details = None }
    );
    param_schema;
    return_schema;
    description;
  }

(* ------------ SERVICE BUILDERS ------------ *)

(** Create a new service definition *)
let create_service
    (service_id: service_id)
    (methods: method_definition list)
    ?(description=None)
    () : service_definition =
  { service_id; methods; description }

(** Add a method to a service *)
let add_method (service: service_definition) (method_def: method_definition) : service_definition =
  (* Check if the method already exists *)
  let methods =
    if List.exists (fun m -> m.method_id = method_def.method_id) service.methods then
      (* Replace the existing method *)
      List.map
        (fun m -> if m.method_id = method_def.method_id then method_def else m)
        service.methods
    else
      (* Add the new method *)
      method_def :: service.methods
  in
  { service with methods }

(* ------------ UTILITY FUNCTIONS ------------ *)

(** Convert a request payload to a value expression *)
let request_to_value (request: request_payload) : Ast.value_expr =
  let context_items = List.map
    (fun (k, v) -> Ast.VList [
      Ast.VAtom (Ast.String k);
      Ast.VAtom (Ast.String v)
    ])
    request.context in
  
  Ast.VMap [
    (Ast.VAtom (Ast.String "service"), Ast.VAtom (Ast.String request.service));
    (Ast.VAtom (Ast.String "method"), Ast.VAtom (Ast.String request.method_name));
    (Ast.VAtom (Ast.String "params"), request.params);
    (Ast.VAtom (Ast.String "request_id"), Ast.VAtom (Ast.String request.request_id));
    (Ast.VAtom (Ast.String "timeout_ms"), match request.timeout_ms with
      | Some ms -> Ast.VAtom (Ast.Integer (Int64.of_int ms))
      | None -> Ast.VUnit);
    (Ast.VAtom (Ast.String "context"), Ast.VList context_items);
  ]

(** Convert a response payload to a value expression *)
let response_to_value (response: response_payload) : Ast.value_expr =
  let context_items = List.map
    (fun (k, v) -> Ast.VList [
      Ast.VAtom (Ast.String k);
      Ast.VAtom (Ast.String v)
    ])
    response.context in
  
  let result_value = match response.result with
    | Success value -> Ast.VMap [
        (Ast.VAtom (Ast.String "type"), Ast.VAtom (Ast.String "success"));
        (Ast.VAtom (Ast.String "value"), value);
      ]
    | Error err -> Ast.VMap [
        (Ast.VAtom (Ast.String "type"), Ast.VAtom (Ast.String "error"));
        (Ast.VAtom (Ast.String "code"), Ast.VAtom (Ast.Integer (Int64.of_int err.code)));
        (Ast.VAtom (Ast.String "message"), Ast.VAtom (Ast.String err.message));
        (Ast.VAtom (Ast.String "details"), match err.details with
          | Some details -> details
          | None -> Ast.VUnit);
      ]
  in
  
  Ast.VMap [
    (Ast.VAtom (Ast.String "request_id"), Ast.VAtom (Ast.String response.request_id));
    (Ast.VAtom (Ast.String "result"), result_value);
    (Ast.VAtom (Ast.String "execution_time_ms"), match response.execution_time_ms with
      | Some ms -> Ast.VAtom (Ast.Integer (Int64.of_int ms))
      | None -> Ast.VUnit);
    (Ast.VAtom (Ast.String "context"), Ast.VList context_items);
  ] 