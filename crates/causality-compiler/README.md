# Causality Compiler

Compilation and optimization tools for the Causality Resource Model framework. This crate transforms Resource definitions, ProcessDataflowBlocks, and Lisp expressions into optimized, content-addressed artifacts for runtime execution.

## Overview

The `causality-compiler` crate provides compilation capabilities for the Causality system, including:

- **Resource Type Compilation**: Transform Resource definitions into optimized runtime representations
- **ProcessDataflowBlock Compilation**: Compile complex dataflow orchestrations into executable artifacts
- **Expression Optimization**: Optimize Lisp expressions for efficient evaluation
- **Content-Addressed Artifacts**: Generate deterministic, content-addressed compilation outputs
- **Multi-Stage Compilation**: Support for incremental and dependency-aware compilation

All compilation outputs maintain consistency with the Resource Model's SSZ-serialized, content-addressed architecture.

## Core Components

### Resource Type Compiler

Compiles Resource type definitions into optimized runtime representations:

```rust
use causality_compiler::resource::{ResourceTypeCompiler, CompilationConfig};

let compiler = ResourceTypeCompiler::new();
let config = CompilationConfig {
    optimization_level: OptimizationLevel::Release,
    target_domain: domain_id,
    enable_zk_compatibility: true,
};

let compiled_resource = compiler.compile_resource_type(
    &resource_definition,
    &config
)?;
```

### ProcessDataflowBlock Compiler

Compiles dataflow orchestrations into executable artifacts:

```rust
use causality_compiler::dataflow::{DataflowCompiler, DataflowArtifact};

let compiler = DataflowCompiler::new();
let artifact = compiler.compile_dataflow_block(
    &dataflow_definition,
    &compilation_context
)?;

let optimized_artifact = compiler.optimize_dataflow(&artifact)?;
```

### Expression Compiler

Optimizes Lisp expressions for runtime evaluation:

```rust
use causality_compiler::expr::{ExpressionCompiler, OptimizationPass};

let compiler = ExpressionCompiler::new();
let optimized_expr = compiler.compile_expression(
    &lisp_expr,
    &[OptimizationPass::ConstantFolding, OptimizationPass::DeadCodeElimination]
)?;

let expr_id = optimized_expr.content_id();
```

### Content-Addressed Compilation

All compilation outputs are content-addressed for deterministic builds:

```rust
use causality_compiler::artifacts::{CompilationArtifact, ArtifactId};

let artifact = CompilationArtifact::new(compiled_data);
let artifact_id = artifact.content_id(); // Deterministic based on content

// Store artifact with content-addressed ID
artifact_store.store(artifact_id, artifact)?;
```

## Compilation Pipeline

### Multi-Stage Compilation

The compiler supports incremental compilation with dependency tracking:

```rust
use causality_compiler::pipeline::{CompilationPipeline, Stage};

let pipeline = CompilationPipeline::new()
    .add_stage(Stage::Parse)
    .add_stage(Stage::TypeCheck)
    .add_stage(Stage::Optimize)
    .add_stage(Stage::CodeGen);

let result = pipeline.compile(&source_files)?;
```

### Dependency Resolution

Automatic dependency resolution for Resource and dataflow definitions:

```rust
use causality_compiler::deps::{DependencyResolver, DependencyGraph};

let resolver = DependencyResolver::new();
let dep_graph = resolver.resolve_dependencies(&project_sources)?;

// Compile in dependency order
for component in dep_graph.topological_order() {
    compiler.compile_component(component)?;
}
```

### Optimization Passes

Multiple optimization passes for different compilation targets:

```rust
use causality_compiler::optimization::{OptimizationPass, PassManager};

let pass_manager = PassManager::new()
    .add_pass(OptimizationPass::ConstantFolding)
    .add_pass(OptimizationPass::DeadCodeElimination)
    .add_pass(OptimizationPass::ExpressionSimplification)
    .add_pass(OptimizationPass::ZkOptimization);

let optimized = pass_manager.run_passes(&compilation_unit)?;
```

## Compilation Targets

### Runtime Target

Compile for efficient runtime execution:

```rust
use causality_compiler::targets::RuntimeTarget;

let target = RuntimeTarget::new();
let runtime_artifact = target.compile(&resource_definition)?;

// Optimized for fast evaluation
assert!(runtime_artifact.is_optimized_for_runtime());
```

### ZK Target

Compile for zero-knowledge proof generation:

```rust
use causality_compiler::targets::ZkTarget;

let target = ZkTarget::new();
let zk_artifact = target.compile(&resource_definition)?;

// Optimized for ZK circuit generation
assert!(zk_artifact.is_zk_compatible());
```

### Cross-Domain Target

Compile for cross-domain operations:

```rust
use causality_compiler::targets::CrossDomainTarget;

let target = CrossDomainTarget::new(source_domain, target_domain);
let cross_domain_artifact = target.compile(&dataflow_block)?;
```

## Configuration

Compiler configuration options:

```toml
[compiler]
optimization_level = "release"
target_architecture = "wasm32"
enable_debug_info = false
parallel_compilation = true

[compiler.optimization]
constant_folding = true
dead_code_elimination = true
expression_simplification = true
zk_optimization = true

[compiler.targets]
default_target = "runtime"
zk_target_enabled = true
cross_domain_enabled = true

[compiler.cache]
enabled = true
cache_dir = ".causality/cache"
max_cache_size = "1GB"
```

## Error Handling

Comprehensive compilation error reporting:

```rust
use causality_compiler::error::{CompilationError, ErrorContext};

match compiler.compile(&source) {
    Ok(artifact) => println!("Compilation successful"),
    Err(CompilationError::ParseError { location, message }) => {
        eprintln!("Parse error at {}: {}", location, message);
    }
    Err(CompilationError::TypeCheckError { expr_id, expected, actual }) => {
        eprintln!("Type error in {}: expected {}, got {}", expr_id, expected, actual);
    }
    Err(CompilationError::OptimizationError { pass, reason }) => {
        eprintln!("Optimization error in {}: {}", pass, reason);
    }
}
```

## Feature Flags

- **default**: Standard compilation features
- **optimization**: Advanced optimization passes
- **zk-target**: ZK proof compilation target
- **cross-domain**: Cross-domain compilation support
- **parallel**: Parallel compilation support
- **cache**: Compilation caching
