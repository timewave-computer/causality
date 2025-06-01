(*
 * External APIs Module Interface
 *
 * This module provides interfaces for integrating with external APIs
 * and services, including authentication, request/response handling,
 * and data transformation capabilities.
 *)

open Ocaml_causality_lang

(** Authentication types *)
type auth_method =
  | ApiKey of string              (** API key authentication *)
  | BasicAuth of string * string  (** Basic username/password auth *)

(** External API types *)
type api_type =
  | REST       (** RESTful HTTP API *)
  | GraphQL    (** GraphQL API *)
  | RPC        (** JSON-RPC API *)
  | Blockchain (** Blockchain API *)
  | Custom of string (** Custom API type *)

(** API credentials *)
type credentials =
  | NoAuth                     (** No authentication *)
  | APIKey of string           (** API key authentication *)
  | Bearer of string           (** Bearer token authentication *)
  | Basic of string * string   (** Basic authentication (username, password) *)
  | OAuth of {
      client_id: string;       (** OAuth client ID *)
      client_secret: string;   (** OAuth client secret *)
      token_url: string;       (** Token endpoint URL *)
      refresh_token: string option; (** Optional refresh token *)
      access_token: string option;  (** Optional access token *)
    } (** OAuth authentication *)

(** API configuration *)
type api_config = {
  api_type: api_type;            (** Type of API *)
  base_url: string;              (** Base URL for the API *)
  credentials: credentials;      (** Authentication credentials *)
  timeout_ms: int;               (** Timeout in milliseconds *)
  retry_count: int;              (** Number of retries *)
  headers: (string * string) list; (** Additional HTTP headers *)
}

(** API error *)
type api_error =
  | ConnectionError of string   (** Connection-related error *)
  | AuthenticationError of string (** Authentication error *)
  | RequestError of string      (** Error in request *)
  | ResponseError of string     (** Error in response *)
  | ParseError of string        (** Error parsing response *)
  | TimeoutError of string      (** Timeout error *)
  | RateLimitError of string    (** Rate limiting error *)

(** Result type for API operations *)
type 'a result = ('a, api_error) Result.t

(** HTTP method *)
type http_method = GET | POST | PUT | DELETE | PATCH | HEAD | OPTIONS

(** HTTP request *)
type http_request = {
  method_: http_method;
  url: string;
  headers: (string * string) list;
  query_params: (string * string) list;
  body: string option;
  timeout_ms: int option;
}

(** HTTP response *)
type http_response = {
  status_code: int;
  headers: (string * string) list;
  body: string;
}

(** Default API configuration *)
val default_api_config : api_config

(** Create an HTTP request *)
val create_request : 
    ?method_:http_method -> 
    url:string -> 
    ?headers:(string * string) list -> 
    ?query_params:(string * string) list -> 
    ?body:string option -> 
    ?timeout_ms:int option -> 
    unit -> 
    http_request

(** Send an HTTP request *)
val send_request : http_request -> http_response result

(** Create an API client *)
val create_api_client : ?config:api_config -> unit -> api_config

(** Add authentication headers to a request *)
val add_auth_headers : http_request -> credentials -> http_request

(** Make an API call *)
val call_api : 
    api_config -> 
    string -> 
    ?method_:http_method -> 
    ?query_params:(string * string) list -> 
    ?headers:(string * string) list -> 
    ?body:string option -> 
    unit -> 
    http_response result

(** Blockchain network type *)
type blockchain_network =
  | Ethereum   (** Ethereum mainnet *)
  | EthereumTestnet of string  (** Ethereum testnet *)
  | Bitcoin    (** Bitcoin network *)
  | Cosmos     (** Cosmos network *)
  | Custom of string (** Custom blockchain *)

(** Blockchain client configuration *)
type blockchain_config = {
  network: blockchain_network;   (** Blockchain network *)
  rpc_url: string;               (** RPC endpoint URL *)
  credentials: credentials;      (** Authentication credentials *)
  timeout_ms: int;               (** Timeout in milliseconds *)
}

(** Create a blockchain client *)
val create_blockchain_client : blockchain_config -> api_config

(** Send a blockchain transaction *)
val send_transaction : blockchain_config -> string -> string result

(** Query blockchain data *)
val query_blockchain : blockchain_config -> string -> Ast.value_expr result

(** Register API effects with the effect system *)
val register_api_effects : unit -> unit

(** Parse JSON response to a value expression *)
val parse_json_response : http_response -> Ast.value_expr result

(** Format a value expression as a JSON request body *)
val format_json_request : Ast.value_expr -> string result 