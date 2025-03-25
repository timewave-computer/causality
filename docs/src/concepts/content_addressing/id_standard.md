<!-- Standard for content IDs -->
<!-- Original file: docs/src/content_id_standard.md -->

# ContentId Standardization

## Overview

This document describes the standardization of `ContentId` usage within the Causality codebase. The purpose is to establish a single canonical definition of `ContentId` and ensure consistent imports across the project.

## Canonical Definition

The canonical definition for `ContentId` is located in:

```
src/crypto/hash.rs
```

According to our crypto-primitives rule, all cryptographic primitives should be imported from the crypto module, and content addressing is a core cryptographic concept in our system.

## Key Features of ContentId

The canonical `ContentId` implements:

- Content-based identity derived from cryptographic hash functions
- Deterministic generation from raw data
- String representation for storage and transmission
- Parsing from string format
- Support for different hash algorithms (Blake3, Poseidon)

## Importing ContentId

When using `ContentId` in your code, always import it directly from the crypto module:

```rust
use crate::crypto::hash::ContentId;
```

Do NOT import it from the types module or other locations.

## Re-export in types.rs

For backward compatibility, `ContentId` is re-exported in `types.rs`:

```rust
// Re-export ContentId from crypto::hash
pub use crate::crypto::hash::ContentId;
```

This allows existing code to continue working while we transition to the canonical imports.

## Diagram

```
┌───────────────┐              ┌───────────────┐
│ crypto/hash.rs│              │   types.rs    │
│               │◄─────────────┤               │
│ ContentId     │  re-exports  │ pub use       │
└───────────────┘              └───────────────┘
        ▲                              ▲
        │                              │
        │                              │
        │                              │
        │                              │
        │                              │
┌───────┴───────┐              ┌───────┴───────┐
│  New Code     │              │ Legacy Code   │
│               │              │               │
│ Direct Import │              │ Import via    │
│               │              │ types         │
└───────────────┘              └───────────────┘
```

## Migration Strategy

1. Use the `scripts/fix_content_id.sh` script to identify files using `ContentId` from sources other than `crypto::hash`.
2. Manually update imports across the codebase to use the canonical source.
3. Fix any double semicolons in imports (e.g., replace `use crate::crypto::hash::ContentId;;` with `use crate::crypto::hash::ContentId;`).
4. Remove any duplicate `ContentId` definitions in favor of the canonical one.

## Benefits

- Simplified mental model with a single source of truth
- Consistent behavior across the codebase
- Better adherence to the crypto-primitives rule
- Easier maintenance and updates
- Reduced risk of bugs due to inconsistent implementations

## Testing

When updating imports, ensure that:

1. All unit tests continue to pass
2. The REPL functionality works correctly
3. Any code that previously used custom `ContentId` implementations behaves the same way

## Conclusion

Standardizing on a single `ContentId` implementation improves code quality and maintainability while reducing potential bugs. All new code should import `ContentId` directly from `crypto::hash`, and we should gradually migrate existing code to do the same. 