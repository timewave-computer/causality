<!-- Glossary of terms -->
<!-- Original file: docs/src/glossary.md -->

# Time-Operators Glossary

## Core Concepts

### Domain
A sequence of states that represents the evolution of a system over time. Domains can be branched, merged, and traversed by Users.

### User
An actor that can navigate between states in a Domain and execute programs that manipulate resources.

### Committee
An actor responsible for maintaining the consistency and integrity of Domains. Committees validate operations and prevent invalid state transitions. Committees implement the Controller interface for their respective Domains.

### Operator
An actor that attempts to exploit vulnerabilities in Domains, often trying to execute double-spend attacks or other manipulation of Domain states.

### Effect
A primitive operation that can be performed on a Domain, such as reading a resource, writing a resource, or branching a Domain.

### Resource
A structured tuple with defined properties including resource logic, fungibility domain, quantity, metadata, and unique identifiers. Resources track their provenance through controller labels and are subject to conservation laws that ensure their total amount remains constant across operations.

### Controller
An entity responsible for managing a Domain and enforcing its rules. Controllers are classified as Safe, Live, or Byzantine based on their security properties and can endorse other controllers' states to enable state reduction optimizations.

### Controller Label
A data structure that tracks the history of a resource as it crosses between Domains, recording the creating controller, terminal controller, affecting controllers, and backup controllers.

### Resource Delta
The net change in resource quantity during an operation. The system enforces that all operations must have a total delta of zero, ensuring conservation of resources.

### Time Map
A data structure that maps Domain states to resources, tracking where resources exist across different Domain states. It serves as a "global clock" for the system, ensuring that all actors have a consistent view of external state.

### Execution Log
A record of all operations performed on Domains, used for debugging, auditing, and verification purposes.

### Program Memory
The state maintained by a program as it executes across different Domain states.

## Advanced Concepts

### Domain Branch
A fork in a Domain that creates a new potential future state path.

### Domain Merge
The operation of reconciling two divergent Domains back into a single consistent Domain.

### Causality Violation
An inconsistent state where an effect depends on a future state that hasn't occurred yet or is in a different branch.

### Double-Spend Attack
An attack where a Operator attempts to use the same resource twice by exploiting branching Domains.

### Temporal Consistency
The property that effects in a Domain are ordered in a way that respects causality.

### Ancestral Validation
A validation mechanism that verifies the provenance of resources by checking their controller history through controller labels.

### Dual Validation
The combination of temporal validation (ensuring causal consistency via time maps) and ancestral validation (verifying controller history), providing defense in depth for cross-domain operations.

### Resource Commitment
A cryptographic commitment to a resource's existence, derived from its properties. Used to prove resource existence without revealing all details.

### Resource Nullifier
A unique value that marks a resource as consumed, preventing double-spending. Created using the resource and a nullifier key.

### Domain Proof
A cryptographic proof that verifies the validity of a Domain state or state transition.

### Resource Ledger
A component that tracks resource ownership and transfers across Domains.

### Domain Descriptor
A specification of a Domain's properties, including its branching model, consensus mechanism, and security parameters.

### Effect Interpreter
A component that translates abstract effects into concrete operations on a specific Domain.

### Program Precondition
A condition that must be satisfied before a program can be executed on a Domain.

### LogTimeMapIntegration
A component that provides the integration between the Time Map and Unified Log System, enabling temporal consistency verification, time-based querying, and causal ordering of log entries.

### Time Map Hash
A cryptographic hash derived from the content of a Time Map, used to ensure integrity when attaching Time Maps to log entries and verifying temporal consistency.

### Time Indexed Entry
A data structure that represents a log entry indexed by time, containing the entry's timestamp, log index, entry type, and associated domain and resource identifiers.

### Time Map Entry
A data structure that represents the state of a domain at a specific point in time, including block height, block hash, and timestamp.

## Content-Addressable Code System

### Content Hash
A cryptographic hash derived from the content of a code definition, used as a unique identifier independent of names.

### Code Definition
A unit of code (function or module) stored in the content-addressable system, identified by its content hash.

### Content-Addressable Repository
A storage system that organizes code by content hash rather than by name, enabling immutability and precise dependency resolution.

### Name Registration
The process of associating a human-readable name with a content hash, allowing code to be referenced by name while maintaining hash-based dependencies.

### Content-Addressable Executor
A runtime component that can execute code retrieved by its content hash, maintaining execution context across invocations.

## Temporal Effect Language (TEL)

### Temporal Effect Language (TEL)
A specialized programming language designed for cross-domain programming in the Causality system, with explicit effects, strong typing, and causal consistency.

### Expression
The basic unit of computation in TEL, which always evaluates to a value.

### Pattern Matching
A mechanism in TEL for destructuring and analyzing values, enabling conditional logic and data transformation.

### Effect Expression
A specialized TEL expression that describes an interaction with external Domains or resources, such as deposit, withdraw, transfer, observe, or emit.

### TEL Interpreter
The component that evaluates TEL expressions, manages effects, and integrates with the Causality runtime.

### TEL Type System
The static type checking system that ensures TEL programs are well-formed and type-safe before execution.

### TEL Program
A collection of function definitions in the Temporal Effect Language that can be deployed to the Causality network.

## Log System

### Log Entry
The fundamental unit of the Unified Log System, which can represent an Effect, Fact, or Event with associated metadata including timestamps, trace IDs, and domain information.

### Fact Entry
A log entry that documents an observed truth or assertion about system state, particularly about external domains.

### Effect Entry
A log entry that records a state change or side effect in the system, which is causally verified against the Time Map.

### Event Entry
A log entry that captures significant occurrences that may not directly change state, used primarily for monitoring and debugging.

### Log Storage
A component responsible for storing and retrieving log entries, with implementations including in-memory, file-based, and distributed storage.

### Log Segment
A portion of the log containing entries within a specific range, used to optimize storage and retrieval operations.

### ReplayEngine
A component that can deterministically replay log entries to reconstruct system state or verify temporal consistency, with support for time-based filtering using the Time Map. 