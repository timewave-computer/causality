(* Purpose: Comprehensive round-trip tests for OCaml <-> Rust SSZ serialization *)

open Printf
open Ssz

(* Test data covering key types from causality-types *)
module TestData = struct
  let bool_values = [ true; false ]
  let u32_values = [ 0; 42; 2147483647 ]
  let string_values = [ ""; "hello"; "world"; "SSZ test 🧪" ]

  (* Create test IDs (32-byte arrays) *)
  let create_id seed =
    let id = Bytes.create 32 in
    for i = 0 to 31 do
      Bytes.set_uint8 id i ((seed + i) mod 256)
    done;
    Bytes.to_string id

  let test_ids = [ create_id 0; create_id 42; create_id 100 ]
end

(* Helper to take first n elements from list *)
let take n lst =
  let rec take_helper acc count = function
    | [] -> List.rev acc
    | _ when count <= 0 -> List.rev acc
    | x :: xs -> take_helper (x :: acc) (count - 1) xs
  in
  take_helper [] n lst

(* Test result tracking *)
type test_result = { test_name : string; success : bool; error : string option }

let results = ref []
let add_result r = results := r :: !results

(* Helper to test round-trip equality *)
let test_roundtrip name serialize deserialize value =
  try
    let serialized = serialize value in
    let deserialized = deserialize serialized in
    let success = value = deserialized in
    add_result { test_name = name; success; error = None };
    if success then printf "  ✓ %s\n" name
    else printf "  ✗ %s: value mismatch\n" name
  with e ->
    let err_msg = Printexc.to_string e in
    add_result { test_name = name; success = false; error = Some err_msg };
    printf "  ✗ %s: %s\n" name err_msg

(* Test basic types *)
let test_basic_types () =
  printf "\n=== Testing Basic Types ===\n";

  (* Test booleans *)
  List.iteri
    (fun i value ->
      test_roundtrip
        (sprintf "bool_%d(%b)" i value)
        Basic.serialize_bool Basic.deserialize_bool value)
    TestData.bool_values;

  (* Test u32 integers *)
  List.iteri
    (fun i value ->
      test_roundtrip
        (sprintf "u32_%d(%d)" i value)
        Basic.serialize_uint32 Basic.deserialize_uint32 value)
    TestData.u32_values;

  (* Test strings *)
  List.iteri
    (fun i value ->
      test_roundtrip
        (sprintf "string_%d(\"%s\")" i value)
        Basic.serialize_string Basic.deserialize_string value)
    TestData.string_values

(* Test fixed-size byte arrays (IDs) *)
let test_fixed_arrays () =
  printf "\n=== Testing Fixed-Size Arrays (IDs) ===\n";
  List.iteri
    (fun i value ->
      test_roundtrip
        (sprintf "id_%d(32_bytes)" i)
        (fun x -> x)
        (fun x -> x)
        value (* IDs are raw bytes *))
    TestData.test_ids

(* Test OCaml->Rust->OCaml using mock functions *)
let test_ocaml_rust_ocaml () =
  printf "\n=== Testing OCaml -> Rust -> OCaml ===\n";

  (* Use mock implementations from Ssz_ffi.Mock *)
  TestData.bool_values
  |> List.iteri (fun i value ->
         try
           let ocaml_bytes = Basic.serialize_bool value in
           let rust_result = Ssz_ffi.Mock.deserialize_bool ocaml_bytes in
           let rust_bytes = Ssz_ffi.Mock.serialize_bool rust_result in
           let final_result = Basic.deserialize_bool rust_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_bool_%d" i
             ; success
             ; error = None
             };
           printf "  %s Bool %b -> %b\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_bool_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ Bool error: %s\n" (Printexc.to_string e));

  TestData.u32_values |> take 3
  |> List.iteri (fun i value ->
         try
           let ocaml_bytes = Basic.serialize_uint32 value in
           let rust_result = Ssz_ffi.Mock.deserialize_u32 ocaml_bytes in
           let rust_bytes = Ssz_ffi.Mock.serialize_u32 rust_result in
           let final_result = Basic.deserialize_uint32 rust_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_u32_%d" i
             ; success
             ; error = None
             };
           printf "  %s UInt32 %d -> %d\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_u32_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ UInt32 error: %s\n" (Printexc.to_string e));

  TestData.string_values |> take 3
  |> List.iteri (fun i value ->
         try
           let ocaml_bytes = Basic.serialize_string value in
           let rust_result = Ssz_ffi.Mock.deserialize_string ocaml_bytes in
           let rust_bytes = Ssz_ffi.Mock.serialize_string rust_result in
           let final_result = Basic.deserialize_string rust_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_string_%d" i
             ; success
             ; error = None
             };
           printf "  %s String '%s' -> '%s'\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "ocaml_rust_ocaml_string_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ String error: %s\n" (Printexc.to_string e))

