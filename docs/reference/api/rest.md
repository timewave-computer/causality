# Causality REST API Reference

This document provides a comprehensive reference for the Causality REST API.

## Overview

The Causality REST API provides HTTP endpoints for interacting with the Causality system, allowing clients to:

- Create and manage resources
- Execute effects
- Query facts and state
- Manage programs and accounts
- Monitor and trace execution

## Base URL

All API endpoints are relative to the base URL of your Causality deployment:

```
https://<causality-host>:<port>/api/v1
```

## Authentication

The API uses capability-based authentication. Each request must include an authorization header with a capability token:

```
Authorization: Bearer <capability-token>
```

Capability tokens encode the specific operations they authorize and can be restricted to specific resources, operations, and time windows.

## Resource Endpoints

### Create Resource

```
POST /resources
```

Creates a new resource in the system.

**Request Body**:
```json
{
  "resourceLogic": "0x...",
  "fungibilityDomain": 1,
  "quantity": 100,
  "metadata": { 
    "name": "Example Resource",
    "description": "An example resource"
  }
}
```

**Response**:
```json
{
  "id": "0x...",
  "resourceLogic": "0x...",
  "fungibilityDomain": 1,
  "quantity": 100,
  "state": 1,
  "metadata": { 
    "name": "Example Resource",
    "description": "An example resource"
  },
  "contentHash": "0x..."
}
```

### Get Resource

```
GET /resources/{resourceId}
```

Retrieves a resource by its ID.

**Response**:
```json
{
  "id": "0x...",
  "resourceLogic": "0x...",
  "fungibilityDomain": 1,
  "quantity": 100,
  "state": 1,
  "metadata": { 
    "name": "Example Resource",
    "description": "An example resource"
  },
  "contentHash": "0x..."
}
```

### Update Resource

```
PUT /resources/{resourceId}
```

Updates an existing resource.

**Request Body**:
```json
{
  "state": 2,
  "metadata": { 
    "name": "Updated Resource Name",
    "description": "Updated description"
  }
}
```

**Response**:
```json
{
  "id": "0x...",
  "resourceLogic": "0x...",
  "fungibilityDomain": 1,
  "quantity": 100,
  "state": 2,
  "metadata": { 
    "name": "Updated Resource Name",
    "description": "Updated description"
  },
  "contentHash": "0x..."
}
```

### List Resources

```
GET /resources?state=1&limit=10&offset=0
```

Lists resources, with optional filtering.

**Query Parameters**:
- `state`: Filter by resource state
- `resourceLogic`: Filter by resource logic
- `limit`: Maximum number of results to return
- `offset`: Number of results to skip

**Response**:
```json
{
  "resources": [
    {
      "id": "0x...",
      "resourceLogic": "0x...",
      "fungibilityDomain": 1,
      "quantity": 100,
      "state": 1,
      "metadata": { 
        "name": "Resource 1",
        "description": "Description 1"
      },
      "contentHash": "0x..."
    },
    {
      "id": "0x...",
      "resourceLogic": "0x...",
      "fungibilityDomain": 1,
      "quantity": 50,
      "state": 1,
      "metadata": { 
        "name": "Resource 2",
        "description": "Description 2"
      },
      "contentHash": "0x..."
    }
  ],
  "pagination": {
    "total": 100,
    "limit": 10,
    "offset": 0,
    "nextOffset": 10
  }
}
```

## Effect Endpoints

### Execute Effect

```
POST /effects
```

Executes an effect in the system.

**Request Body**:
```json
{
  "effectType": "transfer",
  "parameters": {
    "fromResourceId": "0x...",
    "toResourceId": "0x...",
    "amount": 50
  }
}
```

**Response**:
```json
{
  "id": "0x...",
  "effectType": "transfer",
  "status": "completed",
  "result": {
    "success": true,
    "fromResourceId": "0x...",
    "toResourceId": "0x...",
    "amount": 50
  },
  "timestamp": "2023-03-01T12:34:56Z",
  "contentHash": "0x..."
}
```

### Get Effect Status

```
GET /effects/{effectId}
```

Gets the status of a previously executed effect.

**Response**:
```json
{
  "id": "0x...",
  "effectType": "transfer",
  "status": "completed",
  "result": {
    "success": true,
    "fromResourceId": "0x...",
    "toResourceId": "0x...",
    "amount": 50
  },
  "timestamp": "2023-03-01T12:34:56Z",
  "contentHash": "0x..."
}
```

### List Effects

```
GET /effects?resourceId=0x...&limit=10&offset=0
```

Lists effects, with optional filtering.

**Query Parameters**:
- `resourceId`: Filter by related resource ID
- `effectType`: Filter by effect type
- `status`: Filter by effect status
- `limit`: Maximum number of results to return
- `offset`: Number of results to skip

