# Resources and Registers: Core Concepts

This document explains the conceptual relationship between resources and registers in the Causality system and why this distinction is important for the system architecture.

## System Boundaries and Domains

Before diving into the core concepts, it's important to understand the key boundaries in the Causality system:

### Inside vs. Outside System Boundary

- **Outside the System**: The external world including users, external applications, and services that interact with Causality
- **Inside the System**: The components and mechanisms that comprise the Causality protocol and its implementation

### On-Chain vs. Off-Chain Boundary

- **On-Chain Components**: Elements that exist directly on the blockchain, with their state secured by consensus
- **Off-Chain Components**: Elements that exist outside the blockchain but interact with on-chain components

These boundaries help clarify where different components of the system operate and how they interact.

## Core Concepts

### Program Accounts (User Interface)

Program accounts are the primary user touchpoints in the Causality system. They display token balances, data, and available effects that users can invoke, serving as both the interface for user interaction and the execution context for operations.

**Boundary Classification**:
- **System Boundary**: Gateway between outside and inside the system
- **Chain Boundary**: Can have both on-chain components (smart contracts) and off-chain components (user interfaces)

**Key characteristics of program accounts:**
- Present token balances, data, and available actions to users
- Provide the interface through which users interact with the system
- Contain executable code (programs or smart contracts)
- Define rules for resource manipulation
- Control access to underlying resources and registers
- May hold resources temporarily during multi-step operations
- Can span across multiple domains for cross-domain operations

### Effects (Actions and Operations)

Effects are the actions and operations that users can invoke through program accounts. These include transferring tokens, updating data, granting rights, and other state-changing operations that have outcomes visible to users.

**Boundary Classification**:
- **System Boundary**: Inside the system
- **Chain Boundary**: Can be initiated off-chain but must be processed on-chain

**Key characteristics of effects:**
- Operations that users can invoke through program accounts
- Represent actions with concrete outcomes and consequences
- Have intuitive user-facing properties and behaviors
- Are the mechanisms of user intent (transfer tokens, update data, etc.)
- Can be composed together to form complex transactions

### Resources

Resources are system-level abstractions that represent the assets, data, and capabilities managed by the system. They serve as the internal representations that translate user-initiated effects into manageable entities.

**Boundary Classification**:
- **System Boundary**: Entirely inside the system
- **Chain Boundary**: Logical objects that span both on-chain and off-chain

**Key characteristics of resources:**
- Internal system representations of assets, data, and capabilities
- Have a logical identity and lifecycle
- Can be created, transferred, updated, or destroyed
- Provide a consistent interface for the system
- May exist across multiple domains
- Objects that bridge user intents to on-chain state

### Registers

Registers are the fundamental storage and accounting mechanism that underlies resources. They provide the technical implementation that makes resource management possible.

**Boundary Classification**:
- **System Boundary**: Entirely inside the system
- **Chain Boundary**: Explicitly on-chain storage elements

**Key characteristics of registers:**
- On-chain storage units for resource state
- Follow a one-time use model for state transitions
- Contain cryptographic proofs and commitments
- Implement resource conservation laws
- Enable cross-domain verification through ZK proofs
- Provide the source of truth for the state of all resources

## The System Architecture

The relationship between these concepts forms the architecture of the Causality system:

1. **Program Accounts** (user interface layer): What users interact with directly, showing balances and available effects
2. **Effects** (action layer): Operations that users invoke through program accounts
3. **Resources** (system abstraction layer): How the system represents and manages assets and data
4. **Registers** (state and verification layer): Where the actual state is stored and verified cryptographically on-chain

This layered architecture allows for a clean separation of concerns while maintaining a coherent system across the inside/outside and on-chain/off-chain boundaries.

### Program Accounts as User Touchpoints

Program accounts serve as user touchpoints by:
- Displaying token balances, NFTs, data, and other assets to users
- Showing available effects that users can invoke
- Providing an interface for user authorization and consent
- Organizing related functionality in a way that makes sense to users
- Translating user actions from outside the system into operations inside the system

