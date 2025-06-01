(* ------------ CORE CAUSALITY TYPES ------------ *)
(* Purpose: Core Causality types - Intent, Effect, Resource, Handler, Transaction *)

(* Import identifiers from the same module *)
include Identifiers

(* ------------ RESOURCE TYPES ------------ *)

(** Resource flow specification. Defines how resources are consumed or produced. Corresponds to Rust's `ResourceFlow`. *)
type resource_flow = {
  resource_type: str_t;  
  quantity: int64;       
  domain_id: domain_id;  
}

(** Nullifier representing proof that a resource has been consumed. Corresponds to Rust's `Nullifier`. *)
type nullifier = {
  resource_id: entity_id;
  nullifier_hash: bytes;
}

(** Represents a quantifiable asset or capability. Corresponds to Rust's `Resource`. *)
type resource = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  resource_type: str_t; 
  quantity: int64;
  timestamp: timestamp; 
}

(* ------------ CORE CAUSALITY TYPES ------------ *)

(** Represents a desired outcome or goal in the system. Corresponds to Rust's `Intent`. *)
type intent = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  priority: int;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp;
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents a computational effect in the causality system. Corresponds to Rust's `Effect`. *)
type effect = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effect_type: str_t; 
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp; 
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents logic for processing effects or intents. Corresponds to Rust's `Handler`. *)
type handler = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  handles_type: str_t; 
  priority: int; 
  expression: expr_id option;
  timestamp: timestamp;
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents a collection of effects and intents, forming an atomic unit of change. Corresponds to Rust's `Transaction`. *)
type transaction = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effects: entity_id list; 
  intents: entity_id list; 
  inputs: resource_flow list; 
  outputs: resource_flow list; 
  timestamp: timestamp; 
}

(* ------------ HELPER FUNCTIONS ------------ *)

(* TODO: Add type construction and validation functions *)

(* ------------ PRETTY PRINTING ------------ *)

(* TODO: Add pretty printing functions for debugging *) 