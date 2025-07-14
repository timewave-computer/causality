# Causality CLI

Command-line interface for building, testing, and deploying privacy-preserving, cross-chain applications on the Causality framework.

## Quick Start

```bash
# Interactive development
causality repl

# Create project from template
causality project new my-defi-app --template defi

# Development workflow
causality dev compile -i src/main.lisp -o build/
causality dev run -f build/main.ir --trace
causality test unit --coverage
```

## Core Commands

### **Development Workflow**
- `causality repl` - Interactive REPL environment
- `causality project new` - Create projects from templates (defi, privacy, zk, basic)
- `causality dev compile` - Compile source to IR/circuits
- `causality dev run` - Execute compiled programs with tracing
- `causality test` - Unit, integration, and e2e testing

### **Zero-Knowledge Proofs**
- `causality zk compile` - Compile to ZK circuits
- `causality zk prove` - Generate proofs with witness data
- `causality zk verify` - Verify proofs with public inputs

### **Analysis & Debugging**
- `causality analyze code` - Static analysis and optimization suggestions
- `causality analyze resources` - Resource usage and leak detection  
- `causality analyze security` - Security vulnerability analysis
- `causality inspect system` - System health and performance diagnostics

### **Visualization**
- `causality viz effects` - Effect execution flow diagrams
- `causality viz resources` - Resource dependency graphs
- `causality viz architecture` - System architecture overview

### **Cross-Chain Deployment**
- `causality deploy simulate` - Deployment simulation with cost analysis
- `causality deploy submit` - Submit transactions to target chains
- `causality submit-transaction` - Submit ZK proofs and transactions to multiple blockchains

## Learning Path

1. **Start with REPL**: `causality repl --load-tutorial basic`
2. **Create Project**: `causality project new learning --template basic`
3. **Compile & Run**: `causality dev compile && causality dev run`
4. **Test**: `causality test unit && causality test effects`
5. **Analyze**: `causality analyze code src/`
6. **ZK Development**: `causality zk compile && causality zk prove`

## Project Templates

- **defi**: DeFi applications with liquidity operations
- **privacy**: Privacy-focused apps with ZK proofs
- **zk**: Zero-knowledge circuit development
- **basic**: Learning and experimentation

## Command Overview

The CLI is organized into logical groups that mirror your development workflow:

### **Interactive Development**
- `causality repl` - Start the interactive REPL environment
- `causality help` - Comprehensive help system with tutorials

### **Project Management** 
- `causality project new` - Create projects from templates
- `causality project build` - Build your project
- `causality project status` - Check project health

### **Development Workflow**
- `causality dev compile` - Compile source code to various formats
- `causality dev run` - Execute compiled programs
- `causality dev serve` - Start development server with hot reload
- `causality dev fmt` - Format source code

### **Zero-Knowledge Proofs**
- `causality zk compile` - Compile to ZK circuits
- `causality zk prove` - Generate ZK proofs
- `causality zk verify` - Verify ZK proofs
- `causality zk setup` - Trusted setup ceremonies

### **Cross-Chain Deployment**
- `causality deploy simulate` - Simulate deployment with cost analysis
- `causality deploy submit` - Submit transactions to chains
- `causality submit-transaction` - Submit ZK proofs to multiple blockchains with gas optimization

### **Analysis & Diagnostics**
- `causality analyze code` - Static code analysis
- `causality analyze resources` - Resource usage analysis
- `causality analyze effects` - Effect composition analysis
- `causality analyze security` - Security analysis

### **Testing & Validation**
- `causality test unit` - Run unit tests
- `causality test effects` - Test algebraic effects
- `causality test integration` - Integration testing
- `causality test e2e` - End-to-end testing

### **System Inspection**
- `causality inspect system` - System health diagnostics
- `causality inspect artifacts` - Inspect compiled artifacts
- `causality inspect runtime` - Runtime state inspection

### **Visualization**
- `causality viz effects` - Visualize effect execution flows
- `causality viz resources` - Resource dependency diagrams
- `causality viz architecture` - System architecture overview

### **Configuration**
- `causality config show` - View configuration
- `causality config set` - Set configuration values

## Learning Path

### 1. **Start with the REPL**
The REPL is your best friend for learning the framework:

```bash
# Start with a tutorial
causality repl --load-tutorial basic

# Enable debug mode for learning
causality repl --debug --show-state
```

**REPL Commands:**
```lisp
> (alloc 1000)              # Allocate a resource
> (tensor 10 20)            # Create tensor pairs
> (lambda (x) (+ x 1))      # Define functions
```

### 2. **Explore with Help**
Get contextual help for any topic:

```bash
causality help tutorial     # Framework overview
causality help guides       # Step-by-step guides
causality help reference    # Language syntax
causality help examples     # Code examples
```

### 3. **Create Your First Project**
Choose a template that matches your goals:

```bash
# DeFi application
causality project new defi-bridge --template defi

# Privacy-focused app
causality project new private-voting --template privacy

# ZK circuit development
causality project new zk-proof --template zk

# Basic learning project
causality project new learning --template basic
```

### 4. **Development Workflow**
Follow the natural development flow:

```bash
cd my-project

# Compile your code
causality dev compile -i src/main.lisp -o build/

# Run with execution trace
causality dev run -f build/main.ir --trace

# Start development server
causality dev serve --watch --open

# Format your code
causality dev fmt
```

### 5. **Testing Strategy**
Build confidence with comprehensive testing:

```bash
# Start with unit tests
causality test unit --coverage

# Test effect composition
causality test effects --property-based

# Integration testing
causality test integration --env docker

# End-to-end validation
causality test e2e --chains ethereum,polygon
```

### 6. **Analysis & Debugging**
Understand your code deeply:

```bash
# Analyze code quality
causality analyze code src/ --depth comprehensive

# Check resource usage
causality analyze resources src/main.lisp --detailed --check-leaks

# Security analysis
causality analyze security src/ --level strict --report security.json

# System health
causality inspect system --health-check --perf
```

### 7. **Zero-Knowledge Development**
Add privacy to your applications:

```bash
# Compile to ZK circuit
causality zk compile -i build/main.ir -o build/circuit.zk --privacy-level high

# Generate proofs
causality zk prove -c build/circuit.zk -w witness.json -o proof.zk

# Verify proofs
causality zk verify -c build/circuit.zk -p proof.zk --mock
```

### 8. **Deployment & Production**
Deploy with confidence:

```bash
# Simulate deployment
causality deploy simulate -i build/main.ir --chains ethereum,polygon --cost-analysis

# Submit to chains (dry run first)
causality deploy submit -c circuit.zk -p proof.zk --chains ethereum,polygon --dry-run

# Generate compliance report
causality deploy report --scenario bridge-deployment --include-proofs --include-gas --include-privacy -o report.json
```

##  Visualization & Understanding

Gain insights through visual representation:

```bash
# Visualize effect flows
causality viz effects src/main.lisp --format mermaid --interactive

# Resource dependency graphs
causality viz resources src/main.lisp --flow --states

# System architecture
causality viz architecture --detail comprehensive
```

##  Configuration & Customization

Tailor the CLI to your workflow:

```bash
# View current config
causality config show

# Set preferences
causality config set repl.auto_save true
causality config set output.format json --global

# Project-specific settings
causality config set build.target wasm
causality config set test.parallel true
```

##  Advanced Features

### Command Aliases
Save time with shorter commands:
- `causality r` → `causality repl`
- `causality p new` → `causality project new`
- `causality d c` → `causality dev compile`
- `causality t u` → `causality test unit`
- `causality a c` → `causality analyze code`

### Output Formats
Control output format for automation:
```bash
# JSON output for scripts
causality inspect system --format json

# YAML for configuration
causality config show --format yaml

# Plain text for logging
causality test unit --format plain
```

### Global Options
Available on all commands:
- `--verbose` - Detailed output for debugging
- `--quiet` - Minimize output
- `--format` - Control output format
- `--help` - Command-specific help

##  Development Server

The development server provides a web interface for learning and development:

```bash
causality dev serve --port 3000 --watch --open
```

**Features:**
- **Live REPL**: Interactive development at `/repl`
- **Code Compilation**: Real-time compilation at `/compile`
- **Visualization Tools**: Interactive visualizations at `/visualize`
- **Documentation**: API docs and guides at `/docs`
- **Auto-reload**: Automatic refresh on file changes

## Project Templates

### Basic Template
```bash
causality project new hello-causality --template basic
```
Perfect for learning the framework fundamentals.

### DeFi Template
```bash
causality project new defi-bridge --template defi
```
Cross-chain DeFi applications with:
- Token bridging logic
- Liquidity management
- Automated market making
- Yield farming contracts

### Privacy Template
```bash
causality project new private-voting --template privacy
```
Privacy-preserving applications with:
- Zero-knowledge proofs
- Private state transitions
- Encrypted communications
- Anonymous voting systems

### ZK Template
```bash
causality project new zk-circuits --template zk
```
zkSNARK circuit development with:
- Circuit compilation
- Witness generation
- Proof systems integration
- Verification workflows

### Library Template
```bash
causality project new shared-lib --template library
```
Reusable library development with:
- Module exports
- Dependency management
- Testing framework
- Documentation generation

### Advanced Template
```bash
causality project new enterprise-app --template advanced
```
Production-ready multi-chain setup with:
- Multiple deployment targets
- Comprehensive testing
- CI/CD integration
- Monitoring and analytics

## Common Workflows

