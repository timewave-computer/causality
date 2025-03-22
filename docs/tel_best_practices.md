# TEL Resource Integration Best Practices

This document outlines best practices for using the Temporal Effect Language (TEL) resource integration system effectively. Following these guidelines will help you build robust, maintainable applications.

## Resource Management

### Resource Creation and Lifecycle

- **Use Domains for Isolation**: Always assign resources to specific domains to maintain proper isolation between different parts of your application.
  ```rust
  let domain = Domain::new("finance");
  let resource_id = tel.resource_manager.create_resource(&owner, &domain, initial_data)?;
  ```

- **Prefer Builder Pattern**: Use the `TelBuilder` for creating and configuring TEL instances to ensure all components are properly initialized.
  ```rust
  let tel = TelBuilder::new()
      .with_instance_id("production-system")
      .with_snapshot_schedule(Duration::from_secs(3600), 24, true)
      .build();
  ```

- **Clean Up Resources**: Always delete resources when they're no longer needed to prevent resource leaks.
  ```rust
  // When done with a resource
  tel.resource_manager.delete_resource(&resource_id)?;
  ```

- **Use Resource Guards**: When accessing a resource, use the `ResourceGuard` to ensure proper locking and unlocking.
  ```rust
  let guard = tel.resource_manager.access_resource(&resource_id, AccessMode::ReadWrite)?;
  // Work with the resource...
  // Guard automatically unlocks when it goes out of scope
  ```

### Resource Data Structure

- **Choose Appropriate Data Types**: Use the most appropriate `RegisterContents` type for your data:
  - `Text` for human-readable data
  - `Number` for numeric values
  - `Binary` for complex or serialized data
  - `ResourceId` for references to other resources

- **Keep Resources Small**: Avoid storing large amounts of data in a single resource. Instead, split data across multiple resources with relationships.

- **Establish Clear Ownership**: Always set a clear owner for each resource to maintain proper access control.

## Versioning and Snapshots

### Version Control Best Practices

- **Create Versions at Logical Points**: Create versions before making significant changes to resources.
  ```rust
  // Before making significant changes
  let version_id = tel.version_manager.create_version(&resource_id, current_data)?;
  ```

- **Use Descriptive Version Metadata**: Add metadata to versions to make them easier to identify.
  ```rust
  tel.version_manager.create_version_with_metadata(
      &resource_id, 
      current_data, 
      "Monthly account update"
  )?;
  ```

- **Prune Old Versions**: Regularly prune old versions to save space while keeping important history points.
  ```rust
  // Keep only the last 10 versions
  tel.version_manager.prune_versions(&resource_id, None, Some(10))?;
  ```

### Snapshot Management

- **Schedule Regular Snapshots**: Set up automatic snapshots for disaster recovery.
  ```rust
  tel.snapshot_manager.configure_schedule(
      Duration::from_hours(6),  // Every 6 hours
      48,                      // Keep last 48 snapshots (12 days)
      true                     // Enable automatic snapshots
  )?;
  ```

- **Create Snapshots Before Major Operations**: Take snapshots before critical system operations.
  ```rust
  // Before major system update
  let snapshot_id = tel.snapshot_manager.create_snapshot()?;
  ```

- **Test Snapshot Restoration**: Regularly test that your snapshots can be restored correctly.
  ```rust
  // In testing environment
  tel.snapshot_manager.restore_snapshot(&snapshot_id)?;
  ```

## Effect System

### Using Effects Effectively

- **Prefer Effects Over Direct Operations**: Use the effect system instead of directly calling resource operations to ensure proper tracking and replay.
  ```rust
  // Instead of:
  // tel.resource_manager.update_resource(&resource_id, new_data)?;
  
  // Use:
  let operation = ResourceOperation::new(
      ResourceOperationType::Update {
          resource_id,
          new_data,
      }
  );
  let effect = ResourceEffect::new(operation);
  adapter.apply(effect)?;
  ```

- **Compose Related Effects**: Group related effects using `EffectComposer` to ensure they're applied atomically.
  ```rust
  let mut composer = EffectComposer::new();
  composer.add_effect(debit_effect);
  composer.add_effect(credit_effect);
  adapter.apply_sequence(composer.get_effects().to_vec())?;
  ```

- **Use Conditional Effects**: Implement business logic using conditional effects.
  ```rust
  composer.with_condition(balance >= amount, |c| {
      c.add_effect(transfer_effect);
  });
  ```

