# External API Reference

This document outlines the primary Application Programming Interfaces (APIs) that the Causality framework provides for interaction with external systems. These include interfaces for blockchain integration, foreign function calls (FFI), and communication with coprocessors. The APIs expose the three-layer architecture while maintaining linear resource safety and enabling zero-knowledge proof generation.

All core data structures intended for external interaction utilize SSZ (SimpleSerialize) for deterministic, efficient, and verifiable serialization, enabling content-addressed storage and proof generation.

## Core Data Structures for External Interaction

The following core data structures are fundamental when interacting with the framework's external APIs. They incorporate the linear type system, row types for capabilities, and effect structures with pre/post conditions.

### Resource (Layer 0/1)
Represents a linear resource with capabilities tracked via row types. Resources are consumed exactly once unless wrapped in an Object with different linearity.
```rust
pub struct Resource<T> {
    pub id: ResourceId,           // Unique identifier
    pub data: T,                  // Resource data
    pub owner: Address,           // Current owner
    pub consumed: bool,           // Consumption status
    pub capabilities: RowType,    // Compile-time capability tracking
    pub timestamp: Timestamp,     // Creation timestamp
}
```

### Object (Layer 1)
Generalizes resources with configurable linearity qualifiers.
```rust
pub struct Object<T, L: Linearity> {
    pub data: T,                     // Object data
    pub linearity: L,                // Linear | Affine | Relevant | Unrestricted
    pub capabilities: Set<Capability>, // Available capabilities
}
```

### Intent (Layer 2)
Represents a declarative request for a desired state transformation, specifying inputs, outputs, and optional computational logic or hints. Intents are processed by the Layer 2 effects engine, potentially leading to the construction of a Temporal Effect Graph (TEG).
```rust
pub struct Intent {
    pub id: EntityId,                 // Unique identifier for the intent
    pub name: Str,                    // User-friendly name or description for the intent
    pub domain_id: DomainId,          // Identifies the application domain this intent belongs to
    pub priority: u32,                // Execution priority for scheduling or conflict resolution
    pub inputs: Vec<ResourceFlow>,    // Describes the expected input resources or data flows
    pub outputs: Vec<ResourceFlow>,   // Describes the expected output resources or data flows
    pub expression: Option<ExprId>,   // Optional Layer 1 Lisp expression ID defining the intent's core logic or transformation rules
    pub timestamp: Timestamp,         // Timestamp of intent creation or submission
    pub hint: Option<ExprId>,         // Optional Layer 1 Lisp expression ID providing optimization or execution hints for the effects engine
}

// Note: ResourceFlow would be a struct detailing expected resource types, quantities, and potentially specific IDs or constraints.
// type Str = String; // Or a specific interned string type
// type EntityId = [u8; 32]; // Typically an SSZ hash
// type DomainId = [u8; 32]; // Or a more structured identifier
// type ExprId = [u8; 32]; // Content-addressed ID of a Lisp expression
```

### Effect (Layer 2)
Represents a specific, potentially state-changing operation. Effects are the building blocks of computation and resource transformation within Layer 2. Each effect is content-addressable via its SSZ hash, forming its unique `id`.
```rust
pub struct Effect {
    pub id: EntityId,                 // Unique, content-addressed identifier (derived from SSZ hash of its content)
    pub name: Str,                    // User-friendly name for the effect
    pub domain_id: DomainId,          // The application domain this effect pertains to
    pub effect_type: Str,             // A string categorizing the effect (e.g., "transfer_tokens", "compute_data_hash", "invoke_external_service")
    pub inputs: Vec<ResourceFlow>,    // Specifies the input resources or data flows required by the effect
    pub outputs: Vec<ResourceFlow>,   // Specifies the output resources or data flows produced by the effect
    pub expression: Option<ExprId>,   // Optional Layer 1 Lisp expression ID detailing the effect's internal logic or how outputs are derived from inputs
    pub timestamp: Timestamp,         // Timestamp associated with the effect's creation or proposed execution
    pub hint: Option<ExprId>,         // Optional Layer 1 Lisp expression ID providing hints for execution, optimization, or ZK proof generation
}

// Note: ResourceFlow, Str, EntityId, DomainId, ExprId are as defined in the Intent section.
```

### Handler (Layer 2)
In the Causality framework, a `Handler` is conceptually a pure function that transforms one or more input `Effect`s into one or more output `Effect`s. In the Rust implementation, this is typically represented by a `trait`.

