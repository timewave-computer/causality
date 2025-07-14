# 304: OCaml Causality DSL Specification

## Overview

The OCaml Causality DSL provides a functional interface to the Causality linear resource system. This specification documents the complete implementation including type-safe linear resource management, content-addressed storage, expression compilation, and FFI integration with the Rust backend.

### The System Provides

- **Linear Resource Management**: Create, consume, and track resources with automatic linearity enforcement
- **Expression System**: Construct, compile, and evaluate Lisp-style expressions
- **Content Addressing**: Store and retrieve content with cryptographic integrity verification
- **Pattern Matching**: Match and filter resources by type and domain
- **FFI Bridge**: Full integration with Rust causality-core backend via C FFI
- **Error Handling**: Comprehensive error types and graceful failure handling

## Architecture

### Module Structure

```
ocaml_causality/
├── lib/
│   ├── core/                    # Core types and patterns
│   │   ├── ocaml_causality_core.ml  # Unified type definitions
│   │   ├── patterns.ml              # Resource pattern matching
│   │   └── identifiers.ml           # ID types and content addressing
│   ├── lang/                    # Language constructs
│   │   ├── expr.ml                  # Expression AST and operations
│   │   ├── value.ml                 # LispValue module
│   │   └── ast.ml                   # Core AST definitions
│   ├── interop/                 # FFI and external integration
│   │   ├── ffi.ml                   # Main FFI interface
│   │   ├── ffi_stub.c               # C FFI stubs
│   │   ├── causality-ffi.h          # C header definitions
│   │   ├── rust_bridge.ml           # Rust type conversion
│   │   └── external_apis.ml         # External service integration
│   ├── serialization/           # Content addressing and storage
│   │   ├── content_addressing.ml    # Content-addressed storage
│   │   └── system_content_addressing.ml  # SSZ serialization
│   ├── system/                  # System-level functionality
│   ├── machine/                 # Machine execution
│   ├── lambda/                  # Lambda calculus
│   └── effects/                 # Effect system
└── examples/
    ├── e2e_demo.ml              # End-to-end demonstration
    └── integration_test.ml      # Comprehensive test suite
```

## API Documentation

### Core Types

```ocaml
(* Basic identifiers *)
type resource_id = bytes
type expr_id = bytes
type entity_id = bytes  
type domain_id = bytes
type effect_id = bytes

(* Resource management *)
type resource = {
  id: resource_id;
  name: string;
  domain_id: domain_id;
  resource_type: string;
  quantity: int64;
  timestamp: int64;
}

type resource_flow = {
  resource_type: string;
  quantity: int64;
  domain_id: domain_id;
}

type resource_pattern = {
  resource_type: string;
  domain_id: domain_id option;
}

(* Causality primitives *)
type intent = {
  id: entity_id;
  name: string;
  domain_id: domain_id;
  priority: int;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option;
  timestamp: int64;
  hint: expr_id option;
}

type effect = {
  id: effect_id;
  name: string;
  domain_id: domain_id;
  effect_type: string;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option;
  timestamp: int64;
  hint: expr_id option;
}

(* Error handling *)
type causality_error =
  | LinearityViolation of string
  | InvalidResource of resource_id
  | InvalidExpression of expr_id
  | FFIError of string
  | SerializationError of string
  | DomainError of string

(* Lisp values for expressions *)
type lisp_value =
  | Unit
  | Bool of bool
  | Int of int64
  | String of string
  | Symbol of string
  | List of lisp_value list
  | ResourceId of resource_id
  | ExprId of expr_id
  | Bytes of bytes
```

### FFI Interface

```ocaml
module Ffi : sig
  (* Abstract FFI types *)
  type causality_value
  type causality_resource  
  type causality_expr

  (* Initialization *)
  val initialize_ffi : unit -> (unit, causality_error) result
  val cleanup_ffi : unit -> unit
  
  (* Resource management *)
  val safe_create_resource : string -> domain_id -> int64 -> 
    (resource_id option, causality_error) result
  val safe_consume_resource_by_id : resource_id -> 
    (bool, causality_error) result
  
  (* Expression compilation *)
  val safe_compile_expr : string -> 
    (expr_id option, causality_error) result
  val safe_submit_intent : string -> domain_id -> string -> 
    (bool, causality_error) result
  
  (* System metrics *)
  val safe_get_system_metrics : unit -> 
    (string, causality_error) result

  (* Value operations *)
  val create_unit : unit -> causality_value
  val create_bool : bool -> causality_value
  val create_int : int -> causality_value
  val create_string : string -> causality_value
  val create_symbol : string -> causality_value

  (* Serialization *)
  val serialize : causality_value -> (bytes, string) result
  val deserialize : bytes -> (causality_value, string) result
  
  (* Testing *)
  val test_value_roundtrip : causality_value -> bool
  val test_comprehensive_roundtrip : unit -> bool
  val get_version : unit -> string
end
```

