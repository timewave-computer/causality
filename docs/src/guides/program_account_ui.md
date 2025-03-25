<!-- Guide for program account UI -->
<!-- Original file: docs/src/program_account_ui_guide.md -->

# Program Account UI Models Guide

This guide explains how to use the Program Account UI models to represent program accounts in user interfaces.

## Overview

The Program Account UI models provide serializable data structures that can be used to represent program accounts and their related data in user interfaces. These models are designed to be easily serialized to JSON and consumed by frontend applications.

## Core UI Models

The following UI models are available:

- `ProgramAccountView`: The main view model that represents a program account
- `ResourceView`: Represents a resource associated with an account
- `BalanceView`: Represents an asset balance
- `CapabilityView`: Represents a capability granted to an account
- `EffectView`: Represents an effect that can be invoked
- `ParameterView`: Represents a parameter for an effect
- `TransactionView`: Represents a transaction in an account's history
- `FeeView`: Represents transaction fees

## Using the UI Models

### 1. Transforming Program Accounts to UI Views

Use the `ProgramAccountViewTransformer` to transform program accounts to UI views:

```rust
use crate::program_account::{
    ProgramAccount, ProgramAccountView, ViewTransformer, ProgramAccountViewTransformer
};

// Create a transformer
let transformer = ProgramAccountViewTransformer::new();

// Transform a program account to a view
let account: &dyn ProgramAccount = /* your program account */;
let view: ProgramAccountView = transformer.to_view(account);
```

### 2. Serializing UI Views to JSON

Use the serialization functions to convert UI views to JSON:

```rust
use crate::program_account::serialization::{to_json, to_pretty_json, write_to_file};

// Serialize to JSON string
let json = to_json(&view)?;

// Serialize to pretty-printed JSON string
let pretty_json = to_pretty_json(&view)?;

// Write to a file
write_to_file(&view, "account.json")?;
```

### 3. Deserializing JSON to UI Views

Use the deserialization functions to convert JSON back to UI views:

```rust
use crate::program_account::serialization::{from_json, read_from_file};

// Deserialize from JSON string
let view: ProgramAccountView = from_json(&json)?;

// Read from a file
let view: ProgramAccountView = read_from_file("account.json")?;
```

## Frontend Integration

The UI models are designed to be easily consumed by frontend applications. Here's an example of how to use them in a React application:

1. Define TypeScript interfaces that match the UI models
2. Fetch the JSON data from your API
3. Render the data in your UI components

See the example React component in `docs/examples/account_card.tsx` for a complete example.

## Making Custom UI Models

You can create custom UI models by extending the existing ones or creating new ones that meet your specific requirements. Just make sure they implement `Serialize` and `Deserialize` from the `serde` crate.

## Best Practices

1. **Avoid Circular References**: Ensure your UI models don't contain circular references, as these can cause issues during serialization.

2. **Include Only Necessary Data**: Only include data that is actually needed by the UI to avoid unnecessary data transfer.

3. **Use String Representations**: Use string representations for complex types (like addresses and IDs) to ensure they can be easily serialized and deserialized.

4. **Provide Metadata**: Include metadata fields to provide additional context for UI elements.

5. **Handle Missing Data Gracefully**: Design your UI to handle missing or null data gracefully, as not all fields may be available for all accounts.

## Error Handling

The serialization module provides a `SerializationError` enum that represents various errors that can occur during serialization and deserialization. Use this to handle errors in your application:

```rust
match to_json(&view) {
    Ok(json) => {
        // Use the JSON
    },
    Err(err) => match err {
        SerializationError::Json(json_err) => {
            // Handle JSON serialization error
        },
        SerializationError::Io(io_err) => {
            // Handle IO error
        },
        SerializationError::InvalidFormat(msg) => {
            // Handle invalid format error
        },
    },
}
```

## Example: Creating an API Endpoint

Here's an example of how to create an API endpoint that returns a program account as JSON:

```rust
use actix_web::{get, web, HttpResponse, Responder};
use crate::program_account::{
    ProgramAccountRegistry, ViewTransformer, ProgramAccountViewTransformer,
    serialization::to_json,
};

#[get("/api/accounts/{id}")]
async fn get_account(
    registry: web::Data<ProgramAccountRegistry>,
    path: web::Path<String>,
) -> impl Responder {
    let account_id = path.into_inner();
    
    // Get the account from the registry
    match registry.get_account(&account_id) {
        Some(account) => {
            // Transform the account to a view
            let transformer = ProgramAccountViewTransformer::new();
            let view = transformer.to_view(account.as_ref());
            
            // Serialize the view to JSON
            match to_json(&view) {
                Ok(json) => HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json),
                Err(_) => HttpResponse::InternalServerError()
                    .body("Failed to serialize account"),
            }
        },
        None => HttpResponse::NotFound()
            .body(format!("Account not found: {}", account_id)),
    }
}
```

## Conclusion

The Program Account UI models provide a flexible and efficient way to represent program accounts in user interfaces. By using these models, you can ensure consistent representation of program accounts across your application. 