# Resource and Register Creation Pattern Report

Generated on Sun Mar 23 19:02:32 CST 2025

## Summary

### Resource Creation Patterns

Found the following Resource::new patterns:

```rust
src//resource/content_addressed_resource.rs-274-    fn test_resource_content_addressing() {
src//resource/content_addressed_resource.rs-275-        // Create a resource
src//resource/content_addressed_resource.rs:276:        let mut resource = Resource::new("test", "document", b"resource data".to_vec());
src//resource/content_addressed_resource.rs-277-        resource.set_metadata("author", "Test User");
src//resource/content_addressed_resource.rs-278-        
--
src//resource/content_addressed_resource.rs-281-        
src//resource/content_addressed_resource.rs-282-        // Create an identical resource
src//resource/content_addressed_resource.rs:283:        let mut resource2 = Resource::new("test", "document", b"resource data".to_vec());
src//resource/content_addressed_resource.rs-284-        resource2.set_metadata("author", "Test User");
src//resource/content_addressed_resource.rs-285-        
--
src//resource/content_addressed_resource.rs-288-        
src//resource/content_addressed_resource.rs-289-        // Create a different resource
src//resource/content_addressed_resource.rs:290:        let resource3 = Resource::new("different", "document", b"different data".to_vec());
src//resource/content_addressed_resource.rs-291-        
src//resource/content_addressed_resource.rs-292-        // The content hash should be different
--
src//resource/content_addressed_resource.rs-307-    fn test_resource_content_id() {
src//resource/content_addressed_resource.rs-308-        // Create a resource
src//resource/content_addressed_resource.rs:309:        let resource = Resource::new("test", "document", b"resource data".to_vec());
src//resource/content_addressed_resource.rs-310-        
src//resource/content_addressed_resource.rs-311-        // Get the content ID
--
src//resource/content_addressed_resource.rs-330-        
src//resource/content_addressed_resource.rs-331-        // Create some resources
src//resource/content_addressed_resource.rs:332:        let resource1 = Resource::new("res1", "document", b"data1".to_vec());
src//resource/content_addressed_resource.rs:333:        let resource2 = Resource::new("res2", "document", b"data2".to_vec());
src//resource/content_addressed_resource.rs-334-        
src//resource/content_addressed_resource.rs-335-        // Register the resources
--
src//resource/content_addressed_resource.rs-364-    fn test_resource_versioning() {
src//resource/content_addressed_resource.rs-365-        // Create a resource
src//resource/content_addressed_resource.rs:366:        let mut resource = Resource::new("test", "document", b"initial data".to_vec());
src//resource/content_addressed_resource.rs-367-        assert_eq!(resource.version(), 1);
src//resource/content_addressed_resource.rs-368-        
--
src//resource/content_addressed_resource.rs-384-    fn test_migration_compatibility() {
src//resource/content_addressed_resource.rs-385-        // Create a resource using the old way
src//resource/content_addressed_resource.rs:386:        let mut resource = Resource::new("test", "document", b"resource data".to_vec());
src//resource/content_addressed_resource.rs-387-        resource.set_metadata("author", "Test User");
src//resource/content_addressed_resource.rs-388-        
```

### Register Creation Patterns

Found the following Register::new patterns:

```rust
src//tel/resource/model/manager.rs-94-            TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
src//tel/resource/model/manager.rs-95-        
src//tel/resource/model/manager.rs:96:        let register = Register::new(
src//tel/resource/model/manager.rs-97-            register_id,
src//tel/resource/model/manager.rs-98-            owner,
--
src//tel/resource/model/manager.rs-326-                    .clone();
src//tel/resource/model/manager.rs-327-                
src//tel/resource/model/manager.rs:328:                let register = Register::new(
src//tel/resource/model/manager.rs-329-                    operation.target,
src//tel/resource/model/manager.rs-330-                    operation.initiator,
--
src//effect/templates/relationship_validation_tests.rs-58-    
src//effect/templates/relationship_validation_tests.rs-59-    // Create a parent resource
src//effect/templates/relationship_validation_tests.rs:60:    let parent_resource = ResourceRegister::new(
src//effect/templates/relationship_validation_tests.rs-61-        parent_id.clone(),
src//effect/templates/relationship_validation_tests.rs-62-        ResourceLogic::new(),
--
src//effect/templates/relationship_validation_tests.rs-70-    
src//effect/templates/relationship_validation_tests.rs-71-    // Create a child resource
src//effect/templates/relationship_validation_tests.rs:72:    let child_resource = ResourceRegister::new(
src//effect/templates/relationship_validation_tests.rs-73-        child_id.clone(),
src//effect/templates/relationship_validation_tests.rs-74-        ResourceLogic::new(),
--
src//effect/templates/relationship_validation_tests.rs-145-    
src//effect/templates/relationship_validation_tests.rs-146-    // Create the resources
src//effect/templates/relationship_validation_tests.rs:147:    let dependent_resource = ResourceRegister::new(
src//effect/templates/relationship_validation_tests.rs-148-        dependent_id.clone(),
src//effect/templates/relationship_validation_tests.rs-149-        ResourceLogic::new(),
--
src//effect/templates/relationship_validation_tests.rs-156-    );
src//effect/templates/relationship_validation_tests.rs-157-    
src//effect/templates/relationship_validation_tests.rs:158:    let dependency_resource = ResourceRegister::new(
src//effect/templates/relationship_validation_tests.rs-159-        dependency_id.clone(),
src//effect/templates/relationship_validation_tests.rs-160-        ResourceLogic::new(),
--
src//effect/templates/relationship_validation_tests.rs-287-    // Create resources for each ID
src//effect/templates/relationship_validation_tests.rs-288-    let create_resource = |id: &ResourceId| -> ResourceRegister {
src//effect/templates/relationship_validation_tests.rs:289:        ResourceRegister::new(
src//effect/templates/relationship_validation_tests.rs-290-            id.clone(),
src//effect/templates/relationship_validation_tests.rs-291-            ResourceLogic::new(),
--
src//effect/storage.rs-215-        
src//effect/storage.rs-216-        // Create a mock ResourceRegister with the unified model's fields
src//effect/storage.rs:217:        let mock_register = ResourceRegister::new(
src//effect/storage.rs-218-            self.content_id.clone(),
src//effect/storage.rs-219-            crate::resource::resource_register::ResourceLogic::Fungible,
--
src//examples/lifecycle_capabilities.rs-34-    
src//examples/lifecycle_capabilities.rs-35-    // Create the resource register with content addressing
src//examples/lifecycle_capabilities.rs:36:    let register = ResourceRegister::new(
src//examples/lifecycle_capabilities.rs-37-        resource_logic,
src//examples/lifecycle_capabilities.rs-38-        fungibility_domain,
--
src//examples/boundary_aware_resources.rs-56-    
src//examples/boundary_aware_resources.rs-57-    // Create the resource register with content addressing
src//examples/boundary_aware_resources.rs:58:    let register = ResourceRegister::new(
src//examples/boundary_aware_resources.rs-59-        resource_logic,
src//examples/boundary_aware_resources.rs-60-        fungibility_domain,
--
src//resource/content_addressed_resource.rs-43-    /// Create a new resource
src//resource/content_addressed_resource.rs-44-    ///
src//resource/content_addressed_resource.rs:45:    /// MIGRATION NOTE: Consider using ResourceRegister::new directly
src//resource/content_addressed_resource.rs-46-    /// or the migration helper create_resource_register() instead.
src//resource/content_addressed_resource.rs-47-    pub fn new(name: impl Into<String>, resource_type: impl Into<String>, data: Vec<u8>) -> Self {
--
src//resource/relationship_tracker.rs-719-        };
src//resource/relationship_tracker.rs-720-        
src//resource/relationship_tracker.rs:721:        ResourceRegister::new(
src//resource/relationship_tracker.rs-722-            content_id,
src//resource/relationship_tracker.rs-723-            ResourceLogic::Fungible,
--
src//resource/tests/lifecycle_helper_tests.rs-14-    // Create a resource
src//resource/tests/lifecycle_helper_tests.rs-15-    let id = ResourceId::new("test-resource".to_string());
src//resource/tests/lifecycle_helper_tests.rs:16:    let resource = ResourceRegister::new(
src//resource/tests/lifecycle_helper_tests.rs-17-        id.clone(),
src//resource/tests/lifecycle_helper_tests.rs-18-        ResourceLogic::Data,
--
src//resource/tests/lifecycle_helper_tests.rs-51-    // Create a resource
src//resource/tests/lifecycle_helper_tests.rs-52-    let id = ResourceId::new("test-resource-freeze".to_string());
src//resource/tests/lifecycle_helper_tests.rs:53:    let resource = ResourceRegister::new(
src//resource/tests/lifecycle_helper_tests.rs-54-        id.clone(),
src//resource/tests/lifecycle_helper_tests.rs-55-        ResourceLogic::Data,
--
src//resource/tests/lifecycle_helper_tests.rs-88-    // Create a resource
src//resource/tests/lifecycle_helper_tests.rs-89-    let id = ResourceId::new("test-resource-invalid".to_string());
src//resource/tests/lifecycle_helper_tests.rs:90:    let resource = ResourceRegister::new(
src//resource/tests/lifecycle_helper_tests.rs-91-        id.clone(),
src//resource/tests/lifecycle_helper_tests.rs-92-        ResourceLogic::Data,
--
src//resource/tests/lifecycle_helper_tests.rs-128-    // Create a resource
src//resource/tests/lifecycle_helper_tests.rs-129-    let id = ResourceId::new("test-resource-async".to_string());
src//resource/tests/lifecycle_helper_tests.rs:130:    let resource = ResourceRegister::new(
src//resource/tests/lifecycle_helper_tests.rs-131-        id.clone(),
src//resource/tests/lifecycle_helper_tests.rs-132-        ResourceLogic::Data,
--
src//resource/tests/capability_tests.rs-164-    // Create a register
src//resource/tests/capability_tests.rs-165-    let register_id = RegisterId::new(address_gen.generate_unique());
src//resource/tests/capability_tests.rs:166:    let register = Register::new(
src//resource/tests/capability_tests.rs-167-        register_id.clone(),
src//resource/tests/capability_tests.rs-168-        RegisterContents::new(b"test data".to_vec()),
--
src//resource/tests/capability_tests.rs-214-    // Create a register
src//resource/tests/capability_tests.rs-215-    let register_id = RegisterId::new(address_gen.generate_unique());
src//resource/tests/capability_tests.rs:216:    let register = Register::new(
src//resource/tests/capability_tests.rs-217-        register_id.clone(),
src//resource/tests/capability_tests.rs-218-        RegisterContents::new(b"transferable data".to_vec()),
--
src//resource/tests/capability_tests.rs-272-    // Create a register
src//resource/tests/capability_tests.rs-273-    let register_id = RegisterId::new(address_gen.generate_unique());
src//resource/tests/capability_tests.rs:274:    let register = Register::new(
src//resource/tests/capability_tests.rs-275-        register_id.clone(),
src//resource/tests/capability_tests.rs-276-        RegisterContents::new(b"multi-transfer data".to_vec()),
--
src//resource/tests/storage_tests.rs-74-// Helper to create a test resource register
src//resource/tests/storage_tests.rs-75-fn create_test_resource_register(id: &str, logic: ResourceLogic, strategy: StorageStrategy) -> ResourceRegister {
src//resource/tests/storage_tests.rs:76:    ResourceRegister::new(
src//resource/tests/storage_tests.rs-77-        id.to_string(),
src//resource/tests/storage_tests.rs-78-        logic,
--
src//resource/tests/effect_template_integration_tests.rs-119-// Helper to create a test resource
src//resource/tests/effect_template_integration_tests.rs-120-fn create_test_resource(id: &str) -> ResourceRegister {
src//resource/tests/effect_template_integration_tests.rs:121:    ResourceRegister::new(
src//resource/tests/effect_template_integration_tests.rs-122-        ResourceId::from(id.to_string()),
src//resource/tests/effect_template_integration_tests.rs-123-        ResourceLogic::Fungible,
--
src//resource/tests/resource_register_tests.rs-22-// Helper function to create a test resource register
src//resource/tests/resource_register_tests.rs-23-fn create_test_register(name: &str, amount: u128) -> ResourceRegister {
src//resource/tests/resource_register_tests.rs:24:    ResourceRegister::new(
src//resource/tests/resource_register_tests.rs-25-        ResourceLogic::Fungible,
src//resource/tests/resource_register_tests.rs-26-        FungibilityDomain(format!("domain-{}", name)),
--
src//resource/tests/resource_register_tests.rs-47-    // Helper function to create a test register
src//resource/tests/resource_register_tests.rs-48-    fn create_test_register(id: &str) -> ResourceRegister {
src//resource/tests/resource_register_tests.rs:49:        let mut register = ResourceRegister::new_minimal();
src//resource/tests/resource_register_tests.rs-50-        register.domain_id = DomainId("test-domain".to_string());
src//resource/tests/resource_register_tests.rs-51-        register.metadata = HashMap::new();
--
src//resource/fact_observer.rs-352-    
src//resource/fact_observer.rs-353-    fn create_test_register() -> Register {
src//resource/fact_observer.rs:354:        Register::new(
src//resource/fact_observer.rs-355-            RegisterId::new_unique(),
src//resource/fact_observer.rs-356-            crate::types::Address::new("owner1"),
--
src//resource/content_addressed_register.rs-79-impl From<ContentAddressedRegister> for ResourceRegister {
src//resource/content_addressed_register.rs-80-    fn from(register: ContentAddressedRegister) -> Self {
src//resource/content_addressed_register.rs:81:        ResourceRegister::new(
src//resource/content_addressed_register.rs-82-            register.id.clone(),
src//resource/content_addressed_register.rs-83-            register.resource_logic.clone(),
--
src//resource/content_addressed_register.rs-100-    ) -> Self {
src//resource/content_addressed_register.rs-101-        // Create a temporary ResourceRegister with a placeholder ID
src//resource/content_addressed_register.rs:102:        let temp_register = ResourceRegister::new(
src//resource/content_addressed_register.rs-103-            ContentId::nil(),
src//resource/content_addressed_register.rs-104-            resource_logic.clone(),
--
src//resource/content_addressed_register.rs-158-    /// Convert to a ResourceRegister
src//resource/content_addressed_register.rs-159-    pub fn to_resource_register(&self) -> ResourceRegister {
src//resource/content_addressed_register.rs:160:        ResourceRegister::new(
src//resource/content_addressed_register.rs-161-            self.id.clone(),
src//resource/content_addressed_register.rs-162-            self.resource_logic.clone(),
--
src//resource/content_addressed_register.rs-599-    fn test_content_addressing() {
src//resource/content_addressed_register.rs-600-        // Create a register with appropriate constructor
src//resource/content_addressed_register.rs:601:        let register = ContentAddressedRegister::new(
src//resource/content_addressed_register.rs-602-            ResourceLogic::Fungible,
src//resource/content_addressed_register.rs-603-            FungibilityDomain("token".to_string()),
--
src//resource/lifecycle.rs-298-        
src//resource/lifecycle.rs-299-        // Create a test resource
src//resource/lifecycle.rs:300:        let mut resource = ResourceRegister::new(
src//resource/lifecycle.rs-301-            ContentId::nil(),
src//resource/lifecycle.rs-302-            ResourceLogic::Fungible,
--
src//resource/lifecycle.rs-352-        
src//resource/lifecycle.rs-353-        // Create test resources
src//resource/lifecycle.rs:354:        let mut resource1 = ResourceRegister::new(
src//resource/lifecycle.rs-355-            ContentId::nil(),
src//resource/lifecycle.rs-356-            ResourceLogic::Fungible,
--
src//resource/lifecycle.rs-361-        );
src//resource/lifecycle.rs-362-        
src//resource/lifecycle.rs:363:        let mut resource2 = ResourceRegister::new(
src//resource/lifecycle.rs-364-            ContentId::nil(),
src//resource/lifecycle.rs-365-            ResourceLogic::Fungible,
--
src//resource/tel.rs-171-        
src//resource/tel.rs-172-        // Create the register
src//resource/tel.rs:173:        let register = Register::new(
src//resource/tel.rs-174-            register_id,
src//resource/tel.rs-175-            owner,
--
src//resource/resource_register.rs-510-    fn test_resource_register_state_transitions() {
src//resource/resource_register.rs-511-        let id = ContentId::new("test-resource".to_string());
src//resource/resource_register.rs:512:        let mut register = ResourceRegister::new(
src//resource/resource_register.rs-513-            id.clone().into(),
src//resource/resource_register.rs-514-            ResourceLogic::Data,
--
src//resource/resource_register.rs-566-    fn test_resource_state_conversion() {
src//resource/resource_register.rs-567-        let id = ContentId::new("test-resource".to_string());
src//resource/resource_register.rs:568:        let register = ResourceRegister::new(
src//resource/resource_register.rs-569-            id.clone().into(),
src//resource/resource_register.rs-570-            ResourceLogic::Data,
```

