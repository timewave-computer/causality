(* Value system for the Causality register machine with content addressing *)

open Causality_system.System_content_addressing

(* Universal content identifier - all data identified by content hash *)
type entity_id = Causality_system.System_content_addressing.EntityId.t

(* Core primitive values (small, embedded directly) *)
type core_value =
  | Unit
  | Bool of bool
  | Int of int
  | Symbol of string

(* Content-addressed machine values *)
type machine_value =
  | Primitive of core_value                    (* Direct primitive values *)
  | ResourceRef of entity_id                   (* Reference to linear resource *)
  | ExprRef of entity_id                       (* Reference to Layer 1 expression *)
  | EffectRef of entity_id                     (* Reference to Layer 2 effect *)
  | ValueRef of entity_id                      (* Reference to computed value *)

(* Linearity tracking *)
type linearity =
  | Linear       (* Must be used exactly once *)
  | Affine       (* Must be used at most once *)  
  | Relevant     (* Must be used at least once *)
  | Unrestricted (* Can be used any number of times *)

(* Value metadata *)
type value_metadata = {
  consumed : bool;
  created_at : float;
  access_count : int;
}

(* Register values with linearity tracking *)
type register_value = {
  value : machine_value;
  linearity : linearity;
  metadata : value_metadata;
}

(* Pattern matching on machine values *)
type value_pattern =
  | PWildcard
  | PPrimitive of core_value
  | PResourceRef of entity_id option
  | PExprRef of entity_id option
  | PEffectRef of entity_id option
  | PValueRef of entity_id option

(* Built-in functions *)
type builtin_function =
  | Print
  | Add
  | Subtract
  | Multiply
  | Equal
  | LessThan
  | ResolveRef
  | ComputeHash

(* Helper functions for core values *)
module Core_value = struct
  let equal a b =
    match a, b with
    | Unit, Unit -> true
    | Bool a, Bool b -> Bool.equal a b
    | Int a, Int b -> Int.equal a b
    | Symbol a, Symbol b -> String.equal a b
    | _ -> false

  let to_string = function
    | Unit -> "()"
    | Bool b -> Bool.to_string b
    | Int i -> Int.to_string i
    | Symbol s -> s
end

(* Helper functions for machine values *)
module MachineValue = struct
  let is_primitive = function
    | Primitive _ -> true
    | _ -> false

  let is_reference = function
    | ResourceRef _ | ExprRef _ | EffectRef _ | ValueRef _ -> true
    | Primitive _ -> false

  let get_entity_id = function
    | ResourceRef id | ExprRef id | EffectRef id | ValueRef id -> Some id
    | Primitive _ -> None

  let from_core_value cv = Primitive cv

  let from_entity_id id ref_type =
    match ref_type with
    | `Resource -> ResourceRef id
    | `Expr -> ExprRef id
    | `Effect -> EffectRef id
    | `Value -> ValueRef id

  let create_ref data ref_type =
    let id = EntityId.from_content data in
    from_entity_id id ref_type
end

(* Helper functions for linearity *)
module Linearity = struct
  let to_string = function
    | Linear -> "linear"
    | Affine -> "affine"
    | Relevant -> "relevant"
    | Unrestricted -> "unrestricted"
end

(* Helper functions for metadata *)
module Value_metadata = struct
  let empty = {
    consumed = false;
    created_at = Unix.time ();
    access_count = 0;
  }

  let increment_access metadata =
    { metadata with access_count = metadata.access_count + 1 }
end

(* Helper functions for register values *)
module RegisterValue = struct
  let create value linearity =
    { value; linearity; metadata = Value_metadata.empty }

  let create_linear value =
    create value Linear

  let create_affine value =
    create value Affine

  let create_unrestricted value =
    create value Unrestricted

  let is_usable reg_val =
    match reg_val.linearity with
    | Linear -> not reg_val.metadata.consumed
    | Affine -> not reg_val.metadata.consumed
    | Relevant -> true
    | Unrestricted -> true

  let consume reg_val =
    { reg_val with metadata = { reg_val.metadata with consumed = true } }

  let extract reg_val = reg_val.value

  let get_entity_id reg_val = MachineValue.get_entity_id reg_val.value
end

(* Pattern matching utilities *)
module Pattern = struct
  let matches pattern value =
    match pattern, value with
    | PWildcard, _ -> true
    | PPrimitive p, Primitive v -> Core_value.equal p v
    | PResourceRef None, ResourceRef _ -> true
    | PResourceRef (Some id), ResourceRef vid -> EntityId.equal id vid
    | PExprRef None, ExprRef _ -> true
    | PExprRef (Some id), ExprRef vid -> EntityId.equal id vid
    | PEffectRef None, EffectRef _ -> true
    | PEffectRef (Some id), EffectRef vid -> EntityId.equal id vid
    | PValueRef None, ValueRef _ -> true
    | PValueRef (Some id), ValueRef vid -> EntityId.equal id vid
    | _ -> false
end

(* Built-in function implementations *)
module Builtin = struct
  let apply_builtin func args =
    match func, args with
    | Print, [Primitive (Symbol s)] -> 
        Printf.printf "%s\n" s; Ok (Primitive Unit)
    | Print, [Primitive (Int i)] -> 
        Printf.printf "%d\n" i; Ok (Primitive Unit)
    | Print, [Primitive (Bool b)] -> 
        Printf.printf "%b\n" b; Ok (Primitive Unit)
    | Add, [Primitive (Int a); Primitive (Int b)] -> 
        Ok (Primitive (Int (a + b)))
    | Subtract, [Primitive (Int a); Primitive (Int b)] -> 
        Ok (Primitive (Int (a - b)))
    | Multiply, [Primitive (Int a); Primitive (Int b)] -> 
        Ok (Primitive (Int (a * b)))
    | Equal, [Primitive a; Primitive b] -> 
        Ok (Primitive (Bool (Core_value.equal a b)))
    | LessThan, [Primitive (Int a); Primitive (Int b)] -> 
        Ok (Primitive (Bool (a < b)))
    | ComputeHash, [value] ->
        let id = EntityId.from_content (Marshal.to_string value []) in
        Ok (Primitive (Symbol (EntityId.to_hex id)))
    | _ -> Error "Invalid builtin function application"
end

(* Pretty printing *)
module Pretty = struct
  let machine_value_to_string = function
    | Primitive cv -> Core_value.to_string cv
    | ResourceRef id -> 
        Printf.sprintf "ResourceRef(%s)" (EntityId.to_hex id)
    | ExprRef id -> 
        Printf.sprintf "ExprRef(%s)" (EntityId.to_hex id)
    | EffectRef id -> 
        Printf.sprintf "EffectRef(%s)" (EntityId.to_hex id)
    | ValueRef id -> 
        Printf.sprintf "ValueRef(%s)" (EntityId.to_hex id)

  let register_value_to_string reg_val =
    Printf.sprintf "%s [%s]" 
      (machine_value_to_string reg_val.value)
      (Linearity.to_string reg_val.linearity)
end 