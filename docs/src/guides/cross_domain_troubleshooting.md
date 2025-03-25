<!-- Troubleshooting cross-domain issues -->
<!-- Original file: docs/src/cross_domain_troubleshooting.md -->

# Cross-Domain Relationship Troubleshooting

This guide provides solutions for common issues encountered when working with cross-domain relationships.

## Common Issues and Solutions

### Synchronization Failures

#### Issue: One-way synchronization not working

**Symptoms:**
- Updates in the source domain aren't reflected in the target domain
- No error messages in logs
- Synchronization status shows "Success" but no changes occur

**Possible causes and solutions:**

1. **Incorrect permission configuration**
   - Ensure the system has write permissions in the target domain
   - Check resource access rights for both domains
   
   ```rust
   // Verify resource permissions
   let permissions = domain_manager.get_permissions("target_domain", resource_id);
   println!("Resource permissions: {:?}", permissions);
   ```

2. **Transformation issues**
   - Verify data transformers are registered
   - Check for data type compatibility issues
   
   ```rust
   // Register a custom transformer if needed
   sync_manager.register_transformer(
       "source_domain", 
       "target_domain",
       MyCustomTransformer::new()
   );
   ```

3. **Domain connectivity**
   - Check network connectivity to target domain
   - Verify API keys and authentication

#### Issue: Bidirectional sync causing conflicts

**Symptoms:**
- Circular updates
- Data oscillating between different values
- Error logs showing conflict exceptions

**Solutions:**

1. **Implement conflict resolution strategy**
   ```rust
   let sync_options = SyncOptions::default()
       .with_conflict_resolution(ConflictResolution::SourceWins)
       .with_timestamp_fields(vec!["updated_at"]);
   
   sync_manager.sync_relationship(&relationship, direction, sync_options)?;
   ```

2. **Use versioning**
   ```rust
   // Enable version tracking
   let metadata = CrossDomainMetadata {
       // ... other fields
       tracking: ResourceTrackingConfig {
           track_versions: true,
           record_histories: true,
           // ...
       },
   };
   ```

3. **Add synchronization lock**
   ```rust
   // Implement sync locking
   sync_manager.with_sync_lock(|_| {
       // Perform synchronized operations
       Ok(())
   })?;
   ```

### Validation Issues

#### Issue: Strict validation failures

**Symptoms:**
- Relationships fail to validate with "Strict" level
- Error messages about missing required fields
- Inconsistent type information

**Solutions:**

1. **Switch to moderate validation temporarily**
   ```rust
   // Use less strict validation during migration
   let validation_result = validator.validate(
       &relationship, 
       ValidationLevel::Moderate
   )?;
   ```

2. **Pre-validate resources**
   ```rust
   // Ensure resources exist and are compatible
   let pre_validation = resource_validator.validate_compatibility(
       source_resource,
       target_resource
   )?;
   
   if pre_validation.is_valid {
       // Proceed with relationship creation
   }
   ```

3. **Add missing metadata**
   ```rust
   // Complete the metadata with required fields
   let enhanced_metadata = relationship.metadata.clone()
       .with_field("resource_type", "token")
       .with_field("schema_version", "1.2.0");
   
   let updated_relationship = relationship.with_metadata(enhanced_metadata);
   ```

### Scheduler Issues

#### Issue: Scheduled synchronization not running

**Symptoms:**
- Automatic synchronization doesn't occur
- Scheduler logs show no activity 
- Manual synchronization works fine

**Solutions:**

1. **Check scheduler status**
   ```rust
   // Verify scheduler is running
   let status = scheduler.get_status()?;
   if !status.is_running {
       scheduler.start()?;
   }
   ```

2. **Verify relationship sync strategy**
   ```rust
   // Ensure the relationship has a scheduled strategy
   if let SyncStrategy::Manual = relationship.metadata.sync_strategy {
       // Switch to a scheduled strategy
       let updated_metadata = relationship.metadata.clone()
           .with_sync_strategy(SyncStrategy::Periodic(
               Duration::from_secs(3600)
           ));
       
       relationship_manager.update_relationship_metadata(
           relationship.id.clone(), 
           updated_metadata
       )?;
   }
   ```

3. **Check for competing schedulers**
   ```rust
   // Check if multiple schedulers are running
   let running_schedulers = system_monitor.get_running_processes()
       .filter(|p| p.name.contains("sync_scheduler"));
   
   for scheduler in running_schedulers {
       println!("Scheduler process: {} (PID: {})", 
               scheduler.name, scheduler.pid);
   }
   ```

### Performance Issues

