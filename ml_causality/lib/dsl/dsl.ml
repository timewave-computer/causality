(** dsl.ml - Implementation for the Lisp Domain Specific Language (DSL) builders *)

(* Purpose: Implementation of the Lisp DSL, for constructing expressions of type Types.expr. *)

open Ml_causality_lib_types.Types
(* open Lisp_ast  -- disabled pending module integration *)
(* open Ml_causality_lib_ppx_registry -- disabled due to unused warning *)

(*
  Note: Lisp_ast might not be needed directly if all AST construction is via helper functions here.
  If Lisp_ast types (like Lisp_ast.expr) are used directly in this module's interface or implementation in a way
  that `Ml_causality_lib_types.Types.expr` (which aliases it) isn't sufficient, then it might be needed.
  For now, assuming Types.expr is the primary way to refer to the Lisp AST type.
*)
(* open Lisp_ast *)

(* Helper to convert value_expr to a canonical S-expression string for hashing *)
let rec value_expr_to_s_expression (ve: value_expr) : string =
  match ve with
  | VNil -> "nil"
  | VBool b -> string_of_bool b
  | VString s -> Printf.sprintf "\"%s\"" (String.escaped s) (* Ensure strings are properly quoted and escaped *)
  | VInt i -> Int64.to_string i
  | VList items -> 
    let item_strs = List.map value_expr_to_s_expression items in
    Printf.sprintf "(%s)" (String.concat " " item_strs)
  | VMap m ->
    (* Sort entries by key for canonical representation *)
    let sorted_entries = BatMap.bindings m |> List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) in
    let entry_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_entries in
    Printf.sprintf "(map (%s))" (String.concat " " entry_strs)
  | VStruct fields -> 
    (* Sort fields by key for canonical representation if not inherently ordered *)
    let sorted_fields = BatMap.bindings fields |> List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) in
    let field_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_fields in
    Printf.sprintf "(struct (%s))" (String.concat " " field_strs)
  | VRef (VERValue id) -> Printf.sprintf "(ref:value %s)" (Bytes.to_string id)
  | VRef (VERExpr id) -> Printf.sprintf "(ref:expr %s)" (Bytes.to_string id)
  | VLambda { params; body_expr_id; captured_env } ->
    let param_str = String.concat " " params in
    (* Sort captured_env by key for canonical representation *)
    let sorted_env = BatMap.bindings captured_env |> List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) in
    let env_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_env in
    Printf.sprintf "(lambda (%s) %s (env %s))" param_str (Bytes.to_string body_expr_id) (String.concat " " env_strs)
  (* No 'other' catch-all needed if all V* types are explicitly handled from types.mli *)

(* Use digestif to create cryptographic hashes for content addressing *)
let value_expr_to_id (ve: value_expr) : value_expr_id =
  let s_expr = value_expr_to_s_expression ve in
  Bytes.of_string (Digestif.SHA256.to_hex (Digestif.SHA256.digest_string s_expr))

(* Utility function to create a unique, deterministic ID from a list of components *)
let _generate_id (components: string list) : bytes =
  (* Filter out empty strings and sort for canonical order if component presence is variable *)
  let sorted_components = List.filter (fun s -> s <> "") components |> List.sort String.compare in
  let concatenated = String.concat ":" sorted_components in
  Bytes.of_string (Digestif.SHA256.to_hex (Digestif.SHA256.digest_string concatenated))

(* Helper to convert expr to a canonical expr_id - Lisp_ast disabled *)
let _lisp_expr_to_expr_id (e: expr) : expr_id =
  (* Simplified implementation since Lisp_ast is missing *)
  let expr_str = match e with
    | EAtom (AInt i) -> "int:" ^ Int64.to_string i
    | EAtom (AString s) -> "str:" ^ s
    | EAtom (ABoolean b) -> "bool:" ^ string_of_bool b
    | EAtom ANil -> "nil"
    | EVar name -> "var:" ^ name
    | _ -> "expr"
  in
  Bytes.of_string (Digestif.SHA256.to_hex (Digestif.SHA256.digest_string expr_str))

