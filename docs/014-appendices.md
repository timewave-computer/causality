# 014: Appendices

This section provides supplementary materials, quick references, and deeper dives into specific technical aspects of the Causality framework.

---

## Appendix A: Layer 0 Instruction Set Summary

The Layer 0 Typed Register Machine (TRM) forms the execution bedrock of Causality. It features a minimal set of 11 instructions designed for verifiable and efficient resource manipulation. Two additional instructions (`LabelMarker` and `Return`) were added beyond the original 9 to support user-defined function calls in a zero-knowledge proof compatible manner.

| Instruction        | Operands                                     | Description                                                                                                |
|--------------------|----------------------------------------------|------------------------------------------------------------------------------------------------------------|
| `Move`             | `src: RegId, dst: RegId`                     | Moves a value from `src` register to `dst` register. Linearity is maintained by Rust's move semantics if `RegId` itself is linear. |
| `Apply`            | `fn_reg: RegId, arg_reg: RegId, out_reg: RegId` | Applies a function (e.g., a Lisp lambda, identified by `ExprId` in `fn_reg`) to an argument from `arg_reg`, result in `out_reg`. For user-defined functions, pushes return address to call stack and jumps to function's label. |
| `Match`            | `sum_reg: RegId, left_reg: RegId, right_reg: RegId, left_label: Label, right_label: Label` | Pattern matches on sum type in `sum_reg`. If left variant, value goes to `left_reg` and execution jumps to `left_label`. If right variant, value goes to `right_reg` and execution jumps to `right_label`. |
| `Alloc`            | `val_reg: RegId, out_reg: RegId`             | Takes a `LispValue` from `val_reg`, allocates it as a new linear `Resource`, places its `ResourceId` in `out_reg`. |
| `Consume`          | `res_id_reg: RegId, val_out_reg: RegId`      | Takes a `ResourceId` from `res_id_reg`, consumes the resource, places its underlying `LispValue` in `val_out_reg`. |
| `Check`            | `constraint: ConstraintExpr`                 | Verifies that the given constraint expression evaluates to true. Halts execution with error if constraint is violated. |
| `Perform`          | `effect_reg: RegId, out_reg: RegId`          | Takes an `Effect` (or `EffectId`) from `effect_reg`, submits it to Layer 2, places result/status in `out_reg`. |
| `Select`           | `cond_reg: RegId, true_reg: RegId, false_reg: RegId, out_reg: RegId` | Conditionally selects a value. If boolean in `cond_reg` is true, moves value from `true_reg` to `out_reg`; otherwise moves value from `false_reg` to `out_reg`. |
| `Witness`          | `out_reg: RegId`                             | Reads a value from an untrusted external witness source and places it in `out_reg`. Used for external data input. |
| `LabelMarker`      | `label: String`                              | Marks a location in the program with the given label. No-op during execution but serves as target for function calls. Added for ZK-compatible user-defined functions. |
| `Return`           | `result_reg: Option<RegId>`                  | Returns from function call by popping return address from call stack and setting PC. If `result_reg` is specified, that register's value becomes the function's return value. Added for ZK-compatible user-defined functions. |

*Note: The exact operand types (e.g., `RegId`, `FieldKey`, `Label`) and the nature of values held in registers (`LispValue`, `ResourceId`, `ExprId`) are defined by the specific Rust implementation in the `causality-toolkit`.*

**ZK-Compatibility Additions:** The `LabelMarker` and `Return` instructions were added to support user-defined function calls that are compatible with zero-knowledge proof generation. This approach allows functions to reference code locations via labels rather than embedding instruction sequences directly, which is essential for efficient ZK circuit generation and maintains the minimalist philosophy of the instruction set.

---

## Appendix B: Causality Lisp Grammar (Conceptual EBNF)

This provides a simplified, conceptual EBNF-style grammar for Causality Lisp `Expr`essions. The actual parsing and AST structure are handled by the Rust toolkit.

