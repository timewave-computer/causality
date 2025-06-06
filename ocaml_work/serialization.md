# Hybrid Serialization Strategy for Rust/OCaml Interoperability and ZK Circuits

## Status

Proposed

## Goal

*   Enable content-addressing of objects during development using a human-readable, canonical format.
*   Facilitate robust interoperability between Rust and OCaml components for general data exchange.
*   Utilize a ZK-circuit-friendly, deterministic, and compact binary format (ssz) for final ZK witness generation.

## Background: Current Rust-Side Content Addressing and ZK Integration

Before detailing the hybrid strategy, it's important to understand the established patterns for serialization and content addressing within the existing Rust codebase, especially concerning the Zero-Knowledge (ZK) subsystem:

*   **Canonical Serialization for Hashing**: The primary method for achieving content-addressed identifiers (`ExprId`, `ResourceId`, `CircuitId`, etc.) throughout the Rust components (`causality-core`, `causality-compiler`, `causality-zk`) is through **ssz serialization followed by a SHA256 hash**.
    *   Objects are first serialized into a canonical byte vector using ssz (`ssz::to_vec` or `ssz::try_to_vec`).
    *   This byte vector is then hashed using SHA256, and the resulting hash (or a part of it) serves as the object's unique, content-derived identifier.

*   **`causality-zk` Crate Utilities**: The `causality-zk` crate provides core abstractions for this pattern:
    *   A `ContentAddressable` trait offers a `content_hash()` method (which uses ssz + SHA256) and is implemented for any type `T: sszSerialize`.
    *   Helper functions like `id_from_serializable` and `hash_from_serializable` encapsulate this "ssz serialize then SHA256 hash" logic.
    *   ZK-specific data structures (e.g., `ZkResource`, `ZkEffect`) and core identifiers (`WitnessId`, `ProofId`, `CircuitId`) are designed to be ssz-serializable and often derive their identity from this hashing mechanism. `CircuitId`, for instance, is a hash of a ssz-serialized structure containing other content-addressed IDs, forming a hierarchical cryptographic commitment.

*   **ZK Coprocessor Strategy & Performance**:
    *   **Pre-computation of Hashes**: For efficiency, the ZK circuits (running in a `no_std` RISC-V environment) generally avoid performing costly serialization and hashing of large input structures. Instead, these hashes are pre-computed by off-circuit components (like `causality-runtime`) during witness generation.
    *   **Circuit Operation**: The ZK circuit receives these pre-computed hashes as part of its public inputs or witness. Its primary role is to verify that the witness data is consistent with these hashes and that the computation adheres to the rules defined by the (content-addressed) `CircuitId`.
    *   **Serialization within the Circuit**:
        *   While full serialization *of inputs for hashing* is avoided, ssz serialization may still occur *inside* the ZK circuit for:
            *   Serializing structured outputs that the circuit produces.
            *   Content-addressing small, new data items generated *by* the circuit logic itself, if needed.
        *   ssz deserialization of witness inputs also occurs within the circuit.
    *   The overall goal is to minimize computational load within the ZK proof generation step.

This existing robust system for deterministic binary serialization and content addressing using ssz forms a strong foundation for the ZK-specific aspects of the proposed hybrid strategy. The S-expression layer aims to improve developer experience and Rust/OCaml interoperability for general data exchange and development-time content addressing, while ssz remains the designated format for ZK witness finalization.

## Strategy Overview

This strategy employs a two-pronged approach to serialization, catering to different needs at different stages of development and deployment:

### 1. Development & General Interoperability: S-expressions

*   **Format**: Canonical S-expressions.
*   **Usage**:
    *   **Content-Addressing**: Objects will be serialized to a canonical S-expression string representation. The hash of this string will serve as the content address (ID) for the object during development and for general object management. This ensures that identical logical objects get identical IDs.
    *   **Data Exchange**: When Rust and OCaml components need to exchange data structures (not directly intended for ZK witness at this stage), they will use this S-expression format.
    *   **Debugging & Logging**: The human-readable nature of S-expressions will simplify debugging and logging of these data structures.
