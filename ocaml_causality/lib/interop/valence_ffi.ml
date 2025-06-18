(* ------------ VALENCE FFI BINDINGS ------------ *)
(* Purpose: FFI bindings for Valence account factory operations *)

(* Abstract types for Valence FFI handles *)
type valence_account
type valence_library
type valence_transaction

(* Account factory configuration *)
type account_factory_config = {
    owner : string
  ; account_type : string
  ; permissions : string list
}

(* Library approval configuration *)
type library_approval_config = {
    account : string
  ; library : string
  ; permissions : string list
}

(* Transaction submission configuration *)
type transaction_config = {
    account : string
  ; operation : string
  ; data : string
}

(* --------------------------------------------------------------------------- *)
(* External C function declarations for Valence operations *)
(* --------------------------------------------------------------------------- *)

(* Account factory operations *)
external create_account_factory_ffi : string -> string -> string array -> valence_account
  = "ocaml_valence_create_account_factory"

external get_account_status_ffi : string -> string
  = "ocaml_valence_get_account_status"

external is_account_valid_ffi : string -> bool
  = "ocaml_valence_is_account_valid"

(* Library approval operations *)
external approve_library_ffi : string -> string -> string array -> bool
  = "ocaml_valence_approve_library"

external list_approved_libraries_ffi : string -> string array
  = "ocaml_valence_list_approved_libraries"

external is_library_approved_ffi : string -> string -> bool
  = "ocaml_valence_is_library_approved"

(* Transaction operations *)
external submit_transaction_ffi : string -> string -> string -> string
  = "ocaml_valence_submit_transaction"

external get_transaction_history_ffi : string -> int -> string array
  = "ocaml_valence_get_transaction_history"

external get_transaction_status_ffi : string -> string
  = "ocaml_valence_get_transaction_status"

(* Validation operations *)
external validate_account_owner_ffi : string -> bool
  = "ocaml_valence_validate_account_owner"

external validate_library_ffi : string -> bool
  = "ocaml_valence_validate_library"

external validate_transaction_ffi : string -> string -> bool
  = "ocaml_valence_validate_transaction"

(* --------------------------------------------------------------------------- *)
(* High-level OCaml interface *)
(* --------------------------------------------------------------------------- *)

(** Create an account factory account *)
let create_account_factory (config : account_factory_config) =
  let permissions_array = Array.of_list config.permissions in
  try
    let account = create_account_factory_ffi config.owner config.account_type permissions_array in
    Ok account
  with
  | exn -> Error (Printf.sprintf "Failed to create account factory: %s" (Printexc.to_string exn))

(** Get account factory status *)
let get_account_status account_id =
  try
    let status = get_account_status_ffi account_id in
    Ok status
  with
  | exn -> Error (Printf.sprintf "Failed to get account status: %s" (Printexc.to_string exn))

(** Check if account is valid *)
let is_account_valid account_id =
  try
    is_account_valid_ffi account_id
  with
  | _ -> false

(** Approve library for account *)
let approve_library (config : library_approval_config) =
  let permissions_array = Array.of_list config.permissions in
  try
    let success = approve_library_ffi config.account config.library permissions_array in
    if success then Ok () else Error "Library approval failed"
  with
  | exn -> Error (Printf.sprintf "Failed to approve library: %s" (Printexc.to_string exn))

(** List approved libraries for account *)
let list_approved_libraries account_id =
  try
    let libraries = list_approved_libraries_ffi account_id in
    Ok (Array.to_list libraries)
  with
  | exn -> Error (Printf.sprintf "Failed to list approved libraries: %s" (Printexc.to_string exn))

(** Check if library is approved for account *)
let is_library_approved account_id library_id =
  try
    is_library_approved_ffi account_id library_id
  with
  | _ -> false

(** Submit transaction to account *)
let submit_transaction (config : transaction_config) =
  try
    let tx_id = submit_transaction_ffi config.account config.operation config.data in
    Ok tx_id
  with
  | exn -> Error (Printf.sprintf "Failed to submit transaction: %s" (Printexc.to_string exn))

