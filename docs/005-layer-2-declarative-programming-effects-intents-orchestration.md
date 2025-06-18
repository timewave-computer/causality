# 005: Layer 2 - Declarative Programming, Effects, Intents, Sessions & Orchestration

Layer 2 represents the highest abstraction level within the Causality architecture. This layer is where application developers primarily work, creating sophisticated domain-specific logic through a declarative programming model centered around **Effects**, **Intents**, **Session Types**, **Capability-based Access Control**, and cross-domain orchestration capabilities.

Building upon the type-safe resource management of Layer 1 and the verifiable execution substrate of Layer 0, Layer 2 provides the tools needed to express complex business logic, coordinate multi-party interactions, and manage sophisticated resource transformations while maintaining the mathematical guarantees of the underlying system.

**Key Architecture**: Layer 2 includes a complete **capability system**, **object model**, and **record operations**. These features compile down to pure Layer 1 primitives through effect resolution and capability analysis.

## 1. Mathematical Foundation

Layer 2 is built upon the **Kleisli category** for an Effect monad over Layer 1's linear lambda calculus. This categorical structure enables:

- **Composable Effects**: Effects form a monad with well-defined composition laws
- **Natural Transformations**: Handlers represent morphisms between effect algebras  
- **Functorial Semantics**: Resource transformations preserve structural properties
- **Categorical Stratification**: Clean separation between pure computation (Layer 1) and effectful operations (Layer 2)
- **Capability Algebra**: Fine-grained access control through algebraic capability composition

### 1.1 Effect Monad

```
Effect A := Computation that may perform effects and produce value of type A
pure : A → Effect A                    -- Monadic return
bind : Effect A × (A → Effect B) → Effect B  -- Monadic composition
```

### 1.2 Capability Algebra

Capabilities form a **partial order** representing access permissions with algebraic structure:

```
Capability := PowerSet(AccessRight) × ResourceSchema
⊑ := capability implication (subset relation + schema compatibility)
∩ := capability intersection (meet)  
∪ := capability union (join)
⊥ := no capabilities (bottom)
⊤ := all capabilities (admin access)
```

**Capability Levels**:
- `Read` ⊑ `Write` ⊑ `Admin`
- `Read` ⊑ `Execute` ⊑ `Admin`
- Record-specific capabilities with field-level granularity

## 2. Core Layer 2 Components

Layer 2 is built around three fundamental abstractions that work together to provide comprehensive declarative programming capabilities:

1. **Effects** - Structured descriptions of operations to be performed
2. **Intents** - Declarative specifications of desired outcomes  
3. **Session Types** - Type-safe communication protocols with automatic duality

These three pillars complement each other: Effects handle individual operations, Intents coordinate complex workflows, and Session Types ensure safe communication between parties.

### 2.1 Effects - Structured Side Effects

Effects are **pure data structures** that describe operations to be performed. They separate the specification of what should happen from how it's implemented, enabling powerful composition and transformation patterns.

```rust
pub enum Effect {
    // Core effect operations
    Pure(Value),
    Bind(Box<Effect>, Box<dyn Fn(Value) -> Effect>),
    Perform(EffectData),
    
    // Capability-based operations (moved from Layer 1)
    AccessField(ResourceId, FieldName, CapabilityToken),
    UpdateField(ResourceId, FieldName, Value, CapabilityToken),
    CreateRecord(RecordSchema, CapabilityToken),
    DeleteRecord(ResourceId, CapabilityToken),
    ProjectRecord(ResourceId, Vec<FieldName>, CapabilityToken),
    ExtendRecord(ResourceId, RecordSchema, CapabilityToken),
    RestrictRecord(ResourceId, Vec<FieldName>, CapabilityToken),
    
    // Capability management  
    RequireCapability(CapabilityName),
    GrantCapability(ResourceId, CapabilityName),
    RevokeCapability(CapabilityToken),
    ValidateCapability(CapabilityToken, AccessRequirement),
    
    // Resource algebra
    Produce(CapabilityToken, Value),
    Transform(ResourceId, Function),
    Combine(ResourceId, ResourceId),
    Split(ResourceId),
    
    // Object linearity operations (moved from Layer 1)
    CreateLinearObject(Value, CapabilitySet),
    CreateAffineObject(Value, CapabilitySet),
    CreateRelevantObject(Value, CapabilitySet), 
    CreateUnrestrictedObject(Value, CapabilitySet),
    ConsumeObject(ObjectId),
    CopyObject(ObjectId), // Only for Relevant/Unrestricted
    DropObject(ObjectId), // Only for Affine/Unrestricted
    
    // Verification and causality
    Check(Constraint),
    Depend(ResourceId, ResourceId),
    Verify(Proof),
    
    // Transaction orchestration
    Transact(Vec<Effect>),
    Atomic(Box<Effect>),
    Parallel(Vec<Effect>),
    Race(Vec<Effect>),
}
```

