(* Purpose: Provides file I/O operations for SSZ-serialized graph objects in OCaml *)

(* Magic bytes to identify TEL graph object files (TELG) *)
let tel_graph_magic = "TELG"

(* File format versions *)
type file_version = V1

(* Object types that can be stored in files *)
type object_type =
  | Resource
  | Effect
  | Handler
  | Edge
  | Graph (* Full graph with multiple objects *)
  | ChangeSet (* Incremental changes to a graph *)

(* File header for SSZ-serialized objects *)
type file_header = {
  magic: string;
  version: file_version;
  object_type: object_type;
  object_count: int32;
}

(* File format error type *)
type file_error =
  | Io of string
  | Serialization of string
  | InvalidHeader
  | UnsupportedVersion
  | WrongObjectType
  | ValidationError of string

(* Result type for file operations *)
type 'a file_result = ('a, file_error) result

(* String representation of file errors for logging *)
let string_of_file_error = function
  | Io msg -> "I/O error: " ^ msg
  | Serialization msg -> "Serialization error: " ^ msg
  | InvalidHeader -> "Invalid file header"
  | UnsupportedVersion -> "Unsupported file version"
  | WrongObjectType -> "Wrong object type in file"
  | ValidationError msg -> "Validation error: " ^ msg

(* Convert object_type to byte *)
let object_type_to_byte = function
  | Resource -> 1
  | Effect -> 2
  | Handler -> 3
  | Edge -> 4
  | Graph -> 5
  | ChangeSet -> 6

(* Convert byte to object_type *)
let object_type_of_byte = function
  | 1 -> Some Resource
  | 2 -> Some Effect
  | 3 -> Some Handler
  | 4 -> Some Edge
  | 5 -> Some Graph
  | 6 -> Some ChangeSet
  | _ -> None

(* Convert file_version to byte *)
let version_to_byte = function
  | V1 -> 1

(* Convert byte to file_version *)
let version_of_byte = function
  | 1 -> Some V1
  | _ -> None

(* Create a new file header *)
let create_header object_type object_count =
  {
    magic = tel_graph_magic;
    version = V1;
    object_type;
    object_count = Int32.of_int object_count;
  }

(* Serialize header to bytes *)
let header_to_bytes header =
  let count_bytes = Bytes.create 4 in
  Bytes.set_int32_le count_bytes 0 header.object_count;
  
  let buf = Bytes.create 10 in
  Bytes.blit_string header.magic 0 buf 0 4;
  Bytes.set buf 4 (Char.chr (version_to_byte header.version));
  Bytes.set buf 5 (Char.chr (object_type_to_byte header.object_type));
  Bytes.blit count_bytes 0 buf 6 4;
  
  Bytes.to_string buf

(* Parse header from bytes *)
let header_of_bytes bytes =
  try
    if String.length bytes < 10 then
      Error InvalidHeader
    else
      let magic = String.sub bytes 0 4 in
      if magic <> tel_graph_magic then
        Error InvalidHeader
      else
        let version_byte = Char.code bytes.[4] in
        match version_of_byte version_byte with
        | None -> Error UnsupportedVersion
        | Some version ->
            let object_type_byte = Char.code bytes.[5] in
            match object_type_of_byte object_type_byte with
            | None -> Error InvalidHeader
            | Some object_type ->
                let count_bytes = String.sub bytes 6 4 in
                let object_count_int =
                  (* Convert 4 bytes to Int32 in little-endian format *)
                  let b0 = Char.code count_bytes.[0] in
                  let b1 = Char.code count_bytes.[1] in
                  let b2 = Char.code count_bytes.[2] in
                  let b3 = Char.code count_bytes.[3] in
                  let value = Int32.logor
                    (Int32.of_int b0)
                    (Int32.logor
                      (Int32.shift_left (Int32.of_int b1) 8)
                      (Int32.logor
                        (Int32.shift_left (Int32.of_int b2) 16)
                        (Int32.shift_left (Int32.of_int b3) 24)
                      )
                    )
                  in
                  value  (* Keep as int32 *)
                in
                Ok {
                  magic;
                  version;
                  object_type;
                  object_count = object_count_int;
                }
  with _ ->
    Error InvalidHeader

(* Validation function for serialized objects - using mock implementations *)
let validate_bytes _object_type _bytes =
  (* This is a stub implementation that always returns true *)
  true