(** Get transaction history for account *)
let get_transaction_history account_id limit =
  try
    let history = get_transaction_history_ffi account_id limit in
    Ok (Array.to_list history)
  with
  | exn -> Error (Printf.sprintf "Failed to get transaction history: %s" (Printexc.to_string exn))

(** Get transaction status *)
let get_transaction_status tx_id =
  try
    let status = get_transaction_status_ffi tx_id in
    Ok status
  with
  | exn -> Error (Printf.sprintf "Failed to get transaction status: %s" (Printexc.to_string exn))

(** Validate account owner *)
let validate_account_owner owner =
  try
    validate_account_owner_ffi owner
  with
  | _ -> false

(** Validate library *)
let validate_library library_id =
  try
    validate_library_ffi library_id
  with
  | _ -> false

(** Validate transaction *)
let validate_transaction account_id operation =
  try
    validate_transaction_ffi account_id operation
  with
  | _ -> false

(* --------------------------------------------------------------------------- *)
(* Configuration builders *)
(* --------------------------------------------------------------------------- *)

(** Create account factory configuration *)
let make_account_factory_config ~owner ?(account_type="factory") ?(permissions=["read"; "write"; "execute"]) () =
  { owner; account_type; permissions }

(** Create library approval configuration *)
let make_library_approval_config ~account ~library ?(permissions=["read"; "execute"]) () =
  { account; library; permissions }

(** Create transaction configuration *)
let make_transaction_config ~account ~operation ?(data="") () =
  { account; operation; data }

(* --------------------------------------------------------------------------- *)
(* Safe wrapper functions *)
(* --------------------------------------------------------------------------- *)

(** Safe account factory creation with validation *)
let safe_create_account_factory (config : account_factory_config) =
  if not (validate_account_owner config.owner) then
    Error "Invalid account owner"
  else if config.account_type <> "factory" then
    Error "Only factory account type is supported"
  else
    create_account_factory config

(** Safe library approval with validation *)
let safe_approve_library (config : library_approval_config) =
  if not (is_account_valid config.account) then
    Error "Invalid account"
  else if not (validate_library config.library) then
    Error "Invalid library"
  else
    approve_library config

(** Safe transaction submission with validation *)
let safe_submit_transaction (config : transaction_config) =
  if not (is_account_valid config.account) then
    Error "Invalid account"
  else if not (validate_transaction config.account config.operation) then
    Error "Invalid transaction"
  else
    submit_transaction config

(* --------------------------------------------------------------------------- *)
(* Integration with causality-toolkit interface synthesis *)
(* --------------------------------------------------------------------------- *)

(** Generate OCaml interface from account factory configuration *)
let generate_account_interface (config : account_factory_config) =
  let module_name = Printf.sprintf "Account_%s" (String.capitalize_ascii config.owner) in
  let interface_code = Printf.sprintf {|
module %s = struct
  let account_id = "%s"
  let account_type = "%s"
  let permissions = [%s]
  
  let create () = 
    Valence_ffi.safe_create_account_factory {
      owner = "%s";
      account_type = "%s";
      permissions = [%s]
    }
  
  let approve_library library_id =
    Valence_ffi.safe_approve_library {
      account = account_id;
      library = library_id;
      permissions = ["read"; "execute"]
    }
  
  let submit_transaction operation data =
    Valence_ffi.safe_submit_transaction {
      account = account_id;
      operation = operation;
      data = data
    }
  
  let get_status () = Valence_ffi.get_account_status account_id
  let list_libraries () = Valence_ffi.list_approved_libraries account_id
  let get_history limit = Valence_ffi.get_transaction_history account_id limit
end
|} 
    module_name
    config.owner
    config.account_type
    (String.concat "; " (List.map (Printf.sprintf "\"%s\"") config.permissions))
    config.owner
    config.account_type
    (String.concat "; " (List.map (Printf.sprintf "\"%s\"") config.permissions))
  in
  Ok interface_code

