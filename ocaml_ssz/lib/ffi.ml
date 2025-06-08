(* Purpose: FFI bindings to Rust SSZ serialization functions *)

(* Stubbed FFI implementations that will be replaced when Ctypes is available *)

(* In-memory representation of SSZ bytes result *)
type ffi_ssz_bytes = {
    success : bool
  ; data : string option
  ; error_msg : string option
}

(* In-memory representation of validation result *)
type ffi_validation_result = { valid : bool; error_msg : string option }

(*
 * External functions are stubbed out since we don't have Ctypes available yet.
 * These will be properly implemented later.
 *)

(* Stub implementation for serialization *)
let resource_to_ssz _resource =
  { success = false; data = None; error_msg = Some "FFI not implemented yet" }

let ssz_to_resource _bytes _len = failwith "FFI not implemented yet"
let resource_free _resource = ()

let validate_resource_bytes _bytes _len =
  { valid = false; error_msg = Some "FFI not implemented yet" }

(* Effect serialization/deserialization *)
let effect_to_ssz _effect =
  { success = false; data = None; error_msg = Some "FFI not implemented yet" }

let ssz_to_effect _bytes _len = failwith "FFI not implemented yet"
let effect_free _effect = ()

let validate_effect_bytes _bytes _len =
  { valid = false; error_msg = Some "FFI not implemented yet" }

(* Handler serialization/deserialization *)
let handler_to_ssz _handler =
  { success = false; data = None; error_msg = Some "FFI not implemented yet" }

let ssz_to_handler _bytes _len = failwith "FFI not implemented yet"
let handler_free _handler = ()

let validate_handler_bytes _bytes _len =
  { valid = false; error_msg = Some "FFI not implemented yet" }

(* Edge serialization/deserialization *)
let edge_to_ssz _edge =
  { success = false; data = None; error_msg = Some "FFI not implemented yet" }

let ssz_to_edge _bytes _len = failwith "FFI not implemented yet"
let edge_free _edge = ()

let validate_edge_bytes _bytes _len =
  { valid = false; error_msg = Some "FFI not implemented yet" }

(* FFI result management *)
let free_ssz_result _result = ()
let free_validation_result _result = ()

(* Helper functions for safely handling FFI results *)

let unwrap_ssz_result result =
  match result.data with
  | Some data -> data
  | None ->
      let error =
        match result.error_msg with
        | Some msg -> msg
        | None -> "Unknown error in SSZ serialization"
      in
      failwith error

(* Safe wrappers around FFI functions *)

let serialize_resource resource =
  let result = resource_to_ssz resource in
  let data = unwrap_ssz_result result in
  free_ssz_result result;
  data

let deserialize_resource bytes =
  let resource_ptr = ssz_to_resource bytes (String.length bytes) in
  (* Note: In a real implementation, we would need to copy the data from the
     resource_ptr into an OCaml value before freeing it *)
  let resource = resource_ptr in
  resource_free resource_ptr;
  resource

let is_valid_resource_bytes bytes =
  let result = validate_resource_bytes bytes (String.length bytes) in
  let valid = result.valid in
  free_validation_result result;
  valid

let serialize_effect effect =
  let result = effect_to_ssz effect in
  let data = unwrap_ssz_result result in
  free_ssz_result result;
  data

let deserialize_effect bytes =
  let effect_ptr = ssz_to_effect bytes (String.length bytes) in
  (* Note: In a real implementation, we would need to copy the data from the
     effect_ptr into an OCaml value before freeing it *)
  let effect = effect_ptr in
  effect_free effect_ptr;
  effect

let is_valid_effect_bytes bytes =
  let result = validate_effect_bytes bytes (String.length bytes) in
  let valid = result.valid in
  free_validation_result result;
  valid

let serialize_handler handler =
  let result = handler_to_ssz handler in
  let data = unwrap_ssz_result result in
  free_ssz_result result;
  data

let deserialize_handler bytes =
  let handler_ptr = ssz_to_handler bytes (String.length bytes) in
  (* Note: In a real implementation, we would need to copy the data from the
     handler_ptr into an OCaml value before freeing it *)
  let handler = handler_ptr in
  handler_free handler_ptr;
  handler

let is_valid_handler_bytes bytes =
  let result = validate_handler_bytes bytes (String.length bytes) in
  let valid = result.valid in
  free_validation_result result;
  valid

let serialize_edge edge =
  let result = edge_to_ssz edge in
  let data = unwrap_ssz_result result in
  free_ssz_result result;
  data

let deserialize_edge bytes =
  let edge_ptr = ssz_to_edge bytes (String.length bytes) in
  (* Note: In a real implementation, we would need to copy the data from the
     edge_ptr into an OCaml value before freeing it *)
  let edge = edge_ptr in
  edge_free edge_ptr;
  edge

let is_valid_edge_bytes bytes =
  let result = validate_edge_bytes bytes (String.length bytes) in
  let valid = result.valid in
  free_validation_result result;
  valid
