# Test Data for Causality CLI

This directory contains test data files used by the CLI integration tests.

## Files

### `sample_proof.json`
A basic ZK proof file with minimal valid structure. Used for:
- Basic command functionality testing
- Proof file parsing validation
- Dry-run mode testing

### `complex_proof.json`
A more realistic ZK proof file with comprehensive metadata. Used for:
- Complex proof parsing
- Metadata validation
- Performance testing with larger payloads

### `invalid_proof.json`
An invalid proof file with empty/missing fields. Used for:
- Error handling testing
- Validation logic verification
- Negative test cases

## Usage in Tests

Test files are accessed using:

```rust
let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
test_data_path.push("tests/data/sample_proof.json");
```

This ensures tests can find the data files regardless of where the tests are run from.

## Adding New Test Data

When adding new test data files:

1. Place them in this directory
2. Add appropriate documentation here
3. Create corresponding tests that use the data
4. Ensure files follow the expected JSON schema

## Schema

All proof files should follow this structure:

```json
{
  "proof": "0x...",           // Hex-encoded proof data
  "public_inputs": [...],     // Array of hex-encoded public inputs
  "verification_key": "...",  // Verification key identifier
  "circuit_id": "...",        // Circuit identifier
  "metadata": {               // Additional metadata object
    "version": "...",
    "timestamp": "...",
    "prover": "..."
  }
}
```
