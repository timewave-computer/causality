# Causality Compiler

Advanced compilation pipeline transforming high-level Lisp expressions and effects into optimized register machine instructions with comprehensive analysis and zero-knowledge circuit compatibility.

## Core Pipeline

```
Source Code → Analysis → Multi-Layer Compilation → Optimization → Artifacts
```

### Analysis Phase
- **Type Inference**: Linear type checking with constraint solving
- **Linearity Analysis**: Resource usage pattern verification  
- **Effect Analysis**: Effect dependency and capability analysis
- **Semantic Validation**: Program correctness verification

### Compilation Phase
- **Layer 2 → Layer 1**: Effect algebra to lambda calculus
- **Layer 1 → Layer 0**: Lambda expressions to register instructions
- **Symbol Resolution**: Complete reference resolution
- **Code Generation**: Optimized instruction sequences

### Optimization Engine
- **Cross-Layer**: Optimization across architectural boundaries
- **Instruction-Level**: Dead code elimination, constant folding
- **Control Flow**: Branch optimization, tail call optimization
- **Data Flow**: Register allocation, value propagation

## Core Components

- **Pipeline** (`pipeline.rs`): Multi-stage compilation orchestration
- **Checker** (`checker.rs`): Comprehensive semantic analysis
- **Artifact** (`artifact.rs`): Content-addressed compilation artifacts
- **ZK Compiler** (`zk_compiler.rs`): Circuit generation support

## Advanced Features

### Content-Addressed Artifacts
- Deterministic artifact identification by content hash
- Incremental compilation with dependency tracking
- Intelligent caching for build performance

### Storage Integration
- **Almanac Schema**: Structured metadata management
- **Event Storage**: Compilation event tracking
- **State Analysis**: Program state flow analysis

### Backend Integration  
- **Valence Coprocessor**: Production ZK proving integration
- **Traverse Integration**: Cross-chain deployment support
- **Storage Backends**: Multiple storage system support

## Performance

- **Incremental Builds**: Smart recompilation based on content changes
- **Parallel Compilation**: Multi-threaded processing
- **Cache Coherence**: Efficient artifact caching
- **Optimization Metrics**: Detailed performance tracking