### Working with Repeating Effects

- **Set Reasonable Intervals**: Choose appropriate intervals for repeating effects based on your application's needs.

- **Always Set Max Iterations**: Set a reasonable maximum number of iterations to prevent runaway effects.
  ```rust
  let config = RepeatConfig {
      schedule: RepeatSchedule::Interval(Duration::from_secs(60)),
      max_iterations: 100,
      // Other settings...
  };
  ```

- **Monitor Repeating Effects**: Implement logging and monitoring for repeating effects to track their execution.

## Proof System

### Generating and Verifying Proofs

- **Always Verify Proofs**: Before applying effects with proofs, always verify the proofs.
  ```rust
  if verifier.verify_proof(&effect, &proof)? {
      adapter.apply(effect_with_proof)?;
  } else {
      // Handle verification failure
  }
  ```

- **Store Proof Metadata**: Include comprehensive metadata with proofs for better traceability.
  ```rust
  let metadata = EffectProofMetadata::new(
      Some(resource_id),
      creator_address,
      EffectProofFormat::Groth16
  ).with_aux_data(additional_data);
  ```

- **Choose Appropriate Proof Formats**: Use the simplest proof format that meets your security requirements.

## Performance Optimization

### Resource Access Patterns

- **Batch Related Operations**: Group related operations to minimize the number of calls.

- **Use Read-Only Access When Possible**: When you only need to read a resource, use `AccessMode::Read` to allow concurrent access.
  ```rust
  let guard = tel.resource_manager.access_resource(&resource_id, AccessMode::Read)?;
  ```

- **Minimize Resource Locks**: Keep resource locks for the minimum time necessary.

### Memory Management

- **Limit Register Size**: Keep register contents as small as practical to improve performance.

- **Use Arc for Shared Ownership**: When multiple components need access to the same TEL system, use `Arc` to share ownership without duplication.
  ```rust
  let shared_tel = Arc::new(tel);
  ```

## Testing and Debugging

### Effective Testing Strategies

- **Create Test Fixtures**: Set up fixtures with common resource types for testing.

- **Test Effect Composition**: Verify that composed effects behave as expected when applied together.

- **Simulate Failure Scenarios**: Test how your application handles proof verification failures and operation errors.

### Debugging TEL Applications

- **Check Operation Results**: Always check the `success` field of operation results.
  ```rust
  let result = adapter.apply(effect)?;
  if !result.success {
      if let Some(error) = result.error {
          println!("Effect failed: {}", error);
      }
  }
  ```

- **Use Snapshots for Debugging**: Create snapshots before problematic operations to allow easier debugging.

- **Log Important Transitions**: Add logging around key resource state transitions.

## Security Considerations

### Access Control

- **Implement Proper Authentication**: Verify the identity of users before allowing resource operations.

- **Follow Principle of Least Privilege**: Give each component only the access it needs.

- **Audit Resource Access**: Log all access to sensitive resources for security auditing.

### Proof Verification

- **Never Skip Verification**: Always verify proofs before applying effects, even in trusted environments.

- **Regularly Update Verification Keys**: Rotate verification keys periodically for better security.

- **Isolate Verification Process**: Run verification in an isolated context to prevent side-channel attacks.

## Integration with Other Systems

### Domain Adapters

- **Create Domain-Specific Wrappers**: Build domain-specific wrappers around the TEL system for cleaner interfaces.

- **Map Domain Concepts to Resources**: Create a clear mapping between your domain concepts and TEL resources.

- **Handle Concurrency**: Implement proper concurrency control when integrating with external systems.

### Error Handling

- **Use TEL Result Type**: Propagate TEL errors appropriately using the `TelResult` type.
  ```rust
  fn my_function() -> TelResult<ResourceId> {
      // Implementation
  }
  ```

- **Implement Graceful Degradation**: Design your system to handle TEL errors gracefully rather than failing completely.

- **Provide Meaningful Error Messages**: When wrapping TEL errors, include context to help with debugging.
  ```rust
  operation.map_err(|e| format!("Failed to transfer funds: {}", e))?;
  ```

## Conclusion

Following these best practices will help you build robust, maintainable applications using the TEL resource integration system. These guidelines represent accumulated experience working with TEL, but they should be adapted to your specific use case and requirements. 