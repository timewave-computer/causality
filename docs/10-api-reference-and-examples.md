# External API Reference

This document outlines the primary Application Programming Interfaces (APIs) that the Causality framework provides for interaction with external systems. These include interfaces for blockchain integration, foreign function calls (FFI), and communication with coprocessors. The goal is to enable diverse external systems to leverage the framework's capabilities.

All core data structures intended for external interaction utilize SSZ (SimpleSerialize) for deterministic, efficient, and verifiable serialization.

## Core Data Structures for External Interaction

The following core data structures are fundamental when interacting with the framework's external APIs. They are typically serialized for cross-system communication (e.g., to blockchains) or passed via FFI.

### Resource
Represents a quantifiable asset or capability. Its SSZ-serialized form can be recorded or referenced by external systems.
```rust
pub struct Resource {
    pub id: EntityId,          // Unique identifier
    pub name: Str,             // Human-readable name
    pub domain_id: DomainId,   // Domain association
    pub resource_type: Str,    // Type classification
    pub quantity: u64,         // Quantity
    pub timestamp: Timestamp,  // Creation/update timestamp
}
```

### Intent
Represents a request for a resource transformation. External systems can submit SSZ-serialized `Intent`s to the framework.
```rust
pub struct Intent {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub priority: u32,
    pub inputs: Vec<ResourceFlow>,    // Required inputs
    pub outputs: Vec<ResourceFlow>,   // Desired outputs
    pub expression: Option<ExprId>,   // Optional transformation logic
    pub timestamp: Timestamp,
    pub optimization_hint: Option<ExprId>, // Hint for execution strategy
}
```

### Effect
Represents a planned or executed computational effect or resource transformation. SSZ-serialized `Effect`s can be used to communicate outcomes to external systems or record them on-chain.
```rust
pub struct Effect {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effect_type: Str,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>, // Optional associated expression
    pub timestamp: Timestamp,       // Creation timestamp
    pub hint: Option<ExprId>,         // Optional execution hint
}
```

### ResourceFlow
Describes the specification of resources for inputs or outputs in `Intent`s and `Effect`s.
```rust
pub struct ResourceFlow {
    pub resource_type: Str,
    pub quantity: u64,
    pub domain_id: DomainId,
}
```

## Blockchain Integration API

The Causality framework is designed for integration with blockchain systems. This typically involves submitting or retrieving SSZ-serialized data (like `Intent`s or `Effect`s) to/from smart contracts or on-chain storage.

**Key Aspects:**
*   **Serialization**: Core data structures are SSZ-encoded for compact, deterministic representation on-chain.
*   **Smart Contract Interaction**: APIs facilitate interaction with smart contracts that can process or store Causality framework data. This may include functions for:
    *   Submitting `Intent`s for on-chain processing or off-chain execution triggered by on-chain events.
    *   Recording executed `Effect`s or resulting `Resource` state changes.
    *   Querying framework-related data.
*   **Proof Submission**: Mechanisms for submitting proofs (e.g., zero-knowledge proofs related to `Effect` execution) to a blockchain for verification.

*(Specific function signatures and interaction patterns would be detailed per target blockchain.)*

## Foreign Function Interface (FFI)

An FFI layer allows programs in other languages (e.g., C, Python, WebAssembly via JS) to use core Causality framework logic.

**Key Aspects:**
*   **Exposed Functions**: A selection of Rust functions are exposed, potentially including:
    *   Serialization/deserialization of core types (e.g., `Intent_to_ssz_bytes`, `Effect_from_ssz_bytes`).
    *   Submitting an `Intent` to the framework's runtime.
    *   Querying the status or outcome of operations.
    *   Data validation functions.
*   **Data Marshaling**: Clear conventions for passing data (e.g., C-compatible structs, byte buffers for SSZ data) and managing memory across the FFI boundary.
*   **Error Handling**: Standardized error reporting from Rust to the calling language.

**Conceptual FFI Example:**
```c
// C-style declaration for an FFI function
// Takes SSZ-encoded intent bytes, returns an SSZ-encoded effect_id or error.
int32_t causality_submit_intent(
    const uint8_t* intent_ssz_ptr, uint32_t intent_ssz_len,
    uint8_t** out_effect_id_ssz_ptr, uint32_t* out_effect_id_ssz_len
);
void causality_free_ssz_bytes(uint8_t* ssz_ptr);
```

*(The actual FFI definition would provide comprehensive details on all exposed functions and types.)*

## Coprocessor Communication API

The framework can interface with coprocessors for specialized tasks (e.g., ZKP generation, complex simulations).

**Key Aspects:**
*   **Communication Protocol**: Defined protocol (e.g., gRPC, custom IPC) for sending tasks and receiving results.
*   **Data Exchange**: SSZ or other efficient serialization formats (like Protocol Buffers) for data sent to/from the coprocessor (e.g., an `Effect` specification, input data for computation, resulting proofs or data).
*   **Interface Definition**: Clear API for the coprocessor detailing supported operations and data formats.

*(Details depend on the specific coprocessor and integration design.)*