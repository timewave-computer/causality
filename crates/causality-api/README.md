# Causality API

External interface layer for the Causality framework that provides HTTP APIs and ZK coprocessor interaction for seamless integration with external systems while maintaining the verifiable properties of the resource model.

## Purpose

The `causality-api` crate serves as the **external integration gateway** for the Causality system, providing standardized interfaces that enable external systems to interact with the three-layer architecture. It bridges the gap between the internal resource model and external infrastructure including coprocessors and traditional applications.

### Key Responsibilities

- **HTTP API Gateway**: Provide RESTful interfaces for resource operations
- **ZK Coprocessor Interface**: Facilitate communication with zero-knowledge coprocessors
- **Session Management**: Handle execution sessions and state management
- **Protocol Translation**: Convert between internal resource representations and external formats

## Architecture Overview

The API layer is designed around several integration patterns:

### HTTP REST Interface
Standard RESTful APIs for resource operations:
- **Resource Management**: Create, read, update, and consume resources
- **Session Handling**: Manage execution sessions and state
- **System Operations**: Health checks and system monitoring

### Coprocessor Integration
Specialized interfaces for computational coprocessors:
- **ZK Proof Coordination**: Manage zero-knowledge proof generation workflows
- **Computation Offloading**: Delegate intensive computations to coprocessors
- **Result Verification**: Verify coprocessor computation results

## Core Components

### HTTP API Server (`server.rs`)

RESTful API server for resource and session operations:

```rust
use causality_api::{CausalityApi, ApiConfig};

let config = ApiConfig {
    bind_address: "0.0.0.0:8080".to_string(),
    enable_cors: true,
};

let api = CausalityApi::new(config);
api.start().await?;
```

**API Endpoints:**
- **Resource Operations**: Create, read, update, and consume resources
- **Session Management**: Create and manage execution sessions
- **System Monitoring**: Health checks and system metrics

### Session Management (`session.rs`)

Execution session handling for stateful operations:

```rust
use causality_api::{ExecutionSession, SessionConfig};

// Create execution session
let session = ExecutionSession::new(SessionConfig {
    timeout: Duration::from_secs(300),
    max_operations: 1000,
});

// Execute operations within session
let result = session.execute_operation(operation).await?;
```

**Session Features:**
- **State Management**: Maintain execution state across operations
- **Timeout Handling**: Automatic session cleanup and timeout management
- **Operation Tracking**: Track operations and resource usage

### ZK Coprocessor Interface (`coprocessor.rs`)

Integration with zero-knowledge coprocessors:

```rust
use causality_api::{CoprocessorService, CoprocessorConfig};

let coprocessor = CoprocessorService::new(CoprocessorConfig {
    endpoint: "https://coprocessor.valence.xyz".to_string(),
    timeout: Duration::from_secs(600),
})?;

// Submit proof generation request
let proof_id = coprocessor.submit_proof_request(circuit, witness).await?;

// Check proof status
let status = coprocessor.get_proof_status(proof_id).await?;
```

**Coprocessor Features:**
- **Async Proof Generation**: Non-blocking proof generation with status tracking
- **Error Recovery**: Robust error handling and retry mechanisms
- **Result Caching**: Intelligent caching of proof results

### Client Interface (`client.rs`)

Client library for interacting with the API:

```rust
use causality_api::{CausalityClient, ClientConfig};

let client = CausalityClient::new(ClientConfig {
    base_url: "http://localhost:8080".to_string(),
    timeout: Duration::from_secs(30),
});

// Create resource
let resource_id = client.create_resource(resource_data).await?;

// Read resource
let resource = client.get_resource(resource_id).await?;
```

## Configuration Management

### API Configuration

```toml
[api]
bind_address = "0.0.0.0:8080"
enable_cors = true
max_request_size = "10MB"

[session]
default_timeout = "300s"
max_operations = 1000
cleanup_interval = "60s"

[coprocessor]
endpoint = "https://coprocessor.valence.xyz"
timeout = "600s"
retry_attempts = 3
```

## REST API Specification

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
DELETE /api/v1/sessions/{id}           # End session
POST   /api/v1/sessions/{id}/execute   # Execute operation
```

### System Operations
```http
GET    /api/v1/health                  # Health check
GET    /api/v1/metrics                 # System metrics
GET    /api/v1/version                 # API version
```

## Error Handling and Response Formats

### Standardized Error Responses

```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Resource with ID res_123 not found",
    "details": {
      "resource_id": "res_123",
      "timestamp": "2024-01-15T10:30:00Z"
    }
  }
}
```

**Error Categories:**
- **Client Errors (4xx)**: Invalid requests, authentication failures
- **Server Errors (5xx)**: Internal errors, external service failures
- **Coprocessor Errors**: Proof generation and verification errors

## Design Philosophy

### Simplicity First
The API layer prioritizes simplicity and ease of use:
- **RESTful Design**: Follow standard REST conventions
- **Clear Error Messages**: Provide actionable error information
- **Minimal Configuration**: Sensible defaults with simple configuration

### Extensibility
Built for future expansion:
- **Modular Design**: Easy addition of new endpoints and features
- **Protocol Agnostic**: Design allows for additional protocols
- **Backend Independence**: Abstract over different backend systems

### Reliability
Designed for production use:
- **Error Recovery**: Comprehensive error handling and recovery
- **Timeout Management**: Configurable timeouts with graceful handling
- **Resource Cleanup**: Automatic cleanup of resources and sessions

## Testing Framework

Comprehensive testing across all API components:

```rust
#[tokio::test]
async fn test_resource_lifecycle() {
    let api = setup_test_api().await;
    
    // Create resource
    let resource_id = api.create_resource(test_resource_data()).await?;
    
    // Read resource
    let resource = api.get_resource(resource_id).await?;
    assert_eq!(resource.data, test_resource_data());
    
    // Consume resource
    api.consume_resource(resource_id).await?;
    
    // Verify resource is consumed
    assert!(api.get_resource(resource_id).await.is_err());
}

#[tokio::test]
async fn test_coprocessor_integration() {
    let coprocessor = setup_test_coprocessor().await;
    
    let proof_id = coprocessor.submit_proof_request(test_circuit(), test_witness()).await?;
    
    // Wait for proof completion
    loop {
        let status = coprocessor.get_proof_status(proof_id).await?;
        if status.is_complete() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let proof = coprocessor.get_proof(proof_id).await?;
    assert!(proof.verify(&public_inputs));
}
```

This comprehensive API layer enables seamless integration between the Causality system and external infrastructure while maintaining the verifiable and deterministic properties of the resource model.
