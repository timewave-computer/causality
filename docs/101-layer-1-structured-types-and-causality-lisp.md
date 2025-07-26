# 101: Layer 1 - Unified Type System & Causality Lisp

Layer 1 builds upon Layer 0's execution substrate to provide a unified type system that seamlessly integrates structured types, session types, and location awareness. This layer implements Causality Lisp, a linear functional language that compiles to Layer 0's minimal instruction set while maintaining mathematical elegance through category theory.

## Mathematical Foundation: Unified Type Structure

Layer 1 extends Layer 0's category theory foundation with a unified type system:

### Core Type Structure
```rust
pub enum TypeInner {
    /// Base primitive types (Unit, Bool, Int, Symbol)
    Base(BaseType),
    
    /// Linear product type (τ₁ ⊗ τ₂)
    Product(Box<TypeInner>, Box<TypeInner>),
    
    /// Sum type (τ₁ ⊕ τ₂)
    Sum(Box<TypeInner>, Box<TypeInner>),
    
    /// Linear function type (τ₁ ⊸ τ₂)
    LinearFunction(Box<TypeInner>, Box<TypeInner>),
    
    /// Record type with location-aware row polymorphism
    Record(RecordType),
    
    /// Session type - communication protocols
    Session(Box<SessionType>),
    
    /// Transform type - unifies functions and protocols
    Transform {
        input: Box<TypeInner>,
        output: Box<TypeInner>,
        location: Location,
    },
}
```

### Location-Aware Types
All types can be annotated with location information:

```rust
pub enum Location {
    Local,                    // Local computation
    Remote(String),          // Specific remote location
    Domain(String),          // Logical domain
    Any,                     // Location-polymorphic
}
```

## Unified Row Types: Local and Distributed

Layer 1's key innovation is location-aware row types that unify local record operations with distributed communication:

### Row Type Structure
```rust
pub struct RowType {
    /// Named fields with location information
    pub fields: BTreeMap<String, FieldType>,
    
    /// Optional row variable for polymorphism
    pub extension: Option<RowVariable>,
}

pub struct FieldType {
    /// The type of the field
    pub ty: TypeInner,
    
    /// Location constraint for the field
    pub location: Option<Location>,
    
    /// Access permissions
    pub access: FieldAccess,
}
```

### Unified Operations
The same operations work for both local and distributed scenarios:

```rust
// Local field access
let local_balance = project_field(account, "balance", Local);

// Remote field access (automatically generates protocol)
let remote_balance = project_field(account, "balance", Remote("server"));
```

## Session-Linear Integration

Session types are integrated directly into the linear type system:

### Session Types
```rust
pub enum SessionType {
    /// Send a value, continue with protocol
    Send(Box<TypeInner>, Box<SessionType>),
    
    /// Receive a value, continue with protocol  
    Receive(Box<TypeInner>, Box<SessionType>),
    
    /// Internal choice (we choose)
    InternalChoice(Vec<(String, SessionType)>),
    
    /// External choice (other party chooses)
    ExternalChoice(Vec<(String, SessionType)>),
    
    /// End of communication
    End,
    
    /// Recursive protocols
    Recursive(String, Box<SessionType>),
    
    /// Session variable
    Variable(String),
}
```

### Automatic Protocol Derivation
Session types are automatically derived from row operations:

```rust
// This row operation...
let update_remote_field = update_field(
    record_ref,
    "balance", 
    new_value,
    Remote("database")
);

// ...automatically generates this protocol:
// Send(UpdateRequest) → Receive(UpdateResponse) → End
```

## Causality Lisp: Unified Syntax

Causality Lisp provides a unified syntax for all operations:

### Local Operations
```lisp
;; Local computation
(let ((balance (get-field account "balance")))
  (+ balance 100))
```

### Distributed Operations  
```lisp
;; Remote computation (same syntax!)
(let ((remote-balance (get-field remote-account "balance")))
  (+ remote-balance 100))
```

### Session Operations
```lisp
;; Session communication
(with-session PaymentProtocol client-role
  (session-send amount)
  (session-recv receipt))
```

## Compilation to Layer 0

All Layer 1 constructs compile to Layer 0's 5 fundamental instructions:

### Transform Compilation
- **Local operations** → `transform` with local morphisms
- **Remote operations** → `transform` with distributed morphisms  
- **Session operations** → `transform` with protocol morphisms

### Resource Management
- **Record allocation** → `alloc` with record type
- **Session creation** → `alloc` with session type
- **Resource cleanup** → `consume` with appropriate type

### Composition
- **Function composition** → `compose` instruction
- **Parallel operations** → `tensor` instruction

## Benefits of Unification

This unified approach provides:

- **Single Mental Model**: Same concepts for local and distributed programming
- **Automatic Protocols**: Communication protocols derived from type operations
- **Location Transparency**: Operations work the same locally and remotely
- **Mathematical Consistency**: All operations follow the same category theory principles
- **Zero Overhead**: Local operations compile to efficient Layer 0 instructions
