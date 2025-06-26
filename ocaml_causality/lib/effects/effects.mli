(** OCaml Native Algebraic Effects for Causality Layer 2 Interface

    This module provides algebraic effects for linear resources, constraints, and ZK operations
    that integrate with the Causality Layer 2 system. *)

(** {1 Linear Resource Types} *)

(** Linear resource type - tracks consumption to enforce linearity *)
type 'a linear_resource = {
  value: 'a;
  consumed: bool ref;
}

(** Linear function type *)
type 'a linear_function = 'a -> 'a

(** {1 Effect Operations} *)

(** Resource allocation *)
val allocate_resource : unit -> bytes

(** Constraint checking *)
val check_constraint : bool -> bool

(** ZK witness generation *)
val generate_zk_witness : string -> string

(** Linear resource creation *)
val create_linear_resource : 'a -> 'a linear_resource

(** Linear resource consumption with linearity enforcement *)
val consume_linear_resource : 'a linear_resource -> 'a

(** Linear function application *)
val apply_linear_function : 'a linear_function -> 'a -> 'a

(** {1 Effect Runner} *)

(** Simple effect runner - executes computations with effect handling *)
val run_with_effects : (unit -> 'a) -> ('a, exn) result

(** {1 Example Functions} *)

(** Linear pipeline example demonstrating effect composition *)
val linear_pipeline_example : unit -> (int * bool * string)

(** Effect composition example *)
val effect_composition_example : unit -> string option

(** Complex computation example *)
val complex_computation_example : int -> (int * string)

(** Nested computation example *)
val nested_computation_example : unit -> string
