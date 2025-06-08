(* ------------ OCAML CAUSALITY LIBRARY ------------ *)
(* Purpose: Main library module for the reorganized OCaml Causality Framework *)

module Core = Ocaml_causality_core
(** Core types and fundamental abstractions *)

module Lang = Ocaml_causality_lang
(** Language constructs and DSL *)

module Effects = Ocaml_causality_effects
(** Effect system components and coordination *)

module Serialization = Ocaml_causality_serialization
(** Serialization and content addressing *)

module Interop = Ocaml_causality_interop
(** External integrations *)

(* ------------ CONVENIENCE RE-EXPORTS ------------ *)

(** Common types for easy access *)
module Types = struct
  include Ocaml_causality_core
end

(** Common operations *)
module Ops = struct
  include Effects.Effects
  include Serialization.Content_addressing
end
