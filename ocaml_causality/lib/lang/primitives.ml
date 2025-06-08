(* ------------ DOMAIN-SPECIFIC PRIMITIVES ------------ *)
(* Purpose: Domain-specific primitives for tokens, bridges, etc. *)

open Expr
open Value

(* ------------ TOKEN PRIMITIVES ------------ *)

(* Token operation primitives *)
let mint_token =
  Expr.lambda
    [
      LispValue.symbol "token_type"
    ; LispValue.symbol "amount"
    ; LispValue.symbol "recipient"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_mint"))
           [
             Expr.const (LispValue.symbol "token_type")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "create_token"))
           [
             Expr.const (LispValue.symbol "token_type")
           ; Expr.const (LispValue.symbol "amount")
           ; Expr.const (LispValue.symbol "recipient")
           ]
       ])

let burn_token =
  Expr.lambda
    [ LispValue.symbol "token_id"; LispValue.symbol "amount" ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_burn"))
           [
             Expr.const (LispValue.symbol "token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.consume (Bytes.of_string "token_resource")
       ])

let transfer_token =
  Expr.lambda
    [
      LispValue.symbol "from"
    ; LispValue.symbol "to"
    ; LispValue.symbol "amount"
    ; LispValue.symbol "token_type"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_balance"))
           [
             Expr.const (LispValue.symbol "from")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "debit_account"))
           [
             Expr.const (LispValue.symbol "from")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "credit_account"))
           [
             Expr.const (LispValue.symbol "to")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ])

let get_balance =
  Expr.lambda
    [ LispValue.symbol "account"; LispValue.symbol "token_type" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "query_balance"))
       [
         Expr.const (LispValue.symbol "account")
       ; Expr.const (LispValue.symbol "token_type")
       ])

let approve_token =
  Expr.lambda
    [
      LispValue.symbol "owner"
    ; LispValue.symbol "spender"
    ; LispValue.symbol "amount"
    ]
    (Expr.apply
       (Expr.const (LispValue.symbol "set_allowance"))
       [
         Expr.const (LispValue.symbol "owner")
       ; Expr.const (LispValue.symbol "spender")
       ; Expr.const (LispValue.symbol "amount")
       ])

(* ------------ BRIDGE PRIMITIVES ------------ *)

(* Cross-chain bridge primitives *)
let lock_tokens =
  Expr.lambda
    [
      LispValue.symbol "source_chain"
    ; LispValue.symbol "token_id"
    ; LispValue.symbol "amount"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_lock"))
           [
             Expr.const (LispValue.symbol "source_chain")
           ; Expr.const (LispValue.symbol "token_id")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "escrow_tokens"))
           [
             Expr.const (LispValue.symbol "token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_lock_event"))
           [
             Expr.const (LispValue.symbol "source_chain")
           ; Expr.const (LispValue.symbol "token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ])

let mint_wrapped =
  Expr.lambda
    [
      LispValue.symbol "dest_chain"
    ; LispValue.symbol "amount"
    ; LispValue.symbol "proof"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "verify_lock_proof"))
           [ Expr.const (LispValue.symbol "proof") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "mint_wrapped_token"))
           [
             Expr.const (LispValue.symbol "dest_chain")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_mint_event"))
           [
             Expr.const (LispValue.symbol "dest_chain")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ])

let burn_wrapped =
  Expr.lambda
    [ LispValue.symbol "wrapped_token_id"; LispValue.symbol "amount" ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_wrapped_token"))
           [ Expr.const (LispValue.symbol "wrapped_token_id") ]
       ; Expr.consume (Bytes.of_string "wrapped_token_resource")
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_burn_event"))
           [
             Expr.const (LispValue.symbol "wrapped_token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ])

let unlock_tokens =
  Expr.lambda
    [
      LispValue.symbol "source_chain"
    ; LispValue.symbol "token_id"
    ; LispValue.symbol "amount"
    ; LispValue.symbol "burn_proof"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "verify_burn_proof"))
           [ Expr.const (LispValue.symbol "burn_proof") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "release_escrowed_tokens"))
           [
             Expr.const (LispValue.symbol "token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_unlock_event"))
           [
             Expr.const (LispValue.symbol "source_chain")
           ; Expr.const (LispValue.symbol "token_id")
           ; Expr.const (LispValue.symbol "amount")
           ]
       ])

let verify_bridge_proof =
  Expr.lambda
    [ LispValue.symbol "proof"; LispValue.symbol "proof_type" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "zk_verify"))
       [
         Expr.const (LispValue.symbol "proof")
       ; Expr.const (LispValue.symbol "proof_type")
       ])

(* ------------ DOMAIN PRIMITIVES ------------ *)

