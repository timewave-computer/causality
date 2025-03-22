# System Boundaries Documentation

This document describes the system boundaries in the Causality framework, including classifications, security models, and protocols for crossing these boundaries.

## Boundary Classifications

Causality defines several primary boundary types that any component or execution context must be classified under. These boundaries create clear security and execution domains.

### Primary Boundaries

#### Inside/Outside System

- **Inside System Boundary**: Represents code and data that executes within the Causality framework's trusted environment. This environment has access to all internal capabilities and is considered a higher trust level.
  - **Trust Level**: High
  - **Access Control**: Strict capability-based access
  - **Data Security**: Strong validation and sanitization of inputs

- **Outside System Boundary**: Represents code and data that executes outside the Causality framework's trusted environment. This could include external APIs, user-provided code, or other systems.
  - **Trust Level**: Low
  - **Access Control**: Limited to specific capabilities explicitly granted
  - **Data Security**: All data treated as untrusted and must be validated

#### On-Chain/Off-Chain

- **On-Chain Boundary**: Represents code and data that executes on a blockchain virtual machine (e.g., EVM, SVM, MoveVM).
  - **Trust Level**: Medium-High (depends on chain)
  - **Access Control**: Based on blockchain's native access control
  - **Data Security**: Inherits blockchain's security properties

- **Off-Chain Boundary**: Represents code and data that executes in traditional computing environments outside any blockchain.
  - **Trust Level**: Variable
  - **Access Control**: Capability-based with explicit permissions
  - **Data Security**: Depends on the specific environment

### Chain-Specific Boundaries

The Causality framework also recognizes specific virtual machine boundaries for different blockchains:

- **EVM Boundary**: Ethereum Virtual Machine execution environment
- **SVM Boundary**: Solana Virtual Machine execution environment
- **MoveVM Boundary**: Move Virtual Machine (used by Aptos, Sui, etc.) execution environment
- **CosmWasm Boundary**: CosmWasm (Cosmos SDK) execution environment
- **Local Boundary**: Local simulation of blockchain execution
- **Custom Boundary**: Custom execution environments

## Boundary Crossing Protocols

When data or control needs to cross a boundary, the Causality framework enforces strict protocols to maintain security.

### Authentication Methods

Crossing a boundary requires authentication using one or more of the following methods:

1. **Signature-Based**: Uses cryptographic signatures to verify identity
   - Required for: Outside→Inside, OffChain→OnChain crossings
   - Format: `BoundaryAuthentication::Signature(signature_string)`

2. **Capability-Based**: Uses capability tokens to verify permissions
   - Required for: Resource access, privileged operations
   - Format: `BoundaryAuthentication::Capability(capability_id)`

3. **ZK Proof**: Uses zero-knowledge proofs for private authentication
   - Required for: Privacy-preserving operations
   - Format: `BoundaryAuthentication::ZkProof(proof_bytes)`

4. **Multi-Factor**: Combines multiple authentication mechanisms
   - Required for: High-security operations
   - Format: `BoundaryAuthentication::MultiFactor(factors_vector)`

5. **None**: No authentication
   - Allowed only for: Public read-only operations
   - Format: `BoundaryAuthentication::None`

### Crossing Process

A boundary crossing follows these steps:

1. **Preparation**:
   - The source context prepares data for crossing by serializing and annotating it
   - A crossing payload is created with metadata, authentication, and the data
   - The appropriate protocol for the specific boundary types is selected

2. **Verification**:
   - Authentication is verified
   - Rate limiting checks are applied
   - Size limit checks are applied
   - The appropriate permissions are verified

3. **Execution**:
   - The data is deserialized in the target boundary
   - The requested operation is executed
   - Results are prepared for return crossing

4. **Return**:
   - Results are serialized and annotated
   - A crossing payload is created for the return trip
   - The return crossing follows the same verification process

### Security Controls

All boundary crossings implement the following security controls:

1. **Rate Limiting**: Prevents abuse by limiting crossing frequency
2. **Data Validation**: Ensures data is well-formed and meets schema requirements
3. **Size Limiting**: Prevents resource exhaustion attacks
4. **Audit Logging**: Records all crossings for security review
5. **Capability Checking**: Verifies that the crossing has the required capabilities

## Boundary Annotations

The Causality framework provides code annotations to explicitly mark boundary definitions and crossings:

```rust
// Mark a function that executes within a specific boundary
#[boundary("inside_system")]
fn example_inside_function() {
    // This function executes inside the system boundary
}

// Mark a function that crosses boundaries
#[boundary_crossing("inside_to_outside")]
fn example_crossing_function() {
    // This function crosses from inside to outside
}
```

Using the `BoundarySafe` trait:

```rust
impl BoundarySafe for MyType {
    fn target_boundary(&self) -> BoundaryType {
        BoundaryType::InsideSystem
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        // Serialize the type for crossing
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        // Deserialize after crossing
    }
}
```

## Security Model

### Trust Assumptions

Each boundary type has specific trust assumptions:

1. **Inside System**: Trusted code execution with access to sensitive resources
2. **Outside System**: Untrusted code execution with limited permissions
3. **On-Chain**: Trust is based on blockchain consensus and smart contract security
4. **Off-Chain**: Trust varies based on the specific component

### Threat Models

The boundary system is designed to protect against:

1. **Unauthorized Access**: Attempts to access resources without proper capabilities
2. **Data Exfiltration**: Unauthorized extraction of sensitive data
3. **Resource Exhaustion**: DoS attacks through excessive resource usage
4. **Code Injection**: Attempts to execute arbitrary code
5. **Replay Attacks**: Reusing valid boundary crossing requests

### Mitigation Strategies

1. **Capability-Based Security**: All resource access requires explicit capability tokens
2. **Zero-Knowledge Proofs**: Privacy-preserving authentication for sensitive operations
3. **Rate Limiting**: Protection against DoS attacks
4. **Authentication Requirements**: All boundary crossings require authentication
5. **Audit Trails**: All crossings are logged for security analysis

## Implementation Requirements

Components that cross boundaries must:

1. **Be Explicit**: All boundary crossings must be explicitly annotated
2. **Handle Errors**: All boundary crossing code must handle failure cases
3. **Use Authentication**: All crossings must include appropriate authentication
4. **Validate Data**: All data crossing boundaries must be validated
5. **Implement BoundarySafe**: Types crossing boundaries must implement this trait

## Boundary Visualization

```
┌─────────────────────────────────────────────────────────────┐
│                    CAUSALITY SYSTEM                          │
│  ┌─────────────────┐         ┌─────────────────────────┐    │
│  │                 │         │                         │    │
│  │  Inside System  │<───────>│  Resource Subsystem     │    │
│  │  Boundary       │         │  (Capability-Protected) │    │
│  │                 │         │                         │    │
│  └────────┬────────┘         └─────────────────────────┘    │
│           │                                                  │
│           │  Boundary Crossing (Authenticated)               │
│           ▼                                                  │
│  ┌─────────────────┐         ┌─────────────────────────┐    │
│  │                 │         │                         │    │
│  │  On-Chain       │<───────>│  Off-Chain              │    │
│  │  Boundary       │         │  Boundary               │    │
│  │                 │         │                         │    │
│  └────────┬────────┘         └──────────┬──────────────┘    │
└───────────┼──────────────────────────────┼─────────────────┘
            │                              │
            ▼                              ▼
   ┌─────────────────┐          ┌──────────────────┐
   │                 │          │                  │
   │  EVM / SVM /    │          │  External APIs   │
   │  Other VMs      │          │  Components      │
   │                 │          │                  │
   └─────────────────┘          └──────────────────┘
```

## Cross-Chain Boundary Considerations

When working with multiple blockchain environments:

1. **Chain Compatibility**: Not all data formats are compatible across chains
2. **Cross-Chain Messaging**: Requires special protocols (e.g., IBC, LayerZero)
3. **Finality Differences**: Different chains have different finality guarantees
4. **VM Compatibility**: Code execution models differ across VMs

## Best Practices

1. **Minimize Boundary Crossings**: Each crossing introduces overhead and security concerns
2. **Use Capability-Based Access**: Always use the minimum required capabilities
3. **Validate All External Data**: Never trust data crossing from a lower trust boundary
4. **Log All Crossings**: Maintain audit trails for security analysis
5. **Design for Failure**: Boundary crossings can fail, design systems to handle this

## Examples

### Inside-to-Outside Crossing Example

```rust
// Data that will cross the boundary
#[derive(Serialize, Deserialize)]
struct ApiRequest {
    operation: String,
    parameters: HashMap<String, Vec<u8>>,
}

impl BoundarySafe for ApiRequest {
    fn target_boundary(&self) -> BoundaryType {
        BoundaryType::OutsideSystem
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

// Function that crosses from inside to outside
#[boundary_crossing("inside_to_outside")]
async fn call_external_api(request: ApiRequest) -> Result<Vec<u8>, String> {
    // Create payload with capability-based authentication
    let auth = BoundaryAuthentication::Capability("api_access_cap".to_string());
    
    // Get the boundary system
    let system = boundary_system();
    
    // Find the appropriate protocol
    let protocol = system.crossing_registry()
        .find_protocol_for_boundaries(
            BoundaryType::InsideSystem, 
            BoundaryType::OutsideSystem
        )
        .ok_or_else(|| "Protocol not found".to_string())?;
    
    // Prepare the outgoing payload
    let payload = protocol.prepare_outgoing(&request, auth).await?;
    
    // Process the request
    let response = protocol.process_incoming(payload).await?;
    
    Ok(response)
}
```

### On-Chain Verification Example

```rust
// Function that verifies on-chain data
#[boundary_crossing("offchain_to_onchain")]
async fn verify_token_balance(user_id: &str) -> Result<u64, String> {
    // Get the boundary system
    let system = boundary_system();
    
    // Get the EVM adapter
    let evm_adapter = system.on_chain_adapter(OnChainEnvironment::EVM)
        .ok_or_else(|| "EVM adapter not found".to_string())?;
    
    // Create parameters
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), user_id.as_bytes().to_vec());
    
    // Call the contract method
    let response = evm_adapter.call_contract_method(
        ChainAddress::Ethereum("0x1234567890abcdef".to_string()),
        "balanceOf",
        params,
    ).await?;
    
    if response.success {
        let balance_bytes = &response.data;
        if balance_bytes.len() == 8 {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(balance_bytes);
            let balance = u64::from_be_bytes(bytes);
            Ok(balance)
        } else {
            Err("Invalid balance format".to_string())
        }
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
} 