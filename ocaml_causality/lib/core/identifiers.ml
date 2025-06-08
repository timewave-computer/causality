(* ------------ ENTITY IDENTIFIERS ------------ *)
(* Purpose: Entity IDs, content addressing primitives *)

(* ------------ BASIC TYPE ALIASES ------------ *)

type bytes = Bytes.t
(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's
    [u8; N] for IDs. *)

type str_t = string
(** Represents a string. Corresponds to Rust's Str or String. *)

type timestamp = int64
(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to
    Rust's Timestamp. *)

(* ------------ IDENTIFIER TYPES ------------ *)

type expr_id = bytes
(** Unique identifier for an expression. Corresponds to Rust's ExprId. *)

type value_expr_id = bytes
(** Unique identifier for a value expression. (Note: In Rust, ExprId often
    serves for content-addressed values too) *)

type entity_id = bytes
(** Generic unique identifier for an entity. Corresponds to Rust's EntityId. *)

type domain_id = bytes
(** Unique identifier for a domain. Corresponds to Rust's DomainId. *)

type handler_id = bytes
(** Unique identifier for a handler. Corresponds to Rust's HandlerId. *)

type edge_id = bytes
(** Unique identifier for an edge. Corresponds to Rust's EdgeId. *)

type node_id = bytes
(** Unique identifier for a node. Corresponds to Rust's NodeId. *)

(* ------------ CONTENT ADDRESSING ------------ *)

(* TODO: Extract content addressing from lib/content_addressing/ *)

(* ------------ ID GENERATION ------------ *)

(* TODO: Add ID generation functions *)

(* ------------ COMPARISON AND EQUALITY ------------ *)

(* TODO: Add comparison functions for IDs *)
