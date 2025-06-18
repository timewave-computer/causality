(* ------------ GAS ESTIMATION AND FEE MANAGEMENT ------------ *)
(* Purpose: Basic gas estimation and fee management for account factory accounts *)

(* Gas estimation types *)
type gas_estimate = {
  gas_limit : int;
  gas_price : int;
  total_cost : int;
}

(* Fee configuration *)
type fee_config = {
  base_fee : int;
  priority_fee : int;
  max_fee_per_gas : int;
  max_priority_fee_per_gas : int;
}

(* Account factory operation types for gas estimation *)
type account_factory_operation = 
  | CreateAccount
  | ApproveLibrary
  | SubmitTransaction of string (* operation type *)
  | BatchOperations of account_factory_operation list

(* Default gas estimates for account factory operations *)
let rec default_gas_estimates = function
  | CreateAccount -> { gas_limit = 200000; gas_price = 20; total_cost = 4000000 }
  | ApproveLibrary -> { gas_limit = 100000; gas_price = 20; total_cost = 2000000 }
  | SubmitTransaction "swap" -> { gas_limit = 150000; gas_price = 20; total_cost = 3000000 }
  | SubmitTransaction "transfer" -> { gas_limit = 80000; gas_price = 20; total_cost = 1600000 }
  | SubmitTransaction _ -> { gas_limit = 120000; gas_price = 20; total_cost = 2400000 }
  | BatchOperations ops -> 
      let total_gas = List.fold_left (fun acc op -> 
        let estimate = default_gas_estimates op in
        acc + estimate.gas_limit
      ) 0 ops in
      { gas_limit = total_gas; gas_price = 20; total_cost = total_gas * 20 }

(* Create default fee configuration *)
let default_fee_config = {
  base_fee = 20;
  priority_fee = 2;
  max_fee_per_gas = 50;
  max_priority_fee_per_gas = 10;
}

(* Estimate gas for account factory operation *)
let estimate_gas ?(config = default_fee_config) operation =
  let base_estimate = default_gas_estimates operation in
  {
    gas_limit = base_estimate.gas_limit;
    gas_price = max config.base_fee base_estimate.gas_price;
    total_cost = base_estimate.gas_limit * (max config.base_fee base_estimate.gas_price);
  }

(* Calculate total fee for operation *)
let calculate_fee ?(config = default_fee_config) operation =
  let estimate = estimate_gas ~config operation in
  let base_cost = estimate.gas_limit * config.base_fee in
  let priority_cost = estimate.gas_limit * config.priority_fee in
  base_cost + priority_cost

(* Validate fee configuration *)
let validate_fee_config config =
  if config.base_fee <= 0 then
    Error "Base fee must be positive"
  else if config.priority_fee < 0 then
    Error "Priority fee cannot be negative"
  else if config.max_fee_per_gas < config.base_fee then
    Error "Max fee per gas must be at least base fee"
  else if config.max_priority_fee_per_gas < config.priority_fee then
    Error "Max priority fee per gas must be at least priority fee"
  else
    Ok config

(* Optimize gas price based on network conditions *)
let optimize_gas_price ?(network_congestion = 0.5) base_price =
  let congestion_multiplier = 1.0 +. (network_congestion *. 0.5) in
  int_of_float (float_of_int base_price *. congestion_multiplier)

