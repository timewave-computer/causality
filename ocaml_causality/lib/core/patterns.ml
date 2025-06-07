(* ------------ RESOURCE PATTERNS ------------ *)
(* Purpose: Resource patterns and matching logic *)

(* Import identifiers from the same module *)
include Identifiers

(* ------------ TYPE DEFINITIONS ------------ *)

(** Resource pattern for matching resources. Corresponds to Rust's `ResourcePattern`. *)
type resource_pattern = {
  resource_type: str_t;
  domain_id: domain_id option;
}

(** Resource flow specification *)
type resource_flow = {
  resource_type: str_t;  
  quantity: int64;       
  domain_id: domain_id;  
}

(* ------------ PATTERN MATCHING ------------ *)

(** Check if a resource matches a pattern *)
let matches_pattern (pattern: resource_pattern) (resource_type: str_t) (domain: domain_id) : bool =
  let type_matches = String.equal pattern.resource_type resource_type in
  let domain_matches = match pattern.domain_id with
    | None -> true  (* Pattern matches any domain *)
    | Some pattern_domain -> Bytes.equal pattern_domain domain
  in
  type_matches && domain_matches

(** Find all resources matching a pattern from a list *)
let filter_by_pattern (pattern: resource_pattern) (resources: (str_t * domain_id) list) : (str_t * domain_id) list =
  List.filter (fun (resource_type, domain) -> 
    matches_pattern pattern resource_type domain) resources

(* ------------ PATTERN CONSTRUCTION ------------ *)

(** Create a pattern that matches any resource of a specific type *)
let pattern_for_type (resource_type: str_t) : resource_pattern =
  { resource_type; domain_id = None }

(** Create a pattern that matches resources of a specific type in a specific domain *)
let pattern_for_type_and_domain (resource_type: str_t) (domain: domain_id) : resource_pattern =
  { resource_type; domain_id = Some domain }

(** Create a wildcard pattern that matches any resource type in any domain *)
let wildcard_pattern : resource_pattern =
  { resource_type = "*"; domain_id = None }

(* ------------ FLOW SPECIFICATIONS ------------ *)

(** Create a resource flow specification *)
let create_flow (resource_type: str_t) (quantity: int64) (domain: domain_id) : resource_flow =
  { resource_type; quantity; domain_id = domain }

(** Check if a flow satisfies minimum quantity requirements *)
let flow_satisfies_minimum (flow: resource_flow) (min_quantity: int64) : bool =
  flow.quantity >= min_quantity

(** Combine multiple flows of the same type and domain *)
let combine_flows (flows: resource_flow list) : resource_flow list =
  let flow_map = Hashtbl.create 16 in
  List.iter (fun flow ->
    let key = (flow.resource_type, flow.domain_id) in
    let current_quantity = match Hashtbl.find_opt flow_map key with
      | None -> 0L
      | Some existing_flow -> existing_flow.quantity
    in
    let combined_flow = { flow with quantity = Int64.add current_quantity flow.quantity } in
    Hashtbl.replace flow_map key combined_flow
  ) flows;
  Hashtbl.fold (fun _ flow acc -> flow :: acc) flow_map [] 