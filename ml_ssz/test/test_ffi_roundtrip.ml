(* OCaml-Rust SSZ FFI round trip test - using real FFI, not mocks *)

(* Import the Ssz module *)
open Ssz

(* Test OCaml to Rust to OCaml roundtrip using real FFI *)
let test_ocaml_to_rust_to_ocaml () =
  Printf.printf "Testing OCaml -> Rust -> OCaml roundtrip using real FFI...\n";
  
  (* Boolean roundtrip *)
  let bool_values = [true; false] in
  List.iter (fun value ->
    let ocaml_serialized = Basic.serialize_bool value in
    let rust_deserialized = Ssz_ffi.rust_deserialize_bool ocaml_serialized in
    let rust_serialized = Ssz_ffi.rust_serialize_bool rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_bool rust_serialized in
    Printf.printf "Bool %b roundtrip: %b\n" value ocaml_deserialized;
    assert (value = ocaml_deserialized)
  ) bool_values;
  
  (* Integer roundtrip *)
  let int_values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let ocaml_serialized = Basic.serialize_uint32 value in
    let rust_deserialized = Ssz_ffi.rust_deserialize_u32 ocaml_serialized in
    let rust_serialized = Ssz_ffi.rust_serialize_u32 rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_uint32 rust_serialized in
    Printf.printf "Int %d roundtrip: %d\n" value ocaml_deserialized;
    assert (value = ocaml_deserialized)
  ) int_values;
  
  (* String roundtrip *)
  let string_values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let ocaml_serialized = Basic.serialize_string value in
    let rust_deserialized = Ssz_ffi.rust_deserialize_string ocaml_serialized in
    let rust_serialized = Ssz_ffi.rust_serialize_string rust_deserialized in
    let ocaml_deserialized = Basic.deserialize_string rust_serialized in
    Printf.printf "String '%s' roundtrip: '%s'\n" value ocaml_deserialized;
    assert (value = ocaml_deserialized)
  ) string_values;
  
  Printf.printf "OCaml -> Rust -> OCaml roundtrip tests passed!\n"

(* Test Rust to OCaml to Rust roundtrip *)
let test_rust_to_ocaml_to_rust () =
  Printf.printf "\nTesting Rust -> OCaml -> Rust roundtrip...\n";
  
  (* Boolean roundtrip *)
  let bool_values = [true; false] in
  List.iter (fun value ->
    let rust_serialized = Ssz_ffi.rust_serialize_bool value in
    let ocaml_deserialized = Basic.deserialize_bool rust_serialized in
    let ocaml_serialized = Basic.serialize_bool ocaml_deserialized in
    let rust_deserialized = Ssz_ffi.rust_deserialize_bool ocaml_serialized in
    Printf.printf "Bool %b roundtrip: %b\n" value rust_deserialized;
    assert (value = rust_deserialized)
  ) bool_values;
  
  (* Integer roundtrip *)
  let int_values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let rust_serialized = Ssz_ffi.rust_serialize_u32 value in
    let ocaml_deserialized = Basic.deserialize_uint32 rust_serialized in
    let ocaml_serialized = Basic.serialize_uint32 ocaml_deserialized in
    let rust_deserialized = Ssz_ffi.rust_deserialize_u32 ocaml_serialized in
    Printf.printf "Int %d roundtrip: %d\n" value rust_deserialized;
    assert (value = rust_deserialized)
  ) int_values;
  
  (* String roundtrip *)
  let string_values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let rust_serialized = Ssz_ffi.rust_serialize_string value in
    let ocaml_deserialized = Basic.deserialize_string rust_serialized in
    let ocaml_serialized = Basic.serialize_string ocaml_deserialized in
    let rust_deserialized = Ssz_ffi.rust_deserialize_string ocaml_serialized in
    Printf.printf "String '%s' roundtrip: '%s'\n" value rust_deserialized;
    assert (value = rust_deserialized)
  ) string_values;
  
  Printf.printf "Rust -> OCaml -> Rust roundtrip tests passed!\n"

(* Test the direct Rust roundtrip functions *)
let test_rust_direct_roundtrip () =
  Printf.printf "\nTesting direct Rust roundtrip functions...\n";
  
  (* Test Boolean roundtrip *)
  let bool_values = [true; false] in
  List.iter (fun value ->
    let result = Ssz_ffi.rust_roundtrip_bool value in
    Printf.printf "Bool %b direct Rust roundtrip: %b\n" value result;
    assert (value = result)
  ) bool_values;
  
  (* Test Integer roundtrip *)
  let int_values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let result = Ssz_ffi.rust_roundtrip_u32 value in
    Printf.printf "Int %d direct Rust roundtrip: %d\n" value result;
    assert (value = result)
  ) int_values;
  
  (* Test String roundtrip *)
  let string_values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let result = Ssz_ffi.rust_roundtrip_string value in
    Printf.printf "String '%s' direct Rust roundtrip: '%s'\n" value result;
    assert (value = result)
  ) string_values;
  
  Printf.printf "Direct Rust roundtrip tests passed!\n"

(* Run all tests *)
let () =
  Printf.printf "Running OCaml-Rust SSZ FFI roundtrip tests\n\n";
  
  (* Run the tests that use real FFI *)
  test_ocaml_to_rust_to_ocaml ();
  test_rust_to_ocaml_to_rust ();
  test_rust_direct_roundtrip ();
  
  Printf.printf "\nAll FFI roundtrip tests passed!\n" 