```ebnf
expr ::= atom
       | list
       | QUOTE datum
       | ALLOC expr            (* Expression evaluating to value to allocate *)
       | CONSUME expr          (* Expression evaluating to a ResourceId *)
       | READ_FIELD expr field_name (* expr is ResourceId *)
       | UPDATE_FIELD expr field_name expr (* first expr is ResourceId *)
       | LAMBDA LPAREN params RPAREN expr (* Simplified: single body expr *)
       | APP expr expr*          (* First expr is function, rest are args *)
       | LET LPAREN bindings RPAREN expr (* Simplified: single body expr *)
       | IF expr expr expr

atom ::= SYMBOL | INTEGER | STRING | BOOLEAN | UNIT | RESOURCE_ID | EXPR_ID

datum ::= atom | LPAREN datum* RPAREN

list ::= LPAREN expr* RPAREN

params ::= SYMBOL*

bindings ::= LPAREN SYMBOL expr RPAREN*

field_name ::= SYMBOL | STRING (* Represents the key for a field in a resource *)

(* Keywords like QUOTE, ALLOC, CONSUME, LAMBDA, APP, LET, IF are illustrative of Lisp primitives. *)
(* SYMBOL, INTEGER, STRING, BOOLEAN, UNIT, RESOURCE_ID, EXPR_ID represent terminal LispValue types. *)
(* LPAREN, RPAREN are literal parentheses. *)
```

This grammar highlights the core forms. The 11 Lisp primitives are mapped to these syntactic structures or specific `PrimOp` variants within the `Expr` AST.

---

## Appendix C: SSZ Serialization and Content Addressing

A foundational principle in Causality is the use of **SSZ (SimpleSerialize)** for deterministic data serialization. This is crucial for several reasons:

1.  **Determinism**: SSZ guarantees that the same logical data structure will always serialize to the exact same byte string. This is essential for reliable hashing.
2.  **Content Addressing**: Key elements in Causality, such as:
    *   `ExprId` (for Lisp expressions)
    *   `EffectId` (for Effect definitions)
    *   `CircuitId` (for ZKP circuits)
    *   `ProofId` (for ZKP instances)
    *   `WitnessId` (for ZKP private inputs)
    are derived by taking the SSZ hash of their canonical representation. This means the identifier *is* a cryptographic commitment to the content itself.
3.  **Verifiability**: Anyone can re-serialize a piece of data (e.g., a Lisp `Expr`) using SSZ and hash it to verify its `ExprId`. This prevents tampering and ensures consistency across the system.
4.  **Network Efficiency**: While not its primary goal in Causality, SSZ is designed to be reasonably compact and efficient for network transmission.

The consistent use of SSZ underpins much of the verifiability and integrity of the Causality framework. It ensures that all participants have an unambiguous way to identify and reference core data structures.

---

## Appendix D: Inspirations and Further Reading (Conceptual)

The design of Causality draws inspiration from several areas of computer science and mathematics:

-   **Linear Logic & Type Theory**: The core concepts of linearity, affine types, and resource-consciousness are heavily influenced by linear logic, ensuring that resources are handled with formal precision.
-   **Capability Systems**: The idea of fine-grained, unforgeable permissions (capabilities) for accessing and manipulating resources informs the design of resource interactions and row types.
-   **Functional Programming**: Causality Lisp and the emphasis on pure `Handler` functions draw from the principles of functional programming, promoting composability, determinism, and ease of reasoning.
-   **Process Calculi & Actor Models**: While not a direct implementation, the way `Effect`s are handled and orchestrated in TEGs shares conceptual similarities with how concurrent processes or actors interact and manage state through message passing.
-   **Formal Verification**: The entire framework is designed with verifiability in mind, aiming to make it easier to formally prove properties about Causality applications, including resource safety and adherence to specified protocols.
-   **Zero-Knowledge Cryptography**: The integration of ZKPs is fundamental for enabling privacy-preserving verifiable computations and attestations within the system.

Exploring these fields can provide a deeper understanding of the theoretical underpinnings and motivations behind many of Causality's design choices.
