(** Layer 1: Linear Lambda Calculus - Unified Type System

    This module provides the foundation for the linear lambda calculus with
    a unified type system that seamlessly integrates structured types, session 
    types, and location awareness. *)

(** {1 Location System} *)

(** Location information for distributed computation *)
type location =
  | Local                    (** Local computation *)
  | Remote of string        (** Specific remote location *)
  | Domain of string        (** Logical domain *)
  | Any                     (** Location-polymorphic *)
[@@deriving show, eq]

(** {1 Linearity Phantom Types} *)

type linear = [ `Linear ]
(** Phantom type for linear resources (exactly-once use) *)

type affine = [ `Affine ]
(** Phantom type for affine resources (at-most-once use) *)

type relevant = [ `Relevant ]
(** Phantom type for relevant resources (at-least-once use) *)

type unrestricted = [ `Unrestricted ]
(** Phantom type for unrestricted resources (arbitrary use) *)

(** {1 Base Types} *)

(** Primitive base types *)
type base_type = Unit | Bool | Int | Symbol [@@deriving show, eq]

(** {1 Session Types} *)

(** Session types for type-safe communication protocols *)
type session_type =
  | Send of type_inner * session_type     (** Send a value, continue with protocol *)
  | Receive of type_inner * session_type  (** Receive a value, continue with protocol *)
  | InternalChoice of (string * session_type) list  (** We choose *)
  | ExternalChoice of (string * session_type) list  (** Other party chooses *)
  | End                                    (** End of communication *)
  | Recursive of string * session_type     (** Recursive protocols *)
  | Variable of string                     (** Session variable *)

(** Unified type expressions with linearity and location awareness *)
and type_inner =
  | Base of base_type                               (** Base primitive types *)
  | Product of type_inner * type_inner             (** Linear product type (τ₁ ⊗ τ₂) *)
  | Sum of type_inner * type_inner                 (** Sum type (τ₁ ⊕ τ₂) *)
  | LinearFunction of type_inner * type_inner      (** Linear function type (τ₁ ⊸ τ₂) *)
  | Record of record_type                           (** Record type with location-aware row polymorphism *)
  | Session of session_type                         (** Session type - communication protocols *)
  | Transform of {
      input : type_inner;
      output : type_inner;
      location : location;
    }
  | Located of type_inner * location                (** Type with location annotation *)

and record_type = {
    fields : field_type list;
    extension : row_variable option;  (** Optional row variable for polymorphism *)
}

and field_type = {
    name : string;
    ty : type_inner;
    location : location option;       (** Location constraint for the field *)
    access : field_access;            (** Access permissions *)
}

and field_access = Read | Write | ReadWrite

and row_variable = string
[@@deriving show, eq]

(** {1 Linearity-Tracked Types} *)

type 'linearity typed = { inner : type_inner; linearity : 'linearity }
[@@deriving show, eq]
(** Type with linearity phantom parameter *)

type linear_type = linear typed
(** Specific linearity type aliases *)

type affine_type = affine typed
type relevant_type = relevant typed
type unrestricted_type = unrestricted typed

(** {1 Value Types} *)

(** Runtime values in the lambda calculus *)
type value =
  | Unit
  | Bool of bool
  | Int of int32
  | Symbol of string
  | Product of value * value
  | Sum of { tag : int; value : value }
  | Record of { fields : (string * value) list }
  | Closure of { params : string list; body : term; env : (string * value) list }
  | ResourceRef of bytes
  | SessionChannel of { protocol : session_type; state : channel_state }

(** Channel state for session types *)
and channel_state = 
  | Active of session_type  (** Current protocol state *)
  | Closed                  (** Channel has been closed *)

