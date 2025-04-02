# Causality CosmWasm Domain Adapter

This crate provides the domain adapter implementation for interacting with blockchains that utilize the CosmWasm smart contract framework (typically Cosmos SDK based chains) within the Causality system.

## Purpose

Implements the domain adapter traits defined potentially in `causality-domain` or `causality-core` specifically for CosmWasm environments. It handles the translation between Causality effects/operations and CosmWasm concepts like contract instantiation, execution, queries, and events.

Responsibilities include:

- **Connecting to Chain Nodes**: Establishing connections (e.g., gRPC, Tendermint RPC) to nodes of the target CosmWasm chain.
- **Contract Interaction**: Constructing and parsing messages for CosmWasm contract `instantiate`, `execute`, and `query` entry points.
- **Transaction Submission**: Building, signing, and broadcasting Cosmos SDK transactions that contain CosmWasm messages.
- **Fact Observation**: Querying block data, transaction results (including events/logs), and contract state via smart queries.
- **Data Formatting**: Converting data between Causality types and the JSON-based formats typically used in CosmWasm messages and state.
- **Error Handling**: Mapping chain-specific or CosmWasm contract errors to standard Causality errors.

This adapter allows the Causality system to treat CosmWasm-enabled chains as just another domain, enabling cross-domain operations involving CosmWasm assets or contracts.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 