(* Helper to convert a direct string (presumed to be canonical Lisp code) to expr_id *)
let _direct_string_to_expr_id (lisp_code_str: string) : expr_id =
  Bytes.of_string (Digestif.SHA256.to_hex (Digestif.SHA256.digest_string lisp_code_str))

(* --- Atom Builders --- *)
let sym name : expr = EVar name (* Represents a symbol/variable *)
let str_lit s : expr = EAtom (AString s)
let int_lit i : expr = EAtom (AInt i)
let bool_lit b : expr = EAtom (ABoolean b)
let nil_lit : expr = EAtom ANil
let keyword_lit k_name : expr = EAtom (AString (Printf.sprintf ":%s" k_name))

(* --- Const Builder --- *)
let const v_expr : expr = EConst v_expr

(* --- Core Structure Builders --- *)

(* (lambda (param1 param2 ...) body_expr) *)
let lambda params body : expr = ELambda (params, body)

(* (apply func arg1 arg2 ...) *)
let apply func args : expr = EApply (func, args)

(* (dynamic N expr) *)
let dynamic steps expr_val : expr = EDynamic (steps, expr_val)

(* --- Combinator Applications --- *)

(* (list item1 item2 ...) *)
let list_ items : expr = EApply (ECombinator List, items)

(* (if cond then_branch else_branch) *)
let if_ cond then_branch else_branch : expr =
  EApply (ECombinator If, [cond; then_branch; else_branch])

(* (let* ((var1 val1) (var2 val2) ...) body_expr) *)
type binding = string * expr
let let_star bindings body_exprs : expr =
  let binding_forms = 
    List.map (fun (var_name, value_expr) -> list_ [sym var_name; value_expr]) bindings
  in
  EApply (ECombinator Let, [list_ binding_forms; list_ body_exprs])

(* (define symbol value-expr) *)
let define symbol_name value_expr : expr =
  EApply (ECombinator Define, [sym symbol_name; value_expr])

(* (defun func_name (param1 param2 ...) body_expr) *)
let defun name params body : expr =
  EApply (ECombinator Defun, [sym name; list_ (List.map sym params); body])

(* (quote data_expr) *)
let quote data_expr : expr = EApply (ECombinator Quote, [data_expr])

(* (make-map (list (key1_expr val1_expr) (key2_expr val2_expr) ...)) *)
let make_map pairs : expr =
  let pair_to_list_expr (k_expr, v_expr) = list_ [k_expr; v_expr] in
  EApply (ECombinator MakeMap, [list_ (List.map pair_to_list_expr pairs)])

(* -- Logical Operations -- *)
let and_ exprs : expr = EApply (ECombinator And, exprs)
let or_ exprs : expr = EApply (ECombinator Or, exprs)
let not_ e : expr = EApply (ECombinator Not, [e])

(* -- Equality -- *)
let eq e1 e2 : expr = EApply (ECombinator Eq, [e1; e2])

(* -- Arithmetic Operations -- *)
let add e1 e2 : expr = EApply (ECombinator Add, [e1; e2])
let sub e1 e2 : expr = EApply (ECombinator Sub, [e1; e2])
let mul e1 e2 : expr = EApply (ECombinator Mul, [e1; e2])
let div e1 e2 : expr = EApply (ECombinator Div, [e1; e2])

(* -- Comparison Operations -- *)
let gt e1 e2 : expr = EApply (ECombinator Gt, [e1; e2])
let lt e1 e2 : expr = EApply (ECombinator Lt, [e1; e2])
let gte e1 e2 : expr = EApply (ECombinator Gte, [e1; e2])
let lte e1 e2 : expr = EApply (ECombinator Lte, [e1; e2])

(* -- Data Access & Context -- *)
let get_context_value key_expr : expr = EApply (ECombinator GetContextValue, [key_expr])
let get_field target field_name : expr = EApply (ECombinator GetField, [target; field_name])
let completed effect_ref_expr : expr = EApply (ECombinator Completed, [effect_ref_expr])