### 2.2 Capability System - Comprehensive Access Control

The capability system provides fine-grained, unforgeable access control over resources and their fields. This system was moved from Layer 1 to Layer 2 to maintain Layer 1's mathematical purity while providing the rich access control needed for real-world applications.

#### 2.2.1 Capability Types

```rust
pub enum CapabilityLevel {
    Read,     // Read-only access
    Write,    // Write access (implies Read)
    Execute,  // Execute access (implies Read)
    Admin,    // Administrative access (implies all others)
}

pub enum RecordCapability {
    ReadField(FieldName),
    WriteField(FieldName),
    CreateRecord(RecordSchema),
    DeleteRecord,
    ProjectFields(Vec<FieldName>),
    ExtendRecord(RecordSchema),
    RestrictRecord(Vec<FieldName>),
    FullRecordAccess,
}

pub struct Capability {
    pub name: String,
    pub level: CapabilityLevel,
    pub record_capability: Option<RecordCapability>,
}
```

#### 2.2.2 Core Capability Effects

| Effect | Type Signature | Purpose |
|--------|----------------|---------|
| `AccessField` | `Resource A × FieldName × Capability ⊸ Effect (Option B)` | Read field with permission checking |
| `UpdateField` | `Resource A × FieldName × B × Capability ⊸ Effect (Resource A')` | Update field with capability verification |
| `RequireCapability` | `CapabilityName ⊸ Effect Capability` | Demand specific capability for operation |
| `GrantCapability` | `Resource A × CapabilityName ⊸ Effect Capability` | Extract capability from resource ownership |
| `CreateRecord` | `RecordSchema × Capability ⊸ Effect ResourceId` | Create new record with schema |
| `ProjectRecord` | `ResourceId × FieldList × Capability ⊸ Effect ResourceId` | Select subset of fields |

#### 2.2.3 Capability Resolution Process

During Intent compilation, the capability system performs **static analysis** to resolve dynamic operations:

```
1. Capability Analysis
   Intent → Required Capabilities → Static Schema Resolution

2. Schema Monomorphization  
   Polymorphic field access → Concrete field operations with fixed layouts

3. Effect Compilation
   Layer 2 capability effects → Layer 1 tensor/sum operations

4. Code Generation
   Layer 1 terms → Layer 0 instructions with static structure

5. ZK Circuit Compilation
   Layer 0 instructions → Fixed-size zero-knowledge circuits
```

**Example**:
```rust
// Layer 2: Capability-based access
Intent::new()
    .require_capability("account.balance.read")
    .access_field(account_id, "balance")

// After capability resolution: Fixed record structure
// Layer 1: lettensor (balance, other_fields) = consume(account_resource) in alloc(balance)
// Layer 0: consume + tensor projection + alloc instructions
```

### 2.3 Object Model - Linear Resource Management

The object model provides sophisticated resource management with linearity enforcement, moved from Layer 1 to Layer 2 to enable capability-based access control while maintaining type safety.

#### 2.3.1 Object Types with Linearity

```rust
pub struct Object<L: Linearity> {
    pub data: Value,
    pub capabilities: CapabilitySet,
    _linearity: PhantomData<L>,
}

// Linearity markers
pub struct Linear;     // Must use exactly once
pub struct Affine;     // May use at most once  
pub struct Relevant;   // Must use at least once
pub struct Unrestricted; // May use any number of times

pub type LinearObject<T> = Object<Linear, T>;
pub type AffineObject<T> = Object<Affine, T>;
pub type RelevantObject<T> = Object<Relevant, T>;
pub type UnrestrictedObject<T> = Object<Unrestricted, T>;
```

#### 2.3.2 Object Capability Operations

```rust
impl<T> Object<Linear, T> {
    pub fn with_capability(self, cap: Capability) -> Self;
    pub fn has_capability(&self, cap: &Capability) -> bool;
    pub fn consume(self) -> Result<T, LinearityError>;
    // Cannot copy or drop
}

impl<T> Object<Affine, T> {
    // Inherits Linear operations plus:
    pub fn drop_unused(self);  // Safe disposal without use
}

impl<T> Object<Relevant, T> {
    // Must be used but can copy:
    pub fn copy_data(&self) -> Result<T, LinearityError>;
    // Cannot drop without use
}

impl<T> Object<Unrestricted, T> {
    // Full flexibility:
    pub fn copy_data(&self) -> Result<T, LinearityError>;
    pub fn drop_unused(self);
}
```

### 2.4 Record Operations - Row Polymorphism

Record operations provide structured data manipulation with row polymorphism, moved from Layer 1 to enable capability-based field access while maintaining ZK compatibility through static resolution.

#### 2.4.1 Record Schema System

```rust
pub struct RecordSchema {
    pub fields: HashMap<FieldName, TypeName>,
    pub required_capabilities: HashSet<String>,
}

pub enum RowType {
    Empty,
    Cons(FieldName, Type, Box<RowType>),
    Variable(String), // For polymorphism at Layer 2 only
}

pub struct RecordType {
    pub row: RowType,
    pub capabilities: Vec<RecordCapability>,
}
```

#### 2.4.2 Record Effect Operations

| Operation | Type | Purpose |
|-----------|------|---------|
| `access_field` | `ResourceId × FieldName × Capability ⊸ Effect Value` | Safe field access |
| `update_field` | `ResourceId × FieldName × Value × Capability ⊸ Effect ResourceId` | Safe field update |
| `project_record` | `ResourceId × FieldList × Capability ⊸ Effect ResourceId` | Field projection |
| `extend_record` | `ResourceId × RecordSchema × Capability ⊸ Effect ResourceId` | Add fields |
| `restrict_record` | `ResourceId × FieldList × Capability ⊸ Effect ResourceId` | Remove fields |
| `create_record` | `RecordSchema × Capability ⊸ Effect ResourceId` | Create new record |
| `delete_record` | `ResourceId × Capability ⊸ Effect Unit` | Delete entire record |

### 2.5 Intents - Declarative Specifications

Intents represent **declarative specifications** of desired outcomes, enhanced with comprehensive capability requirements and object management.

```rust
pub struct Intent {
    pub name: String,
    pub domain: EntityId,
    
    // Resource bindings with capabilities
    pub input_bindings: Vec<ResourceBinding>,
    pub output_bindings: Vec<ResourceBinding>,
    
    // Capability requirements (comprehensive)
    pub required_capabilities: Vec<CapabilityRequirement>,
    pub granted_capabilities: Vec<CapabilityGrant>,
    
    // Constraint system  
    pub constraints: ConstraintTree,
    
    // Effect specification with capability effects
    pub effects: Vec<Effect>,
    
    // Object linearity requirements
    pub linearity_constraints: Vec<LinearityConstraint>,
    
    // Optimization hints
    pub hints: IntentHints,
}
```

#### 2.5.1 Enhanced Resource Bindings

```rust
pub struct ResourceBinding {
    pub name: String,
    pub resource_type: String,
    pub quantity: Option<u64>,
    pub capabilities: Vec<Capability>,          // Required capabilities
    pub object_linearity: Option<LinearityType>, // Linearity requirements
    pub schema: Option<RecordSchema>,           // Expected record structure
    pub constraints: Vec<Constraint>,
    pub metadata: Value,
}
```

### 2.6 Session Types - Type-Safe Communication Protocols

Session types provide **type-safe communication protocols** with automatic duality checking, enabling distributed systems to maintain correctness guarantees while composing complex multi-party workflows. Session types form the third pillar of Layer 2 alongside Effects and Intents.

#### 2.6.1 Session Type Syntax

Session types describe communication protocols using a precise mathematical syntax:

```rust
pub enum SessionType {
    Send(Type, Box<SessionType>),        // !T.S - Send value of type T, continue with S
    Receive(Type, Box<SessionType>),     // ?T.S - Receive value of type T, continue with S  
    InternalChoice(Vec<SessionType>),    // ⊕{...} - Choose one branch to offer
    ExternalChoice(Vec<SessionType>),    // &{...} - Accept one of several branches
    End,                                 // End - Protocol termination
    Recursive(String, Box<SessionType>), // rec X.S - Recursive protocol
    Variable(String),                    // X - Reference to recursive variable
}
```

#### 2.6.2 Duality Computation

The core innovation of session types is **automatic duality computation** - the ability to generate complementary protocols that are guaranteed to be compatible:

```
dual(!T.S) = ?T.dual(S)           // Send becomes receive
dual(?T.S) = !T.dual(S)           // Receive becomes send  
dual(P ⊕ Q) = dual(P) & dual(Q)   // Internal choice becomes external
dual(P & Q) = dual(P) ⊕ dual(Q)   // External choice becomes internal
dual(End) = End                   // Terminal stays terminal
dual(rec X.S) = rec X.dual(S[X ↦ dual(X)])  // Recursive types
```

This ensures that communication protocols are **deadlock-free** and **type-safe** by construction.

#### 2.6.3 Session Declaration and Usage

Session types are declared with explicit role specifications:

```lisp
;; Session type declaration with automatic duality
(def-session PaymentProtocol
  (client !Amount ?Receipt End)
  (server ?Amount !Receipt End))  ;; Automatically verified as dual

;; Session usage with type safety
(defn handle-payment (amount)
  (with-session PaymentProtocol.server as srv
    (let ((amt (session-recv srv)))
      (let ((receipt (process-payment amt)))
        (session-send srv receipt)))))
```

#### 2.6.4 Session Effects Integration

Session operations integrate seamlessly with the effect system:

```rust
pub enum EffectExprKind {
    // ... existing variants ...
    
    /// Session communication operations
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

#### 2.6.5 Choreographies and Multi-Party Coordination

Session types support **choreographies** for multi-party protocol specification:

```lisp
(choreography EscrowProtocol
  (roles buyer seller arbiter)
  (flow
    buyer → seller: !Item
    seller → arbiter: !Claim  
    arbiter → buyer: (!Release ⊕ !Refund)
    arbiter → seller: (!Payment ⊕ !Return)))
```

#### 2.6.6 Session-Intent Integration

Session types integrate naturally with the Intent system for declarative communication:

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
```

#### 2.6.7 Compilation to Linear Resources

Session types compile to Layer 1 linear resources, maintaining the mathematical guarantees:

```rust
// Session channels as linear resources
pub struct SessionChannel<T> {
    pub protocol: SessionType,
    pub role: String,
    pub state: SessionState,
    _phantom: PhantomData<T>,
}

impl<T> LinearResource for SessionChannel<T> {
    fn alloc(protocol: SessionType) -> Self;
    fn consume(self) -> (Value, SessionType);
}
```

### 2.7 Module Organization

Layer 2 is implemented in the `causality-core/src/effect` module with the following enhanced structure:

```
effect/
├── core.rs         # Core effect types and operations
├── operations.rs   # Effect algebra operations  
├── capability.rs   # Capability system (moved from lambda/)
├── object.rs       # Object model (moved from lambda/)  
├── row.rs          # Record/row operations (moved from lambda/)
├── record.rs       # Record capability effects
├── intent.rs       # Intent-based programming
├── session.rs      # Session types and protocols
├── session_registry.rs # Session registry and management
├── choreography.rs # Multi-party choreographies
├── synthesis.rs    # Effect synthesis and compilation
├── teg.rs          # Temporal Effect Graph
├── resource.rs     # Resource algebra
├── causality.rs    # Causality tracking
└── pattern.rs      # Pattern matching
```

