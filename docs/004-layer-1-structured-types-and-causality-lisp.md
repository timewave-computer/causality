# 004: Layer 1 - Linear Lambda Calculus

Layer 1 of the Causality architecture implements a **pure Linear Lambda Calculus** with exactly 11 core primitives. This layer serves as the mathematical foundation for safe, type-checked functional programming while maintaining the strict linearity guarantees essential to Causality's resource management model.

Building upon the minimal register machine of Layer 0, Layer 1 provides the mathematical abstractions necessary for expressing resource transformations in a type-safe, verifiable manner. The layer is specifically designed to compile to fixed-size zero-knowledge circuits while maintaining the expressive power needed for practical applications.

**Architectural Focus**: Layer 1 maintains mathematical purity by focusing exclusively on linear lambda calculus foundations. Complex operations like capability-based field access, object models, and record operations are handled at Layer 2 as effects, ensuring Layer 1 remains amenable to formal verification and ZK circuit generation.

## 1. Mathematical Foundation

Layer 1 is grounded in **Linear Type Theory**, specifically implementing the term model of a **Symmetric Monoidal Closed Category with Coproducts**. This mathematical foundation ensures that:

- Every operation has a precise categorical interpretation
- Resource linearity is enforced by construction
- Composition and equivalence laws hold by design
- The system is amenable to formal verification
- All operations compile to fixed-size ZK circuits

### 1.1 Pure Type System

```
τ ::= 1                     -- Unit type (terminal object)
    | Bool | Int | Symbol   -- Base types
    | τ₁ ⊗ τ₂              -- Tensor product (monoidal product)
    | τ₁ ⊕ τ₂              -- Sum type (coproduct)
    | τ₁ ⊸ τ₂              -- Linear function (internal hom)
    | Resource τ            -- Linear resource handle
```

**Key Design Principles**:
- **No polymorphism**: All types are monomorphic for ZK compatibility
- **Static structure**: All type layouts determined at compile time

### 1.2 Linearity Constraints

The type system enforces **linear resource usage** without exception:
- Every resource must be used exactly once
- No implicit copying or duplication
- No implicit disposal or garbage collection
- Clear resource lifecycle from allocation to consumption
- Linear context splitting enforced categorically

## 2. The 11 Core Primitives

Layer 1's design philosophy centers on **mathematical minimalism**: providing exactly the primitives needed to form a complete computational basis while maintaining clean categorical semantics. These 11 operations correspond directly to the fundamental constructors and eliminators from linear type theory.

**ZK Circuit Compatibility**: Each primitive is specifically chosen to compile to fixed-size circuit components, enabling efficient zero-knowledge proof generation without dynamic structures or runtime polymorphism.

### 2.1 Unit Type (Terminal Object)

| Primitive | Type | Categorical Role | Purpose |
|-----------|------|------------------|---------|
| `unit` | 1 | Terminal object | Represents "no useful information" |
| `letunit` | 1 ⊗ (1 ⊸ A) ⊸ A | Terminal elimination | Sequential composition after unit |

**Usage**:
```lisp
unit                                    ; Create unit value
(letunit () = unit-expr in body-expr)  ; Eliminate unit, continue with body
```

### 2.2 Tensor Product (Monoidal Product ⊗)

| Primitive | Type | Categorical Role | Purpose |
|-----------|------|------------------|---------|
| `tensor` | A ⊸ B ⊸ (A ⊗ B) | Monoidal product | Combine resources into pairs |
| `lettensor` | (A ⊗ B) ⊸ (A ⊸ B ⊸ C) ⊸ C | Product elimination | Decompose pairs linearly |

**Usage**:
```lisp
(tensor resource1 resource2)                    ; Create pair
(lettensor (x, y) = pair-expr in body-expr)     ; Destructure pair
```

### 2.3 Sum Type (Coproduct ⊕)

| Primitive | Type | Categorical Role | Purpose |
|-----------|------|------------------|---------|
| `inl` | A ⊸ (A ⊕ B) | Left injection | Create tagged union (left variant) |
| `inr` | B ⊸ (A ⊕ B) | Right injection | Create tagged union (right variant) |
| `case` | (A ⊕ B) ⊗ (A ⊸ C) ⊗ (B ⊸ C) ⊸ C | Coproduct universal property | Pattern match on sum types |

**Usage**:
```lisp
(inl value)                                     ; Create left variant
(inr value)                                     ; Create right variant
(case sum-expr                                  ; Pattern match
  [(inl x) => left-body]
  [(inr y) => right-body])
```

### 2.4 Linear Functions (Internal Hom ⊸)

| Primitive | Type | Categorical Role | Purpose |
|-----------|------|------------------|---------|
| `lambda` | (Meta-operation) | Internal hom constructor | Create linear functions |
| `apply` | (A ⊸ B) ⊗ A ⊸ B | Internal hom eliminator | Function application |

**Usage**:
```lisp
(lambda (x : τ) => body-expr)                   ; Function abstraction
(apply function-expr argument-expr)             ; Function application
```

### 2.5 Resource Management

| Primitive | Type | Categorical Role | Purpose |
|-----------|------|------------------|---------|
| `alloc` | A ⊸ Resource A | Resource constructor | Move value to heap, create handle |
| `consume` | Resource A ⊸ A | Resource eliminator | Extract value, enforce single-use |

