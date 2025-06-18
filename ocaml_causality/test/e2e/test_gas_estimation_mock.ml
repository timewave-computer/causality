(* ------------ MOCK GAS ESTIMATION FOR TESTING ------------ *)
(* Purpose: Mock implementation of gas estimation for testing *)

(* Gas estimation types *)
type gas_estimate = {
  gas_limit : int;
  gas_price : int;
  total_cost : int;
}

(* Account factory operation types for gas estimation *)
type account_factory_operation = 
  | CreateAccount
  | ApproveLibrary
  | SubmitTransaction of string

(* Default gas estimates for account factory operations *)
let estimate_gas = function
  | CreateAccount -> { gas_limit = 200000; gas_price = 20; total_cost = 4000000 }
  | ApproveLibrary -> { gas_limit = 100000; gas_price = 20; total_cost = 2000000 }
  | SubmitTransaction "swap" -> { gas_limit = 150000; gas_price = 20; total_cost = 3000000 }
  | SubmitTransaction "transfer" -> { gas_limit = 80000; gas_price = 20; total_cost = 1600000 }
  | SubmitTransaction _ -> { gas_limit = 120000; gas_price = 20; total_cost = 2400000 }

(* Account factory specific gas estimation *)
module AccountFactory = struct
  
  (* Estimate gas for account creation *)
  let estimate_create_account_gas ?(permissions_count = 3) () =
    let base_gas = 150000 in
    let permission_gas = permissions_count * 10000 in
    { gas_limit = base_gas + permission_gas; gas_price = 20; total_cost = (base_gas + permission_gas) * 20 }
  
end

(* Gas price oracle simulation *)
module GasPriceOracle = struct
  
  (* Get recommended gas price *)
  let get_recommended_gas_price () = 25
  
  (* Get fast gas price (for urgent transactions) *)
  let get_fast_gas_price () = 35
  
  (* Get economy gas price (for non-urgent transactions) *)
  let get_economy_gas_price () = 18
  
end 