### Expression System

```ocaml
module Expr : sig
  type t = expr_ast
  
  (* AST type definition *)
  type expr_ast =
    | Const of lisp_value
    | Alloc of expr_ast
    | Consume of resource_id
    | Lambda of lisp_value list * expr_ast
    | Apply of expr_ast * expr_ast list
    | Let of string * expr_ast * expr_ast
    | If of expr_ast * expr_ast * expr_ast
    | Sequence of expr_ast list

  (* Constructors *)
  val const : lisp_value -> t
  val alloc : t -> t
  val consume : resource_id -> t
  val lambda : lisp_value list -> t -> t
  val apply : t -> t list -> t
  val let_binding : string -> t -> t -> t
  val if_then_else : t -> t -> t -> t
  val sequence : t list -> t
  
  (* Convenience constructors *)
  val const_int : int64 -> t
  val const_string : string -> t  
  val const_bool : bool -> t
  val const_unit : t
  
  (* Compilation *)
  val compile_and_register_expr : t -> (expr_id, causality_error) result
  val get_predefined_expr_id : string -> expr_id option
  
  (* Evaluation context *)
  type eval_context = {
    bindings: (string * lisp_value) list;
    resources: resource_id list;
  }
  
  val empty_context : eval_context
  val bind_value : string -> lisp_value -> eval_context -> eval_context
  val lookup_binding : string -> eval_context -> lisp_value option
  
  (* Evaluation *)
  val eval_expr : eval_context -> t -> (lisp_value, causality_error) result
  val to_string : t -> string
  val free_variables : t -> string list
end
```

### Content Addressing

```ocaml
module Content_addressing : sig
  type content_store
  
  val create_store : unit -> content_store
  val store_content : content_store -> bytes -> entity_id
  val retrieve_content : content_store -> entity_id -> bytes option
  val content_exists : content_store -> entity_id -> bool
  val list_content_ids : content_store -> entity_id list
  val verify_content_id : bytes -> entity_id -> bool
  val verify_store_integrity : content_store -> (entity_id * bool) list
  val cleanup_invalid_content : content_store -> int
end
```

### Pattern Matching

```ocaml
(* Pattern construction *)
val pattern_for_type : string -> resource_pattern
val pattern_for_type_and_domain : string -> domain_id -> resource_pattern
val wildcard_pattern : resource_pattern

(* Pattern matching *)
val matches_pattern : resource_pattern -> string -> domain_id -> bool
val filter_by_pattern : resource_pattern -> (string * domain_id) list -> 
  (string * domain_id) list

(* Flow management *)
val create_flow : string -> int64 -> domain_id -> resource_flow
val flow_satisfies_minimum : resource_flow -> int64 -> bool
val combine_flows : resource_flow list -> resource_flow list
```

### Session Types

Session types provide type-safe communication protocols with automatic duality checking in the OCaml DSL. This module integrates seamlessly with the Causality Layer 2 session type system.

```ocaml
(* Session type definitions *)
type session_id = bytes

type session_type =
  | Send of lisp_value * session_type        (* !T.S *)
  | Receive of lisp_value * session_type     (* ?T.S *)
  | InternalChoice of session_type list      (* ⊕{...} *)
  | ExternalChoice of session_type list      (* &{...} *)
  | End                                       (* End *)
  | Recursive of string * session_type       (* rec X.S *)
  | Variable of string                        (* X *)

type session_role = {
  role_name: string;
  protocol: session_type;
}

type session_declaration = {
  session_name: string;
  roles: session_role list;
  verified_duality: bool;
}

(* Session channel type *)
type 'a session_channel = {
  session_id: session_id;
  protocol: session_type;
  role: string;
  channel_data: bytes;
}

(* Session operations *)
type session_operation =
  | SessionSend of lisp_value
  | SessionReceive
  | SessionSelect of string
  | SessionCase of (string * (session_channel -> lisp_value)) list

(* Session errors *)
type session_error =
  | ProtocolViolation of string
  | DualityMismatch of string * string
  | ChannelClosed of session_id
  | InvalidChoice of string list * string
  | SessionNotFound of string
```

### Session Type Module

