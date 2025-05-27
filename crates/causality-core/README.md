# Causality Core

Core business logic and implementation utilities for the Causality Resource Model framework. This crate provides concrete implementations, utility functions, and infrastructure components that support the type definitions in `causality-types`.

## Overview

The `causality-core` crate serves as the implementation layer for the Causality system, providing:

- **Content Addressing**: Utilities for SSZ-based content addressing and hashing
- **Sparse Merkle Trees (SMT)**: Implementation of authenticated data structures
- **Graph Management**: TEG (Temporal Effect Graph) state management and analysis
- **S-Expression Utilities**: Lisp integration and FFI support
- **Domain Management**: Domain configuration and boundary handling
- **Tracing and Debugging**: Execution tracing and analysis tools
- **ZK Integration**: Zero-knowledge proof utilities and state root management

All implementations maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### Content Addressing

Utilities for creating and managing content-addressed identifiers:

```rust
use causality_core::content_addressing::{ContentAddressable, content_id_from_bytes};

let data = b"resource data";
let content_id = content_id_from_bytes(data);
let resource_id = resource.content_id();
```

### Sparse Merkle Trees (SMT)

Authenticated data structures for verifiable storage:

```rust
use causality_core::smt::{SparseMerkleTree, SmtProof};

let mut smt = SparseMerkleTree::new();
smt.insert(key, value)?;

let proof = smt.generate_proof(&key)?;
let verified = proof.verify(&root, &key, &value)?;
```

#### SMT Collections

```rust
use causality_core::smt_collections::{SmtMap, SmtSet};

let mut smt_map = SmtMap::new();
smt_map.insert("key", "value")?;

let mut smt_set = SmtSet::new();
smt_set.insert("element")?;
```

### TEG State Management

Temporal Effect Graph state tracking and management:

```rust
use causality_core::teg_state_root::{TegStateRoot, StateTransition};

let mut state_root = TegStateRoot::new();
let transition = StateTransition::new(effect_id, old_state, new_state);
state_root.apply_transition(transition)?;

let root_hash = state_root.get_root_hash();
```

#### TEG Persistence

```rust
use causality_core::teg_persistence::{TegPersistence, PersistenceConfig};

let config = PersistenceConfig {
    storage_path: "teg_data".to_string(),
    compression_enabled: true,
    backup_enabled: true,
};

let persistence = TegPersistence::new(config)?;
persistence.store_teg_state(&teg_state).await?;
```

### Graph Analysis

Analysis tools for TEG structures:

```rust
use causality_core::graph_analysis::tel_graph_has_cycles;
use causality_core::graph_registry::{NodeRegistry, EdgeRegistry};

let has_cycles = tel_graph_has_cycles(&teg)?;

let mut node_registry = NodeRegistry::new();
let mut edge_registry = EdgeRegistry::new();
node_registry.register_node(node_id, node_data)?;
```

### S-Expression Integration

Lisp integration and FFI support:

```rust
use causality_core::sexpr_utils::{parse_sexpr, sexpr_to_value_expr};
use causality_core::sexpr_ffi::{sexpr_to_ocaml, sexpr_from_ocaml};

let sexpr = parse_sexpr("(+ 1 2 3)")?;
let value_expr = sexpr_to_value_expr(&sexpr)?;

let ocaml_data = sexpr_to_ocaml(&sexpr)?;
```

### Domain Management

Domain configuration and boundary handling:

```rust
use causality_core::domain::{DomainManager, DomainConfig, DomainBoundary};

let domain_config = DomainConfig {
    domain_id: domain_id,
    domain_type: DomainType::Verifiable,
    capabilities: vec!["resource.create", "resource.transfer"],
    constraints: domain_constraints,
};

let mut domain_manager = DomainManager::new();
domain_manager.register_domain(domain_config)?;
```

### Utility Functions

#### Expression Utilities

```rust
use causality_core::utils::expr::{
    compute_expr_hash, value_expr_as_string, create_value_expr_list
};

let expr_hash = compute_expr_hash(&expr);
let string_value = value_expr_as_string(&value_expr)?;
```

#### Resource Utilities

```rust
use causality_core::utils::{compute_resource_hash, create_resource, resource_id};

let resource = create_resource(value_expr, static_expr, domain_id)?;
let resource_hash = compute_resource_hash(&resource);
```

### ZK Integration

Zero-knowledge proof utilities and integration:

```rust
use causality_core::teg_zkp::{ZkProofGenerator, ZkCircuit};

let proof_generator = ZkProofGenerator::new(circuit_config);
let proof = proof_generator.generate_state_transition_proof(
    &old_state,
    &new_state,
    &transition_witness
)?;
```

### Tracing and Debugging

Execution tracing and analysis:

```rust
use causality_core::tracing::{init_tracing, TraceConfig};
use causality_core::trace_utils::{TraceCollector, ExecutionTrace};

let trace_config = TraceConfig {
    level: "debug".to_string(),
    output_format: "json".to_string(),
};
init_tracing(trace_config)?;

let mut trace_collector = TraceCollector::new();
trace_collector.start_trace("resource_validation")?;
```

## Feature Flags

- **default**: Standard core features with getrandom and std
- **std**: Standard library support
- **getrandom**: Random number generation
- **benchmarks**: Performance benchmarking utilities

## Module Structure

```
src/
├── lib.rs                    # Main library interface and re-exports
├── content_addressing.rs     # Content addressing utilities
├── smt.rs                    # Sparse Merkle Tree implementation
├── smt_collections.rs        # SMT-based collections
├── teg_state_root.rs         # TEG state root management
├── teg_persistence.rs        # TEG persistence layer
├── teg_deployment.rs         # TEG deployment utilities
├── teg_zkp.rs               # ZK proof integration
├── graph_registry.rs        # Graph node and edge management
├── graph_analysis.rs        # Graph analysis tools
├── domain.rs                # Domain management
├── sexpr_utils.rs           # S-expression utilities
├── sexpr_ffi.rs             # FFI integration
├── tracing.rs               # Tracing configuration
└── utils/                   # Utility modules
    ├── core.rs              # Core utilities
    └── serialization.rs     # Serialization utilities
```

This crate forms the implementation backbone of the Causality system, providing the concrete functionality needed to build verifiable, deterministic Resource-based applications. 