### DeFi Development Workflow
```bash
# 1. Create DeFi project
causality project new my-defi --template defi

cd my-defi

# 2. Start development server
causality dev serve --watch &

# 3. Interactive development
causality repl --load-tutorial defi

# 4. Compile and test
causality dev compile -i src/bridge.lisp -o build/bridge.ir
causality test unit --filter bridge
causality test integration --env docker

# 5. ZK proof integration
causality zk compile -i build/bridge.ir -o build/bridge.zk --privacy-level high
causality zk prove -c build/bridge.zk -w witness.json -o proof.zk

# 6. Deployment simulation
causality deploy simulate -i build/bridge.ir --chains ethereum,polygon --cost-analysis

# 7. Production deployment
causality deploy submit -c build/bridge.zk -p proof.zk --chains ethereum,polygon --dry-run
```

### Privacy Application Workflow
```bash
# 1. Create privacy project
causality project new private-app --template privacy

cd private-app

# 2. Develop with privacy analysis
causality dev compile -i src/main.lisp -o build/main.ir
causality analyze security src/ --level paranoid

# 3. ZK circuit development
causality zk compile -i build/main.ir -o build/privacy.zk --privacy-level maximum
causality zk setup -c build/privacy.zk -o setup/ --participants 3

# 4. Privacy testing
causality test effects --pattern privacy --property-based --cases 1000

# 5. Privacy audit report
causality deploy report --scenario privacy-app --include-privacy --include-proofs -o privacy-audit.json
```

### Library Development Workflow
```bash
# 1. Create library
causality project new utils-lib --template library

cd utils-lib

# 2. Development with formatting
causality dev fmt --check
causality dev compile -i src/lib.lisp -o build/lib.ir

# 3. Comprehensive testing
causality test unit --coverage
causality test effects --property-based

# 4. Documentation generation
causality viz effects src/lib.lisp --format mermaid -o docs/effects.md

# 5. Quality analysis
causality analyze code src/ --depth comprehensive -o quality-report.json
```

## Transaction Submission

### Submit ZK Proofs to Blockchains
The `submit-transaction` command provides comprehensive blockchain interaction:

```bash
# Submit to single chain
causality submit-transaction --proof proof.zk --chain ethereum --gas-limit 500000

# Multi-chain submission with optimization
causality submit-transaction --proof proof.zk --chain ethereum,polygon,arbitrum --optimize-gas

# Dry run for cost estimation
causality submit-transaction --proof proof.zk --chain ethereum --dry-run --verbose

# Custom gas pricing
causality submit-transaction --proof proof.zk --chain ethereum --gas-price 20 --priority-fee 2
```

**Supported Chains:**
- Ethereum (mainnet, goerli, sepolia)
- Polygon (mainnet, mumbai)
- Arbitrum (mainnet, goerli)
- Optimism (mainnet, goerli)

**Features:**
- **Gas Optimization**: Automatic gas price discovery and optimization
- **Multi-chain Support**: Submit to multiple chains simultaneously
- **Dry Run Mode**: Cost estimation without actual submission
- **Progress Tracking**: Real-time transaction status monitoring
- **Error Recovery**: Automatic retry with exponential backoff
- **Proof Validation**: ZK proof format verification before submission

**Configuration:**
```bash
# Set default chain
causality config set blockchain.default_chain ethereum

# Configure RPC endpoints
causality config set blockchain.ethereum.rpc_url https://mainnet.infura.io/v3/YOUR_KEY

# Set gas preferences
causality config set blockchain.gas.strategy aggressive
causality config set blockchain.gas.max_fee_per_gas 100
```

## Troubleshooting

### Common Issues

**Compilation Errors:**
```bash
# Check syntax with detailed errors
causality dev compile -i src/main.lisp -o /tmp/test.ir --verbose

# Analyze code for issues
causality analyze code src/main.lisp --depth deep
```

**Runtime Issues:**
```bash
# Run with execution trace
causality dev run -f build/main.ir --trace --max-steps 1000

# Inspect runtime state
causality inspect runtime --memory --stats --live
```

**ZK Proof Issues:**
```bash
# Verify circuit compilation
causality zk compile -i build/main.ir -o /tmp/test.zk --stats

# Test with mock runtime
causality zk verify -c circuit.zk -p proof.zk --mock
```

**Performance Issues:**
```bash
# System performance check
causality inspect system --perf --component all

# Resource usage analysis
causality analyze resources src/ --detailed --check-leaks
```

### Getting Help

1. **Built-in Help**: `causality help troubleshooting`
2. **Command Help**: `causality <command> --help`
3. **Verbose Output**: Add `--verbose` to any command
4. **System Diagnostics**: `causality inspect system --health-check`

## Performance & Best Practices

### Development Performance
- Use `--watch` mode for rapid iteration
- Enable `--parallel` for faster testing
- Use `--format json` for script automation

### Build Optimization
```bash
# Optimized builds
causality dev compile -i src/main.lisp -o build/main.ir --optimize --show-stages

# Release builds
causality project build --release --timings
```

### Testing Efficiency
```bash
# Parallel testing
causality test unit --parallel --filter fast

# Targeted testing
causality test effects --pattern transfer --cases 100
```
