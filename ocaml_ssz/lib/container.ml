(** * Container module for SSZ serialization * * Implements serialization for
    composite types like structs, records, * and other container data structures
    in the Simple Serialize format. *)

open Types
open Serialize

(*-----------------------------------------------------------------------------
 * Container Field Definitions
 *---------------------------------------------------------------------------*)

type ('a, 'b) field = {
    typ : 'a t  (** Type of the field *)
  ; get : 'b -> 'a  (** Getter function *)
  ; description : string  (** Field description for documentation *)
}
(** * A field in a container with type information and accessor *)

(** * Create a field specification with type, getter, and description *)
let field typ getter ~description = { typ; get = getter; description }

type ('a, 'b) container = {
    fields : ('a, 'b) field list  (** List of fields in the container *)
  ; construct : (bytes -> int -> 'a list * int) -> bytes -> int -> 'b * int
        (** Function to construct the value from decoded fields *)
}
(** * A container specification defining fields and construction logic *)

(*-----------------------------------------------------------------------------
 * Container Construction
 *---------------------------------------------------------------------------*)

(* Local constants - these match the ones in Types.Constants *)
module Constants = struct
  let bytes_per_length_prefix = 4
  let bytes_per_length_offset = 4
end

(* Calculate fixed part size - returns the total size of the fixed parts *)
let fixed_part_size fields =
  List.fold_left
    (fun acc f ->
      acc
      +
      match f.typ.size with
      | Some s -> s
      | None -> Constants.bytes_per_length_offset)
    0 fields

(** * Create a serializable container type from its specification *)
let make container =
  (* Calculate if the container has fixed size *)
  let all_fixed =
    List.for_all (fun f -> is_fixed_size f.typ) container.fields
  in
  let size =
    if all_fixed then
      Some
        (List.fold_left
           (fun acc f -> acc + fixed_size f.typ)
           0 container.fields)
    else None
  in

  {
    kind = Container
  ; size
  ; encode =
      (fun value ->
        (* For fixed-size containers, we can serialize fields sequentially *)
        if all_fixed then (
          let total_size =
            fixed_size
              {
                kind = Container
              ; size
              ; encode = (fun _ -> Bytes.empty)
              ; decode = (fun _ _ -> (Obj.magic 0, 0))
              }
          in
          let result = Bytes.create total_size in

          let offset = ref 0 in
          List.iter
            (fun field ->
              let field_value = field.get value in
              let encoded = encode field.typ field_value in
              let field_size = fixed_size field.typ in

              copy_bytes encoded 0 result !offset field_size;
              offset := !offset + field_size)
            container.fields;

          result)
        else
          (* For variable-sized containers, we need to build an offset table *)
          let _fixed_fields, _variable_fields =
            List.partition (fun f -> is_fixed_size f.typ) container.fields
          in

          (* Encode all fields *)
          let encoded_fields =
            List.map
              (fun field ->
                let field_value = field.get value in
                encode field.typ field_value)
              container.fields
          in

          (* Calculate total size of fixed part: fixed fields + offsets *)
          let fixed_size_total =
            List.fold_left
              (fun acc f ->
                acc
                +
                match f.typ.size with
                | Some s -> s
                | None -> Constants.bytes_per_length_offset)
              0 container.fields
          in

          (* Calculate total size including variable-sized fields *)
          let variable_data_size =
            List.fold_left2
              (fun acc field encoded ->
                if is_fixed_size field.typ then acc
                else acc + Bytes.length encoded)
              0 container.fields encoded_fields
          in

          let total_size = fixed_size_total + variable_data_size in
          let result = Bytes.create total_size in

          (* Write fixed fields and offset table *)
          let fixed_offset = ref 0 in
          let var_data_offset = ref fixed_size_total in

          List.iter2
            (fun field encoded ->
              if is_fixed_size field.typ then (
                (* Write fixed-size field directly *)
                let field_size = fixed_size field.typ in
                copy_bytes encoded 0 result !fixed_offset field_size;
                fixed_offset := !fixed_offset + field_size)
              else
                (* Write offset to the variable-sized field *)
                write_uint32 result !fixed_offset !var_data_offset;
              fixed_offset := !fixed_offset + Constants.bytes_per_length_offset)
            container.fields encoded_fields;

          (* Write variable-sized data *)
          List.iter2
            (fun field encoded ->
              if not (is_fixed_size field.typ) then (
                let field_size = Bytes.length encoded in
                copy_bytes encoded 0 result !var_data_offset field_size;
                var_data_offset := !var_data_offset + field_size))
            container.fields encoded_fields;

          result)
  ; decode =
      (fun bytes offset ->
        (* Construct the container value by decoding each field *)
        container.construct
          (fun bytes offset ->
            let fixed_offset = ref offset in
            let var_offset_map = ref [] in

            (* Process fields to get their values *)
            let field_values =
              List.map
                (fun field ->
                  if is_fixed_size field.typ then (
                    (* Decode fixed-size field *)
                    let value, new_offset =
                      field.typ.decode bytes !fixed_offset
                    in
                    fixed_offset := new_offset;
                    value)
                  else
                    (* Read offset to variable-sized field *)
                    let var_offset = read_uint32 bytes !fixed_offset in
                    fixed_offset :=
                      !fixed_offset + Constants.bytes_per_length_offset;
                    var_offset_map := (field, var_offset) :: !var_offset_map;
                    Obj.magic 0 (* Placeholder *))
                container.fields
            in

            (* Find the end of the fixed part *)
            let end_of_fixed = !fixed_offset in

            (* Decode variable-sized fields *)
            let field_values =
              if !var_offset_map = [] then field_values
              else
                (* Sort by offset to ensure we decode in the correct order *)
                let sorted_var_fields =
                  List.sort
                    (fun (_, o1) (_, o2) -> compare o1 o2)
                    !var_offset_map
                in

                (* Create pairs of (offset, next_offset) to know how much to read *)
                let offset_ranges =
                  match sorted_var_fields with
                  | [] -> []
                  | items ->
                      let rec make_ranges acc = function
                        | [] -> List.rev acc
                        | [ (field, offset) ] ->
                            List.rev ((field, offset, Bytes.length bytes) :: acc)
                        | (field, offset) :: ((_, next_offset) :: _ as rest) ->
                            make_ranges
                              ((field, offset, next_offset) :: acc)
                              rest
                      in
                      make_ranges [] items
                in

                (* Decode each variable-sized field *)
                List.fold_left
                  (fun acc (field, start_offset, _end_offset) ->
                    let value, _ =
                      field.typ.decode bytes (offset + start_offset)
                    in

                    (* Replace the placeholder in field_values with the real value *)
                    let field_index =
                      let rec find_index i = function
                        | [] -> failwith "Field not found"
                        | f :: rest ->
                            if f == field then i else find_index (i + 1) rest
                      in
                      find_index 0 container.fields
                    in

                    (* Create new list with the value replaced *)
                    let rec replace i = function
                      | [] -> []
                      | x :: xs ->
                          if i = field_index then value :: xs
                          else x :: replace (i + 1) xs
                    in

                    replace 0 acc)
                  field_values offset_ranges
            in

            (* Calculate the end offset (highest variable field end or fixed part end) *)
            let final_offset =
              if !var_offset_map = [] then end_of_fixed
              else
                let last_var_field =
                  List.fold_left
                    (fun acc (_, offset) -> max acc offset)
                    0 !var_offset_map
                in
                offset + last_var_field
              (* This is an approximation - would need real field sizes *)
            in

            (field_values, final_offset))
          bytes offset)
  }

(** Create a container with one field *)
let create1 _type_a ~construct field_a =
  make
    {
      fields = [ field_a ]
    ; construct =
        (fun decoder bytes offset ->
          match decoder bytes offset with
          | [ a ], offset -> (construct a, offset)
          | _ -> failwith "Invalid container decoder result")
    }

(** Create a container with two fields *)
let create2 _type_a _type_b ~construct field_a field_b =
  make
    {
      fields = [ field_a; field_b ]
    ; construct =
        (fun decoder bytes offset ->
          match decoder bytes offset with
          | [ a; b ], offset -> (construct a b, offset)
          | _ -> failwith "Invalid container decoder result")
    }

(** Create a container with three fields *)
let create3 _type_a _type_b _type_c ~construct field_a field_b field_c =
  make
    {
      fields = [ field_a; field_b; field_c ]
    ; construct =
        (fun decoder bytes offset ->
          match decoder bytes offset with
          | [ a; b; c ], offset -> (construct a b c, offset)
          | _ -> failwith "Invalid container decoder result")
    }

(** Create a container with four fields *)
let create4 _type_a _type_b _type_c _type_d ~construct field_a field_b field_c
    field_d =
  make
    {
      fields = [ field_a; field_b; field_c; field_d ]
    ; construct =
        (fun decoder bytes offset ->
          match decoder bytes offset with
          | [ a; b; c; d ], offset -> (construct a b c d, offset)
          | _ -> failwith "Invalid container decoder result")
    }
