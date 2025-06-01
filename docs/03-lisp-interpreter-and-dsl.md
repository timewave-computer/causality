# Causality Lisp (Layer 1) and Register Machine (Layer 0) Execution

The Causality framework features a Lisp-based language at Layer 1, designed for functional programming with linear resources. This Layer 1 Causality Lisp compiles down to a minimal set of 9 instructions for a Typed Register Machine at Layer 0, which provides the fundamental execution substrate.

## 1. Layer 1 Causality Lisp: Data (`LispValue`) and Syntax (`Expr`)

Layer 1 Causality Lisp provides a higher-level programming interface. It defines its own set of data values (`LispValue`) and an Abstract Syntax Tree (`Expr`) structure that represents programs.

### 1.1. `LispValue`: Data in Causality Lisp (Layer 1)

`LispValue` instances represent the concrete data types manipulated within Layer 1 Causality Lisp programs. These are richer than the raw Layer 0 machine values.

```rust
pub enum LispValue {
    Unit,
    Bool(bool),
    Int(i64),
    String(Str),      // UTF-8 String, SSZ-compatible
    Symbol(Str),      // For symbolic atoms
    List(Vec<LispValue>), // Ordered list of LispValues
    Map(std::collections::HashMap<Str, LispValue>), // Key-value map
    Record(std::collections::HashMap<Str, LispValue>), // Structured record with named fields
    
    ResourceId(u64),  // Opaque reference to a Layer 0 resource
    ExprId(u64),      // Opaque reference to a persisted Expr AST
    // Other EntityId variants can also be represented as needed.
}
```
- `LispValue` includes common data structures like strings, lists, maps, and records, suitable for general programming.
- These values can be embedded as constants in Layer 1 `Expr` ASTs.

### 1.2. `Expr`: Causality Lisp Abstract Syntax Tree (Layer 1)

The `Expr` enum defines the structure of Causality Lisp programs at Layer 1. Its variants correspond to the 11 core primitives of the Linear Lambda Calculus, plus common programming constructs.

```rust
// Represents a parameter in a lambda or let binding.
pub struct Param { pub name: Str, pub type_annot: Option<Str> } // Type annotation is for Layer 1 type checking

pub enum Expr {
    // Core Values & Variables
    Const(LispValue),         // Constant LispValue from Layer 1
    Var(Str),                 // Variable reference by name

    // General Programming Constructs
    Let(Str, Option<Str>, Box<Expr>, Box<Expr>), // let name: type = val_expr in body_expr

    // Layer 1 Primitives (Linear Lambda Calculus - 11 Primitives)
    UnitVal,                                 // Introduces the unit value: `unit`
    LetUnit(Box<Expr>, Box<Expr>),           // Eliminates unit: `letunit u = e1 in e2`
    Tensor(Box<Expr>, Box<Expr>),            // Introduces tensor product: `tensor e1 e2`
    LetTensor(Box<Expr>, Str, Str, Box<Expr>), // Eliminates tensor product: `lettensor (x,y) = e_pair in e_body`
    Inl(Box<Expr>),                          // Introduces sum (left injection): `inl e`
    Inr(Box<Expr>),                          // Introduces sum (right injection): `inr e`
    Case(Box<Expr>,                         // Eliminates sum: `case e_sum of inl x => e_left | inr y => e_right`
         Str, Box<Expr>,                  // x, e_left
         Str, Box<Expr>),                 // y, e_right
    Lambda(Vec<Param>, Box<Expr>),           // Introduces linear function: `lambda (p1:t1, ...) => body`
    Apply(Box<Expr>, Vec<Expr>),             // Eliminates linear function: `apply fn_expr arg_exprs`
    Alloc(Box<Expr>),                        // Resource allocation: `alloc e`
    Consume(Box<Expr>),                      // Resource consumption: `consume e`
}
```
- This `Expr` AST is type-checked at Layer 1 (enforcing linearity, row type conformity, etc.) and then compiled into Layer 0 register machine instructions.

## 2. The Typed Register Machine Execution Model (Layer 0)

The core execution environment is a **Typed Register Machine**. This model directly supports the linear nature of resources and provides a simple, verifiable execution substrate ideal for ZK-proof generation.

### 2.1. Abstract Machine State (Layer 0)

