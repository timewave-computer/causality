# Causality CLI

Command-line interface for the Causality Resource Model framework. This crate provides a clean, minimal CLI for managing projects, running simulations, debugging Resource interactions, and working with blockchain intents and ZK proofs.

## Overview

The Causality CLI serves as the primary developer interface for the Causality system, offering:

- **Project Management**: Initialize, build, and test Causality projects
- **Simulation Tools**: Run and analyze Resource interaction simulations
- **Debugging Utilities**: Debug Resource logic and system behavior
- **Intent Operations**: Submit and query blockchain intents
- **ZK Proof Management**: Generate and verify zero-knowledge proofs

All operations maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Installation

```bash
nix develop
causality --help
```

## Commands

### Project Management

```bash
causality project init my-project --template basic
causality project build
causality project test
causality project clean
```

Templates: `basic`, `zk`, `cross-chain`, `simulation`

### Simulation Tools

```bash
causality simulate run --scenario scenarios/basic.json
causality simulate analyze --trace-id trace_123
causality simulate report --output reports/simulation.html
causality simulate list-scenarios
```

### Debugging Tools

```bash
causality debug resource --resource-id res_123 --verbose
causality debug expr --expr-id expr_456 --context context.json
causality debug dataflow --block-id block_789 --step-by-step
causality debug state --snapshot-id snap_101
```

### Intent Operations

```bash
causality intent submit --intent-file intent.json --chain ethereum
causality intent query --intent-id intent_123
causality intent list --status pending
causality intent cancel --intent-id intent_456
```

### ZK Proof Operations

```bash
causality zk prove resource --resource-id res_123 --circuit validation
causality zk prove dataflow --block-id block_456 --instance inst_789
causality zk verify --proof-id proof_101
causality zk list-circuits
```

## Global Options

```bash
causality --verbose <command>
causality --config custom.toml <command>
causality --log-level debug <command>
causality --format json <command>
```

## Configuration

Configuration file (`.causality/config.toml`):

```toml
[project]
default_template = "basic"
build_dir = "target"

[simulation]
default_steps = 1000
output_dir = "simulations"

[blockchain]
default_chain = "ethereum"
rpc_timeout = "10s"

[zk]
coprocessor_endpoint = "https://coprocessor.valence.xyz"
local_proving = false

[debug]
verbose_by_default = false
trace_expressions = true
```

## Example Usage

### Basic Project

```bash
causality project init token-system --template basic

cat > resources/token.lisp << 'EOF'
(define-resource-type "TokenResource"
  (:fields
    (balance :type integer :required true)
    (owner :type string :required true))
  (:static-expr
    (and (>= (get-field *self-resource* "balance") 0)
         (is-string? (get-field *self-resource* "owner"))))
  (:primary-domain "verifiable-domain"))
EOF

causality project build
causality project test
```

### Intent Submission

```json
{
  "intent_type": "CrossChainTransfer",
  "source_chain": "ethereum",
  "target_chain": "neutron",
  "resource_id": "res_123",
  "parameters": {
    "amount": 1000,
    "recipient": "neutron1abc..."
  }
}
```

### Simulation Configuration

```toml
[simulation]
name = "Resource Transfer Simulation"
duration_steps = 1000

[[simulation.resources]]
type = "TokenResource"
initial_state = { balance = 1000, owner = "alice" }

[[simulation.effects]]
type = "TransferEffect"
frequency = 10
parameters = { amount_range = [1, 100] }
```

## Error Handling

Comprehensive error handling with contextual information and actionable suggestions:

```
ERROR: Resource validation failed
  Resource ID: res_123abc...
  Location: static_expr evaluation
  
  Validation Error:
    Expected: balance >= 0
    Actual: balance = -100
    
  Suggestion:
    Check that transfer amount does not exceed current balance
```

## Feature Flags

- **default**: Standard CLI features
- **simulation**: Simulation command support
- **zk**: Zero-knowledge proof operations
- **blockchain**: Blockchain intent operations
- **debug**: Advanced debugging tools

This CLI provides a comprehensive interface for working with the Causality Resource Model, enabling developers to efficiently manage projects, test Resource interactions, and deploy verifiable systems.