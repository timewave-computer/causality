# TEL Runtime Integration Guide

This guide covers how to integrate the Temporal Effect Language (TEL) runtime into your applications and how to leverage its capabilities effectively.

## Runtime Overview

The TEL runtime provides the execution environment for TEL programs and manages their interaction with host applications. It offers a comprehensive set of features:

- Program loading and execution
- Resource management and storage
- Query processing
- Host language integration
- Foreign function interface (FFI)

## Integrating TEL in Your Application

### Setting Up the Runtime

To integrate TEL into your application, first initialize the runtime with appropriate configuration:

```rust
use causality_tel::runtime::{TelRuntime, TelRuntimeConfig};

// Create runtime configuration
let config = TelRuntimeConfig::new()
    .with_resource_limits(true)
    .with_memory_limit(1024 * 1024 * 100) // 100 MB
    .with_execution_timeout(30000) // 30 seconds
    .with_max_query_results(1000);

// Initialize the runtime
let runtime = TelRuntime::new(config);
```

### Loading and Executing Programs

```rust
use causality_tel::runtime::Program;
use std::fs;

// Load a TEL program from a file
let program_source = fs::read_to_string("examples/basic/hello_world.tel")?;
let program = runtime.load_program(&program_source)?;

// Execute the program
let result = runtime.execute(&program)?;
println!("Program result: {:?}", result);
```

### Managing Resources

```rust
use causality_tel::resource::{Resource, ResourceType, FieldType};

// Define a resource type
let user_type = ResourceType::new("User")
    .add_field("id", FieldType::String, true)
    .add_field("name", FieldType::String, true)
    .add_field("email", FieldType::String, true);

// Register the resource type with the runtime
runtime.register_resource_type(user_type);

// Create and store a resource
let user = json!({
    "id": "user-1",
    "name": "John Doe",
    "email": "john@example.com"
});

let content_id = runtime.store_resource("User", &user)?;
println!("Stored user with content ID: {}", content_id);

// Retrieve a resource
let loaded_user = runtime.load_resource(&content_id)?;
```

### Working with Queries

```rust
use causality_tel::runtime::Query;

// Build a query
let query = Query::new()
    .from_resource_type("User")
    .where_field("email", "LIKE", "%@example.com")
    .order_by("name", "ASC")
    .limit(10);

// Execute the query
let results = runtime.execute_query(&query)?;

// Process results
for resource in results {
    println!("Found user: {}", resource["name"]);
}
```

## Host Language Integration

### Registering Host Functions

TEL programs can call functions in the host language. Register these functions with the runtime:

```rust
// Register a host function that can be called from TEL
runtime.register_host_function("getCurrentTime", |_args| {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(json!(now))
});

// Register a host function that takes arguments
runtime.register_host_function("add", |args| {
    if args.len() != 2 {
        return Err("Expected exactly 2 arguments".into());
    }
    
    let a = args[0].as_f64().ok_or("First argument must be a number")?;
    let b = args[1].as_f64().ok_or("Second argument must be a number")?;
    
    Ok(json!(a + b))
});
```

### Calling TEL Functions from Host

TEL programs can export functions for the host to call:

```rust
// Get a function exported by the TEL program
let add_function = runtime.get_exported_function(&program, "addNumbers")?;

// Call the TEL function from the host
let args = vec![json!(5), json!(7)];
let result = runtime.call_function(&add_function, &args)?;
println!("Result of TEL function: {}", result); // Output: 12
```

## Foreign Function Interface (FFI)

TEL provides FFI capabilities to interact with different programming languages.

### C/C++ Integration

```c
#include "tel_ffi.h"

// Initialize the TEL runtime
TelRuntime* runtime = tel_runtime_new();

// Load and execute a TEL program
const char* program_source = "let x = 42; x";
TelProgram* program = tel_load_program(runtime, program_source);
TelValue* result = tel_execute(runtime, program);

// Get the result value
int value = tel_value_as_int(result);
printf("Program result: %d\n", value);

// Clean up
tel_value_free(result);
tel_program_free(program);
tel_runtime_free(runtime);
```

### Python Integration

```python
import tel

# Initialize the TEL runtime
runtime = tel.Runtime()

# Load and execute a TEL program
program_source = 'let greeting = "Hello, Python!"; greeting'
program = runtime.load_program(program_source)
result = runtime.execute(program)

print(f"Program result: {result}")

# Register a Python function to be called from TEL
def multiply(a, b):
    return a * b

runtime.register_host_function("multiply", multiply)

# Execute a program that calls the Python function
program_source = 'Host.call("multiply", 6, 7)'
program = runtime.load_program(program_source)
result = runtime.execute(program)

print(f"Multiply result: {result}")  # Output: 42
```

### JavaScript/Node.js Integration