*   **Implementation**:
    *   **Rust**:
        *   Define `to_canonical_sexpr()` methods for relevant Rust structs.
        *   Implement `from_sexpr()` methods or functions for parsing S-expressions back into Rust structs.
        *   The `lexpr` crate can be utilized for robust S-expression parsing and generation.
    *   **OCaml**:
        *   Define `to_canonical_sexp : t -> Sexplib.Sexp.t` and `of_canonical_sexp : Sexplib.Sexp.t -> t` functions for relevant OCaml types.
        *   Leverage `ppx_sexp_conv` for deriving serializers/deserializers where possible, with custom converters to ensure the specific canonical S-expression format is met (especially regarding type tags and field naming conventions).
*   **Canonicalization**: Strict rules for S-expression generation must be defined and adhered to by both Rust and OCaml implementations. This includes:
    *   Consistent use of type tags (e.g., the first symbol in a list).
    *   Defined order for fields if not using keyword-style fields (keyword-style with alphabetically sorted keywords is preferred for canonicalization).
    *   Uniform representation for basic types (integers, strings, booleans, floats).
    *   Consistent representation of lists, optionals (`nil` or specific tags), and maps (e.g., association lists with sorted keys).

### 2. ZK Witness Generation / Deployment: ssz

*   **Format**: ssz.
*   **Usage**:
    *   When data needs to be prepared as a witness for a ZK circuit, it will be serialized using ssz. This ensures a compact, deterministic, and ZK-friendly binary format.
