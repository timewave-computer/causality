# 018: Session Types Specification

Session types provide type-safe communication protocols with automatic duality checking in Causality's Layer 2. This document provides the complete formal specification for session types, their integration with the effect system, and compilation semantics.

## 1. Overview

Session types are a formal system for describing communication protocols between multiple parties. In Causality, session types form the third pillar of Layer 2 alongside Effects and Intents, enabling type-safe distributed communication while maintaining linearity guarantees and verifiability.

### Key Properties

1. **Type Safety**: Communication protocols are statically verified
2. **Duality**: Complementary protocols are automatically generated and verified
3. **Deadlock Freedom**: Well-typed session protocols cannot deadlock
4. **Linearity Preservation**: Session channels are linear resources
5. **Effect Integration**: Sessions compose seamlessly with effects and intents

## 2. Mathematical Foundation

### 2.1 Session Type Grammar

Session types are defined by the following grammar:

```
SessionType S ::= 
    | !T.S          -- Send value of type T, continue with S
    | ?T.S          -- Receive value of type T, continue with S
    | S₁ ⊕ S₂       -- Internal choice (offer one of S₁ or S₂)
    | S₁ & S₂       -- External choice (accept either S₁ or S₂)
    | End           -- Protocol termination
    | rec X.S       -- Recursive session type
    | X             -- Session type variable

Type T ::=
    | Unit | Bool | Int | String    -- Base types
    | T₁ × T₂                       -- Product types
    | T₁ + T₂                       -- Sum types
    | SessionChannel S              -- Session channel type
```

### 2.2 Duality Computation

The duality function `dual(S)` computes the complementary session type:

```
dual(!T.S) = ?T.dual(S)
dual(?T.S) = !T.dual(S)
dual(S₁ ⊕ S₂) = dual(S₁) & dual(S₂)
dual(S₁ & S₂) = dual(S₁) ⊕ dual(S₂)
dual(End) = End
dual(rec X.S) = rec X.dual(S[X ↦ dual(X)])
dual(X) = X
```

### 2.3 Duality Properties

The duality function satisfies the following properties:

1. **Involution**: `dual(dual(S)) = S` for all session types S
2. **Preservation**: If S is well-formed, then dual(S) is well-formed
3. **Compatibility**: Session types S₁ and S₂ are compatible iff S₁ = dual(S₂)

## 3. Session Type System

### 3.1 Session Channel Types

Session channels are linear resources that carry session protocols:

```rust
SessionChannel<S> : LinearResource
```

Where S is the session type describing the remaining protocol.

### 3.2 Typing Rules

#### Send Operation
```
Γ ⊢ e : T    Γ ⊢ ch : SessionChannel<!T.S>
─────────────────────────────────────────────
Γ ⊢ session_send(ch, e) : SessionChannel<S>
```

#### Receive Operation
```
Γ ⊢ ch : SessionChannel<?T.S>
──────────────────────────────────────────────
Γ ⊢ session_recv(ch) : T × SessionChannel<S>
```

#### Internal Choice (Select)
```
Γ ⊢ ch : SessionChannel<S₁ ⊕ S₂>    i ∈ {1,2}
─────────────────────────────────────────────
Γ ⊢ session_select(ch, i) : SessionChannel<Sᵢ>
```

#### External Choice (Case)
```
Γ ⊢ ch : SessionChannel<S₁ & S₂>
Γ ⊢ f₁ : SessionChannel<S₁> → T
Γ ⊢ f₂ : SessionChannel<S₂> → T
─────────────────────────────────────
Γ ⊢ session_case(ch, f₁, f₂) : T
```

### 3.3 Linearity Constraints

Session channels are linear resources that must be used exactly once:

1. **Single Use**: Each session channel operation consumes the channel
2. **Protocol Progression**: Operations must follow the prescribed protocol
3. **Resource Safety**: Channels cannot be duplicated or dropped improperly

## 4. Session Declarations

### 4.1 Session Declaration Syntax

Session types are declared with explicit roles:

```lisp
(def-session SessionName
  (role₁ S₁)
  (role₂ S₂)
  ...
  (roleₙ Sₙ))
```

### 4.2 Duality Verification

When multiple roles are specified, the system verifies duality relationships:

```lisp
;; Automatic duality verification
(def-session PaymentProtocol
  (client !Amount ?Receipt End)
  (server ?Amount !Receipt End))  ;; Verified as dual(client)
```

