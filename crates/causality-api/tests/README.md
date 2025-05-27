# Causality API Tests

This directory contains tests for the Causality API crate, focusing on the chain client implementation and related components.

## Test Structure

The tests are organized by functionality:

- `chain_client_test.rs`: Tests the core chain client implementation using mocks, verifying transaction submission, balance queries, and intent handling.
- `cross_chain_test.rs`: Tests cross-chain functionality with simulated multi-chain transactions.

## Testing Strategy

The tests use the MockChainClient and MockIntentHandler implementations to verify behavior without external dependencies. Key aspects tested include:

1. **Chain Client Operations**: Basic operations like balance queries and transaction submissions.
2. **Intent Handling**: Full lifecycle of creating, submitting, querying, and updating intents.
3. **Multi-chain Scenarios**: Transactions that span multiple blockchains.

All tests maintain ZK compatibility by using bounded types and deterministic behaviors. This approach ensures that the API can be used in both normal runtime environments and ZK verification contexts.

## Running Tests

To run tests with Nix:

```bash
cd /Users/hxrts/projects/timewave/causality
nix develop
cargo test -p causality-api
```

Or run specific test files:

```bash
cargo test -p causality-api --test chain_client_test
```
