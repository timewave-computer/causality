# 003: Layer 0 - Verifiable Machine Execution

Layer 0 forms the absolute bedrock of the Causality architecture. It defines a minimal, deterministic **Typed Register Machine** that serves as the fundamental execution substrate for all operations within the system. Its design prioritizes simplicity, verifiability, and efficiency, making it an ideal target for compilation from higher-level languages like Causality Lisp and for generating Zero-Knowledge (ZK) proofs of execution.

This layer is where the core principles of linearity and deterministic state transition are most rigorously enforced.

## 1. The Typed Register Machine Model

The Layer 0 execution environment is characterized by:

-   **Registers**: Unique storage locations (`RegisterId`) that hold intermediate values or handles to resources. Each register is typed, ensuring that operations are only applied to compatible data.
-   **Resource Handles**: Linear references to resources stored on the heap. Operations on these handles consume them, reflecting the move-style semantics of resource management: resources are moved between registers, not copied or aliased.
-   **Heap**: A memory area (`Map<ResourceId, (Value, bool)>`) where actual resource data (`Value`) is stored, along with a flag indicating whether the resource has been consumed.
-   **Call Stack**: A stack for managing function calls and returns, essential for user-defined functions and ZK-proof compatibility.

### 1.1. Abstract Machine State

The state of the Layer 0 register machine at any point can be described by:

-   `registers`: A map from `RegisterId` to Layer 0 `Value`s.
-   `heap`: The resource heap, mapping `ResourceId` to its `Value` and consumption status.
-   `pc`: The Program Counter, indicating the next instruction to execute.
-   `call_stack`: A stack of frames for managing function calls and returns.

Layer 0 `Value`s are primitive types such as `Unit`, `Bool`, `Int`, `Symbol`, `RegisterId`, `ResourceId`, `Label` (for jumps), `EffectTag`, `Product` (for pairs), and `Sum` (for variants).

## 2. The Linear Register Machine Instruction Set

Causality Lisp programs (Layer 1) are compiled into a Linear Intermediate Representation (IR) consisting of a sequence of low-level instructions for the Layer 0 register machine. This instruction set is deliberately minimal, comprising eleven essential operations. This design balances the expressiveness needed for complex protocols with the simplicity required for efficient ZK circuit generation and formal verification.

The eleven instructions include the original nine core operations plus two additional instructions (`LabelMarker` and `Return`) that were added to support user-defined function calls in a zero-knowledge proof compatible manner. These additions enable functions to reference code locations via labels rather than embedding instruction sequences directly, which is crucial for efficient ZK circuit generation.

The eleven instructions are:

1.  **`move r₁ r₂`**
    *   **Semantics**: Moves the value from register `r₁` to register `r₂`. `r₁` is invalidated, enforcing linear consumption. This is the fundamental operation for transferring ownership of linear values.
    *   **Purpose**: Identity morphism; value transfer.

2.  **`apply r_fn r_arg r_out`**
    *   **Semantics**: Applies the function in `r_fn` to the argument in `r_arg`, placing the result in `r_out`. For linear functions, `r_arg` is consumed. Handles both primitive operations and user-defined functions. For user-defined functions, this pushes the current program counter onto the call stack and jumps to the function's label.
    *   **Purpose**: Function elimination; computation.

3.  **`alloc r_val r_out`**
    *   **Semantics**: Allocates the Layer 0 `Value` from `r_val` onto the heap. A unique `ResourceId` is generated, associated with the value, and this `ResourceId` is placed in `r_out`. The resource is initially marked as not consumed.
    *   **Purpose**: Resource introduction; heap allocation.

4.  **`consume r_resource r_out`**
    *   **Semantics**: Consumes the resource referenced by the handle in `r_resource`. Its value is extracted and placed in `r_out`. The resource is marked as consumed on the heap and cannot be accessed again.
    *   **Purpose**: Resource elimination; heap deallocation/value extraction.

5.  **`match r_sum r_l r_r l_label r_label`**
    *   **Semantics**: Pattern matches on a Layer 0 `Value::Sum` in `r_sum`. If `Inl(v)`, `v` is moved to `r_l` and PC jumps to `l_label`. If `Inr(v)`, `v` is moved to `r_r` and PC jumps to `r_label`. `r_sum` is consumed.
    *   **Purpose**: Sum elimination; conditional branching based on variant types.

6.  **`select r_cond r_true r_false r_out`**
    *   **Semantics**: Conditionally selects a value. If the boolean in `r_cond` is true, the value from `r_true` is moved to `r_out`; otherwise, the value from `r_false` is moved. The chosen source register (`r_true` or `r_false`) is consumed. Both `r_true` and `r_false` must hold values of the same type.
    *   **Purpose**: Conditional value selection; multiplexing.

7.  **`witness r_out`**
    *   **Semantics**: Reads a value from an untrusted external witness (e.g., external data provider, oracle) and places it into `r_out`.
    *   **Purpose**: Untrusted data input; interaction with external world.

8.  **`check constraint`**
    *   **Semantics**: Verifies that a given `constraint` (e.g., a boolean expression over register values or system state) holds. If violated, execution halts with an error.
    *   **Purpose**: Assertion; validation of witnessed data or invariants.

9.  **`perform effect r_out`**
    *   **Semantics**: Executes an effect. The effect (a structured data value) is processed by handlers (potentially at Layer 2) and then by the stateful interpreter. Results are placed in `r_out`.
    *   **Purpose**: Effect execution; interaction with stateful systems or external APIs.

10. **`labelmarker label`**
    *   **Semantics**: Marks a location in the program with the given `label`. This instruction does not modify machine state but serves as a target for function calls and control flow. During execution, this instruction is effectively a no-op that advances the program counter.
    *   **Purpose**: Control flow labeling; enables user-defined function calls in a ZK-compatible manner by providing addressable code locations.
    *   **ZK Rationale**: Added to support user-defined functions that reference code locations via labels rather than embedding instruction sequences directly, which is essential for efficient ZK circuit generation.

11. **`return result_reg`**
    *   **Semantics**: Returns from a function call by popping the return address from the call stack and setting the program counter to that address. If `result_reg` is specified, the value in that register becomes the function's return value and is placed appropriately for the caller.
    *   **Purpose**: Function return; completes user-defined function call sequences.
    *   **ZK Rationale**: Partners with `labelmarker` and the enhanced `apply` instruction to provide a complete function call mechanism that maintains ZK circuit efficiency by using structured control flow rather than embedded instruction sequences.

## 3. Role in Verification and ZK Proofs

The minimalism and determinism of the Layer 0 instruction set are crucial for several reasons:

-   **Formal Verification**: The simple semantics make it easier to formally prove properties about programs compiled to this layer.
-   **ZK-Circuit Generation**: Each instruction can be translated relatively directly into constraints for a ZK circuit, allowing for proofs of valid state transitions without revealing underlying data. The addition of `labelmarker` and `return` instructions specifically supports ZK-compatible function calls by avoiding the need to embed instruction sequences within function values.
-   **Security**: A small, well-understood trusted computing base reduces the attack surface.
-   **Function Call Efficiency**: The structured approach to user-defined functions (via labels and call stack) enables more efficient ZK proof generation compared to embedding instruction sequences directly in function values.

By providing this robust and verifiable foundation, Layer 0 enables Causality to support complex, high-level operations while maintaining strong guarantees about their execution. The careful extension from 9 to 11 instructions preserves the minimalist philosophy while adding essential capabilities for practical ZK-compatible function calls.
