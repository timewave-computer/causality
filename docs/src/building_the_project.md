# Building and Running the Project

This document provides instructions for building the project and running the examples.

## Prerequisites

- Nix package manager (with flakes enabled)
- Rust toolchain (installed via Nix)

## Building with Nix

The project uses a Nix flake for reproducible builds. To build the project:

```bash
# Clone the repository
git clone https://github.com/yourusername/causality.git
cd causality

# Enter the Nix shell
nix develop

# Build the project
cargo build
```

## Running the Examples

This project includes various examples that demonstrate the functionality of the Causality system.

### Program Account Serialization Example

This example demonstrates how to transform a program account to a UI view, serialize it to JSON, and deserialize it back:

```bash
# Inside the Nix shell
cargo run --example program_account_serialization
```

The example will:
1. Create a mock asset account
2. Transform it to a UI view
3. Serialize the view to JSON
4. Write the JSON to a file
5. Read the JSON back and deserialize it
6. Verify the deserialized view matches the original

The output JSON file will be saved as `example_account.json` in the current directory.

### Program Account API Example

This example demonstrates how to use the UI models in a web service context using the actix-web framework:

```bash
# Inside the Nix shell
cargo run --example program_account_api
```

This will start a web server on `localhost:8080` with the following endpoints:
- `GET /accounts` - Returns a list of all accounts
- `GET /accounts/{id}` - Returns details for a specific account

You can test the API using curl:

```bash
curl http://localhost:8080/accounts
curl http://localhost:8080/accounts/123
```

### Resource Capability Example

This example demonstrates the capability-based resource API for secure access control:

```bash
nix develop
cargo run --example resource_capability
```

The example shows:
1. Creating resources with owner capabilities
2. Delegating read-only access to other users
3. Access control enforcement (preventing unauthorized writes)
4. Revoking capabilities
5. Querying resources based on criteria

## Running Tests

To run the test suite:

```bash
# Inside the Nix shell
cargo test
```

To run tests for the program account UI models specifically:

```bash
# Inside the Nix shell
cargo test program_account::tests::ui_tests
```

## Building Documentation

To generate and view the documentation:

```bash
# Inside the Nix shell
cargo doc --no-deps --open
``` 