### Program Accounts as Resource Controllers

Program accounts act as controllers for resources by:
- Enforcing business logic for resource manipulation
- Applying domain-specific rules to resource operations
- Verifying user authorization before allowing resource operations
- Orchestrating complex operations that may involve multiple resources

### Program Accounts as Register Operators

Program accounts interact with registers by:
- Initiating register operations (create, update, transfer, delete) on the blockchain
- Generating the necessary proofs for register operations
- Verifying proofs for incoming cross-domain register operations
- Managing nullifiers and register state transitions

## The Complete Relationship Model

The relationship between program accounts, effects, resources, and registers can be understood through several perspectives:

### 1. Abstraction and Boundary Layers

Each component exists at a different abstraction layer and boundary:
- **Program Accounts**: User interface layer (boundary between outside and inside the system)
- **Effects**: Action layer (operations that cross from off-chain to on-chain)
- **Resources**: System abstraction layer (logical objects bridging on-chain and off-chain)
- **Registers**: State storage layer (explicitly on-chain)

This separation of concerns allows each layer to evolve independently while maintaining compatibility across boundaries.

### 2. Ownership and State Model

From user perspective to implementation:

```
Outside System:
User (external) → Interacts with Program Account Interface (boundary)

Inside System:
Program Account (boundary) → User invokes transfer effect → Effect processed

On-Chain:
Resource (logical object) → Token resource owned by Alice → Token resource owned by Bob
Register (storage) → Register 1 (consumed) → Register 2 (active)
```

Each state change initiated by a user through a program account invokes an effect, which propagates through the system layers, with resources tracking the logical state and registers managing the on-chain cryptographic state.

### 3. Conservation Laws

Conservation laws (e.g., tokens cannot be duplicated) are enforced across all layers:
- **Program Accounts**: Display consistent balances to users before and after operations
- **Effects**: Define operations that preserve conservation properties
- **Resources**: Track logical conservation of represented assets
- **Registers**: Provide cryptographic proofs of conservation through on-chain nullifiers and ZK circuits

### 4. Cross-Domain Operations

Cross-domain operations involve all layers and boundaries:
- **Program Accounts**: Provide a consistent user experience across domains
- **Effects**: Define operations that work across domain boundaries
- **Resources**: Maintain logical identity across domains
- **Registers**: Provide on-chain cryptographic proofs for cross-domain verification

## Example: Token Transfer Across All Layers and Boundaries

Let's examine a token transfer from Alice to Bob across all layers:

**Outside the System (User):**
- Alice decides to send 10 tokens to Bob
- Alice interacts with a user interface

**Program Account perspective (boundary component):**
- Alice sees her token balance of 100 in her program account interface
- Alice invokes a "transfer" effect to send 10 tokens to Bob
- After confirmation, Alice's balance shows 90 and Bob's shows 110
- The program account interface handles authorization and confirms the transaction

**Effect perspective (crosses from off-chain to on-chain):**
- The transfer effect defines the movement of 10 tokens from Alice to Bob
- The effect carries parameters like amount, sender, and recipient
- The effect has defined rules for when it can be successfully invoked

**Resource perspective (logical on-chain object):**
- Token resource ownership changes from Alice to Bob
- The token resource maintains its identity and properties

**Register perspective (on-chain storage):**
1. Alice's register containing the token is marked as consumed on-chain
2. A nullifier is generated on-chain to prevent reuse
3. A new register is created on-chain for Bob containing the same token amount
4. A ZK proof verifies this was done correctly without creating or destroying tokens

## Program Account Types and Specialized Roles

Different types of program accounts serve different roles in bridging the outside world to the system:

### Asset Program Accounts

- Display and manage specific types of assets (tokens, NFTs, etc.)
- Provide interfaces for asset-specific operations (minting, burning, etc.)
- Enforce asset-specific rules (transfer limits, royalties, etc.)

