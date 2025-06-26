(* ZK-compatible fixed point arithmetic interface *)

(* Type for fixed point numbers *)
type t

(* Constants *)
val zero : t
val one : t
val scale : t

(* Conversion functions *)
val of_int : int -> t
val of_string : string -> t
val of_rational : int -> int -> t
val to_int : t -> int
val to_string : t -> string

(* Basic arithmetic *)
val add : t -> t -> t
val sub : t -> t -> t
val mul : t -> t -> t
val div : t -> t -> t
val neg : t -> t
val abs : t -> t

(* Comparison *)
val equal : t -> t -> bool
val compare : t -> t -> int
val lt : t -> t -> bool
val le : t -> t -> bool
val gt : t -> t -> bool
val ge : t -> t -> bool

(* Min/max *)
val min : t -> t -> t
val max : t -> t -> t

(* Advanced operations *)
val pow : t -> int -> t
val sqrt : t -> t
val inverse : t -> t
val modulo : t -> int -> t

(* Utility functions *)
val is_zero : t -> bool
val is_one : t -> bool
val is_valid_for_zk : t -> bool

(* Common fractions *)
val half : t
val quarter : t
val tenth : t

(* Percentage operations *)
val percent : int -> t
val apply_percent : t -> int -> t
val basis_points : int -> t
val apply_basis_points : t -> int -> t

(* Serialization *)
val to_bytes : t -> bytes
val of_bytes : bytes -> t
val hash : t -> int

(* Testing *)
val random : unit -> t

(* Pretty printing *)
val pp : Format.formatter -> t -> unit 