(* -- List Operations -- *)
let nth index list_expr : expr = EApply (ECombinator Nth, [index; list_expr])
let length list_expr : expr = EApply (ECombinator Length, [list_expr])
let cons item list_expr : expr = EApply (ECombinator Cons, [item; list_expr])
let car list_expr : expr = EApply (ECombinator Car, [list_expr])
let cdr list_expr : expr = EApply (ECombinator Cdr, [list_expr])

(* -- Map Operations -- *)
let map_get key map_expr : expr = EApply (ECombinator MapGet, [key; map_expr])
let map_has_key key map_expr : expr = EApply (ECombinator MapHasKey, [key; map_expr])

(* --- SKI C Combinators --- *)
let s_ () : expr = ECombinator S
let k_ () : expr = ECombinator K
let i_ () : expr = ECombinator I
let c_ () : expr = ECombinator C

(* --- Helpers for creating value expressions --- *)
let vint i : value_expr = VInt i
let vbool b : value_expr = VBool b
let vstr s : value_expr = VString s
let vnil : value_expr = VNil
let vlist items : value_expr = VList items
let vmap entries : value_expr = VMap (BatMap.of_enum (BatList.enum entries))

(*-----------------------------------------------------------------------------
  Schema Generation Utilities (for automatic ProcessDataflow schemas)
-----------------------------------------------------------------------------*)

(** Convert type_schema to string representation *)
let rec string_of_type_schema = function
  | Unit -> "unit"
  | Bool -> "bool" 
  | Integer -> "int"
  | Number -> "number"
  | String -> "string"
  | List inner -> "list(" ^ (string_of_type_schema inner) ^ ")"
  | Optional inner -> "option(" ^ (string_of_type_schema inner) ^ ")"
  | Map (k, v) -> "map(" ^ (string_of_type_schema k) ^ ", " ^ (string_of_type_schema v) ^ ")"
  | Record fields -> "record{" ^ (String.concat "; " (List.map (fun (name, schema) -> name ^ ": " ^ (string_of_type_schema schema)) fields)) ^ "}"
  | Union variants -> "union(" ^ (String.concat " | " (List.map string_of_type_schema variants)) ^ ")"
  | Any -> "any"

(*-----------------------------------------------------------------------------
  Generic AST Compilation Framework
-----------------------------------------------------------------------------*)

(** Generic argument types for function compilation *)
type argument = 
  | String of string
  | Variable of string
  | Value of value_expr
  | Call of function_call
  | List of argument list

(** Structured function call representation *)
and function_call = {
  name: string;
  args: argument list;
}

(** Structured function definition *)
type function_def = {
  name: string;
  params: string list;
  body: function_call;
}

(** Structured let binding *)
type let_binding = {
  var_name: string;
  value: argument;
}

(** Structured conditional *)
type conditional = {
  condition: argument;
  then_branch: argument;
  else_branch: argument;
}

(** Generic compilation from structured representation to AST *)
let rec compile_argument = function
  | String s -> EAtom (AString s)
  | Variable v -> EVar v
  | Value ve -> EConst ve
  | Call fc -> compile_function_call fc
  | List args -> EApply (ECombinator List, List.map compile_argument args)

and compile_function_call fc =
  EApply (EVar fc.name, List.map compile_argument fc.args)

(** Compile function definition to AST *)
let compile_function_def fd =
  EApply (ECombinator Defun, [
    EVar fd.name;
    EApply (ECombinator List, List.map (fun p -> EVar p) fd.params);
    compile_function_call fd.body
  ])

(** Compile let bindings to AST *)
let compile_let_bindings bindings body =
  let binding_list = List.map (fun b -> 
    EApply (ECombinator List, [EVar b.var_name; compile_argument b.value])
  ) bindings in
  EApply (ECombinator Let, [
    EApply (ECombinator List, binding_list);
    compile_argument body
  ])

(** Compile conditional to AST *)
let compile_conditional cond =
  EApply (ECombinator If, [
    compile_argument cond.condition;
    compile_argument cond.then_branch;
    compile_argument cond.else_branch
  ])

(** Helper to create function calls *)
let call name args = { name; args }

(** Helper to create string arguments *)
let arg_str s = String s

