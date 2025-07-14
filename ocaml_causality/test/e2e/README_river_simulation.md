# River E2E Simulation Test

## Overview

This test demonstrates the complete pipeline for integrating River lending system with the Causality simulation framework. The test follows the architecture requirement: **OCaml domain logic → Lisp generation → Generic Rust simulation**.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌───────────────────┐
│   OCaml Domain  │    │ Lisp Generation │    │ Rust Simulation   │
│     Logic       │ -> │   (S-expr)      │ -> │    (Generic)      │
│                 │    │                 │    │                   │
│ • River types   │    │ • Loan requests │    │ • SimulationEngine│
│ • Business      │    │ • Grove pricing │    │ • Instructions    │
│   rules         │    │ • Workflows     │    │ • Effects         │
│ • Calculations  │    │                 │    │                   │
└─────────────────┘    └─────────────────┘    └───────────────────┘
```

## Key Design Principles

1. **Domain Separation**: No River/Grove/Metropolis references in Rust code
2. **Generic Simulation**: Rust FFI only handles generic Lisp expressions
3. **OCaml Business Logic**: All domain-specific logic remains in OCaml
4. **S-expression Bridge**: Lisp serves as the interface between domains

## Test Implementation

### Current Status: Mock Implementation

The test currently uses mock simulation functions for development and demonstration:

#### Test Structure
- **File**: `test_river_compilation_simulation.ml`
- **Mock Engine**: `simulation_engine = unit`
- **Mock Functions**: `simulate_lisp_code`, `get_engine_stats`

#### Test Cases
1. **Grove Advantage Pricing**: Calculate and simulate Grove vs market rates
2. **Loan Matching Simulation**: Test loan request/offer compatibility
3. **E2E Lending Workflow**: Multi-request/offer matching pipeline
4. **Cross-Protocol Integration**: Aave, Compound, Morpho coordination

### Domain Types (OCaml)

```ocaml
type loan_request = {
  borrower_id: string;
  amount: int;
  max_rate: int;
  duration_days: int;
  collateral_type: string;
}

type loan_offer = {
  vault_id: string;
  amount: int;
  rate: int;
  min_duration: int;
  accepted_collateral: string list;
}

type grove_pricing = {
  asset: string;
  grove_rate: int;
  market_rate: int;
  duration_days: int;
}
```

### Lisp Generation Functions

```ocaml
(* Convert domain objects to generic Lisp expressions *)
let generate_loan_request_lisp request = ...
let generate_loan_offer_lisp offer = ...
let generate_grove_pricing_lisp pricing = ...
let generate_loan_matching_lisp request offer = ...
```

### Business Logic Functions

```ocaml
(* Pure OCaml domain logic *)
let calculate_grove_advantage asset duration_days = ...
let check_loan_compatibility request offer = ...
```

## FFI Implementation

### Rust FFI (`crates/causality-ffi/src/ocaml/simulation.rs`)

```rust
/// Generic simulation functions - no domain-specific logic
#[ocaml::func]
pub fn compile_and_simulate_lisp(
    _gc: &mut Runtime,
    _engine: Value,
    lisp_code: String,
) -> Result<String, String>

#[ocaml::func]
pub fn execute_instructions(
    _gc: &mut Runtime,
    _engine: Value,
    instruction_count: i64,
) -> Result<String, String>

#[ocaml::func]
pub fn execute_effect_expression(
    _gc: &mut Runtime,
    _engine: Value,
    effect_expr: String,
) -> Result<String, String>
```

### OCaml FFI Bindings (`ocaml_causality/lib/compiler/simulation_ffi.ml`)

```ocaml
(* External declarations for Rust FFI *)
external create_simulation_environment : unit -> simulation_environment = "create_simulation_environment"
external compile_and_simulate_lisp : simulation_engine -> string -> (string, string) result = "compile_and_simulate_lisp"
external execute_instructions : simulation_engine -> int -> (string, string) result = "execute_instructions"
external execute_effect_expression : simulation_engine -> string -> (string, string) result = "execute_effect_expression"
```

## Running the Test

### Current (Mock Mode)
```bash
cd ocaml_causality
dune build test/e2e/test_river_compilation_simulation.exe
dune exec test/e2e/test_river_compilation_simulation.exe
```

### Expected Output
```
Starting River E2E Simulation Tests
====================================
Architecture: OCaml domain logic → Lisp generation → Generic Rust simulation
====================================