### Utility Program Accounts

- Provide common functionality used by multiple asset programs
- Implement shared operations (swaps, escrow, etc.)
- Manage utility functions and shared state

### Domain Bridge Program Accounts

- Facilitate cross-domain asset transfers
- Display cross-domain transaction status
- Manage verification of cross-domain operations

## Implications for System Design

The layered model with explicit boundaries has important implications for system design:

1. **API Design**: 
   - External APIs operate at the program account level for outside users
   - Effect APIs define available operations at the system boundary
   - Resource APIs handle system-level abstractions inside the system
   - Register APIs manage on-chain state and cryptographic proof

2. **Security Enforcement**:
   - Program accounts enforce user authorization at the system boundary
   - Effects enforce operational constraints as they cross to on-chain
   - Resources enforce logical consistency within the system
   - Registers enforce state integrity and cryptographic verification on-chain

3. **Composability**:
   - Program accounts can be composed at the user interface level (outside system)
   - Effects can be composed into transactions (crossing the boundary)
   - Resources can be composed at the system level (inside system)
   - Registers ensure integrity of composed operations (on-chain)

4. **Upgrade Paths**:
   - External interfaces can evolve without changing internal components
   - Program account interfaces can evolve without changing effect definitions
   - Effects can evolve without changing resource models
   - Resources can evolve without changing on-chain register formats

5. **Regulatory Compliance**:
   - Program accounts can adapt presentation to different jurisdictions
   - Effects can implement jurisdiction-specific constraints
   - Resources can include compliance metadata
   - Registers provide on-chain cryptographic proof of rule enforcement

## Conclusion

Understanding the relationship between program accounts, effects, resources, and registers across system and chain boundaries is crucial for Causality system development. Program accounts bridge the outside world to the system, showing balances and available effects. Effects are operations that cross from off-chain to on-chain. Resources provide the logical representation of assets and data, while registers provide the on-chain implementation mechanism that ensures security, privacy, and interoperability across domains.

This separation of concerns with clear boundaries allows the system to maintain high-level invariants while leveraging specialized implementations for different domains and asset types.

# Addendum: Program Accounts, Resource Access, and Message Flows

## Unified Resource API and Composability Model

While program accounts serve as the boundary objects that represent resources to the outside world, internally the system needs a unified mechanism for programs to interact with resources. This addendum explains how programs interact with resources through a unified API, how this supports privacy, and clarifies the differences between on-chain and user-facing representations of program accounts.

### Unified Resource API

Programs interact with resources through a capability-based Resource API that provides a consistent interface for resource manipulation, with the following design principles:

#### Core Principles of the Unified Resource API

1. **Capability-Based Access**: Programs never access resources directly; they receive unforgeable capability references
2. **Composable Delegation**: Capabilities can be delegated, attenuated, and composed
3. **Intent-Based Operations**: API operations express user intent rather than raw state mutations
4. **Privacy-Preserving**: Operations generate only the minimal proofs needed to validate state transitions
5. **Consistency Guarantees**: The API ensures atomic operations and causal consistency

#### Key Abstractions in the Resource API

```rust
// Core capability type - unforgeable reference to a resource
struct ResourceCapability {
    id: CapabilityId,               // Unique identifier
    resource_id: ResourceId,        // Resource this capability grants access to
    rights: Vec<Rights>,            // What operations this capability allows
    restrictions: Vec<Restriction>, // Constraints on the capability
    expiration: Timestamp,          // When this capability expires
}

// Core resource operations
trait ResourceAPI {
    // Read operations
    async fn query(&self, capability: ResourceCapability, query: Query) -> Result<QueryResult, Error>;
    
    // Write operations
    async fn update(&self, capability: ResourceCapability, operation: Operation) -> Result<UpdateResult, Error>;
    
    // Capability operations
    fn delegate(&self, capability: ResourceCapability, restrictions: Vec<Restriction>) -> ResourceCapability;
    fn compose(&self, capabilities: Vec<ResourceCapability>) -> CompositeCapability;
    async fn revoke(&self, capability: ResourceCapability) -> Result<(), Error>;
    
    // Witness operations (for privacy-preserving workflows)
    fn generate_witness(&self, capability: ResourceCapability, operation: Operation) -> Witness;
    fn verify_witness(&self, witness: Witness, public_inputs: PublicInputs) -> bool;
}
```

