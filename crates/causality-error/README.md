# Causality Error Handling

This crate provides a unified error handling framework for the Causality ecosystem. It's designed to solve several issues:

1. **Circular Dependencies**: By centralizing error handling, we avoid circular dependencies between crates
2. **Consistent Error Patterns**: Common patterns and utilities for all error types
3. **Domain-Specific Errors**: Each domain can define its own errors while maintaining compatibility
4. **Improved Error Context**: Rich error information and standardized formatting

## Core Components

- **`CausalityError` Trait**: Base trait that all error types implement
- **Error Domains**: Categorization of errors by component (Engine, Storage, Crypto, etc.)
- **Error Codes**: Unique identifiers for specific error types
- **Conversion Utilities**: Tools for converting between error types
- **Error Macros**: Simplified error creation and handling

## How to Use

### Defining Domain-Specific Errors

Each crate should define its own error types that implement the `CausalityError` trait:

```rust
use causality_error::{CausalityError, ErrorCode, ErrorDomain};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyDomainError {
    #[error("Something went wrong: {0}")]
    SomethingWrong(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl CausalityError for MyDomainError {
    fn code(&self) -> ErrorCode {
        match self {
            MyDomainError::SomethingWrong(_) => ErrorCode(8001),
            MyDomainError::InvalidOperation(_) => ErrorCode(8002),
        }
    }
    
    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Domain
    }
}
```

### Using Result Types

For public APIs, use the full Rust `Result<T, E>` with explicit error types:

```rust
use causality_error::Result;

// For public APIs that may return any error type
pub fn my_function() -> Result<String> {
    // ...
}

// For internal functions with specific error types
pub fn internal_function() -> std::result::Result<String, MyDomainError> {
    // ...
}
```

### Error Conversion

Convert between error types using the conversion traits:

```rust
use causality_error::IntoBoxError;

fn process_data() -> Result<()> {
    let result = std::fs::read_to_string("file.txt")
        .map_err(|e| e.into_box_error())?;
    
    // Or using the try_result macro
    let result = try_result!(std::fs::read_to_string("file.txt"));
    
    Ok(())
}
```

### Creating Custom Errors

Use the error macros for creating custom errors:

```rust
use causality_error::{error, ErrorDomain, ErrorCode};

fn validate_input(input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(error!(ErrorDomain::Core, ErrorCode(1001), "Input cannot be empty"));
    }
    
    Ok(())
}
```

## Best Practices

1. **Define Crate-Specific Errors**: Each crate should define its own error types
2. **Use Error Codes Consistently**: Maintain a consistent numbering scheme for error codes
3. **Provide Context**: Include relevant details in error messages
4. **Implement Conversions**: Provide conversions from your error types to common error types
5. **Use Macros for Boilerplate**: Use the provided macros to reduce error handling boilerplate
6. **Document Error Conditions**: Clearly document the error conditions for each function

## Migration Guide

When migrating existing code to use this error handling framework:

1. Identify all error types in your crate
2. Move them to a dedicated error.rs module
3. Implement the CausalityError trait for them
4. Update Result type usage to be consistent
5. Add error conversion implementations where needed
