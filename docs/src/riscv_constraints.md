# ZK-VM Constraints and Patterns for Causality

This document outlines the constraints and patterns for generating Rust code targeting established zero-knowledge virtual machines (Risc0 and Succinct) in the Causality system.

## 1. Introduction

Causality now leverages established ZK-VMs like Risc0 and Succinct rather than maintaining a custom RISC-V compiler. This approach offers significant advantages but requires adherence to specific constraints to ensure efficient proof generation and verification.

## 2. ZK-VM Selection and Integration

### 2.1 Supported ZK-VMs

The following ZK-VMs are fully supported:

- **Risc0**: A RISC-V based ZK-VM with strong tooling and performance characteristics
- **Succinct**: A high-performance ZK-VM with advanced proving technology

### 2.2 Backend Selection Criteria

Choose the appropriate backend based on:

- **Workload complexity**: Simpler operations may be more efficient on one backend
- **Proof size requirements**: Different backends produce different proof sizes
- **Verification cost**: On-chain verification costs vary by backend
- **Memory requirements**: Maximum guest program memory varies by backend
- **Proving time constraints**: Different backends have different proving time characteristics

## 3. Rust Code Generation Constraints

### 3.1 Guest Program Structure

ZK-VM guest programs should follow this structure:

```rust
// Required imports
use risc0_zkvm::guest::env;

// Entry point for Risc0 (Example)
#[no_mangle]
pub extern "C" fn main() {
    // Read inputs
    let input: MyInput = env::read();
    
    // Process the input
    let result = process_input(input);
    
    // Commit the result
    env::commit(&result);
}

// Input data structure
#[derive(Debug, Serialize, Deserialize)]
struct MyInput {
    // Input fields
}

// Result data structure
#[derive(Debug, Serialize, Deserialize)]
struct MyResult {
    // Result fields
}
```

### 3.2 Memory Constraints

- Limited heap memory (typically 16-32MB depending on backend)
- No access to the filesystem or network
- Stack usage should be minimized to prevent overflow
- Data structures should be sized appropriately for the available memory

### 3.3 Computation Constraints

- Execution must be deterministic
- Avoid unbounded loops that could exceed ZK-VM cycle limits
- Use fixed-size arrays instead of growing collections where possible
- Prefer static allocation over dynamic allocation

### 3.4 External Dependencies

- Only use dependencies that are compatible with the ZK-VM environment
- Avoid dependencies with platform-specific code
- Use the `#[no_std]` compatible versions of libraries when available
- Minimize the use of external libraries to reduce circuit size

## 4. Rust Code Patterns for ZK-VMs

### 4.1 Effect Implementation Pattern

```rust
// Effect implementation pattern
#[method]
pub fn implement_effect(
    params: EffectParams,
) -> Result<EffectResult, Error> {
    // 1. Validate inputs
    validate_inputs(&params)?;
    
    // 2. Load necessary state
    let state = load_state(params.context_id)?;
    
    // 3. Apply the effect
    let new_state = apply_effect(state, params)?;
    
    // 4. Create result
    let result = create_result(&new_state, params)?;
    
    // 5. Commit changes
    commit_state(params.context_id, &new_state)?;
    
    // 6. Return result
    Ok(result)
}
```

### 4.2 Register Operation Pattern

```rust
// Register operation pattern
#[method]
pub fn register_operation(
    operation: RegisterOp,
    register: Register,
    auth: Authorization,
) -> Result<RegisterReceipt, Error> {
    // 1. Verify authorization
    verify_authorization(&register, &auth)?;
    
    // 2. Apply operation
    let new_register = match operation {
        RegisterOp::Create => create_register(register)?,
        RegisterOp::Update => update_register(register)?,
        RegisterOp::Transfer => transfer_register(register)?,
        RegisterOp::Delete => delete_register(register)?,
    };
    
    // 3. Generate nullifier if needed
    let nullifier = generate_nullifier(&register)?;
    
    // 4. Create receipt
    let receipt = RegisterReceipt::new(
        operation,
        register.id,
        new_register.id,
        nullifier,
    );
    
    // 5. Return receipt
    Ok(receipt)
}
```

### 4.3 State Commitments

```rust
// State commitment pattern
fn commit_state<T: Serialize>(
    id: StateId,
    state: &T,
) -> Result<StateCommitment, Error> {
    // 1. Serialize state
    let serialized = serialize(state)?;
    
    // 2. Hash the serialized state
    let hash = hash(&serialized);
    
    // 3. Create commitment
    let commitment = StateCommitment::new(id, hash);
    
    // 4. Write commitment to journal
    env::commit(&commitment);
    
    Ok(commitment)
}
```

## 5. Guest/Host Communication

### 5.1 Input Protocol

