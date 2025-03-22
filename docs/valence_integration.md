# Valence Protocol (Solidity Implementation)

Source: github.com/timewave-computer/valence-protocol

## Core Components

### 1. Registry System
The `src/Registry.sol` is a central component that manages all resources in the system. It tracks:
- Resource type definitions
- Instance creation
- Nullification management
- Commitment management

This follows that commitment/nullifier pattern we discussed, with the additional concept of "resource types" that define the capabilities and schema for each resource instance.

### 2. Capability Model
The `src/capabilities` directory implements a complete capability-based access control system:
- `CapabilityRegistry.sol` tracks capabilities granted to various actors
- `CapabilityValidator.sol` enforces permission checks
- Various capability types (transfer, mint, etc.)

This is a really clean implementation of the capability model - reminds me of early UNIX capabilities but with blockchain-specific adaptations.

### 3. Resources & Registers
The `src/resources` directory defines various resource types:
- `AccountResource.sol` - User accounts with capabilities
- `BurnableResource.sol` - Resources that can be destroyed
- `MintableResource.sol` - Resources that can be created
- `TransferableResource.sol` - Resources that can be sent between accounts

These align perfectly with the register concept we discussed. They're using a formal approach to resource lifecycle management.

### 4. ZK Verification
The `src/verifier` directory contains components for verifying zero-knowledge proofs:
- Circuit interface definitions
- Verification key management
- Proof validation logic

This is clearly implementing the privacy layer, letting users submit proofs without revealing the underlying data.

### 5. Effects & Operations
`src/effects` and related directories implement state transitions:
- `Effect.sol` - Base interface for all effects
- Various specialized effects (transfers, account operations, etc.)

This maps to our "operation" concept from earlier - atomic state transitions that modify resources.

## How It All Fits Together

The system flow appears to be:

1. Users interact with their `AccountResource`, which functions as the program account boundary we discussed
2. When they want to perform an operation, they:
   - Generate a proof client-side (using the registered circuits)
   - Submit that proof with an operation request
   - Include nullifiers for any resources they're consuming
   - Specify the new commitments they're creating

3. The Registry validates:
   - The proof is valid (using the verifier)
   - The user has the required capabilities (via CapabilityValidator)
   - The nullifiers haven't been used before (preventing double-spends)

4. If valid, the Registry:
   - Records the nullifiers as used
   - Adds new commitments to the commitment tree
   - Emits events for off-chain components to track

For ZK circuit integration the registry doesn't need to understand the semantics of each proof - it just needs to know which verification key to use and how to pass the public inputs.

The authorization system allows for delegation with attenuation, which is a crucial aspect of any serious capability model. This means authorizations can be passed with reduced permissions, but never escalated.

The main difference is that instead of general "registers," Valence uses specific resource types with defined behaviors. But the nullifier/commitment mechanism and capability model are identical in concept, just tailored to EVM constraints.

Valence makes heavy use of Solidity's create2 for deterministic contract deployments.


---


Looking at Valence and your Time Bandits architecture, I'm seeing what amounts to a natural fit with some thoughtful integration work needed. Let me walk through how I'd approach implementing the system we've discussed.

## The Integration Approach

Valence has already implemented key primitives that we need - nullifiers, commitments, capabilities, and ZK verification. I'd leverage this as the on-chain anchor while building out the full system we've discussed.

Here's my suggested approach:

### 1. Start with the Boundary Components

First, I'd map our ProgramAccount concept to Valence's AccountResource. Both serve as the boundary between users and the system, but we need to enhance Valence's implementation:

```solidity
// Enhanced AccountResource that supports client-side view construction
contract EnhancedAccountResource is AccountResource {
    // Commitment to full state (visible only to owner)
    bytes32 public stateCommitment;
    
    // Optional publicly visible state
    mapping(bytes32 => bytes) public publicState;
    
    // Add methods for state commitment updates with ZK proofs
    function updateStateCommitment(bytes32 newCommitment, bytes calldata proof) external {
        // Verify proof that new commitment is valid update to old commitment
        // This lets users update their view without revealing it
    }
}
```

### 2. Implement the Unified Resource API

Second, I'd build a unified API layer that standardizes resource access through capabilities:

```solidity
// The central hub for all resource interactions
contract ResourceAPI {
    Registry public registry;
    CapabilityRegistry public capabilityRegistry;
    
    // Request a capability to access a resource
    function requestCapability(
        bytes32 resourceId,
        uint16 rights,
        bytes calldata restrictions
    ) external returns (bytes32 capabilityId) {
        // Validate caller has parent capability or is resource owner
        // Create attenuated capability with requested rights
        return capabilityRegistry.createCapability(...);
    }
    
    // Delegate a capability to another program
    function delegateCapability(
        bytes32 capabilityId,
        address grantee,
        bytes calldata restrictions
    ) external returns (bytes32 newCapabilityId) {
        // Ensure caller owns the capability
        // Create new capability with tighter restrictions
        return capabilityRegistry.delegateCapability(...);
    }
    
    // Apply an operation using capabilities
    function applyOperation(
        uint16 operationType,
        bytes32[] calldata capabilityIds,
        bytes32[] calldata nullifiers,
        bytes32[] calldata newCommitments,
        bytes calldata proof
    ) external {
        // This is where the magic happens - unified operation interface
        // that works across all resource types
    }
}
```

