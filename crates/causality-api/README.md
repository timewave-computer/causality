# Causality API

External interfaces and API gateways for the Causality Resource Model framework. This crate provides HTTP/gRPC APIs, blockchain integration, ZK coprocessor interaction, and FFI bindings for external system integration.

## Overview

The `causality-api` crate serves as the external interface layer for the Causality system, providing:

- **HTTP/gRPC APIs**: RESTful and streaming interfaces for Resource operations
- **Blockchain Integration**: Direct integration with Ethereum, Neutron, and other chains
- **ZK Coprocessor Interface**: Communication with Valence ZK coprocessors
- **FFI Bindings**: Foreign function interfaces for language interoperability
- **Intent Management**: High-level intent submission and tracking
- **API Gateway**: Unified access point for all external interactions

All APIs maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### HTTP API Server

RESTful API for Resource operations:

```rust
use causality_api::http::{ApiServer, ApiConfig};

let config = ApiConfig {
    bind_address: "0.0.0.0:8080".to_string(),
    cors_enabled: true,
    rate_limiting: true,
};

let server = ApiServer::new(config).await?;
server.start().await?;
```

### gRPC Services

High-performance streaming APIs:

```rust
use causality_api::grpc::{ResourceService, IntentService};

// Resource management service
let resource_service = ResourceService::new(resource_manager);

// Intent submission service  
let intent_service = IntentService::new(intent_processor);
```

### Blockchain Integration

Direct blockchain interaction capabilities:

```rust
use causality_api::blockchain::{EthereumClient, NeutronClient};

// Ethereum integration
let eth_client = EthereumClient::new(rpc_url, private_key).await?;
let tx_hash = eth_client.submit_intent(&intent).await?;

// Neutron integration
let neutron_client = NeutronClient::new(grpc_endpoint).await?;
let result = neutron_client.execute_intent(&intent).await?;
```

### ZK Coprocessor Interface

Integration with Valence ZK coprocessors:

```rust
use causality_api::zk_coprocessor::{CoprocessorClient, ProofRequest};

let client = CoprocessorClient::new("https://coprocessor.valence.xyz").await?;
let proof_request = ProofRequest::new(circuit_id, witness_data);
let proof = client.generate_proof(proof_request).await?;
```

### Intent Management

High-level intent operations:

```rust
use causality_api::intent::{IntentManager, IntentStatus};

let intent_manager = IntentManager::new(blockchain_clients);

// Submit intent
let intent_id = intent_manager.submit_intent(intent).await?;

// Track status
let status = intent_manager.get_status(&intent_id).await?;
```

## API Endpoints

### Resource Operations

- `GET /resources/{id}` - Retrieve Resource by ID
- `POST /resources` - Create new Resource
- `PUT /resources/{id}` - Update Resource state
- `DELETE /resources/{id}` - Nullify Resource

### Intent Operations

- `POST /intents` - Submit new intent
- `GET /intents/{id}` - Get intent status
- `GET /intents` - List intents with filters
- `DELETE /intents/{id}` - Cancel pending intent

### ZK Proof Operations

- `POST /proofs/generate` - Generate ZK proof
- `GET /proofs/{id}` - Get proof status
- `POST /proofs/verify` - Verify proof

### Domain Operations

- `GET /domains` - List available domains
- `GET /domains/{id}/resources` - List domain resources
- `POST /domains/{id}/intents` - Submit domain-specific intent

## FFI Bindings

### C/C++ Interface

```c
// C header definitions
typedef struct CausalityResource CausalityResource;
typedef struct CausalityIntent CausalityIntent;

// Resource operations
CausalityResource* causality_create_resource(const char* data);
int causality_validate_resource(CausalityResource* resource);
void causality_free_resource(CausalityResource* resource);

// Intent operations
CausalityIntent* causality_create_intent(const char* intent_data);
int causality_submit_intent(CausalityIntent* intent);
```

### Python Bindings

```python
import causality_api

# Create resource
resource = causality_api.create_resource(data)

# Submit intent
intent = causality_api.Intent(type="transfer", params=params)
intent_id = causality_api.submit_intent(intent)

# Check status
status = causality_api.get_intent_status(intent_id)
```

### JavaScript/WASM Bindings

```javascript
import { CausalityAPI } from 'causality-api';

const api = new CausalityAPI();

// Create resource
const resource = await api.createResource(data);

// Submit intent
const intentId = await api.submitIntent(intent);
```

## Configuration

API configuration through environment variables or config file:

```toml
[api]
bind_address = "0.0.0.0:8080"
cors_enabled = true
rate_limit_requests_per_minute = 1000

[blockchain.ethereum]
rpc_url = "https://eth-mainnet.alchemyapi.io/v2/API_KEY"
chain_id = 1

[blockchain.neutron]
grpc_endpoint = "https://grpc.neutron.org:443"
chain_id = "neutron-1"

[zk_coprocessor]
endpoint = "https://coprocessor.valence.xyz"
timeout_seconds = 30
```

## Error Handling

Comprehensive error responses with proper HTTP status codes:

```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Resource with ID res_123 not found",
    "details": {
      "resource_id": "res_123",
      "domain_id": "domain_456"
    }
  }
}
```

## Security

- **Authentication**: JWT-based authentication for API access
- **Authorization**: Role-based access control for operations
- **Rate Limiting**: Configurable rate limits per endpoint
- **Input Validation**: Comprehensive validation of all inputs
- **CORS**: Configurable cross-origin resource sharing

## Feature Flags

- **default**: Standard API features
- **http**: HTTP server support
- **grpc**: gRPC server support
- **blockchain**: Blockchain integration
- **zk-coprocessor**: ZK coprocessor integration
- **ffi**: Foreign function interface bindings

This crate provides comprehensive external interfaces for the Causality system, enabling integration with existing infrastructure while maintaining the verifiable and deterministic properties of the Resource Model.
