# Causality Lisp Language Specification

**Version:** 2.0  
**Date:** December 2024  
**Status:** Three-Layer Architecture Aligned

## Table of Contents

1. [Overview](#overview)
2. [Language Philosophy](#language-philosophy)
3. [Three-Layer Architecture](#three-layer-architecture)
4. [Lexical Structure](#lexical-structure)
5. [Type System](#type-system)
6. [Expression Types](#expression-types)
7. [The 11 Core Primitives](#the-11-core-primitives)
8. [Effects and Handlers](#effects-and-handlers)
9. [Temporal Effect Graph](#temporal-effect-graph)
10. [Evaluation Semantics](#evaluation-semantics)
11. [Zero-Knowledge Integration](#zero-knowledge-integration)
12. [Formal Grammar](#formal-grammar)
13. [Examples](#examples)

## Overview

Causality Lisp is a **linear resource language** built upon a formal three-layer architecture. The language is grounded in category theory, with execution on a minimal nine-instruction register machine (Layer 0), structured types with row polymorphism (Layer 1), and declarative programs with algebraic effects (Layer 2). Resource linearity, handler/interpreter separation, and static verification form the core of the design.

### Key Characteristics

- **Linear-First**: Resources are consumed exactly once by default, with configurable linearity through Object types
- **Three-Layer Architecture**: Clear separation between computational substrate, type system, and intentional programs
- **Minimal Instruction Set**: Nine core instructions sufficient for all computation
- **Handler/Interpreter Separation**: Pure effect transformation vs stateful execution
- **Conservation Laws**: Built into the type system and register machine semantics
- **Structured Optimization**: Hints guide execution without affecting correctness
- **Homoiconic**: Code is data; data is computation; computation is resources
- **Content-Addressed**: Expressions have unique identifiers based on their content
- **Zero-Knowledge Native**: Designed for efficient ZK circuit compilation

## Language Philosophy

### The Core Insight: Data IS Computation IS Resources

In Causality Lisp, the distinction between data, computation, and resources disappears. This unity is fundamental:

```lisp
; This is simultaneously:
; - Data: describes a transfer
; - Computation: when evaluated, executes the transfer
; - Resource: can be consumed/transformed like any other resource
(:transfer 
  (:from (:account "alice" (:balance 100)))
  (:to (:account "bob" (:balance 50)))
  (:amount 25))
```

### Design Principles

1. **Linearity as Foundation**: Every value is linear by default. Resources must be consumed exactly once.
2. **Static Verification**: The type system catches resource safety violations at compile time.
3. **Minimal Core**: 11 primitives compile to nine register machine instructions.
4. **Handler/Interpreter Separation**: Pure transformations compose; stateful execution is isolated.
5. **Conservation Laws**: Value is neither created nor destroyed, only transformed.
6. **Declarative Intent**: Specify what, not how; the runtime synthesizes optimal execution.

## Three-Layer Architecture

### Layer 0: Core Computational Substrate

The foundation is a typed register machine with **9 core instructions**. All Layer 1 Causality Lisp programs, including its 11 core primitives, compile down to sequences of these instructions. This minimal set ensures a verifiable and efficient execution model, suitable for ZK circuit generation.

The 9 Layer 0 instructions are:
1.  **Load `rd, rs, offset`**: Loads a value from memory (identified by base address in register `rs` + `offset`) into destination register `rd`.
2.  **Store `rs_addr, offset, rs_val`**: Stores the value from register `rs_val` into memory (identified by base address in register `rs_addr` + `offset`).
3.  **Move `rd, rs`**: Moves the value from source register `rs` to destination register `rd`.
4.  **Call `target_reg`**: Calls a function whose address is in `target_reg`. The return address is typically pushed onto a call stack managed by convention or dedicated registers.
5.  **Return**: Returns from a function call. The program counter is typically restored from a call stack.
6.  **Alloc `rd, type_id_reg, data_reg`**: Allocates a new resource or object. `type_id_reg` holds an identifier for its type, and `data_reg` may point to its initial data or constructor arguments. The ID or memory address of the new entity is placed in `rd`.
7.  **Consume `rd, resource_id_reg`**: Consumes a linear resource. `resource_id_reg` identifies the resource. The result of consumption (e.g., extracted value, status) is placed in `rd`.
8.  **Perform `effect_id_reg, inputs_ptr_reg`**: Initiates a Layer 2 effect. `effect_id_reg` identifies the effect to be performed, and `inputs_ptr_reg` points to a structure containing the inputs for the effect.
9.  **Check `constraint_id_reg, inputs_ptr_reg`**: Verifies a constraint. `constraint_id_reg` identifies the constraint, and `inputs_ptr_reg` points to the relevant data needed for the check.

All computation ultimately compiles to these instructions, ensuring a minimal, verifiable core.

### Layer 1: Structured Types & Resource Semantics

Built on Layer 0, this layer provides:

- **Row Types**: Compile-time extensible records and capabilities
- **Object Types**: Resources with configurable linearity (Linear, Affine, Relevant, Unrestricted)
- **Type-level Operations**: Project, restrict, merge row types at compile time

```lisp
; Row types track capabilities at compile time
(deftype Token [capabilities : Row] = 
  (Object (:token Data) capabilities :linear))

; Compile-time capability extraction
(defn extract-transfer [token : (Token {transfer: TransferPerm | rest})]
  -> [(TransferPerm, Token rest)]
  (row-extract token :transfer))
```

### Layer 2: Intentional Programs & Effect Logic

The highest layer provides:

- **Effects**: Tagged operations with pre/post conditions and hints
- **Handlers**: Pure effect-to-effect transformations
- **Intents**: Declarative resource commitments with constraints
- **TEG**: Temporal Effect Graph for causal ordering

## Lexical Structure

### Comments

```lisp
; Single-line comment
(+ 1 2) ; Inline comment
```

### Identifiers

Valid identifiers exclude whitespace and delimiters `( ) [ ] ' " ;`:

```lisp
x
variable-name
*special-var*
has-capability?
->
```

### Literals

```lisp
42          ; Integer : Int
-17         ; Negative integer : Int
"hello"     ; String : String
true false  ; Booleans : Bool
nil         ; Nil : Unit
:keyword    ; Keyword/Symbol : Symbol
```

## Type System

### Base Types (Layer 0)

```lisp
τ ::= Unit | Bool | Int | Symbol           -- Primitives
    | τ₁ ⊗ τ₂                              -- Linear pair
    | τ₁ ⊕ τ₂                              -- Sum type
    | τ₁ ⊸ τ₂                              -- Linear function
    | Resource⟨τ⟩                          -- Linear resource
    | Object⟨τ,λ⟩                         -- Object with linearity λ
```

### Linearity Qualifiers

```lisp
λ ::= Linear         -- Use exactly once (default)
    | Affine         -- Use at most once  
    | Relevant       -- Use at least once
    | Unrestricted   -- Use any number of times
```

### Row Types (Layer 1)

Row types enable compile-time tracking of capabilities and extensible records:

```lisp
; Row type definition
(deftype Capabilities : Row = 
  {transfer: TransferPerm,
   balance: Balance,
   mint: MintPerm | ...})

; Row operations happen at compile time
(defn get-balance [token : (Token {balance: Balance | rest})]
  -> [(Balance, Token rest)]
  (let [[(balance : Balance, remaining : Token rest)] 
        (row-extract token :balance)]
    [balance remaining]))
```

### Object Types

Objects generalize resources with configurable linearity:

```lisp
; Linear by default (equivalent to Resource)
(deftype Token = (Object TokenData :linear))
; Type: Token ≅ Resource⟨TokenData⟩

; Shared read-only data
(deftype Config = (Object ConfigData :unrestricted))
; Type: Config : Object⟨ConfigData, Unrestricted⟩

; Optional capability
(deftype OptionalPerm = (Object Permission :affine))
; Type: OptionalPerm : Object⟨Permission, Affine⟩
```

## Expression Types

### Atomic Expressions

```lisp
42                  ; Integer atom : Int
"hello"             ; String atom : String
true                ; Boolean atom : Bool
nil                 ; Nil atom : Unit
:symbol             ; Symbol/Keyword : Symbol
```

### Compound Expressions

```lisp
(+ 1 2)             ; Function application : Int
[1 2 3]             ; List literal : List Int
{:key "value"}      ; Map/Record literal : {key: String}
(fn [x : Int] (+ x 1))    ; Lambda expression : Int ⊸ Int
'(a b c)            ; Quoted expression : Expr
```

## The 11 Core Primitives

All Layer 1 Causality Lisp operations are constructed from these 11 core primitives. These primitives, along with their structured data, are compiled by the `causality-lisp-compiler` into sequences of the 9 Layer 0 register machine instructions. This minimal set forms the verifiable foundation of the language.

1.  **`lambda (params...) body...`**
    *   Defines an anonymous function (closure).
    *   `params`: A list of parameter names.
    *   `body...`: One or more expressions forming the function's body.
    *   Example: `(lambda (x y) (+ x y))`

2.  **`app func args...`**
    *   Applies a function `func` to a list of arguments `args`.
    *   `func`: An expression that evaluates to a function (e.g., a `lambda` or a symbol bound to a function).
    *   `args...`: Expressions that evaluate to the arguments for the function.
    *   Example: `(app (lambda (x) (* x x)) 5)`  evaluates to `25`.

3.  **`let ((var1 val1) (var2 val2) ...) body...`**
    *   Creates local variable bindings.
    *   Each `(var val)` pair binds a variable `var` to the result of evaluating `val`.
    *   Bindings are typically sequential or parallel depending on the specific Lisp dialect's `let` semantics (Causality Lisp typically uses sequential binding for `let`, and might offer `let*` or `letrec` for other behaviors if needed, though these are not core primitives themselves).
    *   `body...`: Expressions evaluated in the environment with these bindings.
    *   Example: `(let ((a 10) (b (* a 2))) (+ a b))`

4.  **`if cond then-expr else-expr`**
    *   Conditional evaluation.
    *   `cond`: An expression that evaluates to a boolean.
    *   `then-expr`: Evaluated if `cond` is true.
    *   `else-expr`: Evaluated if `cond` is false.
    *   Example: `(if (> x 0) "positive" "non-positive")`

5.  **`quote datum`**
    *   Returns `datum` literally, without evaluating it. Often abbreviated as `'datum`.
    *   `datum`: Any Lisp data structure (atom, list, etc.).
    *   Example: `(quote (a b c))` evaluates to the list `(a b c)`. `'(+ 1 2)` evaluates to the list `(+ 1 2)`, not `3`.

6.  **`cons head tail`**
    *   Constructs a new pair (list cell) where `head` is the `car` and `tail` is the `cdr`.
    *   Fundamental for building lists.
    *   Example: `(cons 1 (cons 2 nil))` creates the list `(1 2)`.

7.  **`car pair`**
    *   Returns the first element (head) of a `pair`.
    *   Error if `pair` is not a pair or is `nil`.
    *   Example: `(car '(a b c))` evaluates to `a`.

8.  **`cdr pair`**
    *   Returns the rest of the list (tail) after the first element of a `pair`.
    *   Error if `pair` is not a pair or is `nil`.
    *   Example: `(cdr '(a b c))` evaluates to `(b c)`.

9.  **`nil? obj`**
    *   Tests if `obj` is the empty list (`nil`).
    *   Returns a boolean.
    *   Example: `(nil? '())` evaluates to `true`.

10. **`eq? obj1 obj2`**
    *   Tests for equality of basic Lisp atoms (e.g., symbols, numbers, booleans). For structured types, `eq?` typically checks for pointer equality (i.e., if `obj1` and `obj2` are the exact same object in memory).
    *   It does not perform deep structural comparison for lists or other compound types by default.
    *   Example: `(eq? 'a 'a)` evaluates to `true`. `(eq? (cons 1 2) (cons 1 2))` might be `false`.

11. **`primitive-op "op-name" args...`**
    *   Provides access to a set of predefined, built-in operations that are often implemented directly in terms of Layer 0 instructions or highly optimized runtime functions.
    *   `"op-name"`: A string identifying the specific primitive operation.
    *   `args...`: Arguments required by the operation.
    *   This is the primary mechanism for interacting with the underlying system, including:
        *   **Arithmetic operations**: `"+"`, `"-"`, `"*"`, `"/"`, `"="`, `"<"`, `">"`, etc.
        *   **Type predicates**: `"integer?"`, `"symbol?"`, `"pair?"`, `"resource?"`.
        *   **Resource and Object manipulation (Layer 0/1 interface)**: `"alloc-resource"`, `"consume-resource"`, `"read-field"`, `"write-field"`, `"get-capability"`.
        *   **Effect interaction (Layer 0/2 interface)**: `"perform-effect"`, `"check-constraint"`.
    *   Example: `(primitive-op "+" x y)`, `(primitive-op "alloc-resource" token-type initial-data)`.

These 11 primitives, combined with the ability to define and apply functions (`lambda`, `app`), create bindings (`let`), and control flow (`if`), form a Turing-complete language. The `primitive-op` is the gateway to specialized, low-level operations essential for the Causality framework's resource management and effect handling.

## Effects and Handlers

### Effect Definition (Layer 2)

Effects are structured operations with metadata:

```lisp
; Effect type signature
; defeffect : Symbol → List (Symbol × Type) → EffectDef
(defeffect Transfer [from : Address, to : Address, amount : Int]
  :pre (and (is-owner from token)
            (has-capability :transfer)
            (>= (balance from) amount))
  :post (and (= (balance from) (- (old-balance from) amount))
             (= (balance to) (+ (old-balance to) amount)))
  :hints [(batch-with same-target)
          (minimize latency)
          (prefer-domain :ethereum)])
; Type: Effect⟨{from: Address, to: Address, amount: Int}⟩
```

### Handler Definition

Handlers are pure transformations from one effect to another:

```lisp
; Handler type signature  
; defhandler : Symbol → Type → Type → (α ⊸ β) → Handler⟨α,β⟩
(defhandler batch-optimizer : Transfer → BatchedTransfer
  (fn [effect : Transfer] : BatchedTransfer
    (if (should-batch? effect)
      (create-batch effect)
      effect)))

; Handler composition
; compose : Handler⟨α,β⟩ → Handler⟨β,γ⟩ → Handler⟨α,γ⟩
(def optimized-handler : Handler⟨Transfer,OptimizedTransfer⟩
  (compose batch-optimizer 
           route-optimizer
           privacy-handler))
```

### Interpreter Separation

The interpreter executes effects, maintaining state:

```lisp
; Interpreter type signature
; definterpreter : Symbol → (Effect⟨α⟩ → State → State) → Interpreter⟨α⟩
(definterpreter transfer-interpreter
  (fn [effect : Transfer, state : State] : State
    (match effect
      [:transfer from to amount]
      (let [[sufficient? : Bool, new-state : State] 
            (check-and-debit state from amount)]
        (if sufficient?
          (credit new-state to amount)
          (error :insufficient-funds))))))
; Type: Interpreter⟨Transfer⟩
```

## Temporal Effect Graph

### Effect Nodes

Effects form nodes in a directed acyclic graph:

```lisp
; EffectNode type
(deftype EffectNode = 
  {:id : EffectId,
   :effect : Effect⟨α⟩,
   :pre : List Constraint,
   :post : List Constraint,
   :hints : List Hint})

(effect-node
  :id E1
  :effect (Transfer :from alice :to bob :amount 100)
  :pre [(>= (balance alice) 100)
        (has-capability alice :transfer)]
  :post [(= (balance alice) (- (prev-balance alice) 100))
         (= (balance bob) (+ (prev-balance bob) 100))]
  :hints [(minimize latency)])
; Type: EffectNode
```

### Causal Edges

Resource consumption creates causal dependencies:

```lisp
; Edge type
(deftype Edge = 
  {:from : EffectId,
   :to : EffectId,
   :type : EdgeType,
   :resource : ResourceId})

(edge :from E1 :to E2 
      :type :resource-flow
      :resource updated-alice-balance)
; Type: Edge
```

### TEG Properties

- **Linearity**: Each resource consumed exactly once
- **Causality**: Consumption creates irreversible time direction
- **Parallelism**: Independent branches can execute concurrently
- **Verification**: Pre/post conditions checked at each node

## Intent System

### Intent Definition

Intents declare desired outcomes without specifying execution:

```lisp
; Intent type
(deftype Intent = 
  {:resources : List Resource⟨α⟩,
   :constraint : Constraint,
   :effects : List Effect⟨β⟩,
   :hints : List Hint})

(intent
  :resources [alice-token : Token, transfer-capability : TransferPerm]
  :constraint (and (>= (amount alice-token) 100)
                   (before deadline))
  :effects [(Transfer :from alice :to bob :amount 100)]
  :hints [(minimize cost)
          (batch-with similar-transfers)])
; Type: Intent
```

### Flow Synthesis

The runtime synthesizes valid execution flows:

```lisp
; synthesize-flow : Intent → List Resource⟨α⟩ → List ValidFlow
(defn synthesize-flow [intent : Intent, available : List Resource⟨α⟩] 
  : List ValidFlow
  ; 1. Analyze intent requirements
  ; 2. Search for valid primitive sequences
  ; 3. Verify linear safety and conservation
  ; 4. Select optimal flow based on hints
  ...)
```

## Zero-Knowledge Integration

### Privacy Primitives

```lisp
; commit : τ → Commitment⟨τ⟩
(commit value)                      ; Create commitment

; prove : Constraint → Witness → ZKProof
(prove constraint witness)          ; Generate ZK proof

; verify : ZKProof → List PublicInput → Bool
(verify proof public-inputs)        ; Verify proof

; nullify : Resource⟨τ⟩ → Secret → Nullifier
(nullify resource secret)           ; Prevent double-spending
```

### Content-Addressed Optimization

Effects compile to minimal circuits through content addressing:

```lisp
; Instead of proving execution in-circuit
; prove-execution : Effect⟨α⟩ → ZKProof
(prove-execution (Mint :amount 1000))

; Prove hash membership
; prove-membership : Hash → MerkleRoot → ZKProof
(prove-membership 
  :effect-hash 0xabc123...
  :valid-effects merkle-root)
```

### Private Resource Types

```lisp
(deftype PrivateToken =
  (Object 
    {:commitment : Commitment⟨Balance⟩,
     :nullifier : Nullifier,
     :proof : TransferProof}
    :linear))
; Type: PrivateToken : Object⟨{...}, Linear⟩
```

## Evaluation Semantics

### Register Machine Execution

All expressions compile to register machine instructions:

```lisp
; High-level expression with types
(let [[token : Token (mint 100)]
      [cap : TransferPerm (extract token :transfer)]]
  (transfer token cap bob))

; Compiles to typed register operations:
; alloc r1 Token 100 r2       ; r2: Token
; apply extract r2 :transfer r3 r4  ; r3: TransferPerm, r4: Token
; apply transfer r4 bob r3 r5 ; r5: Receipt
```

### Linear Safety

The type system ensures linear safety at compile time:

```lisp
; This won't compile - double use
(let [[token : Token (mint 100)]]
  (transfer token alice 50)
  (transfer token bob 50))    ; ERROR: token : Token already consumed

; This works - proper splitting
(let [[token : Token (mint 100)]
      [[t1 : Token, t2 : Token] (split token 50)]]
  (transfer t1 alice 50)
  (transfer t2 bob 50))
```

### Conservation Verification

Conservation laws are checked at multiple levels:

```lisp
; conservation-flow : List Token → (List Token × List Fee) → Bool
(flow [input-tokens : List Token]
  -> [output-tokens : List Token, fees : List Fee]
  (let [[total-in : Int (sum-values input-tokens)]
        [total-out : Int (+ (sum-values output-tokens) 
                           (sum-values fees))]]
    (assert (= total-in total-out))))
```

## Formal Grammar

```ebnf
Program     := TopLevel*
TopLevel    := DefForm | Expression

DefForm     := DefEffect | DefHandler | DefType | DefIntent
DefEffect   := "(defeffect" Symbol Parameters Metadata ")"
DefHandler  := "(defhandler" Symbol ":" Type "→" Type Body ")"
DefType     := "(deftype" Symbol "=" TypeExpr ")"
DefIntent   := "(intent" IntentSpec ")"

Expression  := Atom | List | Quote | Primitive
Atom        := Integer | String | Boolean | Nil | Symbol | Keyword
List        := "(" Expression* ")" | "[" Expression* "]"
Quote       := "'" Expression
Primitive   := "(" PrimOp Expression* ")"

PrimOp      := "lambda" | "app" | "let" | "if" | "quote"
             | "cons" | "car" | "cdr" | "nil?" | "eq?"
             | "primitive-op"

TypeExpr    := BaseType | RowType | ObjectType | FunctionType
BaseType    := "Unit" | "Bool" | "Int" | "Symbol"
RowType     := "{" (Label ":" TypeExpr)* "|" "..." "}"
ObjectType  := "(Object" TypeExpr Linearity ")"
FunctionType:= "(→" TypeExpr TypeExpr ")"

Linearity   := ":linear" | ":affine" | ":relevant" | ":unrestricted"
```

## Examples

### Basic Resource Manipulation

```lisp
; create-and-consume : Unit → Unit
(defn create-and-consume [] : Unit
  (let ((token-id (primitive-op "alloc-resource" 'MyTokenType '{:amount 100 :capabilities {:transfer true :balance true}})))
    (primitive-op "consume-resource" token-id)))

; extract-and-transfer : TokenId → Receipt ; Assuming token-id is an identifier for a Token resource
(defn extract-and-transfer (token-id) : Receipt
  (let ((transfer-cap (primitive-op "read-field" token-id ':transfer)))
    ; execute-transfer is assumed to be a higher-level function or effect invocation.
    ; Internally, it might use: (primitive-op "perform-effect" 'transfer-effect-name { :from token-id :capability transfer-cap :to 'alice :recipient 'bob :amount 50 })
    (execute-transfer token-id transfer-cap 'alice 'bob 50)))
```

### Effect Definition and Handling

```lisp
; Swap effect with full type signature
(defeffect Swap [token-a : Token, token-b : Token, 
                 amount-a : Int, amount-b : Int]
  :pre (and (owns token-a) (owns token-b))
  :post (swapped token-a token-b)
  :hints [(atomic true)
          (minimize slippage)])
; Type: Effect⟨{token-a: Token, token-b: Token, amount-a: Int, amount-b: Int}⟩

; Handler with type signature
(defhandler swap-batcher : Swap → BatchedSwap
  (fn [swap : Swap] : BatchedSwap
    (if (< (swap-amount swap) MIN_BATCH_SIZE)
      (add-to-batch swap)
      (execute-immediately swap))))
; Type: Handler⟨Swap, BatchedSwap⟩
```

### Intent-Based Programming

```lisp
; swap-intent : Token → Token → Intent
(defn create-swap-intent [my-eth : Token, my-dai : Token] : Intent
  (intent
    :resources [my-eth my-dai]
    :constraint (and (= (amount my-eth) 10)
                     (>= (amount my-dai) 10000))
    :effects [(Swap :from my-eth :to dai-received :min-amount 10000)]
    :hints [(maximize dai-received)
            (deadline (+ now 300))]))
```

### Privacy-Preserving Transfer

```lisp
; private-transfer : TokenId → Address → Int → PrivateTransferProofResourceId
(defn private-transfer (token-id recipient amount) 
  : PrivateTransferProofResourceId ; Assuming it returns an ID to the proof resource
  (let* ((commitment-data {:amount amount :recipient recipient})
         (commitment (primitive-op "commit" commitment-data)) 
         (hashed-commitment (primitive-op "hash" commitment)) 
         (nullifier (primitive-op "nullify" token-id hashed-commitment)) 
         ; 'transfer-constraint' and witness structure are illustrative for the 'prove' op
         (proof (primitive-op "prove" 'transfer-constraint {:token token-id :amount amount :recipient recipient :commitment commitment})))
    (primitive-op "alloc-resource" 
                  'PrivateTransferProofType ; Symbolic type for the new resource
                  '{:commitment commitment ; These are vars from the let* bindings
                    :nullifier nullifier
                    :proof proof
                    :capabilities {:transfer-proof true}})))
```

### TEG Construction

```lisp
; build-payment-teg : List PaymentIntent → TEG
(defn build-payment-teg [payment-intents : List PaymentIntent] : TEG
  (let [[nodes : List EffectNode (map intent->effect-nodes payment-intents)]
        [edges : List Edge (compute-resource-dependencies nodes)]]
    (verify-acyclic edges)
    (verify-conservation nodes)
    (topological-sort nodes edges)))
```

### Typed Flow Definition

```lisp
; payment-flow : Token → Token → PaymentRequest → (Token × Token × Receipt)
(defflow payment-flow 
  [payer : Token, payee : Token, request : PaymentRequest]
  -> [updated-payer : Token, updated-payee : Token, receipt : Receipt]
  
  ; Extract capabilities with types
  (let [[balance : Balance, transfer : TransferPerm, rest : Token] 
        (extract-capabilities payer [:balance :transfer])]
    
    ; Validate payment
    (let [[valid-payment : ValidatedPayment] 
          (validate-payment-amount request balance)]
      
      ; Execute transfer
      (let [[p1 : Token, p2 : Token, r : Receipt] 
            (execute-transfer transfer valid-payment payee)]
        [p1 p2 r]))))
```

### Computational Metering

```lisp
; metered-computation : ComputeBudget → (α ⊸ β) → α → (β × ComputeBudget)
(defn metered-computation 
  [budget : ComputeBudget, f : (α ⊸ β), input : α] 
  : (β × ComputeBudget)
  (let [[cost : Int (estimate-cost f)]
        [remaining : ComputeBudget (subtract budget cost)]]
    (if (>= remaining 0)
      [(f input) remaining]
      (error :insufficient-budget))))
```

## Implementation Notes

### Compilation Strategy

1. Parse s-expressions into typed AST
2. Type check with row type inference
3. Transform effects into handler pipeline
4. Compile to register machine IR
5. Optimize based on hints

### Memory Model

- **Static register allocation** for compile-time known computations
- **Registers statically allocated** where possible, no aliasing
- **Dynamic allocation only when necessary** (loops, batching, runtime synthesis)
- **Resources stored in heap** with register IDs as handles
- **Registers invalidated on consumption** preventing reuse
- **No garbage collection** - explicit consumption required
- **Predictable memory layout** for ZK circuit generation

### Performance Optimizations

- Content-addressed effect caching
- Parallel TEG execution
- Batched effect handling
- Zero-copy resource transfers

### Type Inference

The compiler performs:
- Row type inference for capabilities
- Linearity inference for resources
- Effect type inference for handlers
- Constraint solving for intents

This specification defines Causality Lisp as a linear resource language built on a minimal, verifiable foundation. The three-layer architecture provides clear separation of concerns while enabling powerful abstractions for resource-safe programming with comprehensive type safety. 