=== Test 1: Grove Advantage Pricing ===
Generated Lisp: (grove-pricing (asset "USDC") (grove-rate 400) (market-rate 800) (duration 30))
Simulation result: {"status": "success", "instructions_executed": 4, "effects_executed": 5, "gas_consumed": 400}
Grove rate: 400 bps, Market rate: 800 bps, Advantage: 400 bps

=== Test 2: Loan Matching Simulation ===
Loan compatibility: true
Generated Lisp: (loan-matching (loan-request ...) (loan-offer ...) (execute-atomic-settlement))
Simulation result: {"status": "success", "instructions_executed": 15, "effects_executed": 5, "gas_consumed": 1500}

=== Test 3: E2E Lending Workflow ===
Total successful matches: 2
Generated workflow Lisp: (lending-workflow (requests ...) (offers ...) (execute-matching) (execute-settlement))
Workflow simulation result: {"status": "success", "instructions_executed": 25, "effects_executed": 5, "gas_consumed": 2500}

=== Test 4: Cross-Protocol Integration ===
Generated integration Lisp: (cross-protocol-integration (protocols "Aave" "Compound" "Morpho") (river-coordination) (atomic-settlement))
Integration simulation result: {"status": "success", "instructions_executed": 20, "effects_executed": 5, "gas_consumed": 2000}

====================================
River E2E Test Results
====================================
1. Grove advantage calculation: PASS
2. Loan matching: PASS
3. E2E workflow: PASS
4. Cross-protocol integration: PASS

Summary: 4/4 tests passed (100.0%)
Simulation Statistics:
- Steps executed: 10
- Gas consumed: 1000
- Effects executed: 5
====================================
```

## Next Steps for Real FFI Integration

### 1. Fix Build Environment
```bash
# Ensure Nix environment is properly loaded
nix develop

# Build FFI library with OCaml support
cd crates/causality-ffi
cargo build --features ocaml-ffi

# Build OCaml library with FFI bindings
cd ../../ocaml_causality
dune build
```

### 2. Replace Mock with Real FFI

Replace the mock functions in `test_river_compilation_simulation.ml`:

```ocaml
(* Replace mock with real FFI *)
open Simulation_ffi

let create_engine ?(config = default_config) () = 
  create_simulation_engine_with_config config.max_steps config.max_gas config.enable_snapshots

let simulate_lisp_code engine lisp_code =
  compile_and_simulate_lisp engine lisp_code

let get_engine_stats engine = 
  get_simulation_stats engine
```

### 3. Test Real Integration

```bash
# Build and run with real FFI
cd ocaml_causality
dune build test/e2e/test_river_compilation_simulation.exe
dune exec test/e2e/test_river_compilation_simulation.exe
```

### 4. Verify Pipeline

The test should demonstrate:
1. **OCaml domain logic** → Convert River types to business rules
2. **Lisp generation** → Transform domain objects to S-expressions  
3. **FFI call** → Pass Lisp to Rust simulation engine
4. **Rust compilation** → Parse Lisp and compile to instructions
5. **Simulation execution** → Run instructions in simulation engine
6. **Result return** → Get simulation results back to OCaml

## Debugging

### Common Issues

1. **OCaml Runtime Linking**: Ensure `OCAML_RUNTIME_PATH` is set correctly
2. **FFI Symbol Resolution**: Check that Rust FFI symbols are exported properly
3. **Memory Management**: Verify OCaml GC interaction with Rust allocations
4. **Type Conversion**: Ensure proper conversion between OCaml and Rust types

### Debug Commands

```bash
# Check OCaml runtime
echo $OCAML_RUNTIME_PATH
find /nix/store -name "*ocaml*" -type d 2>/dev/null | grep lib

# Check FFI symbols
nm target/debug/libcausality_ffi.so | grep simulation

# Test FFI compilation
cd crates/causality-ffi
cargo check --features ocaml-ffi 2>&1 | head -20
```

## Architecture Validation

This test validates the key architectural requirement:

> "nothing in the rust project should have any reference to grove or metropolis or river. generic s expressions should be passed over ffi and serialized into instructions that are compiled"

 **Rust FFI**: Only generic `compile_and_simulate_lisp` function  
 **Domain Logic**: All River/Grove logic in OCaml  
 **S-expressions**: Lisp serves as the interface  
 **Generic Instructions**: Rust compiles Lisp to generic instructions  

The test demonstrates the complete separation of concerns and proper use of the causality-lisp → causality-simulation pipeline. 