(** Generate deployment script for account factory *)
let generate_deployment_script (configs : account_factory_config list) =
  let script_header = "#!/bin/bash\n# Account Factory Deployment Script\n# Generated by Causality-Valence Integration\n\n" in
  let script_body = List.fold_left (fun acc config ->
    acc ^ Printf.sprintf "echo \"Creating account factory for %s...\"\n" config.owner ^
    Printf.sprintf "# Account: %s, Type: %s\n" config.owner config.account_type ^
    Printf.sprintf "# Permissions: %s\n\n" (String.concat ", " config.permissions)
  ) "" configs in
  let script_footer = "echo \"Account factory deployment complete\"\n" in
  Ok (script_header ^ script_body ^ script_footer)

(* ------------ VALENCE COPROCESSOR FFI INTEGRATION ------------ *)
(* Purpose: Real FFI bindings to Valence coprocessor APIs *)

open Lwt.Syntax

(* Real external function declarations for Valence coprocessor *)
external valence_create_account_factory : string -> string -> string array -> string = "caml_valence_create_account_factory"
external valence_approve_library : string -> string -> string -> string = "caml_valence_approve_library" 
external valence_execute_transaction : string -> string -> string -> string = "caml_valence_execute_transaction"
external valence_query_account_state : string -> string -> string = "caml_valence_query_account_state"
external valence_get_account_balance : string -> string -> string option -> string = "caml_valence_get_account_balance"
external valence_submit_transaction : string -> string -> string -> string = "caml_valence_submit_transaction"
external valence_wait_for_confirmation : string -> string -> int -> string = "caml_valence_wait_for_confirmation"

(* Real Valence integration types *)
type account_id = string
type library_id = string  
type transaction_hash = string
type chain_id = string

type account_creation_result = {
  account_id : account_id;
  transaction_hash : transaction_hash;
  block_number : int64;
  status : [`Pending | `Confirmed | `Failed of string];
}

type library_approval_result = {
  account_id : account_id;
  library_id : library_id;
  transaction_hash : transaction_hash;
  block_number : int64;
  status : [`Pending | `Confirmed | `Failed of string];
}

