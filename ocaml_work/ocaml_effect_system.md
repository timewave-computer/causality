# OCaml DSL Refactoring Plan (Alignment with graph.md)

This document outlines the tasks for refactoring the OCaml DSL in the `ml_causality` directory to align with the guidance provided in `graph.md`, particularly its section on mapping OCaml Algebraic Effects to the TEL Graph.

## I. Core DSL Structure & Types

- [x] **Define Core TEL Representation Types**: Create OCaml types to represent the essential components of a TEL graph that the DSL will generate (e.g., `TelEffectResource`, `TelHandlerResource`, `TelExpr`, `TelValue`).
- [x] **Establish Translation Entry Points**: Define the main functions or modules in the DSL that will take user-defined OCaml effects and handlers and translate them into the TEL representation.

## II. Mapping OCaml Effects to TEL (graph.md Section 4.1)

- [x] **User-Defined Effects**: Solidify how users define effects in the OCaml DSL.
    - [x] Ensure effect definitions include a clear payload structure.
- [x] **`Effect.perform` Translation**: Implement logic to translate an OCaml `Effect.perform (MyEffect payload)` into a `TelEffectResource` structure.
    - [x] Map OCaml effect constructor & payload to TEL `ValueExpr` for the resource.
    - [x] Integrate with PPX-based transpiler (see Section VI) to associate OCaml static validation functions with an effect, translating them into a Lisp `static_expr` for the `TelEffectResource`.
- [x] **Handler Definition DSL**: Design how users define OCaml effect handlers.
    - [x] Handler definitions should specify which effects they handle.
    - [x] Handler configuration structure should be definable.
    - [x] Integrate with PPX-based transpiler (see Section VI) to associate OCaml functions for static config validation and dynamic effect handling logic, translating them into Lisp `static_expr` and `dynamic_expr` for the `TelHandlerResource`.
- [x] **Effect-Handler Linking**: Determine how performed effects are routed to their appropriate handlers within the generated TEL graph.
    - [x] This involves creating the necessary `TelEdge` instances with correct `EdgeKind` (e.g., `HandlesEffect`, `TriggersEffect`).

## III. State Management API (graph.md Section 4.2)

- [x] **User-Defined Handlers**: Define how users specify effect handler logic in OCaml.
    - [x] This will likely involve a function that takes the effect's payload and an OCaml continuation `k`.
- [x] **Handler Logic Translation**: Implement logic to translate OCaml handler definitions into `TelHandlerResource` structures.
    - [x] The core OCaml handler logic (function body) must be translated into a Lisp S-expression for the `dynamic_expr` of the `TelHandlerResource`.
    - [x] Allow association of an OCaml static validation function for the handler's configuration, translating to a Lisp `static_expr` for the `TelHandlerResource`.
- [x] **Handler Registration/Association**: Implement a DSL mechanism for users to register an OCaml handler for specific OCaml effect type(s).
    - [x] This association will inform the creation of `Applies(HandlerId)` edges or similar relationships in the generated TEL graph representation.

## IV. Handling Continuations (graph.md Section 4.3)

- [x] **Linearity in Translation**: Ensure that the DSL's translation of OCaml handler logic (which uses the continuation `k`) into Lisp `dynamic_expr` correctly reflects the **"exactly once"** resumption semantics of OCaml continuations.
    - [x] The generated Lisp code should represent a single, linear path of execution for the handler logic based on the one-time use of `k`.
- [x] **DSL Guidance**: The DSL's API for defining handlers should implicitly or explicitly guide users towards structuring logic compatible with this linear processing.

## V. Leveraging Typed Effects (graph.md Section 4.4)

- [x] **Type-Driven Translation**: Investigate and implement how OCaml's effect type information (from effect definitions and potentially typed effect signatures) can guide the DSL's translation process.
    - [x] Use this information to ensure correct instantiation and linking of `TelEffectResource` and `TelHandlerResource` representations.

