(* Test round-trip serialization between Rust and OCaml *)

open Ocaml_causality_interop.Ffi

(* Helper functions for convenience *)
let serialize_value v = 
  match serialize v with
  | Result.Ok data -> Ok (data, Bytes.length data)
  | Result.Error err -> Error err

let deserialize_value data length =
  let trimmed_data = if Bytes.length data > length then
    Bytes.sub data 0 length
  else data in
  match deserialize trimmed_data with
  | Result.Ok v -> Ok v
  | Result.Error err -> Error err

let with_value_cleanup create_fn f =
  let v = create_fn () in
  let result = f v in
  free_value v;
  result

let test_unit_roundtrip () =
  Printf.printf "Testing Unit value round-trip...\n";
  with_value_cleanup create_unit (fun v ->
    let success = test_roundtrip v in
    assert success;
    Printf.printf "✓ Unit value round-trip test passed\n")

let test_bool_roundtrip () =
  Printf.printf "Testing Boolean value round-trip...\n";
  let v1 = create_bool true in
  let v2 = create_bool false in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  free_value v1;
  free_value v2;
  assert success1;
  assert success2;
  Printf.printf "✓ Boolean value round-trip test passed\n"

let test_int_roundtrip () =
  Printf.printf "Testing Integer value round-trip...\n";
  let v1 = create_int 42 in
  let v2 = create_int 0 in
  let v3 = create_int 999999 in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  let success3 = test_roundtrip v3 in
  free_value v1;
  free_value v2;
  free_value v3;
  assert success1;
  assert success2;
  assert success3;
  Printf.printf "✓ Integer value round-trip test passed\n"

let test_string_roundtrip () =
  Printf.printf "Testing String value round-trip...\n";
  let v1 = create_string "Hello, World!" in
  let v2 = create_string "" in
  let v3 = create_string "OCaml ↔ Rust FFI" in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  let success3 = test_roundtrip v3 in
  free_value v1;
  free_value v2;
  free_value v3;
  assert success1;
  assert success2;
  assert success3;
  Printf.printf "✓ String value round-trip test passed\n"

let test_symbol_roundtrip () =
  Printf.printf "Testing Symbol value round-trip...\n";
  let v1 = create_symbol "test_symbol" in
  let v2 = create_symbol "another_symbol" in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  free_value v1;
  free_value v2;
  assert success1;
  assert success2;
  Printf.printf "✓ Symbol value round-trip test passed\n"

let test_serialization_deserialization () =
  Printf.printf "Testing manual serialization/deserialization...\n";
  let v = create_string "test serialization" in
  match serialize_value v with
  | Ok (data, length) ->
    (match deserialize_value data length with
    | Ok deserialized ->
      let original_type = get_type v in
      let deserialized_type = get_type deserialized in
      assert (original_type = deserialized_type);
      free_value v;
      free_value deserialized;
      Printf.printf "✓ Manual serialization/deserialization test passed\n"
    | Error err ->
      free_value v;
      failwith ("Deserialization failed: " ^ err))
  | Error err ->
    free_value v;
    failwith ("Serialization failed: " ^ err)

let test_value_inspection () =
  Printf.printf "Testing value inspection...\n";
  
  let v_bool = create_bool true in
  assert (get_type v_bool = Bool);
  (match as_bool v_bool with
  | Some b -> assert b
  | None -> failwith "Failed to extract bool value");
  free_value v_bool;
    
  let v_int = create_int 42 in
  assert (get_type v_int = Int);
  (match as_int v_int with
  | Some i -> assert (i = 42)
  | None -> failwith "Failed to extract int value");
  free_value v_int;
    
  let v_string = create_string "test" in
  assert (get_type v_string = String);
  (match as_string v_string with
  | Some s -> assert (s = "test")
  | None -> failwith "Failed to extract string value");
  free_value v_string;
    
  Printf.printf "✓ Value inspection test passed\n"

let run_comprehensive_tests () =
  Printf.printf "Testing comprehensive FFI round-trips...\n";
  match test_ocaml_roundtrips () with
  | Result.Ok msg -> Printf.printf "✓ %s\n" msg
  | Result.Error err -> Printf.printf "✗ Comprehensive test failed: %s\n" err

let () =
  Printf.printf "OCaml ↔ Rust FFI Round-trip Tests\n";
  Printf.printf "=================================\n\n";
  
  test_unit_roundtrip ();
  test_bool_roundtrip ();
  test_int_roundtrip ();
  test_string_roundtrip ();
  test_symbol_roundtrip ();
  test_serialization_deserialization ();
  test_value_inspection ();
  run_comprehensive_tests ();
  
  Printf.printf "\n🎉 All FFI round-trip tests completed! ✓\n" 