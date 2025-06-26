# Causality API

HTTP API and coprocessor integration gateway providing external interfaces for resource operations, session management, zero-knowledge proof coordination, and multi-chain blockchain interactions.

## Core Components

### HTTP API Server
RESTful interface for resource and session operations:

```rust
use causality_api::{CausalityApi, ApiConfig};

let api = CausalityApi::new(ApiConfig {
    bind_address: "0.0.0.0:8080".to_string(),
    enable_cors: true,
});
api.start().await?;
```

### Session Management
Stateful execution contexts with timeout handling:

```rust
use causality_api::{ExecutionSession, SessionConfig};

let session = ExecutionSession::new(SessionConfig {
    timeout: Duration::from_secs(300),
    max_operations: 1000,
});
```

### ZK Coprocessor Interface
Integration with zero-knowledge proving services:

```rust
use causality_api::{CoprocessorService, CoprocessorConfig};

let coprocessor = CoprocessorService::new(config)?;
let proof_id = coprocessor.submit_proof_request(circuit, witness).await?;
```

### Blockchain Client Interface
Multi-chain transaction submission and monitoring:

```rust
use causality_api::{ChainClient, ChainConfig, TransactionRequest};

let client = ChainClient::new(ChainConfig {
    chain: "ethereum".to_string(),
    rpc_url: "https://mainnet.infura.io/v3/YOUR_KEY".to_string(),
    gas_strategy: GasStrategy::Aggressive,
});

let tx_request = TransactionRequest {
    proof_data: proof_bytes,
    gas_limit: Some(500_000),
    max_fee_per_gas: Some(100_000_000_000), // 100 gwei
    max_priority_fee_per_gas: Some(2_000_000_000), // 2 gwei
};

let tx_hash = client.submit_transaction(tx_request).await?;
```

### Client Library
Type-safe client for API interaction:

```rust
use causality_api::{CausalityClient, ClientConfig};

let client = CausalityClient::new(config);
let resource_id = client.create_resource(resource_data).await?;
```

## REST API Endpoints

### Resource Operations
```http
GET    /api/v1/resources/{id}           # Retrieve resource
POST   /api/v1/resources               # Create resource  
PUT    /api/v1/resources/{id}          # Update resource
DELETE /api/v1/resources/{id}          # Consume resource
```

### Session Operations  
```http
POST   /api/v1/sessions                # Create session
GET    /api/v1/sessions/{id}           # Get session status
POST   /api/v1/sessions/{id}/execute   # Execute operation
DELETE /api/v1/sessions/{id}           # End session
```

### Blockchain Operations
```http
POST   /api/v1/transactions            # Submit transaction to blockchain
GET    /api/v1/transactions/{hash}     # Get transaction status
POST   /api/v1/transactions/estimate   # Estimate gas costs
GET    /api/v1/chains                  # List supported chains
GET    /api/v1/chains/{chain}/status   # Get chain status and gas prices
```

### System Operations
```http
GET    /api/v1/health                  # Health check
GET    /api/v1/metrics                 # System metrics
GET    /api/v1/version                 # API version info
```

## Blockchain Integration

### Supported Chains
- **Ethereum**: Mainnet, Goerli, Sepolia
- **Polygon**: Mainnet, Mumbai
- **Arbitrum**: Mainnet, Goerli
- **Optimism**: Mainnet, Goerli

### Transaction Types
- **ZK Proof Submission**: Submit zero-knowledge proofs with verification
- **Resource Operations**: Create, update, consume linear resources on-chain
- **Cross-Chain Coordination**: Coordinate operations across multiple chains

### Gas Management
- **Automatic Gas Estimation**: EIP-1559 compatible gas pricing
- **Gas Optimization**: Dynamic gas price adjustment based on network conditions
- **Priority Fee Management**: Configurable priority fees for transaction speed
- **Gas Limit Calculation**: Automatic gas limit estimation with safety margins

### Transaction Monitoring
- **Real-time Status**: WebSocket updates for transaction status
- **Confirmation Tracking**: Configurable confirmation requirements
- **Error Recovery**: Automatic retry with exponential backoff
- **Receipt Validation**: Full transaction receipt verification

