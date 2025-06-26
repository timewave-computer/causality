# OCaml Integration Test Harness

This directory contains OCaml-based end-to-end tests that demonstrate cross-language interoperability between OCaml and the Causality framework, focusing on complex DeFi scenarios with zero-knowledge privacy features.

## What is Tested

### 🔗 Cross-Language Integration
- **OCaml ↔ Causality Lisp**: OCaml scenario generation compiled to Causality Lisp
- **OCaml ↔ Rust**: OCaml test harness orchestrating Rust CLI tools
- **Lisp ↔ IR**: Causality Lisp compilation to intermediate representation
- **IR ↔ Machine**: IR compilation to register machine instructions

### 🏦 DeFi Workflow Scenarios
- **Cross-Chain Bridging**: Token transfers between Ethereum, Polygon, Arbitrum
- **Vault Deposits**: Automated yield farming with strategy selection
- **Privacy Features**: Zero-knowledge proofs for transaction privacy
- **Compliance**: Regulatory compliance proof generation
- **Error Recovery**: Comprehensive error handling and transaction reversal

### 🔐 Zero-Knowledge Integration
- **ZK Circuit Generation**: Creating circuits for DeFi operations
- **Proof Generation**: Generating proofs for bridge and vault operations
- **Privacy Levels**: Configurable privacy settings (Low/Medium/High)
- **Compliance Proofs**: ZK-based regulatory compliance verification

## Test Files

| File | Purpose | Language | Description |
|------|---------|----------|-------------|
| **bridge_vault_e2e.ml** | Main test harness | OCaml | Orchestrates complete E2E workflow |
| **bridge_vault_scenario.ml** | DeFi scenario logic | OCaml | Cross-chain bridge and vault operations |
| **bridge_vault_scenario.lisp** | Tensor operations | Causality Lisp | Resource allocation scenario |
| **dune** | Build configuration | Dune | OCaml build and dependency management |

## How to Run

### Prerequisites
Ensure OCaml and Dune are installed in your nix environment:
```bash
# The flake.nix should already provide these
which ocaml dune
```

### Build OCaml Components
```bash
cd e2e/ocaml_harness
dune build
```

### Run Complete E2E Test
```bash
# Run the main test harness
dune exec ./bridge_vault_e2e.exe
```

### Run Individual Components
```bash
# Test the scenario logic
dune exec ./bridge_vault_scenario.exe

# Compile Lisp scenario manually
cd ../..
./target/debug/causality compile --input e2e/ocaml_harness/bridge_vault_scenario.lisp --output /tmp/bridge_vault.ir --format intermediate --verbose
```

### Run with Verbose Output
```bash
# Enable detailed logging
CAUSALITY_LOG=debug dune exec ./bridge_vault_e2e.exe
```

## Test Workflow

The OCaml harness executes a complete 5-step workflow:

### 1. 🔧 Compile OCaml Scenario to Lisp IR
- Compiles OCaml scenario using Dune
- Converts Causality Lisp to intermediate representation
- Verifies IR file creation and validity
- **Expected**: Successful compilation with IR output

### 2. 🔬 Run Cost Simulation via CLI
- Executes cost analysis simulation
- Multi-chain gas price analysis (Ethereum, Polygon)
- Performance metrics collection
- **Expected**: Detailed cost breakdown and timing

### 3. 🌉 Execute Cross-Chain Bridge Operations
- Simulates token bridging between chains
- Privacy-preserving transaction execution
- ZK proof generation for bridge operations
- **Expected**: Successful bridge with privacy proofs

### 4. 🏦 Execute Vault Deposit with Strategy Selection
- Automated vault strategy selection (Aave/Compound/Yearn)
- Yield optimization based on APY and risk parameters
- Compliance checking and proof generation
- **Expected**: Optimal vault selection and successful deposit

### 5. 📋 Generate Comprehensive Compliance Report
- Regulatory compliance verification
- ZK proof aggregation and validation
- Gas analysis and cost optimization report
- **Expected**: Complete compliance documentation

## Scenario Details

