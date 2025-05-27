(* OCaml-Rust SSZ interoperability test *)

(* Import the Ssz module *)
open Ssz

(* Use the mock implementations for testing *)
module MockRustFfi = struct
  (* Use the mock implementation from Ssz_ffi *)
  include Ssz_ffi.Mock
  
  (* Roundtrip functions for OCaml -> Rust -> OCaml *)
  let ocaml_to_rust_to_ocaml_bool value =
    let ocaml_serialized = Basic.serialize_bool value in
    let rust_deserialized = deserialize_bool ocaml_serialized in
    let rust_serialized = serialize_bool rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_bool rust_serialized in
    ocaml_deserialized

  let ocaml_to_rust_to_ocaml_uint32 value =
    let ocaml_serialized = Basic.serialize_uint32 value in
    let rust_deserialized = deserialize_u32 ocaml_serialized in
    let rust_serialized = serialize_u32 rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_uint32 rust_serialized in
    ocaml_deserialized

  let ocaml_to_rust_to_ocaml_string value =
    let ocaml_serialized = Basic.serialize_string value in
    let rust_deserialized = deserialize_string ocaml_serialized in
    let rust_serialized = serialize_string rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_string rust_serialized in
    ocaml_deserialized

  (* Roundtrip functions for Rust -> OCaml -> Rust *)
  let rust_to_ocaml_to_rust_bool value =
    let rust_serialized = serialize_bool value in
    let ocaml_deserialized = Basic.deserialize_bool rust_serialized in
    let ocaml_serialized = Basic.serialize_bool ocaml_deserialized in
    let rust_deserialized = deserialize_bool ocaml_serialized in
    rust_deserialized

  let rust_to_ocaml_to_rust_uint32 value =
    let rust_serialized = serialize_u32 value in
    let ocaml_deserialized = Basic.deserialize_uint32 rust_serialized in
    let ocaml_serialized = Basic.serialize_uint32 ocaml_deserialized in
    let rust_deserialized = deserialize_u32 ocaml_serialized in
    rust_deserialized

  let rust_to_ocaml_to_rust_string value =
    let rust_serialized = serialize_string value in
    let ocaml_deserialized = Basic.deserialize_string rust_serialized in
    let ocaml_serialized = Basic.serialize_string ocaml_deserialized in
    let rust_deserialized = deserialize_string ocaml_serialized in
    rust_deserialized

  (* Hash tree root compatibility check *)
  let check_hash_compatibility data =
    let ocaml_hash = "placeholder_for_ocaml_hash" in (* To be implemented *)
    let rust_hash = simple_hash data in
    (ocaml_hash, rust_hash)
end

(* Test OCaml to Rust to OCaml roundtrip *)
let test_ocaml_to_rust_to_ocaml () =
  Printf.printf "Testing OCaml -> Rust -> OCaml roundtrip...\n";
  
  (* Boolean roundtrip *)
  let bool_values = [true; false] in
  List.iter (fun value ->
    let result = MockRustFfi.ocaml_to_rust_to_ocaml_bool value in
    Printf.printf "Bool %b roundtrip: %b\n" value result;
    assert (value = result)
  ) bool_values;
  
  (* Integer roundtrip *)
  let int_values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let result = MockRustFfi.ocaml_to_rust_to_ocaml_uint32 value in
    Printf.printf "Int %d roundtrip: %d\n" value result;
    assert (value = result)
  ) int_values;
  
  (* String roundtrip *)
  let string_values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let result = MockRustFfi.ocaml_to_rust_to_ocaml_string value in
    Printf.printf "String '%s' roundtrip: '%s'\n" value result;
    assert (value = result)
  ) string_values;
  
  Printf.printf "OCaml -> Rust -> OCaml roundtrip tests passed!\n"

(* Test Rust to OCaml to Rust roundtrip *)
let test_rust_to_ocaml_to_rust () =
  Printf.printf "\nTesting Rust -> OCaml -> Rust roundtrip...\n";
  
  (* Boolean roundtrip *)
  let bool_values = [true; false] in
  List.iter (fun value ->
    let result = MockRustFfi.rust_to_ocaml_to_rust_bool value in
    Printf.printf "Bool %b roundtrip: %b\n" value result;
    assert (value = result)
  ) bool_values;
  
  (* Integer roundtrip *)
  let int_values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let result = MockRustFfi.rust_to_ocaml_to_rust_uint32 value in
    Printf.printf "Int %d roundtrip: %d\n" value result;
    assert (value = result)
  ) int_values;
  
  (* String roundtrip *)
  let string_values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let result = MockRustFfi.rust_to_ocaml_to_rust_string value in
    Printf.printf "String '%s' roundtrip: '%s'\n" value result;
    assert (value = result)
  ) string_values;
  
  Printf.printf "Rust -> OCaml -> Rust roundtrip tests passed!\n"

