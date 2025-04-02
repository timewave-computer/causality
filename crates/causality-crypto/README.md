# Causality Crypto

This crate provides the cryptographic primitives and implementations used within the Causality system.

## Purpose

Consolidates cryptographic operations, ensuring consistent and secure implementations. Responsibilities include:

- **Hashing**: Implementation of cryptographic hash functions (e.g., Blake3, Poseidon) used for content addressing and other integrity checks. Provides traits like `HashFunction` and `ContentHasher`.
- **Signatures**: Implementation and verification logic for digital signature schemes (e.g., Ed25519, ECDSA).
- **Key Management**: Utilities for generating, storing, and handling cryptographic keys.
- **Zero-Knowledge Proofs (ZKP)**: May contain primitives or utilities related to ZKP systems if integrated.
- **Checksums**: Non-cryptographic checksum implementations (like MD5) if needed for specific use cases (feature-gated).

This crate depends on `causality-types` for core cryptographic data structures like `HashOutput`, `ContentHash`, etc., but provides the actual computational logic.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 