### Resource Manager Creation Patterns

Found the following resource_manager.create patterns:

```rust
src//tel/effect/mod.rs-131-            ResourceOperationType::Create { owner, domain, initial_data } => {
src//tel/effect/mod.rs-132-                // Create a new resource
src//tel/effect/mod.rs:133:                let result = self.resource_manager.create_resource(
src//tel/effect/mod.rs-134-                    owner,
src//tel/effect/mod.rs-135-                    domain,
--
src//effect/templates/relationship_validation_tests.rs-300-    
src//effect/templates/relationship_validation_tests.rs-301-    // Add all resources to the manager
src//effect/templates/relationship_validation_tests.rs:302:    resource_manager.add_resource(create_resource(&root_resource_id))?;
src//effect/templates/relationship_validation_tests.rs:303:    resource_manager.add_resource(create_resource(&child1_id))?;
src//effect/templates/relationship_validation_tests.rs:304:    resource_manager.add_resource(create_resource(&child2_id))?;
src//effect/templates/relationship_validation_tests.rs:305:    resource_manager.add_resource(create_resource(&grandchild_id))?;
src//effect/templates/relationship_validation_tests.rs:306:    resource_manager.add_resource(create_resource(&dependency1_id))?;
src//effect/templates/relationship_validation_tests.rs:307:    resource_manager.add_resource(create_resource(&dependency2_id))?;
src//effect/templates/relationship_validation_tests.rs-308-    
src//effect/templates/relationship_validation_tests.rs-309-    // Create shared resource manager reference
--
src//resource/tel.rs-264-        
src//resource/tel.rs-265-        // Create the TEL register
src//resource/tel.rs:266:        let tel_register_id = self.tel_resource_manager.create_register(
src//resource/tel.rs-267-            tel_owner,
src//resource/tel.rs-268-            tel_domain,
--
src//resource/tel.rs-676-    #[test]
src//resource/tel.rs-677-    fn test_import_export() -> Result<()> {
src//resource/tel.rs:678:        let tel_resource_manager = create_test_tel_resource_manager();
src//resource/tel.rs-679-        let register_system = create_test_register_system()?;
src//resource/tel.rs-680-        
--
src//resource/tel.rs-686-        let tel_contents = crate::tel::resource::model::RegisterContents::String("Test content".to_string());
src//resource/tel.rs-687-        
src//resource/tel.rs:688:        let tel_register_id = tel_resource_manager.create_register(
src//resource/tel.rs-689-            tel_owner,
src//resource/tel.rs-690-            tel_domain,
--
src//resource/tel.rs-723-    #[test]
src//resource/tel.rs-724-    fn test_sync() -> Result<()> {
src//resource/tel.rs:725:        let tel_resource_manager = create_test_tel_resource_manager();
src//resource/tel.rs-726-        let register_system = create_test_register_system()?;
src//resource/tel.rs-727-        
--
src//resource/tel.rs-733-        let tel_contents = crate::tel::resource::model::RegisterContents::String("Initial content".to_string());
src//resource/tel.rs-734-        
src//resource/tel.rs:735:        let tel_register_id = tel_resource_manager.create_register(
src//resource/tel.rs-736-            tel_owner.clone(),
src//resource/tel.rs-737-            tel_domain.clone(),
```

