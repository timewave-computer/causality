(** Layer 1: Linear Lambda Calculus Base Types

    This module provides the foundation for the linear lambda calculus,
    including linearity phantom types, base types, and value construction. *)

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

(** Core type expressions with linearity *)
type type_inner =
  | Base of base_type
  | Product of type_inner * type_inner
  | Sum of type_inner * type_inner
  | LinearFunction of type_inner * type_inner
  | Record of record_type
  | ResourceType of string

and record_type = {
    fields : (string * type_inner) list
  ; row_id : bytes option (* Content-addressed row type ID *)
}
[@@deriving show, eq]
(** Record type definition *)

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
  | Closure of { params : string list; body : term } (* Renamed from Lambda *)
  | ResourceRef of bytes

(** Lambda calculus terms *)
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
  (* Record operations *)
  | RecordCreate of (string * term) list
  | RecordProject of term * string
  | RecordExtend of term * string * term
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
          List.map (fun (name, v) -> (name, get_type v)) fields
        in
        Record { fields = field_types; row_id = None }
    | Closure _ -> LinearFunction (Base Unit, Base Unit) (* Placeholder *)
    | ResourceRef _ -> ResourceType "generic"

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
