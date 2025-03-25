<!-- Patterns for using the repository -->
<!-- Original file: docs/src/repository_usage_patterns.md -->

# Content-Addressed Code Repository: Usage Patterns

This document describes common usage patterns for the Causality content-addressed code repository system. It provides guidance on how to effectively organize, store, and retrieve code in a content-addressed manner, ensuring immutability, versioning, and safe dependency management.

## Core Principles

The content-addressed code system is built on several key principles:

1. **Content Addressing**: Code is identified by a cryptographic hash of its content.
2. **Immutability**: Once stored, code cannot be modified; any change creates a new version with a new hash.
3. **Decoupled Naming**: Human-readable names are metadata that point to content hashes.
4. **Safe Dependency Resolution**: Dependencies are resolved precisely based on content hashes.
5. **Versioning**: Multiple versions of code can coexist without conflicts.
6. **Compatibility Metadata**: Code includes metadata about compatibility constraints.

## Repository Organization

### Basic Structure

The content-addressed repository is organized as follows:

- **Objects**: Raw content-addressed code objects stored by hash
- **Names**: A registry mapping human-readable names to content hashes
- **Metadata**: Information about code objects (format, version, dependencies, etc.)

### Code Formats

The repository supports multiple code formats:

- **JSON**: Simple data or function definitions
- **RISC-V**: Executable RISC-V code for the ZK virtual machine
- **Custom Formats**: Extensible to support additional formats

## Common Usage Patterns

### 1. Storing and Retrieving Code

#### Basic Storage

```rust
// Create a repository
let repository = CodeRepository::new(path)?;

// Create a code object
let code = MyCode { /* ... */ };

// Create metadata
let metadata = CodeMetadataBuilder::new()
    .with_name(Some("my_function"))
    .with_format("json")
    .with_version("1.0.0")
    .build();

// Serialize and store
let serialized = bincode::serialize(&code)?;
let hash = repository.store_with_metadata(serialized, metadata)?;

// Register a name
repository.register_name("my_function", &hash)?;
```

#### Retrieving Code

```rust
// Retrieve by hash
let entry = repository.load_by_hash(&hash)?;
let code: MyCode = bincode::deserialize(&entry.data)?;

// Retrieve by name
let resolved_hash = repository.resolve_name("my_function")?;
let entry = repository.load_by_hash(&resolved_hash)?;
let code: MyCode = bincode::deserialize(&entry.data)?;
```

### 2. Versioning Patterns

#### Explicit Versioning

Store versions with explicit version numbers:

```rust
// Register with semantic versioning in name
repository.register_name("my_function@1.0.0", &hash_v1)?;
repository.register_name("my_function@1.1.0", &hash_v1_1)?;
repository.register_name("my_function@2.0.0", &hash_v2)?;

// Default name points to "latest" version
repository.register_name("my_function", &hash_v2)?;
```

#### Working with Multiple Versions

```rust
// Get the latest version
let latest_hash = repository.resolve_name("my_function")?;

// Get a specific version
let v1_hash = repository.resolve_name("my_function@1.0.0")?;

// Update the latest version pointer
repository.register_name("my_function", &new_hash)?;
```

### 3. Dependency Management

#### Defining Dependencies

```rust
// Create dependencies map
let mut dependencies = HashMap::new();
dependencies.insert("dependency1".to_string(), dep1_hash.to_string());
dependencies.insert("dependency2".to_string(), dep2_hash.to_string());

// Create metadata with dependencies
let metadata = CodeMetadataBuilder::new()
    .with_name(Some("main_function"))
    .with_format("json")
    .with_version("1.0.0")
    .with_dependencies(Some(dependencies))
    .build();
```

#### Resolving Dependencies

```rust
// Load a code entry
let entry = repository.load_by_hash(&hash)?;

// Get its dependencies
if let Some(deps) = entry.metadata.dependencies {
    for (name, dep_hash_str) in deps {
        let dep_hash = ContentHash::from_string(&dep_hash_str)?;
        let dep_entry = repository.load_by_hash(&dep_hash)?;
        // Use the dependency...
    }
}
```

### 4. Compatibility Checking

#### Defining Compatibility Constraints