### Register System Creation Patterns

Found the following register_system.create patterns:

```rust
src//resource/tel.rs-542-        
src//resource/tel.rs-543-        // Create register in domain
src//resource/tel.rs:544:        let register = self.register_system.create_register_in_domain(
src//resource/tel.rs-545-            domain_id,
src//resource/tel.rs-546-            register.owner,
--
src//resource/tel.rs-578-        
src//resource/tel.rs-579-        // Create register with time info
src//resource/tel.rs:580:        let register = self.register_system.create_register_with_time_info(
src//resource/tel.rs-581-            register.owner,
src//resource/tel.rs-582-            register.domain,
--
src//resource/tel.rs-677-    fn test_import_export() -> Result<()> {
src//resource/tel.rs-678-        let tel_resource_manager = create_test_tel_resource_manager();
src//resource/tel.rs:679:        let register_system = create_test_register_system()?;
src//resource/tel.rs-680-        
src//resource/tel.rs-681-        let adapter = TelResourceAdapter::new(register_system, tel_resource_manager.clone());
--
src//resource/tel.rs-705-        let contents = RegisterContents::with_string("New register");
src//resource/tel.rs-706-        
src//resource/tel.rs:707:        let register = adapter.register_system().create_register(
src//resource/tel.rs-708-            owner,
src//resource/tel.rs-709-            domain,
--
src//resource/tel.rs-724-    fn test_sync() -> Result<()> {
src//resource/tel.rs-725-        let tel_resource_manager = create_test_tel_resource_manager();
src//resource/tel.rs:726:        let register_system = create_test_register_system()?;
src//resource/tel.rs-727-        
src//resource/tel.rs-728-        let adapter = TelResourceAdapter::new(register_system, tel_resource_manager.clone());
```

No paired Resource/Register creation patterns found

## Recommended Migration Pattern

Replace these patterns with the unified ResourceRegister approach:
```rust
// OLD PATTERN:
let resource = Resource::new(logic, domain, quantity);
resource_manager.create_resource(resource)?;

// Create a register to hold it
let register = Register::new();
register_system.create_register(register.id, resource.id)?;

// NEW PATTERN:
let token = ResourceRegister::new(logic, domain, quantity);

// Store it using a storage effect
effect_system.execute_effect(StorageEffect::StoreOnChain {
    register_id: token.id,
    fields: token.all_fields(),
    domain_id: domain_id,
    continuation: Box::new(|result| {
        // Handle storage result
    }),
}).await?;
```