(** Helper to create variable arguments *)
let arg_var v = Variable v

(** Helper to create value arguments *)
let arg_val ve = Value ve

(** Helper to create call arguments *)
let arg_call fc = Call fc

(** Helper to create list arguments *)
let arg_list args = List args

(*-----------------------------------------------------------------------------
  ID Generation
-----------------------------------------------------------------------------*)

(** [lisp_code_to_expr_id lisp_code_string] generates a content-addressed ID 
    from a Lisp code string. *)
let lisp_code_to_expr_id (lisp_code_string: string) : expr_id =
  _direct_string_to_expr_id lisp_code_string

(** [value_expr_to_id ve] generates a content-addressed ID from a `value_expr`. *)
(* This function is already implemented above as value_expr_to_id *)

(*-----------------------------------------------------------------------------
 * Content-Addressed Effect Mapping Functions
 *---------------------------------------------------------------------------*)

(** In-memory storage for effect mappings *)
let effect_mapping_store : (string, string) Hashtbl.t = Hashtbl.create 100

(** Store mapping from OCaml effect name to TEL effect ID using content-addressed storage *)
let store_effect_mapping (effect_name: string) (tel_effect_id: string) : unit =
  (* Store in memory hashtable for now *)
  Hashtbl.replace effect_mapping_store effect_name tel_effect_id;
  Printf.printf "Stored effect mapping: %s -> %s\n" effect_name tel_effect_id

(** Retrieve TEL effect ID by OCaml effect name from content-addressed storage *)
let get_effect_id_by_name (effect_name: string) : string option =
  (* Retrieve from memory hashtable *)
  let result = Hashtbl.find_opt effect_mapping_store effect_name in
  Printf.printf "Retrieved effect ID for %s: %s\n" effect_name 
    (match result with Some id -> id | None -> "not found");
  result

(*-----------------------------------------------------------------------------
  TypedDomain DSL Functions (Phase 6 Enhancement)
-----------------------------------------------------------------------------*)

(** Create a VerifiableDomain typed domain *)
let create_verifiable_domain ~domain_id ~zk_constraints ~deterministic_only =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "VerifiableDomain");
    ("domain_id", VString (Bytes.to_string domain_id));
    ("zk_constraints", VBool zk_constraints);
    ("deterministic_only", VBool deterministic_only);
  ]))

(** Create a ServiceDomain typed domain *)
let create_service_domain ~domain_id ~external_apis ~non_deterministic_allowed =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "ServiceDomain");
    ("domain_id", VString (Bytes.to_string domain_id));
    ("external_apis", VList (List.map (fun api -> VString api) external_apis));
    ("non_deterministic_allowed", VBool non_deterministic_allowed);
  ]))

(** Create a ComputeDomain typed domain *)
let create_compute_domain ~domain_id ~compute_intensive ~parallel_execution =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "ComputeDomain");
    ("domain_id", VString (Bytes.to_string domain_id));
    ("compute_intensive", VBool compute_intensive);
    ("parallel_execution", VBool parallel_execution);
  ]))

(** DSL function to create typed domain expressions *)
let typed_domain_expr domain_type domain_id options =
  match domain_type with
  | "verifiable" ->
      let zk_constraints = 
        match BatMap.find_opt "zk_constraints" options with
        | Some (VBool b) -> b
        | _ -> true
      in
      let deterministic_only =
        match BatMap.find_opt "deterministic_only" options with
        | Some (VBool b) -> b
        | _ -> true
      in
      EConst (create_verifiable_domain ~domain_id ~zk_constraints ~deterministic_only)
  | "service" ->
      let external_apis =
        match BatMap.find_opt "external_apis" options with
        | Some (VList apis) -> List.map (function VString s -> s | _ -> "") apis
        | _ -> []
      in
      let non_deterministic_allowed =
        match BatMap.find_opt "non_deterministic_allowed" options with
        | Some (VBool b) -> b
        | _ -> false
      in
      EConst (create_service_domain ~domain_id ~external_apis ~non_deterministic_allowed)
  | "compute" ->
      let compute_intensive =
        match BatMap.find_opt "compute_intensive" options with
        | Some (VBool b) -> b
        | _ -> false
      in
      let parallel_execution =
        match BatMap.find_opt "parallel_execution" options with
        | Some (VBool b) -> b
        | _ -> false
      in
      EConst (create_compute_domain ~domain_id ~compute_intensive ~parallel_execution)
  | _ -> 
      (* Default to verifiable domain *)
      EConst (create_verifiable_domain ~domain_id ~zk_constraints:true ~deterministic_only:true)

