# Legacy Code Removal Plan

## Overview

This document outlines the plan for removing the deprecated legacy crates (`causality-resource` and `causality-effects`) from the Causality system. These crates have been consolidated into `causality-core` as part of the architectural simplification effort described in ADR-032.

## Progress So Far

1. ✅ Created comprehensive migration mappings in `migration-mappings.json`
2. ✅ Implemented a migration utility script at `scripts/migration_util.sh`
3. ✅ Run the migration utility to update import paths across the codebase
4. ✅ Added deprecation notices to both legacy crates
5. ✅ Broke circular dependencies between legacy crates

## Next Steps

### 1. Final Import Verification

Before proceeding with crate deletion, we need to verify that there are no remaining dependencies on the legacy crates:

```bash
# Search for remaining references to causality_resource
grep -r "causality_resource::" --include="*.rs" . | grep -v "causality-resource"

# Search for remaining references to causality_effects
grep -r "causality_effects::" --include="*.rs" . | grep -v "causality-effects"

# Check for direct dependencies in Cargo.toml files
grep -r "causality-resource" --include="Cargo.toml" . | grep -v "optional" | grep -v "^#"
grep -r "causality-effects" --include="Cargo.toml" . | grep -v "optional" | grep -v "^#"
```

### 2. Update Cargo.toml Files

For each crate that still depends on the legacy crates, update the `Cargo.toml` by:

1. Removing the direct dependency
2. Ensuring they have a dependency on `causality-core` with appropriate features
3. Updating feature flags if needed

### 3. Compile and Test

After removing dependencies on the legacy crates:

1. Run `cargo check --workspace` to verify no compilation errors
2. Run `cargo test --workspace` to ensure all tests pass
3. Fix any failing tests or compilation errors

### 4. Delete Legacy Crates

Once all tests pass and compilation succeeds:

1. Delete the `causality-resource` crate:
   ```bash
   rm -rf crates/causality-resource
   ```

2. Delete the `causality-effects` crate:
   ```bash
   rm -rf crates/causality-effects
   ```

3. Update the top-level Cargo.toml to remove these crates from the workspace:
   ```toml
   # Remove these lines from [workspace.members]
   "crates/causality-resource",
   "crates/causality-effects",
   ```

### 5. Remove Legacy Interfaces

1. Search for any remaining legacy interfaces in `causality-core`:
   ```bash
   grep -r "legacy" --include="*.rs" crates/causality-core/src/
   ```

2. Remove any remaining legacy interfaces or deprecated patterns
3. Update documentation to remove references to the legacy crates

### 6. Final Verification

1. Full rebuild: `cargo build --workspace`
2. Complete test suite: `cargo test --workspace`
3. Verify the project builds in both debug and release modes

## Completion Criteria

The legacy code removal task will be considered complete when:

1. No references to `causality-resource` or `causality-effects` remain in the codebase
2. The legacy crates have been physically removed from the repository
3. All tests pass and the codebase compiles successfully
4. No backward compatibility layers remain
5. Documentation has been updated to only reference the new consolidated structure

## Timeline

- **Dependency Breaking**: Already complete
- **Import Path Migration**: Already complete
- **Final Verification**: 1 day
- **Crate Deletion**: 1 day
- **Testing and Cleanup**: 1 day

Total estimated time: 3 days 