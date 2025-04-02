# Causality EVM Domain Adapter

This crate provides the domain adapter implementation for interacting with EVM-compatible blockchains (like Ethereum, Polygon, etc.) within the Causality system.

## Purpose

Implements the domain adapter traits defined potentially in `causality-domain` or `causality-core` specifically for EVM chains. It handles the translation between Causality effects/operations and EVM concepts like contract calls, events, and state lookups.

Responsibilities include:

- **Connecting to EVM Nodes**: Establishing RPC connections (HTTP/WebSocket) to EVM nodes.
- **Contract Interaction**: Encoding and decoding function calls and event data using ABIs.
- **Transaction Submission**: Constructing, signing, and broadcasting EVM transactions.
- **Fact Observation**: Querying block data, transaction receipts, logs, and contract state.
- **Data Formatting**: Converting between EVM data types (e.g., `uint256`, `address`) and Causality types.
- **Error Handling**: Mapping EVM-specific errors (e.g., gas issues, reverts) to Causality errors.

This adapter allows the Causality system to treat EVM chains as just another domain, enabling cross-domain operations involving EVM assets or contracts.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 