(*-----------------------------------------------------------------------------
  ProcessDataflowBlock DSL Functions (Phase 6 Enhancement)
-----------------------------------------------------------------------------*)

(** Create a ProcessDataflowBlock node *)
let create_pdb_node ~node_id ~node_type ?typed_domain_policy ?action_template ?(gating_conditions=[]) () =
  VStruct (BatMap.of_enum (BatList.enum [
    ("node_id", VString node_id);
    ("node_type", VString node_type);
    ("typed_domain_policy", match typed_domain_policy with
      | Some domain -> domain
      | None -> VNil);
    ("action_template", match action_template with
      | Some template -> VString (Bytes.to_string template)
      | None -> VNil);
    ("gating_conditions", VList (List.map (fun cond -> VString (Bytes.to_string cond)) gating_conditions));
  ]))

(** Create a ProcessDataflowBlock edge *)
let create_pdb_edge ~from_node ~to_node ?condition ~transition_type () =
  VStruct (BatMap.of_enum (BatList.enum [
    ("from_node", VString from_node);
    ("to_node", VString to_node);
    ("condition", match condition with
      | Some cond -> VString (Bytes.to_string cond)
      | None -> VNil);
    ("transition_type", VString transition_type);
  ]))

(** Create a ProcessDataflowBlock definition with automatic schema generation *)
let create_typed_process_dataflow_definition ~definition_id ~name ~input_generator ~output_generator ~state_generator ~nodes ~edges ~default_typed_domain =
  let input_schema = input_generator.generate_schema () in
  let output_schema = output_generator.generate_schema () in
  let state_schema = state_generator.generate_schema () in
  
  VStruct (BatMap.of_enum (BatList.enum [
    ("definition_id", VString (Bytes.to_string definition_id));
    ("name", VString name);
    ("input_schema_gen", VString (string_of_type_schema input_schema));
    ("output_schema_gen", VString (string_of_type_schema output_schema));
    ("state_schema_gen", VString (string_of_type_schema state_schema));
    ("nodes", VList nodes);
    ("edges", VList edges);
    ("default_typed_domain", default_typed_domain);
  ]))

(** Legacy ProcessDataflowBlock definition for compatibility *)
let create_process_dataflow_definition ~definition_id ~name ~input_schema ~output_schema ~state_schema ~nodes ~edges ~default_typed_domain =
  VStruct (BatMap.of_enum (BatList.enum [
    ("definition_id", VString (Bytes.to_string definition_id));
    ("name", VString name);
    ("input_schema_gen", VNil); (* Legacy - no auto generation *)
    ("output_schema_gen", VNil);
    ("state_schema_gen", VNil);
    ("nodes", VList nodes);
    ("edges", VList edges);
    ("default_typed_domain", default_typed_domain);
  ]))

(** DSL function to create a simple linear ProcessDataflowBlock *)
let linear_process_dataflow ~name ~steps ~default_typed_domain =
  let definition_id = Bytes.of_string (Printf.sprintf "linear_pdb_%s" name) in
  
  (* Create nodes for each step *)
  let nodes = List.mapi (fun i step_name ->
    create_pdb_node 
      ~node_id:(Printf.sprintf "step_%d" i)
      ~node_type:step_name
      ~typed_domain_policy:default_typed_domain
      ()
  ) steps in
  
  (* Create edges connecting sequential steps *)
  let edges = List.mapi (fun i _ ->
    if i < List.length steps - 1 then
      Some (create_pdb_edge 
        ~from_node:(Printf.sprintf "step_%d" i)
        ~to_node:(Printf.sprintf "step_%d" (i + 1))
        ~transition_type:"sequential"
        ())
    else None
  ) steps |> List.filter_map (fun x -> x) in
  
  (* Create basic schemas *)
  let input_schema = BatMap.of_enum (BatList.enum [("input", "any")]) in
  let output_schema = BatMap.of_enum (BatList.enum [("output", "any")]) in
  let state_schema = BatMap.of_enum (BatList.enum [("current_step", "int"); ("data", "any")]) in
  
  create_process_dataflow_definition
    ~definition_id
    ~name
    ~input_schema
    ~output_schema
    ~state_schema
    ~nodes
    ~edges
    ~default_typed_domain

