(* ------------ QUERY STATE PRIMITIVES ------------ *)
(* Purpose: OCaml primitives for state queries with type safety *)

(* Query result types *)
type query_result = 
  | Success of string
  | Error of string

type query_status = 
  | Pending
  | Completed
  | Failed of string
  | Cached

(* Query configuration *)
type query_config = {
  contract_id : string;
  domain : string;
  layout_commitment : string;
  timeout_ms : int;
  use_cache : bool;
}

(* Query handle for async operations *)
type query_handle = {
  query_id : string;
  config : query_config;
  mutable status : query_status;
  submitted_at : float;
}

(* Multi-chain query coordination *)
type multi_chain_query = {
  queries : (string * query_config * string) list; (* (field, config, key) *)
  coordination_strategy : coordination_strategy;
}

and coordination_strategy = 
  | Sequential
  | Parallel
  | Conditional of (query_result -> bool)

(* Core query primitive *)
let query_state (config : query_config) (field : string) (key : string) : query_result =
  try
    (* Mock implementation - would call Almanac API *)
    let result_data = Printf.sprintf "mock_result_%s_%s_%s" config.contract_id field key in
    Success result_data
  with
  | exn -> Error (Printexc.to_string exn)

(* Type-safe query functions *)
let query_balance (config : query_config) (address : string) : query_result =
  query_state config "balances" address

let query_allowance (config : query_config) (owner : string) (spender : string) : query_result =
  let key = Printf.sprintf "%s:%s" owner spender in
  query_state config "allowances" key

let query_total_supply (config : query_config) : query_result =
  query_state config "totalSupply" ""

let query_owner (config : query_config) : query_result =
  query_state config "owner" ""

(* Configuration builders *)
let make_query_config ~contract_id ~domain ?(layout_commitment="") ?(timeout_ms=5000) ?(use_cache=true) () =
  { contract_id; domain; layout_commitment; timeout_ms; use_cache }

let ethereum_config contract_id = 
  make_query_config ~contract_id ~domain:"ethereum" ()

let cosmos_config contract_id = 
  make_query_config ~contract_id ~domain:"cosmos" ()

(* Async query operations *)
let submit_query_async (config : query_config) (field : string) (key : string) : query_handle =
  let query_id = Printf.sprintf "query_%f_%s" (Unix.time ()) field in
  {
    query_id;
    config;
    status = Pending;
    submitted_at = Unix.time ();
  }

let get_query_status (handle : query_handle) : query_status =
  (* Mock implementation - would check actual status *)
  if Unix.time () -. handle.submitted_at > 1.0 then
    Completed
  else
    handle.status

let wait_for_query (handle : query_handle) ?(timeout_ms=5000) () : query_result =
  let start_time = Unix.time () in
  let timeout_seconds = float_of_int timeout_ms /. 1000.0 in
  
  let rec wait () =
    match get_query_status handle with
    | Completed -> query_state handle.config "mock_field" "mock_key"
    | Failed msg -> Error msg
    | Pending | Cached ->
        if Unix.time () -. start_time > timeout_seconds then
          Error "Query timeout"
        else begin
          Unix.sleepf 0.1;
          wait ()
        end
  in
  wait ()

(* Multi-chain query coordination *)
let execute_multi_chain_query (multi_query : multi_chain_query) : (string * query_result) list =
  match multi_query.coordination_strategy with
  | Sequential ->
      List.map (fun (field, config, key) ->
        (field, query_state config field key)
      ) multi_query.queries
  | Parallel ->
      (* Mock parallel execution - would use actual parallel primitives *)
      List.map (fun (field, config, key) ->
        (field, query_state config field key)
      ) multi_query.queries
  | Conditional condition ->
      let rec execute_conditional queries acc =
        match queries with
        | [] -> List.rev acc
        | (field, config, key) :: rest ->
            let result = query_state config field key in
            let new_acc = (field, result) :: acc in
            if condition result then
              execute_conditional rest new_acc
            else
              List.rev new_acc
      in
      execute_conditional multi_query.queries []

(* Query composition and filtering *)
let compose_queries (queries : (query_config * string * string) list) : multi_chain_query =
  let query_list = List.map (fun (config, field, key) -> (field, config, key)) queries in
  { queries = query_list; coordination_strategy = Sequential }

