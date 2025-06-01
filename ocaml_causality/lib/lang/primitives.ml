(*
 * Primitives Module
 *
 * This module provides domain-specific primitive functions and values
 * that can be used in expressions. These primitives offer specialized
 * functionality for specific domains like tokens, bridges, and resources.
 *)

open Ast

(* ------------ TOKEN PRIMITIVES ------------ *)

(** Create a token transfer expression *)
let token_transfer sender receiver amount = 
  let transfer_map = make_map [
    ("operation", str_lit "token_transfer");
    ("sender", sender);
    ("receiver", receiver);
    ("amount", amount);
  ] in
  transfer_map

(** Create a token mint expression *)
let token_mint recipient amount = 
  let mint_map = make_map [
    ("operation", str_lit "token_mint");
    ("recipient", recipient);
    ("amount", amount);
  ] in
  mint_map

(** Create a token burn expression *)
let token_burn sender amount = 
  let burn_map = make_map [
    ("operation", str_lit "token_burn");
    ("sender", sender);
    ("amount", amount);
  ] in
  burn_map

(** Create a token approval expression *)
let token_approve owner spender amount = 
  let approve_map = make_map [
    ("operation", str_lit "token_approve");
    ("owner", owner);
    ("spender", spender);
    ("amount", amount);
  ] in
  approve_map

(* ------------ BRIDGE PRIMITIVES ------------ *)

(** Create a bridge deposit expression *)
let bridge_deposit from_chain to_chain token amount = 
  let deposit_map = make_map [
    ("operation", str_lit "bridge_deposit");
    ("from_chain", from_chain);
    ("to_chain", to_chain);
    ("token", token);
    ("amount", amount);
  ] in
  deposit_map

(** Create a bridge withdrawal expression *)
let bridge_withdraw to_chain recipient token amount proof = 
  let withdraw_map = make_map [
    ("operation", str_lit "bridge_withdraw");
    ("to_chain", to_chain);
    ("recipient", recipient);
    ("token", token);
    ("amount", amount);
    ("proof", proof);
  ] in
  withdraw_map

(** Create a bridge lock expression *)
let bridge_lock token amount timeout = 
  let lock_map = make_map [
    ("operation", str_lit "bridge_lock");
    ("token", token);
    ("amount", amount);
    ("timeout", timeout);
  ] in
  lock_map

(* ------------ RESOURCE PRIMITIVES ------------ *)

(** Create a resource creation expression *)
let create_resource name resource_type quantity = 
  let resource_map = make_map [
    ("operation", str_lit "create_resource");
    ("name", name);
    ("resource_type", resource_type);
    ("quantity", quantity);
  ] in
  resource_map

(** Create a resource transfer expression *)
let transfer_resource resource_id from_id to_id quantity = 
  let transfer_map = make_map [
    ("operation", str_lit "transfer_resource");
    ("resource_id", resource_id);
    ("from_id", from_id);
    ("to_id", to_id);
    ("quantity", quantity);
  ] in
  transfer_map

(** Create a resource transformation expression *)
let transform_resources inputs outputs = 
  let transform_map = make_map [
    ("operation", str_lit "transform_resources");
    ("inputs", inputs);
    ("outputs", outputs);
  ] in
  transform_map

(* ------------ DOMAIN PRIMITIVES ------------ *)

(** Create a domain registration expression *)
let register_domain domain_id domain_type = 
  let domain_map = make_map [
    ("operation", str_lit "register_domain");
    ("domain_id", domain_id);
    ("domain_type", domain_type);
  ] in
  domain_map

(** Create a cross-domain call expression *)
let cross_domain_call from_domain to_domain operation args = 
  let call_map = make_map [
    ("operation", str_lit "cross_domain_call");
    ("from_domain", from_domain);
    ("to_domain", to_domain);
    ("target_operation", operation);
    ("args", args);
  ] in
  call_map

(* ------------ VERIFICATION PRIMITIVES ------------ *)

(** Create a proof verification expression *)
let verify_proof proof public_inputs = 
  let verify_map = make_map [
    ("operation", str_lit "verify_proof");
    ("proof", proof);
    ("public_inputs", public_inputs);
  ] in
  verify_map

(** Create a signature verification expression *)
let verify_signature message signature public_key = 
  let sig_map = make_map [
    ("operation", str_lit "verify_signature");
    ("message", message);
    ("signature", signature);
    ("public_key", public_key);
  ] in
  sig_map 