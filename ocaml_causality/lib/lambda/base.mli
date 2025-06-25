(** Layer 1: Linear Lambda Calculus - Unified Type System Interface *)

type location =
  | Local
  | Remote of string
  | Domain of string
  | Any

type linear = [ `Linear ]
type affine = [ `Affine ]
type relevant = [ `Relevant ]
type unrestricted = [ `Unrestricted ]

type base_type = Unit | Bool | Int | Symbol

type session_type =
  | Send of type_inner * session_type
  | Receive of type_inner * session_type
  | InternalChoice of (string * session_type) list
  | ExternalChoice of (string * session_type) list
  | End
  | Recursive of string * session_type
  | Variable of string

and type_inner =
  | Base of base_type
  | Product of type_inner * type_inner
  | Sum of type_inner * type_inner
  | LinearFunction of type_inner * type_inner
  | Record of record_type
  | Session of session_type
  | Transform of {
      input : type_inner;
      output : type_inner;
      location : location;
    }
  | Located of type_inner * location

and record_type = {
    fields : field_type list;
    extension : row_variable option;
}

and field_type = {
    name : string;
    ty : type_inner;
    location : location option;
    access : field_access;
}

and field_access = Read | Write | ReadWrite

and row_variable = string

type 'linearity typed = { inner : type_inner; linearity : 'linearity }

type linear_type = linear typed
type affine_type = affine typed
type relevant_type = relevant typed
type unrestricted_type = unrestricted typed

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

and channel_state = 
  | Active of session_type
  | Closed

and term =
  | Const of value
  | Var of string
  | Let of string * term * term
  | UnitVal
  | LetUnit of term * term
  | Tensor of term * term
  | LetTensor of string * string * term * term
  | Inl of term
  | Inr of term
  | Case of term * string * term * string * term
  | Lambda of string list * term
  | Apply of term * term list
  | Alloc of term
  | Consume of term
  | RecordCreate of (string * term) list
  | RecordProject of term * string * location option
  | RecordExtend of term * string * term
  | RecordUpdate of term * string * term * location option
  | SessionNew of session_type * string
  | SessionSend of term * term
  | SessionReceive of term
  | SessionClose of term
  | AtLocation of term * location
  | Migrate of term * location * location

module Value : sig
  val get_type : value -> type_inner
  val product : value -> value -> value
  val sum : int -> value -> value
  val record : (string * value) list -> value
  val project_field : value -> string -> value option
end

module Linearity : sig
  type constraint_type =
    | SingleUse
    | Droppable
    | Copyable
    | MustUse

  val check_constraint : 'linearity typed -> constraint_type -> bool
  val linear_resource : type_inner -> linear_type
  val affine_resource : type_inner -> affine_type
  val relevant_resource : type_inner -> relevant_type
  val unrestricted_resource : type_inner -> unrestricted_type
end

module LinearResource : sig
  type 'a t = { value : 'a; consumed : bool }

  val create : 'a -> 'a t
  val consume : 'a t -> 'a
  val is_consumed : 'a t -> bool
  val mark_consumed : 'a t -> 'a t
end