(** Lambda calculus terms - extended with unified operations *)
and term =
  (* Core values and variables *)
  | Const of value
  | Var of string
  | Let of string * term * term
  (* Unit type operations *)
  | UnitVal
  | LetUnit of term * term
  (* Tensor product operations *)
  | Tensor of term * term
  | LetTensor of string * string * term * term
  (* Sum type operations *)
  | Inl of term
  | Inr of term
  | Case of term * string * term * string * term
  (* Linear function operations *)
  | Lambda of string list * term
  | Apply of term * term list
  (* Resource management *)
  | Alloc of term
  | Consume of term
  (* Record operations - location-aware *)
  | RecordCreate of (string * term) list
  | RecordProject of term * string * location option
  | RecordExtend of term * string * term
  | RecordUpdate of term * string * term * location option
  (* Session operations *)
  | SessionNew of session_type * string  (** Create new session with role *)
  | SessionSend of term * term           (** Send on channel *)
  | SessionReceive of term               (** Receive from channel *)
  | SessionClose of term                 (** Close session channel *)
  (* Location operations *)
  | AtLocation of term * location        (** Execute at specific location *)
  | Migrate of term * location * location (** Migrate data between locations *)
[@@deriving show, eq]

(** {1 Value Operations} *)

module Value = struct
  (** Get type of a value *)
  let rec get_type (value : value) : type_inner =
    match value with
    | Unit -> Base Unit
    | Bool _ -> Base Bool
    | Int _ -> Base Int
    | Symbol _ -> Base Symbol
    | Product (v1, v2) -> Product (get_type v1, get_type v2)
    | Sum { tag = _; value } ->
        Sum (get_type value, get_type value) (* Simplified *)
    | Record { fields } ->
        let field_types =
          List.map (fun (name, v) -> {
            name;
            ty = get_type v;
            location = None;
            access = ReadWrite;
          }) fields
        in
        Record { fields = field_types; extension = None }
    | Closure _ -> LinearFunction (Base Unit, Base Unit) (* Placeholder *)
    | ResourceRef _ -> Base Symbol (* Simplified *)
    | SessionChannel { protocol; _ } -> Session protocol

  (** Create product value *)
  let product (v1 : value) (v2 : value) : value = Product (v1, v2)

  (** Create sum value *)
  let sum (tag : int) (value : value) : value = Sum { tag; value }

  (** Create record value *)
  let record (fields : (string * value) list) : value = Record { fields }

  (** Project field from record *)
  let project_field (value : value) (field_name : string) : value option =
    match value with
    | Record { fields } -> List.assoc_opt field_name fields
    | _ -> None
end

(** {1 Linearity Constraints} *)

module Linearity = struct
  (** Linearity constraint types *)
  type constraint_type =
    | SingleUse (* Must be used exactly once *)
    | Droppable (* Can be discarded *)
    | Copyable (* Can be duplicated *)
    | MustUse (* Must be used at least once *)
  [@@deriving show, eq]

  (** Check if a linearity satisfies a constraint *)
  let check_constraint (_linearity : 'a) (constraint_type : constraint_type) :
      bool =
    match constraint_type with
    | SingleUse -> true (* All types can be used once *)
    | Droppable -> true (* Simplified - in reality depends on linearity *)
    | Copyable -> true (* Simplified - in reality depends on linearity *)
    | MustUse -> true (* Simplified - in reality depends on linearity *)

  (** Create linear resource type *)
  let linear_resource (inner_type : type_inner) : linear_type =
    { inner = inner_type; linearity = `Linear }

  (** Create affine resource type *)
  let affine_resource (inner_type : type_inner) : affine_type =
    { inner = inner_type; linearity = `Affine }

  (** Create relevant resource type *)
  let relevant_resource (inner_type : type_inner) : relevant_type =
    { inner = inner_type; linearity = `Relevant }

  (** Create unrestricted resource type *)
  let unrestricted_resource (inner_type : type_inner) : unrestricted_type =
    { inner = inner_type; linearity = `Unrestricted }
end

(** {1 Linear Resource Wrapper} *)

module LinearResource = struct
  type 'a t = { value : 'a; consumed : bool }
  (** Linear resource container *)

  (** Create a linear resource *)
  let create (value : 'a) : 'a t = { value; consumed = false }

  (** Consume a linear resource (can only be done once) *)
  let consume (resource : 'a t) : 'a =
    if resource.consumed then failwith "Linear resource already consumed"
    else resource.value

  (** Check if resource is consumed *)
  let is_consumed (resource : 'a t) : bool = resource.consumed

  (** Mark resource as consumed (for internal use) *)
  let mark_consumed (resource : 'a t) : 'a t = { resource with consumed = true }
end
