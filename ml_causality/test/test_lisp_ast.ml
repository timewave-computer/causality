(* Test cases for Lisp AST S-expression Serialization *)
open Ml_causality_lib_types.Types
open Ml_causality_lib_dsl.Lisp_ast

let%test "serialize_simple_atom" = 
  to_sexpr_string (EAtom (Integer 123L)) = "123"

let%test "serialize_var" = 
  to_sexpr_string (Var "x") = "x"

let%test "serialize_simple_apply" = 
  to_sexpr_string (Apply (Combinator Add, [EAtom (Integer 1L); EAtom (Integer 2L)])) = "(+ 1 2)"

let%test "serialize_lambda" = 
  to_sexpr_string (ELambda (["x"; "y"], Apply (Combinator Add, [Var "x"; Var "y"])))
  = "(lambda (x y) (+ x y))"

let%test "serialize_dynamic" =
  to_sexpr_string (Dynamic (100, Var "some_expr")) = "(dynamic 100 some_expr)"

let%test "serialize_const_value_string" =
  to_sexpr_string (Const (VString "hello")) = "hello"

let%test "serialize_const_value_nil" =
  to_sexpr_string (Const VNil) = "nil"

let%test "serialize_const_value_unit" =
  to_sexpr_string (Const Unit) = "nil"

let%test "serialize_value_list" =
  to_sexpr_string (Const (VList [VNumber (NInteger 1L); Bool true])) = "(1 true)"

let%test "serialize_value_map" =
  to_sexpr_string (Const (VMap [(":key1", VNumber (NInteger 10L)); ("key2", VString "val")])) 
  = "(make-map \":key1\" 10 \"key2\" val)"

let%test "serialize_value_ref_value" =
  to_sexpr_string (Const (Ref (Value "some-value-id"))) = "(ref-value some-value-id)"

let%test "serialize_value_ref_expr" =
  to_sexpr_string (Const (Ref (Expr "some-expr-id"))) = "(ref-expr some-expr-id)"

let%test "serialize_empty_list_in_apply" =
  to_sexpr_string (Apply (Var "my-func", [])) = "(my-func)"

let%test "serialize_empty_value_list" =
  to_sexpr_string (Const (VList [])) = "()" 

(* Tests for fixed and ratio numeric values *)
let%test "serialize_fixed_point_number" =
  to_sexpr_string (Const (VNumber (NFixed (1234L, 2)))) = "12.34"

let%test "serialize_ratio_number" =
  to_sexpr_string (Const (VNumber (NRatio { numerator = 3L; denominator = 4L }))) = "(/ 3 4)"

(* Test runner setup for Alcotest - this is usually in a main test file like test.ml or run_tests.ml *)
(* For ppx_inline_test, often just running the executable is enough if tests are auto-discovered. 
   The dune (test ...) stanza handles creating an executable. 
   If we need an explicit Alcotest runner: 

   let () =
     Alcotest.run "Lisp AST Serialization Tests" [
       "Serialization", [
         Alcotest.test_case "Simple Atom" `Quick (
           fun () -> Alcotest.(check string) "same string" "123" (to_sexpr_string (Atom (Integer 123)))
         );
         (* ... more Alcotest.test_case calls ... *)
       ];
     ]
*) 

(* For now, relying on ppx_inline_test auto-discovery through the (test ...) stanza in dune *) 