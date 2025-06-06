(** Unified error handling system for the Causality framework
    
    This module provides a consistent error handling system across all layers
    of the Causality framework, with proper error propagation and debugging support.
*)

(** {1 Error Types} *)

(** Base error kinds for all Causality operations *)
type error_kind = 
  | TypeError of string
  | MachineError of string  
  | LinearityError of string
  | CapabilityError of string
  | ReductionError of string
  | SynthesisError of string
  | ValidationError of string
  | SerializationError of string
  | NetworkError of string
  [@@deriving show, eq]

(** Causality exception for unrecoverable errors *)
exception Causality_error of error_kind * string

(** {1 Result Type} *)

(** Result type for error handling *)
type ('a, 'e) result = ('a, 'e) Result.t

(** {1 Helper Functions} *)

(** Create an error result *)
let error (kind : error_kind) (_msg : string) : ('a, error_kind) result =
  Error kind

(** Create a success result *)
let ok (value : 'a) : ('a, error_kind) result =
  Ok value

(** Convert error to exception (for unrecoverable errors) *)
let fail (kind : error_kind) (msg : string) : 'a =
  raise (Causality_error (kind, msg))

(** {1 Monadic Operations} *)

module Result = struct
  let bind (result : ('a, 'e) result) (f : 'a -> ('b, 'e) result) : ('b, 'e) result =
    match result with
    | Ok value -> f value
    | Error err -> Error err

  let map (f : 'a -> 'b) (result : ('a, 'e) result) : ('b, 'e) result =
    match result with
    | Ok value -> Ok (f value)
    | Error err -> Error err

  let (>>=) = bind
  let (>>|) result f = map f result

  let return = ok

  let fail_with err = Error err

  (** Combine multiple results *)
  let all (results : ('a, 'e) result list) : ('a list, 'e) result =
    let rec aux acc = function
      | [] -> Ok (List.rev acc)
      | Ok x :: rest -> aux (x :: acc) rest
      | Error e :: _ -> Error e
    in
    aux [] results
end

(** {1 Error Handling Utilities} *)

(** Pretty print error for display *)
let string_of_error_kind = function
  | TypeError msg -> Printf.sprintf "Type Error: %s" msg
  | MachineError msg -> Printf.sprintf "Machine Error: %s" msg
  | LinearityError msg -> Printf.sprintf "Linearity Error: %s" msg
  | CapabilityError msg -> Printf.sprintf "Capability Error: %s" msg
  | ReductionError msg -> Printf.sprintf "Reduction Error: %s" msg
  | SynthesisError msg -> Printf.sprintf "Synthesis Error: %s" msg
  | ValidationError msg -> Printf.sprintf "Validation Error: %s" msg
  | SerializationError msg -> Printf.sprintf "Serialization Error: %s" msg
  | NetworkError msg -> Printf.sprintf "Network Error: %s" msg

(** Log error (placeholder for now) *)
let log_error (kind : error_kind) (msg : string) : unit =
  Printf.eprintf "[ERROR] %s: %s\n" (string_of_error_kind kind) msg;
  flush stderr

(** Handle result by logging errors *)
let handle_result ?(log = true) (result : ('a, error_kind) result) : 'a option =
  match result with
  | Ok value -> Some value
  | Error kind ->
    if log then log_error kind "";
    None 