```rust
// Reading inputs
fn read_inputs<T: for<'a> Deserialize<'a>>() -> Result<T, Error> {
    let input_bytes = env::read_bytes();
    let input: T = deserialize(&input_bytes)?;
    Ok(input)
}
```

### 5.2 Output Protocol

```rust
// Writing outputs
fn write_output<T: Serialize>(output: &T) -> Result<(), Error> {
    let output_bytes = serialize(output)?;
    env::commit(&output_bytes);
    Ok(())
}
```

### 5.3 Private/Public Inputs

```rust
// Handling private and public inputs
struct Inputs {
    // Public inputs (included in the proof verification)
    pub public_inputs: PublicInputs,
    
    // Private inputs (used for computation but not exposed)
    pub private_inputs: PrivateInputs,
}
```

## 6. Verification Integration

### 6.1 Proof Structure

```rust
// Proof structure
struct ZkProof {
    // Backend identifier
    backend: ZkBackend,
    
    // Public inputs (committed in the proof)
    public_inputs: Vec<u8>,
    
    // The proof data
    proof_data: Vec<u8>,
    
    // Optional metadata
    metadata: Option<HashMap<String, String>>,
}
```

### 6.2 Verification Contract Integration

```solidity
// Ethereum verification contract pattern
contract ZkVmVerifier {
    // Verify a proof
    function verifyProof(
        bytes32 programId,
        bytes32[] calldata publicInputs,
        bytes calldata proof
    ) public view returns (bool) {
        // Backend-specific verification
        return backendVerify(programId, publicInputs, proof);
    }
}
```

## 7. Performance Optimization Patterns

### 7.1 Minimize State Size

- Keep state representations compact
- Use bit packing where appropriate
- Use minimal-sized types (u8 instead of u64 when sufficient)

### 7.2 Minimize Cryptographic Operations

- Batch cryptographic operations where possible
- Reuse cryptographic intermediates when appropriate
- Choose cryptographic primitives compatible with the ZK-VM

### 7.3 Efficient Data Structures

- Use flat arrays instead of nested structures when possible
- Prefer stack allocation for small, fixed-size structures
- Minimize dynamic memory allocation

### 7.4 Computation/Memory Tradeoffs

- Cache results of expensive computations
- Use lookup tables for complex functions with small input domains
- Unroll small loops for better circuit generation

## 8. Example: Deposit Effect in Risc0

```rust
// Full example of a deposit effect in Risc0
use risc0_zkvm::guest::{env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DepositInput {
    account_id: u64,
    amount: u64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
struct Account {
    id: u64,
    balance: u64,
    last_updated: u64,
}

#[derive(Serialize, Deserialize)]
struct DepositReceipt {
    account_id: u64,
    previous_balance: u64,
    new_balance: u64,
    amount: u64,
    timestamp: u64,
}

#[no_mangle]
pub extern "C" fn main() {
    // Read the input
    let input: DepositInput = env::read();
    
    // Load account state (in real implementation, this would come from a state database)
    let mut account = Account {
        id: input.account_id,
        balance: 1000, // Example initial balance
        last_updated: 0,
    };
    
    // Record previous balance for the receipt
    let previous_balance = account.balance;
    
    // Apply the deposit
    account.balance += input.amount;
    account.last_updated = input.timestamp;
    
    // Create the receipt
    let receipt = DepositReceipt {
        account_id: input.account_id,
        previous_balance,
        new_balance: account.balance,
        amount: input.amount,
        timestamp: input.timestamp,
    };
    
    // Commit the updated account state
    env::commit(&account);
    
    // Commit the receipt as the main output
    env::commit(&receipt);
}
```

## 9. Migration Considerations

When migrating from the custom RISC-V implementation to Risc0/Succinct:

1. **Code Transformation**: Convert existing RISC-V code to equivalent Rust
2. **Memory Model**: Adapt to the ZK-VM's memory model
3. **Instruction Selection**: Utilize Rust's abstractions rather than manual instruction selection
4. **Guest/Host Boundary**: Add proper boundary handling for external data
5. **Proof Generation**: Update the proof generation flow for the chosen backend

## 10. Comparison of Custom RISC-V vs ZK-VM Backends

| Aspect | Custom RISC-V | Risc0 | Succinct |
|--------|---------------|-------|----------|
| Maintenance Burden | High | Low | Low |
| Proving Performance | Variable | Good | Excellent |
| Circuit Optimization | Manual | Automated | Automated |
| Tooling | Custom | Standard Rust | Standard Rust |
| Security | Requires Audit | Industry Tested | Industry Tested |
| Verification Cost | Variable | Moderate | Low |
| Development Complexity | High | Moderate | Moderate |

The transition to established ZK-VMs improves development velocity, security, and performance, while allowing the Causality system to focus on its core value propositions rather than ZK-VM implementation details. 