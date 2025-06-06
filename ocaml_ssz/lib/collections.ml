(**
 * Collections module for SSZ serialization
 *
 * Implements serialization for collection types like arrays, lists, and vectors
 * in the Simple Serialize (SSZ) format used throughout the project.
 *)

open Types
open Serialize

(*-----------------------------------------------------------------------------
 * Fixed-Size Collections
 *---------------------------------------------------------------------------*)

(** Fixed-size array for elements of the same type with known length *)
let fixed_array element_type length =
  let element_size = match element_type.size with
    | Some size -> size
    | None -> failwith "Fixed array elements must have fixed size"
  in
  {
    kind = Vector;
    size = Some (length * element_size);
    encode = (fun arr ->
      if Array.length arr <> length then
        failwith (Printf.sprintf "Expected array of length %d, got %d" length (Array.length arr));
      
      let result = Bytes.create (length * element_size) in
      let offset = ref 0 in
      
      Array.iter (fun element ->
        let encoded = encode element_type element in
        copy_bytes encoded 0 result !offset element_size;
        offset := !offset + element_size
      ) arr;
      
      result
    );
    decode = (fun bytes offset ->
      let result = Array.make length (Obj.magic 0) in
      let current_offset = ref offset in
      
      for i = 0 to length - 1 do
        let element, new_offset = element_type.decode bytes !current_offset in
        result.(i) <- element;
        current_offset := new_offset
      done;
      
      (result, !current_offset)
    );
  }

(** Variable-length list with maximum length *)
let list element_type max_length =
  {
    kind = List;
    size = None;
    encode = (fun lst ->
      let count = List.length lst in
      if count > max_length then
        failwith (Printf.sprintf "List exceeds maximum length: %d > %d" count max_length);
      
      (* For fixed-size elements, we can optimize *)
      match element_type.size with
      | Some element_size ->
          let data_size = count * element_size in
          let result = Bytes.create (Constants.bytes_per_length_prefix + data_size) in
          
          (* Write length prefix *)
          write_uint32 result 0 count;
          
          (* Write elements sequentially *)
          let offset = ref Constants.bytes_per_length_prefix in
          List.iter (fun element ->
            let encoded = encode element_type element in
            copy_bytes encoded 0 result !offset element_size;
            offset := !offset + element_size
          ) lst;
          
          result
          
      | None ->
          (* For variable-sized elements, we need to build offsets table *)
          let encoded_elements = List.map (fun elem -> encode element_type elem) lst in
          let total_elements_size = List.fold_left (fun acc bytes -> acc + Bytes.length bytes) 0 encoded_elements in
          
          (* Calculate total size: prefix + all encoded elements *)
          let total_size = Constants.bytes_per_length_prefix + total_elements_size in
          let result = Bytes.create total_size in
          
          (* Write length prefix *)
          write_uint32 result 0 count;
          
          (* Copy encoded elements *)
          let offset = ref Constants.bytes_per_length_prefix in
          List.iter (fun elem_bytes ->
            let elem_size = Bytes.length elem_bytes in
            copy_bytes elem_bytes 0 result !offset elem_size;
            offset := !offset + elem_size
          ) encoded_elements;
          
          result
    );
    decode = (fun bytes offset ->
      let count = read_uint32 bytes offset in
      if count > max_length then
        failwith (Printf.sprintf "List exceeds maximum length: %d > %d" count max_length);
      
      let offset = offset + Constants.bytes_per_length_prefix in
      let result = ref [] in
      let current_offset = ref offset in
      
      for _ = 1 to count do
        let element, new_offset = element_type.decode bytes !current_offset in
        result := element :: !result;
        current_offset := new_offset
      done;
      
      (List.rev !result, !current_offset)
    );
  }

(** Vector is a fixed-length homogeneous collection with variable-sized elements *)
let vector element_type =
  {
    kind = Vector;
    size = None; (* Size depends on the elements *)
    encode = (fun arr ->
      (* Calculate total size after encoding all elements *)
      let encoded_elements = Array.map (fun elem -> encode element_type elem) arr in
      let total_elements_size = Array.fold_left (fun acc bytes -> acc + Bytes.length bytes) 0 encoded_elements in
      
      let result = Bytes.create total_elements_size in
      
      (* Copy encoded elements *)
      let offset = ref 0 in
      Array.iter (fun elem_bytes ->
        let elem_size = Bytes.length elem_bytes in
        copy_bytes elem_bytes 0 result !offset elem_size;
        offset := !offset + elem_size
      ) encoded_elements;
      
      result
    );
    decode = (fun _ _ ->
      (* We need to know how many elements to expect *)
      failwith "Vector decode called directly - use with fixed_array for correct decoding"
    );
  }

(** Dictionary as a list of key-value pairs *)
let dict key_type value_type max_pairs =
  let pair_type = {
    kind = Container;
    size = (match key_type.size, value_type.size with
      | Some k_size, Some v_size -> Some (k_size + v_size)
      | _ -> None
    );
    encode = (fun (k, v) ->
      let k_encoded = encode key_type k in
      let v_encoded = encode value_type v in
      let k_size = Bytes.length k_encoded in
      let v_size = Bytes.length v_encoded in
      let result = Bytes.create (k_size + v_size) in
      
      copy_bytes k_encoded 0 result 0 k_size;
      copy_bytes v_encoded 0 result k_size v_size;
      
      result
    );
    decode = (fun bytes offset ->
      let k, next_offset = key_type.decode bytes offset in
      let v, final_offset = value_type.decode bytes next_offset in
      ((k, v), final_offset)
    );
  } in
  
  list pair_type max_pairs 

(* Local constants - these match the ones in Types.Constants *)
module Constants = struct
  let bytes_per_length_prefix = 4
  let bytes_per_length_offset = 4
end 