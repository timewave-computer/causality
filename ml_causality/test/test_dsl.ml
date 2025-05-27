(* ml_causality/lib/dsl/test_dsl.ml *)

open Alcotest
open Ml_causality_lib_types
(* Assuming the functions to test are accessible, e.g., via Dsl.Internal or similar *)
(* For now, let's assume they are exposed via the Dsl module for testing purposes *)
module Dsl_to_test = Ml_causality_lib_dsl.Dsl
module Ppx_registry_to_use = Ml_causality_lib_ppx_registry.Ppx_registry

(* Helper to clear Ppx_registry before tests if needed, though Alcotest runs tests sequentially.
   Ppx_registry uses a Hashtbl, so clearing it or using unique keys per test is important.
   For simplicity, we'll use unique keys in these examples.
*)
let Sexp_creator = Ml_causality_lib_dsl.Lisp_ast.Sexp_creator (* For creating dummy Lisp_sexp.t values *)

let mock_lisp_code_static_effect = Sexp_creator.atom "static-effect-logic"
let mock_lisp_code_dynamic_effect = Sexp_creator.atom "dynamic-effect-logic" (* Though effects primarily use static *)
let mock_lisp_code_static_handler = Sexp_creator.atom "static-handler-logic"
let mock_lisp_code_dynamic_handler = Sexp_creator.atom "dynamic-handler-logic"

(* Alcotest testable type for tel_effect_resource *)
let tel_effect_resource_testable =
  testable
    (fun ppf res ->
      Fmt.pf ppf
        "{ id=%s; ocaml_effect_name=%s; payload_value_id=%s; static_logic_key=%s; static_lisp_expr=%s; dynamic_logic_key=%s; dynamic_lisp_expr=%s; domain_id=%s }"
        res.id res.ocaml_effect_name res.payload_value_id
        (match res.static_logic_key with Some k -> k | None -> "None")
        (match res.static_lisp_expr with Some _ -> "Some Lisp" | None -> "None") (* Simplified for example *)
        (match res.dynamic_logic_key with Some k -> k | None -> "None")
        (match res.dynamic_lisp_expr with Some _ -> "Some Lisp" | None -> "None")
        res.domain_id)
    (fun r1 r2 ->
      r1.id = r2.id &&
      r1.ocaml_effect_name = r2.ocaml_effect_name &&
      r1.payload_value_id = r2.payload_value_id &&
      r1.static_logic_key = r2.static_logic_key &&
      (match r1.static_lisp_expr, r2.static_lisp_expr with
       | Some _, Some _ -> true (* Placeholder for actual Lisp_sexp comparison *)
       | None, None -> true
       | _ -> false) &&
      r1.dynamic_logic_key = r2.dynamic_logic_key &&
      (match r1.dynamic_lisp_expr, r2.dynamic_lisp_expr with
       | Some _, Some _ -> true
       | None, None -> true
       | _ -> false) &&
      r1.domain_id = r2.domain_id)

(* Alcotest testable type for tel_handler_resource *)
let tel_handler_resource_testable =
  testable
    (fun ppf res ->
      Fmt.pf ppf
        "{ id=%s; handler_name=%s; config_value_id=%s; static_logic_key=%s; static_lisp_expr=%s; dynamic_logic_key=%s; dynamic_lisp_expr=%s; domain_id=%s }"
        res.id res.handler_name res.config_value_id
        (match res.static_logic_key with Some k -> k | None -> "None")
        (match res.static_lisp_expr with Some _ -> "Some Lisp" | None -> "None")
        res.dynamic_logic_key
        (match res.dynamic_lisp_expr with Some _ -> "Some Lisp" | None -> "None") (* Should be Some _ for valid cases *)
        res.domain_id)
    (fun r1 r2 ->
      r1.id = r2.id &&
      r1.handler_name = r2.handler_name &&
      r1.config_value_id = r2.config_value_id &&
      r1.static_logic_key = r2.static_logic_key &&
      (match r1.static_lisp_expr, r2.static_lisp_expr with
       | Some _, Some _ -> true
       | None, None -> true
       | _ -> false) &&
      r1.dynamic_logic_key = r2.dynamic_logic_key &&
      (match r1.dynamic_lisp_expr, r2.dynamic_lisp_expr with
       | Some _, Some _ -> true
       (* Handler's dynamic_lisp_expr should not be None if key is valid *)
       | None, None -> true (* This case implies key was invalid or type is option, adjust if not so *)
       | _ -> false) &&
      r1.domain_id = r2.domain_id)


(* Test suite for _define_tel_effect_resource *)
let test_define_effect_resource_basic () =
  let domain_id = "test_domain_eff_basic" in
  let effect_name = "TestEffectBasic" in
  let payload_value_id = "eff_payload_val_1" in
  let expected_id = Dsl_to_test._generate_id [domain_id; effect_name; payload_value_id] in
  match Dsl_to_test._define_tel_effect_resource ~effect_name ~payload_value_id ~static_logic_key:None ~dynamic_logic_key:None ~domain_id with
  | Ok res ->
      check tel_effect_resource_testable "Basic effect resource creation" 
        { id = expected_id; ocaml_effect_name = effect_name; payload_value_id;
          static_logic_key = None; static_lisp_expr = None; 
          dynamic_logic_key = None; dynamic_lisp_expr = None; domain_id }
        res
  | Error e -> Alcotest.fail ("Failed to define effect resource: " ^ e)

let test_define_effect_resource_with_static_logic () =
  let domain_id = "test_domain_eff_static" in
  let effect_name = "TestEffectStatic" in
  let payload_value_id = "eff_payload_val_static" in
  let static_key = "static_eff_key_test_1" in
  Ppx_registry_to_use.register_logic static_key mock_lisp_code_static_effect;
  let expected_id = Dsl_to_test._generate_id [domain_id; effect_name; payload_value_id; static_key] in
  match Dsl_to_test._define_tel_effect_resource ~effect_name ~payload_value_id ~static_logic_key:(Some static_key) ~dynamic_logic_key:None ~domain_id with
  | Ok res ->
      check tel_effect_resource_testable "Effect resource with static logic"
        { id = expected_id; ocaml_effect_name = effect_name; payload_value_id;
          static_logic_key = Some static_key; static_lisp_expr = Some mock_lisp_code_static_effect;
          dynamic_logic_key = None; dynamic_lisp_expr = None; domain_id }
        res
  | Error e -> Alcotest.fail ("Failed with static logic: " ^ e)

let test_define_effect_resource_missing_static_logic () =
  let domain_id = "test_domain_eff_missing" in
  let effect_name = "TestEffectMissingStatic" in
  let payload_value_id = "eff_payload_val_missing" in
  let static_key = "non_existent_eff_static_key" in
  match Dsl_to_test._define_tel_effect_resource ~effect_name ~payload_value_id ~static_logic_key:(Some static_key) ~dynamic_logic_key:None ~domain_id with
  | Error _ -> () (* Expected error *)
  | Ok _ -> Alcotest.fail "Expected error for missing static logic key for effect, but got Ok"

let test_define_effect_resource_with_dynamic_logic () =
  let domain_id = "test_domain_eff_dynamic" in
  let effect_name = "TestEffectDynamic" in
  let payload_value_id = "eff_payload_val_dynamic" in
  let dynamic_key = "dynamic_eff_key_test_1" in
  Ppx_registry_to_use.register_logic dynamic_key mock_lisp_code_dynamic_effect;
  let expected_id = Dsl_to_test._generate_id [domain_id; effect_name; payload_value_id; dynamic_key] in (* ID depends on dynamic key if present *)
   match Dsl_to_test._define_tel_effect_resource ~effect_name ~payload_value_id ~static_logic_key:None ~dynamic_logic_key:(Some dynamic_key) ~domain_id with
  | Ok res ->
      check tel_effect_resource_testable "Effect resource with dynamic logic"
        { id = expected_id; ocaml_effect_name = effect_name; payload_value_id;
          static_logic_key = None; static_lisp_expr = None;
          dynamic_logic_key = Some dynamic_key; dynamic_lisp_expr = Some mock_lisp_code_dynamic_effect; domain_id }
        res
  | Error e -> Alcotest.fail ("Failed with dynamic logic: " ^ e)

(* Test suite for _define_tel_handler_resource *)
let test_define_handler_resource_basic () =
  let domain_id = "test_domain_h_basic" in
  let handler_name = "TestHandlerBasic" in
  let config_value_id = "h_config_val_basic" in
  let dynamic_key = "dynamic_h_key_test_basic" in
  Ppx_registry_to_use.register_logic dynamic_key mock_lisp_code_dynamic_handler;
  let expected_id = Dsl_to_test._generate_id [domain_id; handler_name; config_value_id; dynamic_key] in
  match Dsl_to_test._define_tel_handler_resource ~handler_name ~config_value_id ~static_logic_key:None ~dynamic_logic_key:dynamic_key ~domain_id with
  | Ok res ->
      check tel_handler_resource_testable "Basic handler resource"
        { id = expected_id; handler_name; config_value_id;
          static_logic_key = None; static_lisp_expr = None;
          dynamic_logic_key; dynamic_lisp_expr = Some mock_lisp_code_dynamic_handler; domain_id }
        res
  | Error e -> Alcotest.fail ("Failed basic handler: " ^ e)

let test_define_handler_resource_with_static_logic () =
  let domain_id = "test_domain_h_static" in
  let handler_name = "TestHandlerWithStatic" in
  let config_value_id = "h_config_val_static" in
  let static_key = "static_h_key_test_1" in
  let dynamic_key = "dynamic_h_key_test_static" in
  Ppx_registry_to_use.register_logic static_key mock_lisp_code_static_handler;
  Ppx_registry_to_use.register_logic dynamic_key mock_lisp_code_dynamic_handler;
  let expected_id = Dsl_to_test._generate_id [domain_id; handler_name; config_value_id; static_key; dynamic_key] in
  match Dsl_to_test._define_tel_handler_resource ~handler_name ~config_value_id ~static_logic_key:(Some static_key) ~dynamic_logic_key:dynamic_key ~domain_id with
  | Ok res ->
      check tel_handler_resource_testable "Handler with static logic"
        { id = expected_id; handler_name; config_value_id;
          static_logic_key = Some static_key; static_lisp_expr = Some mock_lisp_code_static_handler;
          dynamic_logic_key; dynamic_lisp_expr = Some mock_lisp_code_dynamic_handler; domain_id }
        res
  | Error e -> Alcotest.fail ("Failed handler with static: " ^ e)

let test_define_handler_resource_missing_dynamic_logic () =
  let domain_id = "test_domain_h_missing_dyn" in
  let handler_name = "TestHandlerMissingDyn" in
  let config_value_id = "h_config_val_missing_dyn" in
  let dynamic_key = "non_existent_dyn_h_key" in
  match Dsl_to_test._define_tel_handler_resource ~handler_name ~config_value_id ~static_logic_key:None ~dynamic_logic_key:dynamic_key ~domain_id with
  | Error _ -> () (* Expected error *)
  | Ok _ -> Alcotest.fail "Expected error for missing dynamic logic key for handler, but got Ok"

let test_define_handler_resource_missing_static_logic () =
  let domain_id = "test_domain_h_missing_static" in
  let handler_name = "TestHandlerMissingStatic" in
  let config_value_id = "h_config_val_missing_static" in
  let static_key = "non_existent_static_h_key" in
  let dynamic_key = "dynamic_h_key_for_missing_static" in (* Dynamic key must exist *)
  Ppx_registry_to_use.register_logic dynamic_key mock_lisp_code_dynamic_handler;
  match Dsl_to_test._define_tel_handler_resource ~handler_name ~config_value_id ~static_logic_key:(Some static_key) ~dynamic_logic_key:dynamic_key ~domain_id with
  | Error _ -> () (* Expected error *)
  | Ok _ -> Alcotest.fail "Expected error for missing static logic key for handler, but got Ok"


let effect_resource_tests = [
  test_case "Basic Effect Resource" `Quick test_define_effect_resource_basic;
  test_case "Effect with Static Logic" `Quick test_define_effect_resource_with_static_logic;
  test_case "Effect Missing Static Logic" `Quick test_define_effect_resource_missing_static_logic;
  test_case "Effect with Dynamic Logic" `Quick test_define_effect_resource_with_dynamic_logic;
]

let handler_resource_tests = [
  test_case "Basic Handler Resource" `Quick test_define_handler_resource_basic;
  test_case "Handler with Static Logic" `Quick test_define_handler_resource_with_static_logic;
  test_case "Handler Missing Dynamic Logic" `Quick test_define_handler_resource_missing_dynamic_logic;
  test_case "Handler Missing Static Logic (but valid dynamic)" `Quick test_define_handler_resource_missing_static_logic;
]

let () =
  run "Dsl_Resource_Creation" [
    ("Effect Resource Creation", effect_resource_tests);
    ("Handler Resource Creation", handler_resource_tests);
  ]
