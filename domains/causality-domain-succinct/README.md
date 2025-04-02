# Causality Succinct Domain Adapter

This crate provides domain adapter implementations related to succinct proof systems (like ZK-SNARKs, STARKs) within the Causality system.

## Purpose

Integrates Causality with environments where computation or state transitions are verified using cryptographic proofs rather than direct execution or consensus. This could involve interacting with on-chain verifier contracts or off-chain proof generation and verification services.

Responsibilities might include:

- **Proof Generation Interaction**: Communicating with services or libraries that generate proofs for specific computations relevant to Causality effects.
- **Proof Verification**: Verifying proofs, potentially using on-chain verifier contracts or off-chain libraries.
- **State Commitments**: Interacting with systems where state is represented by cryptographic commitments (e.g., Merkle roots) that are updated via proofs.
- **Data Formatting**: Translating Causality data into formats suitable for input to proving systems (e.g., field elements) and interpreting outputs.
- **Fact Observation**: Observing facts related to proof submission and verification events on underlying chains or systems.
- **Error Handling**: Handling errors related to proof generation, verification, or interaction with the succinct environment.

This adapter enables Causality to leverage the privacy and scalability benefits of ZK proofs and other succinct systems for certain operations or domains.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 