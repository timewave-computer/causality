# End-to-End Testing Developer Guide

This guide covers how to compile, deploy, and run the end-to-end tests for the Causality framework.

## Overview

The e2e directory contains comprehensive integration tests that verify the complete workflow of the Causality framework, including:

- **Bridge E2E Test**: Tests cross-domain token transfers using the TEL (Temporal Effect Logic) system
- **Compilation Pipeline**: Tests the TEG (Temporal Effect Graph) compilation process
- **Runtime Integration**: Tests the Causality runtime with state management
- **Cross-Domain Operations**: Tests TypedDomain interactions and ProcessDataflowBlock orchestration

## Prerequisites

### Nix Environment (Recommended)

This project uses Nix for reproducible builds. Ensure you have:

1. **Nix with flakes enabled**:
   ```bash
   # Install Nix (if not already installed)
   curl -L https://nixos.org/nix/install | sh
   
   # Enable flakes (add to ~/.config/nix/nix.conf)
   experimental-features = nix-command flakes
   ```

2. **Enter the development shell**:
   ```bash
   # From the project root
   nix develop
   ```

### Manual Setup (Alternative)

If not using Nix, ensure you have:

- **Rust toolchain** (stable, with clippy and rust-analyzer)
- **OCaml 5.1+** with dune, findlib, and required packages
- **Build tools**: cmake, pkg-config
- **System libraries**: openssl, libiconv (macOS)

## Project Structure

```
e2e/
├── Cargo.toml              # E2E test dependencies and configuration
├── README.md               # This guide
└── tests/
    └── bridge_e2e_test.rs   # Main bridge integration test
```

## Compilation

### Using Nix (Recommended)

```bash
# Enter the Nix development shell
nix develop

# Build the e2e tests
cd e2e
cargo build

# Or build everything from the project root
cargo build --package causality-e2e-tests
```

### Manual Compilation

```bash
# From the project root
cd e2e

# Build the e2e test crate
cargo build

# Build with verbose output for debugging
cargo build --verbose
```

### Compilation Troubleshooting

**Missing Dependencies**:
```bash
# Check if all causality crates are built
cargo build --workspace

# Verify OCaml environment (if using Nix)
check-ocaml-paths
```

**Build Errors**:
```bash
# Clean and rebuild
cargo clean
cargo build

# Check for dependency conflicts
cargo tree
```

## Running Tests

### Basic Test Execution

```bash
# Run all e2e tests
cargo test

# Run specific test with output
cargo test bridge_e2e_test -- --nocapture

# Run with environment logging
RUST_LOG=debug cargo test bridge_e2e_test -- --nocapture
```

### Individual Test Components

The bridge e2e test includes several sub-tests that can be run individually:

```bash
# Test only compilation
cargo test test_compilation_only -- --nocapture

# Test runtime initialization
cargo test test_runtime_initialization -- --nocapture

# Test domain and resource modeling
cargo test test_domain_resource_modeling -- --nocapture

# Test dataflow definition creation
cargo test test_dataflow_definition_creation -- --nocapture

# Full end-to-end workflow
cargo test test_bridge_e2e_workflow -- --nocapture
```

### Test Configuration

The tests use several configuration options:

```bash
# Enable detailed logging
export RUST_LOG=causality_e2e_tests=debug,causality_compiler=debug

# Set test timeout (default: 30 seconds)
export TEST_TIMEOUT_SECS=60

# Use mock data (when example files are missing)
export USE_MOCK_DATA=true
```

## Test Components

### Bridge E2E Test (`bridge_e2e_test.rs`)

This comprehensive test verifies:

1. **TEG Compilation**: Compiles the cross-domain token transfer example
2. **Runtime Setup**: Initializes mock state manager and TEL interpreter
3. **Domain Creation**: Sets up test domains (Ethereum-like and Polygon-like)
4. **Resource Management**: Creates and manages test accounts and tokens
5. **Workflow Execution**: Executes the complete bridge transfer workflow
6. **State Verification**: Verifies final state and execution results

#### Test Flow

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Compilation   │───▶│  Runtime Setup   │───▶│   Execution     │
│                 │    │                  │    │                 │
│ • TEG parsing   │    │ • State manager  │    │ • Bridge flow   │
│ • Validation    │    │ • TEL interpreter│    │ • Verification  │
│ • Code gen      │    │ • Domain setup   │    │ • Results       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Mock Components