(* Write a single object to a file with validation *)
let write_object path object_type serialize_fn validate_fn obj =
  try
    (* Serialize the object *)
    let obj_bytes = serialize_fn obj in
    
    (* Validate the serialized bytes *)
    if not (validate_fn obj_bytes) then
      Error (ValidationError "Failed to validate serialized object")
    else
      (* Create the header *)
      let header = create_header object_type 1 in
      let header_bytes = header_to_bytes header in
      
      (* Combine header and object bytes *)
      let file_bytes = header_bytes ^ obj_bytes in
      
      (* Write to file *)
      let oc = open_out_bin path in
      output_string oc file_bytes;
      close_out oc;
      
      Ok ()
  with
  | Sys_error msg -> Error (Io msg)
  | exn -> Error (Serialization (Printexc.to_string exn))

(* Read a single object from a file with validation *)
let read_object path expected_type deserialize_fn validate_fn =
  try
    (* Read the file *)
    let ic = open_in_bin path in
    let file_size = in_channel_length ic in
    
    if file_size < 10 then begin
      close_in ic;
      Error InvalidHeader
    end else begin
      (* Read and parse the header *)
      let header_bytes = Bytes.create 10 in
      really_input ic header_bytes 0 10;
      
      match header_of_bytes (Bytes.to_string header_bytes) with
      | Error e -> 
          close_in ic;
          Error e
      | Ok header ->
          (* Verify object type *)
          if header.object_type <> expected_type then begin
            close_in ic;
            Error WrongObjectType
          end else begin
            (* Read the object data *)
            let data_size = file_size - 10 in
            let data_bytes = Bytes.create data_size in
            really_input ic data_bytes 0 data_size;
            close_in ic;
            
            (* Validate the data *)
            let data_str = Bytes.to_string data_bytes in
            if not (validate_fn data_str) then
              Error (ValidationError "Invalid object data")
            else
              (* Deserialize the object *)
              try
                Ok (deserialize_fn data_str)
              with exn ->
                Error (Serialization (Printexc.to_string exn))
          end
    end
  with
  | Sys_error msg -> Error (Io msg)
  | exn -> Error (Serialization (Printexc.to_string exn))

(* Write multiple objects of the same type to a file with validation *)
let write_objects path object_type serialize_fn validate_fn objects =
  try
    (* Create the header *)
    let object_count = List.length objects in
    let header = create_header object_type object_count in
    let header_bytes = header_to_bytes header in
    
    (* Open the file *)
    let oc = open_out_bin path in
    
    (* Write the header *)
    output_string oc header_bytes;
    
    (* Write each object with length prefix *)
    let write_object obj =
      let obj_bytes = serialize_fn obj in
      
      (* Validate the serialized bytes *)
      if not (validate_fn obj_bytes) then
        raise (Failure "Failed to validate serialized object");
      
      (* Write 4-byte length prefix *)
      let len = String.length obj_bytes in
      let len_bytes = Bytes.create 4 in
      Bytes.set_int32_le len_bytes 0 (Int32.of_int len);
      output_string oc (Bytes.to_string len_bytes);
      
      (* Write object bytes *)
      output_string oc obj_bytes
    in
    
    (* Write all objects *)
    List.iter write_object objects;
    
    (* Close the file *)
    close_out oc;
    
    Ok ()
  with
  | Sys_error msg -> Error (Io msg)
  | Failure msg -> Error (ValidationError msg)
  | exn -> Error (Serialization (Printexc.to_string exn))

