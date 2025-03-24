# Resource and Register Update Pattern Report

Generated on Sun Mar 23 19:02:32 CST 2025

## Summary

### Resource Update Patterns

Found the following resource update patterns:

```rust
src//tel/resource/tracking.rs-116-    
src//tel/resource/tracking.rs-117-    /// Update a resource's state
src//tel/resource/tracking.rs:118:    pub fn update_resource(&self, state: ResourceState) -> TelResult<()> {
src//tel/resource/tracking.rs-119-        let mut resources = self.resources.write().map_err(|_| 
src//tel/resource/tracking.rs-120-            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
--
src//tel/resource/tracking.rs-312-        state.operation_history.push(operation.operation_id);
src//tel/resource/tracking.rs-313-        
src//tel/resource/tracking.rs:314:        self.update_resource(state)
src//tel/resource/tracking.rs-315-    }
src//tel/resource/tracking.rs-316-    
--
src//tel/resource/tracking.rs-368-        state.operation_history.push(operation.operation_id);
src//tel/resource/tracking.rs-369-        
src//tel/resource/tracking.rs:370:        self.update_resource(state)
src//tel/resource/tracking.rs-371-    }
src//tel/resource/tracking.rs-372-    
--
src//tel/resource/tracking.rs-428-        }
src//tel/resource/tracking.rs-429-        
src//tel/resource/tracking.rs:430:        self.update_resource(state)
src//tel/resource/tracking.rs-431-    }
src//tel/resource/tracking.rs-432-    
--
src//tel/resource/tracking.rs-481-        }
src//tel/resource/tracking.rs-482-        
src//tel/resource/tracking.rs:483:        self.update_resource(state)
src//tel/resource/tracking.rs-484-    }
src//tel/resource/tracking.rs-485-    
--
src//tel/resource/tracking.rs-512-                        state.status = ResourceStatus::Active;
src//tel/resource/tracking.rs-513-                        state.updated_at = now;
src//tel/resource/tracking.rs:514:                        let _ = self.update_resource(state);
src//tel/resource/tracking.rs-515-                    }
src//tel/resource/tracking.rs-516-                }
--
src//tel/resource/tests.rs-67-    /// Test updating a resource
src//tel/resource/tests.rs-68-    #[test]
src//tel/resource/tests.rs:69:    fn test_update_resource() {
src//tel/resource/tests.rs-70-        let manager = ResourceManager::new();
src//tel/resource/tests.rs-71-        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
--
src//tel/builder.rs-84-    
src//tel/builder.rs-85-    // Convenience constructor for resource update
src//tel/builder.rs:86:    pub fn update_resource(resource_id: ContentId, contents: ResourceContents) -> Self {
src//tel/builder.rs-87-        Effect::ResourceUpdate { resource_id, contents }
src//tel/builder.rs-88-    }
--
src//effect/transfer_effect.rs-180-                
src//effect/transfer_effect.rs-181-                // Perform the updates
src//effect/transfer_effect.rs:182:                self.resource_api.update_resource(
src//effect/transfer_effect.rs-183-                    source_capability,
src//effect/transfer_effect.rs-184-                    &self.params.source_resource_id,
--
src//effect/transfer_effect.rs-189-                ))?;
src//effect/transfer_effect.rs-190-                
src//effect/transfer_effect.rs:191:                self.resource_api.update_resource(
src//effect/transfer_effect.rs-192-                    dest_capability,
src//effect/transfer_effect.rs-193-                    &self.params.destination_resource_id,
--
src//effect/transfer_effect.rs-201-                // Non-fungible transfer (whole resource)
src//effect/transfer_effect.rs-202-                // Update the destination resource with source data
src//effect/transfer_effect.rs:203:                self.resource_api.update_resource(
src//effect/transfer_effect.rs-204-                    dest_capability,
src//effect/transfer_effect.rs-205-                    &self.params.destination_resource_id,
--
src//effect/transfer_effect.rs-211-                
src//effect/transfer_effect.rs-212-                // Clear the source resource (transfer complete)
src//effect/transfer_effect.rs:213:                self.resource_api.update_resource(
src//effect/transfer_effect.rs-214-                    source_capability,
src//effect/transfer_effect.rs-215-                    &self.params.source_resource_id,
--
src//concurrency/primitives/resource_manager.rs-312-    /// This is used internally by the resource guards to update the value
src//concurrency/primitives/resource_manager.rs-313-    /// when they release the resource.
src//concurrency/primitives/resource_manager.rs:314:    pub(crate) fn update_resource_value<T: Any + Send + Sync>(
src//concurrency/primitives/resource_manager.rs-315-        &self,
src//concurrency/primitives/resource_manager.rs-316-        id: ContentId,
--
src//resource/tests/api_tests.rs-50-
src//resource/tests/api_tests.rs-51-#[tokio::test]
src//resource/tests/api_tests.rs:52:async fn test_update_resource() {
src//resource/tests/api_tests.rs-53-    // Create addresses
src//resource/tests/api_tests.rs-54-    let admin = Address::from("admin:0x1234");
--
src//resource/tests/api_tests.rs-71-    // Update the resource
src//resource/tests/api_tests.rs-72-    let new_data = "Updated content".as_bytes().to_vec();
src//resource/tests/api_tests.rs:73:    api.update_resource(&capability, &resource_id, Some(new_data.clone()), None)
src//resource/tests/api_tests.rs-74-        .await
src//resource/tests/api_tests.rs-75-        .expect("Failed to update resource");
--
src//resource/tests/api_tests.rs-92-    update_options.metadata.insert("version".to_string(), "2.0".to_string());
src//resource/tests/api_tests.rs-93-    
src//resource/tests/api_tests.rs:94:    api.update_resource(&capability, &resource_id, None, Some(update_options))
src//resource/tests/api_tests.rs-95-        .await
src//resource/tests/api_tests.rs-96-        .expect("Failed to update resource metadata");
--
src//resource/tests/api_tests.rs-182-    // Bob should not be able to write
src//resource/tests/api_tests.rs-183-    let new_data = "Bob's modification".as_bytes().to_vec();
src//resource/tests/api_tests.rs:184:    let result = api.update_resource(&bob_capability, &resource_id, Some(new_data), None).await;
src//resource/tests/api_tests.rs-185-    
src//resource/tests/api_tests.rs-186-    assert!(result.is_err());
--
src//resource/tests/api_tests.rs-402-    // Charlie should not be able to write or delete
src//resource/tests/api_tests.rs-403-    let new_data = "Charlie's modification".as_bytes().to_vec();
src//resource/tests/api_tests.rs:404:    let result = api.update_resource(&charlie_capability, &resource_id1, Some(new_data.clone()), None).await;
src//resource/tests/api_tests.rs-405-    assert!(result.is_err());
src//resource/tests/api_tests.rs-406-    
--
src//resource/tests/effect_tests.rs-88-            "update" => {
src//resource/tests/effect_tests.rs-89-                // Update an existing resource
src//resource/tests/effect_tests.rs:90:                self.resource_api.update_resource(
src//resource/tests/effect_tests.rs-91-                    capability,
src//resource/tests/effect_tests.rs-92-                    &self.resource_id,
--
src//resource/tests/effect_template_integration_tests.rs-30-use crate::effect::templates::{
src//resource/tests/effect_template_integration_tests.rs-31-    create_resource_effect,
src//resource/tests/effect_template_integration_tests.rs:32:    update_resource_effect,
src//resource/tests/effect_template_integration_tests.rs-33-    lock_resource_effect,
src//resource/tests/effect_template_integration_tests.rs-34-    unlock_resource_effect,
--
src//resource/facade.rs-77-    
src//resource/facade.rs-78-    /// Update a ResourceRegister's state
src//resource/facade.rs:79:    pub fn update_resource_register(&self, id: &ContentId, new_state: ResourceState) -> Result<()> {
src//resource/facade.rs-80-        // Transition the resource register to the new state
src//resource/facade.rs-81-        self.lifecycle_manager.transition_state(
--
src//resource/facade.rs-174-    
src//resource/facade.rs-175-    fn update_state(&self, id: &ContentId, new_state: ResourceState) -> Result<()> {
src//resource/facade.rs:176:        self.update_resource_register(id, new_state)
src//resource/facade.rs-177-    }
src//resource/facade.rs-178-    
--
src//resource/manager.rs-30-use crate::effect::templates::{
src//resource/manager.rs-31-    create_resource_effect,
src//resource/manager.rs:32:    update_resource_effect,
src//resource/manager.rs-33-    lock_resource_effect,
src//resource/manager.rs-34-    unlock_resource_effect,
--
src//resource/manager.rs-266-            
src//resource/manager.rs-267-            // Update the resource
src//resource/manager.rs:268:            lifecycle_manager.update_resource(content_id, updated_register.clone())
src//resource/manager.rs-269-                .map_err(|e| Error::ResourceError(format!("Failed to update register: {}", e)))?;
src//resource/manager.rs-270-                
--
src//resource/manager.rs-480-                    updated.quantity = Quantity(updated.quantity.0 - qty.0);
src//resource/manager.rs-481-                    
src//resource/manager.rs:482:                    lifecycle_manager.update_resource(content_id, updated.clone())
src//resource/manager.rs-483-                        .map_err(|e| Error::ResourceError(format!("Failed to update source register: {}", e)))?;
src//resource/manager.rs-484-                        
--
src//resource/resource_temporal_consistency.rs-451-                        
src//resource/resource_temporal_consistency.rs-452-                        // Save the updated resource
src//resource/resource_temporal_consistency.rs:453:                        resource_manager.update_resource(&resource_id, updated_resource)?;
src//resource/resource_temporal_consistency.rs-454-                        
src//resource/resource_temporal_consistency.rs-455-                        sync_count += 1;
--
src//resource/capability_api.rs-216-        // Perform the update operation
src//resource/capability_api.rs-217-        self.lifecycle_manager
src//resource/capability_api.rs:218:            .update_resource_state(register_id, updated_state)
src//resource/capability_api.rs-219-            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
src//resource/capability_api.rs-220-    }
--
src//resource/capability_api.rs-267-        // Perform the metadata update operation
src//resource/capability_api.rs-268-        self.lifecycle_manager
src//resource/capability_api.rs:269:            .update_resource_state(register_id, updated_state)
src//resource/capability_api.rs-270-            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
src//resource/capability_api.rs-271-    }
--
src//resource/capability_api.rs-338-                    
src//resource/capability_api.rs-339-                    self.lifecycle_manager
src//resource/capability_api.rs:340:                        .update_resource_state(id, updated_state)
src//resource/capability_api.rs-341-                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
src//resource/capability_api.rs-342-                },
--
src//resource/capability_api.rs-351-                    
src//resource/capability_api.rs-352-                    self.lifecycle_manager
src//resource/capability_api.rs:353:                        .update_resource_state(id, updated_state)
src//resource/capability_api.rs-354-                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
src//resource/capability_api.rs-355-                },
--
src//resource/capability_api.rs-374-                    
src//resource/capability_api.rs-375-                    self.lifecycle_manager
src//resource/capability_api.rs:376:                        .update_resource_state(id, updated_state)
src//resource/capability_api.rs-377-                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
src//resource/capability_api.rs-378-                },
--
src//resource/memory_api.rs-381-    }
src//resource/memory_api.rs-382-    
src//resource/memory_api.rs:383:    async fn update_resource(
src//resource/memory_api.rs-384-        &self,
src//resource/memory_api.rs-385-        capability: &CapabilityRef,
--
src//resource/api.rs-340-    
src//resource/api.rs-341-    /// Update a resource
src//resource/api.rs:342:    async fn update_resource(
src//resource/api.rs-343-        &self,
src//resource/api.rs-344-        capability: &CapabilityRef,
--
src//actor/operator.rs-289-    
src//actor/operator.rs-290-    /// Update the operator's resource allocation
src//actor/operator.rs:291:    pub fn update_resources(&self, resources: HashMap<String, ResourcePool>) -> Result<()> {
src//actor/operator.rs-292-        let mut res = self.resources.write().map_err(|_| {
src//actor/operator.rs-293-            Error::LockError("Failed to acquire write lock on resources".to_string())
--
src//actor/operator.rs-621-                match message.payload {
src//actor/operator.rs-622-                    MessagePayload::UpdateResources { resources } => {
src//actor/operator.rs:623:                        self.update_resources(resources)?;
src//actor/operator.rs-624-                        Ok(None)
src//actor/operator.rs-625-                    },
--
src//actor/operator.rs-741-        };
src//actor/operator.rs-742-        
src//actor/operator.rs:743:        operator.update_resources(resources.clone())?;
src//actor/operator.rs-744-        assert_eq!(operator.resources()?, resources);
src//actor/operator.rs-745-        
```

