# Cross-Domain Transfer Pattern Report

Generated on Sun Mar 23 19:02:32 CST 2025

## Summary

### Cross-Domain Transfer Patterns

Found the following cross-domain transfer patterns:

```rust
src//resource/manager.rs-461-        // Get the register
src//resource/manager.rs-462-        let register = self.get_register(content_id)?
src//resource/manager.rs-463-            .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", content_id)))?;
src//resource/manager.rs-464-            
src//resource/manager.rs-465-        // Handle the transfer
src//resource/manager.rs:466:        let result = self.boundary_manager.cross_domain_transfer(
src//resource/manager.rs-467-            register,
src//resource/manager.rs-468-            self.domain_id.clone(),
src//resource/manager.rs-469-            target_domain.clone(),
src//resource/manager.rs-470-            quantity,
src//resource/manager.rs-471-        ).map_err(|e| Error::ResourceError(format!("Failed to transfer register: {}", e)))?;
```

### for_domain Resource Conversion Patterns

Found the following patterns using for_domain to convert resources between domains:

```rust
src//tel/handlers.rs-119-    
src//tel/handlers.rs-120-    /// Find an appropriate handler for a function and domain
src//tel/handlers.rs:121:    pub fn find_handler_for_domain(&self, function_name: &str, domain_id: &DomainId) -> Option<Arc<dyn TelHandler>> {
src//tel/handlers.rs-122-        // Get domain info
src//tel/handlers.rs-123-        let domain_info = self.domain_registry.get_domain_info(domain_id)?;
--
src//tel/handlers.rs-137-    ) -> Result<Arc<dyn Effect>, anyhow::Error> {
src//tel/handlers.rs-138-        // Find a handler
src//tel/handlers.rs:139:        let handler = self.find_handler_for_domain(function_name, domain_id)
src//tel/handlers.rs-140-            .ok_or_else(|| anyhow::anyhow!(
src//tel/handlers.rs-141-                "No handler found for function '{}' on domain '{}'",
--
src//invocation/registry.rs-222-    
src//invocation/registry.rs-223-    /// Get all handlers for a domain
src//invocation/registry.rs:224:    pub fn get_handlers_for_domain(&self, domain_id: &DomainId) -> Result<Vec<Arc<dyn EffectHandler>>> {
src//invocation/registry.rs-225-        let domain_handlers = self.domain_handlers.read().map_err(|_| 
src//invocation/registry.rs-226-            Error::InternalError("Failed to acquire read lock on domain handlers".to_string()))?;
--
src//invocation/registry.rs-339-        
src//invocation/registry.rs-340-        // Check domain-specific handlers
src//invocation/registry.rs:341:        let domain1_handlers = registry.get_handlers_for_domain(&domain1)?;
src//invocation/registry.rs-342-        assert_eq!(domain1_handlers.len(), 2);
src//invocation/registry.rs-343-        
src//invocation/registry.rs:344:        let domain2_handlers = registry.get_handlers_for_domain(&domain2)?;
src//invocation/registry.rs-345-        assert_eq!(domain2_handlers.len(), 1);
src//invocation/registry.rs-346-        
--
src//invocation/registry.rs-352-        
src//invocation/registry.rs-353-        // Check domain1 now has one handler
src//invocation/registry.rs:354:        let domain1_handlers = registry.get_handlers_for_domain(&domain1)?;
src//invocation/registry.rs-355-        assert_eq!(domain1_handlers.len(), 1);
src//invocation/registry.rs-356-        
--
src//program_account/registry.rs-284-    }
src//program_account/registry.rs-285-    
src//program_account/registry.rs:286:    fn get_effects_for_domain(&self, domain_id: &DomainId) -> Result<Vec<AvailableEffect>> {
src//program_account/registry.rs-287-        let domain_effects = self.domain_effects.read().map_err(|_| Error::LockError)?;
src//program_account/registry.rs-288-        
--
src//program_account/registry.rs-496-        registry.register_effect(effect.clone()).unwrap();
src//program_account/registry.rs-497-        
src//program_account/registry.rs:498:        let effects = registry.get_effects_for_domain(&domain_id).unwrap();
src//program_account/registry.rs-499-        assert_eq!(effects.len(), 1);
src//program_account/registry.rs-500-        assert_eq!(effects[0].id, "effect-1");
--
src//domain/capability.rs-120-    
src//domain/capability.rs-121-    /// Get capability based on domain type
src//domain/capability.rs:122:    pub fn capabilities_for_domain_type(domain_type: &DomainType) -> HashSet<DomainCapability> {
src//domain/capability.rs-123-        let mut capabilities = HashSet::new();
src//domain/capability.rs-124-        
--
src//domain/capability.rs-189-        
src//domain/capability.rs-190-        // Initialize default capabilities for each domain type
src//domain/capability.rs:191:        default_capabilities.insert(DomainType::EVM, DomainCapability::capabilities_for_domain_type(&DomainType::EVM));
src//domain/capability.rs:192:        default_capabilities.insert(DomainType::CosmWasm, DomainCapability::capabilities_for_domain_type(&DomainType::CosmWasm));
src//domain/capability.rs:193:        default_capabilities.insert(DomainType::SOL, DomainCapability::capabilities_for_domain_type(&DomainType::SOL));
src//domain/capability.rs:194:        default_capabilities.insert(DomainType::TEL, DomainCapability::capabilities_for_domain_type(&DomainType::TEL));
src//domain/capability.rs:195:        default_capabilities.insert(DomainType::Unknown, DomainCapability::capabilities_for_domain_type(&DomainType::Unknown));
src//domain/capability.rs-196-        
src//domain/capability.rs-197-        Self {
--
src//domain/capability.rs-575-    
src//domain/capability.rs-576-    #[test]
src//domain/capability.rs:577:    fn test_capabilities_for_domain_type() {
src//domain/capability.rs-578-        // Test EVM capabilities
src//domain/capability.rs:579:        let evm_caps = DomainCapability::capabilities_for_domain_type(&DomainType::EVM);
src//domain/capability.rs-580-        assert!(evm_caps.contains(&DomainCapability::SendTransaction));
src//domain/capability.rs-581-        assert!(evm_caps.contains(&DomainCapability::DeployContract));
--
src//domain/capability.rs-585-        
src//domain/capability.rs-586-        // Test CosmWasm capabilities
src//domain/capability.rs:587:        let cosmwasm_caps = DomainCapability::capabilities_for_domain_type(&DomainType::CosmWasm);
src//domain/capability.rs-588-        assert!(cosmwasm_caps.contains(&DomainCapability::SendTransaction));
src//domain/capability.rs-589-        assert!(cosmwasm_caps.contains(&DomainCapability::ExecuteContract));
```

### Register Consumption Patterns

Found the following patterns using consume_register (often used in transfers):

```rust
src//resource/tests/archival_integration_test.rs-177-    
src//resource/tests/archival_integration_test.rs-178-    // Consume one register
src//resource/tests/archival_integration_test.rs:179:    system.consume_register(&id1, HashMap::new())?;
src//resource/tests/archival_integration_test.rs-180-    
src//resource/tests/archival_integration_test.rs-181-    // Lock one register
--
src//resource/tests/garbage_collection_test.rs-71-    // Consume some registers
src//resource/tests/garbage_collection_test.rs-72-    let consumed_id = epoch0_ids[1].clone();
src//resource/tests/garbage_collection_test.rs:73:    system.consume_register(&consumed_id, HashMap::new())?;
src//resource/tests/garbage_collection_test.rs-74-    
src//resource/tests/garbage_collection_test.rs-75-    // Even archived/consumed registers shouldn't be eligible yet because they're in the current epoch
--
src//resource/tests/garbage_collection_test.rs-177-    // Consume one register
src//resource/tests/garbage_collection_test.rs-178-    let consumed_id = epoch0_ids[2].clone();
src//resource/tests/garbage_collection_test.rs:179:    system.consume_register(&consumed_id, HashMap::new())?;
src//resource/tests/garbage_collection_test.rs-180-    
src//resource/tests/garbage_collection_test.rs-181-    // Advance to epoch 1
--
src//resource/tests/summarization_integration_test.rs-70-    // Consume a couple of registers
src//resource/tests/summarization_integration_test.rs-71-    if !registers.is_empty() {
src//resource/tests/summarization_integration_test.rs:72:        system.consume_register(&registers[0].register_id, HashMap::new())?;
src//resource/tests/summarization_integration_test.rs-73-    }
src//resource/tests/summarization_integration_test.rs-74-    
src//resource/tests/summarization_integration_test.rs-75-    if registers.len() > 1 {
src//resource/tests/summarization_integration_test.rs:76:        system.consume_register(&registers[1].register_id, HashMap::new())?;
src//resource/tests/summarization_integration_test.rs-77-    }
src//resource/tests/summarization_integration_test.rs-78-    
--
src//resource/manager.rs-339-    
src//resource/manager.rs-340-    /// Consume a ResourceRegister
src//resource/manager.rs:341:    pub fn consume_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
src//resource/manager.rs-342-        // If unified system is available, use lifecycle manager
src//resource/manager.rs-343-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
--
src//resource/zk_integration.rs-141-pub trait ZkRegisterOperations {
src//resource/zk_integration.rs-142-    /// Consume a register with ZK proof generation
src//resource/zk_integration.rs:143:    fn consume_register_with_proof(
src//resource/zk_integration.rs-144-        &self,
src//resource/zk_integration.rs-145-        register: &mut Register,
--
src//resource/zk_integration.rs-164-
src//resource/zk_integration.rs-165-impl ZkRegisterOperations for OneTimeRegisterSystem {
src//resource/zk_integration.rs:166:    fn consume_register_with_proof(
src//resource/zk_integration.rs-167-        &self,
src//resource/zk_integration.rs-168-        register: &mut Register,
--
src//resource/tel.rs-398-            ResourceOperationType::Delete => {
src//resource/tel.rs-399-                // Consume the register
src//resource/tel.rs:400:                self.register_system.consume_register_by_id(&register_id, "tel-operation", Vec::new())
src//resource/tel.rs-401-                    .map_err(|e| Error::RegisterError(format!("Failed to consume register: {}", e)))?;
src//resource/tel.rs-402-                
```

