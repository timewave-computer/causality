(* ------------ OCAML CAUSALITY LIBRARY ------------ *)
(* Purpose: Main library module for the reorganized OCaml Causality Framework *)

module Core = Causality_core
(** Core types and fundamental abstractions *)

(** Layer 2 compilation functions *)
module Compiler = struct
  include Causality_compiler.Layer2_compiler
  include Causality_compiler.Intent_compiler  
  include Causality_compiler.Effect_compiler
end

(** End-to-end compilation pipeline *)
module Pipeline = struct
  include Causality_compiler.Pipeline
end
