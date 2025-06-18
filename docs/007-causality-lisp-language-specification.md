# 007: Causality Lisp Language Specification

This document defines the syntax, semantics, and type system of **Causality Lisp**, the Layer 1 functional programming language built on top of Causality's Layer 0 Typed Register Machine. Causality Lisp extends linear lambda calculus with algebraic effects, structured types, and resource management capabilities while maintaining the system's core principles of linearity, immutability, and verifiable correctness.

## 1. Overview

Causality Lisp is a statically-typed, linear functional programming language designed for expressing resource-aware computations. Every value in Causality Lisp is linear by default, meaning it must be consumed exactly once. This restriction enables compile-time verification of resource safety and forms the foundation for Causality's broader guarantees around correctness and conservation.

The language is built around 11 core primitives that correspond directly to fundamental operations of linear lambda calculus with effects. These primitives compile to a minimal set of register machine instructions, ensuring that the high-level expressiveness of the language translates to efficient, verifiable execution.

### Key Features

- **Linear Type System**: All values are linear by default, preventing resource duplication and ensuring proper resource lifecycle management
- **Algebraic Effects**: First-class support for effects with handlers, enabling controlled interaction with external systems
- **Structured Types**: Row types, records, and sum types with compile-time capability tracking
- **Resource Management**: Direct integration with Layer 0's resource allocation and consumption model
- **Verifiable Compilation**: Transparent compilation to register machine instructions with formal correctness guarantees

### Example Application Context

Before diving into the technical specification, consider this illustrative use case—a simple secure event ticketing system:

```lisp
; Event ticket as a linear resource with ownership transfer
; - Issuing: creates new ticket resource
; - Transfer: consumes old ticket, produces new ticket with different owner
; - Verification: proves ticket authenticity without revealing private details  

; Issue a new ticket (returns a linear ResourceId)
(defn issue-ticket (event-name ticket-id owner-pk)
  (alloc (record 
    ("event" event-name)
    ("ticket-id" ticket-id) 
    ("owner" owner-pk)
    ("issued-at" (current-timestamp)))))

; Transfer ticket ownership (linear: consumes input, produces output)
(defn transfer-ticket (ticket-resource new-owner-pk)
  (let ((ticket-data (consume ticket-resource)))
    (alloc (update-record ticket-data "owner" new-owner-pk))))

; Verify ticket without revealing details (using ZK-provable logic)
(defn verify-ticket-ownership (ticket-resource claimed-owner)
  (let ((ticket-data (consume ticket-resource)))
    (tensor 
      (= (record-get ticket-data "owner") claimed-owner)
      (alloc ticket-data)))) ; Re-allocate for continued use
```

This example demonstrates several key aspects of Causality Lisp:
- **Linearity**: `ticket-resource` must be consumed exactly once
- **Resource Management**: `alloc` and `consume` explicitly manage resource lifecycle  
- **Structured Data**: Records with named fields for complex data
- **Computation**: Logic expressed through function composition
- **Conservation**: Transfer preserves value while changing ownership
- **Verification**: Computation can be made verifiable through ZK integration

The same high-level logic compiles to efficient register machine instructions while preserving all safety and correctness guarantees.

### Example Compiled Output

The `transfer-ticket` function above compiles to approximately this sequence of Layer 0 instructions:

```
; Load inputs from parameters 
witness r0           ; ticket-resource
witness r1           ; new-owner-pk

; Consume the ticket resource
consume r0 → r2      ; ticket-data

; Update the owner field  
update_field r2 "owner" r1 → r3

; Allocate the new ticket
alloc r3 → r4        ; new ticket resource

; Return result
move r4 → return
```

This demonstrates how high-level resource operations translate to explicit, verifiable register machine operations while maintaining the same semantic guarantees.

### Core Design Principles

The design of Causality Lisp reflects several fundamental principles that distinguish it from conventional programming languages:

1. **Linearity as Foundation**: Every value is linear by default. Resources must be consumed exactly once. This eliminates entire classes of bugs related to resource management and enables strong static guarantees about program behavior.

2. **Static Verification**: The type system catches resource safety violations at compile time. Programs that type-check are guaranteed to respect resource linearity constraints, preventing issues like double-spending or resource leaks.

3. **Minimal Core**: 11 primitives compile to nine register machine instructions. This minimalism enables formal verification of the entire language implementation while providing sufficient expressiveness for real-world applications.

4. **Handler/Interpreter Separation**: Pure transformations compose cleanly, while stateful execution is isolated to specific handlers. This separation enables powerful abstraction and composition patterns while maintaining predictable behavior.

