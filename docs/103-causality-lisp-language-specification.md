# 103: Causality Lisp Language Specification

Causality Lisp is Layer 1 of the three-layer architecture, providing a linear functional programming language with content-addressed expressions that compiles to the 5 fundamental register machine instructions.

## Key Features

1. **Content-Addressed Expressions**: All expressions identified by `ExprId(EntityId)` enabling structural sharing
2. **Linear Type System**: Resource tracking with use-once semantics  
3. **Unified Compilation**: Compiles to 5 fundamental Layer 0 instructions
4. **Integration**: Seamless integration with Layer 2 transform-based effects

## Language Philosophy

Causality Lisp provides the structured type system and expression language that bridges Layer 0's minimal execution model with Layer 2's declarative programming. Every expression is content-addressed, enabling automatic optimization and verification.

### Content-Addressed Programming

```lisp
;; All expressions are content-addressed by their structure
(define transfer-logic
  (lambda (token amount recipient)
    (let ((balance (consume token)))
      (if (>= balance amount)
          (alloc (record (balance (- balance amount))
                        (owner recipient)))
          (error "insufficient-balance")))))

;; This expression has a deterministic ExprId based on its structure
;; Identical expressions automatically share the same hash
```

## 1. Overview

Causality Lisp is a statically-typed, linear functional programming language designed for expressing resource-aware computations. Every value in Causality Lisp is linear by default, meaning it must be consumed exactly once. This restriction enables compile-time verification of resource safety and forms the foundation for Causality's broader guarantees around correctness and conservation.

The language is built around content-addressed expressions that compile to the 5 fundamental Layer 0 instructions. These operations compile to a minimal set of register machine instructions based on symmetric monoidal closed category theory, ensuring that the high-level expressiveness of the language translates to efficient, verifiable execution.

### Key Features

- **Content-Addressed AST**: All expressions are content-addressed enabling structural sharing and optimization
- **Linear Type System**: All values are linear by default, preventing resource duplication and ensuring proper resource lifecycle management
- **Resource Management**: Direct integration with Layer 0's unified resource allocation and consumption model
- **Verifiable Compilation**: Transparent compilation to the 5 fundamental register machine instructions with formal correctness guarantees

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

3. **Unified Transform Core**: All operations compile to the 5 fundamental Layer 0 instructions (transform, alloc, consume, compose, tensor). This minimalism enables formal verification of the entire language implementation while providing sufficient expressiveness for real-world applications through location-transparent operations.

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

### Why Content-Addressed Expressions?

The design around content-addressed expressions provides several key benefits:

**Theoretical Foundation**: Content addressing enables:
- **Structural Sharing**: Identical subexpressions automatically share storage and computation
- **Global Optimization**: Compiler can optimize across expression boundaries
- **Verification**: Content hashes provide cryptographic guarantees of expression integrity
- **ZK Compatibility**: Fixed representations enable efficient arithmetic circuit compilation

**Practical Considerations**: This approach provides:
- **Automatic Optimization**: Common subexpressions are computed once and cached
- **Deterministic Builds**: Same source always produces same content hashes
- **Incremental Compilation**: Only changed expressions need recompilation
- **Verification**: Expression integrity can be verified cryptographically

### Why Compile to Register Machine?

The choice to compile to a register machine rather than other targets (stack machine, direct interpretation, native code) serves several purposes:

1. **Verification**: Register machines have well-understood formal semantics that enable verification
2. **Efficiency**: Direct register allocation eliminates many intermediate steps
3. **Transparency**: The compilation process is predictable and auditable
4. **Integration**: Register machines integrate naturally with ZK proof systems
5. **Parallelization**: Register-based code is easier to analyze for parallel execution

## 3. Content-Addressed AST and Compilation

### Expression Identity

All expressions in Causality Lisp are identified by their content hash:

```rust
pub struct ExprId(pub EntityId);  // Content hash of the expression structure
```

This enables:
- **Structural Sharing**: Identical expressions share the same `ExprId`
- **Global Optimization**: Compiler optimizations work across expression boundaries
- **Incremental Compilation**: Only modified expressions need recompilation
- **ZK Circuit Reuse**: Verified circuits can be cached and reused by content hash

### Compilation to Layer 0

Causality Lisp expressions compile to the 5 fundamental Layer 0 instructions:

| Lisp Construct | Layer 0 Instructions | Purpose |
|----------------|---------------------|---------|
| `alloc`, `consume` | `alloc`, `consume` | Resource management |
| `lambda`, `apply` | `compose` | Function composition |
| `tensor`, `lettensor` | `tensor` | Parallel composition |
| All other operations | `transform` | General computation |

### Example Compilation

```lisp
;; Source: Function that transfers tokens
(lambda (token amount)
  (let ((balance (consume token)))
    (alloc (record (balance (- balance amount))))))
```

Compiles to Layer 0 instructions:
```
transform consume_fn input_reg temp_reg    ; consume token  
transform subtract_fn amount_reg temp_reg  ; subtract amount
alloc record_type temp_reg output_reg      ; create new token
```

## 4. `LispValue`: Data Types

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
let result1 = consume(resource);  //  First use is valid
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
    (update-balance user-account 500)  ;  Capability verified
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

```