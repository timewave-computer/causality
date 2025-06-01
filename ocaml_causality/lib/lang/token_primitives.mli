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

(** Create a new token resource *)
val create_token : 
  name:string ->
  symbol:string ->
  decimals:int ->
  total_supply:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  token_config * resource

(** Create token balance resource *)
val create_token_balance :
  account_id:Identifiers.entity_id ->
  token_id:Identifiers.entity_id ->
  initial_balance:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  token_balance * resource

(** Create token transfer effect *)
val create_token_transfer_effect :
  token_id:Identifiers.entity_id ->
  from_account:Identifiers.entity_id ->
  to_account:Identifiers.entity_id ->
  amount:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource

(** Create lock tokens effect *)
val create_lock_tokens_effect :
  token_id:Identifiers.entity_id ->
  account_id:Identifiers.entity_id ->
  amount:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource

(** Create unlock tokens effect *)
val create_unlock_tokens_effect :
  token_id:Identifiers.entity_id ->
  account_id:Identifiers.entity_id ->
  amount:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource

(** Create mint tokens effect *)
val create_mint_tokens_effect :
  token_id:Identifiers.entity_id ->
  account_id:Identifiers.entity_id ->
  amount:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource

(** Create burn tokens effect *)
val create_burn_tokens_effect :
  token_id:Identifiers.entity_id ->
  account_id:Identifiers.entity_id ->
  amount:int64 ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource 