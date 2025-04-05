# Error Handling Best Practices

This document outlines the best practices for error handling in the Causality project. Following these guidelines will ensure consistent error handling across the codebase.

## Core Principles

1. **Domain-Specific Errors**: Each crate should define its own error types that implement the `CausalityError` trait.
2. **Consistent Error Codes**: Use the standardized error codes organized by domain.
3. **Rich Context**: Provide detailed error messages and context information.
4. **Error Conversion**: Implement conversion traits to allow seamless error propagation.
5. **Type Safety**: Use explicit error types for internal APIs and boxed errors for public APIs.

## Defining Error Types

Each crate should define its own error types in a dedicated module (typically `error.rs`):

```rust
use thiserror::Error;
use causality_error::{CausalityError, ErrorCode, ErrorDomain};

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

## Error Codes

Error codes should follow this pattern:

- **1xxx**: Core errors (general Causality system)
- **2xxx**: Types errors (data type related)
- **3xxx**: Crypto errors (cryptography related)
- **4xxx**: Storage errors (persistence related)
- **5xxx**: Engine errors (execution related)
- **6xxx**: Network errors (communication related)
- **7xxx**: Domain errors (domain specific)
- **8xxx**: External errors (integrations with external systems)

## Using Result Types

For public APIs, use the centralized `Result` type:

```rust
use causality_error::Result;

pub fn my_function() -> Result<String> {
    // Implementation...
}
```

For internal functions with specific error types:

```rust
pub fn internal_function() -> std::result::Result<String, MyDomainError> {
    // Implementation...
}
```

## Error Conversion

Use the provided conversion traits to convert between error types:

```rust
use causality_error::IntoBoxError;

fn process_data() -> Result<()> {
    let result = std::fs::read_to_string("file.txt")
        .map_err(|e| e.into_box_error())?;
    
    Ok(())
}
```

## Using Error Macros

Use the provided macros for common error handling patterns:

```rust
use causality_error::{error, try_result, try_option, ErrorDomain, ErrorCode};

fn validate_input(input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(error!(ErrorDomain::Core, ErrorCode(1001), "Input cannot be empty"));
    }
    
    let file = try_option!(std::fs::metadata("config.json").ok(), 
        "Config file not found");
    
    let content = try_result!(std::fs::read_to_string("config.json"));
    
    Ok(())
}
```

## Error Context

Provide contextual information in error messages to aid debugging:

```rust
fn process_transaction(tx_id: &str) -> Result<()> {
    let transaction = fetch_transaction(tx_id)
        .map_err(|e| error!(
            ErrorDomain::Domain, 
            ErrorCode(7001), 
            format!("Failed to fetch transaction {}: {}", tx_id, e)
        ))?;
    
    // Process transaction...
    Ok(())
}
```

## Error Propagation

Use the `?` operator for error propagation:

```rust
fn process_data() -> Result<String> {
    let file = open_file("data.txt")?;
    let content = read_content(file)?;
    let processed = process_content(content)?;
    Ok(processed)
}
```

## Testing Errors

Write tests to verify error handling logic:

```rust
#[test]
fn test_error_handling() {
    let result = process_invalid_input();
    assert!(result.is_err());
    
    if let Err(err) = result {
        assert_eq!(err.code(), ErrorCode(1001));
        assert_eq!(err.domain(), ErrorDomain::Core);
    }
}
```

## Best Practices Summary

1. Define domain-specific error types using `thiserror`.
2. Implement the `CausalityError` trait for all error types.
3. Use consistent error codes that follow the established pattern.
4. Provide rich context in error messages.
5. Use conversion traits to seamlessly convert between error types.
6. Use macros for common error handling patterns.
7. Write tests to verify error handling logic.
8. Document expected errors in function documentation. 