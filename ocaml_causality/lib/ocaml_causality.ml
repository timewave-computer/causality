(* ------------ OCAML CAUSALITY LIBRARY ------------ *)
(* Purpose: Main library module for the reorganized OCaml Causality Framework *)

(** Core types and fundamental abstractions *)
module Core = Ocaml_causality_core

(** Language constructs and DSL *)  
module Lang = Ocaml_causality_lang

(** Effect system components and coordination *)
module Effects = Ocaml_causality_effects

(** Serialization and content addressing *)
module Serialization = Ocaml_causality_serialization

(** External integrations *)
module Interop = Ocaml_causality_interop

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