# Migration Guide: Legacy Crates to Core Implementation

This guide provides instructions for migrating from the legacy `causality-effects` and `causality-resource` crates to the consolidated core implementation in `causality-core`.

## Overview

As part of ADR12.13: Legacy Code Removal, we're consolidating functionality from several legacy crates into the new `causality-core` crate. This migration aims to:

1. Reduce code duplication
2. Simplify the architecture
3. Create a more maintainable codebase
4. Improve performance by reducing indirection

## Migration Approach

There are two approaches to migration:

### 1. Automated Migration (Recommended)

We've provided tools to automate most of the migration process:

- **Bash Script**: `scripts/migration_util.sh` - For simple find-and-replace operations
- **Rust Utility**: `migration-util` crate - For more complex refactoring

#### Using the Bash Script

```bash
# Navigate to the project root
cd /path/to/project

# Run the migration script
./scripts/migration_util.sh
```

#### Using the Rust Utility

```bash
# Navigate to the project root
cd /path/to/project

# Build the migration utility
nix develop
cargo build -p migration-util

# Run a dry run first
./target/debug/migration-util --dry-run --verbose

# If the changes look good, run without the dry-run flag
./target/debug/migration-util --verbose
```

### 2. Manual Migration

For more complex scenarios, you may need to manually update your code. Here's a table of common mappings:

| Legacy Import | New Import |
|---------------|------------|
| `causality_resource::ResourceRegister` | `causality_core::resource::Resource` |
| `causality_resource::RegisterState` | `causality_core::resource::state::ResourceState` |
| `causality_resource::RelationshipTracker` | `causality_core::resource::reference::RelationshipManager` |
| `causality_resource::StorageStrategy` | `causality_core::resource::storage::StorageStrategy` |
| `causality_effects::EffectRuntime` | `causality_core::effect::runtime::EffectRuntime` |
| `causality_effects::context::Context` | `causality_core::effect::context::Context` |
| `causality_effects::types::Effect` | `causality_core::effect::types::Effect` |
| `causality_effects::ContentHash` | `causality_core::content::ContentHash` |

For a complete list, see the `migration-mappings.json` file in the project root.

## API Changes and Compatibility

While most functionality has been preserved, there are some API changes to be aware of:

### Resource API Changes

1. `ResourceRegister` is now `Resource` with an expanded API
2. `RegisterState` is now `ResourceState` with improved state tracking
3. `RelationshipTracker` is now `RelationshipManager` with enhanced reference tracking

### Effect API Changes

1. `EffectRuntime` has an improved initialization API
2. `Context` provides more flexible context management
3. Effect handlers have unified interfaces

## Testing After Migration

After migration, ensure you:

1. Run the test suite to verify functionality: `cargo test`
2. Check for compilation warnings
3. Verify that domain-specific integrations still work correctly
4. Test cross-domain resource operations
5. Verify effect execution works with the new implementation

## Troubleshooting

### Common Issues

1. **Compilation Errors**: These are often due to API changes. Check the API documentation or consult the core implementation.

2. **Runtime Errors**: May occur due to behavioral differences. The new implementation aims to be compatible, but some edge cases may behave differently.

3. **Missing Functionality**: If you find missing functionality, check if there's an alternative approach in the new API or create an issue in the project repository.

### Getting Help

If you encounter issues during migration:

1. Check the [API documentation](https://timewave.io/causality/docs)
2. Open an issue in the project repository
3. Ask in the developer community channels

## Timeline

Legacy crates will be maintained during the transition period but will be deprecated after all core functionality is migrated. The tentative timeline is:

1. Dual maintenance period: 3 months
2. Deprecation warnings: 1 month
3. Removal of legacy crates: After 4 months total

## Contributing

If you discover issues or improvements during migration, please contribute:

1. Report issues with the migration tools
2. Submit PRs for documentation improvements
3. Help identify missing functionality

Your contributions will help make the migration process smoother for everyone. 