### 4.3 Multi-Party Sessions

Session types support multi-party protocols through role-based specifications:

```lisp
(def-session ThreePartyEscrow
  (buyer !Item ?Quote !Payment ?Confirmation End)
  (seller ?Item !Quote ?Payment !Delivery End)  
  (escrow ?Payment !Payment ?Confirmation !Delivery End))
```

## 5. Choreographies

### 5.1 Choreography Syntax

Choreographies describe global multi-party protocols:

```lisp
(choreography ChoreographyName
  (roles role₁ role₂ ... roleₙ)
  (protocol 
    (communications...)))
```

### 5.2 Communication Patterns

Choreographies support various communication patterns:

```lisp
;; Point-to-point communication
(buyer → seller: !ItemRequest)

;; Choice communication
(arbiter → buyer: (!Release ⊕ !Refund))

;; Parallel communication  
(parallel
  (buyer → escrow: !Payment)
  (seller → escrow: !ItemProof))

;; Sequential communication
(sequence
  (buyer → seller: !Order)
  (seller → buyer: !Confirmation)
  (seller → buyer: !Delivery))
```

### 5.3 Choreography Compilation

Choreographies compile to individual session types for each role through endpoint projection:

```
project(buyer → seller: !T, buyer) = !T.End
project(buyer → seller: !T, seller) = ?T.End
project(buyer → seller: !T, other) = End
```

## 6. Integration with Effect System

### 6.1 Session Effects

Session operations are integrated as first-class effects:

```rust
pub enum EffectExprKind {
    // Session communication operations
    SessionSend {
        channel: Box<EffectExpr>,
        value: Term,
        continuation: Box<EffectExpr>,
    },
    SessionReceive {
        channel: Box<EffectExpr>,
        continuation: Box<EffectExpr>,
    },
    SessionSelect {
        channel: Box<EffectExpr>,
        choice: String,
        continuation: Box<EffectExpr>,
    },
    SessionCase {
        channel: Box<EffectExpr>,
        branches: Vec<SessionBranch>,
    },
    WithSession {
        session_decl: String,
        role: String,
        body: Box<EffectExpr>,
    },
}
```

### 6.2 Session Effect Handlers

Session effects are handled through specialized handlers that maintain protocol state:

```rust
pub struct SessionHandler {
    pub session_type: SessionType,
    pub current_state: SessionState,
    pub role: String,
}

impl EffectHandler for SessionHandler {
    fn handle_session_send(&mut self, value: Value) -> Result<(), SessionError>;
    fn handle_session_recv(&mut self) -> Result<Value, SessionError>;
    fn handle_session_select(&mut self, choice: String) -> Result<(), SessionError>;
    fn handle_session_case(&mut self, branches: Vec<SessionBranch>) -> Result<Value, SessionError>;
}
```

### 6.3 Session-Intent Integration

Session types integrate with the Intent system through session requirements:

```rust
pub struct Intent {
    // ... existing fields ...
    
    /// Required session protocols
    pub session_requirements: Vec<SessionRequirement>,
    
    /// Session endpoints this intent provides
    pub session_endpoints: Vec<SessionEndpoint>,
}

pub struct SessionRequirement {
    pub session_name: String,
    pub role: String,
    pub required_protocol: SessionType,
}

pub struct SessionEndpoint {
    pub session_name: String,
    pub role: String,
    pub provided_protocol: SessionType,
}
```

## 7. Compilation Semantics

### 7.1 Layer 1 Compilation

Session types compile to Layer 1 linear lambda calculus through the following transformations:

#### Session Channel Allocation
```lisp
;; Layer 2
(with-session PaymentProtocol.client as ch ...)

;; Layer 1
(let ((ch (alloc (session-channel PaymentProtocol.client))))
  ...)
```

#### Session Send Operation
```lisp
;; Layer 2
(session-send ch value)

;; Layer 1
(let ((old-ch (consume ch)))
  (let ((result (tensor value (session-state-transition old-ch "send"))))
    (alloc result)))
```

#### Session Receive Operation
```lisp
;; Layer 2
(session-recv ch)

;; Layer 1
(let ((old-ch (consume ch)))
  (lettensor (value new-state) = (session-state-transition old-ch "recv")
    (tensor value (alloc new-state))))
```

### 7.2 Layer 0 Compilation