```javascript
const tel = require('tel-js');

// Initialize the TEL runtime
const runtime = new tel.Runtime();

// Load and execute a TEL program
const programSource = 'let greeting = "Hello, JavaScript!"; greeting';
const program = runtime.loadProgram(programSource);
const result = runtime.execute(program);

console.log(`Program result: ${result}`);

// Register a JavaScript function to be called from TEL
runtime.registerHostFunction("fetchData", async (url) => {
  const response = await fetch(url);
  return response.json();
});

// Execute a program that calls the JavaScript function
const dataProgram = runtime.loadProgram('Host.call("fetchData", "https://api.example.com/data")');
runtime.execute(dataProgram).then(result => {
  console.log("Fetched data:", result);
});
```

## Advanced Runtime Features

### Concurrency and Parallelism

TEL runtime supports concurrent and parallel execution of programs:

```rust
use causality_tel::runtime::ExecutionStrategy;

// Configure the runtime for parallel execution
let config = TelRuntimeConfig::new()
    .with_execution_strategy(ExecutionStrategy::Parallel)
    .with_thread_pool_size(num_cpus::get());

let runtime = TelRuntime::new(config);

// Execute multiple programs concurrently
let handles: Vec<_> = programs.into_iter().map(|program| {
    let runtime_clone = runtime.clone();
    std::thread::spawn(move || {
        runtime_clone.execute(&program)
    })
}).collect();

// Collect results
let results: Vec<_> = handles.into_iter()
    .map(|handle| handle.join().unwrap())
    .collect();
```

### Caching and Optimization

Enable caching to improve performance when executing similar programs repeatedly:

```rust
// Enable result caching
let config = TelRuntimeConfig::new()
    .with_result_caching(true)
    .with_cache_size(1000);

let runtime = TelRuntime::new(config);

// Cached results will be retrieved without re-execution for identical programs
let result1 = runtime.execute(&program)?;
let result2 = runtime.execute(&program)?; // Uses cached result
```

### Resource Pooling

For high-performance applications, resource pooling can significantly improve efficiency:

```rust
// Enable memory pooling
let config = TelRuntimeConfig::new()
    .with_memory_pooling(true)
    .with_pool_size(100);

let runtime = TelRuntime::new(config);
```

### Custom Storage Backends

TEL supports custom storage backends for resource persistence:

```rust
use causality_tel::runtime::StorageBackend;

// Implement a custom storage backend
struct MyCustomStorage { /* ... */ }

impl StorageBackend for MyCustomStorage {
    // Implement required methods
    fn store(&self, resource_type: &str, data: &Value) -> Result<ContentId, TelError> {
        // Custom storage logic
    }
    
    fn load(&self, content_id: &ContentId) -> Result<Value, TelError> {
        // Custom retrieval logic
    }
    
    // Other required methods...
}

// Use the custom storage backend
let storage = MyCustomStorage::new();
let config = TelRuntimeConfig::new().with_storage_backend(Box::new(storage));
let runtime = TelRuntime::new(config);
```

## Error Handling and Debugging

### Runtime Errors

TEL provides a comprehensive error system:

```rust
match runtime.execute(&program) {
    Ok(result) => println!("Success: {:?}", result),
    Err(error) => match error {
        TelError::Syntax(details) => println!("Syntax error: {}", details),
        TelError::Runtime(details) => println!("Runtime error: {}", details),
        TelError::ResourceNotFound(id) => println!("Resource not found: {}", id),
        TelError::ValidationFailed(results) => {
            println!("Validation failed:");
            for result in results {
                println!("  - {}: {}", result.rule_id, result.message);
            }
        },
        _ => println!("Other error: {:?}", error),
    }
}
```

### Debugging Tools

TEL includes tools for debugging programs:

```rust
// Enable debug mode
let config = TelRuntimeConfig::new().with_debug_mode(true);
let runtime = TelRuntime::new(config);

// Get execution trace
let (result, trace) = runtime.execute_with_trace(&program)?;
for entry in trace {
    println!("{}: {} (line {})", entry.timestamp, entry.operation, entry.line);
}
```

## Performance Considerations

1. **Resource Limits**: Always set appropriate resource limits to prevent runaway programs.
2. **Caching**: Use result caching for frequently executed programs.
3. **Pooling**: Enable memory pooling for high-throughput scenarios.
4. **Parallelism**: Use parallel execution strategy for computationally intensive workloads.
5. **Query Optimization**: Limit query results and use targeted queries.

## Security Considerations

1. **Isolation**: TEL programs run in an isolated environment, but consider additional sandboxing for untrusted code.
2. **Resource Limits**: Set strict resource limits to prevent denial-of-service attacks.
3. **Host Functions**: Carefully secure host functions that provide access to system resources.
4. **Input Validation**: Validate all inputs before passing them to TEL programs.

## Best Practices

1. **Error Handling**: Implement comprehensive error handling for TEL program execution.
2. **Version Control**: Track TEL programs in version control systems.
3. **Testing**: Write tests for TEL programs and their integration with host applications.
4. **Documentation**: Document the interface between TEL and host applications.
5. **Monitoring**: Set up monitoring for runtime metrics and performance.

This guide provides a solid foundation for integrating TEL into your applications. For more specific use cases or advanced features, refer to the API documentation. 