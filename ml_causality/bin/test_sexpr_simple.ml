(* test_sexpr_simple.ml
 *
 * Simple test for S-expression serialization
 * that doesn't depend on external libraries.
 *)

(* Basic S-expression types - simplified version for testing *)
type sexp =
  | Atom of string
  | List of sexp list

(* Function to convert a sexp to a string *)
let rec sexp_to_string = function
  | Atom s -> "\"" ^ String.escaped s ^ "\""
  | List sxs ->
      "(" ^ String.concat " " (List.map sexp_to_string sxs) ^ ")"

(* Resource record type - simplified version for testing *)
type resource = {
  id: string;
  value: string;
  domain: string;
}

(* Convert a resource to an S-expression *)
let resource_to_sexp resource =
  List [
    Atom "resource";
    List [
      List [Atom ":id"; Atom resource.id];
      List [Atom ":value"; Atom resource.value];
      List [Atom ":domain"; Atom resource.domain];
    ]
  ]

(* Convert a resource to a string representation *)
let resource_to_string resource =
  sexp_to_string (resource_to_sexp resource)

(* Main test function *)
let () =
  let resource = {
    id = "test-resource";
    value = "test-value";
    domain = "test-domain"
  } in
  
  (* Convert to S-expression and print *)
  let sexp_str = resource_to_string resource in
  Printf.printf "Resource as S-expression:\n%s\n" sexp_str;
  
  (* Verify expected format *)
  let expected = "(\"resource\" ((\":id\" \"test-resource\") (\":value\" \"test-value\") (\":domain\" \"test-domain\")))" in
  if sexp_str = expected then
    Printf.printf "✅ Test passed: S-expression format matches expected output\n"
  else (
    Printf.printf "❌ Test failed: S-expression format does not match expected output\n";
    Printf.printf "Expected: %s\n" expected;
    Printf.printf "Got:      %s\n" sexp_str;
    exit 1
  ) 