### Register Update Patterns

Found the following register update patterns:

```rust
src//tel/effect/mod.rs-150-                    .id;
src//tel/effect/mod.rs-151-
src//tel/effect/mod.rs:152:                self.resource_manager.update_register(
src//tel/effect/mod.rs-153-                    &register_id,
src//tel/effect/mod.rs-154-                    new_data.clone(),
--
src//tel/resource/vm.rs-333-            
src//tel/resource/vm.rs-334-            // Update the register in the resource manager
src//tel/resource/vm.rs:335:            self.resource_manager.update_register(
src//tel/resource/vm.rs-336-                &register_id,
src//tel/resource/vm.rs-337-                new_contents,
--
src//tel/resource/vm.rs-371-            
src//tel/resource/vm.rs-372-            // Update the register
src//tel/resource/vm.rs:373:            self.resource_manager.update_register(
src//tel/resource/vm.rs-374-                &reg_id,
src//tel/resource/vm.rs-375-                new_contents,
--
src//tel/resource/model/manager.rs-134-    
src//tel/resource/model/manager.rs-135-    /// Update a register's contents
src//tel/resource/model/manager.rs:136:    pub fn update_register(
src//tel/resource/model/manager.rs-137-        &self,
src//tel/resource/model/manager.rs-138-        register_id: &ContentId,
--
src//tel/resource/model/guard.rs-290-        
src//tel/resource/model/guard.rs-291-        self.register.contents = contents.clone();
src//tel/resource/model/guard.rs:292:        self.resource_manager.update_register(&self.register_id, contents)
src//tel/resource/model/guard.rs-293-    }
src//tel/resource/model/guard.rs-294-}
--
src//tel/resource/model/guard.rs-316-        // If changes were made with write access, update the register
src//tel/resource/model/guard.rs-317-        if self.mode == AccessMode::ReadWrite {
src//tel/resource/model/guard.rs:318:            let _ = self.resource_manager.update_register(&self.register_id, self.register.contents.clone());
src//tel/resource/model/guard.rs-319-        }
src//tel/resource/model/guard.rs-320-    }
--
src//tel/resource/tests.rs-77-        // Update the resource
src//tel/resource/tests.rs-78-        let new_contents = RegisterContents::Text("Updated resource".to_string());
src//tel/resource/tests.rs:79:        manager.update_register(&register_id, new_contents.clone()).unwrap();
src//tel/resource/tests.rs-80-        
src//tel/resource/tests.rs-81-        // Check if the register was updated
--
src//tel/resource/tests.rs-143-        // Try to update the resource (should fail)
src//tel/resource/tests.rs-144-        let new_contents = RegisterContents::Text("Updated resource".to_string());
src//tel/resource/tests.rs:145:        let result = manager.update_register(&register_id, new_contents);
src//tel/resource/tests.rs-146-        assert!(result.is_err());
src//tel/resource/tests.rs-147-        
--
src//tel/resource/tests.rs-155-        // Now update should succeed
src//tel/resource/tests.rs-156-        let new_contents = RegisterContents::Text("Updated resource".to_string());
src//tel/resource/tests.rs:157:        manager.update_register(&register_id, new_contents.clone()).unwrap();
src//tel/resource/tests.rs-158-        
src//tel/resource/tests.rs-159-        // Check if the register was updated
--
src//operation/execution.rs-165-            },
src//operation/execution.rs-166-            super::RegisterOperationType::Update => {
src//operation/execution.rs:167:                self.resource_register.update_register(&register_op.register_id, &register_op.data)
src//operation/execution.rs-168-                    .await
src//operation/execution.rs-169-                    .map(|_| HashMap::new())
--
src//resource/content_addressed_resource.rs-17-        create_resource_register,
src//resource/content_addressed_resource.rs-18-        create_register_with_metadata,
src//resource/content_addressed_resource.rs:19:        update_register_data,
src//resource/content_addressed_resource.rs-20-    }
src//resource/content_addressed_resource.rs-21-};
--
src//resource/manager.rs-250-    
src//resource/manager.rs-251-    /// Update a ResourceRegister
src//resource/manager.rs:252:    pub fn update_register(
src//resource/manager.rs-253-        &self,
src//resource/manager.rs-254-        content_id: &ContentId,
--
src//resource/manager.rs-306-        } else {
src//resource/manager.rs-307-            // Legacy mode: Update in our registry
src//resource/manager.rs:308:            self.update_register(content_id, |register| {
src//resource/manager.rs-309-                register.state = RegisterState::Locked;
src//resource/manager.rs-310-                Ok(())
--
src//resource/manager.rs-331-        } else {
src//resource/manager.rs-332-            // Legacy mode: Update in our registry
src//resource/manager.rs:333:            self.update_register(content_id, |register| {
src//resource/manager.rs-334-                register.state = RegisterState::Active;
src//resource/manager.rs-335-                Ok(())
--
src//resource/manager.rs-356-        } else {
src//resource/manager.rs-357-            // Legacy mode: Update in our registry
src//resource/manager.rs:358:            self.update_register(content_id, |register| {
src//resource/manager.rs-359-                register.state = RegisterState::Consumed;
src//resource/manager.rs-360-                Ok(())
--
src//resource/tel.rs-302-            
src//resource/tel.rs-303-            // Update register in system
src//resource/tel.rs:304:            self.register_system.update_register(
src//resource/tel.rs-305-                &register_id, 
src//resource/tel.rs-306-                new_register, 
--
src//resource/tel.rs-335-            
src//resource/tel.rs-336-            // Update TEL register
src//resource/tel.rs:337:            self.tel_resource_manager.update_register(
src//resource/tel.rs-338-                &tel_register_id,
src//resource/tel.rs-339-                tel_contents,
--
src//resource/tel.rs-592-    
src//resource/tel.rs-593-    /// Update a register with time and domain information
src//resource/tel.rs:594:    pub fn update_register_with_time_and_domain(
src//resource/tel.rs-595-        &self,
src//resource/tel.rs-596-        tel_id: &ContentId
--
src//resource/tel.rs-605-        
src//resource/tel.rs-606-        // Update register with time info
src//resource/tel.rs:607:        self.register_system.update_register_with_time_info(&mut register)
src//resource/tel.rs-608-            .map_err(|e| Error::TimeError(format!("Failed to update register with time info: {}", e)))?;
src//resource/tel.rs-609-        
--
src//resource/tel.rs-746-        // Update the TEL register
src//resource/tel.rs-747-        let updated_contents = crate::tel::resource::model::RegisterContents::String("Updated content".to_string());
src//resource/tel.rs:748:        tel_resource_manager.update_register(
src//resource/tel.rs-749-            &tel_register_id, 
src//resource/tel.rs-750-            updated_contents
--
src//resource/tel.rs-764-        updated_register.contents = RegisterContents::with_string("Register updated");
src//resource/tel.rs-765-        
src//resource/tel.rs:766:        adapter.register_system().update_register(
src//resource/tel.rs-767-            &register_id,
src//resource/tel.rs-768-            updated_register,
```