```rust
// Create RISC-V metadata
let riscv_metadata = RiscVMetadata::new()
    .with_isa_extension("RV32I".to_string())
    .with_isa_version("2.1".to_string())
    .with_max_memory(1024 * 1024)
    .with_max_stack_depth(256)
    .with_max_instructions(10_000)
    .build();

// Create metadata with RISC-V compatibility
let metadata = CodeMetadataBuilder::new()
    .with_name(Some("risc_v_function"))
    .with_format("risc-v")
    .with_version("1.0.0")
    .with_riscv_metadata(Some(riscv_metadata))
    .build();
```

#### Checking Compatibility

```rust
// Create a compatibility checker
let checker = CompatibilityChecker::default()
    .with_version("0.2.0".to_string())
    .add_supported_format("risc-v".to_string())
    .allow_effect(EffectType::Read);

// Check compatibility
if let Err(e) = checker.check_compatibility(&entry.metadata) {
    // Handle incompatibility
    println!("Incompatible code: {}", e);
} else {
    // Code is compatible, proceed with execution
}
```

### 5. Execution Patterns

#### Basic Execution

```rust
// Create an executor
let executor = ContentAddressableExecutor::new(repository, resource_manager);

// Create a context
let context = executor.create_context("execution_context", None)?;

// Execute by name
let result = executor.execute_by_name("my_function", vec![], &context)?;

// Execute by hash
let result = executor.execute_by_hash(&hash, vec![], &context)?;
```

#### Sandboxed Execution

```rust
// Create a sandbox with strict limits
let sandbox = SecuritySandbox::default()
    .with_timeout(100)
    .with_memory_limit(1024 * 1024)
    .with_instruction_limit(1000);

// Create an executor with the sandbox
let executor = ContentAddressableExecutor::new(repository, resource_manager)
    .with_sandbox(sandbox);

// Execute with sandbox
let result = executor.execute_with_sandbox(&hash, vec![], "sandbox_context")?;
```

#### Hierarchical Execution Contexts

```rust
// Create a parent context
let parent_context = executor.create_context("parent", None)?;

// Set variables in the parent context
parent_context.set_variable("shared_var".to_string(), Value::Int(42))?;

// Create a child context
let child_context = executor.create_context(
    "child", 
    Some(Arc::clone(&parent_context))
)?;

// Child can see parent's variables
let shared_var = child_context.get_variable("shared_var");
assert_eq!(shared_var, Some(Value::Int(42)));

// Set variables in the child context
child_context.set_variable("child_var".to_string(), Value::String("hello".to_string()))?;

// Parent cannot see child's variables
let child_var = parent_context.get_variable("child_var");
assert_eq!(child_var, None);
```

### 6. Migration Patterns

#### Safe Refactoring

```rust
// Original implementation
let hash_v1 = repository.store_with_metadata(serialized_v1, metadata_v1)?;
repository.register_name("function", &hash_v1)?;

// Refactored implementation
let hash_v2 = repository.store_with_metadata(serialized_v2, metadata_v2)?;

// Register the new version
repository.register_name("function@2.0.0", &hash_v2)?;

// Test the new version
let result = executor.execute_by_hash(&hash_v2, vec![], &context)?;

// Update the default name when ready
repository.register_name("function", &hash_v2)?;
```

#### Gradual Adoption

```rust
// Create a new version with a unique name
repository.register_name("new_approach", &hash_new)?;

// Keep the old version available
repository.register_name("legacy_approach", &hash_old)?;

// Clients can choose which approach to use
let new_hash = repository.resolve_name("new_approach")?;
let legacy_hash = repository.resolve_name("legacy_approach")?;
```

### 7. RISC-V Integration

#### Storing RISC-V Code

```rust
// Create RISC-V metadata
let riscv_metadata = RiscVMetadata::new()
    .with_isa_extension("RV32I".to_string())
    .with_isa_version("2.1".to_string())
    .with_max_memory(1024 * 1024)
    .with_max_stack_depth(256)
    .with_max_instructions(10_000)
    .with_floating_point(false)
    .with_atomics(false)
    .with_mul_div(false)
    .build();

// Create code metadata
let metadata = CodeMetadataBuilder::new()
    .with_name(Some("risc_v_function"))
    .with_format("risc-v")
    .with_version("1.0.0")
    .with_riscv_metadata(Some(riscv_metadata))
    .build();

// Store the RISC-V code
let hash = repository.store_with_metadata(serialized_code, metadata)?;
```

#### RISC-V Compatibility Checking

