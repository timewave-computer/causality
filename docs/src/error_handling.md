# Error Handling System

This document describes the error handling system used throughout the Causality project.

## Overview

The Causality error system is designed to provide consistent error handling across all crates while avoiding circular dependencies and enabling rich error information. It's implemented in the `causality-error` crate which serves as a shared dependency for all other crates.

## Core Components

### Error Traits

The system is built around the `CausalityError` trait, which standardizes how errors are handled:

```rust
pub trait CausalityError: std::error::Error + Send + Sync + 'static {
    fn code(&self) -> ErrorCode;
    fn domain(&self) -> ErrorDomain;
    fn to_error_message(&self) -> ErrorMessage { ... }
    fn error_id(&self) -> String { ... }
}
```

Additional traits expand on this functionality:

- `WithDetails`: For errors that can provide additional structured information
- `Retryable`: For errors that support retry policies
- `ErrorSource`: For components that create errors

### Error Domains

Each component of the system has its own error domain:

```rust
pub enum ErrorDomain {
    Core,       // Core system-wide errors
    Types,      // Type system errors
    Crypto,     // Cryptography errors
    Storage,    // Storage/persistence errors
    Engine,     // Execution engine errors
    Network,    // Network/communication errors
    Domain,     // Blockchain domain-specific errors
    External,   // External services/systems errors
}
```

### Error Codes

Each error type has a unique code within its domain:

```rust
pub struct ErrorCode(pub u32);
```

The convention is:
- Core errors: 1000-1999
- Types errors: 2000-2999
- Crypto errors: 3000-3999
- Storage errors: 4000-4999
- Engine errors: 5000-5999
- Network errors: 6000-6999
- Domain errors: 7000-7999
- External errors: 8000-8999

### Domain-Specific Error Types

Each component defines its own error types that implement the `CausalityError` trait:

```rust
// Example for Storage errors
pub enum StorageError {
    DatabaseError(String),
    KeyNotFound(String),
    SerializationError(String),
    // ...
}

impl CausalityError for StorageError {
    fn code(&self) -> ErrorCode { ... }
    fn domain(&self) -> ErrorDomain { ... }
}
```

### Result Type

The system provides a standardized `Result` type that uses a boxed error:

```rust
pub type Result<T> = std::result::Result<T, BoxError>;
pub type BoxError = Box<dyn CausalityError>;
```

## Error Conversion

The error system supports converting between different error types:

```rust
// Convert any error to a BoxError
pub trait IntoBoxError {
    fn into_box_error(self) -> BoxError;
}

// Automatically implement for any error that implements CausalityError
impl<E: CausalityError> IntoBoxError for E {
    fn into_box_error(self) -> BoxError {
        Box::new(self)
    }
}
```

## Macros and Utilities

The system includes macros for common error handling patterns:

```rust
// Try operation and convert error
try_box_result!(some_operation())

// Create an error with a message
make_error!(ErrorDomain::Core, ErrorCode(1001), "Something went wrong")

// Return early with an error if condition isn't met
ensure!(condition, ErrorDomain::Core, ErrorCode(1001), "Condition failed")

// Bail early with an error
bail!(ErrorDomain::Core, ErrorCode(1001), "Cannot continue")
```

And utility functions:

```rust
// Create common errors
not_found_error("resource");
permission_denied_error("not authorized");
validation_error("invalid input");
```

## Using the Error System

### Defining Domain-Specific Errors

Each crate should define its own error types:

```rust
use causality_error::{CausalityError, ErrorCode, ErrorDomain};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyDomainError {
    #[error("Something went wrong: {0}")]
    SomethingWrong(String),
}

impl CausalityError for MyDomainError {
    fn code(&self) -> ErrorCode {
        match self {
            MyDomainError::SomethingWrong(_) => ErrorCode(7001),
        }
    }
    
    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Domain
    }
}
```

### Function Return Types

For public APIs, use the `Result` type from the error crate:

```rust
use causality_error::Result;

pub fn my_function() -> Result<String> {
    // ...
}
```

For internal functions with specific error types:

```rust
use causality_error::BoxError;

fn internal_function() -> std::result::Result<String, MyDomainError> {
    // ...
}
```

### Error Conversion

Convert between error types using the conversion utilities:

```rust
use causality_error::{IntoBoxError, try_box_result};

fn process_data() -> Result<()> {
    // Convert a specific error to a BoxError
    let result = std::fs::read_to_string("file.txt")
        .map_err(|e| e.into_box_error())?;
    
    // Or use the try_box_result macro
    let result = try_box_result!(std::fs::read_to_string("file.txt"));
    
    Ok(())
}
```

## Best Practices

1. **Define Crate-Specific Errors**: Each crate should define its own error types
2. **Use Error Codes Consistently**: Follow the domain-specific error code ranges
3. **Provide Context**: Include relevant details in error messages
4. **Implement Error Conversions**: Provide conversions between your error types and BoxError
5. **Use Macros for Boilerplate**: Use the provided macros to reduce error handling boilerplate
6. **Document Error Conditions**: Clearly document error cases in function documentation

## Implementation Details

The error system is implemented in the `causality-error` crate, which contains:

- Base traits in `traits.rs`
- Domain-specific error types in `storage.rs`, `crypto.rs`, `types.rs`, etc.
- Conversion utilities in `conversion.rs`
- Error handling macros in `macros.rs`
- Common error types and utilities in `common.rs` 