### Resource Mutation Patterns

Found the following resource mutation patterns using .with_* methods:

```rust
src//tel/effect/mod.rs-572-        let proof = Proof::new("test", vec![1, 2, 3, 4]);
src//tel/effect/mod.rs:573:        let effect = ResourceEffect::new(operation).with_proof(proof.clone());
src//tel/effect/mod.rs-574-        
--
src//operation/zk.rs-282-        )
src//operation/zk.rs:283:        .with_output(ResourceRef {
src//operation/zk.rs-284-            resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/zk.rs-313-        )
src//operation/zk.rs:314:        .with_output(ResourceRef {
src//operation/zk.rs-315-            resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/transformation.rs-299-        )
src//operation/transformation.rs:300:        .with_output(ResourceRef {
src//operation/transformation.rs-301-            resource_id: content_id,
--
src//operation/tests.rs-33-    )
src//operation/tests.rs:34:    .with_output(ResourceRef {
src//operation/tests.rs-35-        resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/tests.rs-53-    )
src//operation/tests.rs:54:    .with_output(ResourceRef {
src//operation/tests.rs-55-        resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/tests.rs-83-    )
src//operation/tests.rs:84:    .with_output(ResourceRef {
src//operation/tests.rs-85-        resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/tests.rs-122-    )
src//operation/tests.rs:123:    .with_output(ResourceRef {
src//operation/tests.rs-124-        resource_id: ContentId::from_str("test:resource:123").unwrap(),
--
src//operation/test_fixtures.rs-32-    )
src//operation/test_fixtures.rs:33:    .with_output(ResourceRef {
src//operation/test_fixtures.rs-34-        resource_id: "test:resource:123".to_string(),
--
src//operation/test_fixtures.rs-52-    )
src//operation/test_fixtures.rs:53:    .with_output(ResourceRef {
src//operation/test_fixtures.rs-54-        resource_id: "test:resource:123".to_string(),
--
src//operation/test_fixtures.rs-82-    )
src//operation/test_fixtures.rs:83:    .with_output(ResourceRef {
src//operation/test_fixtures.rs-84-        resource_id: "test:resource:123".to_string(),
--
src//operation/test_fixtures.rs-121-    )
src//operation/test_fixtures.rs:122:    .with_output(ResourceRef {
src//operation/test_fixtures.rs-123-        resource_id: "test:resource:123".to_string(),
--
--
src//execution/executor.rs-76-        let default_resource_request = ResourceRequest::new()
src//execution/executor.rs:77:            .with_memory_bytes(1024 * 1024) // 1MB
--
--
src//resource/resource_temporal_consistency.rs-692-        let manager = ResourceTemporalConsistency::new(Arc::clone(&time_map))
src//resource/resource_temporal_consistency.rs:693:            .with_relationship_tracker(Arc::clone(&tracker));
```