**Response**:
```json
{
  "effects": [
    {
      "id": "0x...",
      "effectType": "transfer",
      "status": "completed",
      "timestamp": "2023-03-01T12:34:56Z",
      "contentHash": "0x..."
    },
    {
      "id": "0x...",
      "effectType": "consume",
      "status": "completed",
      "timestamp": "2023-03-01T12:30:00Z",
      "contentHash": "0x..."
    }
  ],
  "pagination": {
    "total": 45,
    "limit": 10,
    "offset": 0,
    "nextOffset": 10
  }
}
```

## Fact Endpoints

### Get Fact

```
GET /facts/{factId}
```

Retrieves a fact by its ID.

**Response**:
```json
{
  "id": "0x...",
  "factType": "balance",
  "domainId": "ethereum:mainnet",
  "parameters": {
    "account": "0x...",
    "asset": "ETH"
  },
  "value": "1000000000000000000",
  "observedAt": "2023-03-01T12:00:00Z",
  "contentHash": "0x..."
}
```

### Query Facts

```
POST /facts/query
```

Queries for facts based on criteria.

**Request Body**:
```json
{
  "factType": "balance",
  "domainId": "ethereum:mainnet",
  "parameters": {
    "account": "0x..."
  }
}
```

**Response**:
```json
{
  "facts": [
    {
      "id": "0x...",
      "factType": "balance",
      "domainId": "ethereum:mainnet",
      "parameters": {
        "account": "0x...",
        "asset": "ETH"
      },
      "value": "1000000000000000000",
      "observedAt": "2023-03-01T12:00:00Z",
      "contentHash": "0x..."
    },
    {
      "id": "0x...",
      "factType": "balance",
      "domainId": "ethereum:mainnet",
      "parameters": {
        "account": "0x...",
        "asset": "DAI"
      },
      "value": "5000000000000000000",
      "observedAt": "2023-03-01T12:00:00Z",
      "contentHash": "0x..."
    }
  ]
}
```

## Program Endpoints

### Get Program

```
GET /programs/{programId}
```

Retrieves a program by its ID.

**Response**:
```json
{
  "id": "0x...",
  "name": "Example Program",
  "schema": {
    "version": "1.0.0",
    "fields": [
      {
        "name": "counter",
        "type": "uint64",
        "defaultValue": 0
      }
    ]
  },
  "owner": "0x...",
  "state": {
    "counter": 42
  },
  "contentHash": "0x..."
}
```

### Create Program

```
POST /programs
```

Creates a new program.

**Request Body**:
```json
{
  "name": "Example Program",
  "schema": {
    "version": "1.0.0",
    "fields": [
      {
        "name": "counter",
        "type": "uint64",
        "defaultValue": 0
      }
    ]
  },
  "initialState": {
    "counter": 0
  }
}
```

**Response**:
```json
{
  "id": "0x...",
  "name": "Example Program",
  "schema": {
    "version": "1.0.0",
    "fields": [
      {
        "name": "counter",
        "type": "uint64",
        "defaultValue": 0
      }
    ]
  },
  "owner": "0x...",
  "state": {
    "counter": 0
  },
  "contentHash": "0x..."
}
```

## Error Responses

All API errors follow a standard format:

```json
{
  "error": {
    "code": "resource_not_found",
    "message": "Resource with ID 0x... not found",
    "details": {
      "resourceId": "0x..."
    }
  }
}
```

Common error codes include:

- `unauthorized`: The request lacks valid authentication
- `forbidden`: The authenticated user lacks permission for the requested operation
- `resource_not_found`: The requested resource does not exist
- `invalid_request`: The request is malformed or contains invalid data
- `internal_error`: An internal server error occurred

## Rate Limiting

The API implements rate limiting to prevent abuse. Rate limit headers are included in all responses:

```
X-Rate-Limit-Limit: 100
X-Rate-Limit-Remaining: 99
X-Rate-Limit-Reset: 1614556800
```

If you exceed the rate limit, you'll receive a 429 Too Many Requests response.

## Pagination

List endpoints support pagination using the `limit` and `offset` query parameters. The response includes a pagination object with metadata about the total number of results and next offset.

## Versioning

The API uses versioning in the URL path (`/api/v1/`) to ensure backward compatibility. When breaking changes are introduced, a new version will be created.

## Websocket Support

For real-time updates, a WebSocket API is also available at:

```
wss://<causality-host>:<port>/api/v1/ws
```

Documentation for the WebSocket API is available in a separate document.

## Further Resources

- [API Changelog](changelog.md)
- [Examples](examples.md)
- [Client Libraries](libraries.md) 