```rust
// Conceptual Trait Representation
pub trait Handler {
    type InputEffect;  // Could be a specific Effect type or a generic parameter
    type OutputEffect; // Could be a specific Effect type or a generic parameter

    fn name(&self) -> Str;
    fn transform(&self, input: Self::InputEffect) -> Self::OutputEffect; // Pure transformation
    // Could also include methods for pre/post condition checks, capability requirements, etc.
}
```
Handlers are key to the algebraic effect system in Layer 2, allowing for composable and verifiable transformations. The API might provide mechanisms to register, discover, and invoke implementations of such handler traits.

### TemporalEffectGraph (Layer 2)
Represents the causal ordering of effects based on resource dependencies.
```rust
pub struct TemporalEffectGraph {
    pub nodes: Vec<EffectNode>,       // Effect nodes with metadata
    pub edges: Vec<CausalEdge>,       // Resource flow dependencies
    pub constraints: Vec<Constraint>,  // Global constraints
}
```

## Blockchain Integration API

The Causality framework integrates with blockchain systems while maintaining linear resource safety and enabling zero-knowledge proof generation for private execution.

**Key Aspects:**
* **Linear Resource Tracking**: On-chain representation of linear resources with nullifiers to prevent double-spending
* **Effect Commitment**: Effects can be committed on-chain with their pre/post conditions for later verification
* **Proof Submission**: Zero-knowledge proofs of effect execution, including:
  * Resource conservation proofs
  * Linearity enforcement proofs
  * Effect pre/post condition satisfaction
* **TEG Anchoring**: Temporal Effect Graphs can be anchored on-chain for causal ordering verification

**Example Smart Contract Interface:**
```solidity
interface CausalityBridge {
    // Submit an intent for processing
    function submitIntent(bytes calldata intentSSZ) external returns (bytes32 intentId);
    
    // Commit to an effect execution
    function commitEffect(bytes32 effectHash, bytes32 nullifierSet) external;
    
    // Verify effect execution with ZK proof
    function verifyEffect(
        bytes32 effectHash,
        bytes calldata proof,
        bytes32[] calldata publicInputs
    ) external returns (bool);
    
    // Anchor a TEG for causal verification
    function anchorTEG(bytes32 tegRoot, uint256 timestamp) external;
}
```

## Foreign Function Interface (FFI)

The FFI layer exposes the three-layer architecture while maintaining safety guarantees around linear resource consumption.

**Layer 0 Functions (Register Machine):**
```c
// Execute register machine instructions
int32_t causality_execute_instruction(
    const uint8_t* instruction_ssz, uint32_t instruction_len,
    uint8_t** out_state_ssz, uint32_t* out_state_len
);

// Verify conservation laws
bool causality_verify_conservation(
    const uint8_t* input_resources_ssz, uint32_t inputs_len,
    const uint8_t* output_resources_ssz, uint32_t outputs_len
);
```

**Layer 1 Functions (Type System):**
```c
// Row type operations (compile-time in Rust, runtime via FFI)
int32_t causality_row_type_project(
    const uint8_t* row_type_ssz, uint32_t row_len,
    const char* field_name,
    uint8_t** out_value_ssz, uint32_t* out_value_len
);

// Check linearity constraints
bool causality_check_linearity(
    const uint8_t* resource_ssz, uint32_t resource_len,
    uint8_t linearity_qualifier
);
```

**Layer 2 Functions (Effects & Intents):**
```c
// Submit intent for processing
int32_t causality_submit_intent(
    const uint8_t* intent_ssz, uint32_t intent_len,
    uint8_t** out_teg_ssz, uint32_t* out_teg_len
);

// Apply handler to effect
int32_t causality_apply_handler(
    const uint8_t* effect_ssz, uint32_t effect_len,
    const uint8_t* handler_ssz, uint32_t handler_len,
    uint8_t** out_effect_ssz, uint32_t* out_effect_len
);

// Generate ZK proof for effect execution
int32_t causality_prove_effect(
    const uint8_t* effect_ssz, uint32_t effect_len,
    const uint8_t* witness_ssz, uint32_t witness_len,
    uint8_t** out_proof_ssz, uint32_t* out_proof_len
);

// Memory management
void causality_free_ssz_bytes(uint8_t* ssz_ptr);
```

## Coprocessor Communication API

The framework interfaces with specialized coprocessors for computationally intensive tasks while maintaining the linear resource model.

**Key Integration Points:**
* **Layer 1 Lisp Compilation**: Offload compilation of Layer 1 Causality Lisp (11 core primitives) to Layer 0 register machine instructions (9 instructions).
* **ZK Circuit Generation**: Generate arithmetic circuits from Layer 0 register machine instruction sequences.
* **Row Type Inference**: Complex type inference for Layer 1 row polymorphism (if offloaded).
* **TEG Optimization**: Optimize effect ordering in Layer 2 Temporal Effect Graphs.

