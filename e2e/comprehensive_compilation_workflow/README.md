# Comprehensive Compilation Workflow E2E Test

This test suite verifies the complete compilation pipeline from Causality Lisp source code through all three layers down to executable machine instructions, ensuring the entire toolchain works correctly end-to-end.

## What is Tested

### Complete Compilation Pipeline
- **Layer 2 → Layer 1**: Effects and intents to lambda calculus
- **Layer 1 → Layer 0**: Lambda calculus to register machine instructions
- **Code Generation**: Producing executable machine code
- **Optimization**: Code optimization and constraint solving

### Source Language Features
- **Causality Lisp Parsing**: S-expression parsing and AST generation
- **Type Checking**: Static type analysis and inference
- **Effect Compilation**: Converting effects to executable code
- **Intent Resolution**: Declarative intent compilation

### Machine Code Generation
- **Register Allocation**: Efficient register usage
- **Instruction Selection**: Optimal instruction sequences
- **Control Flow**: Proper branching and loops
- **Resource Management**: Linear resource tracking in generated code

## How to Run

### Run All Compilation Workflow Tests
```bash
cargo test --test comprehensive_compilation_workflow_e2e
```

### Run Individual Test Categories

#### Basic Compilation Pipeline
```bash
cargo test --test comprehensive_compilation_workflow_e2e test_basic_compilation_pipeline
```

#### Complex Effect Compilation
```bash
cargo test --test comprehensive_compilation_workflow_e2e test_complex_effect_compilation
```

#### Intent System Integration
```bash
cargo test --test comprehensive_compilation_workflow_e2e test_intent_system_integration
```

### Run with Verbose Output
```bash
cargo test --test comprehensive_compilation_workflow_e2e -- --nocapture
```

## Test Structure

The test suite covers three main compilation scenarios:

### 1. Basic Compilation Pipeline
Tests the fundamental compilation flow:
- **Input**: Simple Causality Lisp program with basic operations
- **Layer 2**: Parses to effects and basic intents
- **Layer 1**: Compiles to lambda calculus with linear types
- **Layer 0**: Generates register machine instructions
- **Verification**: Executes generated code and validates results

### 2. Complex Effect Compilation
Tests advanced language features:
- **Input**: Complex program with multiple effects, session types, and resources
- **Cross-Chain Effects**: Multi-domain operations and state management
- **Resource Constraints**: Linear resource usage and capability requirements
- **Optimization**: Code optimization and constraint solving
- **Output**: Optimized machine code with resource tracking

### 3. Intent System Integration
Tests declarative programming features:
- **Intent Definition**: High-level declarative specifications
- **Constraint Solving**: Automatic intent resolution and planning
- **Code Generation**: Converting intents to executable effects
- **Verification**: Ensuring generated code satisfies intent specifications

## Dependencies

This test requires all major Causality components:
- **causality-lisp**: Parser and type checker
- **causality-compiler**: Compilation pipeline
- **causality-core**: Core type system and machine model
- **causality-runtime**: Execution engine

## Expected Results

All 3 tests should pass, verifying:
- ✅ Complete source-to-machine compilation works
- ✅ Generated code executes correctly
- ✅ Type safety is preserved through all layers
- ✅ Linear resources are properly managed
- ✅ Effects compile to correct machine operations
- ✅ Intents resolve to valid effect sequences
- ✅ Optimization preserves program semantics

## Compilation Artifacts

Each test generates several intermediate artifacts:
- **AST**: Parsed abstract syntax tree
- **Typed AST**: Type-checked and annotated AST
- **Layer 1 IR**: Lambda calculus intermediate representation
- **Layer 0 Instructions**: Register machine instruction sequence
- **Execution Trace**: Runtime execution log for verification

## Performance Expectations

Compilation times for test programs:
- Basic programs: <50ms end-to-end
- Complex effects: <200ms with optimization
- Intent resolution: <500ms including constraint solving 