5. **Conservation Laws**: Value is neither created nor destroyed, only transformed. All operations preserve the total "value" in the system, enabling powerful reasoning about program behavior and resource flows.

6. **Declarative Intent**: Programs specify what should happen, not how. The runtime and compiler are responsible for synthesizing optimal execution strategies, enabling high-level reasoning about correctness without sacrificing performance.

These principles work together to create a programming model that is both expressive enough for real-world applications and constrained enough to enable strong formal guarantees about program behavior.

### Relationship to Traditional Functional Programming

Causality Lisp builds on the foundation of traditional functional programming while adding crucial innovations for resource management:

**Similarities to Traditional FP:**
- Immutable data structures and referential transparency
- First-class functions and higher-order computation
- Algebraic data types and pattern matching
- Compositional program structure

**Key Differences:**
- **Linear types by default**: Unlike most functional languages where copying is implicit and unlimited, Causality Lisp requires explicit management of linear resources
- **Resource lifecycle tracking**: The type system tracks not just types but also resource ownership and consumption
- **Effect integration**: Effects are first-class citizens with dedicated syntax and compilation support  
- **Compilation target**: Programs compile to register machine instructions rather than interpreting in a runtime environment

This combination enables Causality Lisp to maintain the reasoning benefits of functional programming while providing the control and efficiency needed for resource-critical applications.

## 2. Design Rationale

### Why Linear Types by Default?

Traditional programming languages allow unrestricted copying and aliasing of values, which creates several problems in resource-aware systems:

1. **Resource Safety**: Without linearity, it's easy to accidentally duplicate or forget to clean up resources
2. **Verification Complexity**: Proving properties about resource usage becomes exponentially harder with aliasing
3. **Hidden Costs**: Implicit copying can lead to unexpected performance characteristics
4. **Security Vulnerabilities**: Uncontrolled resource access enables entire classes of attacks

By making linearity the default, Causality Lisp inverts these problems:
- Resource safety is guaranteed by construction
- Verification becomes tractable through linear logic
- All costs are explicit and predictable
- Security properties are enforced by the type system

### Why 11 Primitives?

The choice of exactly 11 primitives reflects a careful balance between expressiveness and simplicity:

**Theoretical Foundation**: These primitives correspond directly to the core constructs of linear lambda calculus:
- **Unit types** (`UnitVal`, `LetUnit`): The trivial type with one inhabitant
- **Tensor products** (`Tensor`, `LetTensor`): Combining multiple values
- **Sum types** (`Inl`, `Inr`, `Case`): Choosing between alternatives
- **Function types** (`Lambda`, `Apply`): First-class functions
- **Resource management** (`Alloc`, `Consume`): Explicit resource lifecycle

**Practical Considerations**: 11 primitives are:
- Few enough to formally verify the entire language implementation
- Rich enough to express complex real-world applications
- Structured enough to enable systematic optimization
- Simple enough for developers to understand completely

### Why Compile to Register Machine?

The choice to compile to a register machine rather than other targets (stack machine, direct interpretation, native code) serves several purposes:

1. **Verification**: Register machines have well-understood formal semantics that enable verification
2. **Efficiency**: Direct register allocation eliminates many intermediate steps
3. **Transparency**: The compilation process is predictable and auditable
4. **Integration**: Register machines integrate naturally with ZK proof systems
5. **Parallelization**: Register-based code is easier to analyze for parallel execution

## 3. `LispValue`: Data Types

`LispValue` defines the set of concrete data types that can be manipulated within Causality Lisp programs. These are richer than the raw Layer 0 machine values and provide a more convenient programming model.

```rust
pub enum LispValue {
    Unit,                                    // The trivial value
    Bool(bool),                             // Boolean values
    Int(i64),                               // 64-bit signed integers
    String(String),                         // UTF-8 strings
    Symbol(String),                         // Atomic identifiers
    List(Vec<LispValue>),                   // Ordered collections
    Map(std::collections::HashMap<String, LispValue>),      // Key-value stores
    Record(std::collections::HashMap<String, LispValue>),   // Structured data
    ResourceId(u64),                        // Linear resource handles
    ExprId(u64),                           // AST node references
}
```

### Type Design Rationale

