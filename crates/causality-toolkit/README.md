# Causality Toolkit

High-level development toolkit providing effects system, testing utilities, and development abstractions for building applications on the three-layer architecture.

## Core Components

### Effect System (`effects/`)
Composable effect building blocks with DeFi primitives:

```rust
use causality_toolkit::effects::{EffectBuilder, defi::LiquiditySwap};

let swap_effect = LiquiditySwap::new()
    .with_input_token(token_a)
    .with_output_token(token_b)
    .with_slippage_tolerance(0.01);
```

**DeFi Effects:**
- **Liquidity Swap**: DEX integration with slippage protection
- **Lending Market**: Borrow/lend operations
- **Fungible Tokens**: Standard token operations
- **NFT Operations**: Non-fungible token handling

### Testing Framework (`testing/`)
Comprehensive testing utilities:

```rust
use causality_toolkit::testing::{TestHarness, MockGenerator};

let test_harness = TestHarness::new()
    .with_mock_executor()
    .with_property_based_testing();
```

### Mock System (`mocks/`)
Configurable mock implementations:

```rust
use causality_toolkit::mocks::{MockResource, MockBlockchain};

let mock_blockchain = MockBlockchain::new()
    .with_latency(Duration::from_millis(100))
    .with_success_rate(0.95);
```

### DSL Support (`dsl/`)
Domain-specific language for effect composition:

```rust
use causality_toolkit::dsl::{IntentBuilder, LispMacros};

let intent = IntentBuilder::new()
    .require_capability("token.transfer")
    .validate_inputs()
    .execute_transfer()
    .build();
```

### Primitives (`primitives/`)
Standard DeFi building blocks:
- **DEX**: Decentralized exchange operations
- **Fungible Token**: ERC-20 style token operations  
- **Lending Market**: Compound-style lending protocols
- **NFT**: Non-fungible token standards

## Key Features

- **Effect Composition**: Build complex workflows from primitives
- **Cross-Language Support**: Rust DSL with OCaml integration
- **Interface Synthesis**: Automatic interface generation
- **Formal Verification**: Property-based testing support
- **Fixed Point Operations**: Mathematical utilities for DeFi

## Integration Points

- **Causality Core**: Direct integration with linear resource model
- **ZK System**: Effects compile to zero-knowledge circuits  
- **Testing**: Mock implementations for rapid development
- **Examples**: Cross-chain DeFi bridge demonstrations