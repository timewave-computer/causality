(** Core computational substrate for the Causality framework.

    This library provides the fundamental types, traits, and implementations for
    the Causality linear resource language, organized as a three-layer
    architecture:

    - {!module:System} - Cross-cutting system utilities (content addressing,
      errors, domains)
    - {!module:Machine} - Layer 0: Register Machine (5 fundamental instructions, minimal
      verifiable execution)
    - {!module:Lambda} - Layer 1: Linear Lambda Calculus (type-safe functional
      programming)
    - {!module:Effect} - Layer 2: Effect Algebra (domain-specific effect
      management) *)

(** {1 System Utilities} *)

module System = struct
  (* Note: These will be implemented as separate modules in the system/ directory *)

end

(** {1 Layer 0: Register Machine} *)

module Machine = struct
  (* Will be implemented in Phase 2 *)
end

(** {1 Layer 1: Linear Lambda Calculus} *)

module Lambda = struct
  (* Will be implemented in Phase 3 *)
end

(** {1 Layer 2: Effect Algebra} *)

module Effect = struct
  (* Will be implemented in Phase 4 *)
end
