# Causality E2E Tests

This directory contains end-to-end tests for the Causality framework, organized into focused test suites with individual documentation.

## Test Organization

Each e2e test suite is now organized in its own subfolder with a dedicated README explaining what is tested and how to run it:

### 🧪 Test Suites

| Test Suite | Location | Description | Tests |
|------------|----------|-------------|-------|
| **Type System Consistency** | [`type_system_consistency/`](type_system_consistency/) | Cross-layer type system verification and consistency | 20 tests |
| **Core Integration** | [`core_integration/`](core_integration/) | Integration between all core Causality components | 6 tests |
| **Simulation ZK Integration** | [`simulation_zk_integration/`](simulation_zk_integration/) | Simulation engine with zero-knowledge proof integration | 3 tests |
| **Comprehensive Compilation** | [`comprehensive_compilation_workflow/`](comprehensive_compilation_workflow/) | Complete compilation pipeline from Lisp to machine code | 3 tests |
| **Simple Simulation** | [`simple_simulation/`](simple_simulation/) | Basic simulation engine functionality | 3 tests |
| **OCaml Integration** | [`ocaml_harness/`](ocaml_harness/) | Cross-language OCaml ↔ Rust integration with DeFi scenarios | 2 executables |

**Total: 35 tests across 5 test suites + OCaml integration harness**

## Quick Start

### Run All E2E Tests
```bash
cargo test --test comprehensive_compilation_workflow_e2e --test core_integration_e2e --test simulation_zk_integration_e2e --test type_system_consistency_e2e --test simple_simulation_test
```

### Run Individual Test Suites
```bash
# Type system consistency (most comprehensive)
cargo test --test type_system_consistency_e2e

# Core component integration
cargo test --test core_integration_e2e

# Simulation with ZK proofs
cargo test --test simulation_zk_integration_e2e

# Complete compilation workflow
cargo test --test comprehensive_compilation_workflow_e2e

# Basic simulation functionality
cargo test --test simple_simulation_test
```

### Run with Verbose Output
```bash
cargo test --test type_system_consistency_e2e -- --nocapture
```

## Test Suite Details

### 🔍 Type System Consistency (20 tests)
**Purpose**: Verifies type system correctness across all three layers
- **Base Types**: Unit, Bool, Int, Symbol with serialization
- **Product Types**: Tuple operations and tensor composition  
- **Function Types**: Linear function types and application
- **Session Types**: Communication protocols and role-based typing
- **Record Types**: Structured data with field access
- **Cross-Layer**: Consistency between Layer 0 ↔ Layer 1 ↔ Layer 2

### 🔗 Core Integration (6 tests)
**Purpose**: Tests integration between all core Causality components
- **Cross-Module Integration**: Core ↔ Runtime ↔ Compiler ↔ ZK
- **Linear Resource Management**: Resource lifecycle and cleanup
- **Effect System**: Effect definition, composition, and execution
- **Session Communication**: Protocol establishment and execution
- **Content Addressing**: Deterministic content identification
- **Error Handling**: Consistent error propagation across modules

### ⚡ Simulation ZK Integration (3 tests)
**Purpose**: Verifies simulation engine with zero-knowledge proof generation
- **Effect Execution**: Running Layer 2 effects in simulation
- **Proof Generation**: Creating ZK proofs for simulated executions
- **Cross-Chain Operations**: Multi-domain effect coordination
- **Verification Pipeline**: Complete proof verification workflow

### 🔧 Comprehensive Compilation (3 tests)
**Purpose**: Tests complete compilation from Causality Lisp to machine code
- **Layer 2 → Layer 1**: Effects and intents to lambda calculus
- **Layer 1 → Layer 0**: Lambda calculus to register machine instructions
- **Optimization**: Code optimization and constraint solving
- **Intent Resolution**: Declarative intent compilation

### 🎯 Simple Simulation (3 tests)
**Purpose**: Basic simulation engine functionality for learning and debugging
- **Effect Creation**: Creating simple effects for testing
- **Simulation Execution**: Running effects in simulation mode
- **State Tracking**: Monitoring state changes during simulation
- **Resource Management**: Basic linear resource handling

## Documentation Structure

Each test suite includes:
- **README.md**: Detailed description of what is tested and how to run
- **Test Organization**: Clear module structure and test categories
- **Command Examples**: Copy-paste commands for different scenarios
- **Expected Results**: What should happen when tests pass
- **Dependencies**: Required components and their purposes

## Development Workflow

1. **Start with Simple Simulation** for basic understanding
2. **Use Core Integration** to verify component interactions
3. **Run Type System Consistency** for comprehensive type verification
4. **Test Compilation Workflow** for end-to-end compilation
5. **Verify Simulation ZK Integration** for cryptographic correctness

## Continuous Integration

All e2e tests are designed to:
- ✅ Run independently without external dependencies
- ✅ Complete in reasonable time (< 30 seconds total)
- ✅ Provide clear failure messages with context
- ✅ Maintain stability across different environments
- ✅ Cover critical integration points and workflows

## Troubleshooting

If tests fail:
1. Check individual test suite READMEs for specific guidance
2. Run tests with `--nocapture` for detailed output
3. Verify all dependencies compile correctly: `cargo check --all`
4. Run individual test categories to isolate issues
5. Check the test structure matches the documented organization 