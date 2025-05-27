(* Basic SSZ tests *)

open Ssz

(* Test boolean serialization *)
let test_bool () =
  let values = [true; false] in
  List.iter (fun value ->
    let serialized = Basic.serialize_bool value in
    let deserialized = Basic.deserialize_bool serialized in
    assert (value = deserialized);
    Printf.printf "Bool %b: serialization roundtrip ok\n" value
  ) values

(* Test integer serialization *)
let test_uint32 () =
  let values = [0; 1; 42; 1000; 0xFFFF; 0xFFFFFF] in
  List.iter (fun value ->
    let serialized = Basic.serialize_uint32 value in
    let deserialized = Basic.deserialize_uint32 serialized in
    assert (value = deserialized);
    Printf.printf "Int %d: serialization roundtrip ok\n" value
  ) values

(* Test string serialization *)
let test_string () =
  let values = [""; "Hello"; "Hello, SSZ!"; "Unicode test"] in
  List.iter (fun value ->
    let serialized = Basic.serialize_string value in
    let deserialized = Basic.deserialize_string serialized in
    assert (value = deserialized);
    Printf.printf "String '%s': serialization roundtrip ok\n" value
  ) values

(* Run all tests *)
let () =
  Printf.printf "Running basic SSZ tests\n\n";
  
  test_bool ();
  test_uint32 ();
  test_string ();
  
  Printf.printf "\nAll tests passed!\n" 