### Program-to-Program Composition

Far from limiting composability, the unified API enables rich composition patterns by making capabilities first-class objects:

1. **Capability Delegation**:
   ```rust
   // Program A delegates a capability to Program B
   let restricted_capability = resource_api.delegate(my_capability, Restrictions {
       max_amount: Some(100),
       time_limit: Some(Duration::from_secs(600)), // 10 minutes
       allowed_operations: vec![Operation::Read, Operation::Transfer],
   });
   
   let result = program_b.invoke("process_payment", InvokeParams {
       payment_capability: restricted_capability,
       amount: 50,
   }).await?;
   ```

2. **Capability Composition**:
   ```rust
   // Compose multiple capabilities into a single atomic operation
   let composite_capability = resource_api.compose(vec![
       token_capability,
       nft_capability,
       storage_capability,
   ]);
   
   // The composite operation will either fully succeed or fully fail
   let result = resource_api.update(composite_capability, Operation::AtomicSwap {
       operations: vec![
           Operation::TransferToken { amount: 100, to: recipient_id },
           Operation::TransferNFT { token_id: nft_id, to: sender_id },
           Operation::StoreData { data: swap_receipt }
       ],
   }).await?;
   ```

3. **Intent-Based Composition**:
   ```rust
   // Higher-level composition based on intent rather than raw operations
   let swap_result = exchange_program.invoke("limit_order", InvokeParams {
       sell_token: token_a_capability,
       buy_token: token_b_capability,
       sell_amount: 100,
       min_buy_amount: 200,
       deadline: timestamp + 3600,
   }).await?;
   ```

This capability-based model creates a clean composition interface while preserving security boundaries, allowing programs to safely interact without exposing their internal implementation details.

## On-Chain vs. User-Facing Program Accounts

There's an important distinction between how program accounts are represented on-chain versus how they appear to users:

### On-Chain Representation (Smart Contracts)

On-chain, a program account is represented by a minimal set of cryptographic commitments and state transitions:

```rust
OnChainProgramAccount {
  id: AccountId,
  owner: PublicKey,
  programLogic: ContentHash,  // Content-addressed program logic
  
  // Privacy-preserving state
  commitments: Map<CommitmentId, Commitment>,
  nullifiers: Set<NullifierId>,
  
  // Public state (if any)
  publicState: Map<StateKey, StateValue>,
  
  // Capabilities granted to other programs
  grantedCapabilities: Map<CapabilityId, CapabilityCommitment>
}
```

This on-chain representation contains the minimal information needed to verify state transitions and enforce access control, without revealing the full contents of the resources controlled by the account.

### User-Facing Representation

For users, a program account is a rich interface showing available resources and possible actions:

```rust
UserFacingProgramAccount {
  id: AccountId,
  displayName: String,
  resources: {
    tokens: Map<TokenId, {
      balance: Amount,
      metadata: TokenMetadata
    }>,
    nfts: Array<NFTMetadata>,
    data: Map<DataKey, DataValue>
  },
  availableActions: Array<{
    id: ActionId,
    name: String,
    description: String,
    parameters: Array<ActionParameter>
  }>,
  activityLog: Array<ActivityEntry>
}
```

This representation is constructed client-side using:
1. Public on-chain data
2. Private user data (notes, witnesses, keys)
3. Decrypted shared data (if applicable)

### Smart Contract vs. Program Account Relationship

The relationship between smart contracts and program accounts can be understood as:

1. **Smart Contracts**: On-chain executable code that enforces rules and processes state transitions
2. **Program Accounts**: User-visible interfaces that present resources and available actions
3. **Connection**: Smart contracts validate program account operations, while program accounts provide interfaces to smart contract functionality

This separation allows the system to evolve the user experience independently of the on-chain logic, while still maintaining security and privacy guarantees.

## Message Flows for Resource Interaction

Different actors interact with resources through different pathways:

### 1. User → Program Account → Resources Flow

When a user wants to interact with resources:

```
┌─────────┐          ┌───────────────┐          ┌────────────┐          ┌──────────┐
│  User   │  ─────►  │Program Account│  ─────►  │Resource API│  ─────►  │ Registers│
└─────────┘          └───────────────┘          └────────────┘          └──────────┘
     │                      │                          │                      │
     │                      │                          │                      │
     │  1. Initiate action  │                          │                      │
     │ ─────────────────────►                          │                      │
     │                      │                          │                      │
     │                      │  2. Request capability   │                      │
     │                      │ ─────────────────────────►                      │
     │                      │                          │                      │
     │                      │  3. Return capability    │                      │
     │                      │ ◄─────────────────────────                      │
     │                      │                          │                      │
     │                      │  4. Invoke operation     │                      │
     │                      │ ─────────────────────────►                      │
     │                      │                          │                      │
     │                      │                          │  5. Update registers │
     │                      │                          │ ─────────────────────►
     │                      │                          │                      │
     │                      │                          │  6. Confirm update   │
     │                      │                          │ ◄─────────────────────
     │                      │                          │                      │
     │                      │  7. Return result        │                      │
     │                      │ ◄─────────────────────────                      │
     │                      │                          │                      │
     │  8. Display result   │                          │                      │
     │ ◄─────────────────────                          │                      │
     │                      │                          │                      │
```

Key aspects of this flow:
1. User initiates an action through the program account interface
2. Program account requests necessary capabilities from the Resource API
3. Operation is executed with appropriate capabilities
4. Registers are updated with new state
5. Result is returned to the user

### 2. Program → Resources Flow

When a program needs to interact with resources:

```
┌────────────┐          ┌────────────┐          ┌──────────┐
│  Program   │  ─────►  │Resource API│  ─────►  │Registers │
└────────────┘          └────────────┘          └──────────┘
      │                        │                      │
      │                        │                      │
      │  1. Request capability │                      │
      │ ───────────────────────►                      │
      │                        │                      │
      │  2. Verify rights      │                      │
      │ ◄───────────────────────                      │
      │                        │                      │
      │  3. Submit operation   │                      │
      │ ───────────────────────►                      │
      │                        │                      │
      │                        │  4. Update registers │
      │                        │ ─────────────────────►
      │                        │                      │
      │                        │  5. Confirm update   │
      │                        │ ◄─────────────────────
      │                        │                      │
      │  6. Return result      │                      │
      │ ◄───────────────────────                      │
      │                        │                      │
```

Key aspects of this flow:
1. Program requests capabilities for resources it needs to access
2. Resource API verifies the program has appropriate rights
3. Program submits operations using the granted capabilities
4. Resource API processes the operations and updates registers
5. Results are returned to the program

### 3. On-Chain Smart Contract → Resources Flow

For on-chain operations:

```
┌─────────────────┐          ┌────────────┐          ┌──────────┐
│ Smart Contract  │ ─────► │ Resource API │ ─────► │ Registers │
└─────────────────┘          └────────────┘          └──────────┘
         │                         │                      │
         │                         │                      │
         │ 1. Verify transaction   │                      │
         │ signature or ZK proof   │                      │
         │                         │                      │
         │ 2. Request capability   │                      │
         │ ────────────────────────►                      │
         │                         │                      │
         │ 3. Process operation    │                      │
         │ ────────────────────────►                      │
         │                         │                      │
         │                         │ 4. Update registers  │
         │                         │ ────────────────────►
         │                         │                      │
         │                         │ 5. Emit events       │
         │                         │ ◄────────────────────
         │                         │                      │
         │ 6. Return transaction   │                      │
         │ result and events       │                      │
         │ ◄────────────────────────                      │
         │                         │                      │
```

