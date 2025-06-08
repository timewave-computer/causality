# Causality CLI End-to-End Test Runner

A comprehensive test suite that validates every CLI command documented in the Causality CLI README and documentation files.

## Overview

This test runner systematically tests all CLI commands and subcommands to ensure they work correctly end-to-end. It covers:

- **Help System** - All help commands and topics
- **REPL** - Interactive development environment 
- **Project Management** - Project creation, building, and management
- **Development Workflow** - Compilation, execution, and formatting
- **Zero-Knowledge Operations** - Circuit compilation, proving, and verification
- **Cross-Chain Deployment** - Transaction simulation and submission
- **Analysis & Diagnostics** - Code analysis and security scanning
- **Testing & Validation** - Unit, integration, and e2e testing
- **System Inspection** - Runtime and artifact inspection
- **Visualization** - Effect and architecture visualization
- **Configuration** - System configuration management

## Usage

### Basic Usage

Run all tests with default settings:

```bash
cd e2e/cli
cargo run
```

### Command Line Options

```bash
# Set custom timeout (default: 120 seconds)
cargo run -- --timeout 60

# Disable parallel execution
cargo run -- --no-parallel

# Set maximum parallel tests (default: 4)
cargo run -- --max-parallel 2

# Stop on first failure
cargo run -- --fail-fast

# Use custom CLI binary path
cargo run -- --cli-binary ./target/debug/causality

# Show help
cargo run -- --help
```

### Example Commands

```bash
# Quick test run with shorter timeout
cargo run -- --timeout 30 --max-parallel 2

# Sequential execution for debugging
cargo run -- --no-parallel --fail-fast

# Test against development build
cargo run -- --cli-binary ../target/debug/causality --timeout 60
```

## Test Categories

### 1. Help System (`help`)
- Basic help commands
- Help topics (tutorial, guides, reference, examples, api, troubleshooting)
- Command-specific help
- Subcommand help
- Invalid help topics

### 2. REPL (`repl`)
- Interactive environment options
- Debug mode and state display
- Tutorial loading
- Auto-save functionality
- Command aliases

### 3. Project Management (`project`)
- Project creation with templates (basic, defi, privacy, zk, library, advanced)
- Project initialization
- Building and cleaning
- Status checking
- Dependency management

### 4. Development Workflow (`dev`)
- Code compilation to various formats
- Program execution with tracing
- Development server
- Code formatting

### 5. Zero-Knowledge Operations (`zk`)
- Circuit compilation with different privacy levels
- Proof generation and verification
- Trusted setup ceremonies
- Multiple proof systems (groth16, plonk, stark, marlin)

### 6. Cross-Chain Deployment (`deploy`)
- Deployment simulation
- Transaction submission
- Cost analysis
- Deployment reporting

### 7. Analysis & Diagnostics (`analyze`)
- Static code analysis
- Resource usage analysis
- Effect composition analysis
- Security analysis

### 8. Testing & Validation (`test`)
- Unit testing
- Effects testing
- Integration testing
- End-to-end testing

### 9. System Inspection (`inspect`)
- System health diagnostics
- Artifact inspection
- Runtime state inspection

### 10. Visualization (`viz`)
- Effect flow visualization
- Resource dependency diagrams
- Architecture overviews

### 11. Configuration (`config`)
- Configuration viewing and setting
- Global vs project-local settings
- Configuration reset

## Test Output

The test runner generates three types of reports:

### 1. Console Output
Real-time progress and summary displayed in the terminal.

### 2. JSON Report (`test-results.json`)
Machine-readable detailed results including:
- Test metadata and timing
- Individual test results
- Environment information
- Category breakdowns

### 3. Markdown Report (`test-results.md`)
Human-readable summary including:
- Executive summary with metrics
- Results by category
- Failed test details
- Environment information
- Detailed test results table

## Test Architecture

### Core Components

- **`main.rs`** - Main test runner orchestration
- **`test_utils.rs`** - Testing framework and utilities
- **`tests/`** - Individual test modules by category

### Test Framework Features

- **Parallel Execution** - Run tests concurrently for speed
- **Timeout Management** - Prevent hanging tests
- **Environment Isolation** - Clean test environments
- **File Management** - Automatic cleanup of test artifacts
- **Flexible Assertions** - Multiple validation options
- **Skip Mechanism** - Skip unimplemented features
- **Comprehensive Logging** - Detailed execution traces

### Test Types

1. **Help Tests** - Validate help output and exit codes
2. **Command Validation** - Test argument parsing and validation
3. **File Operation Tests** - Test file creation and manipulation
4. **Integration Tests** - Test command chains and workflows
5. **Error Handling Tests** - Test invalid inputs and edge cases

## Configuration

Tests can be configured through:

- Command line arguments
- Environment variables (`CAUSALITY_TEST_MODE`, `CAUSALITY_NO_PROMPT`)
- Test-specific settings (timeouts, working directories)

## Implementation Status

Many tests are currently marked as "skipped" because the corresponding CLI functionality may not be fully implemented yet. As the CLI implementation progresses, these tests can be updated to run and validate the actual functionality.

The test framework is designed to:
- Provide immediate value by testing what's currently implemented
- Serve as living documentation of expected CLI behavior
- Enable rapid validation as new features are added
- Support continuous integration and regression testing

## Contributing

When adding new CLI commands or features:

1. Add corresponding tests to the appropriate category module
2. Update test expectations as functionality is implemented
3. Remove skip flags when features become available
4. Add new test categories for major new command groups

The test suite serves both as validation and as documentation of the expected CLI interface. 