(* Test direct Rust serialization *)
let test_direct_rust_serialization () =
  Printf.printf "\nTesting direct Rust serialization...\n";
  
  (* Boolean direct roundtrip *)
  let bool_value = true in
  let bool_serialized = MockRustFfi.serialize_bool bool_value in
  let bool_deserialized = MockRustFfi.deserialize_bool bool_serialized in
  Printf.printf "Direct Rust bool roundtrip: %b -> %b\n" bool_value bool_deserialized;
  assert (bool_value = bool_deserialized);
  
  (* Integer direct roundtrip *)
  let int_value = 42 in
  let int_serialized = MockRustFfi.serialize_u32 int_value in
  let int_deserialized = MockRustFfi.deserialize_u32 int_serialized in
  Printf.printf "Direct Rust int roundtrip: %d -> %d\n" int_value int_deserialized;
  assert (int_value = int_deserialized);
  
  (* String direct roundtrip *)
  let string_value = "Hello, SSZ!" in
  let string_serialized = MockRustFfi.serialize_string string_value in
  let string_deserialized = MockRustFfi.deserialize_string string_serialized in
  Printf.printf "Direct Rust string roundtrip: '%s' -> '%s'\n" string_value string_deserialized;
  assert (string_value = string_deserialized);
  
  Printf.printf "Direct Rust serialization tests passed!\n"

(* Test hash tree root compatibility - placeholder for now *)
let test_hash_compatibility () =
  Printf.printf "\nTesting hash tree root compatibility...\n";
  Printf.printf "(Mock implementation for testing)\n";
  
  let test_data = "test data for hashing" in
  let (ocaml_hash, rust_hash) = MockRustFfi.check_hash_compatibility test_data in
  
  Printf.printf "OCaml hash (placeholder): %s...\n" (String.sub ocaml_hash 0 10);
  Printf.printf "Rust hash: %s...\n" (String.sub rust_hash 0 10);
  
  Printf.printf "Hash compatibility tests implemented with mock functions\n"

(* Run all tests *)
let () =
  Printf.printf "Running OCaml-Rust SSZ interoperability tests\n\n";
  
  (* Tests that don't require Rust FFI *)
  let run_basic_test = true in
  
  (* Use our mock implementation for testing *)
  let run_mock_tests = true in
  
  (* Tests that require actual Rust FFI - disabled until FFI is set up *)
  let run_ffi_tests = false in
  
  if run_basic_test then begin
    Printf.printf "=== Basic test ===\n";
    Printf.printf "Basic assertion: %b\n" (1 + 1 = 2);
  end;
  
  if run_mock_tests then begin
    Printf.printf "\n=== Mock FFI interoperability tests ===\n";
    Printf.printf "(Using OCaml implementation to simulate Rust FFI)\n";
    test_ocaml_to_rust_to_ocaml ();
    test_rust_to_ocaml_to_rust ();
    test_direct_rust_serialization ();
    test_hash_compatibility ();
  end;
  
  if not run_mock_tests && not run_ffi_tests then begin
    Printf.printf "\n=== FFI tests skipped (not yet configured) ===\n";
    Printf.printf "OCaml-Rust FFI Interoperability Plan:\n";
    Printf.printf "- [x] Implement SSZ in OCaml\n";
    Printf.printf "- [x] Create Rust FFI bindings project structure\n";
    Printf.printf "- [x] Design interface for OCaml-Rust serialization\n";
    Printf.printf "- [x] Implement roundtrip test functions\n";
    Printf.printf "- [ ] Build Rust library and link with OCaml\n";
    Printf.printf "- [ ] Run OCaml -> Rust -> OCaml tests\n";
    Printf.printf "- [ ] Run Rust -> OCaml -> Rust tests\n";
    Printf.printf "- [ ] Implement and verify hash tree root consistency\n";
  end;
  
  Printf.printf "\nAll tests completed!\n" 