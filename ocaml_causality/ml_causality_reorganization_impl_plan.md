# ML Causality Reorganization Implementation Plan

## Design Principles

**NO BACKWARDS COMPATIBILITY**: This reorganization aims for a clean, concise, and elegant implementation. We will not maintain backwards compatibility or migration paths. All deprecated code will be deleted immediately.

**Idiomatic OCaml Structure**: All OCaml files must be structured in an idiomatic, readable way with:
- Clear section dividers using comments
- Logical grouping of related functions and types
- Proper documentation comments for all public interfaces
- Consistent naming conventions
- Clean module boundaries

## Implementation Tasks

### Phase 1: Create New Directory Structure ✅ **READY TO START**

#### Task 1.1: Create base directories
- [ ] Create `lib/core/` directory
- [ ] Create `lib/lang/` directory  
- [ ] Create `lib/effects/` directory
- [ ] Create `lib/serialization/` directory
- [ ] Create `lib/interop/` directory

#### Task 1.2: Create dune files for each module
- [ ] Create `lib/core/dune` with proper module configuration
- [ ] Create `lib/lang/dune` with proper module configuration
- [ ] Create `lib/effects/dune` with proper module configuration
- [ ] Create `lib/serialization/dune` with proper module configuration
- [ ] Create `lib/interop/dune` with proper module configuration

#### Task 1.3: Create empty module files with documentation
- [ ] Create core module files with purpose comments and section dividers
- [ ] Create lang module files with purpose comments and section dividers
- [ ] Create effects module files with purpose comments and section dividers
- [ ] Create serialization module files with purpose comments and section dividers
- [ ] Create interop module files with purpose comments and section dividers

### Phase 2: Extract and Reorganize Core Types

#### Task 2.1: Split types.ml into focused modules
- [ ] Extract core Causality types → `core/types.ml`
- [ ] Extract ID and content addressing → `core/identifiers.ml`
- [ ] Extract domain logic → `core/domains.ml`
- [ ] Extract resource patterns → `core/patterns.ml`
- [ ] Structure each file with clear sections and comments

#### Task 2.2: Create public interfaces
- [ ] Create `core/types.mli` with documented public interface
- [ ] Create `core/identifiers.mli` with documented public interface
- [ ] Create `core/domains.mli` with documented public interface
- [ ] Create `core/patterns.mli` with documented public interface

#### Task 2.3: Update core module structure
- [ ] Update `core/dune` to export all modules
- [ ] Verify all types are properly exposed
- [ ] Test compilation of core module

### Phase 3: Reorganize Language Components

#### Task 3.1: Extract AST definitions
- [ ] Extract AST types from `dsl/dsl.ml` → `lang/ast.ml`
- [ ] Structure with clear type definitions section
- [ ] Add comprehensive documentation comments
- [ ] Create `lang/ast.mli` with public interface

#### Task 3.2: Extract DSL builders
- [ ] Extract DSL functions → `lang/builders.ml`
- [ ] Group functions by category with section dividers
- [ ] Add documentation for all builder functions
- [ ] Create `lang/builders.mli` with public interface

#### Task 3.3: Consolidate primitives and combinators
- [ ] Consolidate combinators → `lang/combinators.ml`
- [ ] Consolidate domain primitives → `lang/primitives.ml`
- [ ] Add validation logic → `lang/validation.ml`
- [ ] Structure each file with logical sections

#### Task 3.4: Update lang module structure
- [ ] Update `lang/dune` to export all modules
- [ ] Test compilation of lang module
- [ ] Verify all DSL functions work correctly

### Phase 4: Break Down Effect System

#### Task 4.1: Extract effect registration
- [ ] Extract effect types → `effects/effects.ml`
- [ ] Structure with registration and management sections
- [ ] Create `effects/effects.mli` with public interface
- [ ] Add comprehensive documentation

#### Task 4.2: Extract handler system
- [ ] Extract handler logic → `effects/handlers.ml`
- [ ] Extract registry logic → `effects/registry.ml`
- [ ] Structure with clear functional sections
- [ ] Create corresponding .mli files

#### Task 4.3: Extract execution components
- [ ] Extract execution logic → `effects/execution.ml`
- [ ] Extract TEL graph logic → `effects/graph.ml`
- [ ] Structure with execution flow sections
- [ ] Add detailed function documentation

#### Task 4.4: Update effects module structure
- [ ] Update `effects/dune` to export all modules
- [ ] Test compilation of effects module
- [ ] Verify effect system functionality

### Phase 5: Consolidate Serialization

