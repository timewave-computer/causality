(** Layer 2: OCaml Native Effect System Integration

    This module provides true integration with OCaml 5.0+ native algebraic
    effects, enabling direct-style programming while maintaining linearity and
    verifiability. *)

(** {1 Linear Resource System} *)

(** Linear resource wrapper *)
module LinearResource = struct
  type 'a t = { mutable value : 'a; mutable consumed : bool }

  let create value = { value; consumed = false }

  let consume resource =
    if resource.consumed then failwith "Linear resource already consumed"
    else (
      resource.consumed <- true;
      resource.value)
end

(** {1 OCaml Native Algebraic Effects} *)

(** Causality effects using OCaml's native effect system *)
type _ Effect.t +=
  | LinearResource : 'a -> 'a LinearResource.t Effect.t
  | ConsumeResource : 'a LinearResource.t -> 'a Effect.t
  | AllocResource : unit -> bytes Effect.t
  | ConstraintCheck : bool -> bool Effect.t
  | ZKWitness : string -> string Effect.t

(** {1 Effect Operations - Direct Style} *)

(** Create a linear resource - direct style, no monads *)
let create_linear_resource (value : 'a) : 'a LinearResource.t =
  Effect.perform (LinearResource value)

(** Consume a linear resource - direct style *)
let consume_linear_resource (resource : 'a LinearResource.t) : 'a =
  Effect.perform (ConsumeResource resource)

(** Allocate a typed resource - direct style *)
let allocate_resource () : bytes = Effect.perform (AllocResource ())

(** Check a runtime constraint - direct style *)
let check_constraint (condition : bool) : bool =
  Effect.perform (ConstraintCheck condition)

(** Generate a zero-knowledge witness - direct style *)
let generate_zk_witness (term : string) : string =
  Effect.perform (ZKWitness term)

(** {1 Effect Handlers} *)

(** Combined effect handler for all Causality effects *)
let causality_handler =
  let open Effect.Deep in
  {
    retc = (fun x -> x)
  ; exnc = (fun e -> raise e)
  ; effc =
      (fun (type a) (eff : a Effect.t) ->
        match eff with
        | LinearResource value ->
            Some
              (fun (k : (a, _) continuation) ->
                let resource = LinearResource.create value in
                continue k resource)
        | ConsumeResource resource ->
            Some
              (fun (k : (a, _) continuation) ->
                try
                  let value = LinearResource.consume resource in
                  continue k value
                with Failure msg -> discontinue k (Failure msg))
        | AllocResource () ->
            Some
              (fun (k : (a, _) continuation) ->
                let resource_id = Bytes.create 32 in
                for i = 0 to 31 do
                  Bytes.set_uint8 resource_id i (Random.int 256)
                done;
                continue k resource_id)
        | ConstraintCheck condition ->
            Some (fun (k : (a, _) continuation) -> continue k condition)
        | ZKWitness term ->
            Some
              (fun (k : (a, _) continuation) ->
                let witness = "witness_" ^ term in
                continue k witness)
        | _ -> None)
  }

(** Run computation with Causality effect handling *)
let run_with_effects (computation : unit -> 'a) : ('a, exn) result =
  try Ok (Effect.Deep.match_with computation () causality_handler)
  with exn -> Error exn

(** {1 Linear Function Effects - Direct Style} *)

type 'a linear_function = 'a -> 'a
(** Linear function type - no monads needed *)

(** Apply linear function with automatic resource tracking - direct style *)
let apply_linear_function (f : 'a linear_function) (arg : 'a) : 'a =
  let linear_arg = create_linear_resource arg in
  let consumed_arg = consume_linear_resource linear_arg in
  f consumed_arg

(** {1 Effect Composition - Direct Style} *)

(** Sequence computations naturally using normal control flow *)
let sequence_computations (computations : (unit -> 'a) list) : 'a list =
  List.map (fun comp -> comp ()) computations

(** {1 Testing and Examples - Direct Style} *)

(** Example: Linear resource pipeline using direct style *)
let linear_pipeline_example () : int * bool * string =
  (* No monadic composition needed! *)
  let resource = create_linear_resource 42 in
  let type_ok = check_constraint true in
  let value = consume_linear_resource resource in
  let witness = generate_zk_witness "test_term" in
  (value, type_ok, witness)

(** Example: Effect composition using normal control flow *)
let effect_composition_example () : string option =
  try
    let resource = create_linear_resource "test" in
    let value = consume_linear_resource resource in
    Some value
  with Failure _ -> None

(** Example: Complex computation with natural control flow *)
let complex_computation_example (input : int) : int * string =
  (* This reads like normal imperative code! *)
  let resource1 = create_linear_resource input in
  let resource2 = create_linear_resource (input * 2) in

  let validation1 = check_constraint (input > 0) in
  let validation2 = check_constraint (input < 100) in

  if validation1 && validation2 then
    let value1 = consume_linear_resource resource1 in
    let value2 = consume_linear_resource resource2 in
    let witness = generate_zk_witness ("complex_" ^ string_of_int input) in
    (value1 + value2, witness)
  else failwith "Validation failed"

(** Example: Nested computations with exception handling *)
let nested_computation_example () : string =
  try
    let outer_resource = create_linear_resource "outer" in
    let witness1 = generate_zk_witness "level1" in

    let inner_result =
      let inner_resource = create_linear_resource "inner" in
      let witness2 = generate_zk_witness "level2" in
      let inner_value = consume_linear_resource inner_resource in
      inner_value ^ "_" ^ witness2
    in

    let outer_value = consume_linear_resource outer_resource in
    outer_value ^ "_" ^ witness1 ^ "_" ^ inner_result
  with Failure msg -> "Error: " ^ msg
