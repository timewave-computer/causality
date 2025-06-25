(* Intent Compiler - specialized compilation for Intent structures *)
(* Purpose: Convert OCaml Intent to Rust Intent with transform constraint analysis *)

open Causality_core.Combined_types

(* External FFI functions from Rust *)
external intent_create : string -> bytes -> bytes = "intent_create"
external intent_add_constraint : bytes -> string -> bool = "intent_add_constraint"
external intent_add_capability : bytes -> string -> bool = "intent_add_capability"
external intent_compile : bytes -> bytes = "intent_compile"

(* Convert OCaml Intent to Rust Intent with full constraint analysis *)
let compile_intent_with_constraints (intent : intent) : expr_id =
  (* Step 1: Create the intent in Rust *)
  let rust_intent_id = intent_create intent.name intent.domain_id in
  
  (* Step 2: Add constraints based on inputs/outputs *)
  List.iter (fun (input : resource_flow) ->
    ignore (intent_add_constraint rust_intent_id ("input_" ^ input.resource_type))
  ) intent.inputs;
  List.iter (fun (output : resource_flow) ->
    ignore (intent_add_constraint rust_intent_id ("output_" ^ output.resource_type))
  ) intent.outputs;
  
  (* Step 3: Add basic capability requirements *)
  let capability_name = "transform_" ^ intent.name in
  ignore (intent_add_capability rust_intent_id capability_name);
  
  (* Step 4: Compile to Layer 1 expression *)
  let compiled_bytes = intent_compile rust_intent_id in
  
  (* Convert bytes to expr_id - for now, just use length as ID *)
  compiled_bytes

(* Analyze resource flows for constraint generation *)
let analyze_resource_flows (inputs : resource_flow list) (outputs : resource_flow list) : string list =
  let constraint_types = ref [] in
  
  (* Check for local vs remote constraints based on domain IDs *)
  let domains = List.fold_left (fun acc (flow : resource_flow) -> 
    if not (List.exists (fun d -> Bytes.equal d flow.domain_id) acc) then 
      flow.domain_id :: acc 
    else 
      acc
  ) [] (inputs @ outputs) in
  
  if List.length domains > 1 then
    constraint_types := "cross_domain" :: !constraint_types;
  
  if List.length inputs > 0 && List.length outputs > 0 then
    constraint_types := "transform" :: !constraint_types;
  
  !constraint_types