#### Task 5.1: Move S-expression serialization
- [ ] Move from `types/sexpr.ml` → `serialization/sexpr.ml`
- [ ] Structure with parsing and generation sections
- [ ] Create `serialization/sexpr.mli` with public interface
- [ ] Add clear documentation comments

#### Task 5.2: Consolidate SSZ serialization
- [ ] Consolidate SSZ logic → `serialization/ssz.ml`
- [ ] Import and use separate `ocaml_ssz` module (do not inline)
- [ ] Structure with encoding/decoding sections
- [ ] Create `serialization/ssz.mli` with public interface

#### Task 5.3: Move content addressing and SMT
- [ ] Move content addressing → `serialization/content.ml`
- [ ] Move SMT implementation → `serialization/merkle.ml`
- [ ] Structure each with logical operation sections
- [ ] Create corresponding .mli files

#### Task 5.4: Update serialization module structure
- [ ] Update `serialization/dune` to export all modules and depend on ocaml_ssz
- [ ] Test compilation of serialization module
- [ ] Verify all serialization functions work

### Phase 6: Organize External Integrations

#### Task 6.1: Consolidate FFI components
- [ ] Move FFI bindings → `interop/ffi.ml`
- [ ] Structure with C bindings and Rust FFI sections
- [ ] Create `interop/ffi.mli` with public interface
- [ ] Add safety and usage documentation

#### Task 6.2: Move bridge and capability systems
- [ ] Move bridge workflows → `interop/bridges.ml`
- [ ] Move capability system → `interop/capabilities.ml`
- [ ] Structure with workflow and authorization sections
- [ ] Create corresponding .mli files

#### Task 6.3: Update interop module structure
- [ ] Update `interop/dune` to export all modules
- [ ] Test compilation of interop module
- [ ] Verify FFI and bridge functionality

### Phase 7: Update Dependencies and Tests

#### Task 7.1: Update all import statements
- [ ] Update imports in core modules
- [ ] Update imports in lang modules
- [ ] Update imports in effects modules
- [ ] Update imports in serialization modules
- [ ] Update imports in interop modules

#### Task 7.2: Reorganize test structure
- [ ] Create `test/unit/` directory structure
- [ ] Move and reorganize tests by domain
- [ ] Update test imports to use new module structure
- [ ] Structure test files with clear test sections

#### Task 7.3: Update configuration
- [ ] Update main `dune-project` file
- [ ] Update `lib/dune` to export new modules
- [ ] Update any documentation references
- [ ] Verify all builds and tests pass

### Phase 8: Code Quality and Structure

#### Task 8.1: Ensure idiomatic OCaml structure in all files
- [ ] Add clear section dividers using `(* ------------ SECTION NAME ------------ *)`
- [ ] Group related functions and types logically
- [ ] Add documentation comments for all public functions
- [ ] Ensure consistent naming conventions throughout

#### Task 8.2: Clean up and optimize
- [ ] Remove any remaining dead code
- [ ] Optimize import statements
- [ ] Ensure all .mli files are comprehensive
- [ ] Add missing documentation where needed

#### Task 8.3: Final validation
- [ ] Run `dune build` to verify compilation
- [ ] Run `dune test` to verify all tests pass
- [ ] Run `dune clean && dune build` for clean build test
- [ ] Verify no backwards compatibility code remains

### Phase 9: Cleanup and Legacy Removal

#### Task 9.1: Remove old directory structure
- [ ] Delete old `lib/types/` directory
- [ ] Delete old `lib/dsl/` directory
- [ ] Delete old `lib/ssz_bridge/` directory
- [ ] Delete old `lib/content_addressing/` directory
- [ ] Delete old `lib/smt/` directory
- [ ] Delete old `lib/capability_system/` directory
- [ ] Delete old `lib/ppx_registry/` directory
- [ ] Delete old `lib/effect_system/` directory

#### Task 9.2: Final verification
- [ ] Ensure no broken imports remain
- [ ] Verify all functionality is preserved
- [ ] Run comprehensive test suite
- [ ] Confirm clean, elegant codebase structure

## Success Criteria

1. **Clean Structure**: 5 focused modules with clear responsibilities
2. **Idiomatic Code**: All files follow OCaml best practices with proper documentation
3. **No Legacy Code**: All old structure and backwards compatibility removed
4. **Full Functionality**: All tests pass and functionality preserved
5. **Improved Maintainability**: Smaller, focused modules that are easy to understand and modify

## Notes

- Each task should be completed before moving to the next
- Test compilation frequently during the process
- Maintain comprehensive documentation throughout
- Focus on clean, elegant implementation over feature preservation
- Remove any code that doesn't serve a clear purpose 