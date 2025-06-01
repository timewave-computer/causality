(* Purpose: Bridge workflow system for cross-domain operations *)

open Ocaml_causality_core
open Bridge_primitives

(** Workflow step definition *)
type workflow_step = {
  step_id: string;
  step_name: string;
  step_type: string;
  domain_id: Identifiers.domain_id;
  dependencies: string list;
  estimated_duration: int64;
}

(** Bridge workflow definition *)
type bridge_workflow = {
  workflow_id: Identifiers.entity_id;
  name: string;
  config: bridge_config;
  steps: workflow_step list;
  step_transitions: (string * string * string) list; (* from_step, to_step, condition *)
  created_at: int64;
}

(** Workflow execution state *)
type workflow_execution = {
  execution_id: Identifiers.entity_id;
  workflow_id: Identifiers.entity_id;
  bridge_transfer: bridge_transfer;
  current_step: string;
  completed_steps: string list;
  execution_state: string; (* "running", "completed", "failed" *)
  error_message: string option;
  started_at: int64;
  completed_at: int64 option;
}

(*-----------------------------------------------------------------------------
 * Workflow Creation Functions
 *-----------------------------------------------------------------------------*)

(** Create standard bridge transfer workflow steps *)
let create_standard_bridge_steps (source_domain: Identifiers.domain_id) (target_domain: Identifiers.domain_id) =
  [
    {
      step_id = "validate";
      step_name = "Validate Transfer";
      step_type = "validation";
      domain_id = source_domain;
      dependencies = [];
      estimated_duration = 5L;
    };
    {
      step_id = "lock_source";
      step_name = "Lock Source Tokens";
      step_type = "lock";
      domain_id = source_domain;
      dependencies = ["validate"];
      estimated_duration = 30L;
    };
    {
      step_id = "generate_proof";
      step_name = "Generate Transfer Proof";
      step_type = "proof_generation";
      domain_id = source_domain;
      dependencies = ["lock_source"];
      estimated_duration = 60L;
    };
    {
      step_id = "submit_proof";
      step_name = "Submit Proof to Target";
      step_type = "proof_submission";
      domain_id = target_domain;
      dependencies = ["generate_proof"];
      estimated_duration = 45L;
    };
    {
      step_id = "mint_target";
      step_name = "Mint Target Tokens";
      step_type = "mint";
      domain_id = target_domain;
      dependencies = ["submit_proof"];
      estimated_duration = 30L;
    };
    {
      step_id = "finalize";
      step_name = "Finalize Transfer";
      step_type = "finalization";
      domain_id = target_domain;
      dependencies = ["mint_target"];
      estimated_duration = 15L;
    };
  ]

(** Create a complete bridge transfer workflow *)
let create_bridge_transfer_workflow ~bridge_config () =
  let workflow_id = Printf.sprintf "workflow_%s" bridge_config.name |> Bytes.of_string in
  let steps = create_standard_bridge_steps bridge_config.source_domain bridge_config.target_domain in
  let step_transitions = [
    ("validate", "lock_source", "validation_passed");
    ("lock_source", "generate_proof", "tokens_locked");
    ("generate_proof", "submit_proof", "proof_generated");
    ("submit_proof", "mint_target", "proof_verified");
    ("mint_target", "finalize", "tokens_minted");
  ] in
  {
    workflow_id;
    name = Printf.sprintf "%s_transfer_workflow" bridge_config.name;
    config = bridge_config;
    steps;
    step_transitions;
    created_at = Int64.of_float (Unix.time ());
  }

(*-----------------------------------------------------------------------------
 * Workflow Execution Functions
 *-----------------------------------------------------------------------------*)

(** Create workflow execution instance *)
let create_workflow_execution ~workflow ~bridge_transfer () =
  let execution_id = Printf.sprintf "exec_%s_%s" 
    (Bytes.to_string workflow.workflow_id) (Bytes.to_string bridge_transfer.transfer_id) |> Bytes.of_string in
  {
    execution_id;
    workflow_id = workflow.workflow_id;
    bridge_transfer;
    current_step = "validate";
    completed_steps = [];
    execution_state = "running";
    error_message = None;
    started_at = Int64.of_float (Unix.time ());
    completed_at = None;
  }

(** Find next step in workflow *)
let find_next_step (execution: workflow_execution) (workflow: bridge_workflow) =
  let current_step = execution.current_step in
  let next_transitions = List.filter (fun (from_step, _, _) -> from_step = current_step) workflow.step_transitions in
  match next_transitions with
  | (_, next_step, _) :: _ -> Some next_step
  | [] -> None

(** Execute next step in workflow *)
let execute_workflow_step (execution: workflow_execution) (workflow: bridge_workflow) =
  match find_next_step execution workflow with
  | None -> 
    { execution with 
      execution_state = "completed"; 
      completed_at = Some (Int64.of_float (Unix.time ()))
    }
  | Some next_step ->
    { execution with
      current_step = next_step;
      completed_steps = execution.current_step :: execution.completed_steps;
    }

(** Check if workflow is completed *)
let is_workflow_completed (execution: workflow_execution) (workflow: bridge_workflow) =
  let all_steps = List.map (fun step -> step.step_id) workflow.steps in
  let total_completed = List.length execution.completed_steps in
  let total_steps = List.length all_steps in
  execution.execution_state = "completed" || (total_completed = total_steps)

(** Get next available steps *)
let get_next_steps (execution: workflow_execution) (workflow: bridge_workflow) =
  let completed = execution.completed_steps in
  List.filter (fun step ->
    not (List.mem step.step_id completed) &&
    List.for_all (fun dep -> List.mem dep completed) step.dependencies
  ) workflow.steps

(*-----------------------------------------------------------------------------
 * Intent Integration Functions
 *-----------------------------------------------------------------------------*)

(** Create intent for optimized bridge transfer *)
let create_optimized_bridge_intent ~bridge_workflow ~bridge_transfer ~domain_id () =
  let intent_id = Printf.sprintf "intent_%s" (Bytes.to_string bridge_transfer.transfer_id) |> Bytes.of_string in
  {
    id = intent_id;
    name = "OptimizedBridgeTransfer";
    domain_id;
    resource_type = "bridge_intent";
    quantity = bridge_transfer.amount;
    timestamp = bridge_transfer.created_at;
  }

(*-----------------------------------------------------------------------------
 * Example Usage Functions
 *-----------------------------------------------------------------------------*)

(** Complete bridge transfer example workflow *)
let execute_bridge_transfer_example () =
  (* Create example bridge configuration *)
  let bridge_config = create_bridge_config
    ~name:"ethereum_polygon_bridge"
    ~source_domain:(Bytes.of_string "ethereum")
    ~target_domain:(Bytes.of_string "polygon")
    ~fee_basis_points:30 (* 0.3% fee *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~supported_tokens:["USDC"; "ETH"]
    ~verification_type:"zk_proof"
    () in
  
  (* Create example transfer *)
  let bridge_transfer = create_bridge_transfer
    ~bridge_config_id:bridge_config.bridge_id
    ~source_account:(Bytes.of_string "user123")
    ~target_account:(Bytes.of_string "user123_polygon")
    ~token_type:"USDC"
    ~amount:10000L
    () in
  
  (* Create workflow *)
  let workflow = create_bridge_transfer_workflow ~bridge_config () in
  
  (* Create execution *)
  let execution = create_workflow_execution 
    ~workflow 
    ~bridge_transfer 
    () in
  
  (workflow, execution) 