The register machine maintains the following state:

```rust
State = {
  registers: Map<RegisterId, Value>,        // Register file holding Layer 0 `Value`s
  heap: Map<ResourceId, (Value, bool)>,   // Resource heap: (ResourceValue: Layer 0 `Value`, Consumed: bool)
  pc: ProgramCounter,                       // Program Counter
  call_stack: Stack<Frame>,                 // Call stack for function calls
  // Effects and constraints are managed by Layer 2, though `perform` and `check` interact with them.
}
```
- `Value` here refers to the Layer 0 machine values (`Unit`, `Bool`, `Int`, `Symbol`, `RegisterId`, `ResourceId`, `Label`, `EffectTag`, `Product`, `Sum`).
- The heap stores the actual resource data (as a Layer 0 `Value`) and its consumption status.

### 2.2. Registers and Resource Handles

*   **Registers**: Each linear resource and intermediate value resides in a unique **register**, identified by a `RegisterId`. Registers are typed, ensuring that operations only apply to values of the correct type.
*   **Resource Handles**: When a resource is created or accessed, the machine operates on a **Resource Handle**. This handle is a linear reference to the resource's register. Using or transforming a handle consumes it, invalidating the previous reference and potentially producing new handles to the resulting resources in new registers. This enforces the **move-style dynamic allocation** semantics: resources are moved between conceptual registers, not copied or aliased.

### 2.3. Linear Intermediate Representation (Linear Register Machine)

Causality Lisp programs are compiled into a **Linear Intermediate Representation** (IR). This Register Machine is a sequence of low-level instructions that operate directly on registers. The Register Machine is designed to be minimal, deterministic, and easily translatable into ZK circuit constraints.

The instruction set has been refined to nine essential operations. This design balances expressiveness for complex protocols, minimality for ZK circuits, and a structure that reveals patterns about trust boundaries. The instructions are categorized as follows:

```rust
instr ::=
  // Core Computation
    move r₁ r₂                    ; Identity morphism (Move value between registers)
  | apply r_fn r_arg r_out        ; Function elimination (Function application)
  | alloc r_type r_val r_out      ; Resource introduction (Allocate resource)
  | consume r_resource r_out      ; Resource elimination (Consume resource)  
  | match r_sum r_l r_r l₁ l₂     ; Sum elimination (Pattern matching on sums)

  // Conditional Logic
  | select r_cond r_true r_false r_out  ; Conditional value selection

  // Witness Boundary
  | witness r_out                 ; Read from untrusted witness
  | check constraint              ; Verify constraint holds

  // Effects
  | perform effect r_out          ; Execute effect
```

Detailed instruction semantics:

*   **`move r₁ r₂`**: Moves the value from register `r₁` to register `r₂`. The source register `r₁` is invalidated after the move, enforcing linear consumption. This is the fundamental operation for transferring ownership of linear values.

*   **`apply r_fn r_arg r_out`**: Applies the function in register `r_fn` to the argument in register `r_arg`, placing the result in register `r_out`. For linear functions, the argument register is consumed. This instruction handles both primitive operations and user-defined functions.

*   **`alloc r_val r_out`**: Allocates the Layer 0 `Value` from register `r_val` on the heap. A unique `ResourceId` is generated, associated with the value, and this `ResourceId` is placed in register `r_out`. The resource is initially marked as not consumed. Type information associated with the resource is handled at Layer 1.

*   **`consume r_resource r_out`**: Consumes the resource referenced by the handle in `r_resource`, extracting its value and placing it in `r_out`. The resource is marked as consumed in the heap and cannot be accessed again.

*   **`match r_sum r_l r_r l_label r_label`**: Pattern matches on a Layer 0 `Value::Sum` in register `r_sum`. If `Inl(v)`, `v` is moved to `r_l` and PC jumps to `l_label`. If `Inr(v)`, `v` is moved to `r_r` and PC jumps to `r_label`. `r_sum` is consumed. This is Layer 0's sum elimination. Layer 1 `inl`/`inr` primitives compile to operations that construct these Layer 0 `Value::Sum` types, typically using `apply` with a built-in constructor function if not directly representable.