| Type | Purpose | Linearity | Use Cases |
|------|---------|-----------|-----------|
| `Unit` | Sequencing and void returns | Non-linear | Control flow, initialization |
| `Bool` | Binary decisions | Non-linear | Conditionals, flags |
| `Int` | Numeric computation | Non-linear | Arithmetic, indexing |
| `String` | Text processing | Non-linear | Messages, identifiers |
| `Symbol` | Atomic identifiers | Non-linear | Tags, labels, keys |
| `List` | Sequential data | Linear by default | Collections, sequences |
| `Map` | Associative data | Linear by default | Dictionaries, indices |
| `Record` | Structured data | Linear by default | Objects, entities |
| `ResourceId` | Linear resource handle | Always linear | Assets, capabilities |
| `ExprId` | Code references | Non-linear | Metaprogramming, compilation |

The distinction between `Map` and `Record` serves both semantic and optimization purposes:
- **Maps** are dynamic associative arrays with runtime key lookup
- **Records** are static structures with compile-time field access
- This enables different optimization strategies and type checking approaches

## 4. `Expr`: Abstract Syntax Tree

The `Expr` enum defines the grammatical structure of Causality Lisp programs. Each variant corresponds to a core language construct with specific semantic properties.

```rust
pub enum Expr {
    // Core Values & Variables
    Const(LispValue),                       // Literal constants
    Var(String),                            // Variable references

    // General Programming Constructs  
    Let(String, Box<Expr>, Box<Expr>),      // Local bindings

    // Layer 1 Primitives (Linear Lambda Calculus)
    UnitVal,                                // Unit introduction
    LetUnit(Box<Expr>, Box<Expr>),          // Unit elimination
    Tensor(Box<Expr>, Box<Expr>),           // Pair introduction
    LetTensor(String, String, Box<Expr>, Box<Expr>), // Pair elimination
    Inl(Box<Expr>),                         // Left sum injection
    Inr(Box<Expr>),                         // Right sum injection
    Case(Box<Expr>, String, Box<Expr>, String, Box<Expr>), // Sum elimination
    Lambda(Vec<String>, Box<Expr>),         // Function abstraction
    Apply(Box<Expr>, Vec<Expr>),            // Function application
    Alloc(Box<Expr>),                       // Resource allocation
    Consume(Box<Expr>),                     // Resource consumption
}
```

### Primitive Design Rationale

Each primitive serves a specific role in the linear lambda calculus foundation:

#### Unit Types (`UnitVal`, `LetUnit`)
- **Purpose**: Represent computations that produce no useful value
- **Linearity**: Unit values are non-linear (can be freely copied/discarded)
- **Use Cases**: Sequencing effects, initialization, control flow
- **Design Choice**: Explicit elimination (`LetUnit`) makes sequencing visible in the type system

#### Tensor Products (`Tensor`, `LetTensor`) 
- **Purpose**: Combine multiple values into a single compound value
- **Linearity**: If either component is linear, the tensor is linear
- **Use Cases**: Multiple return values, structured data, state aggregation
- **Design Choice**: Symmetric elimination requires both components to be used

#### Sum Types (`Inl`, `Inr`, `Case`)
- **Purpose**: Represent choice between alternative values
- **Linearity**: Sum values inherit the linearity of their components
- **Use Cases**: Error handling, variant types, conditional logic
- **Design Choice**: Exhaustive pattern matching ensures all cases are handled

#### Function Types (`Lambda`, `Apply`)
- **Purpose**: First-class functions enable abstraction and composition
- **Linearity**: Functions can capture linear values in their closures
- **Use Cases**: Abstractions, callbacks, higher-order programming
- **Design Choice**: Multi-argument application reduces syntactic overhead

#### Resource Management (`Alloc`, `Consume`)
- **Purpose**: Explicit resource lifecycle management
- **Linearity**: Resources are always linear; allocation/consumption preserves linearity
- **Use Cases**: Asset management, capability tracking, state encapsulation
- **Design Choice**: Explicit operations make resource usage visible and auditable

### Syntax Design Principles

The syntax follows several key principles:

1. **Uniformity**: All operations follow consistent prefix notation
2. **Explicitness**: Resource operations are always visible
3. **Compositionality**: Expressions compose naturally without hidden dependencies
4. **Minimalism**: No syntactic sugar that obscures the underlying semantics

## 5. Type System and Compilation Pipeline

The type system serves as the central mechanism for enforcing linearity constraints and enabling safe compilation to register machine code.

### Type Checking Phases

1. **Parsing**: Convert textual syntax into `Expr` AST
2. **Name Resolution**: Resolve variable references and check scope
3. **Linearity Analysis**: Track resource usage and enforce linear constraints  
4. **Type Inference**: Infer types for all expressions
5. **Capability Checking**: Verify row type constraints and permissions
6. **Code Generation**: Compile type-checked AST to register instructions