Key aspects of this flow:
1. Smart contract verifies transaction validity (signature or ZK proof)
2. Smart contract executes the operation through the Resource API
3. Registers are updated with new state
4. Events are emitted for external observers
5. Transaction results are returned

### 4. Privacy-Preserving Program Account → Resources Flow

For privacy-sensitive operations:

```
┌─────────────┐          ┌───────────────┐          ┌────────────┐          ┌──────────┐
│  User       │  ─────►  │Program Account│  ─────►  │Resource API│  ─────►  │ Registers│
│(Client-side)│          └───────────────┘          └────────────┘          └──────────┘
└─────────────┘                  │                         │                      │
      │                          │                         │                      │
      │ 1. Construct operation   │                         │                      │
      │ locally                  │                         │                      │
      │                          │                         │                      │
      │ 2. Generate witness      │                         │                      │
      │ and ZK proof             │                         │                      │
      │                          │                         │                      │
      │ 3. Submit proof and      │                         │                      │
      │ public inputs            │                         │                      │
      │ ─────────────────────────►                         │                      │
      │                          │                         │                      │
      │                          │ 4. Verify ZK proof      │                      │
      │                          │ ────────────────────────►                      │
      │                          │                         │                      │
      │                          │                         │ 5. Update commitment │
      │                          │                         │ and nullifier sets   │
      │                          │                         │ ─────────────────────►
      │                          │                         │                      │
      │                          │                         │ 6. Confirm update    │
      │                          │                         │ ◄─────────────────────
      │                          │                         │                      │
      │                          │ 7. Return result        │                      │
      │                          │ ◄────────────────────────                      │
      │                          │                         │                      │
      │ 8. Update local state    │                         │                      │
      │ with new notes and       │                         │                      │
      │ mark spent notes         │                         │                      │
      │                          │                         │                      │
```

Key aspects of this flow:
1. User constructs the operation client-side using their private notes
2. User generates a zero-knowledge proof that the operation is valid
3. Only the proof and public inputs are submitted to the blockchain
4. System verifies the proof without learning the private inputs
5. Commitments and nullifiers are updated on-chain
6. User locally updates their notes to reflect the new state

## Capability-Based Composition Example

To illustrate how programs interact with resources and compose with each other through the unified Resource API, consider this example of a complex operation:

```rust
// User wants to swap tokens and then use them in a yield strategy
async fn swap_and_stake() -> Result<StakeResponse, Error> {
    // 1. User initiates the action through their program account
    let user_account = get_user_account(user_id).await?;
    
    // 2. Program account requests capabilities for the relevant resources
    let source_token_capability = resource_api.request_capability(CapabilityRequest {
        resource_id: ResourceId::new("token:ETH"),
        rights: vec![Rights::Transfer],
        amount: Some(10),
    }).await?;
    
    // 3. Program account delegates a restricted capability to the DEX program
    let dex_capability = resource_api.delegate(source_token_capability, Restrictions {
        max_amount: Some(10),
        time_limit: Some(Duration::from_secs(300)), // 5 minutes
        allowed_operations: vec![Operation::Swap],
    });
    
    // 4. DEX program executes the swap using its capability
    let swap_result = dex_program.invoke("swap", SwapParams {
        source_capability: dex_capability,
        target_token: "token:USDC".into(),
        min_received: 1800,
    }).await?;
    
    // 5. DEX program returns a capability for the swapped tokens
    let usdc_capability = swap_result.target_capability;
    
    // 6. Program account delegates a capability to the yield program
    let yield_capability = resource_api.delegate(usdc_capability, Restrictions {
        max_amount: Some(swap_result.received),
        time_limit: Some(Duration::from_secs(300)), // 5 minutes
        allowed_operations: vec![Operation::Deposit],
    });
    
    // 7. Yield program uses the capability to stake the tokens
    let stake_result = yield_program.invoke("stake", StakeParams {
        token_capability: yield_capability,
        strategy: "stableCoinYield".into(),
        duration: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
    }).await?;
    
    // 8. Result is returned to the user
    Ok(StakeResponse {
        swapped: swap_result.received,
        staked: stake_result.amount,
        expected_yield: stake_result.projected_yield,
        unlock_date: stake_result.unlock_timestamp,
    })
}
```