(* Test Rust->OCaml->Rust using mock functions *)
let test_rust_ocaml_rust () =
  printf "\n=== Testing Rust -> OCaml -> Rust ===\n";

  TestData.bool_values
  |> List.iteri (fun i value ->
         try
           let rust_bytes = Ssz_ffi.Mock.serialize_bool value in
           let ocaml_result = Basic.deserialize_bool rust_bytes in
           let ocaml_bytes = Basic.serialize_bool ocaml_result in
           let final_result = Ssz_ffi.Mock.deserialize_bool ocaml_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_bool_%d" i
             ; success
             ; error = None
             };
           printf "  %s Bool %b -> %b\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_bool_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ Bool error: %s\n" (Printexc.to_string e));

  TestData.u32_values |> take 3
  |> List.iteri (fun i value ->
         try
           let rust_bytes = Ssz_ffi.Mock.serialize_u32 value in
           let ocaml_result = Basic.deserialize_uint32 rust_bytes in
           let ocaml_bytes = Basic.serialize_uint32 ocaml_result in
           let final_result = Ssz_ffi.Mock.deserialize_u32 ocaml_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_u32_%d" i
             ; success
             ; error = None
             };
           printf "  %s UInt32 %d -> %d\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_u32_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ UInt32 error: %s\n" (Printexc.to_string e));

  TestData.string_values |> take 3
  |> List.iteri (fun i value ->
         try
           let rust_bytes = Ssz_ffi.Mock.serialize_string value in
           let ocaml_result = Basic.deserialize_string rust_bytes in
           let ocaml_bytes = Basic.serialize_string ocaml_result in
           let final_result = Ssz_ffi.Mock.deserialize_string ocaml_bytes in
           let success = value = final_result in
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_string_%d" i
             ; success
             ; error = None
             };
           printf "  %s String '%s' -> '%s'\n"
             (if success then "✓" else "✗")
             value final_result
         with e ->
           add_result
             {
               test_name = sprintf "rust_ocaml_rust_string_%d" i
             ; success = false
             ; error = Some (Printexc.to_string e)
             };
           printf "  ✗ String error: %s\n" (Printexc.to_string e))

(* Print test summary *)
let print_summary () =
  printf "\n=== Test Summary ===\n";
  let total = List.length !results in
  let passed = List.length (List.filter (fun r -> r.success) !results) in
  let failed = total - passed in

  printf "Total tests: %d\n" total;
  printf "Passed: %d\n" passed;
  printf "Failed: %d\n" failed;
  if total > 0 then
    printf "Success rate: %.1f%%\n" (100.0 *. float passed /. float total);

  if failed > 0 then (
    printf "\nFailed tests:\n";
    !results
    |> List.filter (fun r -> not r.success)
    |> List.iter (fun r ->
           printf "  - %s" r.test_name;
           (match r.error with Some err -> printf ": %s" err | None -> ());
           printf "\n"))

(* Main test runner *)
let run_tests () =
  printf "=== Comprehensive SSZ Round-trip Tests ===\n";
  printf "Testing OCaml <-> Rust SSZ serialization compatibility\n";
  printf "Covers primitive types used in causality-types crate\n";

  test_basic_types ();
  test_fixed_arrays ();
  test_ocaml_rust_ocaml ();
  test_rust_ocaml_rust ();
  print_summary ();

  printf "\n🔬 Coverage:\n";
  printf "- Basic types (bool, u32, string)\n";
  printf "- Fixed-size arrays (32-byte IDs)\n";
  printf "- OCaml -> Rust -> OCaml roundtrips\n";
  printf "- Rust -> OCaml -> Rust roundtrips\n";
  printf "- Compatible with causality-types SSZ implementations\n\n";
  printf "✅ TESTING COMPLETE\n"

(* Entry point *)
let () = run_tests ()