Layer 1 session operations compile to Layer 0 machine instructions:

```rust
// Session channel allocation
SessionAlloc {
    protocol_hash: [u8; 32],
    role: String,
    result_reg: RegisterId,
}

// Session send operation  
SessionSend {
    channel_reg: RegisterId,
    value_reg: RegisterId,
    result_reg: RegisterId,
}

// Session receive operation
SessionReceive {
    channel_reg: RegisterId,
    result_reg: RegisterId,
}
```

### 7.3 Protocol State Tracking

Session state is tracked through a combination of static type information and runtime state:

```rust
pub struct SessionState {
    pub protocol: SessionType,
    pub current_position: ProtocolPosition,
    pub role: String,
    pub message_history: Vec<Message>,
}

pub enum ProtocolPosition {
    Ready,
    WaitingSend(Type),
    WaitingReceive(Type),
    WaitingChoice(Vec<String>),
    Terminated,
}
```

## 8. Safety Properties

### 8.1 Type Safety

**Theorem (Session Type Safety)**: If a session program is well-typed, then:
1. Session operations respect the declared protocol
2. Communication partners have dual session types
3. No session operation will fail due to protocol mismatch

### 8.2 Deadlock Freedom

**Theorem (Deadlock Freedom)**: Well-typed session programs that follow the prescribed protocols cannot deadlock.

**Proof Sketch**: The duality relationship ensures that sends are matched with receives, and choice operations are matched with case operations. The linear type system prevents session channels from being used out of order.

### 8.3 Linearity Preservation

**Theorem (Linearity Preservation)**: Session operations preserve the linearity properties of the underlying resource system.

**Proof Sketch**: Session channels are implemented as linear resources. Each session operation consumes the input channel and produces a new channel (or final result), maintaining linear usage.

## 9. Error Handling

### 9.1 Session Errors

```rust
pub enum SessionError {
    ProtocolViolation {
        expected: SessionType,
        actual: SessionOperation,
    },
    DualityMismatch {
        session_name: String,
        role1: String,
        role2: String,
    },
    ChannelClosed {
        session_id: String,
    },
    InvalidChoice {
        available_choices: Vec<String>,
        selected_choice: String,
    },
    RecursionDepthExceeded {
        max_depth: usize,
    },
}
```

### 9.2 Error Recovery

Session type errors are detected at compile time where possible:

1. **Static Checking**: Protocol violations detected during type checking
2. **Duality Verification**: Mismatched roles detected during session declaration
3. **Runtime Validation**: Dynamic protocol state validation during execution

## 10. Advanced Features

### 10.1 Parametric Session Types

Session types can be parameterized by types and values:

```lisp
(def-session-template TransferProtocol<T>
  (sender !T ?Acknowledgment End)
  (receiver ?T !Acknowledgment End))

;; Instantiation
(def-session TokenTransfer (TransferProtocol<TokenAmount>))
```

### 10.2 Session Delegation

Sessions support delegation through session channel passing:

```lisp
;; Delegate session to another party
(session-send mediator-channel client-session-channel)
```

### 10.3 Session Interruption and Recovery

Sessions can be interrupted and recovered through checkpoint mechanisms:

```rust
pub struct SessionCheckpoint {
    pub session_id: String,
    pub protocol_state: SessionType,
    pub message_log: Vec<Message>,
    pub timestamp: u64,
}
```

## 11. Implementation Guidelines

### 11.1 Session Registry

Sessions are managed through a global registry:

```rust
pub struct SessionRegistry {
    sessions: HashMap<String, SessionDeclaration>,
    active_channels: HashMap<EntityId, SessionChannel>,
    choreographies: HashMap<String, Choreography>,
}
```

### 11.2 Protocol Verification

Session protocols are verified at multiple levels:

1. **Declaration Time**: Syntax and well-formedness checking
2. **Compilation Time**: Duality verification and type checking  
3. **Runtime**: Dynamic protocol state validation

### 11.3 Performance Considerations

Session type checking and protocol verification are designed for efficiency:

1. **Static Analysis**: Maximum checking performed at compile time
2. **Optimized State Tracking**: Minimal runtime overhead for protocol state
3. **Protocol Caching**: Compiled session protocols cached for reuse

This specification provides the complete foundation for implementing session types in Causality, enabling type-safe distributed communication while maintaining the mathematical rigor and verifiability that defines the platform. 