#### Issue: Synchronization taking too long

**Symptoms:**
- High latency between updates
- Timeouts during synchronization
- Resource contention

**Solutions:**

1. **Batch operations**
   ```rust
   // Use batch synchronization
   let batch_options = SyncOptions::default()
       .with_batch_size(100)
       .with_parallelism(4);
   
   sync_manager.sync_relationships(relationships, batch_options)?;
   ```

2. **Optimize sync frequency**
   ```rust
   // Adjust sync frequency based on update patterns
   let adaptive_strategy = SyncStrategy::Adaptive {
       min_interval: Duration::from_secs(300),    // 5 minutes minimum
       max_interval: Duration::from_secs(86400),  // 1 day maximum
       change_threshold: 0.05,                    // 5% change triggers sync
   };
   
   let optimized_metadata = relationship.metadata.clone()
       .with_sync_strategy(adaptive_strategy);
   ```

3. **Use incremental synchronization**
   ```rust
   // Enable incremental sync
   let incremental_options = SyncOptions::default()
       .with_incremental(true)
       .with_tracking_field("updated_at");
   
   sync_manager.sync_relationship(
       &relationship, 
       direction, 
       incremental_options
   )?;
   ```

## Debugging Tools

### Relationship Inspector

Use the relationship inspector to examine relationship details:

```rust
let inspector = RelationshipInspector::new();
let details = inspector.inspect(&relationship_id)?;

println!("Relationship: {:#?}", details);
println!("Last sync: {}", details.last_sync.unwrap_or_default());
println!("Sync history: {:?}", details.sync_history);
```

### Sync Tracer

Enable detailed tracing for synchronization operations:

```rust
let tracer = SyncTracer::new()
    .with_verbosity(TracingVerbosity::Detailed)
    .with_output(TracingOutput::File("sync_trace.log"));

// Wrap sync operation with tracer
tracer.trace(|| {
    sync_manager.sync_relationship(&relationship, direction, options)
})?;
```

### Validation Reporter

Generate detailed validation reports:

```rust
let reporter = ValidationReporter::new();
let report = reporter.generate_report(
    &relationship,
    ValidationLevel::Strict
)?;

// Output report to console or file
report.print();
// or
report.save_to_file("validation_report.json")?;
```

## Advanced Troubleshooting

### Enabling Debug Logging

Enable verbose logging to catch synchronization issues:

```rust
// Configure logging
causality::logging::init_with_config(LoggingConfig {
    level: log::LevelFilter::Debug,
    sync_module_level: log::LevelFilter::Trace,
    file: Some("relationship_sync.log"),
    ..Default::default()
})?;
```

### Database Inspection

For direct database inspection of relationships:

```rust
// Connect to storage and inspect raw data
let storage = relationship_manager.get_storage_access()?;
let raw_data = storage.execute_query(
    "SELECT * FROM cross_domain_relationships WHERE id = ?",
    vec![relationship_id.as_str()]
)?;

println!("Raw relationship data: {:?}", raw_data);
```

### Relationship Data Repair

For fixing corrupted relationship data:

```rust
// Create a repair tool
let repair_tool = RelationshipRepairTool::new();

// Analyze and fix issues
let repair_result = repair_tool
    .analyze(&relationship_id)?
    .fix_metadata_inconsistencies()?
    .restore_missing_links()?
    .complete_repair()?;

println!("Repair result: {:?}", repair_result);
```

## Common Error Codes

| Error Code | Description | Solution |
|------------|-------------|----------|
| REL-001 | Relationship not found | Verify relationship ID and domain access |
| REL-002 | Invalid relationship type | Use a supported relationship type or register custom type |
| REL-003 | Synchronization failed | Check domain connectivity and permissions |
| REL-004 | Validation failed | Review validation errors and fix resource compatibility |
| REL-005 | Scheduler error | Check scheduler configuration and status |
| REL-006 | Resource not found | Verify resource exists in both domains |
| REL-007 | Conflict detected | Implement conflict resolution strategy |
| REL-008 | Transformation error | Check data compatibility and register transformers |
| REL-009 | Permission denied | Verify access rights for both domains |
| REL-010 | Timeout | Optimize synchronization or increase timeout threshold |

## Getting Help

If you continue to experience issues with cross-domain relationships, consider:

1. Checking the detailed logs at `logs/relationship_sync.log`
2. Running the relationship diagnostic tool:
   ```
   causality relationship diagnose <relationship_id> --verbose
   ```
3. Creating a minimal reproduction case to isolate the issue
4. Looking for similar issues in the project issue tracker 