(* DeFi primitives *)
let add_liquidity =
  Expr.lambda
    [
      LispValue.symbol "pool"
    ; LispValue.symbol "token_a_amount"
    ; LispValue.symbol "token_b_amount"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_liquidity"))
           [
             Expr.const (LispValue.symbol "token_a_amount")
           ; Expr.const (LispValue.symbol "token_b_amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "deposit_to_pool"))
           [
             Expr.const (LispValue.symbol "pool")
           ; Expr.const (LispValue.symbol "token_a_amount")
           ; Expr.const (LispValue.symbol "token_b_amount")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "mint_lp_tokens"))
           [ Expr.const (LispValue.symbol "pool") ]
       ])

let remove_liquidity =
  Expr.lambda
    [ LispValue.symbol "pool"; LispValue.symbol "lp_token_amount" ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_lp_tokens"))
           [ Expr.const (LispValue.symbol "lp_token_amount") ]
       ; Expr.consume (Bytes.of_string "lp_token_resource")
       ; Expr.apply
           (Expr.const (LispValue.symbol "withdraw_from_pool"))
           [
             Expr.const (LispValue.symbol "pool")
           ; Expr.const (LispValue.symbol "lp_token_amount")
           ]
       ])

let swap_tokens =
  Expr.lambda
    [
      LispValue.symbol "pool"
    ; LispValue.symbol "token_in"
    ; LispValue.symbol "amount_in"
    ; LispValue.symbol "min_amount_out"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "calculate_swap_amount"))
           [
             Expr.const (LispValue.symbol "pool")
           ; Expr.const (LispValue.symbol "amount_in")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "validate_slippage"))
           [ Expr.const (LispValue.symbol "min_amount_out") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "execute_swap"))
           [
             Expr.const (LispValue.symbol "pool")
           ; Expr.const (LispValue.symbol "token_in")
           ; Expr.const (LispValue.symbol "amount_in")
           ]
       ])

(* Oracle primitives *)
let get_price =
  Expr.lambda
    [ LispValue.symbol "asset"; LispValue.symbol "currency" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "oracle_query"))
       [
         Expr.const (LispValue.symbol "asset")
       ; Expr.const (LispValue.symbol "currency")
       ])

let update_price =
  Expr.lambda
    [
      LispValue.symbol "asset"
    ; LispValue.symbol "price"
    ; LispValue.symbol "signature"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "verify_oracle_signature"))
           [ Expr.const (LispValue.symbol "signature") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "update_price_feed"))
           [
             Expr.const (LispValue.symbol "asset")
           ; Expr.const (LispValue.symbol "price")
           ]
       ])

(* Governance primitives *)
let create_proposal =
  Expr.lambda
    [
      LispValue.symbol "title"
    ; LispValue.symbol "description"
    ; LispValue.symbol "actions"
    ]
    (Expr.sequence
       [
         Expr.apply (Expr.const (LispValue.symbol "validate_proposer")) []
       ; Expr.apply
           (Expr.const (LispValue.symbol "create_governance_proposal"))
           [
             Expr.const (LispValue.symbol "title")
           ; Expr.const (LispValue.symbol "description")
           ; Expr.const (LispValue.symbol "actions")
           ]
       ])

let vote_on_proposal =
  Expr.lambda
    [
      LispValue.symbol "proposal_id"
    ; LispValue.symbol "vote"
    ; LispValue.symbol "voting_power"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_voter"))
           [ Expr.const (LispValue.symbol "voting_power") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "cast_vote"))
           [
             Expr.const (LispValue.symbol "proposal_id")
           ; Expr.const (LispValue.symbol "vote")
           ; Expr.const (LispValue.symbol "voting_power")
           ]
       ])

let execute_proposal =
  Expr.lambda
    [ LispValue.symbol "proposal_id" ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_proposal_passed"))
           [ Expr.const (LispValue.symbol "proposal_id") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "execute_proposal_actions"))
           [ Expr.const (LispValue.symbol "proposal_id") ]
       ])

(* ------------ PRIMITIVE UTILITIES ------------ *)

(* Primitive validation and composition functions *)
let validate_primitive_args primitive args =
  try
    let _ = Expr.to_string primitive in
    let expected_params =
      match primitive with Lambda (params, _) -> List.length params | _ -> 0
    in
    List.length args = expected_params
  with _ -> false

let compose_primitives primitives =
  match primitives with
  | [] -> Expr.const_unit
  | [ single ] -> single
  | _ -> Expr.sequence primitives

let create_primitive_call name args =
  Expr.apply (Expr.const (LispValue.symbol name)) args