**REST API Endpoints:**
```
POST /api/v1/compile
Content-Type: application/octet-stream
Body: SSZ-encoded CompileRequest

Response: SSZ-encoded CompileResponse
```

```
POST /api/v1/prove
Content-Type: application/octet-stream
Body: SSZ-encoded ProofRequest

Response: SSZ-encoded ProofResponse
```

**SSZ Type Definitions:**
```rust
// Compilation request/response types
#[derive(Serialize, Deserialize, Ssz)]
pub struct CompileRequest {
    pub lisp_expression: Vec<u8>,
    pub optimization_level: u8,
    pub target: CompilationTarget,
}

#[derive(Serialize, Deserialize, Ssz)]
pub struct CompileResponse {
    pub register_ir: Vec<u8>,
    pub type_annotations: Vec<TypeAnnotation>,
    pub warnings: Vec<String>,
    pub gas_estimate: u64,
}

// Proof generation request/response types
#[derive(Serialize, Deserialize, Ssz)]
pub struct ProofRequest {
    pub register_ir: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub private_witness: Vec<u8>,
    pub proof_type: ProofType,
}

#[derive(Serialize, Deserialize, Ssz)]
pub struct ProofResponse {
    pub proof: Vec<u8>,
    pub verification_key: Vec<u8>,
    pub constraint_count: u64,
    pub proof_size: u64,
}

// TEG optimization request/response
#[derive(Serialize, Deserialize, Ssz)]
pub struct OptimizeTEGRequest {
    pub teg: TemporalEffectGraph,
    pub optimization_hints: Vec<Hint>,
    pub constraints: Vec<Constraint>,
}

#[derive(Serialize, Deserialize, Ssz)]
pub struct OptimizeTEGResponse {
    pub optimized_teg: TemporalEffectGraph,
    pub execution_plan: Vec<ExecutionStep>,
    pub estimated_cost: u64,
}
```

**Example REST API Usage:**
```rust
// Compile Lisp to register machine IR
let compile_request = CompileRequest {
    lisp_expression: b"(let [[x 10]] (+ x 20))".to_vec(),
    optimization_level: 2,
    target: CompilationTarget::RegisterMachine,
};

let response = client
    .post("https://coprocessor.example.com/api/v1/compile")
    .header("Content-Type", "application/octet-stream")
    .body(compile_request.as_ssz_bytes())
    .send()
    .await?;

let compile_response = CompileResponse::from_ssz_bytes(&response.bytes().await?)?;
```

## Example Usage Patterns

### Linear Resource Transfer
```rust
// Create a linear token resource
let token = Resource {
    id: ResourceId::new(),
    data: TokenData { amount: 100 },

let transfer_effect_id = calculate_effect_id(...); // Placeholder for SSZ hashing logic

let transfer_effect = Effect {
    id: transfer_effect_id, // Typically derived from SSZ hash of other fields
    name: "Transfer Token X".into(),
    domain_id: "finance_domain_v1".into(), // Example DomainId
    effect_type: "token_transfer".into(),
    inputs: vec![
        ResourceFlow::new_linear_input(token_resource_id, "TokenX", alice_address, 100u64), // Consumes 100 of TokenX from Alice
    ],
    outputs: vec![
        ResourceFlow::new_linear_output(token_resource_id, "TokenX", bob_address, 100u64), // Produces 100 of TokenX for Bob
    ],
    expression: None, // For a simple transfer, logic might be implicit in the effect_type and runtime handler
    timestamp: current_timestamp(),
    hint: None,
};

// Submit via FFI
let effect_bytes = transfer_effect.as_ssz_bytes();
let mut teg_bytes = std::ptr::null_mut();
let mut teg_len = 0;

let result = unsafe {
    causality_submit_intent(
        effect_bytes.as_ptr(),
        effect_bytes.len() as u32,
        &mut teg_bytes,
        &mut teg_len,
    )
};
```

### Privacy-Preserving Execution
```rust
// Create private effect with commitment
let private_effect = Effect {
    tag: Symbol::from("private_transfer"),
    params: commit(transfer_params),  // Committed parameters
    pre: constraint!(verify_nullifier(nullifier)),
    post: constraint!(new_nullifier_published),
    hints: vec![Hint::RequirePrivacy],
    timestamp: Timestamp::now(),
};

// Generate ZK proof of execution
let witness = generate_witness(transfer_params, private_data);
let proof = prove_effect_execution(private_effect, witness);
```