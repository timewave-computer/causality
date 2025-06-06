(* Test round-trip serialization between Rust and OCaml *)

open Ocaml_causality_interop.Ffi

let test_unit_roundtrip () =
  let v = create_unit () in
  let success = test_roundtrip v in
  free_value v;
  assert success;
  Printf.printf "✓ Unit value round-trip test passed\n"

let test_bool_roundtrip () =
  let v1 = create_bool true in
  let v2 = create_bool false in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  free_value v1;
  free_value v2;
  assert success1;
  assert success2;
  Printf.printf "✓ Bool value round-trip test passed\n"

let test_int_roundtrip () =
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
  Printf.printf "✓ Int value round-trip test passed\n"

let test_string_roundtrip () =
  let v1 = create_string "hello world" in
  let v2 = create_string "" in
  let v3 = create_string "unicode: 🦀♠️🌟" in
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
  let v1 = create_symbol "my-symbol" in
  let v2 = create_symbol "another_symbol" in
  let success1 = test_roundtrip v1 in
  let success2 = test_roundtrip v2 in
  free_value v1;
  free_value v2;
  assert success1;
  assert success2;
  Printf.printf "✓ Symbol value round-trip test passed\n"

let test_serialization_deserialization () =
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
  with_bool_value true (fun v ->
    assert (get_type v = Bool);
    match as_bool v with
    | Some b -> assert b
    | None -> failwith "Failed to extract bool value");
    
  with_int_value 42 (fun v ->
    assert (get_type v = Int);
    match as_int v with
    | Some i -> assert (i = 42)
    | None -> failwith "Failed to extract int value");
    
  with_string_value "test" (fun v ->
    assert (get_type v = String);
    match as_string v with
    | Some s -> assert (s = "test")
    | None -> failwith "Failed to extract string value");
    
  Printf.printf "✓ Value inspection test passed\n"

let run_all_tests () =
  Printf.printf "Running FFI round-trip tests...\n";
  test_unit_roundtrip ();
  test_bool_roundtrip ();
  test_int_roundtrip ();
  test_string_roundtrip ();
  test_symbol_roundtrip ();
  test_serialization_deserialization ();
  test_value_inspection ();
  Printf.printf "🎉 All FFI round-trip tests passed!\n"

let () = run_all_tests () 