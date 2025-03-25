# Code Duplication and Complexity Analysis

## Method Name Duplication

Top repeated method names (could indicate similar functionality in different modules):

```
  8 new
  6 execute
  5 get_dependencies
  5 execute_effect
  5 create_resource
  4 with_capability
  4 release_lock
  4 lock_resource
  4 is_resource_locked
  4 get_resource
  4 get_locked_resources
  3 with_context
  3 validate
  3 register_handler
  3 record_access
  3 is_locked
  3 handle
  3 get_resource_accesses
  3 get_lock_info
  3 get_dependencies_for_target
```

## Resource Management Duplication

The effects and resource modules appear to have duplicated functionality:

### Resource Access
42 methods potentially related to resource access

### Resource Lifecycle
28 methods potentially related to resource lifecycle

### Resource Locking
39 methods potentially related to resource locking

### Resource Dependencies
31 methods potentially related to resource dependencies

## Potentially Dead Code

Approximately 48 functions may be unused or rarely used.

Top potentially unused functions:
```
register_domain_selection_handler: 1 references
register_domain_query_handler: 1 references
register_domain_transaction_handler: 1 references
register_domain_time_map_handler: 1 references
get_transaction_status: 1 references
submit_transaction: 1 references
observe_fact: 1 references
is_handler_registered: 1 references
get_domain_time: 1 references
get_domain_identifiers: 1 references
get_domain_capabilities: 1 references
get_domain_by_id: 1 references
get_all_domains: 1 references
find_domains_with_capability: 1 references
execute_domain_transaction: 1 references
```

## Implementation Complexity

Top most complex functions (by approximate line count):
```
execute,112
handle,94
execute_effect,82
new,76
create_registry,71
register_handler,68
handle_effect,62
handle_verification,57
handle_transaction,56
acquire_lock,52
with_context,47
execute,45
register_handler,44
validate,43
lock_resource,42
```

## Refactoring Targets

Based on this analysis, the following are primary targets for refactoring:

1. **Resource Management Duplication**
   - Effects and resource modules contain significant duplicated functionality
   - Consolidate or create clear abstractions between these modules

2. **Complex Implementations**
   - Several functions have high line counts, indicating complexity
   - These should be simplified, broken down, or refactored

3. **Potentially Dead Code**
   - Several public functions appear to have few or no references
   - These should be reviewed for removal or reduction to internal visibility

4. **Method Name Duplication**
   - Methods with similar names may indicate duplicated functionality
   - Consider consolidating or creating shared utility functions

5. **Domain-Specific Handlers**
   - Many domain-specific handlers show up in the potentially unused code list
   - These should be reviewed for consolidation or simplification 