<!-- Migration issues for resource registers -->
<!-- Original file: docs/src/resource_register_migration_issues.md -->

# ResourceRegister Migration: Remaining Issues

This document tracks the remaining instances of legacy Resource/Register patterns that need to be migrated to the unified ResourceRegister model.

## Progress Update

**Last Updated:** Current Date

**Completed Issues:**
- ✅ Updated `src/log/entry.rs` to use ContentId instead of ResourceId
- ✅ Updated `src/effect/templates/state_transition.rs` to use ContentId instead of ResourceId
- ✅ Updated `src/program_account/base_account.rs` to use ContentId instead of RegisterId
- ✅ Renamed `synchronize_resources` to `synchronize_resource_registers` and replaced all `ResourceId` references with `ContentId` in `src/resource/resource_temporal_consistency.rs`
- ✅ Updated `src/resource/capability/validation.rs` to use ContentId instead of ResourceId
- ✅ Updated `src/resource/capability/delegation.rs` to use ContentId instead of ResourceId
- ✅ Updated `src/resource/fact_observer.rs` to use ContentId instead of RegisterId
- ✅ Renamed `transform_abstract_to_register` to `transform_abstract_to_resource_register` in `src/operation/transformation.rs`
- ✅ Renamed `to_register_id_bytes` to `to_resource_register_id_bytes` in `src/domain_adapters/evm/storage_strategy.rs`
- ✅ Renamed `from_register` to `from_resource_register` in `src/concurrency/primitives/resource_guard.rs`
- ✅ Confirmed `from_resource_register` implementation in `src/resource/archival.rs`
- ✅ Updated `src/resource/zk_integration.rs` to use UnifiedRegistry instead of OneTimeRegisterSystem
- ✅ Updated `src/resource/tel.rs` to use UnifiedRegistry instead of OneTimeRegisterSystem
- ✅ Fixed linter errors in `src/resource/zk_integration.rs` and `src/resource/tel.rs`
- ✅ Verified `Resource::new` pattern in `src/resource/content_addressed_resource.rs` - intentionally kept for backward compatibility
- ✅ Verified `ResourceRegistry::new` pattern in `src/resource/content_addressed_resource.rs` - intentionally kept for backward compatibility

**Note on Implementation**:
All migration tasks have been completed. The zk_integration.rs and tel.rs files were updated to use the unified ResourceRegister model, and separate work was done to fix linter errors in these files. The linter issues were mostly related to imports and type compatibility, and they have now been addressed.

The `Resource::new` and `ResourceRegistry::new` patterns in content_addressed_resource.rs have been verified and are intentionally maintained as part of a compatibility layer. These patterns have appropriate migration notes in the code indicating they're being phased out, and they provide automatic conversion to the new ResourceRegister model internally.

**Next Steps:**
None - Migration complete!

## Summary

Our migration to the unified ResourceRegister model is now complete. All identified legacy patterns have either been updated to use the unified model or verified as intentional compatibility layers with appropriate documentation.

## Issues By Pattern

### 1. `Resource::new` Pattern

**Status**: ✅ Verified as intentional compatibility layer
**Priority**: Low - This file is intentionally maintaining compatibility

**Files to update**:
- `src/resource/content_addressed_resource.rs` - Verified that these occurrences are part of the compatibility layer with appropriate migration notes

**Notes**: 
This file is designed to provide backward compatibility, with clear migration notes in the code. The implementations internally convert to and from ResourceRegister, providing a smooth transition path.

### 2. `ResourceRegistry::new` Pattern

**Status**: ✅ Verified as intentional compatibility layer
**Priority**: Low - This file is intentionally maintaining compatibility

**Files to update**:
- `src/resource/content_addressed_resource.rs` - Verified that these occurrences are part of the compatibility layer with appropriate migration notes

**Notes**:
Same as above - this implementation is maintained for backward compatibility and internally uses ResourceRegister for all operations.

### 3. `ResourceId::` Pattern

**Status**: ✅ Resolved
**Priority**: High

**Files to update**:
- ~~`src/effect/templates/state_transition.rs`~~ ✅ Resolved: Updated to use ContentId
- ~~`src/log/entry.rs`~~ ✅ Resolved: Updated to use ContentId
- ~~`src/resource/capability/validation.rs`~~ ✅ Resolved: Updated to use ContentId
- ~~`src/resource/capability/delegation.rs`~~ ✅ Resolved: Updated to use ContentId

**Notes**:
These instances have been updated to use ContentId directly.

### 4. `RegisterId::` Pattern

**Status**: ✅ Resolved
**Priority**: High

**Files to update**:
- ~~`src/program_account/base_account.rs`~~ ✅ Resolved: Updated to use ContentId
- ~~`src/resource/fact_observer.rs`~~ ✅ Resolved: Updated to use ContentId

**Notes**:
These have been updated to use ContentId directly.

### 5. `to_register` Pattern

**Status**: ✅ Resolved
**Priority**: Medium

**Files to update**:
- ~~`src/operation/transformation.rs`~~ ✅ Resolved: Renamed `transform_abstract_to_register` to `transform_abstract_to_resource_register`
- ~~`src/domain_adapters/evm/storage_strategy.rs`~~ ✅ Resolved: Renamed `to_register_id_bytes` to `to_resource_register_id_bytes`

**Notes**:
These functions have been renamed to work with the unified model terminology.

### 6. `from_register` Pattern

**Status**: ✅ Resolved
**Priority**: Medium

**Files to update**:
- ~~`src/concurrency/primitives/resource_guard.rs`~~ ✅ Resolved: Renamed `from_register` to `from_resource_register` with backward compatibility
- ~~`src/resource/archival.rs`~~ ✅ Resolved: Already had been updated with `from_resource_register` and backward compatibility

**Notes**:
These functions have been renamed to reflect the unified model terminology while maintaining backward compatibility.

### 7. `synchronize_resource` Pattern

**Status**: ✅ Resolved
**Priority**: Medium

**Files to update**:
- ~~`src/resource/resource_temporal_consistency.rs`~~ ✅ Resolved: Renamed to `synchronize_resource_registers`

**Notes**:
This function has been renamed to match the unified terminology.

### 8. `OneTimeRegisterSystem` Pattern

**Status**: ✅ Resolved
**Priority**: High

**Files to update**:
- ~~`src/resource/zk_integration.rs`~~ ✅ Resolved: Updated to use UnifiedRegistry and fixed linter errors
- ~~`src/resource/tel.rs`~~ ✅ Resolved: Updated to use UnifiedRegistry and fixed linter errors

**Notes**:
These integrations have been updated to use the UnifiedRegistry. Linter warnings have been addressed through proper implementation of custom types and correction of method calls.

## Tracking

| Pattern | Files Remaining | Status |
|---------|----------------|--------|
| Resource::new | 0 | ✅ Verified compatibility |
| ResourceRegistry::new | 0 | ✅ Verified compatibility |
| ResourceId:: | 0 | ✅ Complete |
| RegisterId:: | 0 | ✅ Complete |
| to_register | 0 | ✅ Complete |
| from_register | 0 | ✅ Complete |
| synchronize_resource | 0 | ✅ Complete |
| ::RegisterSystem | 0 | ✅ Complete |
| OneTimeRegisterSystem | 0 | ✅ Complete |

## Testing Plan

After addressing each item:

1. Run the `check_legacy_resource_usage.sh` script again to verify removal
2. Run the full test suite to ensure no regressions
3. Run the REPL to verify core functionality still works 