*   **`select r_cond r_true r_false r_out`**: Conditionally selects a value. If the boolean value in `r_cond` is true, the value from `r_true` is moved to `r_out`. Otherwise, the value from `r_false` is moved to `r_out`. Both `r_true` and `r_false` must hold values of the same type, and the chosen value's source register is consumed. This instruction provides conditional computation without the complexity of type constructors, mapping efficiently to circuit multiplexers.

*   **`witness r_out`**: Reads a value from an untrusted external witness and places it into register `r_out`. This instruction is fundamental for introducing external data into the trusted computational environment. It forms part of the "witness-check" pattern for secure interaction across trust boundaries, where data imported via `witness` is subsequently validated using `check`.

*   **`check constraint`**: Verifies that the given `constraint` (e.g., a boolean expression involving register values or system state) holds. If the constraint is violated, execution halts with an error. This instruction is crucial for validating data received via `witness` or ensuring invariants before/after operations.

*   **`perform effect r_out`**: Executes an effect. The effect (itself a structured data value) is first processed by pure handlers (Layer 2) for transformation, then by the stateful interpreter for execution. Results are placed in `r_out`. This is the bridge to external computation and complex state changes, including those that might result in the construction of sum types.

This refined set of nine instructions provides a powerful yet minimal foundation for linear resource management, expressive control flow (including conditional logic and sum type elimination), secure interaction with external data via the witness-check pattern, and robust effect handling. This design is optimized for verification and ZK circuit generation.

## 3. Mapping Layer 1 Lisp Primitives to Layer 0 Register Operations

Each of the 11 Layer 1 Causality Lisp primitives, represented as `Expr` AST nodes, is compiled into a sequence of the 9 Layer 0 register machine instructions. This compilation process bridges the gap between the higher-level functional representation and the low-level execution model.

Below are conceptual mappings for each Layer 1 primitive:

1.  **`UnitVal` (L1: `unit`)**
    *   Loads a Layer 0 `Value::Unit` into a designated register.
    *   Example L0: `move <literal_unit_value_source_or_constructor> r_out` (or effectively a no-op if `unit` is just a type concept and its value is implicit).

2.  **`LetUnit(e1, e2)` (L1: `letunit u = e1 in e2`)**
    *   Evaluate `e1` (result in `r_e1`).
    *   `check` if `r_e1` contains `Value::Unit` (or this is a type system check).
    *   Evaluate `e2`.

3.  **`Tensor(e1, e2)` (L1: `tensor e1 e2`)**
    *   Evaluate `e1` (result in `r_1`).
    *   Evaluate `e2` (result in `r_2`).
    *   `apply <product_constructor_fn> r_1 r_2 r_out` (where `r_out` now holds `Value::Product(Box(val1), Box(val2))`).
    *   Alternatively: `move r_1 r_pair_part1; move r_2 r_pair_part2` if products are implicit register pairs at L0 for some operations, though `Value::Product` is more explicit.

4.  **`LetTensor(e_pair, x, y, e_body)` (L1: `lettensor (x,y) = e_pair in e_body`)**
    *   Evaluate `e_pair` (result in `r_p`).
    *   `apply <product_destructor_fn> r_p r_x_val r_y_val` (extracting parts into new registers).
    *   `move r_x_val r_x` (bind to `x`).
    *   `move r_y_val r_y` (bind to `y`).
    *   Evaluate `e_body` in the new scope.
    *   `consume r_p` (if `e_pair` was a linear resource).

5.  **`Inl(e)` (L1: `inl e`)**
    *   Evaluate `e` (result in `r_val`).
    *   `apply <sum_inl_constructor_fn> r_val r_out` (where `r_out` holds `Value::Sum(SumVariant::Inl(Box(val)))`).

6.  **`Inr(e)` (L1: `inr e`)**
    *   Evaluate `e` (result in `r_val`).
    *   `apply <sum_inr_constructor_fn> r_val r_out` (where `r_out` holds `Value::Sum(SumVariant::Inr(Box(val)))`).

7.  **`Case(e_sum, x, e_left, y, e_right)` (L1: `case e_sum of inl x => e_left | inr y => e_right`)**
    *   Evaluate `e_sum` (result in `r_s`).
    *   `match r_s r_x_val r_y_val label_left_branch label_right_branch`.
    *   `label_left_branch:`
        *   `move r_x_val r_x` (bind to `x`).
        *   Evaluate `e_left`.
        *   `jump label_end_case`.
    *   `label_right_branch:`
        *   `move r_y_val r_y` (bind to `y`).
        *   Evaluate `e_right`.
    *   `label_end_case:`