*   **Implementation**:
    *   **Rust**:
        *   Core Rust structs that form the basis of ZK circuit inputs will derive `sszSerialize` and `sszDeserialize`.
        *   Dedicated Rust functions will be responsible for taking high-level application data and preparing/serializing the precise witness structure using ssz.
    *   **OCaml**:
        *   **Primary Role**: OCaml components will typically prepare data and might pass it to Rust (e.g., via S-expression interop or other FFI if needed) for final ssz-based ZK witness generation if the ZK Prover and Verifier are primarily Rust-based.
        *   **Secondary/Future Role (Complex)**: If OCaml *must* directly generate a byte-for-byte identical ssz witness for a Rust-defined struct layout (e.g., for an OCaml-based ZK prover needing to match Rust's ssz), this would require a highly specialized OCaml ssz serialization library or custom serializers in OCaml that meticulously replicate Rust's ssz output for the specific target structs. This is considered a more advanced and potentially brittle task and will be deferred unless strictly necessary. The initial focus is on Rust generating the ssz witness.
*   **Determinism**: ssz is inherently deterministic, which is ideal for ZK circuits.

## Work Plan

### Phase 1: Canonical S-expression Definitions & Tooling

*   **Task 1.1: Define Canonical S-expression Schemas (Cross-Language)**
    *   Identify key Rust structs and OCaml types that need to be exchanged and/or content-addressed.
    *   For each, define a precise, canonical S-expression representation.
        *   Specify type tags (e.g., `(point ...)`, `(resource ...)`).
        *   Specify field representation (e.g., `(field_name value)` or positional). Prefer keyword-style with alphabetical sorting of keywords for easier canonicalization.
        *   Specify representation for primitives (integers, strings, bools), collections (lists, maps - e.g., association lists with sorted keys), and optionals (`nil` vs. specific tag like `(some ...)`).
    *   Document these schemas in `ml_work/s_expression_schema.md` (or similar).
    *   **Participants**: Rust team, OCaml team.
    *   **Deliverable**: Documented S-expression schemas.

*   **Task 1.2: Implement Rust S-expression Serializers/Deserializers**
    *   For each Rust struct identified in Task 1.1, implement:
        *   `fn to_canonical_sexpr_string(&self) -> String` (or `fn to_lexpr_value(&self) -> lexpr::Value`).
        *   `fn from_sexpr_string(s: &str) -> Result<Self, MyError>` (or `fn from_lexpr_value(val: &lexpr::Value) -> Result<Self, MyError>`).
    *   Ensure these adhere strictly to the canonical schemas from Task 1.1.
    *   Utilize the `lexpr` crate for parsing and generation.
    *   Add unit tests for all conversions.
    *   **Owner**: Rust Team.
    *   **Deliverable**: Rust code for S-expression conversion.

*   **Task 1.3: Implement OCaml S-expression Serializers/Deserializers**
    *   For each OCaml type identified in Task 1.1, implement:
        *   `val to_canonical_sexp : t -> Sexplib0.Sexp.t`
        *   `val of_canonical_sexp : Sexplib0.Sexp.t -> t`
    *   Ensure these adhere strictly to the canonical schemas from Task 1.1.
    *   Use `Sexplib0` (or `Sexplib` if preferred) and `ppx_sexp_conv` where possible. Write custom converters if `ppx_sexp_conv` defaults do not match the canonical S-expression format (e.g., to ensure specific type tags or field name conventions from Task 1.1).
    *   Add unit tests (e.g., using Alcotest, `ppx_inline_test`) for all conversions.
    *   **Owner**: OCaml Team.
    *   **Deliverable**: OCaml code for S-expression conversion.

*   **Task 1.4: Implement Content-Addressing based on S-expressions**
    *   Develop a utility function (potentially in both Rust and OCaml, or primarily in Rust if IDs are generated there) that:
        *   Takes an object/data structure.
        *   Serializes it to its canonical S-expression string (using functions from Task 1.2 or 1.3).
        *   Computes a cryptographic hash (e.g., SHA256) of this canonical string.
        *   Returns this hash as the content address/ID.
    *   Ensure the S-expression serialization used for hashing is absolutely deterministic (e.g., sorted map keys, consistent whitespace if not using a library that normalizes this).
    *   **Note**: This S-expression based content addressing is for development-time object identification and cross-language interop. It is distinct from Rust's internal, established content addressing which uses ssz serialization before hashing (primarily for ZK-related identifiers and internal canonical representation). The hashing algorithm (e.g., SHA256) can be consistent.
    *   **Owner**: Rust Team (primary if IDs are Rust-centric), OCaml Team (for consistency check or if OCaml also generates IDs).
    *   **Deliverable**: Content-addressing utility functions and examples for S-expression based IDs.

*   **Task 1.5: Implement Developer Tooling for ID Conversion (ssz-ID <-> S-expression-ID)**
    *   Develop utility functions (primarily in Rust) that allow for the dynamic lookup and conversion between internal ssz-derived IDs and their S-expression-derived ID counterparts.
    *   **Functionality 1 (ssz-ID to S-expression-ID):**
        *   Input: An internal ssz-derived ID (e.g., `ExprId`, `ResourceId`) and context/type information to retrieve the underlying Rust object.
        *   Process: Retrieve the Rust object, serialize it to its canonical S-expression string (using functions from Task 1.2), and then hash that string (using functions from Task 1.4) to get the S-expression-derived ID.
    *   **Functionality 2 (S-expression-ID to ssz-ID):**
        *   Input: An S-expression-derived ID and the canonical S-expression string (or means to retrieve it), plus context/type information.
        *   Process: Parse the S-expression string into the native Rust object (using functions from Task 1.2), then use existing internal Rust mechanisms to compute its ssz-derived ID (e.g., via a `.id()` helper or utility).
    *   **Usage Context**: Intended for non-performance-critical paths such as developer tooling, debugging aids, enhanced logging, and manual inspection APIs.
    *   This requires access to the live data or definitions corresponding to the IDs.
    *   **Owner**: Rust Team (primary, as it involves internal Rust object handling and ID generation).
    *   **Deliverable**: Rust utility functions for ID conversion, with examples of usage in developer-facing scenarios.

### Phase 2: ssz Serialization for ZK Witness

*   **Task 2.1: Review and Confirm Core Rust Structs for ZK Witness**
    *   Review existing Rust data structures in `causality-types`, `causality-zk`, and other relevant crates that will serve as direct inputs (or parts of the witness) to the ZK circuits.
    *   Confirm that these structs derive `sszSerialize` and `sszDeserialize`. Many ZK-relevant types should already meet this requirement as per the existing Rust ZK framework.
    *   Document the definitive list of these structs and their roles in witness formation.
    *   **Owner**: ZK Team / Rust Team.
    *   **Deliverable**: Documented list of core Rust structs for ZK witnesses, confirming their `sszSerialize` compatibility.

*   **Task 2.2: Implement ZK Witness Generation Functions in Rust**
    *   Create dedicated Rust functions that:
        *   Accept application-level data (which might be richer than the direct ZK witness).
        *   Transform and prepare this data into the precise structure required by the ZK circuit.
        *   Serialize this prepared witness structure to `Vec<u8>` using ssz.
    *   These functions will be the definitive source for generating ZK-compliant witness byte strings.
    *   **Owner**: Rust Team / ZK Team.
    *   **Deliverable**: Rust functions for generating ssz-serialized ZK witnesses.

*   **Task 2.3: Define and Implement Rust FFI for OCaml <-> ssz Interaction (via S-expressions)**
    *   **Goal**: Enable OCaml components to interact with ssz-serialized data by calling Rust FFI functions. OCaml will send/receive data as S-expressions, and Rust will handle the conversion to/from ssz bytes for specific, ZK-critical or interop-critical Rust structs.
    *   **Activities**:
        *   Identify key Rust structs for which OCaml needs to trigger ssz serialization (e.g., to generate a ZK witness component) or parse ssz bytes (e.g., to consume a ZK output).
        *   Define clear FFI function signatures in Rust for these operations. Examples:
            *   `fn rust_sexpr_to_ssz_bytes(sexpr_input: *const c_char, type_hint: RelevantRustType) -> FfiByteResult` (returns ssz bytes or error).
            *   `fn rust_ssz_bytes_to_sexpr(ssz_input: *const u8, input_len: usize, type_hint: RelevantRustType) -> FfiStringResult` (returns S-expression string or error).
        *   Implement these Rust FFI functions. They will internally:
            *   Use S-expression parsing (from Task 1.2) to convert S-expression input from OCaml into native Rust structs.
            *   Use `sszSerialize` to convert Rust structs to ssz byte arrays.
            *   Use `sszDeserialize` to convert ssz byte arrays (from OCaml) into Rust structs.
            *   Use S-expression generation (from Task 1.2) to convert resulting Rust structs back into S-expression strings for OCaml.
            *   Handle memory management and error reporting across the FFI boundary carefully.
        *   Implement the corresponding OCaml bindings to call these Rust FFI functions.
    *   This approach centralizes ssz logic in Rust, leveraging its native support and avoiding the need for a direct OCaml ssz implementation for these specific Rust types.
    *   **Owner**: Rust Team (FFI implementation), OCaml Team (OCaml bindings and usage).
    *   **Deliverable**: A documented and tested Rust FFI library and corresponding OCaml bindings for S-expression <-> ssz conversion for specified types.

### Phase 3: Integration & Testing

*   **Task 3.1: Test S-expression Interop (Rust <-> OCaml)**
    *   Create integration tests where:
        *   Rust serializes an object to S-expression. OCaml deserializes it and verifies content.
        *   OCaml serializes an object to S-expression. Rust deserializes it and verifies content.
    *   Test content-addressing consistency: an object serialized and hashed in Rust should yield the same ID as the equivalent object serialized and hashed in OCaml.
    *   **Owner**: Rust Team & OCaml Team.
    *   **Deliverable**: Passing integration tests for S-expression interop.

*   **Task 3.2: Test ZK Witness Generation (ssz from Rust)**
    *   Create tests that use the Rust functions from Task 2.2 to generate ssz-serialized witnesses for various scenarios.
    *   Verify the byte output against expected patterns or by feeding them to a test harness for the ZK circuit (if available).
    *   **Owner**: Rust Team / ZK Team.
    *   **Deliverable**: Passing tests for ssz witness generation.

*   **Task 3.3: Document Serialization Formats and Usage**
    *   Update developer documentation:
        *   Clearly specify the chosen canonical S-expression schemas (referencing or embedding from `s_expression_schema.md`).
        *   Explain how to use the S-expression and ssz serialization/deserialization utilities in both Rust and OCaml.
        *   Provide guidelines for content-addressing, clarifying the distinction and relationship between:
            *   S-expression based IDs (for dev/interop).
            *   The established internal Rust ssz-based IDs (for ZK structures, `CircuitId`, `ExprId` within Rust, etc.).
        *   Explain the workflow for preparing ZK witnesses.
    *   **Owner**: Rust Team & OCaml Team.
    *   **Deliverable**: Comprehensive documentation.

---

This document outlines the strategy. Next, we can start detailing `s_expression_schema.md` or begin implementing parts of Phase 1. 