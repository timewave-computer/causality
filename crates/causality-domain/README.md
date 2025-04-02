# Causality Domain

This crate provides implementations of domain adapters for various external systems, such as blockchains and data stores, used by the Causality system.

## Purpose

Acts as the bridge between the abstract effect system defined in `causality-core` and the concrete operations required to interact with specific external domains. It translates generic effects into domain-specific API calls or transactions.

Responsibilities include:

- **Domain Adapter Interface**: May define or implement core traits for domain adapters.
- **Specific Adapters**: Implementing adapters for different domains (e.g., Ethereum, CosmWasm, local storage, external APIs).
- **Fact Observation Logic**: Implementing the domain-specific logic for observing facts (e.g., reading block data, querying contract state).
- **Transaction Submission**: Handling the specifics of submitting transactions or updates to a particular domain.
- **Data Translation**: Converting data between the Causality system's internal formats and the domain's native formats.
- **Error Handling**: Mapping domain-specific errors to the common error types defined in `causality-error`.

This crate allows the core Causality system to remain agnostic to the specifics of external chains or services, promoting modularity and extensibility.

Individual domain implementations might reside in sub-crates or submodules within this crate, or potentially in separate crates within the `domains/` directory (like `domains/causality-domain-evm`).

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 