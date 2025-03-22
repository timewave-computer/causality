# Fact System Migration Plan

This document outlines the specific steps required to complete Phase 5 of the fact management implementation plan - replacing the old fact system with the new `FactType`-based system.

## Overview

The migration involves:

1. Replacing prototype adapter implementations with production versions
2. Updating all domain adapters to use the new fact system exclusively
3. Removing all usage of old fact types and interfaces
4. Integrating the fact system with the effect system

## Migration Steps

### 1. Domain Adapter Replacements

- [ ] **Replace the Domain Adapter Trait**
  - Move `src/domain/adapter_next.rs` to replace `src/domain/adapter.rs`
  - Update all imports to reference the new trait
  - Run tests to verify basic functionality

- [ ] **Replace the EVM Adapter Implementation**
  - Move `src/domain_adapters/evm/adapter_next.rs` to replace `src/domain_adapters/evm/adapter.rs`
  - Update all EVM adapter imports
  - Test EVM adapter functionality

### 2. Update Other Domain Adapters

- [ ] **Update Bitcoin Adapter**
  - Modify the Bitcoin adapter to return `FactType` instead of `ObservedFact`
  - Implement proper pattern matching for Bitcoin-specific facts
  - Add support for Bitcoin register facts if applicable

- [ ] **Update Solana Adapter**
  - Modify the Solana adapter to return `FactType` instead of `ObservedFact`
  - Implement proper pattern matching for Solana-specific facts
  - Add support for Solana register facts if applicable

- [ ] **Update Other Domain Adapters**
  - Apply similar changes to all remaining domain adapters

### 3. Remove Old Fact Code

- [ ] **Remove Old Fact Interfaces**
  - Delete `ObservedFact` struct and implementation
  - Delete `VerifiedFact` struct and implementation
  - Delete old fact observer interfaces
  - Delete old fact verifier interfaces
  - Remove any bridge code once migration is complete

- [ ] **Update Tests**
  - Update all test code to use `FactType` instead of `ObservedFact`/`VerifiedFact`
  - Add tests for new fact type functionality
  - Verify all existing tests pass with the new implementation

### 4. Effect System Integration

- [ ] **Update Effect Dependencies**
  - Modify effects to use `FactType` for dependencies
  - Update the effect fact dependency tracking mechanism
  - Update effect tests to use the new fact types

- [ ] **Resource Manager Integration**
  - Ensure the resource manager emits proper `FactType` facts
  - Update resource manager to interact with the new fact system

- [ ] **Replay and Simulation Integration**
  - Update replay engine to use only new fact types
  - Update simulators to work with new fact system
  - Test fact replay functionality

## Implementation Approach

To minimize disruption, we'll follow this approach:

1. Use the script `scripts/fact_system_migration.sh` to identify old fact code
2. Implement changes in a feature branch
3. Test each component after migration
4. Gradually phase out old code once all consumers are migrated

## Testing Strategy

For each part of the migration:

1. Write tests for the new implementation
2. Verify existing tests pass with new implementation
3. Run integration tests to ensure system behavior is unchanged
4. Add new tests for features only available in the new system

## Rollback Plan

If issues are encountered:

1. Keep old fact code temporarily while resolving issues
2. Use feature flags to toggle between old and new implementations
3. Develop a comprehensive test suite before complete removal

## Timeline

The migration will be completed in three phases:

1. **Week 1**: Replace main adapter interfaces and EVM implementation
2. **Week 2**: Update other domain adapters and start removing old code
3. **Week 3**: Complete effect system integration and cleanup

## Conclusion

This migration plan provides a structured approach to completing Phase 5 of the fact management implementation. By following these steps, we'll ensure a smooth transition to the new `FactType`-based fact system while maintaining backward compatibility during the migration period. 