type transaction_result = {
  transaction_hash : transaction_hash;
  block_number : int64;
  gas_used : int64;
  status : [`Pending | `Confirmed | `Failed of string];
  logs : Yojson.Safe.t;
}

type valence_config = {
  coprocessor_endpoint : string;
  default_gas_limit : int64;
  default_gas_price : int64;
  transaction_timeout_seconds : int;
  max_retry_attempts : int;
}

(* Default configuration *)
let default_config = {
  coprocessor_endpoint = "http://localhost:8080";
  default_gas_limit = 500_000L;
  default_gas_price = 20_000_000_000L;
  transaction_timeout_seconds = 300;
  max_retry_attempts = 3;
}

(* Error handling *)
exception Valence_error of string
exception Account_creation_failed of string
exception Library_approval_failed of string
exception Transaction_failed of string

(* Parse JSON responses from Rust FFI *)
let parse_account_creation_result json_str =
  try
    let json = Yojson.Safe.from_string json_str in
    let open Yojson.Safe.Util in
    let account_id = json |> member "account_id" |> to_string in
    let transaction_hash = json |> member "transaction_hash" |> to_string in
    let block_number = json |> member "block_number" |> to_int64 in
    let status_str = json |> member "status" |> to_string in
    let status = match status_str with
      | "Pending" -> `Pending
      | "Confirmed" -> `Confirmed
      | s when String.starts_with s "Failed:" -> `Failed (String.sub s 7 (String.length s - 7))
      | s -> `Failed ("Unknown status: " ^ s)
    in
    { account_id; transaction_hash; block_number; status }
  with
  | Yojson.Json_error msg -> raise (Valence_error ("JSON parse error: " ^ msg))
  | exn -> raise (Valence_error ("Parse error: " ^ Printexc.to_string exn))

let parse_library_approval_result json_str =
  try
    let json = Yojson.Safe.from_string json_str in
    let open Yojson.Safe.Util in
    let account_id = json |> member "account_id" |> to_string in
    let library_id = json |> member "library_id" |> to_string in
    let transaction_hash = json |> member "transaction_hash" |> to_string in
    let block_number = json |> member "block_number" |> to_int64 in
    let status_str = json |> member "status" |> to_string in
    let status = match status_str with
      | "Pending" -> `Pending
      | "Confirmed" -> `Confirmed
      | s when String.starts_with s "Failed:" -> `Failed (String.sub s 7 (String.length s - 7))
      | s -> `Failed ("Unknown status: " ^ s)
    in
    { account_id; library_id; transaction_hash; block_number; status }
  with
  | Yojson.Json_error msg -> raise (Valence_error ("JSON parse error: " ^ msg))
  | exn -> raise (Valence_error ("Parse error: " ^ Printexc.to_string exn))

let parse_transaction_result json_str =
  try
    let json = Yojson.Safe.from_string json_str in
    let open Yojson.Safe.Util in
    let transaction_hash = json |> member "transaction_hash" |> to_string in
    let block_number = json |> member "block_number" |> to_int64 in
    let gas_used = json |> member "gas_used" |> to_int64 in
    let status_str = json |> member "status" |> to_string in
    let status = match status_str with
      | "Pending" -> `Pending
      | "Confirmed" -> `Confirmed
      | s when String.starts_with s "Failed:" -> `Failed (String.sub s 7 (String.length s - 7))
      | s -> `Failed ("Unknown status: " ^ s)
    in
    let logs = json |> member "logs" in
    { transaction_hash; block_number; gas_used; status; logs }
  with
  | Yojson.Json_error msg -> raise (Valence_error ("JSON parse error: " ^ msg))
  | exn -> raise (Valence_error ("Parse error: " ^ Printexc.to_string exn))

(* Real Valence API functions *)
let create_account_factory ~chain_id ~owner_address ~initial_libraries () =
  Lwt.wrap (fun () ->
    try
      let libraries_array = Array.of_list initial_libraries in
      let result_json = valence_create_account_factory chain_id owner_address libraries_array in
      parse_account_creation_result result_json
    with
    | exn -> raise (Account_creation_failed (Printexc.to_string exn))
  )

let approve_library ~chain_id ~account_id ~library_id () =
  Lwt.wrap (fun () ->
    try
      let result_json = valence_approve_library chain_id account_id library_id in
      parse_library_approval_result result_json
    with
    | exn -> raise (Library_approval_failed (Printexc.to_string exn))
  )

let execute_transaction ~chain_id ~account_id ~transaction_config () =
  Lwt.wrap (fun () ->
    try
      let config_json = Yojson.Safe.to_string transaction_config in
      let result_json = valence_execute_transaction chain_id account_id config_json in
      parse_transaction_result result_json
    with
    | exn -> raise (Transaction_failed (Printexc.to_string exn))
  )

let query_account_state ~chain_id ~account_id () =
  Lwt.wrap (fun () ->
    try
      let result_json = valence_query_account_state chain_id account_id in
      Yojson.Safe.from_string result_json
    with
    | Yojson.Json_error msg -> raise (Valence_error ("JSON parse error: " ^ msg))
    | exn -> raise (Valence_error ("Query error: " ^ Printexc.to_string exn))
  )

let get_account_balance ~chain_id ~account_id ?token_address () =
  Lwt.wrap (fun () ->
    try
      let result = valence_get_account_balance chain_id account_id token_address in
      result
    with
    | exn -> raise (Valence_error ("Balance query error: " ^ Printexc.to_string exn))
  )

let submit_transaction ~chain_id ~from_account ~transaction_data () =
  Lwt.wrap (fun () ->
    try
      let result_json = valence_submit_transaction chain_id from_account transaction_data in
      result_json
    with
    | exn -> raise (Transaction_failed (Printexc.to_string exn))
  )

let wait_for_confirmation ~chain_id ~transaction_hash ~timeout_seconds () =
  Lwt.wrap (fun () ->
    try
      let result_json = valence_wait_for_confirmation chain_id transaction_hash timeout_seconds in
      let json = Yojson.Safe.from_string result_json in
      let open Yojson.Safe.Util in
      let confirmed = json |> member "confirmed" |> to_bool in
      let block_number = json |> member "block_number" |> to_int64_option in
      let error = json |> member "error" |> to_string_option in
      (confirmed, block_number, error)
    with
    | Yojson.Json_error msg -> raise (Valence_error ("JSON parse error: " ^ msg))
    | exn -> raise (Valence_error ("Confirmation error: " ^ Printexc.to_string exn))
  )

(* High-level convenience functions *)
let create_and_setup_account ~chain_id ~owner_address ~libraries () =
  let* creation_result = create_account_factory ~chain_id ~owner_address ~initial_libraries:[] () in
  match creation_result.status with
  | `Failed msg -> Lwt.fail (Account_creation_failed msg)
  | `Pending | `Confirmed ->
    (* Approve each library *)
    let approve_library_fn lib_id =
      approve_library ~chain_id ~account_id:creation_result.account_id ~library_id:lib_id ()
    in
    let* approval_results = Lwt_list.map_s approve_library_fn libraries in
    (* Check if all approvals succeeded *)
    let failed_approvals = List.filter (fun result ->
      match result.status with `Failed _ -> true | _ -> false
    ) approval_results in
    if List.length failed_approvals > 0 then
      let errors = List.map (fun result ->
        match result.status with `Failed msg -> msg | _ -> "Unknown error"
      ) failed_approvals in
      Lwt.fail (Library_approval_failed (String.concat "; " errors))
    else
      Lwt.return (creation_result, approval_results)

(* Token swap convenience function *)
let execute_token_swap ~chain_id ~account_id ~from_token ~to_token ~amount () =
  let swap_config = `Assoc [
    ("type", `String "token_swap");
    ("from_token", `String from_token);
    ("to_token", `String to_token);
    ("amount", `String amount);
  ] in
  execute_transaction ~chain_id ~account_id ~transaction_config:swap_config ()

(* Account factory pattern helpers *)
let get_factory_account_info ~chain_id ~account_id () =
  let* state = query_account_state ~chain_id ~account_id () in
  let open Yojson.Safe.Util in
  try
    let owner = state |> member "current_owner" |> to_string_option in
    let pending_owner = state |> member "pending_owner" |> to_string_option in
    let libraries = state |> member "libraries" |> to_list |> List.map to_string in
    let last_update_block = state |> member "last_update_block" |> to_int64 in
    Lwt.return (owner, pending_owner, libraries, last_update_block)
  with
  | exn -> Lwt.fail (Valence_error ("Failed to parse account info: " ^ Printexc.to_string exn))

(* Error recovery and retry logic *)
let with_retry ~max_attempts f =
  let rec attempt n =
    if n <= 0 then
      Lwt.fail (Valence_error "Max retry attempts exceeded")
    else
      Lwt.catch f (fun exn ->
        if n = 1 then Lwt.fail exn
        else
          let* () = Lwt_unix.sleep 1.0 in (* Wait 1 second between retries *)
          attempt (n - 1)
      )
  in
  attempt max_attempts

(* Batch operations *)
let batch_approve_libraries ~chain_id ~account_id ~library_ids () =
  let approve_fn lib_id = approve_library ~chain_id ~account_id ~library_id:lib_id () in
  Lwt_list.map_s approve_fn library_ids

(* Health check for Valence integration *)
let health_check ~chain_id () =
  Lwt.catch (fun () ->
    (* Try a simple query to see if the integration is working *)
    let* _state = query_account_state ~chain_id ~account_id:"test_account" () in
    Lwt.return `Healthy
  ) (fun _exn ->
    Lwt.return `Unhealthy
  ) 