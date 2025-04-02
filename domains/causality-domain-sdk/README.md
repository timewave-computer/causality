# Causality Domain SDK

This crate provides a Software Development Kit (SDK) for building new domain adapters for the Causality system.

## Purpose

Contains shared utilities, traits, testing tools, and boilerplate code to simplify the process of integrating new blockchains, data stores, or other external systems as domains within Causality.

Responsibilities might include:

- **Core Domain Traits**: Defining the essential traits that all domain adapters must implement (if not already defined in `causality-core` or `causality-domain`).
- **Helper Functions**: Providing common functions for tasks like data serialization/deserialization, error mapping, or interacting with core Causality components.
- **Testing Utilities**: Offering mock objects, test harnesses, or standard test suites to ensure adapter correctness and compatibility.
- **Boilerplate Generation**: Potentially including macros or scripts to generate skeleton code for new adapters.
- **Common Adapter Logic**: Factoring out logic common to multiple adapters (e.g., basic fact observation polling, connection management).

Developers creating new domain adapters would typically depend on this SDK to leverage existing infrastructure and adhere to Causality's standards.

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 