(** Layer 0: Register Machine - Unified 5-Instruction System Interface *)

module RegisterId : sig
  type t = int32
  val create : int32 -> t
  val to_int : t -> int32
  val compare : t -> t -> int
  val zero : t
  val show : t -> string
end

module Label : sig
  type t = string
  val create : string -> t
  val to_string : t -> string
  val show : t -> string
end

type instruction =
  | Transform of {
      morph_reg : RegisterId.t;
      input_reg : RegisterId.t;
      output_reg : RegisterId.t;
    }
  | Alloc of {
      type_reg : RegisterId.t;
      init_reg : RegisterId.t;
      output_reg : RegisterId.t;
    }
  | Consume of {
      resource_reg : RegisterId.t;
      output_reg : RegisterId.t;
    }
  | Compose of {
      first_reg : RegisterId.t;
      second_reg : RegisterId.t;
      output_reg : RegisterId.t;
    }
  | Tensor of {
      left_reg : RegisterId.t;
      right_reg : RegisterId.t;
      output_reg : RegisterId.t;
    }

val show : instruction -> string
val operation_type : instruction -> string
val verify_category_laws : instruction -> bool
val is_linear : instruction -> bool
val reads_from : instruction -> RegisterId.t list
val writes_to : instruction -> RegisterId.t list
val is_control_flow : instruction -> bool
