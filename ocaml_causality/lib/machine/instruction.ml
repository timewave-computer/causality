(** Layer 0: Register Machine Instructions

    This module implements the 11-instruction register machine that forms the
    verifiable execution core of the Causality framework. *)

(** {1 Register System} *)

(** Register identifiers for the machine *)
module RegisterId = struct
  type t = int32 [@@deriving show, eq]

  (** Create a register ID from an integer *)
  let create (id : int32) : t = id

  (** Convert register ID to integer *)
  let to_int (reg : t) : int32 = reg

  (** Compare register IDs *)
  let compare (r1 : t) (r2 : t) : int = Int32.compare r1 r2

  (** Zero register (convention) *)
  let zero : t = 0l
end

(** {1 Control Flow} *)

(** Labels for control flow *)
module Label = struct
  type t = string [@@deriving show, eq]

  (** Create a label *)
  let create (name : string) : t = name

  (** Convert to string *)
  let to_string (label : t) : string = label
end

(** {1 Constraint System} *)

(** Constraint expressions for runtime verification *)
type constraint_expr =
  | True
  | False
  | And of constraint_expr * constraint_expr
  | Or of constraint_expr * constraint_expr
  | Not of constraint_expr
  | Equal of RegisterId.t * RegisterId.t
  | LessThan of RegisterId.t * RegisterId.t
  | GreaterThan of RegisterId.t * RegisterId.t
  | HasType of RegisterId.t * string
  | IsConsumed of RegisterId.t
  | HasCapability of RegisterId.t * string
  | Predicate of string * RegisterId.t list
[@@deriving show, eq]

(** {1 Effect System} *)

(** Optimization hints for effect execution *)
type hint =
  | Parallel
  | Sequential
  | Domain of string
  | Priority of int32
  | Deadline of int64
  | Custom of string
[@@deriving show, eq]

type effect_call = {
    tag : string
  ; pre : constraint_expr
  ; post : constraint_expr
  ; hints : hint list
}
[@@deriving show, eq]
(** Effect calls in the register machine *)

(** {1 Core Instructions} *)

(** The 11 fundamental register machine instructions *)
type instruction =
  (* 1. Move: Copy value between registers *)
  | Move of { src : RegisterId.t; dst : RegisterId.t }
  (* 2. Apply: Function application *)
  | Apply of {
        fn_reg : RegisterId.t
      ; arg_reg : RegisterId.t
      ; out_reg : RegisterId.t
    }
  (* 3. Match: Sum type pattern matching *)
  | Match of {
        sum_reg : RegisterId.t
      ; left_reg : RegisterId.t
      ; right_reg : RegisterId.t
      ; left_label : string
      ; right_label : string
    }
  (* 4. Alloc: Resource allocation *)
  | Alloc of {
        type_reg : RegisterId.t
      ; val_reg : RegisterId.t
      ; out_reg : RegisterId.t
    }
  (* 5. Consume: Linear resource consumption *)
  | Consume of { resource_reg : RegisterId.t; out_reg : RegisterId.t }
  (* 6. Check: Runtime constraint verification *)
  | Check of { expr : constraint_expr }
  (* 7. Perform: Effect execution *)
  | Perform of { effect : effect_call; out_reg : RegisterId.t }
  (* 8. Select: Conditional selection *)
  | Select of {
        cond_reg : RegisterId.t
      ; true_reg : RegisterId.t
      ; false_reg : RegisterId.t
      ; out_reg : RegisterId.t
    }
  (* 9. Witness: Zero-knowledge witness generation *)
  | Witness of { out_reg : RegisterId.t }
  (* 10. LabelMarker: Control flow target *)
  | LabelMarker of string
  (* 11. Return: Function return *)
  | Return of { result_reg : RegisterId.t option }
[@@deriving show, eq]

(** {1 Instruction Utilities} *)

(** Get all registers read by an instruction *)
let reads_from (instr : instruction) : RegisterId.t list =
  match instr with
  | Move { src; dst = _ } -> [ src ]
  | Apply { fn_reg; arg_reg; out_reg = _ } -> [ fn_reg; arg_reg ]
  | Match
      { sum_reg; left_reg = _; right_reg = _; left_label = _; right_label = _ }
    ->
      [ sum_reg ]
  | Alloc { type_reg; val_reg; out_reg = _ } -> [ type_reg; val_reg ]
  | Consume { resource_reg; out_reg = _ } -> [ resource_reg ]
  | Check _ -> []
  | Perform _ -> []
  | Select { cond_reg; true_reg; false_reg; out_reg = _ } ->
      [ cond_reg; true_reg; false_reg ]
  | Witness _ -> []
  | LabelMarker _ -> []
  | Return { result_reg = Some reg } -> [ reg ]
  | Return { result_reg = None } -> []

(** Get all registers written by an instruction *)
let writes_to (instr : instruction) : RegisterId.t list =
  match instr with
  | Move { src = _; dst } -> [ dst ]
  | Apply { fn_reg = _; arg_reg = _; out_reg } -> [ out_reg ]
  | Match { sum_reg = _; left_reg; right_reg; left_label = _; right_label = _ }
    ->
      [ left_reg; right_reg ]
  | Alloc { type_reg = _; val_reg = _; out_reg } -> [ out_reg ]
  | Consume { resource_reg = _; out_reg } -> [ out_reg ]
  | Check _ -> []
  | Perform { effect = _; out_reg } -> [ out_reg ]
  | Select { cond_reg = _; true_reg = _; false_reg = _; out_reg } -> [ out_reg ]
  | Witness { out_reg } -> [ out_reg ]
  | LabelMarker _ -> []
  | Return _ -> []

(** Check if instruction is a control flow instruction *)
let is_control_flow (instr : instruction) : bool =
  match instr with Match _ | LabelMarker _ | Return _ -> true | _ -> false