(* Account factory specific gas estimation *)
module AccountFactory = struct
  
  (* Estimate gas for account creation *)
  let estimate_create_account_gas ?(permissions_count = 3) () =
    let base_gas = 150000 in
    let permission_gas = permissions_count * 10000 in
    { gas_limit = base_gas + permission_gas; gas_price = 20; total_cost = (base_gas + permission_gas) * 20 }
  
  (* Estimate gas for library approval *)
  let estimate_approve_library_gas ?(library_complexity = 1.0) () =
    let base_gas = 80000 in
    let complexity_gas = int_of_float (float_of_int base_gas *. library_complexity) in
    { gas_limit = base_gas + complexity_gas; gas_price = 20; total_cost = (base_gas + complexity_gas) * 20 }
  
  (* Estimate gas for transaction submission *)
  let estimate_transaction_gas ?(data_size = 0) operation_type =
    let base_gas = match operation_type with
      | "swap" -> 120000
      | "transfer" -> 60000
      | "approve" -> 50000
      | _ -> 100000
    in
    let data_gas = data_size * 16 in (* 16 gas per byte of data *)
    { gas_limit = base_gas + data_gas; gas_price = 20; total_cost = (base_gas + data_gas) * 20 }
  
  (* Calculate batch operation gas *)
  let estimate_batch_gas operations =
    let total_gas = List.fold_left (fun acc (op_type, data_size) ->
      let estimate = estimate_transaction_gas ~data_size op_type in
      acc + estimate.gas_limit
    ) 0 operations in
    let batch_overhead = 20000 in (* Additional gas for batch processing *)
    { gas_limit = total_gas + batch_overhead; gas_price = 20; total_cost = (total_gas + batch_overhead) * 20 }
  
end

(* Fee management utilities *)
module FeeManager = struct
  
  (* Track fee history for optimization *)
  type fee_history = {
    operation : string;
    estimated_fee : int;
    actual_fee : int;
    timestamp : float;
  }
  
  let fee_history = ref []
  
  (* Record fee usage *)
  let record_fee_usage operation estimated actual =
    let entry = {
      operation;
      estimated_fee = estimated;
      actual_fee = actual;
      timestamp = Unix.time ();
    } in
    fee_history := entry :: !fee_history
  
  (* Get average fee for operation type *)
  let get_average_fee operation_type =
    let relevant_fees = List.filter (fun entry -> entry.operation = operation_type) !fee_history in
    if List.length relevant_fees = 0 then
      None
    else
      let total = List.fold_left (fun acc entry -> acc + entry.actual_fee) 0 relevant_fees in
      Some (total / List.length relevant_fees)
  
  (* Suggest optimal fee based on history *)
  let suggest_optimal_fee operation_type =
    match get_average_fee operation_type with
    | Some avg -> int_of_float (float_of_int avg *. 1.1) (* 10% buffer *)
    | None -> calculate_fee (SubmitTransaction operation_type)
  
  (* Clear old fee history (keep last 100 entries) *)
  let cleanup_fee_history () =
    let rec take n lst =
      match n, lst with
      | 0, _ | _, [] -> []
      | n, x :: xs -> x :: take (n - 1) xs
    in
    let sorted = List.sort (fun a b -> compare b.timestamp a.timestamp) !fee_history in
    fee_history := take 100 sorted
  
end

(* Gas price oracle simulation *)
module GasPriceOracle = struct
  
  (* Simulated network conditions *)
  type network_state = {
    congestion_level : float; (* 0.0 to 1.0 *)
    base_fee : int;
    suggested_priority_fee : int;
  }
  
  (* Get current network state (mock implementation) *)
  let get_network_state () =
    let current_time = Unix.time () in
    let congestion = 0.3 +. (0.4 *. sin (current_time /. 3600.0)) in (* Simulate daily congestion cycle *)
    {
      congestion_level = max 0.0 (min 1.0 congestion);
      base_fee = 15 + int_of_float (congestion *. 20.0);
      suggested_priority_fee = 1 + int_of_float (congestion *. 5.0);
    }
  
  (* Get recommended gas price *)
  let get_recommended_gas_price () =
    let state = get_network_state () in
    state.base_fee + state.suggested_priority_fee
  
  (* Get fast gas price (for urgent transactions) *)
  let get_fast_gas_price () =
    let recommended = get_recommended_gas_price () in
    int_of_float (float_of_int recommended *. 1.5)
  
  (* Get economy gas price (for non-urgent transactions) *)
  let get_economy_gas_price () =
    let recommended = get_recommended_gas_price () in
    int_of_float (float_of_int recommended *. 0.8)
  
end 