## Key Features

- **Resource Management**: Create, read, update, consume linear resources
- **Session Handling**: Stateful execution with automatic cleanup
- **ZK Integration**: Async proof generation with status tracking
- **Multi-Chain Support**: Submit transactions to multiple blockchains simultaneously
- **Gas Optimization**: Intelligent gas pricing and limit management
- **Transaction Monitoring**: Real-time transaction status and confirmation tracking
- **Error Recovery**: Comprehensive error handling with retry logic
- **Protocol Translation**: Internal resource model to external format conversion

## Configuration

```toml
[api]
bind_address = "0.0.0.0:8080"
enable_cors = true

[session]  
default_timeout = "300s"
max_operations = 1000

[coprocessor]
endpoint = "https://coprocessor.valence.xyz"
timeout = "600s"

[blockchain]
# Default chain for operations
default_chain = "ethereum"

# Chain-specific configurations
[blockchain.ethereum]
rpc_url = "https://mainnet.infura.io/v3/YOUR_KEY"
chain_id = 1
gas_strategy = "aggressive"
max_fee_per_gas = 100_000_000_000  # 100 gwei
max_priority_fee_per_gas = 2_000_000_000  # 2 gwei
confirmation_blocks = 12

[blockchain.polygon]
rpc_url = "https://polygon-mainnet.infura.io/v3/YOUR_KEY"
chain_id = 137
gas_strategy = "standard"
max_fee_per_gas = 50_000_000_000  # 50 gwei
max_priority_fee_per_gas = 1_000_000_000  # 1 gwei
confirmation_blocks = 64

[blockchain.arbitrum]
rpc_url = "https://arbitrum-mainnet.infura.io/v3/YOUR_KEY"
chain_id = 42161
gas_strategy = "standard"
confirmation_blocks = 1

[blockchain.optimism]
rpc_url = "https://optimism-mainnet.infura.io/v3/YOUR_KEY"
chain_id = 10
gas_strategy = "standard"
confirmation_blocks = 1
```

## Usage Examples

### Submit ZK Proof to Multiple Chains
```rust
use causality_api::{ChainClient, TransactionRequest, ProofData};

// Create proof data
let proof_data = ProofData {
    proof: proof_bytes,
    public_inputs: public_inputs,
    verification_key: vk_bytes,
};

// Submit to Ethereum
let eth_client = ChainClient::ethereum(config.clone()).await?;
let eth_tx = eth_client.submit_proof_transaction(proof_data.clone()).await?;

// Submit to Polygon
let polygon_client = ChainClient::polygon(config.clone()).await?;
let polygon_tx = polygon_client.submit_proof_transaction(proof_data).await?;

// Monitor both transactions
let eth_receipt = eth_client.wait_for_confirmation(eth_tx, 12).await?;
let polygon_receipt = polygon_client.wait_for_confirmation(polygon_tx, 64).await?;
```

### Gas Price Optimization
```rust
use causality_api::{GasEstimator, GasStrategy};

let estimator = GasEstimator::new("ethereum").await?;

// Get current gas prices
let gas_prices = estimator.get_gas_prices().await?;
println!("Base fee: {} gwei", gas_prices.base_fee / 1_000_000_000);
println!("Priority fee: {} gwei", gas_prices.priority_fee / 1_000_000_000);

// Optimize for speed vs cost
let optimized = estimator.optimize_gas(GasStrategy::Fast).await?;
```

### Cross-Chain Resource Coordination
```rust
use causality_api::{ResourceCoordinator, ChainSet};

let coordinator = ResourceCoordinator::new(vec![
    ChainClient::ethereum(config.clone()).await?,
    ChainClient::polygon(config.clone()).await?,
    ChainClient::arbitrum(config.clone()).await?,
]).await?;

// Coordinate resource operations across chains
let resource_id = coordinator.create_cross_chain_resource(resource_spec).await?;
let operations = coordinator.execute_cross_chain_operations(resource_id, operations).await?;
```
