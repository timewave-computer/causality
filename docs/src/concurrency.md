# Causality Concurrency Patterns

This document describes the concurrency patterns implemented in the Causality system, focusing on resource-scoped concurrency and deterministic execution.

## Overview

The Causality concurrency system is designed for resource-scoped concurrency with deterministic execution. This means:

1. Resources are explicitly acquired and released
2. Concurrency is bounded by resource constraints
3. Deadlocks are detected and prevented
4. Execution is deterministic regardless of timing variations

These properties make the concurrency model suitable for both native Rust execution and compilation to RISC-V for zero-knowledge proofs.

## Core Abstractions

### ResourceId and ResourceGuard

Resources are identified by a `ResourceId`, which represents any resource in the system (e.g., an account, a transaction, etc.). Resources are acquired and released via RAII with `ResourceGuard`.

```rust
// Acquire a resource
let guard = resource_manager.acquire_resource(resource_id, "requestor").await?;

// Resource is automatically released when guard goes out of scope
drop(guard);
```

### WaitQueue

The `WaitQueue` manages resource contention with a deterministic ordering. It also provides deadlock detection to prevent programs from hanging indefinitely.

```rust
// The wait queue is used internally by the ResourceManager
let queue = WaitQueue::new();
queue.add_requestor(resource_id, "requestor", currently_owned_resources)?;
```

### TaskManager

The `TaskManager` manages concurrent tasks and their resource allocations. It tracks which tasks are waiting for which resources to facilitate deadlock detection.

```rust
let task_manager = TaskManager::new(resource_manager);
let task_id = task_manager.spawn(my_future);
let result = task_manager.join(&task_id).await?;
```

## Concurrency Primitives

### Fork/Join

The `fork` and `fork_join` primitives allow for concurrent execution of multiple tasks, collecting all results when they complete.

```rust
// Simple fork
let results = fork(vec![task1, task2, task3]).await;

// Fork with error handling
let results = fork_join(vec![task1, task2, task3]).await?;
```

### Race

The `race` primitive runs multiple tasks concurrently and returns the result of the first one to complete. There's also a `race_ok` variant that returns the first successful result.

```rust
// Basic race - returns first result
let winner = race(vec![task1, task2, task3]).await;

// Race with error handling - returns first success
let result = race_ok(vec![task1, task2, task3]).await?;
```

### Barrier

The `barrier` primitive blocks until a condition is met, possibly involving resource acquisition.

```rust
// Wait for a condition to be true
let passed = barrier(vec![resource1, resource2], || condition()).await?;
```

### Timeout

The `timeout` primitive adds a time limit to any operation.

```rust
// Run with a timeout
let result = timeout(Duration::from_secs(5), my_future).await?;

// Add a timeout to an existing future
let future_with_timeout = with_timeout(my_future, Duration::from_secs(5));
let result = future_with_timeout.await?;
```

## Common Patterns

### Resource Acquisition with Timeout

```rust
let result = timeout(
    Duration::from_secs(5),
    resource_manager.acquire_resource(resource_id, "requestor")
).await?;
```

### Parallel Resource Operations

```rust
// Create tasks for each resource
let tasks = resources.iter().map(|resource| {
    let resource = resource.clone();
    let manager = resource_manager.clone();
    async move {
        let guard = manager.acquire_resource(resource, "requestor").await?;
        // Do something with the resource
        Ok(())
    }
}).collect::<Vec<_>>();

// Run all tasks in parallel
let results = fork_join(tasks).await?;
```

### Race with Fallback

```rust
// Try primary method first, fall back to secondary
let result = race_ok(vec![
    primary_method(),
    async {
        // Wait a bit before trying secondary
        sleep(Duration::from_millis(100)).await;
        secondary_method().await
    }
]).await?;
```

### Barrier for Synchronization

```rust
// Wait for a set of resources to be in a specific state
let condition = || {
    // Check if all required conditions are met
    all_conditions_met()
};

let passed = barrier(needed_resources, condition).wait().await?;
if passed {
    // Proceed with operation
} else {
    // Handle timeout case
}
```

## Deadlock Prevention

The concurrency system automatically detects and prevents deadlocks. When a potential deadlock is detected, the operation fails with a `ResourceDeadlock` error rather than hanging indefinitely.

```rust
// This would result in a deadlock if not detected
let result = resource_manager.acquire_resource(resource1, "requestor1").await?;
// In another task:
// let result = resource_manager.acquire_resource(resource2, "requestor2").await?;
// Later:
// let result = resource_manager.acquire_resource(resource1, "requestor2").await?; // Fails with ResourceDeadlock
// let result = resource_manager.acquire_resource(resource2, "requestor1").await?; // Fails with ResourceDeadlock
```

## Best Practices

1. **Explicit Resource Management**: Always be explicit about which resources a task requires.
2. **Timeout Usage**: Use timeouts for all resource acquisitions to prevent indefinite waiting.
3. **Resource Ordering**: When acquiring multiple resources, always acquire them in a consistent order to prevent deadlocks.
4. **Minimal Resource Scope**: Release resources as soon as they're no longer needed.
5. **Error Handling**: Always handle resource errors properly, especially `ResourceDeadlock` and `Timeout` errors.
6. **Testing**: Test concurrency patterns with stress tests to ensure proper behavior under contention.

By following these patterns and best practices, you can build highly concurrent applications that remain deterministic and safe for cross-chain execution. 