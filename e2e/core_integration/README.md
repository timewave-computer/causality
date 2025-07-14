# Core Integration E2E Test

This test suite verifies the integration between all core Causality components, ensuring that the fundamental systems work together correctly across module boundaries.

## What is Tested

### Cross-Module Integration
- **Core ↔ Runtime**: Type system integration with execution engine
- **Core ↔ Compiler**: Type system integration with compilation pipeline  
- **Core ↔ ZK**: Type system integration with zero-knowledge proofs
- **Runtime ↔ Simulation**: Execution engine integration with simulation

### Fundamental Operations
- **Linear Resource Management**: Resource creation, usage, and disposal
- **Effect System**: Effect definition, composition, and execution
- **Session Types**: Communication protocol establishment and execution
- **Content Addressing**: Deterministic content identification and retrieval

### System-Level Features
- **Error Handling**: Consistent error propagation across modules
- **Serialization**: Cross-module data serialization and deserialization
- **Concurrency**: Multi-threaded execution and resource safety
- **Memory Management**: Proper resource cleanup and leak prevention

## How to Run

### Run All Core Integration Tests
```bash
cargo test --test core_integration_e2e
```

### Run Individual Test Categories

#### Basic Integration Test
```bash
cargo test --test core_integration_e2e test_basic_integration
```

#### Linear Resource Management
```bash
cargo test --test core_integration_e2e test_linear_resource_management
```

#### Effect System Integration
```bash
cargo test --test core_integration_e2e test_effect_system_integration
```

#### Session Type Communication
```bash
cargo test --test core_integration_e2e test_session_type_communication
```

#### Content Addressing System
```bash
cargo test --test core_integration_e2e test_content_addressing_system
```

#### Cross-Module Error Handling
```bash
cargo test --test core_integration_e2e test_cross_module_error_handling
```

### Run with Verbose Output
```bash
cargo test --test core_integration_e2e -- --nocapture
```

## Test Structure

The test suite is organized into six integration scenarios:

### 1. Basic Integration Test
- Verifies fundamental cross-module communication
- Tests basic type system operations
- Validates core data structure compatibility
- Ensures module initialization works correctly

### 2. Linear Resource Management
- Creates linear resources using the core type system
- Manages resource lifecycle through runtime
- Verifies linear discipline enforcement
- Tests resource cleanup and disposal

### 3. Effect System Integration
- Defines effects using core primitives
- Compiles effects through the compiler
- Executes effects in the runtime
- Validates effect composition and sequencing

### 4. Session Type Communication
- Establishes session protocols using core session types
- Implements communication through runtime channels
- Verifies protocol adherence and type safety
- Tests session lifecycle management

### 5. Content Addressing System
- Creates content using core content addressing
- Stores and retrieves content through runtime
- Verifies deterministic addressing properties
- Tests content integrity and authenticity

### 6. Cross-Module Error Handling
- Generates errors in different modules
- Verifies error propagation across boundaries
- Tests error recovery and cleanup
- Validates consistent error reporting

## Dependencies

This test exercises the integration between:
- **causality-core**: Core type system and primitives
- **causality-runtime**: Execution engine and resource management
- **causality-compiler**: Compilation and code generation
- **causality-zk**: Zero-knowledge proof integration
- **causality-simulation**: Simulation engine integration

## Expected Results

All 6 tests should pass, verifying:
-  All core modules integrate correctly
-  Cross-module APIs are compatible
-  Data serialization works across boundaries
-  Error handling is consistent system-wide
-  Resource management prevents leaks
-  Type safety is maintained across modules
-  Performance is acceptable for integration overhead

## Integration Points Tested

The test suite specifically validates these critical integration points:

### Type System Integration
- Core types serialize/deserialize correctly
- Runtime can execute operations on core types
- Compiler can generate code for core type operations

### Resource Management Integration  
- Core linear resources integrate with runtime lifecycle
- Compiler generates correct resource management code
- ZK system can prove resource usage patterns

### Communication Integration
- Core session types work with runtime communication
- Compiler generates correct protocol code
- Simulation can model communication patterns

### Error Integration
- Errors propagate correctly across module boundaries
- Error types are compatible across modules
- Error recovery works in integrated scenarios 