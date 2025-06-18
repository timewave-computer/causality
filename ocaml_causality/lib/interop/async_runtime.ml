(* ------------ ASYNC RUNTIME FOR ACCOUNT FACTORY OPERATIONS ------------ *)
(* Purpose: Basic async runtime for coordinating account factory transactions *)

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
  submitted_at : float;
  mutable status : transaction_status;
}

(* Runtime configuration *)
type runtime_config = {
  max_concurrent_transactions : int;
  transaction_timeout_seconds : float;
  retry_attempts : int;
  retry_delay_seconds : float;
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
  transaction_timeout_seconds = 30.0;
  retry_attempts = 3;
  retry_delay_seconds = 1.0;
}

(* Create runtime state *)
let create_runtime ?(config = default_runtime_config) () = {
  config;
  active_transactions = Hashtbl.create 16;
  next_tx_id = 1;
}

(* Generate unique transaction ID *)
let generate_tx_id runtime =
  let id = Printf.sprintf "tx_%d_%f" runtime.next_tx_id (Unix.time ()) in
  runtime.next_tx_id <- runtime.next_tx_id + 1;
  id

(* Simplified transaction submission for testing *)
let mock_submit_transaction account operation _data =
  (* Mock implementation - would call actual Valence FFI *)
  if String.length account > 0 && String.length operation > 0 then
    Ok (Printf.sprintf "mock_tx_%s_%s" account operation)
  else
    Error "Invalid transaction parameters"

(* Submit transaction asynchronously *)
let submit_transaction_async runtime account operation data =
  let tx_id = generate_tx_id runtime in
  let handle = {
    tx_id;
    account;
    operation;
    submitted_at = Unix.time ();
    status = Pending;
  } in
  
  (* Check if we're at max concurrent transactions *)
  if Hashtbl.length runtime.active_transactions >= runtime.config.max_concurrent_transactions then
    Error "Maximum concurrent transactions reached"
  else begin
    (* Add to active transactions *)
    Hashtbl.add runtime.active_transactions tx_id handle;
    
    (* Submit transaction (simplified - would be async in real implementation) *)
    match mock_submit_transaction account operation data with
    | Ok _actual_tx_id ->
        handle.status <- Confirmed;
        Ok handle
    | Error msg ->
        handle.status <- Failed msg;
        Error msg
  end

(* Check transaction status *)
let check_transaction_status runtime tx_id =
  match Hashtbl.find_opt runtime.active_transactions tx_id with
  | Some handle -> Ok handle.status
  | None -> Error "Transaction not found"

(* Wait for transaction completion *)
let wait_for_transaction runtime tx_id =
  let rec wait_loop attempts =
    match Hashtbl.find_opt runtime.active_transactions tx_id with
    | None -> Error "Transaction not found"
    | Some handle ->
        let elapsed = Unix.time () -. handle.submitted_at in
        
        (* Check for timeout *)
        if elapsed > runtime.config.transaction_timeout_seconds then begin
          handle.status <- Timeout;
          Hashtbl.remove runtime.active_transactions tx_id;
          Error "Transaction timeout"
        end
        else begin
          match handle.status with
          | Pending when attempts > 0 ->
              (* Sleep and retry (simplified - would use proper async in real implementation) *)
              Unix.sleepf runtime.config.retry_delay_seconds;
              wait_loop (attempts - 1)
          | Pending ->
              Error "Transaction still pending after max retries"
          | Confirmed ->
              Hashtbl.remove runtime.active_transactions tx_id;
              Ok "Transaction confirmed"
          | Failed msg ->
              Hashtbl.remove runtime.active_transactions tx_id;
              Error msg
          | Timeout ->
              Error "Transaction timeout"
        end
  in
  wait_loop runtime.config.retry_attempts

(* Get all active transactions *)
let get_active_transactions runtime =
  Hashtbl.fold (fun _ handle acc -> handle :: acc) runtime.active_transactions []

(* Clean up completed transactions *)
let cleanup_completed_transactions runtime =
  let to_remove = ref [] in
  Hashtbl.iter (fun tx_id handle ->
    match handle.status with
    | Confirmed | Failed _ | Timeout -> to_remove := tx_id :: !to_remove
    | Pending -> ()
  ) runtime.active_transactions;
  
  List.iter (Hashtbl.remove runtime.active_transactions) !to_remove;
  List.length !to_remove

(* Batch transaction submission *)
let submit_batch_transactions runtime transactions =
  let results = ref [] in
  let errors = ref [] in
  
  List.iter (fun (account, operation, data) ->
    match submit_transaction_async runtime account operation data with
    | Ok handle -> results := handle :: !results
    | Error msg -> errors := (account, operation, msg) :: !errors
  ) transactions;
  
  (!results, !errors)

(* Mock account factory creation *)
let mock_create_account_factory owner _permissions =
  if String.length owner > 0 then
    Ok (Printf.sprintf "account_factory_%s" owner)
  else
    Error "Invalid owner"

(* Account factory specific operations *)
module AccountFactory = struct
  
  (* Create account factory asynchronously *)
  let create_account_async runtime owner permissions =
    match mock_create_account_factory owner permissions with
    | Ok account -> 
        let tx_id = generate_tx_id runtime in
        let handle = {
          tx_id;
          account = owner;
          operation = "create_account_factory";
          submitted_at = Unix.time ();
          status = Confirmed;
        } in
        Hashtbl.add runtime.active_transactions tx_id handle;
        Ok (account, handle)
    | Error msg -> Error msg
  
  (* Approve library asynchronously *)
  let approve_library_async runtime account library permissions =
    submit_transaction_async runtime account "approve_library" 
      (Printf.sprintf "%s:%s" library (String.concat "," permissions))
  
  (* Submit account factory transaction asynchronously *)
  let submit_factory_transaction_async runtime account operation data =
    submit_transaction_async runtime account operation data
  
end

(* Runtime statistics *)
type runtime_stats = {
  total_transactions : int;
  successful_transactions : int;
  failed_transactions : int;
  active_transactions : int;
  average_completion_time : float;
}

(* Get runtime statistics *)
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
    average_completion_time = 2.5; (* Would calculate this in real implementation *)
  }

(* Shutdown runtime *)
let shutdown_runtime (runtime : runtime_state) =
  (* Wait for all active transactions to complete or timeout *)
  let active_tx_ids = Hashtbl.fold (fun tx_id _ acc -> tx_id :: acc) runtime.active_transactions [] in
  
  List.iter (fun tx_id ->
    match wait_for_transaction runtime tx_id with
    | Ok _ -> Printf.printf "Transaction %s completed\n" tx_id
    | Error msg -> Printf.printf "Transaction %s failed: %s\n" tx_id msg
  ) active_tx_ids;
  
  Hashtbl.clear runtime.active_transactions;
  Printf.printf "Runtime shutdown complete\n" 