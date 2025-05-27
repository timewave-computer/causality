(* OCaml S-expression serialization tests *)

open Ml_causality.Lib.Types
open Ml_causality.Lib.Types.Sexpr

let test_expr_serialization () =
  let expr = ELambda (["x"; "y"], 
                Apply (Combinator Add, 
                       [Var "x"; Var "y"])) in
  let sexp_str = expr_to_string expr in
  print_endline "Original expression: lambda(x, y) -> add(x, y)";
  print_endline ("S-expression: " ^ sexp_str);
  
  (* Deserialize *)
  let expr' = expr_from_string sexp_str in
  
  (* Check if expr = expr' *)
  match expr' with
  | ELambda (params, Apply (Combinator Add, [Var x; Var y])) ->
      print_endline "Round-trip test passed!";
      let param_str = String.concat ", " params in
      print_endline ("Parameters: [" ^ param_str ^ "]");
      print_endline ("Arguments: " ^ x ^ ", " ^ y)
  | _ ->
      print_endline "Round-trip test failed!";
      print_endline "Expected lambda with apply(add, [var, var])"

let test_value_expr_serialization () =
  let value = VList [
    VString "hello";
    VNumber (NInteger 42L);
    Bool true;
  ] in
  let sexp_str = value_expr_to_string value in
  print_endline "\nOriginal value: [\"hello\", 42, true]";
  print_endline ("S-expression: " ^ sexp_str);
  
  (* Deserialize - will fail because our parser is incomplete *)
  try 
    let _ = value_expr_from_string sexp_str in
    print_endline "Round-trip test passed!"
  with e ->
    print_endline ("Round-trip test failed: " ^ Printexc.to_string e)

let test_content_hash () =
  let expr1 = ELambda (["x"; "y"], 
              Apply (Combinator Add, 
                     [Var "x"; Var "y"])) in
  let expr2 = ELambda (["x"; "y"], 
              Apply (Combinator Add, 
                     [Var "x"; Var "y"])) in
  let expr3 = ELambda (["a"; "b"], 
              Apply (Combinator Add, 
                     [Var "a"; Var "b"])) in
                     
  let hash1 = sexpr_content_hash_hex expr_to_sexp expr1 in
  let hash2 = sexpr_content_hash_hex expr_to_sexp expr2 in
  let hash3 = sexpr_content_hash_hex expr_to_sexp expr3 in
  
  print_endline "\nContent hash tests:";
  print_endline ("Hash of expr1: " ^ hash1);
  print_endline ("Hash of expr2: " ^ hash2);
  print_endline ("Hash of expr3: " ^ hash3);
  print_endline ("expr1 hash = expr2 hash: " ^ string_of_bool (hash1 = hash2));
  print_endline ("expr1 hash = expr3 hash: " ^ string_of_bool (hash1 = hash3))

let () =
  print_endline "Running S-expression serialization tests...";
  test_expr_serialization ();
  test_value_expr_serialization ();
  test_content_hash () 