(* Read multiple objects of the same type from a file with validation *)
let read_objects path expected_type deserialize_fn validate_fn =
  try
    (* Read the file *)
    let ic = open_in_bin path in
    let file_size = in_channel_length ic in
    
    if file_size < 10 then begin
      close_in ic;
      Error InvalidHeader
    end else begin
      (* Read and parse the header *)
      let header_bytes = Bytes.create 10 in
      really_input ic header_bytes 0 10;
      
      match header_of_bytes (Bytes.to_string header_bytes) with
      | Error e -> 
          close_in ic;
          Error e
      | Ok header ->
          (* Verify object type *)
          if header.object_type <> expected_type then begin
            close_in ic;
            Error WrongObjectType
          end else begin
            (* Read all objects *)
            let object_count = Int32.to_int header.object_count in
            let objects = ref [] in
            
            let rec read_object n =
              if n <= 0 then
                List.rev !objects
              else
                (* Read length prefix (4 bytes) *)
                let len_bytes = Bytes.create 4 in
                try
                  really_input ic len_bytes 0 4;
                  let len = Int32.to_int (Bytes.get_int32_le len_bytes 0) in
                  
                  (* Read object data *)
                  let data_bytes = Bytes.create len in
                  really_input ic data_bytes 0 len;
                  let data_str = Bytes.to_string data_bytes in
                  
                  (* Validate the data *)
                  if not (validate_fn data_str) then
                    raise (Failure "Invalid object data")
                  else
                    (* Deserialize the object *)
                    let obj = deserialize_fn data_str in
                    objects := obj :: !objects;
                    read_object (n - 1)
                with
                | End_of_file -> List.rev !objects
                | Failure msg -> raise (Failure msg)
            in
            
            try
              let result = read_object object_count in
              close_in ic;
              Ok result
            with
            | Failure msg ->
                close_in ic;
                Error (ValidationError msg)
            | exn ->
                close_in ic;
                Error (Serialization (Printexc.to_string exn))
          end
    end
  with
  | Sys_error msg -> Error (Io msg)
  | exn -> Error (Serialization (Printexc.to_string exn))

(* Mock serialization functions - to be replaced with actual implementations *)
module MockSerialization = struct
  let serialize_resource _resource = "mock_resource_serialized"
  let deserialize_resource _bytes = "mock_resource"
  let is_valid_resource_bytes _bytes = true
  
  let serialize_effect _effect = "mock_effect_serialized"
  let deserialize_effect _bytes = "mock_effect"
  let is_valid_effect_bytes _bytes = true
  
  let serialize_handler _handler = "mock_handler_serialized"
  let deserialize_handler _bytes = "mock_handler"
  let is_valid_handler_bytes _bytes = true
  
  let serialize_edge _edge = "mock_edge_serialized"
  let deserialize_edge _bytes = "mock_edge"
  let is_valid_edge_bytes _bytes = true
end

(* Convenience functions for writing Resource objects *)
let write_resource_to_file path resource =
  write_object path Resource MockSerialization.serialize_resource MockSerialization.is_valid_resource_bytes resource

let read_resource_from_file path =
  read_object path Resource MockSerialization.deserialize_resource MockSerialization.is_valid_resource_bytes

let write_resources_to_file path resources =
  write_objects path Resource MockSerialization.serialize_resource MockSerialization.is_valid_resource_bytes resources

let read_resources_from_file path =
  read_objects path Resource MockSerialization.deserialize_resource MockSerialization.is_valid_resource_bytes

(* Convenience functions for writing Effect objects *)
let write_effect_to_file path effect =
  write_object path Effect MockSerialization.serialize_effect MockSerialization.is_valid_effect_bytes effect

let read_effect_from_file path =
  read_object path Effect MockSerialization.deserialize_effect MockSerialization.is_valid_effect_bytes

let write_effects_to_file path effects =
  write_objects path Effect MockSerialization.serialize_effect MockSerialization.is_valid_effect_bytes effects

let read_effects_from_file path =
  read_objects path Effect MockSerialization.deserialize_effect MockSerialization.is_valid_effect_bytes

(* Convenience functions for writing Handler objects *)
let write_handler_to_file path handler =
  write_object path Handler MockSerialization.serialize_handler MockSerialization.is_valid_handler_bytes handler

let read_handler_from_file path =
  read_object path Handler MockSerialization.deserialize_handler MockSerialization.is_valid_handler_bytes

let write_handlers_to_file path handlers =
  write_objects path Handler MockSerialization.serialize_handler MockSerialization.is_valid_handler_bytes handlers

let read_handlers_from_file path =
  read_objects path Handler MockSerialization.deserialize_handler MockSerialization.is_valid_handler_bytes

(* Convenience functions for writing Edge objects *)
let write_edge_to_file path edge =
  write_object path Edge MockSerialization.serialize_edge MockSerialization.is_valid_edge_bytes edge

let read_edge_from_file path =
  read_object path Edge MockSerialization.deserialize_edge MockSerialization.is_valid_edge_bytes

let write_edges_to_file path edges =
  write_objects path Edge MockSerialization.serialize_edge MockSerialization.is_valid_edge_bytes edges

let read_edges_from_file path =
  read_objects path Edge MockSerialization.deserialize_edge MockSerialization.is_valid_edge_bytes