## VI. Lisp Generation & Compilation Target

- [x] **PPX-based OCaml-to-Lisp Transpiler**:
    - [x] **Define Translatable OCaml Subset**: Specify a subset of OCaml constructs (e.g., basic arithmetic, boolean logic, if/else, record/variant access, limited function calls) that the transpiler will support for TEL `static_expr` and `dynamic_expr` logic.
    - [x] **PPX Rewriter Implementation (`Ppxlib`)**:
        - [x] Recognizes specially annotated OCaml functions (e.g., `[@@tel_static_logic]`, `[@@tel_dynamic_logic]`).
        - [x] Accesses the OCaml AST of these functions.
        - [x] Translates the OCaml AST (from the defined subset) into Lisp S-expressions (strings). This involves mapping OCaml constructs to their Lisp equivalents (e.g., OCaml `if` to Lisp `if`, OCaml `let` to Lisp `let`).
        - [x] Leverages `Lisp_ast.to_sexpr_string` for final S-expression serialization.
    - [x] **Integration with DSL**: Ensure the `define_effect_resource` and `define_handler_resource` functions (or the PPX itself) can use the Lisp S-expressions generated by the transpiler to create `ExprId`s for `static_expr` and `dynamic_expr` fields in TEL resources.

- [x] **TEL Graph Output**: Define the output format of the DSL. This should be a representation of the TEL graph components (nodes and edges with their `ExprId` references) that can be serialized as S-expressions and consumed by the `causality-compiler`.

### Translatable OCaml Subset for TEL Logic

The `ppx_tel` transpiler aims to convert a subset of OCaml expressions into Lisp S-expressions for use in `static_expr` and `dynamic_expr` fields of TEL resources. The initial targeted subset includes:

**Supported Constructs:**

*   **Literals:**
    *   Integers (e.g., `10`, `0xFF`)
    *   Strings (e.g., `"hello"`)
    *   Booleans (`true`, `false`)
    *   Unit (`()`)
*   **Variables:**
    *   Local variables (introduced by `let` or function parameters).
    *   Access to parameters of the annotated function.
*   **Operators:**
    *   Integer arithmetic: `+`, `-`, `*`, `/`, `mod`
    *   Boolean logic: `&&`, `||`, `not`
    *   Comparisons: `=`, `<>`, `<`, `>`, `<=`, `>=` (for integers, strings, booleans)
*   **Control Flow:**
    *   `if <cond> then <expr1> else <expr2>`
*   **Bindings:**
    *   `let <var> = <expr1> in <expr2>`
    *   `let rec <fun_name> <params> = <expr1> in <expr2>` (for simple, non-mutually recursive functions, to be translated to Lisp `labels` or `flet`/`letrec`)
*   **Data Structures (Basic):**
    *   Tuples: Construction `(e1, e2, ...)` and access (e.g., `fst`, `snd`, or pattern matching in `let`).
    *   Lists: Construction `[]`, `e1 :: e2` and potentially basic pattern matching (`match lis with [] -> ... | h :: t -> ...`).
    *   Records: Construction `{ field1 = e1; ... }` and field access `record.field1` (assuming record types are known or simple).
*   **Function Calls:**
    *   Calls to other OCaml functions that are also part of this translatable subset or are known host functions callable from the Lisp environment.
    *   Simple anonymous functions: `fun x -> expr` (translated to Lisp `lambda`).

**Currently Out of Scope (for initial version):**

*   Complex pattern matching (beyond simple list/tuple deconstruction).
*   Mutable state (`ref`, `:=`, `!`) within the transpiled logic.
*   Loops (`for`, `while`).
*   Objects and classes.
*   Advanced module system features (e.g., functors, first-class modules) within the body of transpiled functions.
*   Exceptions (`try ... with ...`, `raise`).
*   Lazy evaluation.
*   Imperative I/O operations.
*   Extensive use of the OCaml standard library functions unless explicitly mapped to Lisp host functions.