## 3. Effect Algebra Operations

### 3.1 Core Monad Operations

| Operation | Type | Purpose |
|-----------|------|---------|
| `pure` | `A ⊸ Effect A` | Lift pure values into effect context |
| `bind` | `Effect A ⊗ (A ⊸ Effect B) ⊸ Effect B` | Sequence effectful computations |
| `perform` | `EffectData ⊸ Effect Result` | Execute primitive effects |
| `handle` | `Effect A ⊗ Handler ⊸ Effect B` | Apply effect transformations |

### 3.2 Capability Operations

| Operation | Type | Purpose |
|-----------|------|---------|
| `access_field` | `ResourceId × FieldName × Capability ⊸ Effect Value` | Safe field reading |
| `update_field` | `ResourceId × FieldName × Value × Capability ⊸ Effect ResourceId` | Safe field updating |
| `require_capability` | `CapabilityName ⊸ Effect Capability` | Capability acquisition |
| `grant_capability` | `ResourceId × CapabilityName ⊸ Effect Capability` | Capability derivation |
| `revoke_capability` | `Capability ⊸ Effect Unit` | Capability invalidation |

### 3.3 Resource Algebra

| Operation | Type | Purpose |
|-----------|------|---------|
| `produce` | `Capability ⊗ Data ⊸ Effect (Resource Data)` | Create resources with access control |
| `transform` | `Resource A ⊗ (A ⊸ B) ⊸ Effect (Resource B)` | Functorial resource transformation |
| `combine` | `Resource A ⊗ Resource B ⊸ Effect (Resource (A ⊗ B))` | Resource composition |
| `split` | `Resource (A ⊕ B) ⊸ Effect ((Resource A) ⊕ (Resource B))` | Resource decomposition |

### 3.4 Conservation and Causality

| Operation | Type | Purpose |
|-----------|------|---------|
| `check` | `List (Resource A) ⊗ List (Resource A) ⊸ Effect Unit` | Verify conservation laws |
| `depend` | `Resource A ⊗ Resource B ⊸ Effect (A, B, Proof)` | Establish causal relationships |
| `sequence` | `(A, B, Proof) ⊗ (B, C, Proof) ⊸ Effect (A, C, Proof)` | Chain causal dependencies |
| `verify` | `(A, B, Proof) ⊸ Effect Proof` | Extract verifiable proofs |

### 3.5 Session Operations

| Operation | Type | Purpose |
|-----------|------|---------|
| `session_send` | `SessionChannel A ⊗ A ⊸ Effect (SessionChannel B)` | Send value through session channel |
| `session_recv` | `SessionChannel A ⊸ Effect (A ⊗ SessionChannel B)` | Receive value from session channel |
| `session_select` | `SessionChannel A ⊗ String ⊸ Effect (SessionChannel B)` | Select branch in session protocol |
| `session_case` | `SessionChannel A ⊗ Handlers ⊸ Effect B` | Handle incoming session choices |
| `with_session` | `SessionType ⊗ String ⊗ (SessionChannel A ⊸ Effect B) ⊸ Effect B` | Create and use session channel |

### 3.6 Transaction Orchestration

| Operation | Type | Purpose |
|-----------|------|---------|
| `transact` | `List (Effect A) ⊸ Effect (List A)` | Atomic multi-effect execution |
| `atomic` | `Effect A ⊸ Effect (Effect A)` | Transaction context lifting |
| `commit` | `Effect (Effect A) ⊸ Effect A` | Transaction finalization |
| `rollback` | `Effect (Effect A) ⊸ Effect Error` | Transaction abortion |

To sum this all up:

- Resources are data.
- Effects are transformations of resources.
- Handlers are transformations of effects.
- Transactions are committed graphs of resource transformations.

Linear types tie everything together, ensuring every resource and effect is used exactly once, accounted for, and safely composed.

## 4. Programming Model Examples

### 4.1 Token Transfer with Capabilities

