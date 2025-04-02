# Causality Types

This crate defines the core data structures, traits, and fundamental types used throughout the Causality system.

## Purpose

Provides foundational building blocks for other crates, ensuring consistency in how core concepts are represented. This includes:

- Cryptographic primitives types (`ContentId`, `ContentHash`, `HashOutput`, `HashAlgorithm`, `HashError`)
- Core identifiers (`DomainId`, `TraceId`)
- Temporal types (`Timestamp`, `BlockHeight`, `BlockHash`, `LamportTime`)
- Resource and state representations (`RegisterState`, `Asset`, `Amount`, `Metadata`)
- Common traits like `ContentAddressed`.
- Shared error types or structures for verification and content addressing.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 