The tests use mock implementations for components not yet fully integrated:

- **MockStateManager**: Simulates state persistence and retrieval
- **MockTelInterpreter**: Simulates TEL expression evaluation
- **Mock Domains**: Represents different blockchain domains
- **Mock Resources**: Represents accounts, tokens, and other resources

## Deployment

### Local Development

```bash
# Run tests in watch mode during development
cargo watch -x "test --package causality-e2e-tests"

# Run with specific features
cargo test --features "mock-runtime"
```

### CI/CD Integration

For continuous integration, use:

```bash
# In CI environment
cargo test --package causality-e2e-tests --release

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --package causality-e2e-tests
```

### Docker Deployment

```dockerfile
# Example Dockerfile for e2e testing
FROM nixos/nix:latest

WORKDIR /app
COPY . .

RUN nix develop --command cargo test --package causality-e2e-tests
```

## Configuration

### Test Dependencies

The e2e tests depend on:

- **causality-types**: Core type definitions
- **causality-core**: Core functionality
- **causality-compiler**: TEG compilation
- **causality-lisp**: Lisp expression evaluation

### Optional Dependencies

Some dependencies are conditionally included:

```toml
# In Cargo.toml
[dependencies]
# causality-runtime = { path = "../crates/causality-runtime" } # Disabled
# causality-zk = { path = "../crates/causality-zk", optional = true } # Disabled
```

### Environment Variables

```bash
# Logging configuration
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Test-specific configuration
export CAUSALITY_TEST_MODE=true
export CAUSALITY_MOCK_RUNTIME=true
```

## Debugging

### Common Issues

**Compilation Failures**:
```bash
# Check workspace dependencies
cargo check --workspace

# Verify feature flags
cargo check --package causality-e2e-tests --all-features
```

**Test Failures**:
```bash
# Run with detailed output
cargo test -- --nocapture --test-threads=1

# Enable debug logging
RUST_LOG=debug cargo test
```

**Missing Example Files**:
The tests automatically fall back to mock data if example files are missing:
```
⚠️ Example file not found at "examples/cross_domain_token_transfer.teg", using mock
```

### Debug Output

The tests provide detailed debug output:

```
📖 Found TEG example file at "examples/cross_domain_token_transfer.teg"
✅ Successfully compiled TEG program
🚀 Initializing Mock Causality Runtime
✅ Mock state manager initialized
✅ Mock TEL interpreter initialized
🔄 Executing Bridge Transfer Workflow
✅ Bridge transfer completed successfully
```

## Performance Considerations

### Test Execution Time

- **Compilation Test**: ~1-2 seconds
- **Runtime Initialization**: ~100ms
- **Full E2E Workflow**: ~5-10 seconds
- **Complete Test Suite**: ~15-30 seconds

### Resource Usage

- **Memory**: ~50-100MB during test execution
- **CPU**: Moderate usage during compilation phase
- **Disk**: Minimal temporary file usage

## Future Enhancements

### Planned Improvements

1. **Real Runtime Integration**: Replace mock components with actual runtime
2. **ZK Integration**: Add zero-knowledge proof verification tests
3. **Multi-Domain Tests**: Expand to test more complex cross-domain scenarios
4. **Performance Benchmarks**: Add performance regression testing
5. **Fuzz Testing**: Add property-based testing for robustness

### Contributing

When adding new e2e tests:

1. Follow the existing test structure and naming conventions
2. Include both positive and negative test cases
3. Add appropriate debug output and error handling
4. Update this README with new test descriptions
5. Ensure tests work with both real and mock components

## Troubleshooting

### Common Error Messages

**"TEG compilation failed"**:
- Check that the example file exists and is valid
- Verify compiler dependencies are built
- Enable debug logging to see detailed error messages

**"Mock state manager initialization failed"**:
- Usually indicates a dependency issue
- Check that all required crates are built
- Verify async runtime is properly configured

**"Bridge transfer workflow failed"**:
- Check test data setup
- Verify domain and resource creation
- Enable detailed logging to trace execution

### Getting Help

1. Check the test output for specific error messages
2. Enable debug logging with `RUST_LOG=debug`
3. Review the test code in `tests/bridge_e2e_test.rs`
4. Check the project's main documentation
5. File an issue with detailed error information

---

For more information about the Causality framework, see the main project documentation. 