No paired resource/register update patterns found

### Synchronization Patterns

Found the following resource/register synchronization patterns:

```rust
src//examples/cross_domain_relationships.rs-48-    // Step 5: Synchronize a relationship
src//examples/cross_domain_relationships.rs-49-    println!("\n=== Step 5: Synchronizing Relationships ===");
src//examples/cross_domain_relationships.rs:50:    synchronize_relationship(relationship_manager.clone(), sync_manager.clone())?;
src//examples/cross_domain_relationships.rs-51-    
src//examples/cross_domain_relationships.rs-52-    // Step 6: Scheduler demonstration
--
src//examples/cross_domain_relationships.rs-314-
src//examples/cross_domain_relationships.rs-315-/// Synchronize a relationship
src//examples/cross_domain_relationships.rs:316:fn synchronize_relationship(
src//examples/cross_domain_relationships.rs-317-    manager: Arc<CrossDomainRelationshipManager>,
src//examples/cross_domain_relationships.rs-318-    sync_manager: Arc<CrossDomainSyncManager>,
--
src//examples/cross_domain_relationships.rs-323-    let relationships = manager.get_all_relationships()?;
src//examples/cross_domain_relationships.rs-324-    if relationships.is_empty() {
src//examples/cross_domain_relationships.rs:325:        println!("No relationships to synchronize");
src//examples/cross_domain_relationships.rs-326-        return Ok(());
src//examples/cross_domain_relationships.rs-327-    }
--
src//log/sync.rs-105-    /// End time of the sync operation
src//log/sync.rs-106-    pub end_time: Option<Instant>,
src//log/sync.rs:107:    /// Number of entries synchronized
src//log/sync.rs-108-    pub entries_synced: usize,
src//log/sync.rs:109:    /// Number of segments synchronized
src//log/sync.rs-110-    pub segments_synced: usize,
src//log/sync.rs-111-    /// Differences between logs before sync
--
src//resource/resource_temporal_consistency.rs-411-    
src//resource/resource_temporal_consistency.rs-412-    /// Synchronize resource states across domains using the time map
src//resource/resource_temporal_consistency.rs:413:    pub fn synchronize_resources(
src//resource/resource_temporal_consistency.rs-414-        &mut self,
src//resource/resource_temporal_consistency.rs-415-        resource_manager: &mut ResourceManager,
--
src//resource/resource_temporal_consistency.rs-466-    
src//resource/resource_temporal_consistency.rs-467-    /// Synchronize relationships across domains
src//resource/resource_temporal_consistency.rs:468:    pub async fn synchronize_relationships(&self) -> Result<usize> {
src//resource/resource_temporal_consistency.rs-469-        // We need the relationship tracker to sync relationships
src//resource/resource_temporal_consistency.rs-470-        let tracker = match &self.relationship_tracker {
--
src//resource/resource_temporal_consistency.rs-624-                // When requested to sync, trigger relationship and resource sync
src//resource/resource_temporal_consistency.rs-625-                if let Some(manager) = &self.lifecycle_manager {
src//resource/resource_temporal_consistency.rs:626:                    // We'd need a ResourceManager to synchronize, which we don't have direct access to
src//resource/resource_temporal_consistency.rs-627-                    // Log that we received a sync request
src//resource/resource_temporal_consistency.rs-628-                    eprintln!("Sync request received, but no ResourceManager available");
--
src//resource/resource_temporal_consistency.rs-632-                    // Asynchronous operations can't be started from this sync method
src//resource/resource_temporal_consistency.rs-633-                    // Log that we should sync relationships
src//resource/resource_temporal_consistency.rs:634:                    eprintln!("Relationship sync should be triggered separately via synchronize_relationships()");
src//resource/resource_temporal_consistency.rs-635-                }
src//resource/resource_temporal_consistency.rs-636-            }
--
src//resource/relationship/sync.rs-96-    /// Synchronization was successful
src//resource/relationship/sync.rs-97-    Success {
src//resource/relationship/sync.rs:98:        /// ID of the relationship that was synchronized
src//resource/relationship/sync.rs-99-        relationship_id: String,
src//resource/relationship/sync.rs-100-        
--
src//resource/relationship/sync.rs-117-    /// Synchronization is in progress
src//resource/relationship/sync.rs-118-    InProgress {
src//resource/relationship/sync.rs:119:        /// ID of the relationship being synchronized
src//resource/relationship/sync.rs-120-        relationship_id: String,
src//resource/relationship/sync.rs-121-        
--
src//resource/relationship/sync.rs-184-#[derive(Debug, Clone, Serialize, Deserialize)]
src//resource/relationship/sync.rs-185-pub struct SyncHistoryEntry {
src//resource/relationship/sync.rs:186:    /// Relationship being synchronized
src//resource/relationship/sync.rs-187-    pub relationship_id: String,
src//resource/relationship/sync.rs-188-    
--
src//resource/relationship/sync.rs-449-    }
src//resource/relationship/sync.rs-450-    
src//resource/relationship/sync.rs:451:    /// Check if a resource should be synchronized based on its strategy
src//resource/relationship/sync.rs-452-    pub fn should_sync(&self, relationship: &CrossDomainRelationship) -> bool {
src//resource/relationship/sync.rs-453-        // Skip if synchronization is not required
--
src//resource/relationship/scheduler.rs-440-            let relationship_id = &relationship.id;
src//resource/relationship/scheduler.rs-441-            
src//resource/relationship/scheduler.rs:442:            // Check if relationship should be synchronized
src//resource/relationship/scheduler.rs-443-            if self.sync_manager.should_sync(&relationship) {
src//resource/relationship/scheduler.rs-444-                // Create a task for this relationship
--
src//resource/relationship/scheduler.rs-589-            Err(e) => {
src//resource/relationship/scheduler.rs-590-                error!(
src//resource/relationship/scheduler.rs:591:                    "Failed to synchronize relationship {}: {}",
src//resource/relationship/scheduler.rs-592-                    task.relationship_id, e
src//resource/relationship/scheduler.rs-593-                );
--
src//resource/relationship/scheduler.rs-607-            SyncStatus::Success => {
src//resource/relationship/scheduler.rs-608-                info!(
src//resource/relationship/scheduler.rs:609:                    "Successfully synchronized relationship {} in {:?}",
src//resource/relationship/scheduler.rs-610-                    task.relationship_id, elapsed
src//resource/relationship/scheduler.rs-611-                );
--
src//resource/relationship/scheduler.rs-630-            SyncStatus::Failed => {
src//resource/relationship/scheduler.rs-631-                warn!(
src//resource/relationship/scheduler.rs:632:                    "Failed to synchronize relationship {}: {}",
src//resource/relationship/scheduler.rs-633-                    task.relationship_id,
src//resource/relationship/scheduler.rs-634-                    result.error.as_deref().unwrap_or("Unknown error")
--
src//domain/map/sync.rs-67-    /// Status of the sync operation
src//domain/map/sync.rs-68-    pub status: SyncStatus,
src//domain/map/sync.rs:69:    /// Domains that were synchronized
src//domain/map/sync.rs-70-    pub synced_domains: HashSet<DomainId>,
src//domain/map/sync.rs:71:    /// Domains that failed to synchronize
src//domain/map/sync.rs-72-    pub failed_domains: HashMap<DomainId, String>,
src//domain/map/sync.rs-73-    /// Updated time map (if successful)
--
src//domain/map/map.rs-351-    }
src//domain/map/map.rs-352-    
src//domain/map/map.rs:353:    /// Find entries from different domains that are approximately synchronized
src//domain/map/map.rs-354-    /// within the given time tolerance (in seconds)
src//domain/map/map.rs:355:    pub fn find_synchronized(&self, tolerance: u64) -> Vec<Vec<&TimeMapEntry>> {
src//domain/map/map.rs-356-        let mut entries: Vec<&TimeMapEntry> = self.entries.values().collect();
src//domain/map/map.rs-357-        
--
src//domain/time_sync.rs-84-    /// Configuration
src//domain/time_sync.rs-85-    config: SyncConfig,
src//domain/time_sync.rs:86:    /// Time map to synchronize
src//domain/time_sync.rs-87-    time_map: SharedTimeMap,
src//domain/time_sync.rs-88-    /// Domain adapters
```

## Recommended Migration Pattern

Replace these patterns with the unified ResourceRegister approach:
```rust
// OLD PATTERN:
// Step 1: Update the resource
let new_resource = resource.with_quantity(new_amount);
resource_manager.update_resource(resource.id, new_resource)?;

// Step 2: Update the register
register_system.update_register(register.id, new_resource.id)?;

// NEW PATTERN:
// Update the unified model
token.update_quantity(new_amount)?;

// Update storage
effect_system.execute_effect(StorageEffect::StoreOnChain {
    register_id: token.id,
    fields: HashSet::from([String::from("quantity")]),
    domain_id: domain_id,
    continuation: Box::new(|result| {
        // Handle update result
    }),
}).await?;
```

