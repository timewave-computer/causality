# 000: Introduction to Causality

Welcome to the Causality project! This documentation will guide you through the principles, architecture, and practical application of Causality, a language for verifiable distributed computing.

## What is Causality?

Causality is a programming environment and runtime for verifiable distributed programs. At its core is a linear resource language, where resources are, by default, consumed exactly once. On this foundational concept, Causality builds a layered type system and execution model that **unifies computation and communication as transformations**. Causality aims to provide a robust framework for building systems where the lifecycle of every critical resource is explicit, causally consistent, and verifiable.

## Core Claims

Causality is motivated by several claims about what distributed programming should look like, which you will see reflected throughout this documentation:

1.  **Linear & Immutable**: Resources should be consumed exactly once and transformations should produce new instances, ensuring predictable state updates and eliminating resource safety exceptions.

2.  **Self-describing**: Data, code, and computational effects should be treated uniformly as content-addressed resources, enabling consistent composition through algebraic effects, verifiable global state, and natural deduplication.

3.  **Verifiable**: The system should enable verification at all levels. Static analysis ensures type safety, while runtime privacy and integrity is provided by zero knowledge verification.

4.  **Declarative & Composable**: The framework should enable developers to describe *what* they want to achieve, letting the system handle the *how*, synthesizing optimal, provably correct execution paths. Transform-based effects unify computation and communication, enabling seamless local and distributed programming through a single constraint language.

5.  **Location Transparent**: Operations work the same whether local or remote, with location awareness provided where needed. Communication protocols are automatically derived from data access patterns, eliminating the need for separate protocol specifications.

## Navigating the Documentation

Documentation is structured to build comprehension of the system layer-by-layer:

*   **Design Principles (`001-`)**: A deeper dive into the foundational ideas.
*   **Layered Architecture (`002-` to `005-`)**: Detailed explanations of Layer 0 (Execution Core), Layer 1 (Structured Types & Lisp), and Layer 2 (Transform-Based Effects & Intents).
*   **Practical Guides (`006-` onwards)**: Tutorials, language specifications, toolkit usage, and advanced topics.

Now time to get into Causality's [design principles](./001-design-principles.md).
