(** Value System for the Causality Register Machine Interface *)

type entity_id = Causality_system.System_content_addressing.EntityId.t

type core_value = Unit | Bool of bool | Int of int | Symbol of string

type machine_value =
  | Primitive of core_value
  | ResourceRef of entity_id
  | ExprRef of entity_id
  | EffectRef of entity_id
  | ValueRef of entity_id

type linearity =
  | Linear
  | Affine
  | Relevant
  | Unrestricted

type value_metadata = {
    consumed : bool
  ; created_at : float
  ; access_count : int
}

type register_value = {
    value : machine_value
  ; linearity : linearity
  ; metadata : value_metadata
}

type value_pattern =
  | PWildcard
  | PPrimitive of core_value
  | PResourceRef of entity_id option
  | PExprRef of entity_id option
  | PEffectRef of entity_id option
  | PValueRef of entity_id option

type builtin_function =
  | Print
  | Add
  | Subtract
  | Multiply
  | Equal
  | LessThan
  | ResolveRef
  | ComputeHash

module Core_value : sig
  val equal : core_value -> core_value -> bool
  val to_string : core_value -> string
end

module MachineValue : sig
  val is_primitive : machine_value -> bool
  val is_reference : machine_value -> bool
  val get_entity_id : machine_value -> entity_id option
  val from_core_value : core_value -> machine_value
  val from_entity_id : entity_id -> [< `Resource | `Expr | `Effect | `Value ] -> machine_value
  val create_ref : 'a -> [< `Resource | `Expr | `Effect | `Value ] -> machine_value
end

module Linearity : sig
  val to_string : linearity -> string
end

module Value_metadata : sig
  val empty : value_metadata
  val increment_access : value_metadata -> value_metadata
end

module RegisterValue : sig
  val create : machine_value -> linearity -> register_value
  val create_linear : machine_value -> register_value
  val create_affine : machine_value -> register_value
  val create_unrestricted : machine_value -> register_value
  val is_usable : register_value -> bool
  val consume : register_value -> register_value
  val extract : register_value -> machine_value
  val get_entity_id : register_value -> entity_id option
end

module Pattern : sig
  val matches : value_pattern -> machine_value -> bool
end

module Builtin : sig
  val apply_builtin : builtin_function -> machine_value list -> (machine_value, string) result
end

module Pretty : sig
  val machine_value_to_string : machine_value -> string
  val register_value_to_string : register_value -> string
end