```ocaml
module SessionType : sig
  (* Session type construction *)
  val send : lisp_value -> session_type -> session_type
  val receive : lisp_value -> session_type -> session_type
  val internal_choice : session_type list -> session_type
  val external_choice : session_type list -> session_type
  val end_session : session_type
  val recursive : string -> session_type -> session_type
  val variable : string -> session_type
  
  (* Duality computation *)
  val compute_dual : session_type -> session_type
  val verify_duality : session_type -> session_type -> bool
  
  (* Session type utilities *)
  val to_string : session_type -> string
  val is_well_formed : session_type -> bool
  val substitute : string -> session_type -> session_type -> session_type
end

module Session : sig
  (* Session declaration *)
  val declare_session : string -> session_role list -> 
    (session_declaration, session_error) result
  
  (* Session creation and management *)
  val create_session : string -> string -> session_type -> 
    ('a session_channel, session_error) result
  
  (* Session operations *)
  val session_send : 'a session_channel -> lisp_value -> 
    ('b session_channel, session_error) result
  
  val session_receive : 'a session_channel -> 
    (lisp_value * 'b session_channel, session_error) result
  
  val session_select : 'a session_channel -> string -> 
    ('b session_channel, session_error) result
  
  val session_case : 'a session_channel -> 
    (string * ('b session_channel -> 'c)) list -> 
    ('c, session_error) result
  
  (* Session context management *)
  val with_session : string -> string -> 
    ('a session_channel -> ('b, session_error) result) -> 
    ('b, session_error) result
  
  (* Session registry *)
  val register_session : session_declaration -> (unit, session_error) result
  val get_session : string -> (session_declaration option, session_error) result
  val list_sessions : unit -> (string list, session_error) result
end
```

### Choreography Support

```ocaml
(* Choreography definitions *)
type choreography_communication = {
  from_role: string;
  to_role: string;
  message_type: lisp_value;
}

type choreography_protocol =
  | Communication of choreography_communication
  | Choice of string * choreography_protocol list
  | Parallel of choreography_protocol list
  | Sequential of choreography_protocol list

type choreography = {
  choreography_name: string;
  roles: string list;
  protocol: choreography_protocol;
}

module Choreography : sig
  (* Choreography construction *)
  val create_choreography : string -> string list -> choreography_protocol -> choreography
  
  (* Communication patterns *)
  val point_to_point : string -> string -> lisp_value -> choreography_protocol
  val choice : string -> choreography_protocol list -> choreography_protocol
  val parallel : choreography_protocol list -> choreography_protocol
  val sequential : choreography_protocol list -> choreography_protocol
  
  (* Endpoint projection *)
  val project_role : choreography -> string -> (session_type, session_error) result
  
  (* Choreography validation *)
  val validate_choreography : choreography -> (unit, session_error) result
  
  (* Choreography execution *)
  val execute_role : choreography -> string -> 
    (lisp_value session_channel -> (lisp_value, session_error) result) -> 
    (lisp_value, session_error) result
end
```

### Session FFI Integration

```ocaml
module SessionFfi : sig
  (* Session-specific FFI operations *)
  val create_session_channel : string -> string -> bytes -> 
    (session_id option, causality_error) result
  
  val session_send_ffi : session_id -> bytes -> 
    (session_id option, causality_error) result
  
  val session_receive_ffi : session_id -> 
    (bytes option * session_id option, causality_error) result
  
  val session_select_ffi : session_id -> string -> 
    (session_id option, causality_error) result
  
  val session_case_ffi : session_id -> string list -> 
    (string option * session_id option, causality_error) result
  
  (* Session registry FFI *)
  val register_session_ffi : string -> bytes -> 
    (bool, causality_error) result
  
  val get_session_protocol_ffi : string -> string -> 
    (bytes option, causality_error) result
  
  (* Session validation FFI *)
  val verify_session_duality_ffi : bytes -> bytes -> 
    (bool, causality_error) result
end
```

## Usage Examples

### Linear Resource Management

```ocaml
(* Initialize the system *)
let _ = Ffi.initialize_ffi () in

(* Create a properly sized domain (32 bytes) *)
let domain_id = 
  let base = "my_domain" in
  let padded = base ^ String.make (32 - String.length base) '\000' in
  Bytes.of_string padded in

(* Create resources *)
match Ffi.safe_create_resource "token" domain_id 100L with
| Ok (Some resource_id) -> 
    (* First consumption succeeds *)
    let _ = Ffi.safe_consume_resource_by_id resource_id in
    (* Second consumption fails due to linearity *)
    let _ = Ffi.safe_consume_resource_by_id resource_id in
    ()
| _ -> ()
```

### Expression Compilation

```ocaml
open Value

(* Create expressions *)
let const_expr = Expr.const (LispValue.int 42L) in
let lambda_expr = Expr.lambda 
  [LispValue.symbol "x"] 
  (Expr.const (LispValue.symbol "x")) in

(* Compile to content-addressed IDs *)
match Expr.compile_and_register_expr lambda_expr with
| Ok expr_id -> Printf.printf "Compiled to: %s\n" (Bytes.to_string expr_id)
| Error err -> Printf.printf "Compilation failed\n"
```

### Content-Addressed Storage

