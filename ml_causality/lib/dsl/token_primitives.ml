(* Purpose: Token primitives for the OCaml DSL *)

open Ml_causality_lib_types.Types

(*-----------------------------------------------------------------------------
 * Token Types
 *-----------------------------------------------------------------------------*)

(** Token configuration *)
type token_config = {
  token_id: entity_id;
  name: string;
  symbol: string;
  decimals: int;
  total_supply: int64;
  domain_id: domain_id;
}

(** Token balance for an account *)
type token_balance = {
  account_id: entity_id;
  token_id: entity_id;
  balance: int64;
  locked_balance: int64;
}

(** Token transfer metadata *)
type token_transfer = {
  transfer_id: entity_id;
  token_id: entity_id;
  from_account: entity_id;
  to_account: entity_id;
  amount: int64;
  timestamp: int64;
}

(*-----------------------------------------------------------------------------
 * Token Creation Functions
 *-----------------------------------------------------------------------------*)

(** Create a new token resource *)
let create_token ~name ~symbol ~decimals ~total_supply ~domain_id () =
  let token_id = Bytes.of_string (Printf.sprintf "token_%s_%s" name symbol) in
  let token_config = {
    token_id;
    name;
    symbol;
    decimals;
    total_supply;
    domain_id;
  } in
  let token_resource = {
    id = token_id;
    name = Printf.sprintf "%s (%s)" name symbol;
    domain_id;
    resource_type = "token";
    quantity = total_supply;
    timestamp = 0L;
  } in
  (token_config, token_resource)

(** Create token balance resource *)
let create_token_balance ~account_id ~token_id ~initial_balance ~domain_id () =
  let balance_id = Bytes.of_string (Printf.sprintf "balance_%s_%s" 
    (Bytes.to_string account_id) (Bytes.to_string token_id)) in
  let balance_config = {
    account_id;
    token_id;
    balance = initial_balance;
    locked_balance = 0L;
  } in
  let balance_resource = {
    id = balance_id;
    name = "Token Balance";
    domain_id;
    resource_type = "token_balance";
    quantity = initial_balance;
    timestamp = 0L;
  } in
  (balance_config, balance_resource)

(*-----------------------------------------------------------------------------
 * Token Operations as Effects
 *-----------------------------------------------------------------------------*)

(** Create token transfer effect *)
let create_token_transfer_effect ~token_id ~from_account ~to_account ~amount ~domain_id () =
  let transfer_id = Bytes.of_string (Printf.sprintf "transfer_%s_%s_%Ld" 
    (Bytes.to_string from_account) (Bytes.to_string to_account) amount) in

  {
    id = transfer_id;
    name = "TokenTransfer";
    domain_id;
    effect_type = "token_transfer";
    inputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    outputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    expression = Some (Bytes.of_string "token_transfer_logic");
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  }

(** Create lock tokens effect *)
let create_lock_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let lock_id = Bytes.of_string (Printf.sprintf "lock_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount) in

  {
    id = lock_id;
    name = "LockTokens";
    domain_id;
    effect_type = "token_lock";
    inputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    outputs = [{
      resource_type = "locked_tokens";
      quantity = amount;
      domain_id;
    }];
    expression = Some (Bytes.of_string "token_lock_logic");
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  }

(** Create unlock tokens effect *)
let create_unlock_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let unlock_id = Bytes.of_string (Printf.sprintf "unlock_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount) in

  {
    id = unlock_id;
    name = "UnlockTokens";
    domain_id;
    effect_type = "token_unlock";
    inputs = [{
      resource_type = "locked_tokens";
      quantity = amount;
      domain_id;
    }];
    outputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    expression = Some (Bytes.of_string "token_unlock_logic");
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  }

(** Create mint tokens effect *)
let create_mint_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let mint_id = Bytes.of_string (Printf.sprintf "mint_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount) in

  {
    id = mint_id;
    name = "MintTokens";
    domain_id;
    effect_type = "token_mint";
    inputs = [];
    outputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    expression = Some (Bytes.of_string "token_mint_logic");
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  }

(** Create burn tokens effect *)
let create_burn_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let burn_id = Bytes.of_string (Printf.sprintf "burn_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount) in

  {
    id = burn_id;
    name = "BurnTokens";
    domain_id;
    effect_type = "token_burn";
    inputs = [{
      resource_type = "token_balance";
      quantity = amount;
      domain_id;
    }];
    outputs = [];
    expression = Some (Bytes.of_string "token_burn_logic");
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  }

(*-----------------------------------------------------------------------------
 * Token Utility Functions
 *-----------------------------------------------------------------------------*)

(** Validate token transfer amount *)
let validate_transfer_amount ~balance ~amount =
  amount > 0L && amount <= balance

(** Calculate token amount with decimals *)
let amount_with_decimals ~amount ~decimals =
  Int64.mul amount (Int64.of_int (int_of_float (10.0 ** float_of_int decimals)))

(** Format token amount for display *)
let format_token_amount ~amount ~decimals ~symbol =
  let divisor = Int64.of_int (int_of_float (10.0 ** float_of_int decimals)) in
  let whole_part = Int64.div amount divisor in
  let fractional_part = Int64.rem amount divisor in
  Printf.sprintf "%Ld.%0*Ld %s" whole_part decimals fractional_part symbol 