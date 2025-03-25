<!-- Example applications -->
<!-- Original file: docs/src/example_applications.md -->

# Example Applications

This document describes the example applications provided with the Causality system to demonstrate its features and capabilities.

## Fact System Usage Example

**Location**: `examples/fact_system_usage.rs`

This example demonstrates how to use the new standardized fact system, including:

- Initializing domain adapters
- Creating fact queries for different fact types
- Processing fact results using pattern matching
- Working with register facts (creation, update, transfer)

### Key Components

1. **Adapter Initialization**:
   - Shows how to create and configure an Ethereum adapter
   - Demonstrates proper error handling when setting up adapters

2. **Balance Fact Queries**:
   - Creates a fact query for an Ethereum address balance
   - Processes the resulting `BalanceFact` using pattern matching
   - Extracts address, amount, timestamp, and block height information

3. **Block Fact Queries**:
   - Creates a fact query for a specific block height
   - Processes the resulting `BlockFact` using pattern matching
   - Extracts height, hash, timestamp, and parent hash information

4. **Transaction Fact Queries**:
   - Creates a fact query for a specific transaction hash
   - Processes the resulting `TransactionFact` using pattern matching
   - Extracts transaction details like from/to addresses, value, status, etc.

5. **Register Fact Operations**:
   - Demonstrates a complete register lifecycle with:
     - Register creation
     - Register value update
     - Register ownership transfer
   - Shows how to process different `RegisterFact` variants

### Running the Example

To run the fact system usage example:

```bash
cargo run --example fact_system_usage
```

Note: The example uses a placeholder Infura API key. Replace `YOUR_INFURA_KEY` with a valid key to connect to the Ethereum network for real-world testing.

### Example Output

When run successfully, the example will produce output showing the results of querying the different fact types:

```
Ethereum adapter initialized with domain ID: 0x01

Example 1: Querying a balance fact
Address: 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
Balance: 1234567890 wei
Timestamp: 2023-06-15T12:34:56Z
Block Height: 18000000

Example 2: Querying a block fact
Block Height: 18000000
Block Hash: 0xa1b2c3d4e5f6...
Timestamp: 2023-06-15T12:34:56Z
Parent Hash: 0xf6e5d4c3b2a1...

Example 3: Querying a transaction fact
Transaction Hash: 0x7d5cb4e46e5f213831288218e3f6734ed37a1a8e3b38c39a2d0c95d4cb8ceef6
From: 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
To: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
Value: 100000000
Status: Success
Block Height: 18000000

Example 4: Working with register facts
Step 1: Creating a register
Register test-register-1 created by 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Register type: token
Initial value: 0

Step 2: Updating the register
Register test-register-1 updated to 100
Previous value: 0
Updated by: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

Step 3: Transferring the register
Register test-register-1 transferred from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 to 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

## Future Examples

More examples are planned to demonstrate additional aspects of the Causality system:

1. **Effect System Example**: Demonstrates creating and executing effects with algebraic effect handlers
2. **Domain Adapter Integration**: Shows how to work with multiple domain adapters
3. **Resource Manager Example**: Illustrates resource management patterns for safe concurrent access
4. **Fact Replay Example**: Demonstrates the fact replay engine for deterministic testing 