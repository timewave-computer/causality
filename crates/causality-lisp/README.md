# Causality Lisp

Expression language for the Causality framework, serving as Layer 1 in the three-layer architecture. Provides a functional programming interface for expressing computations that operate on linear resources while maintaining full compatibility with zero-knowledge proof generation.

## Purpose

The `causality-lisp` crate implements **Layer 1** of the Causality architecture - a structured functional programming language that operates on linear resources. It provides a high-level, expressive interface for writing verifiable programs while maintaining direct compilation paths to both the register machine (Layer 0) and seamless integration with the effect system (Layer 2).

### Key Responsibilities

- **Expression Evaluation**: Lisp-based functional programming with linear resource awareness
- **Resource Integration**: Direct integration with Causality's linear resource model
- **Compilation Bridge**: Compilation targets for register machine and zero-knowledge circuits
- **Effect Composition**: Functional composition of effects and intents

## Architecture Overview

Causality Lisp is designed around several core principles that distinguish it from traditional Lisp implementations:

### Linear Resource Awareness
Unlike traditional Lisps, Causality Lisp is built from the ground up to understand and enforce linear resource constraints:
- **Resource Types**: Direct support for Causality resource types in the language
- **Linearity Enforcement**: Compile-time and runtime checks for linear resource usage
- **Zero-Copy Integration**: Efficient interoperation with the core resource system

### Three-Layer Integration
The language serves as the crucial middle layer in the Causality architecture:
- **Layer 0 Compilation**: Direct compilation to register machine instructions
- **Layer 2 Integration**: Seamless effect system interoperability
- **Cross-Layer Optimization**: Optimization opportunities spanning multiple layers

### Verifiable Computation
All language constructs are designed for verifiable computation:
- **Deterministic Semantics**: All operations produce deterministic, reproducible results
- **ZK-Compatible**: Language constructs compile efficiently to zero-knowledge circuits
- **Content Addressable**: Expressions are content-addressable for integrity verification

## Core Language Features

### Functional Programming Foundation

```lisp
;; Lambda expressions with lexical scoping
(define square (lambda (x) (* x x)))
(define add (lambda (x y) (+ x y)))

;; Higher-order functions
(define compose 
  (lambda (f g) 
    (lambda (x) (f (g x)))))

;; Function composition
(define add-one-then-square (compose square (lambda (x) (+ x 1))))
(add-one-then-square 4)  ; => 25
```

### Resource-Aware Operations
Direct integration with Causality's linear resource model:

```lisp
;; Create a linear resource
(define my-token 
  (resource-create "TokenResource" 
    (record (balance 1000) (owner "alice"))))

;; Linear consumption (resource becomes invalid after use)
(define balance (resource-consume my-token "balance"))
;; my-token is now consumed and cannot be used again
```

### Effect Composition
Functional composition of effects and intents:

```lisp
;; Define an intent through functional composition
(define transfer-intent
  (intent
    (inputs (resource-ref token-id))
    (outputs (resource-create "TokenResource" 
               (record (balance new-balance) (owner new-owner))))
    (constraints (= (+ transferred-amount new-balance) original-balance))))
```

### Pattern Matching
Structural pattern matching for complex data:

```lisp
(define analyze-resource
  (lambda (resource)
    (match resource
      ((record (type "Token") (balance b) (owner o))
       (format "Token: {} owned by {}" b o))
      ((record (type "NFT") (id i))
       (format "NFT with ID: {}" i))
      (unknown
       "Unknown resource type"))))
```

## Integration Points

### Register Machine Integration
The Lisp interpreter can compile expressions down to register machine instructions:

```rust
use causality_lisp::compiler::compile_to_machine;

let lisp_expr = "(+ (* x 2) y)";
let instructions = compile_to_machine(lisp_expr)?;
// Results in register machine instruction sequence
```

### Effect System Integration
Seamless integration with Layer 2 effect orchestration:

```rust
use causality_lisp::effects::compile_intent;

let intent_expr = r#"
(intent "transfer"
  (inputs token-resource)
  (outputs new-token-resource)
  (logic transfer-function))
"#;

let intent = compile_intent(intent_expr)?;
```

### Zero-Knowledge Integration
Expressions can be compiled into zero-knowledge circuits:

```rust
use causality_lisp::zk::compile_to_circuit;

let verifiable_expr = "(and (> balance 100) (= owner sender))";
let circuit = compile_to_circuit(verifiable_expr)?;
```

## Type System Integration

Causality Lisp includes built-in understanding of the Causality type system:

### Linear Types
```lisp
;; Linear resources must be consumed exactly once
(define transfer-token
  (lambda (token recipient)
    (let ((balance (resource-consume token "balance")))
      (resource-create "TokenResource"
        (record (balance balance) (owner recipient))))))
```

### Affine Types
```lisp
;; Affine resources can be consumed at most once
(define maybe-use-resource
  (lambda (resource condition)
    (if condition
        (resource-consume resource)
        resource)))  ;; Resource may remain unconsumed
```

### Content Addressing
```lisp
;; Expressions are content-addressable
(define my-function (lambda (x) (* x x)))
;; Function has deterministic hash based on its structure
(expression-hash my-function)  ;; => "0x1234..."
```

## Performance Characteristics

### Evaluation Model
- **Eager Evaluation**: Arguments evaluated before function application
- **Tail-Call Optimization**: Recursive functions optimized for constant stack usage
- **Lexical Scoping**: Efficient environment chain lookup
- **First-Class Functions**: Minimal overhead for function values

### Memory Management
- **Reference Counting**: Automatic memory management for complex values
- **Copy-on-Write**: Efficient handling of immutable data structures
- **Resource Integration**: Zero-copy integration with Causality resources

### Compilation Targets
- **Interpreter Mode**: Direct evaluation for development and testing
- **Register Machine**: Compilation to Layer 0 instructions
- **ZK Circuits**: Compilation to zero-knowledge proof circuits

## Testing Framework

The crate includes comprehensive testing infrastructure:

```rust
// Property-based testing for evaluator correctness
#[test]
fn test_arithmetic_properties() {
    proptest!(|(a in any::<i64>(), b in any::<i64>())| {
        let expr = format!("(+ {} {})", a, b);
        let result = eval(&expr)?;
        assert_eq!(result, Value::Integer(a + b));
    });
}

// Integration tests with Causality components
#[test]
fn test_resource_integration() {
    let lisp_code = r#"
        (define token (resource-create "Token" (record (balance 100))))
        (resource-access token "balance")
    "#;
    let result = eval_with_resources(lisp_code)?;
    assert_eq!(result, Value::Integer(100));
}
```

## Design Philosophy

### Mathematical Foundation
The interpreter is built on solid mathematical principles:
- **Lambda Calculus**: Proper implementation of function abstraction and application
- **Type Theory**: Integration with linear and dependent type systems
- **Category Theory**: Compositional semantics for effect operations

### Deterministic Execution
All operations are designed to be deterministic:
- **Pure Functions**: No hidden side effects
- **Deterministic Resource IDs**: Content-addressable resource identification
- **Reproducible Evaluation**: Same expressions always produce same results

### Compositional Design
The language is designed for composition:
- **Function Composition**: Natural composition operators
- **Effect Composition**: Functional composition of side effects
- **Module Composition**: Clean module and namespace system

This design makes Causality Lisp suitable for expressing complex distributed computations while maintaining the mathematical properties necessary for zero-knowledge proofs and decentralized consensus. 