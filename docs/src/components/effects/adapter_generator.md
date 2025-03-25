<!-- Effect adapter generator -->
<!-- Original file: docs/src/effect_adapter_generator.md -->

# Effect Adapter Generator

The Effect Adapter Generator is a tool for automatically generating adapter code based on schema definitions. It supports generating adapters in multiple languages to help integrate various domains with the Causality framework.

## Installation

The adapter generator is included as part of the Causality project. To install it, simply build the project using:

```bash
cargo build --release
```

The `adapter-gen` binary will be available in the `target/release` directory.

## Usage

```bash
adapter-gen --input <SCHEMA_FILE> --output <OUTPUT_DIR> --language <LANGUAGE> [--verbose]
```

### Arguments

- `--input`, `-i`: Path to the adapter schema file (TOML format)
- `--output`, `-o`: Directory where the generated adapter code will be written
- `--language`, `-l`: Target language for the generated code (rust, javascript, or go)
- `--verbose`, `-v`: Enable verbose output for debugging

### Example

```bash
adapter-gen --input examples/ethereum_schema.toml --output ./generated --language typescript
```

This will generate a TypeScript implementation of an Ethereum adapter based on the schema defined in `examples/ethereum_schema.toml`, and write the generated code to the `./generated` directory.

## Schema Format

Adapter schemas are defined in TOML format and describe the structure and behavior of an adapter for a specific domain. A schema includes:

- Basic metadata about the adapter
- Time synchronization configuration
- Effect definitions (operations that can be performed)
- Fact definitions (data that can be observed)
- Proof definitions (verifiable claims)
- RPC interface specifications

Here's a simple example of an adapter schema:

```toml
id = "example"
domain_type = "blockchain"
version = "0.1.0"

[common_metadata]
display_name = "Example Adapter"
description = "An example adapter schema"
author = "Causality Team"
license = "MIT"

[time_sync]
time_model = "block-based"
time_point_call = "getBlockNumber"
finality_window = 10
block_time = 15
block_based = true

[[effect_definitions]]
effect_type = "transfer"
required_fields = ["from", "to", "value"]
optional_fields = ["gas", "gasPrice"]
metadata = { gas_limit = "21000" }

[[fact_definitions]]
fact_type = "balance"
required_fields = ["address"]
optional_fields = ["block_number"]
metadata = { update_frequency = "on_demand" }

[[proof_definitions]]
proof_type = "transaction"
required_fields = ["tx_hash", "block_hash"]
metadata = { proof_format = "json" }
```

For a more comprehensive example, see the Ethereum schema in the `examples/` directory.

## Supported Languages

Currently, the adapter generator supports the following languages:

- Rust
- TypeScript

## Extending the Generator

To add support for a new language:

1. Create a new module in `src/effect_adapters/codegen/` for the language
2. Implement the `CodeGenerator` trait for the new language
3. Add templates for the language in `src/effect_adapters/codegen/templates/`
4. Update the `CodegenTarget` enum and `create_generator` function in `src/effect_adapters/codegen/mod.rs`