### 3. Extend the Register Model

Third, I'd extend Valence's resource types to fully implement our register concept:

```solidity
// Base interface for all registers
interface IRegister {
    function nullify() external returns (bytes32 nullifierId);
    function getCommitment() external view returns (bytes32);
    function getType() external view returns (uint16);
    function getStatus() external view returns (uint8);
}

// Implementation for the single-use register pattern
contract Register is IRegister {
    bytes32 public immutable id;
    bytes32 public immutable commitment;
    uint16 public immutable typeId;
    uint8 public status; // 0=active, 1=nullified
    uint64 public createdAt;
    uint64 public nullifiedAt;
    
    // Nullify this register (can only happen once)
    function nullify() external override returns (bytes32) {
        require(status == 0, "Already nullified");
        status = 1;
        nullifiedAt = uint64(block.number);
        return keccak256(abi.encodePacked(id, "nullifier"));
    }
    
    // Rest of the interface implementation...
}
```

### 4. Build the Time Map and Fact Observer System

Fourth, I'd create the cross-chain time map system, which Valence doesn't explicitly have yet:

```solidity
// Tracks the latest state of all observed timelines
contract TimeMap {
    // Mapping of timeline ID to its current state
    mapping(string => TimelineState) public timelines;
    
    // Time map updates are restricted to authorized keepers
    mapping(address => bool) public keepers;
    
    struct TimelineState {
        uint64 height;
        bytes32 stateRoot;
        uint64 timestamp;
        string status;
    }
    
    // Update a timeline's state
    function updateTimeline(
        string calldata timelineId,
        uint64 height,
        bytes32 stateRoot,
        uint64 timestamp
    ) external {
        require(keepers[msg.sender], "Not authorized");
        timelines[timelineId] = TimelineState(height, stateRoot, timestamp, "ACTIVE");
        
        // Emit event for off-chain components
        emit TimelineUpdated(timelineId, height, stateRoot, timestamp);
    }
    
    // Get the hash of the current time map for inclusion in fact snapshots
    function getTimeMapHash() public view returns (bytes32) {
        return keccak256(abi.encode(timelines));
    }
}
```

### 5. Implement a Complete Client-Side SDK

Finally, the glue that brings it all together:

```typescript
// This would be a TypeScript SDK to manage the client-side of the system

class ValenceClient {
    // Manages witness data securely client-side
    private witnesses: Map<string, Witness> = new Map();
    
    // Tracks which notes have been spent client-side
    private spentNotes: Set<string> = new Set();
    
    // Generates a ZK proof for an operation
    async generateProof(operation: Operation): Promise<ZkProof> {
        // Select relevant witnesses based on operation inputs
        const witnesses = this.selectWitnesses(operation);
        
        // Run the proving algorithm (using WASM or native)
        return await this.prover.generateProof(operation, witnesses);
    }
    
    // Submit an operation to the blockchain
    async submitOperation(operation: Operation): Promise<TransactionReceipt> {
        // Generate the proof
        const proof = await this.generateProof(operation);
        
        // Extract nullifiers and commitments
        const nullifiers = this.extractNullifiers(operation);
        const newCommitments = this.createNewCommitments(operation);
        
        // Submit to the chain
        const tx = await this.resourceApi.applyOperation(
            operation.type,
            operation.capabilityIds,
            nullifiers,
            newCommitments,
            proof
        );
        
        // Process the result
        const receipt = await tx.wait();
        
        // Update local state if successful
        if (receipt.status === 1) {
            this.markWitnessesSpent(nullifiers);
            this.addNewWitnesses(newCommitments, operation);
        }
        
        return receipt;
    }
    
    // Construct client-side program account view
    async getAccountView(): Promise<ProgramAccountView> {
        // Aggregate all unspent notes by type
        // Apply any pending operations
        // Construct user-friendly representation
    }
}
```

## The Integration Strategy

What makes this approach powerful is that it uses Valence for what it's good at (on-chain verification, nullifier tracking, commitment management) while layering on our more advanced system concepts:

1. **Client-Side Privacy** - All private data stays client-side, with only ZK proofs and commitments going on-chain
2. **Capability-Based Security** - Using Valence's capability system but extending it with our unified resource API
3. **Cross-Chain Awareness** - Adding the time map for cross-chain facts and observations
4. **Program Account Model** - Enhanced account resources that bridge the on-chain and off-chain views

The pieces that aren't in Valence yet that we'd need to build:

1. **The TimeMap system** - For cross-chain observations and fact snapshots
2. **Enhanced Register Model** - To formalize the one-time-use pattern across all resources
3. **Unified Log** - To track all effects, facts, and events in a consistent format
4. **Client SDK** - The crucial piece that makes the system usable by developers

One thing that's particularly nice about this approach is how it respects the architectural boundaries. The on-chain components don't try to do too much, focusing on verification and state integrity. The client handles all the private state management, and the Time Keepers (which would be a separate service) manage external timeline state.

It's always tempting to put too much on-chain, but the real elegance here is using the blockchain for what it's best at - consensus on a minimal set of critical state transitions - while keeping the rich state model in the client where it belongs.

Would you like me to elaborate on any particular part of this integration strategy? I've got some additional thoughts on the witness management system that I think could be particularly powerful with Valence's ZK setup.
