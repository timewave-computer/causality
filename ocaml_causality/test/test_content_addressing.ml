(* Test content addressing functionality *)

open Causality_system.System_content_addressing

let () =
  Printf.printf "=== Content Addressing Test ===\n";
  
  (* Test basic EntityId creation *)
  let data1 = "hello world" in
  let data2 = "hello world" in
  let data3 = "different data" in
  
  let id1 = EntityId.from_content data1 in
  let id2 = EntityId.from_content data2 in
  let id3 = EntityId.from_content data3 in
  
  Printf.printf "Data1: '%s' -> %s\n" data1 (EntityId.to_hex id1);
  Printf.printf "Data2: '%s' -> %s\n" data2 (EntityId.to_hex id2);
  Printf.printf "Data3: '%s' -> %s\n" data3 (EntityId.to_hex id3);
  
  (* Test that identical content produces identical IDs *)
  if EntityId.equal id1 id2 then
    Printf.printf "✓ Identical content produces identical IDs\n"
  else
    Printf.printf "✗ ERROR: Identical content should produce identical IDs\n";
  
  (* Test that different content produces different IDs *)
  if not (EntityId.equal id1 id3) then
    Printf.printf "✓ Different content produces different IDs\n"
  else
    Printf.printf "✗ ERROR: Different content should produce different IDs\n";
  
  (* Test content addressing with structured data *)
  let struct1 = (42, "test", true) in
  let struct2 = (42, "test", true) in
  let struct3 = (43, "test", true) in
  
  let struct_id1 = EntityId.from_content struct1 in
  let struct_id2 = EntityId.from_content struct2 in
  let struct_id3 = EntityId.from_content struct3 in
  
  Printf.printf "\nStructured data test:\n";
  Printf.printf "Struct1: %s\n" (EntityId.to_hex struct_id1);
  Printf.printf "Struct2: %s\n" (EntityId.to_hex struct_id2);
  Printf.printf "Struct3: %s\n" (EntityId.to_hex struct_id3);
  
  if EntityId.equal struct_id1 struct_id2 then
    Printf.printf "✓ Identical structured data produces identical IDs\n"
  else
    Printf.printf "✗ ERROR: Identical structured data should produce identical IDs\n";
  
  if not (EntityId.equal struct_id1 struct_id3) then
    Printf.printf "✓ Different structured data produces different IDs\n"
  else
    Printf.printf "✗ ERROR: Different structured data should produce different IDs\n";
  
  Printf.printf "\n=== Content Addressing Test Complete ===\n" 