```ocaml
(* Create store and add content *)
let store = Content_addressing.create_store () in
let content = Bytes.of_string "Hello, Causality!" in
let content_id = Content_addressing.store_content store content in

(* Retrieve and verify *)
match Content_addressing.retrieve_content store content_id with
| Some retrieved -> Printf.printf "Retrieved: %s\n" (Bytes.to_string retrieved)
| None -> Printf.printf "Content not found\n"

(* Verify integrity *)
let integrity_results = Content_addressing.verify_store_integrity store in
List.iter (fun (id, valid) -> 
  Printf.printf "%s: %s\n" (Bytes.to_string id) (if valid then "VALID" else "INVALID")
) integrity_results
```

### Session Type Usage

```ocaml
open SessionType
open Session

(* Define a payment protocol with automatic duality verification *)
let payment_protocol =
  let client_role = {
    role_name = "client";
    protocol = send (Int 0L) (receive (String "") end_session);
  } in
  let server_role = {
    role_name = "server"; 
    protocol = receive (Int 0L) (send (String "") end_session);
  } in
  match declare_session "PaymentProtocol" [client_role; server_role] with
  | Ok session_decl -> 
      Printf.printf "Payment protocol declared with duality: %b\n" 
        session_decl.verified_duality;
      Some session_decl
  | Error err -> 
      Printf.printf "Failed to declare session: %s\n" 
        (match err with
         | DualityMismatch (r1, r2) -> "Duality mismatch between " ^ r1 ^ " and " ^ r2
         | _ -> "Unknown error");
      None

(* Client-side payment implementation *)
let handle_payment_client amount =
  with_session "PaymentProtocol" "client" (fun client_channel ->
    match session_send client_channel (Int amount) with
    | Ok updated_channel ->
        (match session_receive updated_channel with
         | Ok (receipt, final_channel) ->
             Printf.printf "Payment completed, receipt: %s\n" 
               (match receipt with String s -> s | _ -> "invalid");
             Ok receipt
         | Error err -> Error err)
    | Error err -> Error err
  )

(* Server-side payment implementation *)
let handle_payment_server () =
  with_session "PaymentProtocol" "server" (fun server_channel ->
    match session_receive server_channel with
    | Ok (amount, updated_channel) ->
        let receipt = Printf.sprintf "Receipt for %Ld" 
          (match amount with Int i -> i | _ -> 0L) in
        (match session_send updated_channel (String receipt) with
         | Ok final_channel -> Ok (String receipt)
         | Error err -> Error err)
    | Error err -> Error err
  )

(* Multi-party escrow choreography *)
let escrow_choreography =
  let buyer_to_seller = point_to_point "buyer" "seller" (String "item_request") in
  let seller_to_buyer = point_to_point "seller" "buyer" (String "item_details") in
  let buyer_to_arbiter = point_to_point "buyer" "arbiter" (Int 0L) in
  let seller_to_arbiter = point_to_point "seller" "arbiter" (String "delivery_proof") in
  
  let negotiation = sequential [buyer_to_seller; seller_to_buyer] in
  let escrow_setup = parallel [buyer_to_arbiter; seller_to_arbiter] in
  let protocol = sequential [negotiation; escrow_setup] in
  
  create_choreography "EscrowChoreography" ["buyer"; "seller"; "arbiter"] protocol

(* Execute buyer role in escrow *)
let execute_buyer_escrow item_request payment =
  match project_role escrow_choreography "buyer" with
  | Ok buyer_session_type ->
      (match create_session "EscrowChoreography" "buyer" buyer_session_type with
       | Ok buyer_channel ->
           (* Send item request *)
           (match session_send buyer_channel (String item_request) with
            | Ok updated_channel ->
                (* Receive item details *)
                (match session_receive updated_channel with
                 | Ok (item_details, final_channel) ->
                     (* Send payment to arbiter *)
                     session_send final_channel (Int payment)
                 | Error err -> Error err)
            | Error err -> Error err)
       | Error err -> Error err)
  | Error err -> Error err

(* Session type validation example *)
let validate_session_types () =
  let client_protocol = send (Int 0L) (receive (String "") end_session) in
  let server_protocol = receive (Int 0L) (send (String "") end_session) in
  
  if verify_duality client_protocol server_protocol then
    Printf.printf " Protocols are valid duals\n"
  else
    Printf.printf "✗ Protocol duality verification failed\n";
    
  (* Test duality involution property *)
  let dual_of_dual = compute_dual (compute_dual client_protocol) in
  if dual_of_dual = client_protocol then
    Printf.printf " Duality involution property holds\n"
  else
    Printf.printf "✗ Duality involution property failed\n"
```

## Conclusion

The OCaml Causality DSL provides a comprehensive implementation for building distributed applications with verifiable resource management. The framework integrates seamlessly with the Rust backend through FFI, enabling type-safe linear resource management, content-addressed storage, and expression compilation while maintaining the safety and expressiveness of OCaml.