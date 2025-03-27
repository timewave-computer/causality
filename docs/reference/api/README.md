# API Reference

*This documentation provides reference information for the Causality API.*

*Last updated: 2023-05-15*

## Overview

The Causality API is organized into several categories, each providing specific functionality for working with the Causality system. This reference documentation provides detailed information about the available API endpoints, request and response formats, and examples of how to use them.

## API Categories

- **Resource API**: Endpoints for creating, reading, updating, and deleting resources
- **Capability API**: Endpoints for capability management and delegation
- **Effect API**: Endpoints for creating and executing effects
- **Domain API**: Endpoints for integrating with external blockchains and data sources
- **Time API**: Endpoints for causal and clock time operations
- **Identity API**: Endpoints for identity management and verification

## Authentication

All API requests require authentication using one of the following methods:

- **Bearer Token**: Token-based authentication using JWT
- **Capability Proof**: Cryptographic proof of capability ownership
- **Identity Signature**: Cryptographic signature proving identity

## API Versioning

The Causality API uses versioning to ensure backward compatibility. The current version is `v1`.

## Base URL

The base URL for all API endpoints is:

```
https://api.causality.dev/v1
```

## Resource API

The Resource API provides endpoints for managing resources within the Causality system.

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/resources` | GET | List resources |
| `/resources/{id}` | GET | Get a resource by ID |
| `/resources` | POST | Create a new resource |
| `/resources/{id}` | PUT | Update a resource |
| `/resources/{id}` | DELETE | Delete a resource |
| `/resources/query` | POST | Query resources |

## Capability API

The Capability API provides endpoints for managing capabilities within the Causality system.

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/capabilities` | GET | List capabilities |
| `/capabilities/{id}` | GET | Get a capability by ID |
| `/capabilities` | POST | Create a new capability |
| `/capabilities/{id}/delegate` | POST | Delegate a capability |
| `/capabilities/{id}/verify` | POST | Verify a capability |
| `/capabilities/{id}` | DELETE | Revoke a capability |

## Effect API

The Effect API provides endpoints for creating and executing effects within the Causality system.

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/effects` | POST | Execute an effect |
| `/effects/batch` | POST | Execute multiple effects |
| `/effects/{id}` | GET | Get effect status |
| `/effects/query` | POST | Query effects |
| `/effects/types` | GET | List available effect types |

## Status Codes

The Causality API uses standard HTTP status codes to indicate success or failure of an API request:

| Status Code | Description |
|-------------|-------------|
| 200 | OK - Request succeeded |
| 201 | Created - Resource created successfully |
| 400 | Bad Request - Invalid request format or parameters |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists or state conflict |
| 500 | Internal Server Error - Server-side error |

## Error Handling

All error responses follow a standard format:

```json
{
  "error": {
    "code": "error_code",
    "message": "Human-readable error message",
    "details": {
      "field": "Additional error information"
    }
  }
}
```

## Future Documentation

Detailed documentation for each API endpoint, including request and response formats, will be added in future updates. 