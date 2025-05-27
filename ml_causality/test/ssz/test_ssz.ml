(** Test module for SSZ.
    Simple tests to verify the SSZ implementation. *)

open Ssz

(** Test basic type serialization *)
let test_basic_types () =
  let test_bool = Basic.bool in
  let encoded_true = Serialize.encode test_bool true in
  let decoded_true = Serialize.decode test_bool encoded_true in
  assert (decoded_true = true);
  
  let encoded_false = Serialize.encode test_bool false in
  let decoded_false = Serialize.decode test_bool encoded_false in
  assert (decoded_false = false);
  
  let test_uint8 = Basic.uint8 in
  let encoded_uint8 = Serialize.encode test_uint8 42 in
  let decoded_uint8 = Serialize.decode test_uint8 encoded_uint8 in
  assert (decoded_uint8 = 42);
  
  let test_uint16 = Basic.uint16 in
  let encoded_uint16 = Serialize.encode test_uint16 12345 in
  let decoded_uint16 = Serialize.decode test_uint16 encoded_uint16 in
  assert (decoded_uint16 = 12345);
  
  let test_uint32 = Basic.uint32 in
  let encoded_uint32 = Serialize.encode test_uint32 123456789 in
  let decoded_uint32 = Serialize.decode test_uint32 encoded_uint32 in
  assert (decoded_uint32 = 123456789);
  
  let test_uint64 = Basic.uint64 in
  let encoded_uint64 = Serialize.encode test_uint64 123456789012345L in
  let decoded_uint64 = Serialize.decode test_uint64 encoded_uint64 in
  assert (decoded_uint64 = 123456789012345L);
  
  let test_string = Basic.string in
  let encoded_string = Serialize.encode test_string "Hello, SSZ!" in
  let decoded_string = Serialize.decode test_string encoded_string in
  assert (decoded_string = "Hello, SSZ!");
  
  print_endline "Basic type tests passed!"

(** Test collection type serialization *)
let test_collections () =
  (* Fixed array test *)
  let test_fixed_array = Collections.fixed_array Basic.uint16 3 in
  let arr = [|1; 2; 3|] in
  let encoded_array = Serialize.encode test_fixed_array arr in
  let decoded_array = Serialize.decode test_fixed_array encoded_array in
  assert (decoded_array.(0) = 1);
  assert (decoded_array.(1) = 2);
  assert (decoded_array.(2) = 3);
  
  (* List test *)
  let test_list = Collections.list Basic.uint32 10 in
  let lst = [10; 20; 30; 40] in
  let encoded_list = Serialize.encode test_list lst in
  let decoded_list = Serialize.decode test_list encoded_list in
  assert (decoded_list = [10; 20; 30; 40]);
  
  (* String list test *)
  let test_string_list = Collections.list Basic.string 10 in
  let str_lst = ["hello"; "ssz"; "serialization"] in
  let encoded_str_list = Serialize.encode test_string_list str_lst in
  let decoded_str_list = Serialize.decode test_string_list encoded_str_list in
  assert (decoded_str_list = ["hello"; "ssz"; "serialization"]);
  
  print_endline "Collection type tests passed!"

(** Test container type serialization with tuples instead of records *)
let test_containers () =
  (* Simple container type (x, y) pair *)
  let x_field = Container.field Basic.uint32 (fun (x, _) -> x) ~description:"X coordinate" in
  let y_field = Container.field Basic.uint32 (fun (_, y) -> y) ~description:"Y coordinate" in
  
  let point_type = Container.create2
    Basic.uint32 Basic.uint32
    ~construct:(fun x y -> (x, y))
    x_field y_field
  in
  
  let p = (123, 456) in
  let encoded_point = Serialize.encode point_type p in
  let decoded_point = Serialize.decode point_type encoded_point in
  
  assert (fst decoded_point = 123);
  assert (snd decoded_point = 456);
  
  (* Nested tuple container *)
  let start_field = Container.field point_type (fun (start, _) -> start) ~description:"Start point" in
  let end_field = Container.field point_type (fun (_, end_point) -> end_point) ~description:"End point" in
  
  let line_type = Container.create2
    point_type point_type
    ~construct:(fun start end_point -> (start, end_point))
    start_field end_field
  in
  
  let line = ((10, 20), (30, 40)) in
  let encoded_line = Serialize.encode line_type line in
  let decoded_line = Serialize.decode line_type encoded_line in
  
  let start_point = fst decoded_line in
  let end_point = snd decoded_line in
  assert (fst start_point = 10);
  assert (snd start_point = 20);
  assert (fst end_point = 30);
  assert (snd end_point = 40);
  
  print_endline "Container type tests passed!"

(** Test merkleization *)
let test_merkleization () =
  (* Test hash tree root for basic type *)
  let bool_root_true = Merkle.hash_tree_root Basic.bool true in
  let bool_root_false = Merkle.hash_tree_root Basic.bool false in
  
  (* They should be different *)
  assert (not (Bytes.equal bool_root_true bool_root_false));
  
  (* Test hash tree root for list *)
  let list_type = Collections.list Basic.uint32 10 in
  let list1 = [1; 2; 3; 4] in
  let list2 = [1; 2; 3; 5] in
  
  let root1 = Merkle.hash_tree_root list_type list1 in
  let root2 = Merkle.hash_tree_root list_type list2 in
  
  (* Small change should result in different roots *)
  assert (not (Bytes.equal root1 root2));
  
  print_endline "Merkleization tests passed!"

(** Run all tests *)
let run_tests () =
  test_basic_types ();
  test_collections ();
  test_containers ();
  test_merkleization ();
  print_endline "All tests passed!"

(* Run tests when executed *)
let () = run_tests () 