### Linearity Enforcement

The type checker tracks the usage of each variable through the computation:

```rust
// Example: This would be rejected by the type checker
let resource = alloc(my_data);
let result1 = consume(resource);  // ✓ First use is valid
let result2 = consume(resource);  // ✗ Error: resource used after consumption
```

The type system maintains an environment that tracks:
- **Available variables**: What variables are in scope
- **Linear variables**: Which variables must be consumed exactly once
- **Consumed variables**: Which linear variables have already been used

### Row Type Integration

Row types enable extensible records with compile-time capability checking:

```lisp
; Record with specific capabilities
(let ((user-account (record 
  ("balance" 1000)
  ("read-capability" true)
  ("write-capability" false))))
  
  ; This would be checked at compile time
  (if (record-has-capability user-account "write-capability")
    (update-balance user-account 500)  ; ✓ Capability verified
    (error "Insufficient permissions"))) ; ✗ No write capability
```

### Compilation Strategy

The compiler translates high-level constructs into register machine instructions using a systematic approach:

1. **Register Allocation**: Assign variables to machine registers
2. **Instruction Selection**: Choose appropriate machine instructions for each primitive
3. **Control Flow**: Handle conditionals and function calls
4. **Resource Tracking**: Ensure linear resources are properly managed
5. **Optimization**: Apply register-level optimizations while preserving semantics

This compilation strategy ensures that the high-level safety guarantees of Causality Lisp are preserved at the register machine level while enabling efficient execution.

## 6. Effect System Integration

Causality Lisp integrates effects as first-class language constructs, enabling controlled interaction with external systems while maintaining purity guarantees.

### Effect Declaration

Effects are declared with explicit signatures and handled through dedicated constructs:

```lisp
; Declare an effect type
(effect Transfer
  (transfer amount from to)
  → TransferReceipt)

; Use an effect in computation
(defn make-payment (amount sender receiver)
  (perform Transfer (transfer amount sender receiver)))
```

### Handler Composition

Effect handlers can be composed to create complex interaction patterns:

```lisp
; Log all transfer operations
(with-handler
  (Transfer (lambda (transfer-op)
    (do
      (log "Transfer initiated: " transfer-op)
      (resume transfer-op))))
  
  ; Execute computation with logging
  (make-payment 100 alice bob))
```

This design enables separation of concerns between pure computation and effectful operations while maintaining type safety and compositional reasoning.

## 7. Session Types Integration

Causality Lisp integrates session types as first-class language constructs at Layer 2, providing type-safe communication protocols with automatic duality checking. Session types complement effects and intents to form the complete Layer 2 programming model.

### Session Type Declaration

Session types are declared with explicit role specifications and automatic duality verification:

```lisp
; Declare a session type with two roles
(def-session PaymentProtocol
  (client !Amount ?Receipt End)
  (server ?Amount !Receipt End))  ; Automatically verified as dual

; Multi-party session with three roles
(def-session EscrowProtocol
  (buyer !Item ?Quote !Payment ?Confirmation End)
  (seller ?Item !Quote ?Payment !Delivery End)
  (arbiter ?Payment !Payment ?Confirmation !Delivery End))
```

### Session Type Syntax in Expressions

Session operations are integrated into the expression system:

```rust
pub enum Expr {
    // ... existing variants ...
    
    // Session type declarations
    SessionDeclaration {
        name: String,
        roles: Vec<SessionRole>,
    },
    
    // Session usage
    WithSession {
        session: String,
        role: String,
        body: Box<Expr>,
    },
    
    // Session operations
    SessionSend { channel: Box<Expr>, value: Box<Expr> },
    SessionReceive { channel: Box<Expr> },
    SessionSelect { channel: Box<Expr>, choice: String },
    SessionCase { channel: Box<Expr>, branches: Vec<SessionBranch> },
}
```

### Session Primitives

Causality Lisp provides dedicated primitives for session operations:

| Primitive | Syntax | Purpose | Type Signature |
|-----------|--------|---------|----------------|
| `def-session` | `(def-session name roles...)` | Declare session protocol | Creates session type |
| `with-session` | `(with-session protocol.role as var body)` | Create session context | Session scope |
| `session-send` | `(session-send channel value)` | Send value through channel | `!T.S → T → S` |
| `session-recv` | `(session-recv channel)` | Receive value from channel | `?T.S → (T × S)` |
| `session-select` | `(session-select channel choice)` | Select branch in protocol | `S₁ ⊕ S₂ → String → Sᵢ` |
| `session-case` | `(session-case channel branches...)` | Handle incoming choices | `S₁ & S₂ → Handlers → T` |