let filter_results (results : (string * query_result) list) (predicate : query_result -> bool) : (string * query_result) list =
  List.filter (fun (_, result) -> predicate result) results

let map_results (results : (string * query_result) list) (mapper : query_result -> query_result) : (string * query_result) list =
  List.map (fun (field, result) -> (field, mapper result)) results

(* Caching utilities *)
module QueryCache = struct
  let cache = Hashtbl.create 64
  
  let cache_key (config : query_config) (field : string) (key : string) : string =
    Printf.sprintf "%s:%s:%s:%s" config.domain config.contract_id field key
  
  let get (config : query_config) (field : string) (key : string) : query_result option =
    let key = cache_key config field key in
    Hashtbl.find_opt cache key
  
  let put (config : query_config) (field : string) (key : string) (result : query_result) : unit =
    let cache_key = cache_key config field key in
    Hashtbl.replace cache cache_key result
  
  let invalidate (config : query_config) : unit =
    let prefix = Printf.sprintf "%s:%s:" config.domain config.contract_id in
    let keys_to_remove = Hashtbl.fold (fun k _ acc ->
      if String.length k >= String.length prefix && 
         String.sub k 0 (String.length prefix) = prefix then
        k :: acc
      else
        acc
    ) cache [] in
    List.iter (Hashtbl.remove cache) keys_to_remove
  
  let clear () : unit =
    Hashtbl.clear cache
end

(* Cached query functions *)
let cached_query_state (config : query_config) (field : string) (key : string) : query_result =
  if config.use_cache then
    match QueryCache.get config field key with
    | Some result -> result
    | None ->
        let result = query_state config field key in
        QueryCache.put config field key result;
        result
  else
    query_state config field key

let cached_query_balance (config : query_config) (address : string) : query_result =
  cached_query_state config "balances" address

let cached_query_allowance (config : query_config) (owner : string) (spender : string) : query_result =
  let key = Printf.sprintf "%s:%s" owner spender in
  cached_query_state config "allowances" key

(* Error handling utilities *)
let is_success = function
  | Success _ -> true
  | Error _ -> false

let is_error = function
  | Success _ -> false
  | Error _ -> true

let extract_result = function
  | Success data -> Some data
  | Error _ -> None

let extract_error = function
  | Success _ -> None
  | Error msg -> Some msg

let result_to_string = function
  | Success data -> data
  | Error msg -> Printf.sprintf "Error: %s" msg

(* Query result combinators *)
let bind_result (result : query_result) (f : string -> query_result) : query_result =
  match result with
  | Success data -> f data
  | Error msg -> Error msg

let map_result (result : query_result) (f : string -> string) : query_result =
  match result with
  | Success data -> Success (f data)
  | Error msg -> Error msg

let combine_results (results : query_result list) : query_result =
  let rec combine acc = function
    | [] -> Success (String.concat "," (List.rev acc))
    | Success data :: rest -> combine (data :: acc) rest
    | Error msg :: _ -> Error msg
  in
  combine [] results

(* Validation helpers *)
let validate_config (config : query_config) : bool =
  String.length config.contract_id > 0 &&
  String.length config.domain > 0 &&
  config.timeout_ms > 0

let validate_field (field : string) : bool =
  String.length field > 0 && 
  not (String.contains field ' ')

let validate_key (key : string) : bool =
  String.length key >= 0  (* Empty keys are allowed for some queries *)

(* Safe query wrappers *)
let safe_query_state (config : query_config) (field : string) (key : string) : query_result =
  if not (validate_config config) then
    Error "Invalid query configuration"
  else if not (validate_field field) then
    Error "Invalid field name"
  else if not (validate_key key) then
    Error "Invalid key"
  else
    cached_query_state config field key

let safe_query_balance (config : query_config) (address : string) : query_result =
  if String.length address = 0 then
    Error "Address cannot be empty"
  else
    safe_query_state config "balances" address

let safe_query_allowance (config : query_config) (owner : string) (spender : string) : query_result =
  if String.length owner = 0 || String.length spender = 0 then
    Error "Owner and spender addresses cannot be empty"
  else
    let key = Printf.sprintf "%s:%s" owner spender in
    safe_query_state config "allowances" key 