```rust
// Create a RISC-V compatibility checker
let checker = RiscVCompatibilityChecker::default();

// Check if RISC-V code is compatible with the virtual machine
if let Some(riscv_metadata) = &entry.metadata.riscv_metadata {
    if let Err(e) = checker.check_compatibility(riscv_metadata) {
        // Handle incompatibility
        println!("Incompatible RISC-V code: {}", e);
    } else {
        // RISC-V code is compatible with the VM
    }
}
```

## Best Practices

### 1. Version Naming Conventions

Follow these naming conventions for consistent versioning:

- **Unversioned name**: Points to the "current" or "latest" version (e.g., `function_name`)
- **Versioned name**: Includes the version number (e.g., `function_name@1.2.3`)
- **Specific variant**: Includes a specific variant or configuration (e.g., `function_name:variant`)

### 2. Dependency Management

- **Pin dependencies to specific hashes**: Always reference dependencies by their content hash for perfect reproducibility.
- **Create explicit dependency maps**: Document dependencies clearly in metadata.
- **Isolate dependency chains**: Use hierarchical contexts to isolate different dependency trees.

### 3. RISC-V Compatibility

- **Include detailed RISC-V metadata**: Always specify ISA extensions, memory requirements, and other constraints.
- **Check compatibility before execution**: Verify that code is compatible with the VM before attempting to execute it.
- **Test with actual ZK virtual machine**: Validate that code works correctly in the target environment.

### 4. Metadata Practices

- **Include descriptive names**: Use clear, descriptive names that indicate the purpose of the code.
- **Document versions**: Always specify a version in metadata, even for experimental code.
- **Add detailed descriptions**: Include descriptions that explain what the code does and how to use it.

### 5. Security Considerations

- **Use sandboxed execution**: Always execute untrusted code in a sandbox with appropriate resource limits.
- **Verify effect permissions**: Check that code only uses effects it has permission to use.
- **Validate RISC-V constraints**: Ensure RISC-V code adheres to the constraints of the ZK virtual machine.

## Advanced Patterns

### 1. Code Composition

Compose multiple code fragments into a larger program:

```rust
// Store individual components
let hash1 = repository.store_with_metadata(serialized1, metadata1)?;
let hash2 = repository.store_with_metadata(serialized2, metadata2)?;

// Create a composition with dependencies on components
let mut dependencies = HashMap::new();
dependencies.insert("component1".to_string(), hash1.to_string());
dependencies.insert("component2".to_string(), hash2.to_string());

// Store the composition
let composition_metadata = CodeMetadataBuilder::new()
    .with_name(Some("composition"))
    .with_format("json")
    .with_version("1.0.0")
    .with_dependencies(Some(dependencies))
    .build();

let hash_composition = repository.store_with_metadata(serialized_composition, composition_metadata)?;
```

### 2. Effect Management

Manage effects required by code:

```rust
// Create a set of required effects
let mut required_effects = HashSet::new();
required_effects.insert(EffectType::Read);
required_effects.insert(EffectType::Write);

// Store code with effect requirements
let metadata = CodeMetadataBuilder::new()
    .with_name(Some("effect_using_function"))
    .with_format("json")
    .with_version("1.0.0")
    .with_required_effects(Some(required_effects))
    .build();

// Create an executor that allows specific effects
let sandbox = SecuritySandbox::default()
    .allow_effect(EffectType::Read)
    .allow_effect(EffectType::Write);

// Create a compatibility checker that allows specific effects
let checker = CompatibilityChecker::default()
    .allow_effect(EffectType::Read)
    .allow_effect(EffectType::Write);
```

### 3. Execution Tracing

Trace and analyze code execution:

```rust
// Execute code
executor.execute_by_name("my_function", args, &context)?;

// Get the execution trace
let trace = context.execution_trace()?;

// Analyze the trace
for event in trace {
    match event {
        ExecutionEvent::FunctionInvocation { code_hash, arguments } => {
            println!("Called function with hash {}", code_hash);
        },
        ExecutionEvent::FunctionReturn { code_hash, value } => {
            println!("Function {} returned {:?}", code_hash, value);
        },
        ExecutionEvent::EffectApplication { effect_type, details } => {
            println!("Applied effect {:?}: {}", effect_type, details);
        },
        // Process other event types...
    }
}
```

## Conclusion

The content-addressed code repository system provides a robust foundation for storing, versioning, and executing code in a deterministic and reproducible manner. By following these patterns and best practices, you can effectively manage code in the Causality system, ensuring consistency and reliability across different environments.

For more information, refer to the API documentation and the implementation details in the codebase. 