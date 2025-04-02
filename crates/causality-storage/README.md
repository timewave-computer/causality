# Causality Storage

This crate provides abstractions and connectors for various storage backends used within the Causality system.

## Purpose

Handles the persistence and retrieval of data, particularly focusing on content-addressed objects and potentially state managed by different components.

Responsibilities include:

- **Storage Traits**: Defining common interfaces for different storage types (e.g., key-value, object storage, relational).
- **Content-Addressed Storage Interaction**: Implementing or extending the `ContentAddressedStorage` trait from `causality-types` for specific backends.
- **Specific Implementations**: Providing concrete storage implementations (e.g., in-memory, file system, RocksDB-based) - though some complex implementations like RocksDB might live in `causality-db`.
- **Sparse Merkle Tree (SMT) Storage**: Handling the persistence layer required for SMTs, potentially integrating with `ContentAddressedStorage`.
- **Caching Layers**: Implementing caching strategies on top of underlying storage backends.

This crate bridges the gap between abstract data management needs in other crates (like `causality-core`) and concrete persistence mechanisms.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 