```rust
// Define Intent with capability requirements
let transfer_intent = Intent::new("TokenTransfer")
    .require_capability("token.balance.read")
    .require_capability("token.balance.write")
    .input_resource("source_token", token_id)
    .output_resource("target_token", token_spec)
    .constraint(conservation("amount"))
    .effect(
        bind(
            access_field(token_id, "balance", balance_read_cap),
            |current_balance| {
                bind(
                    check(constraint!("current_balance >= transfer_amount")),
                    |_| update_field(
                        token_id, 
                        "balance", 
                        current_balance - transfer_amount,
                        balance_write_cap
                    )
                )
            }
        )
    );
```

### 4.2 Multi-Party Atomic Swap

```rust
let atomic_swap = Intent::new("AtomicSwap")
    .require_capability("alice.token_a.transfer")
    .require_capability("bob.token_b.transfer")
    .effect(
        transact(vec![
            // Alice transfers Token A to Bob
            perform(Transfer {
                from: alice_account,
                to: bob_account,
                token: token_a,
                amount: amount_a,
            }),
            // Bob transfers Token B to Alice  
            perform(Transfer {
                from: bob_account,
                to: alice_account,
                token: token_b,
                amount: amount_b,
            }),
        ])
    );
```

### 4.3 Privacy-Preserving Auction

```rust
let sealed_auction = Intent::new("SealedBidAuction")
    .require_capability("auction.bid.submit")
    .require_capability("auction.result.verify")
    .effect(
        bind(
            parallel(
                bidders.map(|bidder| 
                    perform(SubmitSealedBid {
                        bidder_id: bidder.id,
                        encrypted_bid: bidder.sealed_bid,
                        proof: bidder.bid_proof,
                    })
                )
            ),
            |sealed_bids| {
                bind(
                    perform(RevealPhase { sealed_bids }),
                    |revealed_bids| {
                        let winner = determine_winner(revealed_bids);
                        perform(AwardItem {
                            winner: winner.bidder_id,
                            winning_bid: winner.bid_amount,
                        })
                    }
                )
            }
        )
    );
```

### 4.4 Session-Based Payment Protocol

```rust
// Define payment session protocol
let payment_session = SessionDeclaration::new("PaymentProtocol")
    .add_role("client", SessionType::Send(
        Type::Int, 
        Box::new(SessionType::Receive(Type::String, Box::new(SessionType::End)))
    ))
    .add_role("server", SessionType::Receive(
        Type::Int,
        Box::new(SessionType::Send(Type::String, Box::new(SessionType::End)))
    ));

// Client-side payment intent with session requirement
let payment_intent = Intent::new("PaymentRequest")
    .require_session("PaymentProtocol", "client")
    .input_resource("payment_amount", amount)
    .constraint(Constraint::GreaterThan(
        ValueExpr::ResourceRef("payment_amount".to_string()),
        ValueExpr::Literal(Value::Int(0))
    ))
    .effect(
        with_session("PaymentProtocol", "client", |channel| {
            bind(
                session_send(channel, ValueExpr::ResourceRef("payment_amount".to_string())),
                |updated_channel| {
                    session_recv(updated_channel)
                }
            )
        })
    );
```

### 4.5 Multi-Party Escrow with Session Choreography

```lisp
;; Define escrow choreography
(choreography EscrowChoreography
  (roles buyer seller arbiter)
  (protocol
    ;; Initial setup phase
    (buyer → seller: !ItemRequest)
    (seller → buyer: !ItemDetails)
    (buyer → arbiter: !EscrowRequest)
    
    ;; Payment and delivery phase  
    (buyer → arbiter: !Payment)
    (seller → arbiter: !ItemProof)
    
    ;; Resolution phase
    (arbiter → buyer: (!ItemReceived ⊕ !Dispute))
    (arbiter → seller: (!PaymentRelease ⊕ !PaymentWithhold))))

;; Implement buyer role
(defn buyer-escrow-role (item-request payment-amount)
  (with-session EscrowChoreography.buyer as buyer-chan
    (do
      ;; Send item request to seller
      (session-send buyer-chan item-request)
      
      ;; Receive item details
      (let ((item-details (session-recv buyer-chan)))
        
        ;; Send escrow request to arbiter
        (session-send buyer-chan (create-escrow-request item-details payment-amount))
        
        ;; Send payment to arbiter
        (session-send buyer-chan payment-amount)
        
        ;; Wait for resolution
        (session-case buyer-chan
          (ItemReceived receipt -> (complete-purchase receipt))
          (Dispute details -> (handle-dispute details)))))))
```

