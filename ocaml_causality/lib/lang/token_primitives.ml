(* Purpose: Token primitives for the OCaml DSL *)

open Ocaml_causality_core

(** Token configuration *)
type token_config = {
  token_id: Identifiers.entity_id;
  name: string;
  symbol: string;
  decimals: int;
  total_supply: int64;
  domain_id: Identifiers.domain_id;
}

(** Token balance for an account *)
type token_balance = {
  account_id: Identifiers.entity_id;
  token_id: Identifiers.entity_id;
  balance: int64;
  locked_balance: int64;
}

(** Token transfer metadata *)
type token_transfer = {
  transfer_id: Identifiers.entity_id;
  token_id: Identifiers.entity_id;
  from_account: Identifiers.entity_id;
  to_account: Identifiers.entity_id;
  amount: int64;
  timestamp: int64;
}

(*-----------------------------------------------------------------------------
 * Token Creation Functions
 *-----------------------------------------------------------------------------*)

(** Create a new token resource *)
let create_token ~name ~symbol ~decimals ~total_supply ~domain_id () =
  let token_id = Printf.sprintf "token_%s_%s" name symbol |> Bytes.of_string in
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
  let balance_id = Printf.sprintf "balance_%s_%s" 
    (Bytes.to_string account_id) (Bytes.to_string token_id) |> Bytes.of_string in
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

(** Create simplified effect for token operations *)
let create_effect_simple ~effect_id ~name ~domain_id ~effect_type ~_inputs ~_outputs () = {
  id = effect_id;
  name;
  domain_id;
  resource_type = effect_type; (* Using resource_type field since no effect type exists *)
  quantity = 0L; (* Placeholder *)
  timestamp = 0L;
}

(*-----------------------------------------------------------------------------
 * Token Operations as Effects
 *-----------------------------------------------------------------------------*)

(** Create token transfer effect *)
let create_token_transfer_effect ~token_id:_ ~from_account ~to_account ~amount ~domain_id () =
  let transfer_id = Printf.sprintf "transfer_%s_%s_%Ld" 
    (Bytes.to_string from_account) (Bytes.to_string to_account) amount |> Bytes.of_string in
  create_effect_simple 
    ~effect_id:transfer_id
    ~name:"TokenTransfer"
    ~domain_id
    ~effect_type:"token_transfer"
    ~_inputs:[]
    ~_outputs:[]
    ()

(** Create lock tokens effect *)
let create_lock_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let lock_id = Printf.sprintf "lock_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount |> Bytes.of_string in
  create_effect_simple
    ~effect_id:lock_id
    ~name:"LockTokens"
    ~domain_id
    ~effect_type:"token_lock"
    ~_inputs:[]
    ~_outputs:[]
    ()

(** Create unlock tokens effect *)
let create_unlock_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let unlock_id = Printf.sprintf "unlock_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount |> Bytes.of_string in
  create_effect_simple
    ~effect_id:unlock_id
    ~name:"UnlockTokens"
    ~domain_id
    ~effect_type:"token_unlock"
    ~_inputs:[]
    ~_outputs:[]
    ()

(** Create mint tokens effect *)
let create_mint_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let mint_id = Printf.sprintf "mint_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount |> Bytes.of_string in
  create_effect_simple
    ~effect_id:mint_id
    ~name:"MintTokens"
    ~domain_id
    ~effect_type:"token_mint"
    ~_inputs:[]
    ~_outputs:[]
    ()

(** Create burn tokens effect *)
let create_burn_tokens_effect ~token_id ~account_id ~amount ~domain_id () =
  let burn_id = Printf.sprintf "burn_%s_%s_%Ld" 
    (Bytes.to_string token_id) (Bytes.to_string account_id) amount |> Bytes.of_string in
  create_effect_simple
    ~effect_id:burn_id
    ~name:"BurnTokens"
    ~domain_id
    ~effect_type:"token_burn"
    ~_inputs:[]
    ~_outputs:[]
    () 