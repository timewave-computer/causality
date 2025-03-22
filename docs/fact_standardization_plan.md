# Fact Standardization Completion Plan

This document outlines the remaining tasks to complete the standardization of the fact system across the Causality codebase, with a focus on completely replacing the old fact system.

## Current Status

We have successfully implemented:

1. A standardized `FactType` enum system to represent all fact types
2. Initial conversion helpers between old and new fact systems
3. An enhanced `DomainAdapter` trait as a starting point for migration
4. Updated implementation of the EVM adapter with initial support for the new system
5. Documentation of the standardization approach

## Remaining Tasks

### 1. Update Domain Adapters

- [ ] Fully migrate the EVM adapter to use the new fact types exclusively
  - [ ] Implement proper proof handling for all fact types
  - [ ] Add explicit support for register facts
  - [ ] Remove all usages of the old fact system
- [ ] Update other domain adapters to use only the standardized fact system
- [ ] Add tests for all domain adapters using the new fact types

### 2. Verify Integration Points

- [ ] Ensure `ResourceManager` fully integrates with the new fact system
- [ ] Update all `FactObserver` components to use the standardized fact types
- [ ] Migrate the fact replay engine to handle only standardized fact types
- [ ] Update the fact simulator to create only standardized facts

### 3. Remove Old Fact Implementations

- [ ] Identify all places using the old fact system
- [ ] Replace all old fact implementations with the standardized fact system
- [ ] Add tests to ensure functionality is preserved after migration
- [ ] Remove all deprecated fact interfaces after migration is complete

### 4. Integration with Effect System

- [ ] Update effect creation to use only the standardized fact system
- [ ] Ensure effects properly depend on facts using the standardized system
- [ ] Improve validation of fact dependencies in effects
- [ ] Update the replay engine to use only standardized facts

### 5. Testing and Documentation

- [ ] Create comprehensive test suite for the standardized fact system
- [ ] Update all documentation to reflect the standardized approach
- [ ] Add examples of using the standardized fact system in documentation
- [ ] Create a migration guide for adapting existing code

## Timeline and Priorities

1. **High Priority** (Complete in 1-2 weeks):
   - Update all domain adapters to use only the new system
   - Ensure proper proof handling
   - Remove old fact implementations

2. **Medium Priority** (Complete in 2-4 weeks):
   - Update all integration points
   - Remove all remaining legacy code
   - Improve documentation

3. **Low Priority** (Complete as needed):
   - Performance optimization
   - Enhanced type coverage
   - Additional tooling

## Migration Strategy

1. **Phase 1: Component Migration** (Current)
   - Start with domain adapters and work outward
   - Migrate one component at a time, ensuring tests pass
   - Use temporary conversion where needed for interfaces

2. **Phase 2: Integration** (After component migration)
   - Connect migrated components
   - Remove temporary conversion code
   - Ensure all systems work with new fact types only

3. **Phase 3: Cleanup** (Final phase)
   - Remove all old fact interfaces
   - Remove all conversion/bridge code
   - Complete verification of system integrity

## Conclusion

The fact standardization effort is focused on completely replacing the old fact system with the new standardized approach. By following this plan, we will achieve a fully standardized fact system that improves type safety, consistency, and maintainability across the Causality codebase without the technical debt of maintaining dual systems. 