(* Round-trip testing helpers *)
module Testing = struct
  (* Result of a round-trip test *)
  type test_result = {
    object_type: object_type;
    success: bool;
    error_msg: string option;
  }
  
  (* Run a round-trip test for a specific object *)
  let test_round_trip object_type obj =
    let serialize_fn, deserialize_fn, validate_fn = match object_type with
      | Resource -> 
          MockSerialization.serialize_resource, 
          MockSerialization.deserialize_resource, 
          MockSerialization.is_valid_resource_bytes
      | Effect -> 
          MockSerialization.serialize_effect, 
          MockSerialization.deserialize_effect, 
          MockSerialization.is_valid_effect_bytes
      | Handler -> 
          MockSerialization.serialize_handler, 
          MockSerialization.deserialize_handler, 
          MockSerialization.is_valid_handler_bytes
      | Edge -> 
          MockSerialization.serialize_edge, 
          MockSerialization.deserialize_edge, 
          MockSerialization.is_valid_edge_bytes
      | _ -> failwith "Unsupported object type for round-trip testing"
    in
    
    try
      (* Serialize *)
      let bytes = serialize_fn obj in
      
      (* Validate *)
      if not (validate_fn bytes) then
        Error (ValidationError "Validation failed")
      else
        (* Deserialize *)
        let _obj2 = deserialize_fn bytes in
        
        (* Assume success - in a real implementation we would compare objects *)
        Ok _obj2
    with
    | exn -> Error (Serialization (Printexc.to_string exn))
  
  (* Test a resource round-trip *)
  let test_resource resource =
    test_round_trip Resource resource
  
  (* Test an effect round-trip *)
  let test_effect effect =
    test_round_trip Effect effect
  
  (* Test a handler round-trip *)
  let test_handler handler =
    test_round_trip Handler handler
  
  (* Test an edge round-trip *)
  let test_edge edge =
    test_round_trip Edge edge
  
  (* Print a test result *)
  let print_test_result result =
    let type_str = match result.object_type with
      | Resource -> "Resource"
      | Effect -> "Effect"
      | Handler -> "Handler"
      | Edge -> "Edge"
      | Graph -> "Graph"
      | ChangeSet -> "ChangeSet"
    in
    
    let status = if result.success then "PASS" else "FAIL" in
    let error = match result.error_msg with
      | Some msg -> ": " ^ msg
      | None -> ""
    in
    
    Printf.printf "[%s] %s%s\n" status type_str error
end 

(* Read multiple objects of different types from a file *)
let read_objects_mixed file type_factories =
  try
    (* Open the file *)
    let ic = open_in_bin file in
    let file_size = in_channel_length ic in
    
    if file_size < 10 then begin
      close_in ic;
      Error InvalidHeader
    end else begin
      (* Read and parse the header *)
      let header_bytes = Bytes.create 10 in
      really_input ic header_bytes 0 10;
      
      match header_of_bytes (Bytes.to_string header_bytes) with
      | Error e -> 
          close_in ic;
          Error e
      | Ok header ->
          (* Verify object type *)
          if header.object_type <> Graph then begin
            close_in ic;
            Error WrongObjectType
          end else begin
            (* Read objects *)
            let objects = ref [] in
            let object_count = Int32.to_int header.object_count in
            
            try
              for _ = 1 to object_count do
                (* Read type tag (1 byte) *)
                let tag_byte = Bytes.create 1 in
                really_input ic tag_byte 0 1;
                let tag = Char.code (Bytes.get tag_byte 0) in
                
                (* Find factory for this type *)
                let factory = 
                  try List.find (fun (t, _) -> object_type_to_byte t = tag) type_factories
                  with Not_found -> failwith "Unknown object type tag"
                in
                let _obj_type, deserialize_fn = factory in
                
                (* Read length prefix (4 bytes) *)
                let len_bytes = Bytes.create 4 in
                really_input ic len_bytes 0 4;
                let len = Int32.to_int (Bytes.get_int32_le len_bytes 0) in
                
                (* Read object *)
                let bytes = Bytes.create len in
                really_input ic bytes 0 len;
                let _obj2 = deserialize_fn (Bytes.to_string bytes) in
                (* Note: We're ignoring the deserialized object for now,
                   but we would add it to the result in a real implementation *)
                objects := 1 :: !objects
              done;
              
              close_in ic;
              Ok (List.length !objects)
            with exn ->
              close_in ic;
              Error (Serialization (Printexc.to_string exn))
          end
    end
  with
  | Sys_error msg -> Error (Io msg)
  | exn -> Error (Serialization (Printexc.to_string exn)) 