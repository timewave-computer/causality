# Causality CLI

Command-line interface for the Causality framework that provides comprehensive tooling for development, debugging, visualization, and REPL interaction with the three-layer architecture.

## Purpose

The `causality-cli` crate serves as the primary developer interface for the Causality system, offering a unified command-line experience for working with all aspects of the three-layer architecture. It provides essential tooling for development, testing, debugging, and interactive exploration while maintaining consistency with the resource model's content-addressed, deterministic properties.

### Key Responsibilities

- **Interactive Development**: REPL environment for exploring the Causality system
- **System Diagnostics**: Comprehensive diagnostic tools for debugging and analysis
- **Visualization**: Graphical representation of effects, resources, and system state
- **Development Workflow**: Streamline the development and testing workflow

## Architecture Overview

The CLI is structured around several operational domains:

### Interactive REPL Domain
Complete REPL environment for interactive development:
- **Expression Evaluation**: Real-time evaluation of Causality expressions
- **Resource Inspection**: Interactive resource exploration and manipulation
- **Effect Visualization**: Visual representation of effect execution
- **System Integration**: Direct integration with all system components

### Diagnostics and Analysis Domain
Comprehensive diagnostic and analysis capabilities:
- **System Analysis**: Deep analysis of system state and behavior
- **Effect Profiling**: Performance analysis and optimization guidance
- **Error Diagnostics**: Detailed error analysis with actionable suggestions
- **Resource Validation**: Comprehensive resource validation and verification

### Visualization Domain
Rich visualization capabilities for system understanding:
- **Effect Flow Visualization**: Graphical representation of effect execution flows
- **Resource State Diagrams**: Visual representation of resource states and transitions
- **System Architecture Views**: Comprehensive system architecture visualization

## Core Commands

### Interactive REPL (`repl`)

Interactive Read-Eval-Print Loop for development:

```bash
# Start interactive REPL
causality repl

# REPL commands:
> create resource "token" { balance: 1000, owner: "alice" }
> eval (transfer token 100 "bob")
> inspect resource token
> visualize effects last
```

**REPL Features:**
- **Expression Evaluation**: Evaluate Causality expressions interactively
- **Resource Creation**: Create and manipulate resources
- **Effect Execution**: Execute effects and observe results
- **System Inspection**: Inspect system state and configuration

### System Diagnostics (`diagnostics`)

Comprehensive system analysis and debugging:

```bash
# Run full system diagnostics
causality diagnostics --comprehensive

# Analyze specific components
causality diagnostics --component machine --verbose
causality diagnostics --component effects --trace-dependencies

# Check system health
causality diagnostics --health-check --report-format json
```

**Diagnostic Capabilities:**
- **Component Analysis**: Deep analysis of individual system components
- **Performance Profiling**: Identify performance bottlenecks and optimization opportunities
- **Dependency Analysis**: Analyze component dependencies and interactions
- **Health Monitoring**: Comprehensive system health checks

### Visualization Tools (`visualizer`)

Rich visualization capabilities for system understanding:

```bash
# Visualize effect execution flow
causality visualizer effects --flow --output flow.svg

# Generate resource state diagrams
causality visualizer resources --state-diagram --format png

# Create system architecture overview
causality visualizer system --architecture --interactive
```

**Visualization Features:**
- **Effect Flow Diagrams**: Visual representation of effect execution patterns
- **Resource State Visualization**: Graphical resource state transitions
- **System Architecture**: Comprehensive system architecture diagrams
- **Interactive Exploration**: Interactive visualization with drill-down capabilities

### Testing Framework (`test-effects`)

Comprehensive effect testing and validation:

```bash
# Run effect test suite
causality test-effects --all --parallel

# Test specific effect patterns
causality test-effects --pattern transfer --verbose

# Validate effect correctness
causality test-effects --validate --property linearity
```

**Testing Capabilities:**
- **Property-Based Testing**: Validate effect properties and invariants
- **Regression Testing**: Comprehensive regression testing for effects
- **Performance Testing**: Effect performance testing and benchmarking
- **Correctness Validation**: Verify effect correctness and safety properties

## Configuration Management

### Global Configuration

CLI configuration system supporting multiple environments:

```toml
# ~/.causality/cli-config.toml

[repl]
auto_save = true
history_size = 1000
prompt_style = "enhanced"

[diagnostics]
default_verbosity = "info"
enable_profiling = true
output_format = "pretty"

[visualization]
default_format = "svg"
theme = "dark"
interactive_mode = true

[testing]
parallel_execution = true
default_timeout = "30s"
generate_reports = true
```

## Error Handling and User Experience

### Contextual Error Messages

Comprehensive error handling with actionable feedback:

```
ERROR: Effect execution failed
  Effect: transfer
  Error: Insufficient balance
  
  Context:
    Source balance: 50
    Transfer amount: 100
    Required: balance >= transfer_amount
  
  Suggestions:
    1. Check source resource balance before transfer
    2. Reduce transfer amount to 50 or less
    3. Add balance validation to transfer effect
```

## Design Philosophy

### Developer-Centric Design
The CLI prioritizes developer experience and productivity:
- **Intuitive Commands**: Natural language-inspired command structure
- **Rich Feedback**: Comprehensive output with actionable information
- **Progressive Discovery**: Help users discover capabilities as they learn
- **Interactive Exploration**: Powerful REPL for hands-on experimentation

### Consistency with Resource Model
All CLI operations maintain consistency with the underlying resource model:
- **Content Addressing**: All artifacts referenced by content-addressed identifiers
- **Deterministic Operations**: Commands produce reproducible results
- **Verifiable Outputs**: All operations maintain audit trails and verifiability

### Performance and Efficiency
Optimized for both interactive use and automated workflows:
- **Lazy Loading**: Load components only when needed
- **Caching**: Intelligent caching of frequently accessed data
- **Responsive Interface**: Fast response times for interactive operations

## Testing Framework

Comprehensive testing across all CLI functionality:

```rust
#[test]
fn test_repl_basic_operations() {
    let mut repl = setup_test_repl();
    
    // Test resource creation
    let result = repl.execute("create resource test { value: 42 }");
    assert!(result.is_ok());
    
    // Test resource inspection
    let result = repl.execute("inspect resource test");
    assert!(result.unwrap().contains("value: 42"));
}

#[test]
fn test_diagnostics_system_health() {
    let diagnostics = setup_test_diagnostics();
    
    let health_report = diagnostics.run_health_check();
    assert!(health_report.overall_status.is_healthy());
    
    let component_status = diagnostics.check_component("machine");
    assert!(component_status.is_operational());
}

#[test]
fn test_visualizer_effect_flow() {
    let visualizer = setup_test_visualizer();
    
    let effect_flow = create_test_effect_flow();
    let diagram = visualizer.generate_flow_diagram(&effect_flow);
    
    assert!(diagram.contains_nodes(&["transfer", "validate", "update"]));
    assert!(diagram.has_edges_between("transfer", "validate"));
}
```

This comprehensive CLI provides developers with powerful, intuitive tools for working with the entire Causality ecosystem while maintaining the mathematical rigor and verifiable properties essential for distributed zero-knowledge computation.