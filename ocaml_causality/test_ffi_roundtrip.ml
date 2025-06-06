open Ocaml_causality_interop.Ffi

let test_unit () =
  Printf.printf "Testing Unit value round-trip...\\n";
  let success = test_unit_roundtrip () in
  assert success;
  Printf.printf "✓ Unit value round-trip test passed\\n"

let test_bool () =
  Printf.printf "Testing Boolean value round-trip...\\n";
  let success1 = test_bool_roundtrip true in
  let success2 = test_bool_roundtrip false in
  assert success1;
  assert success2;
  Printf.printf "✓ Boolean value round-trip test passed\\n"

let test_int () =
  Printf.printf "Testing Integer value round-trip...\\n";
  let success1 = test_int_roundtrip 42 in
  let success2 = test_int_roundtrip 0 in
  let success3 = test_int_roundtrip 999999 in
  assert success1;
  assert success2;
  assert success3;
  Printf.printf "✓ Integer value round-trip test passed\\n"

let test_string () =
  Printf.printf "Testing String value round-trip...\\n";
  let success1 = test_string_roundtrip "Hello, World!" in
  let success2 = test_string_roundtrip "" in
  let success3 = test_string_roundtrip "OCaml ↔ Rust FFI" in
  assert success1;
  assert success2;
  assert success3;
  Printf.printf "✓ String value round-trip test passed\\n"

let test_symbol () =
  Printf.printf "Testing Symbol value round-trip...\\n";
  let success1 = test_symbol_roundtrip "test_symbol" in
  let success2 = test_symbol_roundtrip "another_symbol" in
  assert success1;
  assert success2;
  Printf.printf "✓ Symbol value round-trip test passed\\n"

let () =
  Printf.printf "OCaml ↔ Rust FFI Round-trip Tests\\n";
  Printf.printf "=================================\\n\\n";
  
  test_unit ();
  test_bool ();
  test_int ();
  test_string ();
  test_symbol ();
  
  Printf.printf "\\nAll FFI round-trip tests passed! ✓\\n" 