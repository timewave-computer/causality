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
  bridge_config: bridge_config;
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

(** Create a complete bridge transfer workflow *)
val create_bridge_transfer_workflow :
  bridge_config:bridge_config ->
  unit ->
  bridge_workflow

(** Create workflow execution instance *)
val create_workflow_execution :
  workflow_id:Identifiers.entity_id ->
  bridge_transfer:bridge_transfer ->
  unit ->
  workflow_execution

(** Execute next step in workflow *)
val execute_workflow_step :
  workflow_execution ->
  bridge_workflow ->
  workflow_execution

(** Check if workflow is completed *)
val is_workflow_completed :
  workflow_execution ->
  bridge_workflow ->
  bool

(** Get next available steps *)
val get_next_steps :
  workflow_execution ->
  bridge_workflow ->
  workflow_step list

(** Create intent for optimized bridge transfer *)
val create_optimized_bridge_intent :
  bridge_config:bridge_config ->
  bridge_transfer:bridge_transfer ->
  domain_id:Identifiers.domain_id ->
  unit ->
  resource

(** Complete bridge transfer example workflow *)
val execute_bridge_transfer_example :
  unit ->
  bridge_workflow * workflow_execution 