### TEL Graph Output Format (S-expressions)

The DSL will output the TEL graph as a list of S-expressions, typically contained within a top-level form or simply as a sequence of definitions. Each resource (node) and relationship (edge) will be represented by a specific S-expression form.

**Schema Version:** 1.0 (Initial S-expression proposal)

**Top-Level Structure:**
A list of S-expressions, where each S-expression defines a node or an edge.
Example:
```lisp
(
  (define-effect-resource eff_abc123
    :ocaml-effect-name "MyEffect"
    :domain "my_domain"
    :value "val_payload_001" ; ID of the value expr for payload
    :static-expr '(lisp for static validation of MyEffect) ; Quoted Lisp S-expr
    :dynamic-expr nil ; or '(lisp for dynamic logic)
  )

  (define-handler-resource hnd_xyz789
    :handler-name "MyHandlerForMyEffect"
    :domain "my_domain"
    :value "val_config_002" ; ID of the value expr for config
    :static-expr '(lisp for static validation of MyHandler config)
    :dynamic-expr '(lisp for dynamic logic of MyHandler)
  )

  (define-edge edge_123_to_456
    :source eff_abc123 ; Reference by ID
    :target hnd_xyz789 ; Reference by ID
    :kind (handles-effect) ; Example kind
    :condition nil ; or '(lisp for edge condition)
  )
  ;; ... more node and edge definitions
)
```

**Node Definition S-expressions:**

*   **Effect Resource:**
    `(define-effect-resource <id:symbol>`
    `  :ocaml-effect-name <name:string>`
    `  :domain <domain_id:string>`
    `  :value <value_expr_id:string>`
    `  :static-expr <lisp_s_expression_or_nil:sexpr|nil>`
    `  :dynamic-expr <lisp_s_expression_or_nil:sexpr|nil> )`

*   **Handler Resource:**
    `(define-handler-resource <id:symbol>`
    `  :handler-name <name:string>`
    `  :domain <domain_id:string>`
    `  :value <value_expr_id:string>`
    `  :static-expr <lisp_s_expression_or_nil:sexpr|nil>`
    `  :dynamic-expr <lisp_s_expression:sexpr> )`
    
    *(Note: `<id:symbol>` means the ID will be an unquoted symbol in the S-expression, e.g., `eff_abc123`, not `"eff_abc123"`)*

**Edge Definition S-expression:**
`(define-edge <id:symbol>`
`  :source <node_id:symbol>`
`  :target <node_id:symbol>`
`  :kind <kind_s_expression:sexpr>`
`  :condition <lisp_s_expression_or_nil:sexpr|nil> )`

**Edge Kind S-expressions (`<kind_s_expression>`):**
The `:kind` field will take an S-expression that specifies the type of the edge and any associated data.
*   `(applies <handler_id:symbol>)`
*   `(depends-on <effect_id:symbol>)`
*   `(scoped-by <scope_id:symbol>)`
*   `(input <resource_id:symbol> <semantic:keyword>)`
    *   `semantic` can be `:consumed`, `:read-only`, `:referenced`.
*   `(output <resource_id:symbol> <semantic:keyword>)`
    *   `semantic` can be `:created`, `:modified`, `:derived`.
*   `(handles-effect)`
*   `(triggers-effect)`
*   `(other-edge <description:string>)`

**Example Edge with a specific kind:**
```lisp
(define-edge edge_applies_001
  :source some_node_id
  :target handler_node_id
  :kind (applies handler_node_id) 
)

(define-edge edge_input_002
  :source task_node_id
  :target data_resource_id
  :kind (input data_resource_id :consumed)
)
```

## VII. Testing and Examples