let validate_domain_primitive domain primitive =
  let domain_prefixes =
    [
      ("token", [ "mint_"; "burn_"; "transfer_"; "approve_"; "balance_" ])
    ; ( "bridge"
      , [ "lock_"; "unlock_"; "mint_wrapped"; "burn_wrapped"; "verify_" ] )
    ; ("defi", [ "add_liquidity"; "remove_liquidity"; "swap_"; "pool_" ])
    ; ("oracle", [ "get_price"; "update_price"; "oracle_" ])
    ; ("governance", [ "create_proposal"; "vote_"; "execute_proposal" ])
    ]
  in
  match List.assoc_opt domain domain_prefixes with
  | Some prefixes ->
      List.exists (fun prefix -> String.starts_with ~prefix primitive) prefixes
  | None -> false

(* Primitive registry *)
module PrimitiveRegistry = struct
  type t = {
      token_primitives : (string * Expr.t) list
    ; bridge_primitives : (string * Expr.t) list
    ; defi_primitives : (string * Expr.t) list
    ; oracle_primitives : (string * Expr.t) list
    ; governance_primitives : (string * Expr.t) list
  }

  let create () =
    {
      token_primitives = []
    ; bridge_primitives = []
    ; defi_primitives = []
    ; oracle_primitives = []
    ; governance_primitives = []
    }

  let register_token registry name primitive =
    {
      registry with
      token_primitives = (name, primitive) :: registry.token_primitives
    }

  let register_bridge registry name primitive =
    {
      registry with
      bridge_primitives = (name, primitive) :: registry.bridge_primitives
    }

  let register_defi registry name primitive =
    {
      registry with
      defi_primitives = (name, primitive) :: registry.defi_primitives
    }

  let register_oracle registry name primitive =
    {
      registry with
      oracle_primitives = (name, primitive) :: registry.oracle_primitives
    }

  let register_governance registry name primitive =
    {
      registry with
      governance_primitives =
        (name, primitive) :: registry.governance_primitives
    }

  let lookup_primitive registry domain name =
    let primitives =
      match domain with
      | "token" -> registry.token_primitives
      | "bridge" -> registry.bridge_primitives
      | "defi" -> registry.defi_primitives
      | "oracle" -> registry.oracle_primitives
      | "governance" -> registry.governance_primitives
      | _ -> []
    in
    List.assoc_opt name primitives

  let list_primitives registry domain =
    let primitives =
      match domain with
      | "token" -> registry.token_primitives
      | "bridge" -> registry.bridge_primitives
      | "defi" -> registry.defi_primitives
      | "oracle" -> registry.oracle_primitives
      | "governance" -> registry.governance_primitives
      | _ -> []
    in
    List.map fst primitives
end

(* Default primitive registry *)
let default_primitive_registry =
  let registry = PrimitiveRegistry.create () in
  let registry =
    PrimitiveRegistry.register_token registry "mint_token" mint_token
  in
  let registry =
    PrimitiveRegistry.register_token registry "burn_token" burn_token
  in
  let registry =
    PrimitiveRegistry.register_token registry "transfer_token" transfer_token
  in
  let registry =
    PrimitiveRegistry.register_token registry "get_balance" get_balance
  in
  let registry =
    PrimitiveRegistry.register_token registry "approve_token" approve_token
  in
  let registry =
    PrimitiveRegistry.register_bridge registry "lock_tokens" lock_tokens
  in
  let registry =
    PrimitiveRegistry.register_bridge registry "mint_wrapped" mint_wrapped
  in
  let registry =
    PrimitiveRegistry.register_bridge registry "burn_wrapped" burn_wrapped
  in
  let registry =
    PrimitiveRegistry.register_bridge registry "unlock_tokens" unlock_tokens
  in
  let registry =
    PrimitiveRegistry.register_bridge registry "verify_bridge_proof"
      verify_bridge_proof
  in
  let registry =
    PrimitiveRegistry.register_defi registry "add_liquidity" add_liquidity
  in
  let registry =
    PrimitiveRegistry.register_defi registry "remove_liquidity" remove_liquidity
  in
  let registry =
    PrimitiveRegistry.register_defi registry "swap_tokens" swap_tokens
  in
  let registry =
    PrimitiveRegistry.register_oracle registry "get_price" get_price
  in
  let registry =
    PrimitiveRegistry.register_oracle registry "update_price" update_price
  in
  let registry =
    PrimitiveRegistry.register_governance registry "create_proposal"
      create_proposal
  in
  let registry =
    PrimitiveRegistry.register_governance registry "vote_on_proposal"
      vote_on_proposal
  in
  let registry =
    PrimitiveRegistry.register_governance registry "execute_proposal"
      execute_proposal
  in
  registry