### Cross-Chain Bridge Parameters
```ocaml
type bridge_params = {
    source_chain : chain              (* Ethereum | Polygon | Arbitrum *)
  ; dest_chain : chain
  ; token : token                     (* USDC | WETH | WMATIC *)
  ; amount : int64
  ; privacy_level : [`Low | `Medium | `High]
  ; zk_proof_required : bool
  ; gas_optimization : bool
}
```

### Vault Strategy Configuration
```ocaml
type vault_params = {
    chain : chain
  ; token : token
  ; amount : int64
  ; strategies : vault_strategy list  (* [Aave; Compound; Yearn] *)
  ; min_apy : float
  ; max_risk : [`Low | `Medium | `High]
  ; privacy_preserving : bool
  ; compliance_check : bool
}
```

### Compliance Proof Structure
```ocaml
type compliance_proof = {
    scenario : string
  ; timestamp : float
  ; zk_proofs_count : int
  ; gas_analysis_included : bool
  ; privacy_score : float              (* 0.0 - 1.0 *)
}
```

## Expected Results

### Successful Execution Output (Current Implementation)
```
=== Step 1: Compile OCaml Scenario to Lisp IR ===
✅ OCaml scenario compilation successful
🔄 Compiling Lisp scenario to Causality IR...
✅ Lisp → IR compilation successful
✅ IR file created: /tmp/bridge_vault.ir

=== Step 2: Run Cost Simulation via CLI ===
❌ Simulation failed (simulate command not yet implemented)

Note: Currently only Step 1 (compilation) is fully functional.
Steps 2-5 require CLI commands that are not yet implemented.
```

### Future Complete Workflow (When CLI is Extended)
```
=== Step 1: Compile OCaml Scenario to Lisp IR ===
✅ OCaml scenario compilation successful
🔄 Compiling Lisp scenario to Causality IR...
✅ Lisp → IR compilation successful
✅ IR file created: /tmp/bridge_vault.ir

=== Step 2: Run Cost Simulation via CLI ===
🔬 Running cost simulation...
✅ Simulation completed successfully
   Total gas cost: 245000 gwei
   Execution time: 1250 ms
   Success probability: 0.98

=== Step 3: Execute Cross-Chain Bridge Operations ===
🌉 Bridging 1000000000 USDC from Ethereum to Polygon
   Privacy level: High
   ZK proof required: true
✅ Bridge operation completed with ZK privacy

=== Step 4: Execute Vault Deposit with Strategy Selection ===
🔍 Finding optimal vault on Polygon for USDC
✓ Selected: Aave (APY: 8.5%)
🏦 Depositing 995000000 USDC into vault
✅ Vault deposit successful

=== Step 5: Generate Comprehensive Compliance Report ===
📋 Generating compliance proof for scenario: bridge_vault_e2e
✅ Compliance proof generated
   ZK proofs: 3
   Privacy score: 0.95
   Gas analysis: included

🎉 All steps completed successfully!
```

## Error Handling

The harness includes comprehensive error recovery:

### Bridge Failures
- **Revert Transaction**: Automatic bridge transaction reversal
- **Refund to Source**: Return funds to original chain
- **Gas Refund**: Partial gas cost recovery

### Vault Failures  
- **Withdraw from Vault**: Emergency vault withdrawal
- **Strategy Fallback**: Alternative vault strategy selection
- **Partial Execution**: Handle partial deposits

### ZK Proof Failures
- **Proof Regeneration**: Retry with different parameters
- **Privacy Downgrade**: Fallback to lower privacy levels
- **Circuit Optimization**: Alternative circuit implementations

## Integration Points

### With Causality CLI
- Uses `causality compile` for Lisp → IR compilation
- Uses `causality simulate` for cost analysis
- Uses `causality prove` for ZK proof generation

### With Causality Core
- Leverages linear resource management
- Integrates with effect system
- Uses session types for communication

### With Causality ZK
- Generates privacy-preserving proofs
- Validates circuit constraints
- Manages witness generation

## Current Status

### Working Components ✅
- **Step 1**: OCaml scenario compilation and Lisp → IR conversion
- **Individual OCaml scenarios**: Full execution with mock DeFi operations
- **Cross-language integration**: OCaml ↔ Rust CLI tool interaction

### In Development 🚧
- **Steps 2-5**: Require CLI commands (`simulate`, `prove`) not yet implemented
- **Real CLI Integration**: Waiting for extended CLI command set
- **ZK Proof Pipeline**: Requires causality prove command
- **Multi-chain simulation**: Requires causality simulate command

### Test Results
```bash
# ✅ This works - Individual scenario
dune exec ./bridge_vault_scenario.exe

# 🚧 This partially works - E2E harness (Step 1 only)
dune exec ./bridge_vault_e2e.exe
```

## Development Notes

### Mock Implementation
The current implementation uses mock responses for:
- Blockchain interactions (no real chain calls)
- Vault APY calculations (hardcoded rates)
- ZK proof generation (simulated proofs)
- Gas price estimates (fixed values)

### Future Enhancements
- **Real Chain Integration**: Connect to actual blockchain networks
- **Live Vault Data**: Real-time APY and strategy data
- **Production ZK**: Actual zero-knowledge proof generation
- **Advanced Scenarios**: More complex DeFi workflows

## Troubleshooting

### OCaml Compilation Issues
```bash
# Check OCaml installation
ocaml --version
dune --version

# Clean and rebuild
dune clean
dune build
```

### Causality CLI Issues
```bash
# Verify CLI is built
cargo build --bin causality

# Check CLI help
./target/debug/causality --help
```

### Missing Dependencies
```bash
# Rebuild all dependencies
nix develop
cargo build --all
```

This OCaml harness demonstrates the power of Causality's cross-language integration capabilities while testing complex real-world DeFi scenarios with privacy and compliance requirements. 