8.  **`Lambda(params, body)` (L1: `lambda (p1:t1, ...) => body`)**
    *   This is primarily a compile-time operation creating a closure. The closure (function pointer + captured environment) is stored.
    *   When applied, the compiled body of the lambda is executed. The body itself is a sequence of L0 instructions.

9.  **`Apply(fn_expr, arg_exprs)` (L1: `apply fn_expr arg_exprs`)**
    *   Evaluate `fn_expr` to get a function closure/pointer (result in `r_fn`).
    *   Evaluate `arg_exprs` (results in `r_arg1`, `r_arg2`, ...).
    *   Set up arguments for the call (e.g., move to specific registers or stack locations).
    *   `apply r_fn r_actual_arg r_result` (if single argument) or a sequence for multiple arguments, potentially involving stack operations for a proper call frame.

10. **`Alloc(e)` (L1: `alloc e`)**
    *   Evaluate `e` (result in `r_val`).
    *   `alloc r_val r_resource_id` (Layer 0 `alloc` instruction).
    *   The register `r_resource_id` now holds the `ResourceId` for the allocated resource.

11. **`Consume(e)` (L1: `consume e`)**
    *   Evaluate `e` (which should yield a `ResourceId`, result in `r_resource_id`).
    *   `consume r_resource_id r_val_out` (Layer 0 `consume` instruction).
    *   The register `r_val_out` now holds the value of the consumed resource.

## 4. Computational Metering

Computation itself is treated as a linear resource, tracked via a **Compute Budget**. Each instruction executed on the Register Machine consumes a specific amount of this budget:

```rust
instruction_costs = {
  move: 1,
  apply: 5,
  alloc: 10,
  consume: 10,
  match: 3,
  select: 3,    // Placeholder: cost TBD, example value
  witness: 5,   // Placeholder: cost TBD, example value
  check: 20,
  perform: 50,
}
```

*   **Budget Allocation**: Programs receive a `ComputeBudget` resource upon invocation
*   **Consumption**: Each instruction decrements the budget by its cost
*   **Halting**: If the budget is exhausted, the machine halts with an error

## 5. Effect Handling Architecture (Layer 2 Integration)

The `perform` instruction integrates with the effect handling system:

1. **Effect Creation**: Effects are created as structured data with pre/post conditions and hints
2. **Handler Transformation**: Pure handlers (Layer 2) transform effects without side effects
3. **Interpreter Execution**: The stateful interpreter executes the transformed effects
4. **Result Production**: New resources or values are produced and placed in the output register

This separation ensures that effect transformation logic remains pure and composable while actual state changes are controlled and auditable.

## 6. Layer 0 Built-in Functions (Primitives/Combinators)

The Layer 0 `apply r_fn r_arg r_out` instruction is versatile. When `r_fn` refers to a user-defined lambda (compiled to L0 instructions), it executes that code. However, `r_fn` can also refer to built-in, low-level primitive functions (sometimes called combinators) provided by the register machine itself. These implement fundamental operations directly in the runtime, such as:

*   **Arithmetic Operations**: e.g., `add`, `subtract` on Layer 0 `Value::Int`.
*   **Logical Operations**: e.g., `and`, `or`, `not` on Layer 0 `Value::Bool`.
*   **Comparison Operations**: e.g., `equal`, `less_than` on compatible Layer 0 `Value`s.
*   **Layer 0 Value Constructors/Destructors**: If not directly handled by dedicated instructions, `apply` could be used with built-in functions to create/deconstruct `Value::Product` or `Value::Symbol`.

These built-in functions are distinct from the 11 Layer 1 Lisp primitives. The Layer 1 primitives are syntactic constructs that compile *to* sequences of Layer 0 instructions, which may include `apply` calls to these Layer 0 built-in functions.

## 7. Compiler and Interpreter Implementation

The system uses a two-tiered approach:

### 7.1. Causality Lisp Compiler (Layer 1 to Layer 0)
- **Parsing**: Parses Layer 1 Causality Lisp S-expressions into the Layer 1 `Expr` AST.
- **Type Checking (Layer 1)**: Performs type checking on the `Expr` AST, including:
    - Enforcing linearity rules (Linear, Affine, Relevant, Unrestricted).
    - Validating against `RowType` schemas for records and capabilities.
    - Resolving compile-time row operations (projection, restriction, etc.).
- **Compilation to Layer 0**: Translates the type-checked Layer 1 `Expr` AST into a linear sequence of Layer 0 register machine instructions.
- **Optimization**: May perform optimizations on the generated Layer 0 instruction sequence.

### 7.2. Register Machine Interpreter (Layer 0 Execution)
- Resides likely in `causality-core` or a dedicated `causality-runtime`.
- Executes the sequence of 9 Layer 0 instructions.
- Manages the Layer 0 machine state (registers, heap, PC, stack).
- Handles resource allocation (`alloc`) and consumption (`consume`) on the heap.
- Enforces computational metering (budget checks).
- Interfaces with the Layer 2 effect system when a `perform` instruction is encountered, passing control to the appropriate effect handlers and interpreter.

## 8. Integration with Resource Model

The register machine is deeply integrated with the resource model:

*   **Resource Allocation**: The `alloc` instruction creates new resources with unique IDs
*   **Linear Consumption**: The `consume` instruction enforces one-time use
*   **State Validation**: Resources can have `static_expr` validation rules executed via register machine
*   **Capability Management**: Row-typed capabilities are managed through register operations
*   **Effect Resources**: Effects themselves are resources processed by the machine

## 9. Layer 1 Causality Lisp S-Expression Syntax Examples

This section provides examples of the S-expression syntax for some of the Layer 1 Lisp primitives and constructs.

```lisp
;; Let binding
(let ((x :Int 10))
  (let ((y :Int 20))
    ;; ... body using x and y
    ))

;; Unit type
(unit) ; value
(letunit u = (unit) in (do-something)) ; eliminator

;; Tensor product (pair)
(tensor x y) ; constructor
(lettensor (a b) = my_pair in
  (use a b))

;; Sum type (variant)
(inl my_value) ; left injection
(inr other_value) ; right injection
(case my_sum_value
  ((left_val) => (handle-left left_val))
  ((right_val) => (handle-right right_val)))

;; Lambda (linear function)
(lambda ((arg1 :Type1) (arg2 :Type2)) :ReturnType
  (body-expression arg1 arg2))

;; Application
(apply my_function arg1 arg2)

;; Resource management
(alloc (MyRecord "field1" 123)) ; allocate a record as a resource
(consume my_resource_handle)

;; Example using a let to bind result of alloc
(let ((new_res :MyResourceType (alloc (MyData "initial"))))
  (lettensor (val cap) = (consume new_res) in 
    ;; ... use val and cap (assuming consume returns a pair)
    (unit) ; final expression often unit if all resources consumed
  ))
```

S-expressions provide the human-readable syntax compiled to Linear IR:

```clojure
; Simple arithmetic
(+ 1 2) ; Compiles to: apply + r1 r2 r3

; Resource consumption
(let ((value (consume my-resource)))
  (process value))
; Compiles to: consume r_resource r_value; apply process r_value r_result

; Pattern matching
(match result
  ((:ok value) (use value))
  ((:error msg) (handle-error msg)))
; Compiles to: match r_result r_ok r_err label_ok label_err

; Effect execution
(perform (Transfer :from alice :to bob :amount 100))
; Compiles to: perform r_effect r_result
```

## 10. Error Handling and Debugging

The register machine provides comprehensive error handling:

*   **Register Errors**: Invalid register access, use of consumed registers
*   **Resource Errors**: Double consumption, invalid resource handles
*   **Constraint Violations**: Failed `check` instructions
*   **Budget Exhaustion**: Computational limits exceeded
*   **Effect Failures**: Handler or interpreter errors

Debugging support includes register state inspection, execution traces, and constraint violation details.

## 11. Performance and Optimization

Performance optimizations include:

*   **Register Allocation**: Efficient register reuse for non-linear values
*   **Instruction Fusion**: Combining common instruction sequences
*   **Dead Code Elimination**: Removing unreachable instructions
*   **Constant Propagation**: Pre-computing known values
*   **ZK-Specific Optimizations**: Structuring Register Machine for efficient circuit generation