### Session Usage Examples

Session types enable type-safe communication protocols:

```lisp
; Simple payment protocol usage
(defn handle-payment-client (amount)
  (with-session PaymentProtocol.client as client-chan
    (do
      ; Send payment amount
      (session-send client-chan amount)
      
      ; Receive receipt
      (let ((receipt (session-recv client-chan)))
        (process-receipt receipt)))))

; Server-side payment handling
(defn handle-payment-server ()
  (with-session PaymentProtocol.server as server-chan
    (do
      ; Receive payment amount
      (let ((amount (session-recv server-chan)))
        (let ((receipt (generate-receipt amount)))
          ; Send receipt back
          (session-send server-chan receipt))))))
```

### Session Types with Choice Operations

Session types support choice operations for branching protocols:

```lisp
; Protocol with choices
(def-session NegotiationProtocol
  (proposer !Offer (?Counter !Accept End ⊕ ?Accept End))
  (acceptor ?Offer (!Counter ?Accept End ⊕ !Accept End)))

; Proposer implementation
(defn make-proposal (initial-offer)
  (with-session NegotiationProtocol.proposer as prop-chan
    (do
      ; Send initial offer
      (session-send prop-chan initial-offer)
      
      ; Handle response
      (session-case prop-chan
        (Counter counter-offer ->
          (session-send prop-chan (accept-counter counter-offer)))
        (Accept acceptance ->
          (finalize-deal acceptance))))))
```

### Integration with Effects and Intents

Session types compose naturally with effects and intents:

```lisp
; Session-based intent
(intent "PaymentRequest"
  (requires-session PaymentProtocol.client)
  (input-resource "amount" int)
  (constraint (> amount 0))
  (effect
    (with-session PaymentProtocol.client as client
      (bind
        (session-send client amount)
        (session-recv client)))))

; Session effect handlers
(handle-session-effect PaymentProtocol.server
  (session-recv amount ->
    (perform DatabaseWrite (log-payment amount))
    (let ((receipt (generate-receipt amount)))
      (session-send receipt))))
```

### Choreography Support

Causality Lisp supports choreographies for multi-party coordination:

```lisp
; Define a choreography
(choreography EscrowChoreography
  (roles buyer seller arbiter)
  (protocol
    ; Initial negotiation
    (buyer → seller: !ItemRequest)
    (seller → buyer: !ItemDetails)
    
    ; Escrow setup
    (buyer → arbiter: !EscrowRequest)
    (seller → arbiter: !ItemConfirmation)
    
    ; Payment and delivery
    (buyer → arbiter: !Payment)
    (seller → arbiter: !DeliveryProof)
    
    ; Resolution
    (arbiter → buyer: (!ItemReceived ⊕ !Dispute))
    (arbiter → seller: (!PaymentRelease ⊕ !PaymentWithhold))))

; Implement buyer role from choreography
(defn buyer-escrow-implementation (item-request payment)
  (with-choreography EscrowChoreography.buyer as buyer-role
    (do
      ; Follow choreography protocol
      (session-send buyer-role item-request)
      (let ((item-details (session-recv buyer-role)))
        (session-send buyer-role (create-escrow-request item-details))
        (session-send buyer-role payment)
        
        ; Handle final resolution
        (session-case buyer-role
          (ItemReceived receipt -> (complete-purchase receipt))
          (Dispute details -> (initiate-dispute-resolution details)))))))
```

### Session Type Safety Properties

Session types in Causality Lisp provide strong safety guarantees:

1. **Protocol Compliance**: All session operations must follow the declared protocol
2. **Duality Verification**: Communication partners automatically have compatible protocols
3. **Deadlock Freedom**: Well-typed session programs cannot deadlock
4. **Linearity Preservation**: Session channels are linear resources that cannot be duplicated
5. **Type Safety**: Communication values are statically type-checked

### Compilation to Layer 1

Session operations compile to Layer 1 linear lambda calculus:

```lisp
; Layer 2 session operation
(session-send channel value)

; Layer 1 compilation
(let ((old-channel (consume channel)))
  (let ((new-state (session-state-transition old-channel value "send")))
    (alloc new-state)))
```

This compilation strategy ensures that session types maintain all the linearity and verification properties of the underlying system while providing high-level communication abstractions.