(** DSL function to create a conditional ProcessDataflowBlock *)
let conditional_process_dataflow ~name ~condition_node ~true_branch ~false_branch ~default_typed_domain =
  let definition_id = Bytes.of_string (Printf.sprintf "conditional_pdb_%s" name) in
  
  (* Create nodes *)
  let condition_node_val = create_pdb_node 
    ~node_id:"condition"
    ~node_type:condition_node
    ~typed_domain_policy:default_typed_domain
    () in
  
  let true_node_val = create_pdb_node
    ~node_id:"true_branch"
    ~node_type:true_branch
    ~typed_domain_policy:default_typed_domain
    () in
    
  let false_node_val = create_pdb_node
    ~node_id:"false_branch"
    ~node_type:false_branch
    ~typed_domain_policy:default_typed_domain
    () in
  
  let nodes = [condition_node_val; true_node_val; false_node_val] in
  
  (* Create conditional edges *)
  let true_condition = Bytes.of_string "condition_result == true" in
  let false_condition = Bytes.of_string "condition_result == false" in
  
  let edges = [
    create_pdb_edge
      ~from_node:"condition"
      ~to_node:"true_branch"
      ~condition:true_condition
      ~transition_type:"conditional"
      ();
    create_pdb_edge
      ~from_node:"condition"
      ~to_node:"false_branch"
      ~condition:false_condition
      ~transition_type:"conditional"
      ();
  ] in
  
  (* Create schemas *)
  let input_schema = BatMap.of_enum (BatList.enum [("input", "any"); ("condition_data", "any")]) in
  let output_schema = BatMap.of_enum (BatList.enum [("output", "any")]) in
  let state_schema = BatMap.of_enum (BatList.enum [("condition_result", "bool"); ("data", "any")]) in
  
  create_process_dataflow_definition
    ~definition_id
    ~name
    ~input_schema
    ~output_schema
    ~state_schema
    ~nodes
    ~edges
    ~default_typed_domain

(*-----------------------------------------------------------------------------
  Enhanced Intent and Effect Creation
-----------------------------------------------------------------------------*)

(** Create an Intent with optimization hints *)
let create_intent ~id ~name ~domain_id ~priority ~inputs ~outputs ?expression ?hint ~timestamp () =
  {
    id;
    name;
    domain_id;
    priority;
    inputs;
    outputs;
    expression;
    timestamp;
    hint;
  }

(** Create an Effect with optimization hints *)
let create_effect ~id ~name ~domain_id ~effect_type ~inputs ~outputs ?expression ~timestamp ?hint () =
  {
    id;
    name;
    domain_id;
    effect_type;
    inputs;
    outputs;
    expression;
    timestamp;
    hint;
  }

(*-----------------------------------------------------------------------------
  ProcessDataflowBlock Instance Management
-----------------------------------------------------------------------------*)

(** Create a ProcessDataflowBlock instance state *)
let create_pdb_instance_state ~instance_id ~definition_id ~current_node_id ~state_values ~created_timestamp ~last_updated =
  {
    instance_id;
    definition_id;
    current_node_id;
    state_values;
    created_timestamp;
    last_updated;
  }

(** DSL function to transition a ProcessDataflowBlock instance *)
let transition_pdb_instance ~instance_state ~target_node_id ~new_state_values ~timestamp =
  {
    instance_state with
    current_node_id = target_node_id;
    state_values = new_state_values;
    last_updated = timestamp;
  }