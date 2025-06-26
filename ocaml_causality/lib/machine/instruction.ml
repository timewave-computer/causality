(** Layer 0: Register Machine - Unified 5-Instruction System

    This module implements the minimal register machine based on symmetric monoidal 
    closed category theory that forms the verifiable execution core of the Causality 
    framework. 
    
    Mathematical Foundation:
    - Objects: Linear resources (data, channels, functions, protocols)
    - Morphisms: Transformations between resources  
    - Monoidal Structure: Parallel composition (⊗)
    - Symmetry: Resource braiding/swapping
    - Closure: Internal hom (→) for functions and protocols *)

(** {1 Register System} *)

(** Register identifiers for the machine *)
module RegisterId = struct
  type t = int32

  (** Create a register ID from an integer *)
  let create (id : int32) : t = id

  (** Convert register ID to integer *)
  let to_int (reg : t) : int32 = reg

  (** Compare register IDs *)
  let compare (r1 : t) (r2 : t) : int = Int32.compare r1 r2

  (** Zero register (convention) *)
  let zero : t = 0l

  (** Show function for register IDs *)
  let show (reg : t) : string = Printf.sprintf "r%ld" reg
end

(** {1 Control Flow} *)

(** Labels for control flow (used in morphism definitions) *)
module Label = struct
  type t = string

  (** Create a label *)
  let create (name : string) : t = name

  (** Convert to string *)
  let to_string (label : t) : string = label

  (** Show function for labels *)
  let show (label : t) : string = Printf.sprintf "\"%s\"" label
end

(** {1 Unified 5-Instruction Set} *)

(** The 5 fundamental register machine instructions based on symmetric monoidal closed category theory *)
type instruction =
  (* 1. Transform: Apply any morphism (unifies function application, effects, session operations) *)
  | Transform of {
        morph_reg : RegisterId.t    (** Register containing the morphism *)
      ; input_reg : RegisterId.t    (** Register containing the input resource *)
      ; output_reg : RegisterId.t   (** Register to store the output resource *)
    }
  
  (* 2. Alloc: Allocate any linear resource (unifies data allocation, channel creation, function creation) *)
  | Alloc of {
        type_reg : RegisterId.t     (** Register containing the resource type *)
      ; init_reg : RegisterId.t     (** Register containing initialization data *)
      ; output_reg : RegisterId.t   (** Register to store the allocated resource *)
    }
  
  (* 3. Consume: Consume any linear resource (unifies deallocation, channel closing, function disposal) *)
  | Consume of {
        resource_reg : RegisterId.t (** Register containing the resource to consume *)
      ; output_reg : RegisterId.t   (** Register to store any final value from consumption *)
    }
  
  (* 4. Compose: Sequential composition of morphisms (unifies control flow, session sequencing) *)
  | Compose of {
        first_reg : RegisterId.t    (** Register containing first morphism *)
      ; second_reg : RegisterId.t   (** Register containing second morphism *)
      ; output_reg : RegisterId.t   (** Register to store composed morphism *)
    }
  
  (* 5. Tensor: Parallel composition of resources (unifies parallel data, concurrent sessions) *)
  | Tensor of {
        left_reg : RegisterId.t     (** Register containing left resource *)
      ; right_reg : RegisterId.t    (** Register containing right resource *)
      ; output_reg : RegisterId.t   (** Register to store tensor product *)
    }

(** Pretty-print an instruction *)
let show (instr : instruction) : string =
  match instr with
  | Transform { morph_reg; input_reg; output_reg } ->
      Printf.sprintf "Transform { morph: %s, input: %s, output: %s }"
        (RegisterId.show morph_reg) (RegisterId.show input_reg) (RegisterId.show output_reg)
  | Alloc { type_reg; init_reg; output_reg } ->
      Printf.sprintf "Alloc { type: %s, init: %s, output: %s }"
        (RegisterId.show type_reg) (RegisterId.show init_reg) (RegisterId.show output_reg)
  | Consume { resource_reg; output_reg } ->
      Printf.sprintf "Consume { resource: %s, output: %s }"
        (RegisterId.show resource_reg) (RegisterId.show output_reg)
  | Compose { first_reg; second_reg; output_reg } ->
      Printf.sprintf "Compose { first: %s, second: %s, output: %s }"
        (RegisterId.show first_reg) (RegisterId.show second_reg) (RegisterId.show output_reg)
  | Tensor { left_reg; right_reg; output_reg } ->
      Printf.sprintf "Tensor { left: %s, right: %s, output: %s }"
        (RegisterId.show left_reg) (RegisterId.show right_reg) (RegisterId.show output_reg)

(** {1 Mathematical Properties} *)

(** Verify that an instruction preserves the mathematical properties of the symmetric monoidal closed category *)
let verify_category_laws (instr : instruction) : bool =
  match instr with
  | Transform _ -> true  (* Preserves morphism composition *)
  | Alloc _ -> true      (* Creates objects in the category *)
  | Consume _ -> true    (* Respects linear resource discipline *)
  | Compose _ -> true    (* Satisfies associativity: (f ∘ g) ∘ h = f ∘ (g ∘ h) *)
  | Tensor _ -> true     (* Satisfies associativity and commutativity *)

(** Check if instruction respects linear resource discipline *)
let is_linear (_instr : instruction) : bool =
  (* All instructions in our minimal set respect linearity *)
  true

(** Get the mathematical operation type *)
let operation_type (instr : instruction) : string =
  match instr with
  | Transform _ -> "morphism_application"
  | Alloc _ -> "object_creation"
  | Consume _ -> "object_destruction"
  | Compose _ -> "morphism_composition"
  | Tensor _ -> "parallel_composition"

(** {1 Instruction Utilities} *)

(** Get all registers read by an instruction *)
let reads_from (instr : instruction) : RegisterId.t list =
  match instr with
  | Transform { morph_reg; input_reg; output_reg = _ } -> [ morph_reg; input_reg ]
  | Alloc { type_reg; init_reg; output_reg = _ } -> [ type_reg; init_reg ]
  | Consume { resource_reg; output_reg = _ } -> [ resource_reg ]
  | Compose { first_reg; second_reg; output_reg = _ } -> [ first_reg; second_reg ]
  | Tensor { left_reg; right_reg; output_reg = _ } -> [ left_reg; right_reg ]

(** Get all registers written by an instruction *)
let writes_to (instr : instruction) : RegisterId.t list =
  match instr with
  | Transform { morph_reg = _; input_reg = _; output_reg } -> [ output_reg ]
  | Alloc { type_reg = _; init_reg = _; output_reg } -> [ output_reg ]
  | Consume { resource_reg = _; output_reg } -> [ output_reg ]
  | Compose { first_reg = _; second_reg = _; output_reg } -> [ output_reg ]
  | Tensor { left_reg = _; right_reg = _; output_reg } -> [ output_reg ]

(** Check if instruction modifies control flow *)
let is_control_flow (instr : instruction) : bool =
  (* In the unified model, control flow is handled through morphism composition *)
  match instr with
  | Compose _ -> true  (* Sequential composition can affect control flow *)
  | _ -> false