**Usage**:
```lisp
(alloc value-expr)                              ; Allocate resource on heap
(consume resource-expr)                         ; Consume resource, extract value
```

## 3. Compilation to Layer 0

Each Layer 1 primitive compiles to a sequence of Layer 0 instructions, maintaining semantic equivalence while enabling low-level execution:

### 3.1 Compilation Examples

```lisp
;; Layer 1: Create and consume a resource
(consume (alloc 42))

;; Compiles to Layer 0:
; alloc r_val r_resource    ; Create resource
; consume r_resource r_out  ; Extract value
```

```lisp
;; Layer 1: Function application
(apply increment-fn 5)

;; Compiles to Layer 0:
; apply r_fn r_arg r_out    ; Direct instruction mapping
```

```lisp
;; Layer 1: Pattern matching
(case either-value
  [(inl x) => (process-left x)]
  [(inr y) => (process-right y)])

;; Compiles to Layer 0:
; match r_sum r_l r_r left_label right_label
; left_label:
;   ; code for process-left
; right_label:
;   ; code for process-right
```

### 3.2 Circuit Generation

The compilation process ensures that resulting Layer 0 code has the properties required for ZK circuit generation:

- **Fixed Structure**: All data layouts determined at compile time
- **Static Control Flow**: All branches and loops unrolled or bounded
- **Deterministic Execution**: Same inputs produce identical circuit execution
- **Resource Bounds**: Memory usage statically analyzable

## 4. Type Safety and Linearity

### 4.1 Linear Typing Rules

The type system enforces linearity through careful management of typing contexts:

```
Γ ::= ∅ | Γ, x:τ                    -- Linear contexts
Γ₁ ∪ Γ₂ where Γ₁ ∩ Γ₂ = ∅           -- Context splitting
```

**Key Typing Rules**:

```
───────── (Unit)
Γ ⊢ unit : 1

Γ, x:τ₁ ⊢ e : τ₂
────────────────── (Lambda)
Γ ⊢ λx:τ₁.e : τ₁ ⊸ τ₂

Γ₁ ⊢ e₁ : τ₁ ⊸ τ₂    Γ₂ ⊢ e₂ : τ₁    Γ₁ ∩ Γ₂ = ∅
─────────────────────────────────────────────── (Apply)
Γ₁ ∪ Γ₂ ⊢ e₁ e₂ : τ₂
```

### 4.2 Resource Lifecycle

Resources have a clear, enforced lifecycle:

1. **Creation**: `alloc` moves a value to the heap, creating a linear handle
2. **Transformation**: Functions can be applied to transform resource contents
3. **Consumption**: `consume` extracts the value and invalidates the handle

This lifecycle prevents common resource management errors:
- **Double-spending**: Resources cannot be consumed twice
- **Resource leaks**: All allocated resources must eventually be consumed
- **Use-after-free**: Consumed resources cannot be accessed again

## 5. Integration with Higher Layers

### 5.1 Layer 2 Compilation Target

Layer 2 effect operations compile down to Layer 1 expressions. The compilation process performs **effect resolution** that transforms complex Layer 2 operations into pure Layer 1 primitives:

**Example: Capability-based field access**
```rust
// Layer 2 Effect
access_field(account_resource, "balance", read_capability)

// Compiles to Layer 1 (after capability resolution)
lettensor (account_data, metadata) = consume(account_resource) in
lettensor (balance, other_fields) = account_data in
alloc(balance)  // Return balance as new resource
```

**Example: Record update with capabilities**
```rust
// Layer 2 Effect  
update_field(record_resource, "amount", new_value, write_capability)

// Compiles to Layer 1 (after schema monomorphization)
lettensor (old_amount, other_data) = consume(record_resource) in
tensor(new_value, other_data)  // Reconstruct with new value
```

### 5.2 Module Organization

Layer 1 is implemented in the `causality-core/src/lambda` module with the following structure:

```
lambda/
├── base.rs         # Core types (BaseType, Value, etc.)
├── linear.rs       # Linearity system and tracking
├── tensor.rs       # Tensor product implementation  
├── sum.rs          # Sum type implementation
├── function.rs     # Linear function implementation
├── symbol.rs       # Symbol type
├── term.rs         # AST and term representation
└── interface.rs    # Layer 0 compilation interface
```

## 6. ZK Circuit Properties

Layer 1's design ensures compatibility with zero-knowledge proof systems:

### 6.1 Static Analysis Properties

- **Monomorphic Types**: All polymorphism resolved at Layer 2
- **Fixed Memory Layout**: All data structures have compile-time-known sizes
- **Bounded Computation**: No unbounded loops or recursion
- **Deterministic Control Flow**: All branches statically analyzable

### 6.2 Circuit Generation

```
Layer 1 Term → Circuit Components

unit         → Identity circuit
tensor       → Parallel composition
case         → Conditional circuit with both branches
lambda/apply → Circuit with input/output wires
alloc/consume → Memory allocation circuits
```

This ensures that every Layer 1 program can be compiled to a zero-knowledge circuit with predictable size and structure, enabling efficient proof generation while maintaining the expressive power needed for real-world applications.