In this example:
1. The user triggers a complex workflow
2. Multiple programs collaborate through capability delegation
3. Each program only receives the minimal capabilities it needs
4. Operations compose naturally through capability flows
5. The user gets a coherent result from the composed operation

This capability-based composition model allows for rich, secure interactions between programs while preserving the unified resource access model.

## Privacy and On-Chain Representation

For privacy-preserving workflows, the on-chain representation contains only the minimal cryptographic proof needed to validate state transitions:

```rust
OnChainPrivacyPreservingState {
  // Commitment set (what exists)
  commitments: MerkleTree<CommitmentHash>,
  
  // Nullifier set (what's been spent)
  nullifiers: Set<NullifierId>,
  
  // Public parameters for the ZK circuits
  verificationKey: VerificationKey,
  
  // Transaction outputs
  transactions: Array<{
    publicInputs: PublicInputs,
    proof: ZKProof,
    newCommitments: Array<CommitmentHash>,
    nullifiers: Array<NullifierId>
  }>
}
```

The user's client-side program account maintains the private state:

```rust
ClientSideAccount {
  // Private notes representing resources
  notes: Array<{
    commitment: CommitmentHash,
    nullifier: NullifierId,
    resourceId: ResourceId,
    value: Value,
    blinding: Randomness
  }>,
  
  // Tracking which notes have been spent
  spentNotes: Set<NullifierId>,
  
  // Witness data for generating proofs
  witnessData: Map<CommitmentHash, WitnessData>
}
```

When a user wants to perform a private operation:
1. They select the notes they want to use
2. They construct the operation locally
3. They generate a zero-knowledge proof that the operation is valid
4. They submit only the proof and the public inputs to the blockchain
5. The smart contract verifies the proof and updates the on-chain state
6. The user updates their local state accordingly

This approach allows users to maintain complete privacy while still enabling complex composed operations across multiple programs.

## Achieving Composability with Privacy

Privacy and composability might seem at odds, but they can be reconciled through:

1. **Capability-Based Authorization**: Programs receive capabilities that grant specific rights without revealing the underlying resources

2. **Private Inputs to Shared Circuits**: Programs compose by agreeing on circuit interfaces while keeping their inputs private

3. **Revealed Outputs with Hidden Internals**: Programs can reveal the outputs of operations while keeping the inputs and intermediate steps private

4. **Multi-Party Computation Models**: For advanced use cases, programs can use MPC techniques to compute over encrypted data

By carefully designing the Resource API and the capability model, we can achieve both rich composability and strong privacy guarantees.

## Key Architectural Implications

The proposed unified Resource API approach with different program account representations has several important implications:

1. **Protocol/Interface Separation**: Clear separation between the protocol (Resource API) and interfaces (program accounts)

2. **Upgradability**: User interfaces can evolve independently from on-chain logic

3. **Multiple Representations**: The same resources can have different representations for different contexts (public, private, client-side, on-chain)

4. **Capability-Based Security**: Security is based on unforgeable capabilities rather than identity

5. **Intent-Based Operations**: Operations express user intent rather than raw state mutations

This model provides a solid foundation for building a system that is both highly composable and privacy-preserving, while maintaining a clear separation between the user-facing experience and the underlying cryptographic mechanics.