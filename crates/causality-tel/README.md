# Causality TEL (Temporal Effect Language)

A purely functional, statically typed language with effect tracking for the causality system.

## Overview

The Temporal Effect Language (TEL) is a combinator-based language that serves as the runtime for causality/effect chain tracking. It is designed to be:

- **Purely Functional**: All operations are free of side effects
- **Statically Typed**: Strong typing with row polymorphism
- **Effect-Aware**: Effects are tracked through the computation
- **Content-Addressable**: Expressions can be uniquely identified and verified

TEL is built on a foundation of combinatory logic (S, K, I, B, C combinators) with additional primitives for effects, state transitions, and content addressing.

## Migration Notice: Transition to TEG

> **Important:** TEL execution has been migrated to use the Temporal Effect Graph (TEG) intermediate representation. 

Deprecated APIs:
- Direct execution of `TelEffect` via `execute_tel_effect` has been removed
- `TelHandlerAdapter` implementation in the TEL crate has been deprecated
- Direct effect handling in combinators has been removed

New Approach:
- Use `causality-ir::TemporalEffectGraph` for representing TEL programs
- Use `Program.to_teg()` to convert TEL programs to TEG
- Execute TEGs using `causality-engine::effect::tel::TelEffectExecutor`

Example:
```rust
// Convert a TEL program to TEG
let program = parse_program(source)?;
let teg = program.to_teg()?;

// Execute using the engine
let executor = TelEffectExecutor::new(engine);
let result = executor.execute_teg(&teg).await?;
```

## TEG: Category Theory Foundation

The Temporal Effect Graph (TEG) serves as an intermediate representation for TEL programs based on category theory. The TEG implementation follows these principles:

- **Categorical Adjunction**: TEL and TEG form an adjunction `F ⊣ G` where:
  - `F: TEL → TEG` translates TEL programs to TEG
  - `G: TEG → TEL` translates TEG back to TEL programs
  - Natural isomorphism: `Hom_TEL(A, G(B)) ≅ Hom_TEG(F(A), B)`

- **Monoidal Structure**: Resources in TEG form a symmetric monoidal category with:
  - Tensor product for resource composition
  - Identity element for empty resources
  - Associativity and symmetry properties

- **Content Addressing**: Graph nodes are content-addressed to ensure:
  - Semantic equivalence of programs
  - Deterministic execution
  - Content-based verification

For in-depth details, see the [IR Theory documentation](../docs/architecture/ir-theory.md).

## Core Components

### Type System

TEL features a sophisticated type system with:

- **Base Types**: Int, String, Bool, Float, Unit, etc.
- **Row Types**: Extensible records with structural typing
- **Effect Types**: Row-polymorphic effect typing with handlers

```rust
// Example of defining a record type
let person_type = RecordType {
    fields: {
        "name": TelType::Base(BaseType::String),
        "age": TelType::Base(BaseType::Int)
    },
    extension: None
};

// Example of defining an effect row
let io_effect = EffectRow {
    effects: {
        "read": TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String))
        ),
        "write": TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::Unit))
        )
    },
    extension: None
};
```

### Combinators

The core of TEL is a combinator language with standard combinators:

```rust
// Example of a TEL expression: S K K x
let expr = parse_combinator("S K K x").unwrap();

// This should reduce to x (equivalent to the identity combinator I)
let mut reducer = BetaReducer::with_settings(ReducerSettings::default());
let result = reducer.eval(&expr).unwrap();
assert_eq!(result.expr, Combinator::Ref("x".to_string()));
```

### Effect System

TEL has a powerful effect system that allows tracking and handling effects:

```rust
// Define an effect
let effect = Combinator::effect("log", vec![
    Combinator::Literal(Literal::String("Hello, world!".to_string()))
]);

// Create a handler for the effect
let handler = TelType::Function(
    Box::new(TelType::Function(
        Box::new(TelType::Base(BaseType::String)),
        Box::new(TelType::Base(BaseType::Unit))
    )),
    Box::new(TelType::Base(BaseType::Unit))
);
```

### Content Addressing

TEL expressions can be content-addressed for verification:

```rust
// Create a Merkle node from an expression
let expr = parse_combinator("S K K x").unwrap();
let node = MerkleNode::from_combinator(&expr).unwrap();
let content_id = node.content_id.clone();

// Verify the expression by its content ID
let found = node.find_by_id(&content_id);
assert!(found.is_some());
```

## Usage Examples

### Defining a State Transition

```rust
// Parse a state transition expression
let transition = parse_combinator("transition Account {balance: 100}").unwrap();

// The transition will create a new state with the specified fields
if let Combinator::StateTransition { target_state, fields } = transition {
    assert_eq!(target_state, "Account");
    assert_eq!(fields["balance"], Combinator::Literal(Literal::Int(100)));
}
```

### Creating and Composing Effect Handlers

```rust
// Create handlers for different effects
let read_handler = /* handler for read effect */;
let write_handler = /* handler for write effect */;

// Compose the handlers to handle both effects
let combined_handler = /* composition of read and write handlers */;
```

## Getting Started

1. Add the crate to your dependencies:
```toml
[dependencies]
causality-tel = { git = "https://github.com/yourusername/causality.git" }
```

2. Import and use the TEL components:
```rust
use causality_tel::combinators::{Combinator, Literal};
use causality_tel::combinators::parser::parse_combinator;
use causality_tel::combinators::reducer::{BetaReducer, ReducerSettings};
use causality_tel::types::{TelType, BaseType, RecordType};
use causality_tel::types::effect::EffectRow;

// Your code here
```

## License

This project is licensed under the terms of the MIT license. 