## 5. Compilation and Optimization

### 5.1 Intent Compilation Pipeline

```
Layer 2 Intent
    ↓ capability analysis
Required Capabilities + Static Schema  
    ↓ constraint solving
Validated Effect Graph
    ↓ monomorphization
Concrete Field Access Operations
    ↓ effect compilation
Layer 1 Terms (fixed record structure)
    ↓ optimization
Optimized Layer 1 Code
    ↓ Layer 0 compilation
Register Machine Instructions
    ↓ circuit generation
ZK Circuit (fixed topology)
```

### 5.2 Capability Resolution

The capability system performs static analysis to convert dynamic field operations into fixed record structures:

1. **Capability Inference**: Analyze Intent to determine required capabilities
2. **Schema Generation**: Create concrete record types from capability requirements  
3. **Access Pattern Analysis**: Determine optimal field access order
4. **Monomorphization**: Convert polymorphic operations to concrete implementations
5. **Code Generation**: Emit Layer 1 code with fixed data layouts

### 5.3 Optimization Strategies

- **Effect Fusion**: Combine compatible effects to reduce overhead
- **Capability Batching**: Group related capability operations
- **Resource Lifetime Analysis**: Optimize allocation and consumption
- **Parallel Execution**: Identify independent effect sequences
- **ZK Circuit Optimization**: Minimize circuit size and constraint count

## 6. Security and Verification

### 6.1 Capability Security Model

The capability system provides several security guarantees:

- **Unforgeable Capabilities**: Capabilities cannot be created or modified by unauthorized code
- **Least Privilege**: Operations can only access explicitly granted capabilities
- **Capability Confinement**: Capabilities cannot be leaked between security domains
- **Audit Trail**: All capability usage is recorded and verifiable

### 6.2 Static Verification

Layer 2 enables powerful static verification:

- **Effect Safety**: All effects have verified handlers
- **Resource Conservation**: Conservation laws checked at Intent level
- **Capability Completeness**: All required capabilities are available
- **Temporal Consistency**: Causal dependencies are acyclic

### 6.3 Runtime Guarantees

- **Atomic Execution**: Transactions either complete fully or roll back completely
- **Linear Resource Usage**: Resources consumed exactly once across all effects
- **Capability Enforcement**: Runtime capability checks complement static analysis
- **Proof Generation**: Verifiable proofs of correct execution

## 7. Integration with Lower Layers

### 7.1 Layer 1 Target

Layer 2 effects compile to Layer 1 expressions that use only the 11 core primitives. The capability system ensures this compilation maintains:

- **Mathematical Purity**: Layer 1 code uses only categorical primitives
- **Fixed Structure**: All record layouts determined statically
- **ZK Compatibility**: Resulting code compiles to fixed-size circuits
- **Type Safety**: Linear type constraints preserved throughout compilation

### 7.2 Layer 0 Execution

The compilation chain ensures that high-level Intent operations ultimately execute as sequences of the 11 register machine instructions, maintaining:

- **Deterministic Execution**: Same Intent always produces same instruction sequence
- **Resource Linearity**: Hardware-enforced resource tracking preserved
- **Verifiable Execution**: All operations verifiable at the instruction level

## 8. Summary

Layer 2 represents the culmination of Causality's three-layer architecture, providing:

- **Declarative Programming**: Express what should happen, not how
- **Capability Security**: Fine-grained, unforgeable access control
- **Effect Composition**: Powerful combinators for complex operations
- **Static Verification**: Comprehensive analysis before execution
- **ZK Compatibility**: Clean compilation to fixed-size circuits

The capability system elegantly solves the tension between mathematical purity and practical expressiveness, enabling complex record operations while preserving the categorical foundations that make formal verification possible. By handling all dynamic operations at Layer 2, the system maintains clean abstraction boundaries while providing the flexibility needed for real-world applications.
