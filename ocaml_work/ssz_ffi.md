# TEL Graph SSZ FFI Replacement Work Plan

**STATUS: COMPLETED** - All tasks in this work plan have been successfully completed. The codebase now fully uses SSZ serialization for TEL graph objects exchange between OCaml and Rust.

This plan outlines the steps to replace the current S-expression/ssz-based FFI system with a more efficient SSZ (Simple Serialize) serialization system for TEL graph objects exchange between OCaml and Rust.

## Phase 1: Core SSZ Infrastructure (2 weeks)

### 1.1 Rust SSZ Implementation
- [x] Create a Rust SSZ implementation or adopt an existing library
- [x] Define SSZ type wrappers for core TEL types (Resource, Effect, Handler, Edge)
- [x] Implement serialization and deserialization for primitive types
- [x] Create a feature flag system to enable gradual transition

### 1.2 OCaml SSZ Integration
- [x] Extend the existing `ml_causality/lib/ssz` module for TEL graph types
- [x] Implement serialization routines for OCaml TEL representations
- [x] Add SSZ type definitions for all TEL graph components
- [x] Create serialization helpers to simplify usage

### 1.3 Type Schema Alignment
- [x] Define canonical SSZ schemas for all shared types
- [x] Document the binary layout standards for cross-language compatibility
- [x] Create validators to ensure OCaml and Rust implementations produce identical bytes
- [x] Implement schema versioning mechanism for future-proofing

## Phase 2: FFI Layer Redesign (2 weeks)

### 2.1 New FFI Interface Design
- [x] Design clean new FFI interface for Rust-OCaml boundary
- [x] Create SSZ-based serialization/deserialization functions for FFI
- [x] Implement memory management for crossing language boundaries
- [x] Add error handling and validation at boundary

### 2.2 TEL Graph Serialization Format
- [x] Define canonical SSZ serialization format for complete TEL graphs
- [x] Implement graph serialization/deserialization in both languages
- [x] Create incremental update mechanism for efficient transfer
- [x] Add support for partial graph transfers

### 2.3 FFI Testing Framework
- [x] Create comprehensive test vectors for all TEL types
- [x] Implement round-trip testing framework
- [x] Add fuzzing tests for serialization robustness
- [x] Create benchmark suite to measure performance improvements

### 2.4 Disk Serialization
- [x] Implement file I/O operations for SSZ-serialized objects in Rust
- [x] Implement file I/O operations for SSZ-serialized objects in OCaml
- [x] Create a common file format specification with versioning
- [x] Add tests for file serialization/deserialization

## Phase 3: OCaml DSL Integration (2 weeks)

### 3.1 DSL Output Format Update
- [x] Refactor OCaml DSL to emit SSZ-serializable TEL objects
- [x] Update PPX transpiler to use SSZ-compatible outputs
- [x] Modify graph construction code to work with new serialization
- [x] Add validation for generated SSZ outputs

### 3.2 OCaml Runtime Adaptation
- [x] Update OCaml runtime to consume SSZ-serialized TEL graphs
- [x] Modify graph execution code to work with new serialization
- [x] Create seamless type conversions for OCaml native types
- [x] Update any tooling that depends on S-expressions

## Phase 4: Rust Runtime Integration (2 weeks)

### 4.1 Rust API Update
- [x] Update Rust APIs to accept SSZ-serialized inputs
- [x] Modify core TEL execution code to use SSZ deserialization 
- [x] Update any type-specific handling of TEL objects
- [x] Create conversion utilities for legacy code

### 4.2 Performance Optimization
- [x] Implement zero-copy deserialization where possible
- [x] Create caching mechanisms for repeated structures
- [x] Optimize memory allocation patterns
- [x] Add performance benchmarks against previous implementation

## Phase 5: Migration Strategy (3 weeks)

### 5.1 Dual-Mode Support
- [x] Implement feature flags for toggling between old and new serialization
- [x] Create adapter functions to translate between formats during transition
- [x] Add detection mechanisms to identify serialization format
- [x] Build version negotiation protocol for mixed environments

### 5.2 Refactor FFI Call Sites
- [x] Identify and update all FFI call sites in OCaml
- [x] Update all FFI function implementations in Rust
- [x] Create unified error handling approach
- [x] Add logging for serialization debugging

### 5.3 Full System Testing
- [x] Test full TEL graph serialization/deserialization flows
- [x] Verify compatibility with all existing functionality
- [x] Create integration tests for complex graphs
- [x] Validate merkleization capabilities

## Phase 6: Documentation and Cleanup (1 week)

### 6.1 Documentation
- [x] Update API documentation for all FFI functions
- [x] Create developer guide for SSZ serialization patterns
- [x] Document binary format specifications
- [x] Create examples for common serialization tasks

### 6.2 Cleanup
- [x] Remove all ssz and S-expression serialization code
- [x] Clean up adapter functions after migration
- [x] Remove feature flags after full transition
- [x] Update build system for new dependencies

## Technical Considerations

1. **Binary Compatibility**: Ensure exact byte-level compatibility between OCaml and Rust SSZ implementations.

2. **Memory Management**: Be particularly careful with memory allocation and deallocation across FFI boundaries.

3. **Error Handling**: Create consistent error reporting mechanisms for serialization failures.

4. **Performance**: Focus on minimal copying and efficient memory layout for large graph structures.

5. **Testing Strategy**: Use property-based testing to ensure serialization correctness across languages.

## Implementation Schedule

- **Weeks 1-2**: Core SSZ Infrastructure
- **Weeks 3-4**: FFI Layer Redesign
- **Weeks 5-6**: OCaml DSL Integration
- **Weeks 7-8**: Rust Runtime Integration
- **Weeks 9-11**: Migration Strategy
- **Week 12**: Documentation and Cleanup

## Success Criteria

- Complete removal of S-expression and ssz serialization from FFI
- Byte-identical serialization between OCaml and Rust
- Equal or better performance than previous system
- Comprehensive test coverage for all serialized types
- Clean, well-documented API for future modifications 