- [x] **New DSL Examples**: Create clear examples demonstrating how to use the refactored DSL to define effects, handlers, and compose them.

  *The following conceptual example should be placed in `ml_causality/examples/dsl_usage_example.ml` once the `examples` directory is created.* 

  ```ocaml
  (* ml_causality/examples/dsl_usage_example.ml *)
  
  open Ml_causality_lib_types (* For effect, handler, domain_id etc. *)
  (* Assuming a module DslInterface that exposes the public user-facing DSL functions *)
  (* module Dsl = Ml_causality_lib_dsl.Dsl (* Or whatever the actual DSL module is *) *)
  
  (* --- 0. Preliminary Type Definitions --- *)
  
  type user_id = int
  type user_info = { name: string; email: string }
  type error_reason = string
  
  (* --- 1. Define an Effect --- *)
  
  (* Effect declaration: Fetch user information by ID *)
  (* This would typically be: type _ eff += FetchUser : user_id -> user_info eff *)
  (* For the DSL, we register it by name. *)
  
  let effect_name_fetch_user = "FetchUser"
  
  (*
    Imagine some OCaml functions that will be transpiled to Lisp by ppx_tel.
    These are identified by string keys for now.
  *)
  
  (* Static logic for the FetchUser effect (e.g., validate user_id > 0) *)
  (* This function would be annotated with [@@tel_static_logic "fetch_user_static_key"] *)
  let validate_fetch_user_params (uid: user_id) : bool =
    uid > 0
  
  let fetch_user_static_key = "fetch_user_static_key"
  
  (* Let's assume the DSL provides a way to register these OCaml functions
     with their keys in the Ppx_registry. For this example, we'll assume
     Ppx_registry.register_logic has been called elsewhere by the PPX.
  *)
  
  (* --- 2. Define a Handler --- *)
  
  let handler_name_db_lookup = "UserDbHandler"
  
  (* Static logic for the handler (e.g., validate DB config) *)
  (* Annotated with [@@tel_static_logic "user_db_handler_static_config_key"] *)
  type db_config = { connection_string: string }
  let validate_db_config (cfg: db_config) : bool =
    cfg.connection_string <> ""
  
  let user_db_handler_static_config_key = "user_db_handler_static_config_key"
  
  (* Dynamic logic for the handler (actually fetch the user) *)
  (* Annotated with [@@tel_dynamic_logic "user_db_handler_dynamic_fetch_key"] *)
  let fetch_user_from_db (uid: user_id) (_cfg: db_config) : (user_info, error_reason) result =
    if uid = 1 then Ok { name = "Alice"; email = "alice@example.com" }
    else if uid = 2 then Ok { name = "Bob"; email = "bob@example.com" }
    else Error "User not found"
  
  let user_db_handler_dynamic_fetch_key = "user_db_handler_dynamic_fetch_key"
  
  (* --- 3. Using the DSL (Conceptual) --- *)
  (* The actual DSL functions (define_effect, define_handler) would internally call
     _define_tel_effect_resource and _define_tel_handler_resource.
     They would also create value_expr_id for payloads/configs.
  *)
  
  (*
  module MyDomainDsl = struct
    let domain_id : domain_id = "MyExampleDomain" (* Domain ID for these resources *)
  
    (* Public facing DSL function to define an effect *)
  *)
    let define_effect
        ~(effect_name: string)
        ?~(static_logic_key: string option)
        (* ... other params like payload type representation potentially ... *)
        () : (effect_id, string) result =
      (* 1. Create value_expr_id for effect parameters schema (omitted for brevity) *)
      let dummy_payload_value_id : value_expr_id = "val_payload_" ^ effect_name in
      (* 2. Call the internal DSL function *)
      Ml_causality_lib_dsl._define_tel_effect_resource
        ~effect_name
        ~payload_value_id:dummy_payload_value_id
        ~static_logic_key
        ~domain_id
      |> Result.map (fun res -> res.id)
  
  
    (* Public facing DSL function to define a handler *)
    let define_handler
        ~(handler_name: string)
        ?~(static_logic_key: string option)
        ~(dynamic_logic_key: string)
        (* ... other params like config type representation ... *)
        () : (handler_id, string) result =
      (* 1. Create value_expr_id for handler config schema (omitted for brevity) *)
      let dummy_config_value_id : value_expr_id = "val_config_" ^ handler_name in
      (* 2. Call the internal DSL function *)
      Ml_causality_lib_dsl._define_tel_handler_resource
        ~handler_name
        ~config_value_id:dummy_config_value_id
        ~static_logic_key
        ~dynamic_logic_key
        ~domain_id
      |> Result.map (fun res -> res.id)
  
  end
  *)
  
  (* --- 4. Registering Resources (Example Usage of Conceptual DSL) --- *)
  let () =
    (* Register the effect *)
    match MyDomainDsl.define_effect ~effect_name:effect_name_fetch_user ~static_logic_key:fetch_user_static_key () with
    | Ok eff_id -> Printf.printf "Registered effect '%s' with ID: %s\n" effect_name_fetch_user eff_id
    | Error msg -> Printf.printf "Error registering effect '%s': %s\n" effect_name_fetch_user msg;
  
    (* Register the handler *)
    match MyDomainDsl.define_handler
            ~handler_name:handler_name_db_lookup
            ~static_logic_key:user_db_handler_static_config_key
            ~dynamic_logic_key:user_db_handler_dynamic_fetch_key
            ()
    with
    | Ok hnd_id -> Printf.printf "Registered handler '%s' with ID: %s\n" handler_name_db_lookup hnd_id
    | Error msg -> Printf.printf "Error registering handler '%s': %s\n" handler_name_db_lookup msg
  *)
  
  (* --- 5. Performing an Effect (Conceptual) --- *)
  (* This part would involve the actual effect system runtime, not just the DSL for definition *)
  let get_user_info (uid: user_id) : (user_info, error_reason) result =
    (* In a real system, perform would take the effect constructor directly.
       Here, we simulate the idea that 'perform' knows about registered effects. *)
    Printf.printf "Attempting to perform effect '%s' for user_id: %d\n" effect_name_fetch_user uid;
    if uid = 1 then (* Simulate handler logic directly for now *)
      fetch_user_from_db uid {connection_string="dummy_conn_str"}
    else Error "Effect perform simulation: User not found or unhandled."
  
  let () =
    let user_result = get_user_info 1 in
    match user_result with
    | Ok info -> Printf.printf "Got user: %s, %s\n" info.name info.email
    | Error reason -> Printf.printf "Failed to get user: %s\n" reason
  
  let () =
    Printf.printf "DSL Usage Example (conceptual).\n";
    Printf.printf "Effect Name: %s (static key: %s)\n" effect_name_fetch_user fetch_user_static_key;
    Printf.printf "Handler Name: %s (static key: %s, dynamic key: %s)\n"
      handler_name_db_lookup user_db_handler_static_config_key user_db_handler_dynamic_fetch_key
  
  (*
    Further examples would show:
    - How value_expr_ids are created for actual payloads and configs (e.g. from OCaml type definitions).
    - How graph edges are defined using the DSL.
    - Integration with the OCaml effects and handlers syntax (`type _ eff += ...`, `match_with`).
  *)
  ```

- [x] **Unit Tests**: Add unit tests for the DSL's core translation logic (e.g., OCaml effect definition to `TelEffectResource` structure, OCaml handler to `TelHandlerResource` structure, OCaml data to Lisp S-expression).

## VIII. Refactoring `ml_causality` Directory

- [x] **File Structure Review**: Review and potentially reorganize the file structure within `ml_causality` for clarity and maintainability.
- [x] **Identify Existing Code**: Map existing DSL components to the new plan and identify what can be refactored vs. what needs to be rewritten.
- [x] **Iterative Implementation**: Apply the changes from sections I-VII to the codebase.
