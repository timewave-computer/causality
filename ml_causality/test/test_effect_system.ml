(* Purpose: Unit tests for the OCaml effect system *)

open Ml_causality_lib_types.Types
open Ml_causality_lib_effect_system
open Ml_causality_lib_dsl
open Ml_causality_lib_ppx_registry

(* OUnit2 for unit testing *)
open OUnit2

(* Test that effect types can be registered and retrieved *)
let test_effect_type_registry _ctx =
  (* Register a test effect type *)
  let effect_name = "TestEffect" in
  let _ = register_effect_type
    ~effect_name
    ~payload_validator:(fun v -> match v with VString _ -> true | _ -> false)
    ~default_handler_id:(Some "test_handler_1")
    ()
  in
  
  (* Check that it can be retrieved *)
  match get_effect_type effect_name with
  | None -> assert_failure "Failed to retrieve registered effect type"
  | Some config ->
      assert_equal effect_name config.effect_name;
      assert_equal (Some "test_handler_1") config.default_handler_id;
      assert_bool "Validator function should accept string" 
        (match config.payload_validator with 
         | Some validator -> validator (VString "test")
         | None -> false);
      assert_bool "Validator function should reject non-string"
        (match config.payload_validator with
         | Some validator -> not (validator (VNumber (NInteger Int64.zero)))
         | None -> false)

(* Test that handlers can be registered and linked to effects *)
let test_handler_registration _ctx =
  (* Register a test effect type *)
  let effect_name = "LinkTestEffect" in
  let _ = register_effect_type ~effect_name () in
  
  (* Register a handler for this effect *)
  let handler_id = "test_handler_2" in
  let handler_name = "Test Handler 2" in
  let config = VRecord [("timeout", VNumber (NInteger (Int64.of_int 30)))] in
  let dynamic_logic_ref = "test_dynamic_logic" in
  
  (* Register the handler logic in the PPX registry *)
  Ppx_registry.register_logic dynamic_logic_ref "(lambda (effect k) (resume-with k \"result\"))";
  
  (* Register the handler *)
  let handler = register_handler
    ~handler_id
    ~handler_name
    ~handles_effects:[effect_name]
    ~config
    ~dynamic_logic_ref
    ()
  in
  
  (* Check handler registration was successful *)
  assert_equal handler_id handler.handler_id;
  assert_equal [effect_name] handler.handles_effects;
  
  (* Check that handler can be found for the effect *)
  let handlers = find_handlers_for_effect_type effect_name in
  assert_bool "Handler should be found for effect" (List.mem handler_id handlers)

(* Test effect-handler linking *)
let test_effect_handler_linking _ctx =
  (* Register effect and handler *)
  let effect_name = "LinkTestEffect2" in
  let _ = register_effect_type ~effect_name () in
  
  let handler_id = "test_handler_3" in
  let _ = register_handler
    ~handler_id
    ~handler_name:"Test Handler 3"
    ~handles_effects:[effect_name]
    ~config:VNil
    ~dynamic_logic_ref:"test_dynamic_logic" (* Reuse existing logic *)
    ()
  in
  
  (* Create an effect instance *)
  let effect = create_effect
    ~effect_type:effect_name
    ~params:(VString "test param")
    ()
  in
  
  (* Connect it to handlers *)
  let edges = link_effect_to_handlers 
    ~effect_id:effect.id
    ~effect_type:effect_name
    ()
  in
  
  (* Verify edges were created *)
  assert_bool "At least one edge should be created" (List.length edges > 0);
  
  (* Check that the edges link to our handler *)
  let has_handler_edge = List.exists (fun edge ->
    edge.target_node_id = handler_id
  ) edges in
  
  assert_bool "Should have edge connecting to handler" has_handler_edge

(* Test continuation usage validation *)
let test_continuation_validation _ctx =
  (* Test cases with different continuation usage patterns *)
  let valid_logic = "(lambda (effect k) (resume-with k \"result\"))" in
  let missing_continuation = "(lambda (effect k) \"result\")" in
  let multiple_continuations = "(lambda (effect k) (let ((r1 (resume-with k 1))) (resume-with k 2)))" in
  
  assert_bool "Valid logic should pass validation" 
    (validate_continuation_usage ~dynamic_logic_code:valid_logic);
    
  assert_bool "Missing continuation should fail validation"
    (not (validate_continuation_usage ~dynamic_logic_code:missing_continuation));
    
  assert_bool "Multiple continuations should fail validation"
    (not (validate_continuation_usage ~dynamic_logic_code:multiple_continuations))

(* Test type-driven translation *)
let test_type_driven_translation _ctx =
  (* Test extracting type info from OCaml code *)
  let ocaml_code = "
    type 'a effect +=
      | GetUser : user_id -> user_info effect
      | UpdateBalance : { account: string; amount: int } -> unit effect
  " in
  
  match extract_effect_type ~ocaml_code with
  | None -> assert_failure "Failed to extract effect type from OCaml code"
  | Some effect_type ->
      assert_equal "GetUser" effect_type.effect_name;
      assert_equal "user_id" effect_type.parameter_type;
      assert_equal "user_info effect" effect_type.return_type;
      
      (* Test generating value expressions from types *)
      let record_type = "{ name: string; age: int; active: bool }" in
      let record_value = generate_value_expr_from_type ~type_str:record_type in
      
      match record_value with
      | VRecord fields ->
          let field_names = List.map fst fields in
          assert_bool "Record should contain name field" (List.mem "name" field_names);
          assert_bool "Record should contain age field" (List.mem "age" field_names);
          assert_bool "Record should contain active field" (List.mem "active" field_names)
      | _ -> assert_failure "Generated value is not a record"

(* Test suite setup *)
let suite =
  "OCaml Effect System Tests" >::: [
    "test_effect_type_registry" >:: test_effect_type_registry;
    "test_handler_registration" >:: test_handler_registration;
    "test_effect_handler_linking" >:: test_effect_handler_linking;
    "test_continuation_validation" >:: test_continuation_validation;
    "test_type_driven_translation" >:: test_type_driven_translation;
  ]

(* Run the tests *)
let () =
  run_test_tt_main suite 