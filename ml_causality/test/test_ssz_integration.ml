(* Purpose: Test SSZ integration between ml_ssz and ml_causality *)

open Printf
open Ssz_bridge

(* Test basic serialization functionality *)
let test_basic_serialization () =
  printf "=== Testing Basic SSZ Serialization ===\n";
  
  (* Test boolean serialization *)
  let test_bool = true in
  let serialized_bool = Core.serialize_bool test_bool in
  let deserialized_bool = Core.deserialize_bool serialized_bool in
  printf "Bool roundtrip: %b -> %b (%s)\n" 
    test_bool deserialized_bool 
    (if test_bool = deserialized_bool then "✓" else "✗");
  
  (* Test uint32 serialization *)
  let test_uint32 = 42 in
  let serialized_uint32 = Core.serialize_uint32 test_uint32 in
  let deserialized_uint32 = Core.deserialize_uint32 serialized_uint32 in
  printf "UInt32 roundtrip: %d -> %d (%s)\n" 
    test_uint32 deserialized_uint32 
    (if test_uint32 = deserialized_uint32 then "✓" else "✗");
  
  (* Test string serialization *)
  let test_string = "Hello SSZ!" in
  let serialized_string = Core.serialize_string test_string in
  let deserialized_string = Core.deserialize_string serialized_string in
  printf "String roundtrip: '%s' -> '%s' (%s)\n" 
    test_string deserialized_string 
    (if test_string = deserialized_string then "✓" else "✗")

(* Test FFI functionality *)
let test_ffi_functionality () =
  printf "\n=== Testing FFI Functionality ===\n";
  
  let test_data = "test data for FFI" in
  
  (* Test hex conversion *)
  let hex = Ffi.bytes_to_hex test_data in
  let back_to_bytes = Ffi.hex_to_bytes hex in
  printf "Hex conversion: '%s' -> '%s' -> '%s' (%s)\n"
    test_data hex back_to_bytes
    (if test_data = back_to_bytes then "✓" else "✗");
    
  (* Test serialize to hex *)
  match Ffi.serialize_to_hex Core.serialize_string test_data with
  | Ok hex_result -> 
      printf "Serialize to hex: success -> %s\n" hex_result;
      (match Ffi.deserialize_from_hex Core.deserialize_string hex_result with
       | Ok final_result -> 
           printf "Deserialize from hex: success -> '%s' (%s)\n" 
             final_result (if test_data = final_result then "✓" else "✗")
       | Error err -> printf "Deserialize from hex failed: %s\n" err)
  | Error err -> printf "Serialize to hex failed: %s\n" err

(* Test content addressing *)
let test_content_addressing () =
  printf "\n=== Testing Content Addressing ===\n";
  
  let test_data = "content for addressing" in
  
  match ContentAddressing.compute_content_hash_hex Core.serialize_string test_data with
  | Ok hash -> 
      printf "Content hash (hex): %s\n" hash;
      printf "Hash length: %d bytes\n" (String.length hash / 2)
  | Error err -> 
      printf "Content hash failed: %s\n" err

(* Main test runner *)
let () =
  printf "SSZ Bridge Integration Test\n";
  printf "===========================\n";
  printf "Version: %s\n" version;
  printf "SSZ Enabled: %b\n" ssz_enabled;
  printf "FFI Enabled: %b\n\n" ffi_enabled;
  
  test_basic_serialization ();
  test_ffi_functionality ();
  test_content_addressing ();
  
  printf "\n✅ Integration test complete!\n" 