# Causality Lisp

Layer 1 implementation providing a linear functional language with content-addressed expressions that compiles to the register machine while integrating with Layer 2 effects.

## Purpose

Causality Lisp implements **Layer 1** of the three-layer architecture - a linear functional programming language that operates on content-addressed expressions and integrates seamlessly with the unified transform system.

## Core Features

### Content-Addressed Expressions
All expressions are content-addressed via `ExprId(EntityId)`:
- Automatic structural sharing of identical subexpressions
- Deterministic compilation with global optimization
- ZK-circuit friendly fixed representations

### Linear Resource Integration
Direct support for linear resource semantics:
```lisp
;; Create linear resource
(define token (resource-create "Token" (record (balance 1000))))

;; Linear consumption (resource becomes invalid after use)
(define balance (resource-consume token "balance"))
```

### Unified Type System
Integration with Layer 1's unified type system:
- Base types: Unit, Bool, Int, Symbol
- Linear functions: `A ⊸ B`
- Products/sums: `A ⊗ B`, `A ⊕ B`
- Row types with location awareness
- Session types for communication protocols

### Effect Integration
Seamless composition with Layer 2 effects:
```lisp
;; Effect composition through functional programming
(define transfer-intent
  (intent
    (inputs (resource-ref token-id))
    (outputs (resource-create "Token" new-state))
    (constraints (conservation-law))))
```

## Core Components

- **Parser** (`parser.rs`): S-expression parsing with error recovery
- **AST** (`ast.rs`): Content-addressed abstract syntax tree
- **Type Checker** (`type_checker.rs`): Linear type inference and checking
- **Interpreter** (`interpreter.rs`): Direct evaluation for development
- **Compiler** (`compiler.rs`): Compilation to Layer 0 instructions
- **Value System** (`value.rs`): Linear value representation

## Compilation Pipeline

```
Lisp Source → AST → Type Check → Optimization → Layer 0 Instructions
```

All compilation stages preserve:
- Linear resource constraints
- Content addressing properties
- ZK circuit compatibility
- Mathematical foundations

## Integration Points

- **Layer 0**: Compiles to 5 fundamental register machine instructions
- **Layer 2**: Provides functional interface for effect composition
- **ZK System**: Expressions compile to arithmetic circuits
- **Content Store**: All expressions stored by content hash 