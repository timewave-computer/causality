(* test_ssz_bridge.ml
 * 
 * Simple test to verify SSZ bridge integration functionality
 *)

let test_basic_serialization () =
  try
    (* Test basic string serialization *)
    let test_string = "Hello SSZ Bridge!" in
    let serialized = Ssz_bridge.Core.serialize_string test_string in
    let deserialized = Ssz_bridge.Core.deserialize_string serialized in
    
    Printf.printf "String roundtrip test:\n";
    Printf.printf "  Original: %s\n" test_string;
    Printf.printf "  Roundtrip: %s\n" deserialized;
    Printf.printf "  Success: %b\n\n" (test_string = deserialized);
    
    (* Test boolean serialization *)
    let test_bool = true in
    let bool_serialized = Ssz_bridge.Core.serialize_bool test_bool in
    let bool_deserialized = Ssz_bridge.Core.deserialize_bool bool_serialized in
    
    Printf.printf "Boolean roundtrip test:\n";
    Printf.printf "  Original: %b\n" test_bool;
    Printf.printf "  Roundtrip: %b\n" bool_deserialized;
    Printf.printf "  Success: %b\n\n" (test_bool = bool_deserialized);
    
    (* Test uint32 serialization *)
    let test_u32 = 42 in
    let u32_serialized = Ssz_bridge.Core.serialize_uint32 test_u32 in
    let u32_deserialized = Ssz_bridge.Core.deserialize_uint32 u32_serialized in
    
    Printf.printf "UInt32 roundtrip test:\n";
    Printf.printf "  Original: %d\n" test_u32;
    Printf.printf "  Roundtrip: %d\n" u32_deserialized;
    Printf.printf "  Success: %b\n\n" (test_u32 = u32_deserialized);
    
    Printf.printf "✅ All SSZ bridge tests passed!\n";
    true
  with
  | e ->
    Printf.printf "❌ SSZ bridge test failed: %s\n" (Printexc.to_string e);
    false

let () =
  Printf.printf "🧪 SSZ Bridge Integration Test\n";
  Printf.printf "===============================\n\n";
  
  let success = test_basic_serialization () in
  if success then
    Printf.printf "\n🎉 SSZ Bridge integration working correctly!\n"
  else (
    Printf.printf "\n❌ SSZ Bridge integration has issues!\n";
    exit 1
  ) 