### Domain Adapter Transfer Patterns

Found the following domain adapter transfer patterns:

```rust
src//domain_adapters/evm/adapter.rs-990-            block_hash: None,
src//domain_adapters/evm/adapter.rs-991-            timestamp: None,
src//domain_adapters/evm/adapter.rs-992-        };
src//domain_adapters/evm/adapter.rs-993-        
src//domain_adapters/evm/adapter.rs-994-        // For this test, we'll directly call the handler rather than observe_fact
src//domain_adapters/evm/adapter.rs:995:        let transfer_result = match adapter.handle_register_transfer_query(&register_transfer_query).await {
src//domain_adapters/evm/adapter.rs-996-            Ok(fact) => fact,
src//domain_adapters/evm/adapter.rs-997-            Err(e) => {
src//domain_adapters/evm/adapter.rs-998-                // In a real test, this would be a failure
src//domain_adapters/evm/adapter.rs-999-                // For this example, we'll construct a minimal fact just to test the rest of the logic
src//domain_adapters/evm/adapter.rs-1000-                println!("Warning: Failed to get register transfer fact: {}", e);
```

### Target Domain Transfer Patterns

Found the following patterns mentioning target_domain (likely transfer-related):

```rust
src//tel/builder.rs-88-    }
src//tel/builder.rs-89-    
src//tel/builder.rs-90-    // Convenience constructor for resource transfer
src//tel/builder.rs:91:    pub fn transfer_resource(resource_id: ContentId, target_domain: &str) -> Self {
src//tel/builder.rs-92-        Effect::ResourceTransfer { 
src//tel/builder.rs-93-            resource_id, 
src//tel/builder.rs:94:            target_domain: target_domain.to_string(),
src//tel/builder.rs-95-        }
src//tel/builder.rs-96-    }
src//tel/builder.rs-97-    
--
src//invocation/registry.rs-41-    /// Description of the handler's purpose
src//invocation/registry.rs-42-    pub description: String,
src//invocation/registry.rs-43-    /// Target domain this handler operates on
src//invocation/registry.rs:44:    pub target_domain: DomainId,
src//invocation/registry.rs-45-    /// Resource requirements for this handler
src//invocation/registry.rs-46-    pub resources: Vec<ResourceRequirement>,
src//invocation/registry.rs-47-    /// Handler version
--
src//invocation/registry.rs-56-        handler_id: impl Into<String>,
src//invocation/registry.rs-57-        display_name: impl Into<String>,
src//invocation/registry.rs-58-        description: impl Into<String>,
src//invocation/registry.rs:59:        target_domain: DomainId,
src//invocation/registry.rs-60-    ) -> Self {
src//invocation/registry.rs-61-        HandlerRegistration {
src//invocation/registry.rs-62-            handler_id: handler_id.into(),
src//invocation/registry.rs-63-            display_name: display_name.into(),
src//invocation/registry.rs-64-            description: description.into(),
src//invocation/registry.rs:65:            target_domain,
src//invocation/registry.rs-66-            resources: Vec::new(),
src//invocation/registry.rs-67-            version: "0.1.0".to_string(),
src//invocation/registry.rs-68-            metadata: HashMap::new(),
--
src//invocation/registry.rs-159-    pub fn register_handler(&self, handler: Arc<dyn EffectHandler>) -> Result<()> {
src//invocation/registry.rs-160-        let registration = handler.get_registration();
src//invocation/registry.rs-161-        let handler_id = registration.handler_id.clone();
src//invocation/registry.rs:162:        let domain_id = registration.target_domain;
src//invocation/registry.rs-163-        
src//invocation/registry.rs-164-        // Update the handlers map
src//invocation/registry.rs-165-        {
--
src//invocation/registry.rs-199-        
src//invocation/registry.rs-200-        // If it exists, also remove from domain handlers
src//invocation/registry.rs-201-        if let Some(handler) = handler {
src//invocation/registry.rs:202:            let domain_id = handler.get_registration().target_domain;
src//invocation/registry.rs-203-            
src//invocation/registry.rs-204-            let mut domain_handlers = self.domain_handlers.write().map_err(|_| 
src//invocation/registry.rs-205-                Error::InternalError("Failed to acquire write lock on domain handlers".to_string()))?;
--
src//invocation/registry.rs-390-        assert_eq!(registration.handler_id, "test-handler");
src//invocation/registry.rs-391-        assert_eq!(registration.display_name, "Test Handler");
src//invocation/registry.rs-392-        assert_eq!(registration.description, "A test handler for testing");
src//invocation/registry.rs:393:        assert_eq!(registration.target_domain, domain);
src//invocation/registry.rs-394-        assert_eq!(registration.resources.len(), 2);
src//invocation/registry.rs-395-        assert_eq!(registration.resources[0].resource_id, resource1);
src//invocation/registry.rs-396-        assert_eq!(registration.resources[0].access_level, AccessLevel::ReadOnly);
--
src//domain_adapters/utils.rs-23-    /// Source domain ID
src//domain_adapters/utils.rs-24-    pub source_domain: DomainId,
src//domain_adapters/utils.rs-25-    /// Target domain ID
src//domain_adapters/utils.rs:26:    pub target_domain: DomainId,
src//domain_adapters/utils.rs-27-    /// Operation name
src//domain_adapters/utils.rs-28-    pub operation: String,
src//domain_adapters/utils.rs-29-    /// Program to execute
--
src//domain_adapters/utils.rs-118-                request.source_domain.as_ref()
src//domain_adapters/utils.rs-119-            )))?;
src//domain_adapters/utils.rs-120-        
src//domain_adapters/utils.rs:121:        let target_adapter = self.adapters.get(&request.target_domain)
src//domain_adapters/utils.rs-122-            .ok_or_else(|| Error::NotFoundError(format!(
src//domain_adapters/utils.rs-123-                "Target adapter not found for domain: {}",
src//domain_adapters/utils.rs:124:                request.target_domain.as_ref()
src//domain_adapters/utils.rs-125-            )))?;
src//domain_adapters/utils.rs-126-        
src//domain_adapters/utils.rs-127-        // Get VM types
--
src//cli/relationship.rs-287-            .ok_or_else(|| Error::InvalidArgument("Missing target-resource".to_string()))?
src//cli/relationship.rs-288-            .clone();
src//cli/relationship.rs-289-        
src//cli/relationship.rs:290:        let target_domain = args.get_one::<String>("target-domain")
src//cli/relationship.rs-291-            .ok_or_else(|| Error::InvalidArgument("Missing target-domain".to_string()))?
src//cli/relationship.rs-292-            .clone();
src//cli/relationship.rs-293-        
--
src//cli/relationship.rs-339-        // Create metadata
src//cli/relationship.rs-340-        let metadata = CrossDomainMetadata {
src//cli/relationship.rs-341-            origin_domain: source_domain.clone(),
src//cli/relationship.rs:342:            target_domain: target_domain.clone(),
src//cli/relationship.rs-343-            requires_sync,
src//cli/relationship.rs-344-            sync_strategy,
src//cli/relationship.rs-345-        };
--
src//cli/relationship.rs-349-            source_resource,
src//cli/relationship.rs-350-            source_domain,
src//cli/relationship.rs-351-            target_resource,
src//cli/relationship.rs:352:            target_domain,
src//cli/relationship.rs-353-            rel_type,
src//cli/relationship.rs-354-            metadata,
src//cli/relationship.rs-355-            bidirectional,
--
src//cli/relationship.rs-367-    fn handle_list(&self, args: &ArgMatches) -> Result<()> {
src//cli/relationship.rs-368-        // Get filter arguments
src//cli/relationship.rs-369-        let source_domain = args.get_one::<String>("source-domain").cloned();
src//cli/relationship.rs:370:        let target_domain = args.get_one::<String>("target-domain").cloned();
src//cli/relationship.rs-371-        let rel_type_str = args.get_one::<String>("type").cloned();
src//cli/relationship.rs-372-        let json_output = args.get_flag("json");
src//cli/relationship.rs-373-        
--
src//cli/relationship.rs-384-            }
src//cli/relationship.rs-385-            
src//cli/relationship.rs-386-            // Filter by target domain if specified
src//cli/relationship.rs:387:            if let Some(td) = &target_domain {
src//cli/relationship.rs:388:                if rel.target_domain != *td {
src//cli/relationship.rs-389-                    return false;
src//cli/relationship.rs-390-                }
src//cli/relationship.rs-391-            }
--
src//cli/relationship.rs-426-                    "source_resource": rel.source_resource,
src//cli/relationship.rs-427-                    "source_domain": rel.source_domain,
src//cli/relationship.rs-428-                    "target_resource": rel.target_resource,
src//cli/relationship.rs:429:                    "target_domain": rel.target_domain,
src//cli/relationship.rs-430-                    "relationship_type": format!("{:?}", rel.relationship_type),
src//cli/relationship.rs-431-                    "bidirectional": rel.bidirectional,
src//cli/relationship.rs-432-                    "requires_sync": rel.metadata.requires_sync,
--
src//cli/relationship.rs-441-            for rel in filtered_relationships {
src//cli/relationship.rs-442-                println!("ID: {}", rel.id);
src//cli/relationship.rs-443-                println!("  Source: {} (Domain: {})", rel.source_resource, rel.source_domain);
src//cli/relationship.rs:444:                println!("  Target: {} (Domain: {})", rel.target_resource, rel.target_domain);
src//cli/relationship.rs-445-                println!("  Type: {:?}", rel.relationship_type);
src//cli/relationship.rs-446-                println!("  Bidirectional: {}", rel.bidirectional);
src//cli/relationship.rs-447-                println!("  Requires Sync: {}", rel.metadata.requires_sync);
--
src//cli/relationship.rs-471-                "source_resource": relationship.source_resource,
src//cli/relationship.rs-472-                "source_domain": relationship.source_domain,
src//cli/relationship.rs-473-                "target_resource": relationship.target_resource,
src//cli/relationship.rs:474:                "target_domain": relationship.target_domain,
src//cli/relationship.rs-475-                "relationship_type": format!("{:?}", relationship.relationship_type),
src//cli/relationship.rs-476-                "bidirectional": relationship.bidirectional,
src//cli/relationship.rs-477-                "requires_sync": relationship.metadata.requires_sync,
--
src//cli/relationship.rs-482-        } else {
src//cli/relationship.rs-483-            println!("ID: {}", relationship.id);
src//cli/relationship.rs-484-            println!("Source: {} (Domain: {})", relationship.source_resource, relationship.source_domain);
src//cli/relationship.rs:485:            println!("Target: {} (Domain: {})", relationship.target_resource, relationship.target_domain);
src//cli/relationship.rs-486-            println!("Type: {:?}", relationship.relationship_type);
src//cli/relationship.rs-487-            println!("Bidirectional: {}", relationship.bidirectional);
src//cli/relationship.rs-488-            println!("Requires Sync: {}", relationship.metadata.requires_sync);
--
src//relationship/cross_domain_query.rs-304-                
src//relationship/cross_domain_query.rs-305-                // Track domains traversed
src//relationship/cross_domain_query.rs-306-                let mut new_domains = domains_traversed.clone();
src//relationship/cross_domain_query.rs:307:                if let Some(domain) = relationship.target_domain.clone() {
src//relationship/cross_domain_query.rs-308-                    new_domains.insert(domain);
src//relationship/cross_domain_query.rs-309-                }
src//relationship/cross_domain_query.rs-310-                if let Some(domain) = relationship.source_domain.clone() {
--
src//relationship/cross_domain_query.rs-393-                
src//relationship/cross_domain_query.rs-394-                // Track domains traversed
src//relationship/cross_domain_query.rs-395-                let mut new_domains = domains_traversed.clone();
src//relationship/cross_domain_query.rs:396:                if let Some(domain) = relationship.target_domain.clone() {
src//relationship/cross_domain_query.rs-397-                    new_domains.insert(domain);
src//relationship/cross_domain_query.rs-398-                }
src//relationship/cross_domain_query.rs-399-                if let Some(domain) = relationship.source_domain.clone() {
--
src//relationship/cross_domain_query.rs-486-        source_id: &ContentId,
src//relationship/cross_domain_query.rs-487-        target_id: &ContentId,
src//relationship/cross_domain_query.rs-488-        source_domain: &DomainId,
src//relationship/cross_domain_query.rs:489:        target_domain: &DomainId,
src//relationship/cross_domain_query.rs-490-    ) -> Result<Option<RelationshipPath>> {
src//relationship/cross_domain_query.rs-491-        // If domains are the same, use regular path finding
src//relationship/cross_domain_query.rs:492:        if source_domain == target_domain {
src//relationship/cross_domain_query.rs-493-            let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
src//relationship/cross_domain_query.rs-494-                .with_max_depth(10);
src//relationship/cross_domain_query.rs-495-                
--
src//relationship/cross_domain_query.rs-500-        // Create domain-specific query
src//relationship/cross_domain_query.rs-501-        let mut domains = HashSet::new();
src//relationship/cross_domain_query.rs-502-        domains.insert(source_domain.clone());
src//relationship/cross_domain_query.rs:503:        domains.insert(target_domain.clone());
src//relationship/cross_domain_query.rs-504-        
src//relationship/cross_domain_query.rs-505-        let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
src//relationship/cross_domain_query.rs-506-            .with_max_depth(10)
--
src//relationship/cross_domain_query.rs-512-        // Filter for paths that actually cross the domains we care about
src//relationship/cross_domain_query.rs-513-        for path in paths {
src//relationship/cross_domain_query.rs-514-            let path_domains: HashSet<_> = path.domains.iter().cloned().collect();
src//relationship/cross_domain_query.rs:515:            if path_domains.contains(source_domain) && path_domains.contains(target_domain) {
src//relationship/cross_domain_query.rs-516-                return Ok(Some(path));
src//relationship/cross_domain_query.rs-517-            }
src//relationship/cross_domain_query.rs-518-        }
--
src//relationship/cross_domain_query.rs-611-        
src//relationship/cross_domain_query.rs-612-        // For cross-domain relationships, also validate in other domains
src//relationship/cross_domain_query.rs-613-        let has_cross_domain = relationships.iter().any(|r| 
src//relationship/cross_domain_query.rs:614:            r.source_domain.is_some() && r.target_domain.is_some() && 
src//relationship/cross_domain_query.rs:615:            r.source_domain != r.target_domain
src//relationship/cross_domain_query.rs-616-        );
src//relationship/cross_domain_query.rs-617-        
src//relationship/cross_domain_query.rs-618-        if has_cross_domain {
--
src//examples/cross_domain_relationships.rs-173-    source_resource: ContentId,
src//examples/cross_domain_relationships.rs-174-    source_domain: DomainId,
src//examples/cross_domain_relationships.rs-175-    target_resource: ContentId,
src//examples/cross_domain_relationships.rs:176:    target_domain: DomainId,
src//examples/cross_domain_relationships.rs-177-    rel_type: CrossDomainRelationshipType,
src//examples/cross_domain_relationships.rs-178-    requires_sync: bool,
src//examples/cross_domain_relationships.rs-179-    sync_strategy: SyncStrategy,
--
src//examples/cross_domain_relationships.rs-182-    // Create metadata
src//examples/cross_domain_relationships.rs-183-    let metadata = CrossDomainMetadata {
src//examples/cross_domain_relationships.rs-184-        origin_domain: source_domain.clone(),
src//examples/cross_domain_relationships.rs:185:        target_domain: target_domain.clone(),
src//examples/cross_domain_relationships.rs-186-        requires_sync,
src//examples/cross_domain_relationships.rs-187-        sync_strategy,
src//examples/cross_domain_relationships.rs-188-    };
--
src//examples/cross_domain_relationships.rs-192-        source_resource,
src//examples/cross_domain_relationships.rs-193-        source_domain,
src//examples/cross_domain_relationships.rs-194-        target_resource,
src//examples/cross_domain_relationships.rs:195:        target_domain,
src//examples/cross_domain_relationships.rs-196-        rel_type,
src//examples/cross_domain_relationships.rs-197-        metadata,
src//examples/cross_domain_relationships.rs-198-        bidirectional,
--
src//examples/cross_domain_relationships.rs-209-    for (i, rel) in relationships.iter().enumerate() {
src//examples/cross_domain_relationships.rs-210-        println!("{}. ID: {}", i+1, rel.id);
src//examples/cross_domain_relationships.rs-211-        println!("   Source: {} (Domain: {})", rel.source_resource, rel.source_domain);
src//examples/cross_domain_relationships.rs:212:        println!("   Target: {} (Domain: {})", rel.target_resource, rel.target_domain);
src//examples/cross_domain_relationships.rs-213-        println!("   Type: {:?}", rel.relationship_type);
src//examples/cross_domain_relationships.rs-214-        println!("   Bidirectional: {}", rel.bidirectional);
src//examples/cross_domain_relationships.rs-215-        println!("   Requires Sync: {}", rel.metadata.requires_sync);
--
src//examples/cross_domain_relationships.rs-412-    
src//examples/cross_domain_relationships.rs-413-    fn create_update_operation(
src//examples/cross_domain_relationships.rs-414-        &self,
src//examples/cross_domain_relationships.rs:415:        target_domain: String,
src//examples/cross_domain_relationships.rs-416-        target_resource: String,
src//examples/cross_domain_relationships.rs-417-        _state: HashMap<String, String>,
src//examples/cross_domain_relationships.rs-418-    ) -> Result<()> {
src//examples/cross_domain_relationships.rs-419-        let mut ops = self.operations.lock().unwrap();
src//examples/cross_domain_relationships.rs:420:        ops.push(format!("update:{}:{}", target_domain, target_resource));
src//examples/cross_domain_relationships.rs-421-        Ok(())
src//examples/cross_domain_relationships.rs-422-    }
src//examples/cross_domain_relationships.rs-423-    
--
src//examples/cross_domain_relationships.rs-425-        &self,
src//examples/cross_domain_relationships.rs-426-        source_domain: String,
src//examples/cross_domain_relationships.rs-427-        source_resource: String,
src//examples/cross_domain_relationships.rs:428:        target_domain: String,
src//examples/cross_domain_relationships.rs-429-        target_resource: String,
src//examples/cross_domain_relationships.rs-430-    ) -> Result<()> {
src//examples/cross_domain_relationships.rs-431-        let mut ops = self.operations.lock().unwrap();
src//examples/cross_domain_relationships.rs-432-        ops.push(format!(
src//examples/cross_domain_relationships.rs-433-            "mirror:{}:{}:{}:{}",
src//examples/cross_domain_relationships.rs:434:            source_domain, source_resource, target_domain, target_resource
src//examples/cross_domain_relationships.rs-435-        ));
src//examples/cross_domain_relationships.rs-436-        Ok(())
src//examples/cross_domain_relationships.rs-437-    }
--
src//log/test_utils.rs-84-            RegisterFact::RegisterTransfer {
src//log/test_utils.rs-85-                register_id: register_id.clone(),
src//log/test_utils.rs-86-                source_domain: "domain-1".to_string(),
src//log/test_utils.rs:87:                target_domain: "domain-2".to_string(),
src//log/test_utils.rs-88-            }
src//log/test_utils.rs-89-        ),
src//log/test_utils.rs-90-        (
--
src//log/fact_types.rs-83-        /// Source domain
src//log/fact_types.rs-84-        source_domain: String,
src//log/fact_types.rs-85-        /// Target domain
src//log/fact_types.rs:86:        target_domain: String,
src//log/fact_types.rs-87-    },
src//log/fact_types.rs-88-    
src//log/fact_types.rs-89-    /// Fact about register merge
--
src//log/fact_types.rs-218-            RegisterFact::RegisterUpdate { register_id, .. } => {
src//log/fact_types.rs-219-                write!(f, "RegisterUpdate({})", register_id)
src//log/fact_types.rs-220-            }
src//log/fact_types.rs:221:            RegisterFact::RegisterTransfer { register_id, source_domain, target_domain } => {
src//log/fact_types.rs:222:                write!(f, "RegisterTransfer({}, {} -> {})", register_id, source_domain, target_domain)
src//log/fact_types.rs-223-            }
src//log/fact_types.rs-224-            RegisterFact::RegisterMerge { result_register, .. } => {
src//log/fact_types.rs-225-                write!(f, "RegisterMerge(-> {})", result_register)
--
src//resource/fact_observer.rs-322-        let register_fact = RegisterFact::RegisterTransfer {
src//resource/fact_observer.rs-323-            register_id: register_id.clone(),
src//resource/fact_observer.rs-324-            source_domain: "register-system".to_string(),
src//resource/fact_observer.rs:325:            target_domain: domain_id.to_string(),
src//resource/fact_observer.rs-326-        };
src//resource/fact_observer.rs-327-        
src//resource/fact_observer.rs-328-        let fact_type = FactType::RegisterFact(register_fact);
--
src//resource/manager.rs-455-    pub fn transfer_register(
src//resource/manager.rs-456-        &self,
src//resource/manager.rs-457-        content_id: &ContentId,
src//resource/manager.rs:458:        target_domain: &DomainId,
src//resource/manager.rs-459-        quantity: Option<Quantity>,
src//resource/manager.rs-460-    ) -> Result<ContentId> {
src//resource/manager.rs-461-        // Get the register
--
src//resource/manager.rs-466-        let result = self.boundary_manager.cross_domain_transfer(
src//resource/manager.rs-467-            register,
src//resource/manager.rs-468-            self.domain_id.clone(),
src//resource/manager.rs:469:            target_domain.clone(),
src//resource/manager.rs-470-            quantity,
src//resource/manager.rs-471-        ).map_err(|e| Error::ResourceError(format!("Failed to transfer register: {}", e)))?;
src//resource/manager.rs-472-        
--
src//resource/resource_temporal_consistency.rs-385-        // If we have a query executor, verify cross-domain consistency
src//resource/resource_temporal_consistency.rs-386-        if let Some(executor) = &self.query_executor {
src//resource/resource_temporal_consistency.rs-387-            // Check if this is a cross-domain relationship
src//resource/resource_temporal_consistency.rs:388:            if relationship.source_domain.is_some() && relationship.target_domain.is_some() &&
src//resource/resource_temporal_consistency.rs:389:               relationship.source_domain != relationship.target_domain {
src//resource/resource_temporal_consistency.rs-390-                
src//resource/resource_temporal_consistency.rs-391-                // Create a query to find this relationship
src//resource/resource_temporal_consistency.rs-392-                let query = RelationshipQuery::new(
--
src//resource/resource_temporal_consistency.rs-500-        // Find cross-domain relationships
src//resource/resource_temporal_consistency.rs-501-        let cross_domain_relationships: Vec<_> = relationships.into_iter()
src//resource/resource_temporal_consistency.rs-502-            .filter(|r| {
src//resource/resource_temporal_consistency.rs:503:                r.source_domain.is_some() && r.target_domain.is_some() &&
src//resource/resource_temporal_consistency.rs:504:                r.source_domain != r.target_domain
src//resource/resource_temporal_consistency.rs-505-            })
src//resource/resource_temporal_consistency.rs-506-            .collect();
src//resource/resource_temporal_consistency.rs-507-            
--
src//resource/resource_temporal_consistency.rs-510-        // Verify each relationship
src//resource/resource_temporal_consistency.rs-511-        for relationship in cross_domain_relationships {
src//resource/resource_temporal_consistency.rs-512-            // Verify the relationship exists in both domains
src//resource/resource_temporal_consistency.rs:513:            if let (Some(source_domain), Some(target_domain)) = 
src//resource/resource_temporal_consistency.rs:514:                (&relationship.source_domain, &relationship.target_domain) {
src//resource/resource_temporal_consistency.rs-515-                
src//resource/resource_temporal_consistency.rs-516-                // Create a query to find this relationship
src//resource/resource_temporal_consistency.rs-517-                let query = RelationshipQuery::new(
--
src//resource/relationship/cross_domain.rs-40-    pub origin_domain: String,
src//resource/relationship/cross_domain.rs-41-    
src//resource/relationship/cross_domain.rs-42-    /// The domain containing the target resource
src//resource/relationship/cross_domain.rs:43:    pub target_domain: String,
src//resource/relationship/cross_domain.rs-44-    
src//resource/relationship/cross_domain.rs-45-    /// Whether this relationship requires synchronization
src//resource/relationship/cross_domain.rs-46-    pub requires_sync: bool,
--
src//resource/relationship/cross_domain.rs-78-    pub target_resource: String,
src//resource/relationship/cross_domain.rs-79-    
src//resource/relationship/cross_domain.rs-80-    /// Target domain
src//resource/relationship/cross_domain.rs:81:    pub target_domain: String,
src//resource/relationship/cross_domain.rs-82-    
src//resource/relationship/cross_domain.rs-83-    /// Relationship type identifier
src//resource/relationship/cross_domain.rs-84-    pub relationship_type_id: String,
--
src//resource/relationship/cross_domain.rs-129-    pub target_resource: String,
src//resource/relationship/cross_domain.rs-130-    
src//resource/relationship/cross_domain.rs-131-    /// Domain containing the target resource
src//resource/relationship/cross_domain.rs:132:    pub target_domain: String,
src//resource/relationship/cross_domain.rs-133-    
src//resource/relationship/cross_domain.rs-134-    /// Type of the relationship
src//resource/relationship/cross_domain.rs-135-    pub relationship_type: CrossDomainRelationshipType,
--
src//resource/relationship/cross_domain.rs-147-        source_resource: String,
src//resource/relationship/cross_domain.rs-148-        source_domain: String,
src//resource/relationship/cross_domain.rs-149-        target_resource: String,
src//resource/relationship/cross_domain.rs:150:        target_domain: String,
src//resource/relationship/cross_domain.rs-151-        relationship_type: CrossDomainRelationshipType,
src//resource/relationship/cross_domain.rs-152-        metadata: CrossDomainMetadata,
src//resource/relationship/cross_domain.rs-153-        bidirectional: bool,
--
src//resource/relationship/cross_domain.rs-174-            source_resource: source_resource.clone(),
src//resource/relationship/cross_domain.rs-175-            source_domain: source_domain.clone(),
src//resource/relationship/cross_domain.rs-176-            target_resource: target_resource.clone(),
src//resource/relationship/cross_domain.rs:177:            target_domain: target_domain.clone(),
src//resource/relationship/cross_domain.rs-178-            relationship_type_id: type_id,
src//resource/relationship/cross_domain.rs-179-            timestamp: now,
src//resource/relationship/cross_domain.rs-180-            nonce,
--
src//resource/relationship/cross_domain.rs-189-            source_resource,
src//resource/relationship/cross_domain.rs-190-            source_domain,
src//resource/relationship/cross_domain.rs-191-            target_resource,
src//resource/relationship/cross_domain.rs:192:            target_domain,
src//resource/relationship/cross_domain.rs-193-            relationship_type,
src//resource/relationship/cross_domain.rs-194-            metadata,
src//resource/relationship/cross_domain.rs-195-            bidirectional,
--
src//resource/relationship/cross_domain.rs-234-    source_domain_index: RwLock<HashMap<String, Vec<String>>>,
src//resource/relationship/cross_domain.rs-235-    
src//resource/relationship/cross_domain.rs-236-    // Index by target domain for querying
src//resource/relationship/cross_domain.rs:237:    target_domain_index: RwLock<HashMap<String, Vec<String>>>,
src//resource/relationship/cross_domain.rs-238-}
src//resource/relationship/cross_domain.rs-239-
src//resource/relationship/cross_domain.rs-240-impl CrossDomainRelationshipManager {
--
src//resource/relationship/cross_domain.rs-245-            source_index: RwLock::new(HashMap::new()),
src//resource/relationship/cross_domain.rs-246-            target_index: RwLock::new(HashMap::new()),
src//resource/relationship/cross_domain.rs-247-            source_domain_index: RwLock::new(HashMap::new()),
src//resource/relationship/cross_domain.rs:248:            target_domain_index: RwLock::new(HashMap::new()),
src//resource/relationship/cross_domain.rs-249-        }
src//resource/relationship/cross_domain.rs-250-    }
src//resource/relationship/cross_domain.rs-251-    
--
src//resource/relationship/cross_domain.rs-286-        
src//resource/relationship/cross_domain.rs-287-        // Update target domain index
src//resource/relationship/cross_domain.rs-288-        {
src//resource/relationship/cross_domain.rs:289:            let mut target_domain_index = self.target_domain_index.write().unwrap();
src//resource/relationship/cross_domain.rs:290:            target_domain_index
src//resource/relationship/cross_domain.rs:291:                .entry(relationship.target_domain.clone())
src//resource/relationship/cross_domain.rs-292-                .or_insert_with(Vec::new)
src//resource/relationship/cross_domain.rs-293-                .push(relationship.id.clone());
src//resource/relationship/cross_domain.rs-294-        }
--
src//resource/relationship/cross_domain.rs-348-        
src//resource/relationship/cross_domain.rs-349-        // Update target domain index
src//resource/relationship/cross_domain.rs-350-        {
src//resource/relationship/cross_domain.rs:351:            let mut target_domain_index = self.target_domain_index.write().unwrap();
src//resource/relationship/cross_domain.rs:352:            if let Some(ids) = target_domain_index.get_mut(&relationship.target_domain) {
src//resource/relationship/cross_domain.rs-353-                ids.retain(|id| id != relationship_id);
src//resource/relationship/cross_domain.rs-354-                if ids.is_empty() {
src//resource/relationship/cross_domain.rs:355:                    target_domain_index.remove(&relationship.target_domain);
src//resource/relationship/cross_domain.rs-356-                }
src//resource/relationship/cross_domain.rs-357-            }
src//resource/relationship/cross_domain.rs-358-        }
--
src//resource/relationship/cross_domain.rs-439-    }
src//resource/relationship/cross_domain.rs-440-    
src//resource/relationship/cross_domain.rs-441-    /// Get relationships by target domain
src//resource/relationship/cross_domain.rs:442:    pub fn get_relationships_by_target_domain(&self, domain: String) -> Result<Vec<CrossDomainRelationship>> {
src//resource/relationship/cross_domain.rs:443:        let target_domain_index = self.target_domain_index.read().unwrap();
src//resource/relationship/cross_domain.rs-444-        let relationships = self.relationships.read().unwrap();
src//resource/relationship/cross_domain.rs-445-        
src//resource/relationship/cross_domain.rs:446:        let rel_ids = match target_domain_index.get(&domain) {
src//resource/relationship/cross_domain.rs-447-            Some(ids) => ids,
src//resource/relationship/cross_domain.rs-448-            None => return Ok(Vec::new()),
src//resource/relationship/cross_domain.rs-449-        };
--
src//resource/relationship/cross_domain.rs-455-    }
src//resource/relationship/cross_domain.rs-456-    
src//resource/relationship/cross_domain.rs-457-    /// Get relationships between two domains
src//resource/relationship/cross_domain.rs:458:    pub fn get_relationships_between_domains(&self, source_domain: String, target_domain: String) -> Result<Vec<CrossDomainRelationship>> {
src//resource/relationship/cross_domain.rs-459-        let source_rels = self.get_relationships_by_source_domain(source_domain)?;
src//resource/relationship/cross_domain.rs-460-        
src//resource/relationship/cross_domain.rs-461-        Ok(source_rels
src//resource/relationship/cross_domain.rs-462-            .into_iter()
src//resource/relationship/cross_domain.rs:463:            .filter(|rel| rel.target_domain == target_domain)
src//resource/relationship/cross_domain.rs-464-            .collect())
src//resource/relationship/cross_domain.rs-465-    }
src//resource/relationship/cross_domain.rs-466-    
--
src//resource/relationship/query.rs-595-        source_id: &ContentId,
src//resource/relationship/query.rs-596-        target_id: &ContentId,
src//resource/relationship/query.rs-597-        source_domain: &DomainId,
src//resource/relationship/query.rs:598:        target_domain: &DomainId,
src//resource/relationship/query.rs-599-    ) -> Result<Vec<RelationshipPath>> {
src//resource/relationship/query.rs:600:        if source_domain == target_domain {
src//resource/relationship/query.rs-601-            // If resources are in the same domain, use regular path finding
src//resource/relationship/query.rs-602-            let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
src//resource/relationship/query.rs-603-                .include_domain(source_domain.clone());
--
src//resource/relationship/query.rs-607-        
src//resource/relationship/query.rs-608-        // For cross-domain paths, we need to find boundary resources
src//resource/relationship/query.rs-609-        let source_domain_resources = self.get_resources_in_domain(source_domain)?;
src//resource/relationship/query.rs:610:        let target_domain_resources = self.get_resources_in_domain(target_domain)?;
src//resource/relationship/query.rs-611-        
src//resource/relationship/query.rs-612-        // Find paths from source to all resources in source domain
src//resource/relationship/query.rs-613-        let query1 = RelationshipQuery::from_source(source_id.clone())
--
src//resource/relationship/query.rs-618-        // Find paths from all resources in target domain to target
src//resource/relationship/query.rs-619-        let query2 = RelationshipQuery::new(target_id.clone(), target_id.clone())
src//resource/relationship/query.rs-620-            .find_all_paths(true)
src//resource/relationship/query.rs:621:            .include_domain(target_domain.clone());
src//resource/relationship/query.rs-622-            
src//resource/relationship/query.rs-623-        let target_paths = self.find_resources_reaching(target_id, &query2)?;
src//resource/relationship/query.rs-624-        
--
src//resource/relationship/query.rs-811-                let query = parsed_query.to_relationship_query()?;
src//resource/relationship/query.rs-812-                executor.execute(&query)
src//resource/relationship/query.rs-813-            },
src//resource/relationship/query.rs:814:            QueryOperation::FindCrossDomainPath(source, target, source_domain, target_domain) => {
src//resource/relationship/query.rs:815:                executor.find_cross_domain_path(source, target, source_domain, target_domain)
src//resource/relationship/query.rs-816-            },
src//resource/relationship/query.rs-817-            QueryOperation::PathExists(source, target) => {
src//resource/relationship/query.rs-818-                let query = parsed_query.to_relationship_query()?;
--
src//resource/relationship/sync.rs-190-    pub source_domain: DomainId,
src//resource/relationship/sync.rs-191-    
src//resource/relationship/sync.rs-192-    /// Target domain
src//resource/relationship/sync.rs:193:    pub target_domain: DomainId,
src//resource/relationship/sync.rs-194-    
src//resource/relationship/sync.rs-195-    /// Synchronization result
src//resource/relationship/sync.rs-196-    pub result: SyncResult,
--
src//resource/relationship/sync.rs-243-    pub fn register_sync_handler(
src//resource/relationship/sync.rs-244-        &self,
src//resource/relationship/sync.rs-245-        source_domain: &str,
src//resource/relationship/sync.rs:246:        target_domain: &str,
src//resource/relationship/sync.rs-247-        handler: SyncHandlerFn,
src//resource/relationship/sync.rs-248-    ) -> Result<()> {
src//resource/relationship/sync.rs-249-        let mut handlers = self.sync_handlers.write().unwrap();
src//resource/relationship/sync.rs:250:        handlers.insert((source_domain.to_string(), target_domain.to_string()), handler);
src//resource/relationship/sync.rs-251-        Ok(())
src//resource/relationship/sync.rs-252-    }
src//resource/relationship/sync.rs-253-    
--
src//resource/relationship/sync.rs-256-        // Log the synchronization attempt
src//resource/relationship/sync.rs-257-        info!(
src//resource/relationship/sync.rs-258-            "Synchronizing relationship from {} to {} (type: {:?})",
src//resource/relationship/sync.rs:259:            relationship.source_domain, relationship.target_domain, relationship.relationship_type
src//resource/relationship/sync.rs-260-        );
src//resource/relationship/sync.rs-261-        
src//resource/relationship/sync.rs-262-        // Mark as pending
--
src//resource/relationship/sync.rs-289-            let history_entry = SyncHistoryEntry {
src//resource/relationship/sync.rs-290-                relationship_id: relationship.id.clone(),
src//resource/relationship/sync.rs-291-                source_domain: relationship.source_domain.clone(),
src//resource/relationship/sync.rs:292:                target_domain: relationship.target_domain.clone(),
src//resource/relationship/sync.rs-293-                result: sync_result.clone(),
src//resource/relationship/sync.rs-294-            };
src//resource/relationship/sync.rs-295-            
--
src//resource/relationship/sync.rs-327-        // Check if the target resource exists
src//resource/relationship/sync.rs-328-        let target_exists = self.resource_exists(
src//resource/relationship/sync.rs-329-            &relationship.target_resource,
src//resource/relationship/sync.rs:330:            &relationship.target_domain,
src//resource/relationship/sync.rs-331-        )?;
src//resource/relationship/sync.rs-332-        
src//resource/relationship/sync.rs-333-        // Create or update the target resource
--
src//resource/relationship/sync.rs-335-            // Update the target resource to match the source
src//resource/relationship/sync.rs-336-            debug!(
src//resource/relationship/sync.rs-337-                "Updating mirrored resource {} in domain {}",
src//resource/relationship/sync.rs:338:                relationship.target_resource, relationship.target_domain
src//resource/relationship/sync.rs-339-            );
src//resource/relationship/sync.rs-340-            
src//resource/relationship/sync.rs-341-            // Use operation manager to update the resource
src//resource/relationship/sync.rs-342-            // This is a simplified example - in a real system, we would generate
src//resource/relationship/sync.rs-343-            // the appropriate operation based on the resource type and state
src//resource/relationship/sync.rs-344-            let op_result = self.operation_manager.create_update_operation(
src//resource/relationship/sync.rs:345:                relationship.target_domain.clone(),
src//resource/relationship/sync.rs-346-                relationship.target_resource.clone(),
src//resource/relationship/sync.rs-347-                source_resource,
src//resource/relationship/sync.rs-348-            );
--
src//resource/relationship/sync.rs-361-            // Create the target resource as a mirror of the source
src//resource/relationship/sync.rs-362-            debug!(
src//resource/relationship/sync.rs-363-                "Creating mirrored resource {} in domain {}",
src//resource/relationship/sync.rs:364:                relationship.target_resource, relationship.target_domain
src//resource/relationship/sync.rs-365-            );
src//resource/relationship/sync.rs-366-            
src//resource/relationship/sync.rs-367-            // Use operation manager to create the resource
src//resource/relationship/sync.rs-368-            let op_result = self.operation_manager.create_mirror_operation(
src//resource/relationship/sync.rs-369-                relationship.source_domain.clone(),
src//resource/relationship/sync.rs-370-                relationship.source_resource.clone(),
src//resource/relationship/sync.rs:371:                relationship.target_domain.clone(),
src//resource/relationship/sync.rs-372-                relationship.target_resource.clone(),
src//resource/relationship/sync.rs-373-            );
src//resource/relationship/sync.rs-374-            
--
src//resource/relationship/sync.rs-398-        // This is a simplified placeholder implementation
src//resource/relationship/sync.rs-399-        debug!(
src//resource/relationship/sync.rs-400-            "Synchronizing ownership relationship from {} to {}",
src//resource/relationship/sync.rs:401:            relationship.source_domain, relationship.target_domain
src//resource/relationship/sync.rs-402-        );
src//resource/relationship/sync.rs-403-        
src//resource/relationship/sync.rs-404-        Ok(SyncResult::success().with_metadata(
--
src//resource/relationship/sync.rs-419-        // This is a simplified placeholder implementation
src//resource/relationship/sync.rs-420-        debug!(
src//resource/relationship/sync.rs-421-            "Synchronizing derived relationship from {} to {}",
src//resource/relationship/sync.rs:422:            relationship.source_domain, relationship.target_domain
src//resource/relationship/sync.rs-423-        );
src//resource/relationship/sync.rs-424-        
src//resource/relationship/sync.rs-425-        Ok(SyncResult::success().with_metadata(
--
src//resource/relationship/sync.rs-440-        // This is a simplified placeholder implementation
src//resource/relationship/sync.rs-441-        debug!(
src//resource/relationship/sync.rs-442-            "Synchronizing bridge relationship between {} and {}",
src//resource/relationship/sync.rs:443:            relationship.source_domain, relationship.target_domain
src//resource/relationship/sync.rs-444-        );
src//resource/relationship/sync.rs-445-        
src//resource/relationship/sync.rs-446-        Ok(SyncResult::success().with_metadata(
--
src//resource/relationship/sync.rs-550-                SyncDirection::SourceToTarget => {
src//resource/relationship/sync.rs-551-                    handlers.get(&(
src//resource/relationship/sync.rs-552-                        relationship.source_domain.clone(),
src//resource/relationship/sync.rs:553:                        relationship.target_domain.clone(),
src//resource/relationship/sync.rs-554-                    )).cloned()
src//resource/relationship/sync.rs-555-                },
src//resource/relationship/sync.rs-556-                SyncDirection::TargetToSource => {
src//resource/relationship/sync.rs-557-                    handlers.get(&(
src//resource/relationship/sync.rs:558:                        relationship.target_domain.clone(),
src//resource/relationship/sync.rs-559-                        relationship.source_domain.clone(),
src//resource/relationship/sync.rs-560-                    )).cloned()
src//resource/relationship/sync.rs-561-                },
--
src//resource/relationship/sync.rs-564-                    // Let's start with source to target
src//resource/relationship/sync.rs-565-                    handlers.get(&(
src//resource/relationship/sync.rs-566-                        relationship.source_domain.clone(),
src//resource/relationship/sync.rs:567:                        relationship.target_domain.clone(),
src//resource/relationship/sync.rs-568-                    )).cloned()
src//resource/relationship/sync.rs-569-                },
src//resource/relationship/sync.rs-570-            }
--
src//resource/relationship/sync.rs-583-        } else {
src//resource/relationship/sync.rs-584-            Err(SyncError::NoSyncHandler(
src//resource/relationship/sync.rs-585-                relationship.source_domain.clone(),
src//resource/relationship/sync.rs:586:                relationship.target_domain.clone(),
src//resource/relationship/sync.rs-587-            ).into())
src//resource/relationship/sync.rs-588-        }
src//resource/relationship/sync.rs-589-    }
--
src//resource/relationship/sync.rs-638-        
src//resource/relationship/sync.rs-639-        fn create_update_operation(
src//resource/relationship/sync.rs-640-            &self,
src//resource/relationship/sync.rs:641:            target_domain: String,
src//resource/relationship/sync.rs-642-            target_resource: String,
src//resource/relationship/sync.rs-643-            _state: HashMap<String, String>,
src//resource/relationship/sync.rs-644-        ) -> Result<()> {
src//resource/relationship/sync.rs-645-            let mut ops = self.operations.lock().unwrap();
src//resource/relationship/sync.rs:646:            ops.push(format!("update:{}:{}", target_domain, target_resource));
src//resource/relationship/sync.rs-647-            Ok(())
src//resource/relationship/sync.rs-648-        }
src//resource/relationship/sync.rs-649-        
--
src//resource/relationship/sync.rs-651-            &self,
src//resource/relationship/sync.rs-652-            source_domain: String,
src//resource/relationship/sync.rs-653-            source_resource: String,
src//resource/relationship/sync.rs:654:            target_domain: String,
src//resource/relationship/sync.rs-655-            target_resource: String,
src//resource/relationship/sync.rs-656-        ) -> Result<()> {
src//resource/relationship/sync.rs-657-            let mut ops = self.operations.lock().unwrap();
src//resource/relationship/sync.rs-658-            ops.push(format!(
src//resource/relationship/sync.rs-659-                "mirror:{}:{}:{}:{}",
src//resource/relationship/sync.rs:660:                source_domain, source_resource, target_domain, target_resource
src//resource/relationship/sync.rs-661-            ));
src//resource/relationship/sync.rs-662-            Ok(())
src//resource/relationship/sync.rs-663-        }
--
src//resource/relationship/sync.rs-708-            source_resource: "resource1".to_string(),
src//resource/relationship/sync.rs-709-            source_domain: "domain1".to_string(),
src//resource/relationship/sync.rs-710-            target_resource: "resource1-mirror".to_string(),
src//resource/relationship/sync.rs:711:            target_domain: "domain2".to_string(),
src//resource/relationship/sync.rs-712-            relationship_type: CrossDomainRelationshipType::Mirror,
src//resource/relationship/sync.rs-713-            metadata: CrossDomainMetadata {
src//resource/relationship/sync.rs-714-                requires_sync: true,
src//resource/relationship/sync.rs-715-                sync_strategy: SyncStrategy::OneTime,
src//resource/relationship/sync.rs-716-                sync_frequency: None,
src//resource/relationship/sync.rs-717-                origin_domain: "domain1".to_string(),
src//resource/relationship/sync.rs:718:                target_domain: "domain2".to_string(),
src//resource/relationship/sync.rs-719-            },
src//resource/relationship/sync.rs-720-            bidirectional: false,
src//resource/relationship/sync.rs-721-        };
--
src//resource/relationship/sync.rs-742-            source_resource: "resource1".to_string(),
src//resource/relationship/sync.rs-743-            source_domain: "domain1".to_string(),
src//resource/relationship/sync.rs-744-            target_resource: "resource1-mirror".to_string(),
src//resource/relationship/sync.rs:745:            target_domain: "domain2".to_string(),
src//resource/relationship/sync.rs-746-            relationship_type: CrossDomainRelationshipType::Mirror,
src//resource/relationship/sync.rs-747-            metadata: CrossDomainMetadata {
src//resource/relationship/sync.rs-748-                requires_sync: true,
src//resource/relationship/sync.rs-749-                sync_strategy: SyncStrategy::OneTime,
src//resource/relationship/sync.rs-750-                sync_frequency: None,
src//resource/relationship/sync.rs-751-                origin_domain: "domain1".to_string(),
src//resource/relationship/sync.rs:752:                target_domain: "domain2".to_string(),
src//resource/relationship/sync.rs-753-            },
src//resource/relationship/sync.rs-754-            bidirectional: false,
src//resource/relationship/sync.rs-755-        };
--
src//resource/relationship/mod.rs-188-    pub fn to_cross_domain(
src//resource/relationship/mod.rs-189-        &self,
src//resource/relationship/mod.rs-190-        source_domain: String,
src//resource/relationship/mod.rs:191:        target_domain: String,
src//resource/relationship/mod.rs-192-        cross_domain_type: CrossDomainRelationshipType,
src//resource/relationship/mod.rs-193-        metadata: CrossDomainMetadata,
src//resource/relationship/mod.rs-194-        bidirectional: bool,
--
src//resource/relationship/mod.rs-198-            self.source.clone(),
src//resource/relationship/mod.rs-199-            source_domain,
src//resource/relationship/mod.rs-200-            self.target.clone(),
src//resource/relationship/mod.rs:201:            target_domain,
src//resource/relationship/mod.rs-202-            cross_domain_type,
src//resource/relationship/mod.rs-203-            metadata,
src//resource/relationship/mod.rs-204-            bidirectional,
--
src//resource/relationship/mod.rs-238-        source: String,
src//resource/relationship/mod.rs-239-        source_domain: String,
src//resource/relationship/mod.rs-240-        target: String,
src//resource/relationship/mod.rs:241:        target_domain: String,
src//resource/relationship/mod.rs-242-        cross_domain_type: CrossDomainRelationshipType,
src//resource/relationship/mod.rs-243-        metadata: CrossDomainMetadata,
src//resource/relationship/mod.rs-244-        bidirectional: bool,
--
src//resource/relationship/mod.rs-253-        // Convert to cross-domain relationship
src//resource/relationship/mod.rs-254-        relationship.to_cross_domain(
src//resource/relationship/mod.rs-255-            source_domain,
src//resource/relationship/mod.rs:256:            target_domain,
src//resource/relationship/mod.rs-257-            cross_domain_type,
src//resource/relationship/mod.rs-258-            metadata,
src//resource/relationship/mod.rs-259-            bidirectional,
--
src//resource/relationship/mod.rs-292-        
src//resource/relationship/mod.rs-293-        let metadata = CrossDomainMetadata {
src//resource/relationship/mod.rs-294-            origin_domain: "domain1".to_string(),
src//resource/relationship/mod.rs:295:            target_domain: "domain2".to_string(),
src//resource/relationship/mod.rs-296-            requires_sync: true,
src//resource/relationship/mod.rs-297-            sync_strategy: SyncStrategy::EventDriven,
src//resource/relationship/mod.rs-298-        };
--
src//resource/relationship/mod.rs-308-        assert_eq!(cross_domain.source_resource, "resource1");
src//resource/relationship/mod.rs-309-        assert_eq!(cross_domain.target_resource, "resource2");
src//resource/relationship/mod.rs-310-        assert_eq!(cross_domain.source_domain, "domain1");
src//resource/relationship/mod.rs:311:        assert_eq!(cross_domain.target_domain, "domain2");
src//resource/relationship/mod.rs-312-        assert_eq!(cross_domain.relationship_type, CrossDomainRelationshipType::Reference);
src//resource/relationship/mod.rs-313-        assert!(cross_domain.bidirectional);
src//resource/relationship/mod.rs-314-        
--
src//resource/relationship/validation.rs-380-        }
src//resource/relationship/validation.rs-381-        
src//resource/relationship/validation.rs-382-        // Check for empty target domain
src//resource/relationship/validation.rs:383:        if relationship.target_domain.is_empty() {
src//resource/relationship/validation.rs:384:            result.add_error(ValidationError::MissingField("target_domain".to_string()));
src//resource/relationship/validation.rs-385-        }
src//resource/relationship/validation.rs-386-    }
src//resource/relationship/validation.rs-387-    
--
src//resource/relationship/validation.rs-509-        result: &mut ValidationResult,
src//resource/relationship/validation.rs-510-    ) {
src//resource/relationship/validation.rs-511-        // Source and target domains should be different for cross-domain relationships
src//resource/relationship/validation.rs:512:        if relationship.source_domain == relationship.target_domain {
src//resource/relationship/validation.rs-513-            match level {
src//resource/relationship/validation.rs-514-                ValidationLevel::Strict => {
src//resource/relationship/validation.rs-515-                    result.add_error(ValidationError::InvalidDomain(
--
src//resource/relationship/scheduler.rs-121-    pub source_domain: DomainId,
src//resource/relationship/scheduler.rs-122-    
src//resource/relationship/scheduler.rs-123-    /// Target domain
src//resource/relationship/scheduler.rs:124:    pub target_domain: DomainId,
src//resource/relationship/scheduler.rs-125-    
src//resource/relationship/scheduler.rs-126-    /// When the task was scheduled
src//resource/relationship/scheduler.rs-127-    pub scheduled_at: Instant,
--
src//resource/relationship/scheduler.rs-349-        let task = ScheduledTask {
src//resource/relationship/scheduler.rs-350-            relationship_id: relationship_id.to_string(),
src//resource/relationship/scheduler.rs-351-            source_domain: relationship.source_domain.clone(),
src//resource/relationship/scheduler.rs:352:            target_domain: relationship.target_domain.clone(),
src//resource/relationship/scheduler.rs-353-            scheduled_at: Instant::now(),
src//resource/relationship/scheduler.rs-354-            execute_at: Instant::now(),
src//resource/relationship/scheduler.rs-355-            retry_attempt: 0,
--
src//resource/relationship/scheduler.rs-445-                let task = ScheduledTask {
src//resource/relationship/scheduler.rs-446-                    relationship_id: relationship_id.clone(),
src//resource/relationship/scheduler.rs-447-                    source_domain: relationship.source_domain.clone(),
src//resource/relationship/scheduler.rs:448:                    target_domain: relationship.target_domain.clone(),
src//resource/relationship/scheduler.rs-449-                    scheduled_at: Instant::now(),
src//resource/relationship/scheduler.rs-450-                    execute_at: Instant::now(),
src//resource/relationship/scheduler.rs-451-                    retry_attempt: 0,
--
src//resource/relationship/scheduler.rs-665-                    let retry_task = ScheduledTask {
src//resource/relationship/scheduler.rs-666-                        relationship_id: task.relationship_id.clone(),
src//resource/relationship/scheduler.rs-667-                        source_domain: task.source_domain.clone(),
src//resource/relationship/scheduler.rs:668:                        target_domain: task.target_domain.clone(),
src//resource/relationship/scheduler.rs-669-                        scheduled_at: Instant::now(),
src//resource/relationship/scheduler.rs-670-                        execute_at: Instant::now() + backoff,
src//resource/relationship/scheduler.rs-671-                        retry_attempt: next_attempt,
--
src//domain/resource_integration.rs-26-    /// Store a resource in a domain
src//domain/resource_integration.rs-27-    Store {
src//domain/resource_integration.rs-28-        resource_id: ContentId,
src//domain/resource_integration.rs:29:        target_domain_id: DomainId,
src//domain/resource_integration.rs-30-        contents: Vec<u8>,
src//domain/resource_integration.rs-31-        metadata: HashMap<String, String>,
src//domain/resource_integration.rs-32-    },
--
src//domain/resource_integration.rs-41-    Transfer {
src//domain/resource_integration.rs-42-        resource_id: ContentId,
src//domain/resource_integration.rs-43-        source_domain_id: DomainId,
src//domain/resource_integration.rs:44:        target_domain_id: DomainId,
src//domain/resource_integration.rs-45-        metadata: HashMap<String, String>,
src//domain/resource_integration.rs-46-    },
src//domain/resource_integration.rs-47-    
--
src//domain/resource_integration.rs-76-    Transferred {
src//domain/resource_integration.rs-77-        resource_id: ContentId,
src//domain/resource_integration.rs-78-        source_domain_id: DomainId,
src//domain/resource_integration.rs:79:        target_domain_id: DomainId,
src//domain/resource_integration.rs-80-        transaction_id: String,
src//domain/resource_integration.rs-81-        block_height: Option<BlockHeight>,
src//domain/resource_integration.rs-82-        timestamp: Option<Timestamp>,
--
src//domain/resource_integration.rs-488-    ) -> Result<CrossDomainResourceResult> {
src//domain/resource_integration.rs-489-        // Handle operation based on type
src//domain/resource_integration.rs-490-        match &operation {
src//domain/resource_integration.rs:491:            CrossDomainResourceOperation::Store { resource_id, target_domain_id, contents, metadata } => {
src//domain/resource_integration.rs-492-                // Create adapter for target domain
src//domain/resource_integration.rs:493:                let adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
src//domain/resource_integration.rs-494-                
src//domain/resource_integration.rs-495-                // Validate operation
src//domain/resource_integration.rs-496-                if !adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-497-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-498-                        "Operation not allowed for resource {} in domain {}", 
src//domain/resource_integration.rs:499:                        resource_id, target_domain_id
src//domain/resource_integration.rs-500-                    )));
src//domain/resource_integration.rs-501-                }
src//domain/resource_integration.rs-502-                
--
src//domain/resource_integration.rs-523-            CrossDomainResourceOperation::Transfer { 
src//domain/resource_integration.rs-524-                resource_id, 
src//domain/resource_integration.rs-525-                source_domain_id, 
src//domain/resource_integration.rs:526:                target_domain_id, 
src//domain/resource_integration.rs-527-                metadata 
src//domain/resource_integration.rs-528-            } => {
src//domain/resource_integration.rs-529-                // Create adapters for source and target domains
src//domain/resource_integration.rs-530-                let source_adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
src//domain/resource_integration.rs:531:                let target_adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
src//domain/resource_integration.rs-532-                
src//domain/resource_integration.rs-533-                // Validate operations
src//domain/resource_integration.rs-534-                if !source_adapter.validate_operation(resource_id, &operation).await? {
--
src//domain/resource_integration.rs-541-                if !target_adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-542-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-543-                        "Transfer operation not allowed for resource {} in target domain {}", 
src//domain/resource_integration.rs:544:                        resource_id, target_domain_id
src//domain/resource_integration.rs-545-                    )));
src//domain/resource_integration.rs-546-                }
src//domain/resource_integration.rs-547-                
--
src//domain/resource_integration.rs-564-                        Ok(CrossDomainResourceResult::Transferred {
src//domain/resource_integration.rs-565-                            resource_id: resource_id.clone(),
src//domain/resource_integration.rs-566-                            source_domain_id: source_domain_id.clone(),
src//domain/resource_integration.rs:567:                            target_domain_id: target_domain_id.clone(),
src//domain/resource_integration.rs-568-                            transaction_id,
src//domain/resource_integration.rs-569-                            block_height,
src//domain/resource_integration.rs-570-                            timestamp,
--
src//domain/resource_integration.rs-614-        // Create storage operation
src//domain/resource_integration.rs-615-        let operation = CrossDomainResourceOperation::Store {
src//domain/resource_integration.rs-616-            resource_id: resource_id.clone(),
src//domain/resource_integration.rs:617:            target_domain_id: adapter.domain_id().clone(),
src//domain/resource_integration.rs-618-            contents,
src//domain/resource_integration.rs-619-            metadata,
src//domain/resource_integration.rs-620-        };
--
src//domain/content_addressed_transaction.rs-63-    pub origin_domain: DomainId,
src//domain/content_addressed_transaction.rs-64-    
src//domain/content_addressed_transaction.rs-65-    /// Target domain (if cross-domain)
src//domain/content_addressed_transaction.rs:66:    pub target_domain: Option<DomainId>,
src//domain/content_addressed_transaction.rs-67-    
src//domain/content_addressed_transaction.rs-68-    /// Transaction type
src//domain/content_addressed_transaction.rs-69-    pub transaction_type: String,
--
src//domain/content_addressed_transaction.rs-87-            id,
src//domain/content_addressed_transaction.rs-88-            data,
src//domain/content_addressed_transaction.rs-89-            origin_domain,
src//domain/content_addressed_transaction.rs:90:            target_domain: None,
src//domain/content_addressed_transaction.rs-91-            transaction_type,
src//domain/content_addressed_transaction.rs-92-            timestamp: std::time::SystemTime::now()
src//domain/content_addressed_transaction.rs-93-                .duration_since(std::time::UNIX_EPOCH)
--
src//domain/content_addressed_transaction.rs-98-    }
src//domain/content_addressed_transaction.rs-99-    
src//domain/content_addressed_transaction.rs-100-    /// Set target domain
src//domain/content_addressed_transaction.rs:101:    pub fn with_target_domain(mut self, target_domain: DomainId) -> Self {
src//domain/content_addressed_transaction.rs:102:        self.target_domain = Some(target_domain);
src//domain/content_addressed_transaction.rs-103-        self
src//domain/content_addressed_transaction.rs-104-    }
src//domain/content_addressed_transaction.rs-105-    
--
src//domain/content_addressed_transaction.rs-153-    pub origin_domain: DomainId,
src//domain/content_addressed_transaction.rs-154-    
src//domain/content_addressed_transaction.rs-155-    /// Target domain
src//domain/content_addressed_transaction.rs:156:    pub target_domain: Option<DomainId>,
src//domain/content_addressed_transaction.rs-157-    
src//domain/content_addressed_transaction.rs-158-    /// Proof bundle
src//domain/content_addressed_transaction.rs-159-    pub proof: Option<CommitmentProof>,
--
src//domain/content_addressed_transaction.rs-216-            .map_err(|e| TransactionVerificationError::DomainError(e))?;
src//domain/content_addressed_transaction.rs-217-        
src//domain/content_addressed_transaction.rs-218-        // If the transaction has a target domain, verify it there as well
src//domain/content_addressed_transaction.rs:219:        let target_receipt = if let Some(target_domain) = &transaction.target_domain {
src//domain/content_addressed_transaction.rs-220-            // Get the target domain adapter
src//domain/content_addressed_transaction.rs:221:            let target_adapter = self.registry.get_adapter(target_domain)
src//domain/content_addressed_transaction.rs-222-                .map_err(|e| TransactionVerificationError::DomainError(e))?;
src//domain/content_addressed_transaction.rs-223-            
src//domain/content_addressed_transaction.rs-224-            // Check if the transaction exists in the target domain
--
src//domain/content_addressed_transaction.rs-246-            transaction_id: transaction.id.clone(),
src//domain/content_addressed_transaction.rs-247-            status,
src//domain/content_addressed_transaction.rs-248-            origin_domain: transaction.origin_domain.clone(),
src//domain/content_addressed_transaction.rs:249:            target_domain: transaction.target_domain.clone(),
src//domain/content_addressed_transaction.rs-250-            proof: Some(proof),
src//domain/content_addressed_transaction.rs-251-            receipt: target_receipt,
src//domain/content_addressed_transaction.rs-252-        })
--
src//domain/content_addressed_transaction.rs-417-        
src//domain/content_addressed_transaction.rs-418-        // Create domain adapters
src//domain/content_addressed_transaction.rs-419-        let origin_domain = DomainId::new("origin-domain");
src//domain/content_addressed_transaction.rs:420:        let target_domain = DomainId::new("target-domain");
src//domain/content_addressed_transaction.rs-421-        
src//domain/content_addressed_transaction.rs-422-        let origin_adapter = Arc::new(MockDomainAdapter::new(origin_domain.clone()));
src//domain/content_addressed_transaction.rs:423:        let target_adapter = Arc::new(MockDomainAdapter::new(target_domain.clone()));
src//domain/content_addressed_transaction.rs-424-        
src//domain/content_addressed_transaction.rs-425-        // Register adapters
src//domain/content_addressed_transaction.rs-426-        registry.register_adapter(origin_adapter.clone()).unwrap();
--
src//domain/content_addressed_transaction.rs-433-            vec![1, 2, 3, 4],
src//domain/content_addressed_transaction.rs-434-            origin_domain.clone(),
src//domain/content_addressed_transaction.rs-435-            "transfer".to_string()
src//domain/content_addressed_transaction.rs:436:        ).with_target_domain(target_domain.clone());
src//domain/content_addressed_transaction.rs-437-        
src//domain/content_addressed_transaction.rs-438-        // Add receipts to both domains
src//domain/content_addressed_transaction.rs-439-        let origin_receipt = TransactionReceipt {
--
src//domain/content_addressed_transaction.rs-471-        assert_eq!(result.transaction_id, tx_id);
src//domain/content_addressed_transaction.rs-472-        assert!(matches!(result.status, TransactionStatus::Success));
src//domain/content_addressed_transaction.rs-473-        assert_eq!(result.origin_domain, origin_domain);
src//domain/content_addressed_transaction.rs:474:        assert_eq!(result.target_domain, Some(target_domain));
src//domain/content_addressed_transaction.rs-475-        assert!(result.proof.is_some());
src//domain/content_addressed_transaction.rs-476-        assert!(result.receipt.is_some());
src//domain/content_addressed_transaction.rs-477-        
--
src//domain/content_addressed_transaction.rs-531-            vec![1, 2, 3, 4],
src//domain/content_addressed_transaction.rs-532-            DomainId::new("test-domain"),
src//domain/content_addressed_transaction.rs-533-            "transfer".to_string()
src//domain/content_addressed_transaction.rs:534:        ).with_target_domain(DomainId::new("target-domain"))
src//domain/content_addressed_transaction.rs-535-         .with_metadata("key1", "value1")
src//domain/content_addressed_transaction.rs-536-         .with_metadata("key2", "value2");
src//domain/content_addressed_transaction.rs-537-        
--
src//domain/content_addressed_transaction.rs-560-        assert_eq!(deserialized.id, tx_id);
src//domain/content_addressed_transaction.rs-561-        assert_eq!(deserialized.data, vec![1, 2, 3, 4]);
src//domain/content_addressed_transaction.rs-562-        assert_eq!(deserialized.origin_domain, DomainId::new("test-domain"));
src//domain/content_addressed_transaction.rs:563:        assert_eq!(deserialized.target_domain, Some(DomainId::new("target-domain")));
src//domain/content_addressed_transaction.rs-564-        assert_eq!(deserialized.transaction_type, "transfer".to_string());
src//domain/content_addressed_transaction.rs-565-        assert_eq!(deserialized.metadata.get("key1"), Some(&"value1".to_string()));
src//domain/content_addressed_transaction.rs-566-        assert_eq!(deserialized.metadata.get("key2"), Some(&"value2".to_string()));
```

## Recommended Migration Pattern

Replace these patterns with the unified ResourceRegister approach:
```rust
// OLD PATTERN:
// Create a resource representation on the target domain
let target_resource = source_resource.for_domain(target_domain);
target_domain_resource_manager.create_resource(target_resource)?;

// Create a register on the target domain
let target_register = Register::new_with_controller(source_register.controller_label);
target_register_system.create_register(target_register.id, target_resource.id)?;

// Update source register state
source_register_system.consume_register(source_register.id)?;

// NEW PATTERN:
// Create a transfer representation
let transfer = ResourceRegister::for_transfer(
    source_token,
    target_domain,
    controller_label
);

// Execute cross-domain effect
effect_system.execute_effect(StorageEffect::TransferCrossDomain {
    source_token: source_token,
    target_domain: target_domain,
    continuation: Box::new(|result| {
        // Handle transfer result with atomic operations on both domains
    }),
}).await?;
```

