# Type System Consistency E2E Test

This test suite verifies the consistency and correctness of the Causality type system across all three layers (Layer 0: Machine, Layer 1: Lambda Calculus, Layer 2: Effects/Intents).

## What is Tested

### Core Type System Features
- **Base Types**: Unit, Bool, Int, Symbol types with proper serialization
- **Product Types**: Tuple types with tensor composition and decomposition
- **Linear Function Types**: Function types with linear resource discipline
- **Session Types**: Communication protocol types with role-based typing
- **Record Types**: Structured data types with field access and location tracking
- **Capability Types**: Resource access control and permission management

### Cross-Layer Consistency
- **Layer 0 ↔ Layer 1**: Machine values ↔ Lambda calculus terms
- **Layer 1 ↔ Layer 2**: Lambda calculus ↔ Effects and intents
- **Serialization Roundtrips**: SSZ encoding/decoding consistency
- **Type Inference**: Automatic type derivation and constraint solving

### Linear Resource Management
- **Resource Tracking**: Ensuring resources are used exactly once
- **Location Constraints**: Verifying data locality requirements
- **Session Protocols**: Communication pattern enforcement
- **Capability Enforcement**: Access control validation

## How to Run

### Run All Type System Tests
```bash
cargo test --test type_system_consistency_e2e
```

### Run Individual Test Categories

#### Base Type Consistency
```bash
cargo test --test type_system_consistency_e2e base_type_consistency
```

#### Product Type Operations
```bash
cargo test --test type_system_consistency_e2e product_type_consistency
```

#### Function Type Handling
```bash
cargo test --test type_system_consistency_e2e function_type_consistency
```

#### Session Type Protocols
```bash
cargo test --test type_system_consistency_e2e session_type_consistency
```

#### Record Type Structure
```bash
cargo test --test type_system_consistency_e2e record_type_consistency
```

#### Capability System
```bash
cargo test --test type_system_consistency_e2e capability_type_consistency
```

### Run with Verbose Output
```bash
cargo test --test type_system_consistency_e2e -- --nocapture
```

## Test Structure

The test suite is organized into modules:

- `base_type_consistency`: Tests for primitive types (Unit, Bool, Int, Symbol)
- `product_type_consistency`: Tests for tuple/product types and tensor operations
- `function_type_consistency`: Tests for linear function types and application
- `session_type_consistency`: Tests for communication protocols and session roles
- `record_type_consistency`: Tests for structured data and field access
- `capability_type_consistency`: Tests for resource access control

Each module tests:
1. **Type Construction**: Creating types with proper constraints
2. **Value Creation**: Instantiating values of the types
3. **Serialization**: SSZ encoding/decoding roundtrips
4. **Cross-Layer Conversion**: Converting between layer representations
5. **Constraint Validation**: Ensuring type safety and linear discipline

## Expected Results

All 20 tests should pass, verifying:
- ✅ Type system mathematical soundness
- ✅ Linear resource discipline enforcement
- ✅ Cross-layer representation consistency
- ✅ Serialization format stability
- ✅ Constraint system correctness 