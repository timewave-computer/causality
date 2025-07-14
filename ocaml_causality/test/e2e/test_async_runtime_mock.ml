(* ------------ MOCK ASYNC RUNTIME FOR TESTING ------------ *)
(* Purpose: Mock implementation of async runtime for testing *)

open Test_valence_ffi_mock

(* Transaction status types *)
type transaction_status = 
  | Pending
  | Confirmed
  | Failed of string
  | Timeout

(* Transaction handle for tracking *)
type transaction_handle = {
  tx_id : string;
  account : string;
  operation : string;
  submitted_at : int64;
  mutable status : transaction_status;
}

(* Runtime configuration *)
type runtime_config = {
  max_concurrent_transactions : int;
  transaction_timeout_seconds : int;
  retry_attempts : int;
  retry_delay_seconds : int;
}

(* Runtime state *)
type runtime_state = {
  config : runtime_config;
  active_transactions : (string, transaction_handle) Hashtbl.t;
  mutable next_tx_id : int;
}

(* Create default runtime configuration *)
let default_runtime_config = {
  max_concurrent_transactions = 10;
  transaction_timeout_seconds = 30;
  retry_attempts = 3;
  retry_delay_seconds = 1;
}

(* Create runtime state *)
let create_runtime ?(config = default_runtime_config) () = {
  config;
  active_transactions = Hashtbl.create 16;
  next_tx_id = 1;
}

(* Generate unique transaction ID *)
let generate_tx_id runtime =
  let id = Printf.sprintf "tx_%d_%Ld" runtime.next_tx_id (Int64.of_float (Unix.time () *. 1000.0)) in
  runtime.next_tx_id <- runtime.next_tx_id + 1;
  id

(* Submit transaction asynchronously *)
let submit_transaction_async runtime account operation data =
  let tx_id = generate_tx_id runtime in
  let handle = {
    tx_id;
    account;
    operation;
    submitted_at = Int64.of_float (Unix.time () *. 1000.0);
    status = Pending;
  } in
  
  (* Check if we're at max concurrent transactions *)
  if Hashtbl.length runtime.active_transactions >= runtime.config.max_concurrent_transactions then
    Error "Maximum concurrent transactions reached"
  else begin
    (* Add to active transactions *)
    Hashtbl.add runtime.active_transactions tx_id handle;
    
    (* Submit transaction (mock implementation) *)
    let config = make_transaction_config ~account ~operation ~data () in
    match submit_transaction config with
    | Ok _actual_tx_id ->
        handle.status <- Confirmed;
        Ok handle
    | Error msg ->
        handle.status <- Failed msg;
        Error msg
  end

(* Get runtime statistics *)
type runtime_stats = {
  total_transactions : int;
  successful_transactions : int;
  failed_transactions : int;
  active_transactions : int;
  average_completion_time : int;
}

let get_runtime_stats (runtime : runtime_state) =
  let active = Hashtbl.length runtime.active_transactions in
  let total = runtime.next_tx_id - 1 in
  
  (* Calculate success/failure rates (simplified) *)
  let successful = max 0 (total - active) in
  let failed = 0 in (* Would track this in real implementation *)
  
  {
    total_transactions = total;
    successful_transactions = successful;
    failed_transactions = failed;
    active_transactions = active;
    average_completion_time = 250; (* Would calculate this in real implementation *)
  }

(* Account factory specific operations *)
module AccountFactory = struct
  
  (* Create account factory asynchronously *)
  let create_account_async runtime owner permissions =
    match safe_create_account_factory (make_account_factory_config ~owner ~permissions ()) with
    | Ok account -> 
        let tx_id = generate_tx_id runtime in
        let handle = {
          tx_id;
          account = owner;
          operation = "create_account_factory";
          submitted_at = Int64.of_float (Unix.time () *. 1000.0);
          status = Confirmed;
        } in
        Hashtbl.add runtime.active_transactions tx_id handle;
        Ok (account, handle)
    | Error msg -> Error msg
  
end 