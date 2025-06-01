(*
 * RPC Module Interface
 *
 * This module provides remote procedure call functionality for
 * communicating with external services and distributed systems.
 *)

open Ocaml_causality_lang
open Ocaml_causality_effects

(** RPC service ID *)
type service_id = string

(** RPC method ID *)
type method_id = string

(** RPC error type *)
type rpc_error =
  | ConnectionError of string    (** Connection-related error *)
  | RequestError of string       (** Error in request format/encoding *)
  | ResponseError of string      (** Error in response format/decoding *)
  | TimeoutError of string       (** Timeout while waiting for response *)
  | ServiceError of string       (** Error from the service *)
  | UnavailableError of string   (** Service or method unavailable *)

(** Result type for RPC operations *)
type 'a result = ('a, rpc_error) Result.t

(** RPC request payload *)
type request_payload = {
  service: service_id;           (** Target service *)
  method_name: method_id;        (** Target method *)
  params: Ast.value_expr;        (** Parameters as a value expression *)
  request_id: string;            (** Unique request ID *)
  timeout_ms: int option;        (** Optional timeout in milliseconds *)
  context: (string * string) list; (** Request context *)
}

(** RPC response payload *)
type response_payload = {
  request_id: string;            (** Corresponding request ID *)
  result: response_result;       (** Response result *)
  execution_time_ms: int option; (** Execution time in milliseconds *)
  context: (string * string) list; (** Response context *)
}

(** Response result *)
and response_result =
  | Success of Ast.value_expr    (** Successful result value *)
  | Error of {
      code: int;                 (** Error code *)
      message: string;           (** Error message *)
      details: Ast.value_expr option; (** Optional error details *)
    } (** Error information *)

(** RPC service definition *)
type service_definition = {
  service_id: service_id;        (** Unique service identifier *)
  methods: method_definition list; (** Available methods *)
  description: string option;    (** Optional service description *)
}

(** RPC method definition *)
and method_definition = {
  method_id: method_id;          (** Unique method identifier *)
  handler: request_payload -> response_result; (** Method implementation *)
  param_schema: string option;   (** Optional JSON schema for parameters *)
  return_schema: string option;  (** Optional JSON schema for return value *)
  method_description: string option;    (** Optional method description *)
}

(** RPC client configuration *)
type client_config = {
  default_timeout_ms: int;       (** Default timeout in milliseconds *)
  retry_count: int;              (** Number of retries on failure *)
  retry_delay_ms: int;           (** Delay between retries *)
}

(** RPC server configuration *)
type server_config = {
  max_concurrent_requests: int;  (** Maximum concurrent requests *)
  request_timeout_ms: int;       (** Request timeout in milliseconds *)
  max_request_size_bytes: int;   (** Maximum request size *)
}

(** Default client configuration *)
val default_client_config : client_config

(** Default server configuration *)
val default_server_config : server_config

(** Register a service with the registry *)
val register_service : service_definition -> unit

(** Get a service definition from the registry *)
val get_service : service_id -> service_definition option

(** Get a method definition from a service *)
val get_method : service_id -> method_id -> method_definition option

(** List all registered services *)
val list_services : unit -> service_definition list

(** Create a new RPC client *)
val create_client : ?config:client_config -> unit -> client_config

(** Create a request payload *)
val create_request :
  service:service_id ->
  method_name:method_id ->
  params:Ast.value_expr ->
  ?request_id:string ->
  ?timeout_ms:int option ->
  ?context:(string * string) list ->
  unit ->
  request_payload

(** Send an RPC request *)
val send_request : client_config -> request_payload -> response_payload result

(** Create a new RPC server *)
val create_server : ?config:server_config -> unit -> server_config

(** Start the server *)
val start_server : server_config -> int -> unit

(** Stop the server *)
val stop_server : server_config -> unit

(** Create a method that executes a pure function *)
val create_pure_method :
  method_id ->
  (Ast.value_expr -> Ast.value_expr) ->
  ?param_schema:string option ->
  ?return_schema:string option ->
  ?description:string option ->
  unit ->
  method_definition

(** Create a method that executes an effect *)
val create_effect_method :
  method_id ->
  Effects.effect_tag ->
  ?param_schema:string option ->
  ?return_schema:string option ->
  ?description:string option ->
  unit ->
  method_definition

(** Create a new service definition *)
val create_service :
  service_id ->
  method_definition list ->
  ?description:string option ->
  unit ->
  service_definition

(** Add a method to a service *)
val add_method : service_definition -> method_definition -> service_definition

(** Convert a request payload to a value expression *)
val request_to_value : request_payload -> Ast.value_expr

(** Convert a response payload to a value expression *)
val response_to_value : response_payload -> Ast.value_expr 