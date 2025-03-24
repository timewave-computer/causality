# Redundant Code Analysis Report

Generated on Sun Mar 23 19:02:33 CST 2025

## Summary

### Resource/Register Conversion Functions

Found the following resource/register conversion functions that can be removed:

```rust
src//operation/transformation.rs-65-    let transformed_operation = match (from_env, to_env) {
src//operation/transformation.rs-66-        (ExecutionEnvironment::Abstract, ExecutionEnvironment::Register) => {
src//operation/transformation.rs:67:            transform_abstract_to_register(operation, target_context)?
src//operation/transformation.rs-68-        },
src//operation/transformation.rs-69-        (ExecutionEnvironment::Register, ExecutionEnvironment::OnChain(_)) => {
--
src//operation/transformation.rs-120-
src//operation/transformation.rs-121-/// Transform an abstract operation to a register operation
src//operation/transformation.rs:122:pub fn transform_abstract_to_register<C: ExecutionContext>(
src//operation/transformation.rs-123-    operation: &Operation<C>,
src//operation/transformation.rs-124-    target_context: RegisterContext,
--
src//operation/transformation.rs-285-    
src//operation/transformation.rs-286-    #[test]
src//operation/transformation.rs:287:    fn test_transform_abstract_to_register() {
src//operation/transformation.rs-288-        // Create an abstract operation
src//operation/transformation.rs-289-        let abstract_context = AbstractContext::new(ExecutionPhase::Planning);
--
src//operation/transformation.rs-312-        );
src//operation/transformation.rs-313-        
src//operation/transformation.rs:314:        let transformed = transform_abstract_to_register(&operation, register_context)
src//operation/transformation.rs-315-            .expect("Transformation should succeed");
src//operation/transformation.rs-316-        
--
src//operation/tests.rs-142-    
src//operation/tests.rs-143-    #[test]
src//operation/tests.rs:144:    fn test_abstract_to_register_transformation() {
src//operation/tests.rs-145-        let abstract_op = create_test_abstract_operation();
src//operation/tests.rs-146-        
--
src//operation/tests.rs-152-        
src//operation/tests.rs-153-        // Use the transformation module to transform the operation
src//operation/tests.rs:154:        let register_op = transform_abstract_to_register(
src//operation/tests.rs-155-            &abstract_op,
src//operation/tests.rs-156-            register_context
--
src//operation/test_fixtures.rs-141-    
src//operation/test_fixtures.rs-142-    #[test]
src//operation/test_fixtures.rs:143:    fn test_abstract_to_register_transformation() {
src//operation/test_fixtures.rs-144-        let abstract_op = create_test_abstract_operation();
src//operation/test_fixtures.rs-145-        
--
src//operation/test_fixtures.rs-151-        
src//operation/test_fixtures.rs-152-        // Use the transformation module to transform the operation
src//operation/test_fixtures.rs:153:        let register_op = transformation::transform_abstract_to_register(
src//operation/test_fixtures.rs-154-            &abstract_op,
src//operation/test_fixtures.rs-155-            register_context
--
src//domain_adapters/evm/storage_strategy.rs-103-    /// Store a register on-chain
src//domain_adapters/evm/storage_strategy.rs-104-    pub async fn store_register(&self, id: &ContentId, data: Vec<u8>, visibility: u8) -> Result<H256> {
src//domain_adapters/evm/storage_strategy.rs:105:        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-106-        
src//domain_adapters/evm/storage_strategy.rs-107-        let call = self.contract.method::<_, bool>(
--
src//domain_adapters/evm/storage_strategy.rs-122-    /// Get a register from on-chain
src//domain_adapters/evm/storage_strategy.rs-123-    pub async fn get_register(&self, id: &ContentId) -> Result<Vec<u8>> {
src//domain_adapters/evm/storage_strategy.rs:124:        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-125-        
src//domain_adapters/evm/storage_strategy.rs-126-        let result: Vec<u8> = self.contract.method::<_, Vec<u8>>(
--
src//domain_adapters/evm/storage_strategy.rs-137-    /// Store a commitment on-chain
src//domain_adapters/evm/storage_strategy.rs-138-    pub async fn store_commitment(&self, id: &ContentId, commitment: &[u8; 32]) -> Result<H256> {
src//domain_adapters/evm/storage_strategy.rs:139:        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-140-        
src//domain_adapters/evm/storage_strategy.rs-141-        let call = self.contract.method::<_, bool>(
--
src//domain_adapters/evm/storage_strategy.rs-156-    /// Store a nullifier on-chain
src//domain_adapters/evm/storage_strategy.rs-157-    pub async fn store_nullifier(&self, id: &ContentId, nullifier: &[u8; 32]) -> Result<H256> {
src//domain_adapters/evm/storage_strategy.rs:158:        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-159-        
src//domain_adapters/evm/storage_strategy.rs-160-        let call = self.contract.method::<_, bool>(
--
src//domain_adapters/evm/storage_strategy.rs-175-
src//domain_adapters/evm/storage_strategy.rs-176-/// Convert a resource ID to bytes32 for Ethereum contracts
src//domain_adapters/evm/storage_strategy.rs:177:fn to_register_id_bytes(id: &ContentId) -> Result<[u8; 32]> {
src//domain_adapters/evm/storage_strategy.rs-178-    let mut bytes = [0u8; 32];
src//domain_adapters/evm/storage_strategy.rs-179-    let id_bytes = id.as_bytes();
--
src//resource/tests/archival_integration_test.rs-76-    
src//resource/tests/archival_integration_test.rs-77-    // Verify the archive reference matches
src//resource/tests/archival_integration_test.rs:78:    let ref_from_register = register.archive_reference.unwrap();
src//resource/tests/archival_integration_test.rs:79:    assert_eq!(ref_from_register.epoch, archive_ref.epoch);
src//resource/tests/archival_integration_test.rs:80:    assert_eq!(ref_from_register.archive_hash, archive_ref.archive_hash);
src//resource/tests/archival_integration_test.rs-81-    
src//resource/tests/archival_integration_test.rs-82-    // Verify we can retrieve from archive
--
src//resource/tests/archival_test.rs-73-    // It should have an archive reference
src//resource/tests/archival_test.rs-74-    assert!(retrieved.archive_reference.is_some());
src//resource/tests/archival_test.rs:75:    let ref_from_register = retrieved.archive_reference.unwrap();
src//resource/tests/archival_test.rs:76:    assert_eq!(ref_from_register.epoch, archive_ref.epoch);
src//resource/tests/archival_test.rs:77:    assert_eq!(ref_from_register.archive_hash, archive_ref.archive_hash);
src//resource/tests/archival_test.rs-78-    
src//resource/tests/archival_test.rs-79-    Ok(())
--
src//resource/tel.rs-28-pub struct TelResourceMapping {
src//resource/tel.rs-29-    /// Mapping from TEL resource IDs to register IDs
src//resource/tel.rs:30:    tel_to_register: RwLock<HashMap<ContentId, ContentId>>,
src//resource/tel.rs-31-    
src//resource/tel.rs-32-    /// Mapping from register IDs to TEL resource IDs
--
src//resource/tel.rs-38-    pub fn new() -> Self {
src//resource/tel.rs-39-        Self {
src//resource/tel.rs:40:            tel_to_register: RwLock::new(HashMap::new()),
src//resource/tel.rs-41-            register_to_tel: RwLock::new(HashMap::new()),
src//resource/tel.rs-42-        }
--
src//resource/tel.rs-45-    /// Map a TEL resource ID to a register ID
src//resource/tel.rs-46-    pub fn map_resource(&self, tel_id: ContentId, register_id: ContentId) -> Result<()> {
src//resource/tel.rs:47:        let mut tel_to_register = self.tel_to_register.write()
src//resource/tel.rs:48:            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
src//resource/tel.rs-49-        
src//resource/tel.rs-50-        let mut register_to_tel = self.register_to_tel.write()
src//resource/tel.rs-51-            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
src//resource/tel.rs-52-        
src//resource/tel.rs:53:        tel_to_register.insert(tel_id.clone(), register_id.clone());
src//resource/tel.rs-54-        register_to_tel.insert(register_id, tel_id);
src//resource/tel.rs-55-        
--
src//resource/tel.rs-59-    /// Get the register ID for a TEL resource ID
src//resource/tel.rs-60-    pub fn get_register_id(&self, tel_id: &ContentId) -> Result<Option<ContentId>> {
src//resource/tel.rs:61:        let tel_to_register = self.tel_to_register.read()
src//resource/tel.rs:62:            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
src//resource/tel.rs-63-        
src//resource/tel.rs:64:        Ok(tel_to_register.get(tel_id).cloned())
src//resource/tel.rs-65-    }
src//resource/tel.rs-66-    
--
src//resource/tel.rs-75-    /// Remove a mapping
src//resource/tel.rs-76-    pub fn remove_mapping(&self, tel_id: &ContentId, register_id: &ContentId) -> Result<()> {
src//resource/tel.rs:77:        let mut tel_to_register = self.tel_to_register.write()
src//resource/tel.rs:78:            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
src//resource/tel.rs-79-        
src//resource/tel.rs-80-        let mut register_to_tel = self.register_to_tel.write()
src//resource/tel.rs-81-            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
src//resource/tel.rs-82-        
src//resource/tel.rs:83:        tel_to_register.remove(tel_id);
src//resource/tel.rs-84-        register_to_tel.remove(register_id);
src//resource/tel.rs-85-        
--
src//resource/tel.rs-89-    /// Get all TEL resource IDs
src//resource/tel.rs-90-    pub fn get_all_tel_ids(&self) -> Result<HashSet<ContentId>> {
src//resource/tel.rs:91:        let tel_to_register = self.tel_to_register.read()
src//resource/tel.rs:92:            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
src//resource/tel.rs-93-        
src//resource/tel.rs:94:        Ok(tel_to_register.keys().cloned().collect())
src//resource/tel.rs-95-    }
src//resource/tel.rs-96-    
--
src//resource/tel.rs-130-    
src//resource/tel.rs-131-    /// Convert a TEL register to our register format
src//resource/tel.rs:132:    pub fn convert_tel_register_to_register(&self, tel_register: &TelRegister) -> Result<Register> {
src//resource/tel.rs-133-        // Convert owner and domain
src//resource/tel.rs-134-        let owner = Address::new(&tel_register.owner.to_string());
--
src//resource/tel.rs-232-        
src//resource/tel.rs-233-        // Convert to our register format
src//resource/tel.rs:234:        let register = self.convert_tel_register_to_register(&tel_register)?;
src//resource/tel.rs-235-        
src//resource/tel.rs-236-        // Import into register system
--
src//resource/tel.rs-299-        if tel_register.updated_at > current_register.updated_at {
src//resource/tel.rs-300-            // Convert to our register format
src//resource/tel.rs:301:            let new_register = self.convert_tel_register_to_register(&tel_register)?;
src//resource/tel.rs-302-            
src//resource/tel.rs-303-            // Update register in system
--
src//resource/tel.rs-422-
src//resource/tel.rs-423-    /// Convert TEL operation to Register operation
src//resource/tel.rs:424:    pub fn convert_tel_to_register_operation(
src//resource/tel.rs-425-        &self,
src//resource/tel.rs-426-        tel_operation: &ResourceOperation
--
src//resource/tel.rs-468-    ) -> Result<()> {
src//resource/tel.rs-469-        // Convert TEL operation to register operation
src//resource/tel.rs:470:        let register_op = self.convert_tel_to_register_operation(operation)?;
src//resource/tel.rs-471-        
src//resource/tel.rs-472-        // Get the register
--
src//resource/tel.rs-539-        
src//resource/tel.rs-540-        // Convert to our register format
src//resource/tel.rs:541:        let register = self.convert_tel_register_to_register(&tel_register)?;
src//resource/tel.rs-542-        
src//resource/tel.rs-543-        // Create register in domain
--
src//resource/tel.rs-575-        
src//resource/tel.rs-576-        // Convert to our register format
src//resource/tel.rs:577:        let register = self.convert_tel_register_to_register(&tel_register)?;
src//resource/tel.rs-578-        
src//resource/tel.rs-579-        // Create register with time info
--
src//resource/archival.rs-256-impl RegisterArchive {
src//resource/archival.rs-257-    /// Create a new register archive from a register
src//resource/archival.rs:258:    pub fn from_register(
src//resource/archival.rs-259-        register: &Register,
src//resource/archival.rs-260-        epoch: EpochId,
--
src//resource/archival.rs-411-    
src//resource/archival.rs-412-    /// Restore register from archive
src//resource/archival.rs:413:    pub fn to_register(&mut self) -> Result<Register> {
src//resource/archival.rs-414-        // Decompress if needed
src//resource/archival.rs-415-        if self.compression != CompressionFormat::None {
--
src//resource/archival.rs-486-    ) -> Result<ArchiveReference> {
src//resource/archival.rs-487-        // Create archive
src//resource/archival.rs:488:        let mut archive = RegisterArchive::from_register(
src//resource/archival.rs-489-            register,
src//resource/archival.rs-490-            epoch,
--
src//resource/archival.rs-544-                
src//resource/archival.rs-545-                // Restore register
src//resource/archival.rs:546:                let register = archive.to_register()?;
src//resource/archival.rs-547-                
src//resource/archival.rs-548-                return Ok(Some(register));
--
src//resource/archival.rs-723-        
src//resource/archival.rs-724-        // Create archive
src//resource/archival.rs:725:        let archive = RegisterArchive::from_register(
src//resource/archival.rs-726-            &register,
src//resource/archival.rs-727-            1, // epoch
--
src//resource/archival.rs-753-        
src//resource/archival.rs-754-        // Create uncompressed archive for comparison
src//resource/archival.rs:755:        let uncompressed = RegisterArchive::from_register(
src//resource/archival.rs-756-            &register,
src//resource/archival.rs-757-            1,
--
src//resource/archival.rs-761-        
src//resource/archival.rs-762-        // Create gzip compressed archive
src//resource/archival.rs:763:        let gzip = RegisterArchive::from_register(
src//resource/archival.rs-764-            &register,
src//resource/archival.rs-765-            1,
--
src//resource/archival.rs-769-        
src//resource/archival.rs-770-        // Create zstd compressed archive
src//resource/archival.rs:771:        let zstd = RegisterArchive::from_register(
src//resource/archival.rs-772-            &register,
src//resource/archival.rs-773-            1,
--
src//resource/archival.rs-797-        
src//resource/archival.rs-798-        // Create archive
src//resource/archival.rs:799:        let archive = RegisterArchive::from_register(
src//resource/archival.rs-800-            &register,
src//resource/archival.rs-801-            1,
--
src//resource/archival.rs-835-        
src//resource/archival.rs-836-        // Create archive
src//resource/archival.rs:837:        let mut archive = RegisterArchive::from_register(
src//resource/archival.rs-838-            &original,
src//resource/archival.rs-839-            1,
--
src//resource/archival.rs-843-        
src//resource/archival.rs-844-        // Convert back to register
src//resource/archival.rs:845:        let restored = archive.to_register()?;
src//resource/archival.rs-846-        
src//resource/archival.rs-847-        // Check fields
```

### Resource/Register Synchronization Code

Found the following synchronization code that can be removed:

```rust
src//operation/execution.rs-153-    
src//operation/execution.rs-154-    /// Execute a register operation
src//operation/execution.rs:155:    async fn execute_register_operation(
src//operation/execution.rs-156-        &self,
src//operation/execution.rs-157-        register_op: &RegisterOperation
--
src//operation/api.rs-103-    
src//operation/api.rs-104-    /// Execute an operation in the register context
src//operation/api.rs:105:    pub async fn execute_register(
src//operation/api.rs-106-        &self,
src//operation/api.rs-107-        operation: &Operation<RegisterContext>,
--
src//operation/api.rs-123-    
src//operation/api.rs-124-    /// Transform and execute an operation from abstract to register context
src//operation/api.rs:125:    pub async fn execute_as_register(
src//operation/api.rs-126-        &self,
src//operation/api.rs-127-        operation: &Operation<AbstractContext>,
--
src//operation/api.rs-156-    
src//operation/api.rs-157-    /// Create and execute a transfer operation
src//operation/api.rs:158:    pub async fn transfer_resource(
src//operation/api.rs-159-        &self,
src//operation/api.rs-160-        resource_id: ContentId,
--
src//operation/api.rs-197-    
src//operation/api.rs-198-    /// Create and execute a deposit operation
src//operation/api.rs:199:    pub async fn deposit_resource(
src//operation/api.rs-200-        &self,
src//operation/api.rs-201-        resource_id: ContentId,
--
src//operation/api.rs-230-    
src//operation/api.rs-231-    /// Create and execute a withdrawal operation
src//operation/api.rs:232:    pub async fn withdraw_resource(
src//operation/api.rs-233-        &self,
src//operation/api.rs-234-        resource_id: ContentId,
--
src//committee/indexer.rs-226-    
src//committee/indexer.rs-227-    /// Register an indexer for a chain
src//committee/indexer.rs:228:    pub async fn register_indexer(&self, config: IndexerConfig) -> Result<()> {
src//committee/indexer.rs-229-        let chain_id = config.chain_id.clone();
src//committee/indexer.rs-230-        let indexer = self.factory.create_indexer(config)?;
--
src//domain_adapters/evm/adapter.rs-259-    
src//domain_adapters/evm/adapter.rs-260-    /// Handle register creation fact query
src//domain_adapters/evm/adapter.rs:261:    async fn handle_register_create_query(&self, query: &FactQuery) -> Result<FactType> {
src//domain_adapters/evm/adapter.rs-262-        // Extract register ID from parameters
src//domain_adapters/evm/adapter.rs-263-        let register_id = query.parameters.get("register_id")
--
src//domain_adapters/evm/adapter.rs-310-    
src//domain_adapters/evm/adapter.rs-311-    /// Handle register update fact query
src//domain_adapters/evm/adapter.rs:312:    async fn handle_register_update_query(&self, query: &FactQuery) -> Result<FactType> {
src//domain_adapters/evm/adapter.rs-313-        // Extract register ID from parameters
src//domain_adapters/evm/adapter.rs-314-        let register_id = query.parameters.get("register_id")
--
src//domain_adapters/evm/adapter.rs-361-    
src//domain_adapters/evm/adapter.rs-362-    /// Handle register transfer fact query
src//domain_adapters/evm/adapter.rs:363:    async fn handle_register_transfer_query(&self, query: &FactQuery) -> Result<FactType> {
src//domain_adapters/evm/adapter.rs-364-        // Extract register ID from parameters
src//domain_adapters/evm/adapter.rs-365-        let register_id = query.parameters.get("register_id")
--
src//domain_adapters/evm/adapter.rs-407-    
src//domain_adapters/evm/adapter.rs-408-    /// Generate proof data for register facts
src//domain_adapters/evm/adapter.rs:409:    async fn generate_register_proof_data(&self, register_id: &str, address: &str, block_hash: H256) -> Result<Option<Vec<u8>>> {
src//domain_adapters/evm/adapter.rs-410-        // For EVM chains that support storage proofs
src//domain_adapters/evm/adapter.rs-411-        // we would implement Merkle proof generation here
--
src//domain_adapters/evm/adapter.rs-847-    // Unit tests that don't require an actual RPC endpoint
src//domain_adapters/evm/adapter.rs-848-    #[tokio::test]
src//domain_adapters/evm/adapter.rs:849:    async fn test_register_facts() {
src//domain_adapters/evm/adapter.rs-850-        // Setup mock adapter
src//domain_adapters/evm/adapter.rs-851-        let config = EthereumConfig {
--
src//domain_adapters/evm/storage_strategy.rs-102-    
src//domain_adapters/evm/storage_strategy.rs-103-    /// Store a register on-chain
src//domain_adapters/evm/storage_strategy.rs:104:    pub async fn store_register(&self, id: &ContentId, data: Vec<u8>, visibility: u8) -> Result<H256> {
src//domain_adapters/evm/storage_strategy.rs-105-        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-106-        
--
src//domain_adapters/evm/storage_strategy.rs-121-    
src//domain_adapters/evm/storage_strategy.rs-122-    /// Get a register from on-chain
src//domain_adapters/evm/storage_strategy.rs:123:    pub async fn get_register(&self, id: &ContentId) -> Result<Vec<u8>> {
src//domain_adapters/evm/storage_strategy.rs-124-        let id_bytes = to_register_id_bytes(id)?;
src//domain_adapters/evm/storage_strategy.rs-125-        
--
src//effect/templates/relationship_validation_tests.rs-203-
src//effect/templates/relationship_validation_tests.rs-204-#[tokio::test]
src//effect/templates/relationship_validation_tests.rs:205:async fn test_complex_resource_relationships() -> Result<()> {
src//effect/templates/relationship_validation_tests.rs-206-    // Create a more complex scenario with multiple relationship types
src//effect/templates/relationship_validation_tests.rs-207-    
--
src//concurrency/patterns/barrier.rs-163-/// This is a convenience function that creates a barrier that waits
src//concurrency/patterns/barrier.rs-164-/// for a set of resources to be available.
src//concurrency/patterns/barrier.rs:165:pub async fn wait_for_resources(
src//concurrency/patterns/barrier.rs-166-    resources: Vec<ContentId>,
src//concurrency/patterns/barrier.rs-167-    resource_manager: SharedResourceManager,
--
src//concurrency/patterns/barrier.rs-230-    
src//concurrency/patterns/barrier.rs-231-    #[tokio::test]
src//concurrency/patterns/barrier.rs:232:    async fn test_resource_barrier() -> Result<()> {
src//concurrency/patterns/barrier.rs-233-        // Create a resource manager
src//concurrency/patterns/barrier.rs-234-        let resource_manager = Arc::new(ResourceManager::new());
--
src//concurrency/patterns/barrier.rs-267-    
src//concurrency/patterns/barrier.rs-268-    #[tokio::test]
src//concurrency/patterns/barrier.rs:269:    async fn test_wait_for_resources() -> Result<()> {
src//concurrency/patterns/barrier.rs-270-        // Create a resource manager
src//concurrency/patterns/barrier.rs-271-        let resource_manager = Arc::new(ResourceManager::new());
--
src//concurrency/primitives/resource_manager.rs-354-    
src//concurrency/primitives/resource_manager.rs-355-    #[tokio::test]
src//concurrency/primitives/resource_manager.rs:356:    async fn test_resource_manager_basic() -> Result<()> {
src//concurrency/primitives/resource_manager.rs-357-        let manager = ResourceManager::new();
src//concurrency/primitives/resource_manager.rs-358-        
--
src//concurrency/primitives/resource_manager.rs-385-    
src//concurrency/primitives/resource_manager.rs-386-    #[tokio::test]
src//concurrency/primitives/resource_manager.rs:387:    async fn test_resource_manager_contention() -> Result<()> {
src//concurrency/primitives/resource_manager.rs-388-        let manager = Arc::new(ResourceManager::new());
src//concurrency/primitives/resource_manager.rs-389-        
--
src//concurrency/primitives/resource_manager.rs-420-    
src//concurrency/primitives/resource_manager.rs-421-    #[tokio::test]
src//concurrency/primitives/resource_manager.rs:422:    async fn test_resource_manager_update_value() -> Result<()> {
src//concurrency/primitives/resource_manager.rs-423-        let manager = ResourceManager::new();
src//concurrency/primitives/resource_manager.rs-424-        
--
src//verification/examples.rs-395-    
src//verification/examples.rs-396-    #[tokio::test]
src//verification/examples.rs:397:    async fn test_resource_operation_verification() {
src//verification/examples.rs-398-        // Create a resource operation
src//verification/examples.rs-399-        let resource_id = ContentId::from("test-resource-123");
--
src//relationship/cross_domain_query.rs-68-
src//relationship/cross_domain_query.rs-69-    /// Invalidates cache entries related to a resource
src//relationship/cross_domain_query.rs:70:    pub async fn invalidate_for_resource(&self, resource_id: &ContentId) {
src//relationship/cross_domain_query.rs-71-        let mut cache = self.paths.write().await;
src//relationship/cross_domain_query.rs-72-        cache.retain(|k, _| k.source_id != *resource_id && k.target_id != *resource_id);
--
src//relationship/cross_domain_query.rs-197-    
src//relationship/cross_domain_query.rs-198-    /// Checks if a resource exists in this domain
src//relationship/cross_domain_query.rs:199:    async fn resource_exists(&self, resource_id: &ContentId) -> Result<bool>;
src//relationship/cross_domain_query.rs-200-}
src//relationship/cross_domain_query.rs-201-
--
src//relationship/cross_domain_query.rs-223-
src//relationship/cross_domain_query.rs-224-    /// Registers a domain relationship provider
src//relationship/cross_domain_query.rs:225:    pub async fn register_domain_provider(&self, provider: Arc<dyn DomainRelationshipProvider>) {
src//relationship/cross_domain_query.rs-226-        let mut providers = self.domain_providers.write().await;
src//relationship/cross_domain_query.rs-227-        providers.insert(provider.domain_id().clone(), provider);
--
src//relationship/cross_domain_query.rs-564-    
src//relationship/cross_domain_query.rs-565-    /// Invalidates cached paths involving a resource
src//relationship/cross_domain_query.rs:566:    pub async fn invalidate_cache_for_resource(&self, resource_id: &ContentId) {
src//relationship/cross_domain_query.rs-567-        self.cache.invalidate_for_resource(resource_id).await;
src//relationship/cross_domain_query.rs-568-    }
--
src//examples/boundary_aware_resources.rs-18-
src//examples/boundary_aware_resources.rs-19-/// Example function demonstrating boundary-aware resource creation and crossing
src//examples/boundary_aware_resources.rs:20:pub async fn boundary_aware_resource_example() -> Result<()> {
src//examples/boundary_aware_resources.rs-21-    // Set up the resource management components
src//examples/boundary_aware_resources.rs-22-    let lifecycle_manager = Arc::new(ResourceRegisterLifecycleManager::new(HashMap::new()));
--
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
src//resource/tests/api_tests.rs-7-
src//resource/tests/api_tests.rs-8-#[tokio::test]
src//resource/tests/api_tests.rs:9:async fn test_create_resource() {
src//resource/tests/api_tests.rs-10-    // Create addresses
src//resource/tests/api_tests.rs-11-    let admin = Address::from("admin:0x1234");
--
src//resource/tests/api_tests.rs-50-
src//resource/tests/api_tests.rs-51-#[tokio::test]
src//resource/tests/api_tests.rs:52:async fn test_update_resource() {
src//resource/tests/api_tests.rs-53-    // Create addresses
src//resource/tests/api_tests.rs-54-    let admin = Address::from("admin:0x1234");
--
src//resource/tests/api_tests.rs-106-
src//resource/tests/api_tests.rs-107-#[tokio::test]
src//resource/tests/api_tests.rs:108:async fn test_delete_resource() {
src//resource/tests/api_tests.rs-109-    // Create addresses
src//resource/tests/api_tests.rs-110-    let admin = Address::from("admin:0x1234");
--
src//resource/tests/api_tests.rs-237-
src//resource/tests/api_tests.rs-238-#[tokio::test]
src//resource/tests/api_tests.rs:239:async fn test_resource_query() {
src//resource/tests/api_tests.rs-240-    // Create addresses
src//resource/tests/api_tests.rs-241-    let admin = Address::from("admin:0x1234");
--
src//resource/tests/effect_tests.rs-155-
src//resource/tests/effect_tests.rs-156-#[tokio::test]
src//resource/tests/effect_tests.rs:157:async fn test_resource_effect_execution() {
src//resource/tests/effect_tests.rs-158-    // Setup resource API
src//resource/tests/effect_tests.rs-159-    let resource_api = Arc::new(MemoryResourceAPI::new());
--
src//resource/tests/effect_template_integration_tests.rs-142-
src//resource/tests/effect_template_integration_tests.rs-143-#[tokio::test]
src//resource/tests/effect_template_integration_tests.rs:144:async fn test_create_resource_effect_integration() -> Result<()> {
src//resource/tests/effect_template_integration_tests.rs-145-    // Create a resource and domain
src//resource/tests/effect_template_integration_tests.rs-146-    let resource = create_test_resource("resource1");
--
src//resource/tests/effect_template_integration_tests.rs-320-
src//resource/tests/effect_template_integration_tests.rs-321-#[tokio::test]
src//resource/tests/effect_template_integration_tests.rs:322:async fn test_boundary_aware_resource_creation() -> Result<()> {
src//resource/tests/effect_template_integration_tests.rs-323-    // Create a resource and domain
src//resource/tests/effect_template_integration_tests.rs-324-    let resource = create_test_resource("resource4");
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
src//resource/resource_temporal_consistency.rs-622-            }
src//resource/resource_temporal_consistency.rs-623-            TimeEvent::SyncRequest => {
src//resource/resource_temporal_consistency.rs:624:                // When requested to sync, trigger relationship and resource sync
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
src//resource/relationship/cross_domain.rs-56-    Periodic(Duration),
src//resource/relationship/cross_domain.rs-57-    
src//resource/relationship/cross_domain.rs:58:    /// Event-driven synchronization (when resources change)
src//resource/relationship/cross_domain.rs-59-    EventDriven,
src//resource/relationship/cross_domain.rs-60-    
--
src//resource/relationship/sync.rs-67-#[derive(Debug, Clone)]
src//resource/relationship/sync.rs-68-pub struct SyncOptions {
src//resource/relationship/sync.rs:69:    /// Whether to force synchronization even if resources are up-to-date
src//resource/relationship/sync.rs-70-    pub force: bool,
src//resource/relationship/sync.rs-71-    
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
src//resource/relationship/sync.rs-146-    ValidationFailed(String),
src//resource/relationship/sync.rs-147-    
src//resource/relationship/sync.rs:148:    /// No sync handler registered for this domain pair
src//resource/relationship/sync.rs-149-    NoSyncHandler(String, String),
src//resource/relationship/sync.rs-150-    
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
src//resource/capability_system.rs-182-    
src//resource/capability_system.rs-183-    /// Get all capabilities for a resource
src//resource/capability_system.rs:184:    async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>>;
src//resource/capability_system.rs-185-    
src//resource/capability_system.rs-186-    /// Get all capabilities owned by an address
--
src//resource/capability_system.rs-445-    }
src//resource/capability_system.rs-446-    
src//resource/capability_system.rs:447:    async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>> {
src//resource/capability_system.rs-448-        let capabilities = self.capabilities.read().unwrap();
src//resource/capability_system.rs-449-        let mut result = Vec::new();
--
src//resource/tel.rs-313-    
src//resource/tel.rs-314-    /// Sync a TEL register with its register counterpart (from register to TEL)
src//resource/tel.rs:315:    pub fn sync_to_tel(&self, register_id: &ContentId) -> Result<()> {
src//resource/tel.rs-316-        // Get the TEL resource ID
src//resource/tel.rs-317-        let tel_id = self.mapping.get_tel_id(register_id)?
--
src//resource/tel.rs-394-            ResourceOperationType::Update => {
src//resource/tel.rs-395-                // Sync from TEL to get the latest changes
src//resource/tel.rs:396:                self.sync_from_tel(&tel_resource_id)
src//resource/tel.rs-397-            },
src//resource/tel.rs-398-            ResourceOperationType::Delete => {
--
src//resource/tel.rs-406-            ResourceOperationType::Transfer => {
src//resource/tel.rs-407-                // Sync from TEL to get the owner change
src//resource/tel.rs:408:                self.sync_from_tel(&tel_resource_id)
src//resource/tel.rs-409-            },
src//resource/tel.rs-410-            ResourceOperationType::Lock => {
--
src//resource/tel.rs-610-        // Sync the register with its domain if it has one
src//resource/tel.rs-611-        if let Ok(Some(_)) = self.register_system.get_domain_for_register(&register_id) {
src//resource/tel.rs:612:            self.register_system.sync_register_with_domain(&register_id)
src//resource/tel.rs:613:                .map_err(|e| Error::DomainError(format!("Failed to sync register with domain: {}", e)))?;
src//resource/tel.rs-614-        }
src//resource/tel.rs-615-        
--
src//resource/tel.rs-752-        
src//resource/tel.rs-753-        // Sync from TEL to register
src//resource/tel.rs:754:        adapter.sync_from_tel(&tel_resource_id)?;
src//resource/tel.rs-755-        
src//resource/tel.rs-756-        // Verify the register was updated
--
src//resource/tel.rs-771-        
src//resource/tel.rs-772-        // Sync from register to TEL
src//resource/tel.rs:773:        adapter.sync_to_tel(&register_id)?;
src//resource/tel.rs-774-        
src//resource/tel.rs-775-        // Verify the TEL register was updated
--
src//resource/storage_adapter.rs-54-    
src//resource/storage_adapter.rs-55-    /// Compatibility method for storing a resource on-chain
src//resource/storage_adapter.rs:56:    pub async fn store_resource(&self, resource_id: &ContentId, domain_id: &DomainId) -> Result<String> {
src//resource/storage_adapter.rs-57-        // Get the resource from the lifecycle manager
src//resource/storage_adapter.rs-58-        let resource_state = self.lifecycle_manager.get_state(resource_id)?;
--
src//resource/storage_adapter.rs-234-    
src//resource/storage_adapter.rs-235-    /// Compatibility method for reading a resource from storage
src//resource/storage_adapter.rs:236:    pub async fn read_resource(&self, 
src//resource/storage_adapter.rs-237-        resource_id: &ContentId, 
src//resource/storage_adapter.rs-238-        domain_id: &DomainId
--
src//resource/memory_api.rs-126-    
src//resource/memory_api.rs-127-    /// Check if a capability can access a resource
src//resource/memory_api.rs:128:    async fn check_resource_access(
src//resource/memory_api.rs-129-        &self,
src//resource/memory_api.rs-130-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-160-#[async_trait]
src//resource/memory_api.rs-161-impl ResourceAPI for MemoryResourceAPI {
src//resource/memory_api.rs:162:    async fn create_resource(
src//resource/memory_api.rs-163-        &self,
src//resource/memory_api.rs-164-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-232-    }
src//resource/memory_api.rs-233-    
src//resource/memory_api.rs:234:    async fn get_resource(
src//resource/memory_api.rs-235-        &self,
src//resource/memory_api.rs-236-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-250-    }
src//resource/memory_api.rs-251-    
src//resource/memory_api.rs:252:    async fn get_resource_mut(
src//resource/memory_api.rs-253-        &self,
src//resource/memory_api.rs-254-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-270-    }
src//resource/memory_api.rs-271-    
src//resource/memory_api.rs:272:    async fn find_resources(
src//resource/memory_api.rs-273-        &self,
src//resource/memory_api.rs-274-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-381-    }
src//resource/memory_api.rs-382-    
src//resource/memory_api.rs:383:    async fn update_resource(
src//resource/memory_api.rs-384-        &self,
src//resource/memory_api.rs-385-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-435-    }
src//resource/memory_api.rs-436-    
src//resource/memory_api.rs:437:    async fn delete_resource(
src//resource/memory_api.rs-438-        &self,
src//resource/memory_api.rs-439-        capability: &CapabilityRef,
--
src//resource/memory_api.rs-455-    }
src//resource/memory_api.rs-456-    
src//resource/memory_api.rs:457:    async fn resource_exists(
src//resource/memory_api.rs-458-        &self,
src//resource/memory_api.rs-459-        capability: &CapabilityRef,
--
src//resource/api.rs-295-pub trait ResourceAPI: Send + Sync {
src//resource/api.rs-296-    /// Create a new resource
src//resource/api.rs:297:    async fn create_resource(
src//resource/api.rs-298-        &self,
src//resource/api.rs-299-        capability: &CapabilityRef,
--
src//resource/api.rs-305-    
src//resource/api.rs-306-    /// Create a resource with structured data
src//resource/api.rs:307:    async fn create_structured_resource<T: serde::Serialize + Send + Sync>(
src//resource/api.rs-308-        &self,
src//resource/api.rs-309-        capability: &CapabilityRef,
--
src//resource/api.rs-319-    
src//resource/api.rs-320-    /// Get a resource by ID
src//resource/api.rs:321:    async fn get_resource(
src//resource/api.rs-322-        &self,
src//resource/api.rs-323-        capability: &CapabilityRef,
--
src//resource/api.rs-326-    
src//resource/api.rs-327-    /// Get a mutable resource by ID
src//resource/api.rs:328:    async fn get_resource_mut(
src//resource/api.rs-329-        &self,
src//resource/api.rs-330-        capability: &CapabilityRef,
--
src//resource/api.rs-333-    
src//resource/api.rs-334-    /// Find resources based on a query
src//resource/api.rs:335:    async fn find_resources(
src//resource/api.rs-336-        &self,
src//resource/api.rs-337-        capability: &CapabilityRef,
--
src//resource/api.rs-340-    
src//resource/api.rs-341-    /// Update a resource
src//resource/api.rs:342:    async fn update_resource(
src//resource/api.rs-343-        &self,
src//resource/api.rs-344-        capability: &CapabilityRef,
--
src//resource/api.rs-349-    
src//resource/api.rs-350-    /// Delete a resource
src//resource/api.rs:351:    async fn delete_resource(
src//resource/api.rs-352-        &self,
src//resource/api.rs-353-        capability: &CapabilityRef,
--
src//resource/api.rs-356-    
src//resource/api.rs-357-    /// Check if a resource exists
src//resource/api.rs:358:    async fn resource_exists(
src//resource/api.rs-359-        &self,
src//resource/api.rs-360-        capability: &CapabilityRef,
--
src//domain/resource_integration.rs-99-    
src//domain/resource_integration.rs-100-    /// Store a resource in the domain
src//domain/resource_integration.rs:101:    async fn store_resource(
src//domain/resource_integration.rs-102-        &self, 
src//domain/resource_integration.rs-103-        resource_id: &ResourceId, 
--
src//domain/resource_integration.rs-107-    
src//domain/resource_integration.rs-108-    /// Retrieve a resource from the domain
src//domain/resource_integration.rs:109:    async fn retrieve_resource(
src//domain/resource_integration.rs-110-        &self, 
src//domain/resource_integration.rs-111-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-113-    
src//domain/resource_integration.rs-114-    /// Verify a resource exists in the domain
src//domain/resource_integration.rs:115:    async fn verify_resource(
src//domain/resource_integration.rs-116-        &self, 
src//domain/resource_integration.rs-117-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-144-    }
src//domain/resource_integration.rs-145-    
src//domain/resource_integration.rs:146:    async fn store_resource(
src//domain/resource_integration.rs-147-        &self, 
src//domain/resource_integration.rs-148-        resource_id: &ResourceId, 
--
src//domain/resource_integration.rs-183-    }
src//domain/resource_integration.rs-184-    
src//domain/resource_integration.rs:185:    async fn retrieve_resource(
src//domain/resource_integration.rs-186-        &self, 
src//domain/resource_integration.rs-187-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-224-    }
src//domain/resource_integration.rs-225-    
src//domain/resource_integration.rs:226:    async fn verify_resource(
src//domain/resource_integration.rs-227-        &self, 
src//domain/resource_integration.rs-228-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-287-    }
src//domain/resource_integration.rs-288-    
src//domain/resource_integration.rs:289:    async fn store_resource(
src//domain/resource_integration.rs-290-        &self, 
src//domain/resource_integration.rs-291-        resource_id: &ResourceId, 
--
src//domain/resource_integration.rs-326-    }
src//domain/resource_integration.rs-327-    
src//domain/resource_integration.rs:328:    async fn retrieve_resource(
src//domain/resource_integration.rs-329-        &self, 
src//domain/resource_integration.rs-330-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-367-    }
src//domain/resource_integration.rs-368-    
src//domain/resource_integration.rs:369:    async fn verify_resource(
src//domain/resource_integration.rs-370-        &self, 
src//domain/resource_integration.rs-371-        resource_id: &ResourceId
--
src//domain/resource_integration.rs-597-    
src//domain/resource_integration.rs-598-    /// Store a resource in the most appropriate domain based on selection strategy
src//domain/resource_integration.rs:599:    pub async fn store_resource_by_strategy(
src//domain/resource_integration.rs-600-        &self,
src//domain/resource_integration.rs-601-        resource_id: ContentId,
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
--
src//domain/capability.rs-518-        }
src//domain/capability.rs-519-        
src//domain/capability.rs:520:        async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>> {
src//domain/capability.rs-521-            Ok(self.capabilities
src//domain/capability.rs-522-                .values()
```

No dual lookup patterns found

### Duplicate Validation Logic

Found the following duplicate validation logic that can be consolidated:

```rust
src//tel/resource/vm.rs-138-    /// Whether to auto-commit changes on context exit
src//tel/resource/vm.rs-139-    pub auto_commit_on_exit: bool,
src//tel/resource/vm.rs:140:    /// Whether to validate resource access against time system
src//tel/resource/vm.rs-141-    pub validate_time_access: bool,
src//tel/resource/vm.rs-142-    /// Memory section for resource data
--
src//tel/resource/model/register.rs-327-    
src//tel/resource/model/register.rs-328-    /// Validate a mutation on the resource
src//tel/resource/model/register.rs:329:    fn validate_mutation(&self, resource: &Resource, new_contents: &RegisterContents) -> TelResult<()>;
src//tel/resource/model/register.rs-330-    
src//tel/resource/model/register.rs-331-    /// Process a transfer of the resource
--
src//effect/templates/relationship_validation.rs-1-// Relationship Validation Effect
src//effect/templates/relationship_validation.rs-2-//
src//effect/templates/relationship_validation.rs:3:// This module provides an effect that validates resource operations against
src//effect/templates/relationship_validation.rs-4-// relationship constraints, ensuring that operations don't violate the relationship
src//effect/templates/relationship_validation.rs-5-// rules between resources.
--
src//relationship/cross_domain_query.rs-67-    }
src//relationship/cross_domain_query.rs-68-
src//relationship/cross_domain_query.rs:69:    /// Invalidates cache entries related to a resource
src//relationship/cross_domain_query.rs:70:    pub async fn invalidate_for_resource(&self, resource_id: &ContentId) {
src//relationship/cross_domain_query.rs-71-        let mut cache = self.paths.write().await;
src//relationship/cross_domain_query.rs-72-        cache.retain(|k, _| k.source_id != *resource_id && k.target_id != *resource_id);
--
src//relationship/cross_domain_query.rs-563-    }
src//relationship/cross_domain_query.rs-564-    
src//relationship/cross_domain_query.rs:565:    /// Invalidates cached paths involving a resource
src//relationship/cross_domain_query.rs:566:    pub async fn invalidate_cache_for_resource(&self, resource_id: &ContentId) {
src//relationship/cross_domain_query.rs:567:        self.cache.invalidate_for_resource(resource_id).await;
src//relationship/cross_domain_query.rs-568-    }
src//relationship/cross_domain_query.rs-569-}
--
src//relationship/cross_domain_query.rs-664-        }
src//relationship/cross_domain_query.rs-665-        
src//relationship/cross_domain_query.rs:666:        // Invalidate any cached queries involving this resource
src//relationship/cross_domain_query.rs:667:        self.query_executor.invalidate_cache_for_resource(resource_id).await;
src//relationship/cross_domain_query.rs-668-        
src//relationship/cross_domain_query.rs-669-        Ok(())
--
src//examples/boundary_aware_resources.rs-160-    }
src//examples/boundary_aware_resources.rs-161-    
src//examples/boundary_aware_resources.rs:162:    fn validate_grant(&self, _grant: &crate::resource::request::ResourceGrant) -> Result<bool> {
src//examples/boundary_aware_resources.rs-163-        Ok(true)
src//examples/boundary_aware_resources.rs-164-    }
--
src//log/fact_dependency_validator.rs-72-    
src//log/fact_dependency_validator.rs-73-    /// Validate register dependencies
src//log/fact_dependency_validator.rs:74:    pub fn validate_register_dependencies(&self, register_dependencies: &[ContentId]) -> bool {
src//log/fact_dependency_validator.rs-75-        for register_id in register_dependencies {
src//log/fact_dependency_validator.rs-76-            if !self.register_observations.contains_key(register_id) {
--
src//resource/lifecycle_manager.rs-363-    ) -> Result<()> {
src//resource/lifecycle_manager.rs-364-        // Validate the operation first
src//resource/lifecycle_manager.rs:365:        if !self.validate_operation(resource_id, operation_type, capability_ids)? {
src//resource/lifecycle_manager.rs-366-            return Err(Error::PermissionDenied(format!(
src//resource/lifecycle_manager.rs-367-                "Operation {:?} not allowed on resource {} with the provided capabilities",
--
src//resource/relationship/sync.rs-73-    pub timeout_seconds: u64,
src//resource/relationship/sync.rs-74-    
src//resource/relationship/sync.rs:75:    /// Whether to validate resources after synchronization
src//resource/relationship/sync.rs-76-    pub validate: bool,
src//resource/relationship/sync.rs-77-    
--
src//resource/relationship/validation.rs-528-    
src//resource/relationship/validation.rs-529-    /// Validate that a resource exists in a domain
src//resource/relationship/validation.rs:530:    fn validate_resource_exists(&self, resource_id: &ContentId, domain_id: &DomainId) -> Result<()> {
src//resource/relationship/validation.rs-531-        if let Some(lifecycle_manager) = self.lifecycle_managers.get(domain_id) {
src//resource/relationship/validation.rs-532-            // Check if the resource exists
--
src//resource/relationship/validation.rs-549-    
src//resource/relationship/validation.rs-550-    /// Validate that a resource is in one of the valid states
src//resource/relationship/validation.rs:551:    fn validate_resource_state(
src//resource/relationship/validation.rs-552-        &self,
src//resource/relationship/validation.rs-553-        resource_id: &ContentId,
--
src//resource/relationship/validation.rs-659-    
src//resource/relationship/validation.rs-660-    #[test]
src//resource/relationship/validation.rs:661:    fn test_validate_existing_resources() {
src//resource/relationship/validation.rs-662-        let mut validator = CrossDomainRelationshipValidator::new();
src//resource/relationship/validation.rs-663-        
--
src//resource/relationship/validation.rs-684-    
src//resource/relationship/validation.rs-685-    #[test]
src//resource/relationship/validation.rs:686:    fn test_validate_missing_resources() {
src//resource/relationship/validation.rs-687-        let mut validator = CrossDomainRelationshipValidator::new();
src//resource/relationship/validation.rs-688-        
--
src//resource/capability_api.rs-128-        
src//resource/capability_api.rs-129-        self.capability_system
src//resource/capability_api.rs:130:            .validate_capability(capability_id, holder, &right, register_id)
src//resource/capability_api.rs-131-            .map_err(ResourceApiError::Capability)
src//resource/capability_api.rs-132-    }
--
src//resource/capability_system.rs-648-}
src//resource/capability_system.rs-649-
src//resource/capability_system.rs:650:/// Service that validates capabilities against resource lifecycle states
src//resource/capability_system.rs-651-pub struct AuthorizationService {
src//resource/capability_system.rs-652-    /// Reference to the lifecycle manager
--
src//resource/tel.rs-384-        
src//resource/tel.rs-385-        // Validate the operation
src//resource/tel.rs:386:        self.validate_operation(&register_id, operation)?;
src//resource/tel.rs-387-        
src//resource/tel.rs-388-        // Process operation based on type
--
src//resource/tel.rs-475-        
src//resource/tel.rs-476-        // Validate operation against register state
src//resource/tel.rs:477:        self.register_system.validate_operation(&register, &register_op.operation_type)
src//resource/tel.rs-478-            .map_err(|e| Error::ValidationError(format!("Invalid operation for register state: {}", e)))?;
src//resource/tel.rs-479-        
--
src//resource/authorization.rs-2-//
src//resource/authorization.rs-3-// This module provides a capability-based authorization system for resource operations.
src//resource/authorization.rs:4:// It validates that operations on resources are only performed by entities with
src//resource/authorization.rs-5-// the appropriate capabilities, and enforces authorization based on the resource lifecycle state.
src//resource/authorization.rs-6-
--
src//domain/resource_integration.rs-494-                
src//domain/resource_integration.rs-495-                // Validate operation
src//domain/resource_integration.rs:496:                if !adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-497-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-498-                        "Operation not allowed for resource {} in domain {}", 
--
src//domain/resource_integration.rs-510-                
src//domain/resource_integration.rs-511-                // Validate operation
src//domain/resource_integration.rs:512:                if !adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-513-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-514-                        "Operation not allowed for resource {} in domain {}", 
--
src//domain/resource_integration.rs-532-                
src//domain/resource_integration.rs-533-                // Validate operations
src//domain/resource_integration.rs:534:                if !source_adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-535-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-536-                        "Transfer operation not allowed for resource {} in source domain {}", 
--
src//domain/resource_integration.rs-539-                }
src//domain/resource_integration.rs-540-                
src//domain/resource_integration.rs:541:                if !target_adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-542-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-543-                        "Transfer operation not allowed for resource {} in target domain {}", 
--
src//domain/resource_integration.rs-583-                
src//domain/resource_integration.rs-584-                // Validate operation
src//domain/resource_integration.rs:585:                if !adapter.validate_operation(resource_id, &operation).await? {
src//domain/resource_integration.rs-586-                    return Err(Error::AccessDenied(format!(
src//domain/resource_integration.rs-587-                        "Verify operation not allowed for resource {} in domain {}", 
```

### Redundant Error Types

Found the following redundant error types that can be consolidated:

```rust
src//tel/error.rs-16-    
src//tel/error.rs-17-    #[error("Resource error: {0}")]
src//tel/error.rs:18:    ResourceError(String),
src//tel/error.rs-19-    
src//tel/error.rs-20-    #[error("Authorization error: {0}")]
src//tel/error.rs-21-    AuthorizationError(String),
src//tel/error.rs-22-    
src//tel/error.rs-23-    #[error("Resource access denied: {0}")]
--
src//tel/resource/vm.rs-312-        // Get the register
src//tel/resource/vm.rs-313-        let register = self.resource_manager.get_register_by_id(&register_id)?
src//tel/resource/vm.rs:314:            .ok_or_else(|| TelError::ResourceError(format!("Register not found: {:?}", register_id)))?;
src//tel/resource/vm.rs-315-        
src//tel/resource/vm.rs-316-        // Check access control
src//tel/resource/vm.rs-317-        let access_result = self.check_resource_access(
src//tel/resource/vm.rs-318-            &register,
src//tel/resource/vm.rs-319-            ctx,
--
src//tel/resource/version.rs-466-        
src//tel/resource/version.rs-467-        let versions = self.versions.read().map_err(|_| {
src//tel/resource/version.rs:468:            TelError::ResourceError("Failed to acquire read lock on versions".to_string())
src//tel/resource/version.rs-469-        })?;
src//tel/resource/version.rs-470-        
src//tel/resource/version.rs-471-        let resource_versions = versions.get(register_id).cloned()
src//tel/resource/version.rs-472-            .unwrap_or_default()
src//tel/resource/version.rs-473-            .into_iter()
--
src//tel/resource/version.rs-518-    ) -> TelResult<Option<ResourceChange>> {
src//tel/resource/version.rs-519-        let versions = self.versions.read().map_err(|_| {
src//tel/resource/version.rs:520:            TelError::ResourceError("Failed to acquire read lock on versions".to_string())
src//tel/resource/version.rs-521-        })?;
src//tel/resource/version.rs-522-        
src//tel/resource/version.rs-523-        for versions_list in versions.values() {
src//tel/resource/version.rs-524-            for version in versions_list {
src//tel/resource/version.rs-525-                if version.id == *version_id {
--
src//tel/resource/version.rs-557-        let version = match self.get_version(version_id)? {
src//tel/resource/version.rs-558-            Some(v) => v,
src//tel/resource/version.rs:559:            None => return Err(TelError::ResourceError(format!("Version {} not found", version_id))),
src//tel/resource/version.rs-560-        };
src//tel/resource/version.rs-561-        
src//tel/resource/version.rs-562-        // If we don't have contents to restore, we can't do a restore
src//tel/resource/version.rs-563-        let contents = match &version.new_contents {
src//tel/resource/version.rs-564-            Some(c) => c,
src//tel/resource/version.rs:565:            None => return Err(TelError::ResourceError("Cannot restore version with no content".to_string())),
src//tel/resource/version.rs-566-        };
src//tel/resource/version.rs-567-        
src//tel/resource/version.rs-568-        // Call the register update function with the contents
src//tel/resource/version.rs-569-        register_fn(contents)?;
src//tel/resource/version.rs-570-        
--
src//tel/resource/version.rs-610-        
src//tel/resource/version.rs-611-        let mut versions = self.versions.write().map_err(|_| {
src//tel/resource/version.rs:612:            TelError::ResourceError("Failed to acquire write lock on versions".to_string())
src//tel/resource/version.rs-613-        })?;
src//tel/resource/version.rs-614-        
src//tel/resource/version.rs-615-        let mut results = PruningResults {
src//tel/resource/version.rs-616-            versions_pruned: 0,
src//tel/resource/version.rs-617-            resources_affected: 0,
--
src//tel/resource/version.rs-683-        let v1 = match self.get_version(version1)? {
src//tel/resource/version.rs-684-            Some(v) => v,
src//tel/resource/version.rs:685:            None => return Err(TelError::ResourceError(format!("Version {} not found", version1))),
src//tel/resource/version.rs-686-        };
src//tel/resource/version.rs-687-        
src//tel/resource/version.rs-688-        let v2 = match self.get_version(version2)? {
src//tel/resource/version.rs-689-            Some(v) => v,
src//tel/resource/version.rs:690:            None => return Err(TelError::ResourceError(format!("Version {} not found", version2))),
src//tel/resource/version.rs-691-        };
src//tel/resource/version.rs-692-        
src//tel/resource/version.rs-693-        // Make sure they're for the same resource
src//tel/resource/version.rs-694-        if v1.register_id != v2.register_id {
src//tel/resource/version.rs:695:            return Err(TelError::ResourceError("Cannot compare versions from different resources".to_string()));
src//tel/resource/version.rs-696-        }
src//tel/resource/version.rs-697-        
src//tel/resource/version.rs-698-        let mut diff = VersionDiff {
src//tel/resource/version.rs-699-            register_id: v1.register_id.clone(),
src//tel/resource/version.rs-700-            version1: version1.clone(),
--
src//tel/resource/version.rs-826-    fn add_change(&self, change: ResourceChange) -> TelResult<()> {
src//tel/resource/version.rs-827-        let mut versions = self.versions.write().map_err(|_| {
src//tel/resource/version.rs:828:            TelError::ResourceError("Failed to acquire write lock on versions".to_string())
src//tel/resource/version.rs-829-        })?;
src//tel/resource/version.rs-830-        
src//tel/resource/version.rs-831-        let register_id = change.register_id.clone();
src//tel/resource/version.rs-832-        let resource_versions = versions.entry(register_id).or_insert_with(VecDeque::new);
src//tel/resource/version.rs-833-        
--
src//tel/resource/model/register.rs-509-                Ok(())
src//tel/resource/model/register.rs-510-            }
src//tel/resource/model/register.rs:511:            _ => Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-512-                "Cannot lock register in state {:?}", self.state
src//tel/resource/model/register.rs-513-            ))),
src//tel/resource/model/register.rs-514-        }
src//tel/resource/model/register.rs-515-    }
src//tel/resource/model/register.rs-516-    
--
src//tel/resource/model/register.rs-526-                Ok(())
src//tel/resource/model/register.rs-527-            }
src//tel/resource/model/register.rs:528:            _ => Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-529-                "Cannot unlock register in state {:?}", self.state
src//tel/resource/model/register.rs-530-            ))),
src//tel/resource/model/register.rs-531-        }
src//tel/resource/model/register.rs-532-    }
src//tel/resource/model/register.rs-533-    
--
src//tel/resource/model/register.rs-543-                Ok(())
src//tel/resource/model/register.rs-544-            }
src//tel/resource/model/register.rs:545:            _ => Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-546-                "Cannot mark register for deletion in state {:?}", self.state
src//tel/resource/model/register.rs-547-            ))),
src//tel/resource/model/register.rs-548-        }
src//tel/resource/model/register.rs-549-    }
src//tel/resource/model/register.rs-550-    
--
src//tel/resource/model/register.rs-562-                Ok(())
src//tel/resource/model/register.rs-563-            }
src//tel/resource/model/register.rs:564:            _ => Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-565-                "Cannot convert register to tombstone in state {:?}", self.state
src//tel/resource/model/register.rs-566-            ))),
src//tel/resource/model/register.rs-567-        }
src//tel/resource/model/register.rs-568-    }
src//tel/resource/model/register.rs-569-    
--
src//tel/resource/model/register.rs-580-    pub fn update_contents(&mut self, contents: RegisterContents) -> TelResult<()> {
src//tel/resource/model/register.rs-581-        if !self.is_active() {
src//tel/resource/model/register.rs:582:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-583-                "Cannot update register in state {:?}", self.state
src//tel/resource/model/register.rs-584-            )));
src//tel/resource/model/register.rs-585-        }
src//tel/resource/model/register.rs-586-        
src//tel/resource/model/register.rs-587-        self.contents = contents;
--
src//tel/resource/model/register.rs-612-    pub fn transfer(&mut self, new_owner: Address) -> TelResult<()> {
src//tel/resource/model/register.rs-613-        if !self.is_active() {
src//tel/resource/model/register.rs:614:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/register.rs-615-                "Cannot transfer register in state {:?}", self.state
src//tel/resource/model/register.rs-616-            )));
src//tel/resource/model/register.rs-617-        }
src//tel/resource/model/register.rs-618-        
src//tel/resource/model/register.rs-619-        self.owner = new_owner;
--
src//tel/resource/model/manager.rs-128-        registers.get(register_id)
src//tel/resource/model/manager.rs-129-            .cloned()
src//tel/resource/model/manager.rs:130:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-131-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-132-            )))
src//tel/resource/model/manager.rs-133-    }
src//tel/resource/model/manager.rs-134-    
src//tel/resource/model/manager.rs-135-    /// Update a register's contents
--
src//tel/resource/model/manager.rs-143-        
src//tel/resource/model/manager.rs-144-        let register = registers.get_mut(register_id)
src//tel/resource/model/manager.rs:145:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-146-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-147-            )))?;
src//tel/resource/model/manager.rs-148-        
src//tel/resource/model/manager.rs-149-        if !register.is_active() {
src//tel/resource/model/manager.rs:150:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-151-                "Cannot update register {:?} in state {:?}", register_id, register.state
src//tel/resource/model/manager.rs-152-            )));
src//tel/resource/model/manager.rs-153-        }
src//tel/resource/model/manager.rs-154-        
src//tel/resource/model/manager.rs-155-        register.contents = contents;
--
src//tel/resource/model/manager.rs-168-        
src//tel/resource/model/manager.rs-169-        let register = registers.get_mut(register_id)
src//tel/resource/model/manager.rs:170:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-171-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-172-            )))?;
src//tel/resource/model/manager.rs-173-        
src//tel/resource/model/manager.rs-174-        if !register.is_active() && !register.is_locked() && !register.is_frozen() {
src//tel/resource/model/manager.rs:175:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-176-                "Cannot delete register {:?} in state {:?}", register_id, register.state
src//tel/resource/model/manager.rs-177-            )));
src//tel/resource/model/manager.rs-178-        }
src//tel/resource/model/manager.rs-179-        
src//tel/resource/model/manager.rs-180-        // Mark for deletion
--
src//tel/resource/model/manager.rs-199-        
src//tel/resource/model/manager.rs-200-        let register = registers.get_mut(register_id)
src//tel/resource/model/manager.rs:201:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-202-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-203-            )))?;
src//tel/resource/model/manager.rs-204-        
src//tel/resource/model/manager.rs-205-        // Check ownership
src//tel/resource/model/manager.rs-206-        if &register.owner != from {
--
src//tel/resource/model/manager.rs-211-        
src//tel/resource/model/manager.rs-212-        if !register.is_active() {
src//tel/resource/model/manager.rs:213:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-214-                "Cannot transfer register {:?} in state {:?}", register_id, register.state
src//tel/resource/model/manager.rs-215-            )));
src//tel/resource/model/manager.rs-216-        }
src//tel/resource/model/manager.rs-217-        
src//tel/resource/model/manager.rs-218-        // Transfer ownership
--
src//tel/resource/model/manager.rs-232-        
src//tel/resource/model/manager.rs-233-        let register = registers.get_mut(register_id)
src//tel/resource/model/manager.rs:234:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-235-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-236-            )))?;
src//tel/resource/model/manager.rs-237-        
src//tel/resource/model/manager.rs-238-        if !register.is_active() {
src//tel/resource/model/manager.rs:239:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-240-                "Cannot lock register {:?} in state {:?}", register_id, register.state
src//tel/resource/model/manager.rs-241-            )));
src//tel/resource/model/manager.rs-242-        }
src//tel/resource/model/manager.rs-243-        
src//tel/resource/model/manager.rs-244-        register.state = RegisterState::Locked;
--
src//tel/resource/model/manager.rs-257-        
src//tel/resource/model/manager.rs-258-        let register = registers.get_mut(register_id)
src//tel/resource/model/manager.rs:259:            .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-260-                "Register {:?} not found", register_id
src//tel/resource/model/manager.rs-261-            )))?;
src//tel/resource/model/manager.rs-262-        
src//tel/resource/model/manager.rs-263-        if !register.is_locked() {
src//tel/resource/model/manager.rs:264:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-265-                "Cannot unlock register {:?} in state {:?}", register_id, register.state
src//tel/resource/model/manager.rs-266-            )));
src//tel/resource/model/manager.rs-267-        }
src//tel/resource/model/manager.rs-268-        
src//tel/resource/model/manager.rs-269-        register.state = RegisterState::Active;
--
src//tel/resource/model/manager.rs-309-                
src//tel/resource/model/manager.rs-310-                if registers.contains_key(&operation.target) {
src//tel/resource/model/manager.rs:311:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-312-                        "Register {:?} already exists", operation.target
src//tel/resource/model/manager.rs-313-                    )));
src//tel/resource/model/manager.rs-314-                }
src//tel/resource/model/manager.rs-315-                
src//tel/resource/model/manager.rs-316-                // Create the register
--
src//tel/resource/model/manager.rs-321-                // Get contents from inputs
src//tel/resource/model/manager.rs-322-                let contents = operation.inputs.get(0)
src//tel/resource/model/manager.rs:323:                    .ok_or_else(|| TelError::ResourceError(
src//tel/resource/model/manager.rs-324-                        "Create operation must have contents as input".to_string()
src//tel/resource/model/manager.rs-325-                    ))?
src//tel/resource/model/manager.rs-326-                    .clone();
src//tel/resource/model/manager.rs-327-                
src//tel/resource/model/manager.rs-328-                let register = Register::new(
--
src//tel/resource/model/manager.rs-356-                
src//tel/resource/model/manager.rs-357-                let register = registers.get_mut(&operation.target)
src//tel/resource/model/manager.rs:358:                    .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-359-                        "Register {:?} not found", operation.target
src//tel/resource/model/manager.rs-360-                    )))?;
src//tel/resource/model/manager.rs-361-                
src//tel/resource/model/manager.rs-362-                // Verify ownership
src//tel/resource/model/manager.rs-363-                if register.owner != operation.initiator {
--
src//tel/resource/model/manager.rs-370-                // Check register state
src//tel/resource/model/manager.rs-371-                if !register.is_active() {
src//tel/resource/model/manager.rs:372:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-373-                        "Cannot update register {:?} in state {:?}",
src//tel/resource/model/manager.rs-374-                        operation.target, register.state
src//tel/resource/model/manager.rs-375-                    )));
src//tel/resource/model/manager.rs-376-                }
src//tel/resource/model/manager.rs-377-                
src//tel/resource/model/manager.rs-378-                // Get contents from inputs
src//tel/resource/model/manager.rs-379-                let contents = operation.inputs.get(0)
src//tel/resource/model/manager.rs:380:                    .ok_or_else(|| TelError::ResourceError(
src//tel/resource/model/manager.rs-381-                        "Update operation must have contents as input".to_string()
src//tel/resource/model/manager.rs-382-                    ))?
src//tel/resource/model/manager.rs-383-                    .clone();
src//tel/resource/model/manager.rs-384-                
src//tel/resource/model/manager.rs-385-                // Update register
--
src//tel/resource/model/manager.rs-399-                
src//tel/resource/model/manager.rs-400-                let register = registers.get_mut(&operation.target)
src//tel/resource/model/manager.rs:401:                    .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-402-                        "Register {:?} not found", operation.target
src//tel/resource/model/manager.rs-403-                    )))?;
src//tel/resource/model/manager.rs-404-                
src//tel/resource/model/manager.rs-405-                // Verify ownership
src//tel/resource/model/manager.rs-406-                if register.owner != operation.initiator {
--
src//tel/resource/model/manager.rs-413-                // Check register state
src//tel/resource/model/manager.rs-414-                if !register.is_active() && !register.is_locked() && !register.is_frozen() {
src//tel/resource/model/manager.rs:415:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-416-                        "Cannot delete register {:?} in state {:?}",
src//tel/resource/model/manager.rs-417-                        operation.target, register.state
src//tel/resource/model/manager.rs-418-                    )));
src//tel/resource/model/manager.rs-419-                }
src//tel/resource/model/manager.rs-420-                
--
src//tel/resource/model/manager.rs-435-                
src//tel/resource/model/manager.rs-436-                let register = registers.get_mut(&operation.target)
src//tel/resource/model/manager.rs:437:                    .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-438-                        "Register {:?} not found", operation.target
src//tel/resource/model/manager.rs-439-                    )))?;
src//tel/resource/model/manager.rs-440-                
src//tel/resource/model/manager.rs-441-                // Verify ownership
src//tel/resource/model/manager.rs-442-                if register.owner != operation.initiator {
--
src//tel/resource/model/manager.rs-449-                // Check register state
src//tel/resource/model/manager.rs-450-                if !register.is_active() {
src//tel/resource/model/manager.rs:451:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-452-                        "Cannot transfer register {:?} in state {:?}",
src//tel/resource/model/manager.rs-453-                        operation.target, register.state
src//tel/resource/model/manager.rs-454-                    )));
src//tel/resource/model/manager.rs-455-                }
src//tel/resource/model/manager.rs-456-                
--
src//tel/resource/model/manager.rs-458-                let recipient = operation.parameters.get("recipient")
src//tel/resource/model/manager.rs-459-                    .and_then(|v| v.as_str())
src//tel/resource/model/manager.rs:460:                    .ok_or_else(|| TelError::ResourceError(
src//tel/resource/model/manager.rs-461-                        "Transfer operation must specify recipient in parameters".to_string()
src//tel/resource/model/manager.rs-462-                    ))?;
src//tel/resource/model/manager.rs-463-                
src//tel/resource/model/manager.rs-464-                // Parse recipient address
src//tel/resource/model/manager.rs-465-                let to_address = recipient.parse().map_err(|_| 
--
src//tel/resource/model/manager.rs-482-                
src//tel/resource/model/manager.rs-483-                let register = registers.get_mut(&operation.target)
src//tel/resource/model/manager.rs:484:                    .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-485-                        "Register {:?} not found", operation.target
src//tel/resource/model/manager.rs-486-                    )))?;
src//tel/resource/model/manager.rs-487-                
src//tel/resource/model/manager.rs-488-                // Verify ownership
src//tel/resource/model/manager.rs-489-                if register.owner != operation.initiator {
--
src//tel/resource/model/manager.rs-496-                // Check register state
src//tel/resource/model/manager.rs-497-                if !register.is_active() {
src//tel/resource/model/manager.rs:498:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-499-                        "Cannot lock register {:?} in state {:?}",
src//tel/resource/model/manager.rs-500-                        operation.target, register.state
src//tel/resource/model/manager.rs-501-                    )));
src//tel/resource/model/manager.rs-502-                }
src//tel/resource/model/manager.rs-503-                
--
src//tel/resource/model/manager.rs-518-                
src//tel/resource/model/manager.rs-519-                let register = registers.get_mut(&operation.target)
src//tel/resource/model/manager.rs:520:                    .ok_or_else(|| TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-521-                        "Register {:?} not found", operation.target
src//tel/resource/model/manager.rs-522-                    )))?;
src//tel/resource/model/manager.rs-523-                
src//tel/resource/model/manager.rs-524-                // Verify ownership
src//tel/resource/model/manager.rs-525-                if register.owner != operation.initiator {
--
src//tel/resource/model/manager.rs-532-                // Check register state
src//tel/resource/model/manager.rs-533-                if !register.is_locked() {
src//tel/resource/model/manager.rs:534:                    return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-535-                        "Cannot unlock register {:?} in state {:?}",
src//tel/resource/model/manager.rs-536-                        operation.target, register.state
src//tel/resource/model/manager.rs-537-                    )));
src//tel/resource/model/manager.rs-538-                }
src//tel/resource/model/manager.rs-539-                
--
src//tel/resource/model/manager.rs-549-            },
src//tel/resource/model/manager.rs-550-            _ => {
src//tel/resource/model/manager.rs:551:                return Err(TelError::ResourceError(format!(
src//tel/resource/model/manager.rs-552-                    "Unsupported operation type: {:?}", operation.operation_type
src//tel/resource/model/manager.rs-553-                )));
src//tel/resource/model/manager.rs-554-            }
src//tel/resource/model/manager.rs-555-        }
src//tel/resource/model/manager.rs-556-        
--
src//tel/resource/model/guard.rs-127-        
src//tel/resource/model/guard.rs-128-        // Register not found in ACL
src//tel/resource/model/guard.rs:129:        Err(TelError::ResourceError(format!(
src//tel/resource/model/guard.rs-130-            "Register {:?} not found in ACL", register_id
src//tel/resource/model/guard.rs-131-        )))
src//tel/resource/model/guard.rs-132-    }
src//tel/resource/model/guard.rs-133-    
src//tel/resource/model/guard.rs-134-    /// Check if an address has access to a register
--
src//tel/resource/model/guard.rs-207-        // Check if register is active
src//tel/resource/model/guard.rs-208-        if !register.is_active() {
src//tel/resource/model/guard.rs:209:            return Err(TelError::ResourceError(format!(
src//tel/resource/model/guard.rs-210-                "Register {:?} is not in active state", register_id
src//tel/resource/model/guard.rs-211-            )));
src//tel/resource/model/guard.rs-212-        }
src//tel/resource/model/guard.rs-213-        
src//tel/resource/model/guard.rs-214-        // Create guard
--
src//tel/resource/tracking.rs-96-        
src//tel/resource/tracking.rs-97-        if resources.contains_key(&state.resource_id) {
src//tel/resource/tracking.rs:98:            return Err(TelError::ResourceError(format!(
src//tel/resource/tracking.rs-99-                "Resource already exists: {:?}", state.resource_id
src//tel/resource/tracking.rs-100-            )));
src//tel/resource/tracking.rs-101-        }
src//tel/resource/tracking.rs-102-        
src//tel/resource/tracking.rs-103-        resources.insert(state.resource_id, state);
--
src//tel/resource/tracking.rs-210-                        },
src//tel/resource/tracking.rs-211-                        ResourceStatus::Locked { operation_id, expiry } => {
src//tel/resource/tracking.rs:212:                            return Err(TelError::ResourceError(format!(
src//tel/resource/tracking.rs-213-                                "Resource {:?} is locked by operation {:?} until {}", 
src//tel/resource/tracking.rs-214-                                resource_id, operation_id, expiry
src//tel/resource/tracking.rs-215-                            )));
src//tel/resource/tracking.rs-216-                        },
src//tel/resource/tracking.rs-217-                        ResourceStatus::Frozen { reason, .. } => {
src//tel/resource/tracking.rs:218:                            return Err(TelError::ResourceError(format!(
src//tel/resource/tracking.rs-219-                                "Resource {:?} is frozen: {}", resource_id, reason
src//tel/resource/tracking.rs-220-                            )));
src//tel/resource/tracking.rs-221-                        },
src//tel/resource/tracking.rs-222-                        ResourceStatus::PendingDeletion { .. } => {
src//tel/resource/tracking.rs:223:                            return Err(TelError::ResourceError(format!(
src//tel/resource/tracking.rs-224-                                "Resource {:?} is pending deletion", resource_id
src//tel/resource/tracking.rs-225-                            )));
src//tel/resource/tracking.rs-226-                        },
src//tel/resource/tracking.rs-227-                        ResourceStatus::Tombstone { .. } => {
src//tel/resource/tracking.rs:228:                            return Err(TelError::ResourceError(format!(
src//tel/resource/tracking.rs-229-                                "Resource {:?} has been deleted", resource_id
src//tel/resource/tracking.rs-230-                            )));
src//tel/resource/tracking.rs-231-                        },
src//tel/resource/tracking.rs-232-                    }
src//tel/resource/tracking.rs-233-                },
--
src//operation/transformation.rs-40-
src//operation/transformation.rs-41-    #[error("Resource transformation error: {0}")]
src//operation/transformation.rs:42:    ResourceError(String),
src//operation/transformation.rs-43-
src//operation/transformation.rs-44-    #[error("Authorization transformation error: {0}")]
src//operation/transformation.rs-45-    AuthorizationError(String),
src//operation/transformation.rs-46-
src//operation/transformation.rs-47-    #[error("Transformation not implemented: {0}")]
--
src//operation/transformation.rs-152-    let register_id = operation.outputs.first()
src//operation/transformation.rs-153-        .map(|output| output.resource_id.clone())
src//operation/transformation.rs:154:        .ok_or_else(|| TransformationError::ResourceError(
src//operation/transformation.rs-155-            "No output resource specified for register operation".to_string()
src//operation/transformation.rs-156-        ))?;
src//operation/transformation.rs-157-    
src//operation/transformation.rs-158-    // Map the operation type to register operation type
src//operation/transformation.rs-159-    let register_op_type = match operation.op_type {
--
src//operation/transformation.rs-206-        block_height: None, // Will be filled in after execution
src//operation/transformation.rs-207-        data: serde_json::to_vec(&register_op)
src//operation/transformation.rs:208:            .map_err(|e| TransformationError::ResourceError(e.to_string()))?,
src//operation/transformation.rs-209-    };
src//operation/transformation.rs-210-    
src//operation/transformation.rs-211-    // Create a new operation with the physical context
src//operation/transformation.rs-212-    Ok(Operation {
src//operation/transformation.rs-213-        id: operation.id.clone(),
--
src//effect/transfer_effect.rs-105-            &self.params.source_resource_id,
src//effect/transfer_effect.rs-106-        ).await.map_err(|e| match e {
src//effect/transfer_effect.rs:107:            ResourceApiError::NotFound(_) => EffectError::ResourceError(
src//effect/transfer_effect.rs-108-                format!("Source resource not found: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-109-            ),
src//effect/transfer_effect.rs-110-            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
src//effect/transfer_effect.rs-111-                format!("Access denied to source resource: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-112-            ),
--
src//effect/transfer_effect.rs-119-            &self.params.destination_resource_id,
src//effect/transfer_effect.rs-120-        ).await.map_err(|e| match e {
src//effect/transfer_effect.rs:121:            ResourceApiError::NotFound(_) => EffectError::ResourceError(
src//effect/transfer_effect.rs-122-                format!("Destination resource not found: {}", self.params.destination_resource_id)
src//effect/transfer_effect.rs-123-            ),
src//effect/transfer_effect.rs-124-            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
src//effect/transfer_effect.rs-125-                format!("Access denied to destination resource: {}", self.params.destination_resource_id)
src//effect/transfer_effect.rs-126-            ),
--
src//effect/transfer_effect.rs-130-        // Validate transfer constraints
src//effect/transfer_effect.rs-131-        if source_resource.is_locked() {
src//effect/transfer_effect.rs:132:            return Err(EffectError::ResourceError(
src//effect/transfer_effect.rs-133-                format!("Source resource is locked: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-134-            ));
src//effect/transfer_effect.rs-135-        }
src//effect/transfer_effect.rs-136-        
src//effect/transfer_effect.rs-137-        if dest_resource.is_locked() {
src//effect/transfer_effect.rs:138:            return Err(EffectError::ResourceError(
src//effect/transfer_effect.rs-139-                format!("Destination resource is locked: {}", self.params.destination_resource_id)
src//effect/transfer_effect.rs-140-            ));
src//effect/transfer_effect.rs-141-        }
src//effect/transfer_effect.rs-142-        
src//effect/transfer_effect.rs-143-        let source_data = source_resource.data(source_capability).await.map_err(|e| 
src//effect/transfer_effect.rs:144:            EffectError::ResourceError(format!("Failed to read source data: {}", e))
src//effect/transfer_effect.rs-145-        )?;
src//effect/transfer_effect.rs-146-        let dest_data = dest_resource.data(dest_capability).await.map_err(|e|
src//effect/transfer_effect.rs:147:            EffectError::ResourceError(format!("Failed to read destination data: {}", e))
src//effect/transfer_effect.rs-148-        )?;
src//effect/transfer_effect.rs-149-        
src//effect/transfer_effect.rs-150-        if source_data.is_empty() {
src//effect/transfer_effect.rs:151:            return Err(EffectError::ResourceError(
src//effect/transfer_effect.rs-152-                format!("Source resource is empty: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-153-            ));
src//effect/transfer_effect.rs-154-        }
src//effect/transfer_effect.rs-155-        
src//effect/transfer_effect.rs-156-        // Handle specific transfer logic depending on resource types
--
src//effect/transfer_effect.rs-159-                // Fungible asset transfer
src//effect/transfer_effect.rs-160-                let source_amount = source_resource.get_amount()
src//effect/transfer_effect.rs:161:                    .ok_or_else(|| EffectError::ResourceError(
src//effect/transfer_effect.rs-162-                        format!("Source resource does not have an amount: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-163-                    ))?;
src//effect/transfer_effect.rs-164-                
src//effect/transfer_effect.rs-165-                if source_amount < amount {
src//effect/transfer_effect.rs:166:                    return Err(EffectError::ResourceError(
src//effect/transfer_effect.rs-167-                        format!("Insufficient amount in source resource: {}", self.params.source_resource_id)
src//effect/transfer_effect.rs-168-                    ));
src//effect/transfer_effect.rs-169-                }
src//effect/transfer_effect.rs-170-                
src//effect/transfer_effect.rs-171-                let dest_amount = dest_resource.get_amount().unwrap_or(0);
--
src//effect/mod.rs-222-    
src//effect/mod.rs-223-    #[error("Resource error: {0}")]
src//effect/mod.rs:224:    ResourceError(String),
src//effect/mod.rs-225-    
src//effect/mod.rs-226-    #[error("Execution error: {0}")]
src//effect/mod.rs-227-    ExecutionError(String),
src//effect/mod.rs-228-    
src//effect/mod.rs-229-    #[error("Boundary error: {0}")]
--
src//execution/context.rs-166-    EffectError(String),
src//execution/context.rs-167-    /// Resource error
src//execution/context.rs:168:    ResourceError(String),
src//execution/context.rs-169-    /// Security error
src//execution/context.rs-170-    SecurityError(String),
src//execution/context.rs-171-    /// Timeout error
src//execution/context.rs-172-    TimeoutError,
src//execution/context.rs-173-    /// Out of memory error
--
src//execution/context.rs-182-            ExecutionError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
src//execution/context.rs-183-            ExecutionError::EffectError(msg) => write!(f, "Effect error: {}", msg),
src//execution/context.rs:184:            ExecutionError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
src//execution/context.rs-185-            ExecutionError::SecurityError(msg) => write!(f, "Security error: {}", msg),
src//execution/context.rs-186-            ExecutionError::TimeoutError => write!(f, "Execution timed out"),
src//execution/context.rs-187-            ExecutionError::OutOfMemory => write!(f, "Out of memory"),
src//execution/context.rs-188-        }
src//execution/context.rs-189-    }
--
src//resource/tests/effect_tests.rs-82-                    vec![1, 2, 3, 4],
src//resource/tests/effect_tests.rs-83-                    metadata,
src//resource/tests/effect_tests.rs:84:                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
src//resource/tests/effect_tests.rs-85-                
src//resource/tests/effect_tests.rs-86-                ResourceChange::Created { resource_id: self.resource_id.clone() }
src//resource/tests/effect_tests.rs-87-            },
src//resource/tests/effect_tests.rs-88-            "update" => {
src//resource/tests/effect_tests.rs-89-                // Update an existing resource
--
src//resource/tests/effect_tests.rs-93-                    Some(vec![5, 6, 7, 8]),
src//resource/tests/effect_tests.rs-94-                    None,
src//resource/tests/effect_tests.rs:95:                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
src//resource/tests/effect_tests.rs-96-                
src//resource/tests/effect_tests.rs-97-                ResourceChange::Updated { resource_id: self.resource_id.clone() }
src//resource/tests/effect_tests.rs-98-            },
src//resource/tests/effect_tests.rs-99-            "delete" => {
src//resource/tests/effect_tests.rs-100-                // Delete an existing resource
--
src//resource/tests/effect_tests.rs-102-                    capability,
src//resource/tests/effect_tests.rs-103-                    &self.resource_id,
src//resource/tests/effect_tests.rs:104:                ).await.map_err(|e| crate::effect::EffectError::ResourceError(e.to_string()))?;
src//resource/tests/effect_tests.rs-105-                
src//resource/tests/effect_tests.rs-106-                ResourceChange::Deleted { resource_id: self.resource_id.clone() }
src//resource/tests/effect_tests.rs-107-            },
src//resource/tests/effect_tests.rs-108-            other => {
src//resource/tests/effect_tests.rs-109-                // Unknown action
--
src//resource/content_addressed_register.rs-256-/// Error types for register operations
src//resource/content_addressed_register.rs-257-#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
src//resource/content_addressed_register.rs:258:pub enum RegisterError {
src//resource/content_addressed_register.rs-259-    /// Register not found
src//resource/content_addressed_register.rs-260-    NotFound(String),
src//resource/content_addressed_register.rs-261-    
src//resource/content_addressed_register.rs-262-    /// Register already exists
src//resource/content_addressed_register.rs-263-    AlreadyExists(String),
--
src//resource/content_addressed_register.rs-443-        &mut self, 
src//resource/content_addressed_register.rs-444-        operation: ContentAddressedRegisterOperation
src//resource/content_addressed_register.rs:445:    ) -> std::result::Result<ContentId, RegisterError> {
src//resource/content_addressed_register.rs-446-        // Get the target register
src//resource/content_addressed_register.rs-447-        let target_register = self.get_register(&operation.target_register)
src//resource/content_addressed_register.rs:448:            .ok_or_else(|| RegisterError::NotFound(
src//resource/content_addressed_register.rs-449-                format!("Register not found: {:?}", operation.target_register)
src//resource/content_addressed_register.rs-450-            ))?
src//resource/content_addressed_register.rs-451-            .clone();
src//resource/content_addressed_register.rs-452-            
src//resource/content_addressed_register.rs-453-        // Process based on operation type
--
src//resource/content_addressed_register.rs-455-            RegisterOperationType::CreateRegister => {
src//resource/content_addressed_register.rs-456-                // Can't create an existing register
src//resource/content_addressed_register.rs:457:                return Err(RegisterError::AlreadyExists(
src//resource/content_addressed_register.rs-458-                    format!("Register already exists: {:?}", operation.target_register)
src//resource/content_addressed_register.rs-459-                ));
src//resource/content_addressed_register.rs-460-            },
src//resource/content_addressed_register.rs-461-            RegisterOperationType::UpdateRegister => {
src//resource/content_addressed_register.rs-462-                let mut register = target_register.clone();
--
src//resource/manager.rs-182-    pub fn allocate_resources(&self, request: ResourceRequest) -> Result<ResourceGuard> {
src//resource/manager.rs-183-        let grant = self.allocator.allocate(request)
src//resource/manager.rs:184:            .map_err(|e| Error::ResourceError(format!("Failed to allocate resources: {}", e)))?;
src//resource/manager.rs-185-        
src//resource/manager.rs-186-        // Add to active grants
src//resource/manager.rs-187-        let mut active_grants = self.active_grants.write().unwrap();
src//resource/manager.rs-188-        active_grants.insert(grant.grant_id.clone(), grant.clone());
src//resource/manager.rs-189-        
--
src//resource/manager.rs-209-        // Allocate resources for the register
src//resource/manager.rs-210-        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
src//resource/manager.rs:211:            Error::ResourceError("No grant returned from allocator".to_string())
src//resource/manager.rs-212-        })?.clone();
src//resource/manager.rs-213-        
src//resource/manager.rs-214-        let content_id = register.id.clone();
src//resource/manager.rs-215-        
src//resource/manager.rs-216-        // If unified system is available, create through lifecycle manager
src//resource/manager.rs-217-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-218-            lifecycle_manager.register_resource(register.clone())
src//resource/manager.rs:219:                .map_err(|e| Error::ResourceError(format!("Failed to register resource: {}", e)))?;
src//resource/manager.rs-220-                
src//resource/manager.rs-221-            // Store the register in our registry
src//resource/manager.rs-222-            let mut registers = self.registers.write().unwrap();
src//resource/manager.rs-223-            registers.insert(content_id.clone(), register);
src//resource/manager.rs-224-        } else {
--
src//resource/manager.rs-241-                if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-242-                    lifecycle_manager.get_resource_by_id(content_id)
src//resource/manager.rs:243:                        .map_err(|e| Error::ResourceError(format!("Failed to get register: {}", e)))
src//resource/manager.rs-244-                } else {
src//resource/manager.rs-245-                    Ok(None)
src//resource/manager.rs-246-                }
src//resource/manager.rs-247-            }
src//resource/manager.rs-248-        }
--
src//resource/manager.rs-259-            // Get current register
src//resource/manager.rs-260-            let register = self.get_register(content_id)?
src//resource/manager.rs:261:                .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", content_id)))?;
src//resource/manager.rs-262-            
src//resource/manager.rs-263-            // Create updated register
src//resource/manager.rs-264-            let mut updated_register = register.clone();
src//resource/manager.rs-265-            update_fn(&mut updated_register)?;
src//resource/manager.rs-266-            
src//resource/manager.rs-267-            // Update the resource
src//resource/manager.rs-268-            lifecycle_manager.update_resource(content_id, updated_register.clone())
src//resource/manager.rs:269:                .map_err(|e| Error::ResourceError(format!("Failed to update register: {}", e)))?;
src//resource/manager.rs-270-                
src//resource/manager.rs-271-            // Update our registry
src//resource/manager.rs-272-            let mut registers = self.registers.write().unwrap();
src//resource/manager.rs-273-            registers.insert(content_id.clone(), updated_register);
src//resource/manager.rs-274-            
--
src//resource/manager.rs-283-                    Ok(())
src//resource/manager.rs-284-                },
src//resource/manager.rs:285:                None => Err(Error::ResourceError(format!("Register not found: {:?}", content_id))),
src//resource/manager.rs-286-            }
src//resource/manager.rs-287-        }
src//resource/manager.rs-288-    }
src//resource/manager.rs-289-    
src//resource/manager.rs-290-    /// Lock a ResourceRegister
--
src//resource/manager.rs-293-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-294-            lifecycle_manager.lock_resource(content_id, reason)
src//resource/manager.rs:295:                .map_err(|e| Error::ResourceError(format!("Failed to lock register: {}", e)))?;
src//resource/manager.rs-296-                
src//resource/manager.rs-297-            // Update our registry
src//resource/manager.rs-298-            if let Some(mut register) = self.get_register(content_id)? {
src//resource/manager.rs-299-                register.state = RegisterState::Locked;
src//resource/manager.rs-300-                
--
src//resource/manager.rs-318-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-319-            lifecycle_manager.unlock_resource(content_id, reason)
src//resource/manager.rs:320:                .map_err(|e| Error::ResourceError(format!("Failed to unlock register: {}", e)))?;
src//resource/manager.rs-321-                
src//resource/manager.rs-322-            // Update our registry
src//resource/manager.rs-323-            if let Some(mut register) = self.get_register(content_id)? {
src//resource/manager.rs-324-                register.state = RegisterState::Active;
src//resource/manager.rs-325-                
--
src//resource/manager.rs-343-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-344-            lifecycle_manager.consume_resource(content_id, reason)
src//resource/manager.rs:345:                .map_err(|e| Error::ResourceError(format!("Failed to consume register: {}", e)))?;
src//resource/manager.rs-346-                
src//resource/manager.rs-347-            // Update our registry
src//resource/manager.rs-348-            if let Some(mut register) = self.get_register(content_id)? {
src//resource/manager.rs-349-                register.state = RegisterState::Consumed;
src//resource/manager.rs-350-                
--
src//resource/manager.rs-375-            // Get the source and target registers
src//resource/manager.rs-376-            let source = self.get_register(source_id)?
src//resource/manager.rs:377:                .ok_or_else(|| Error::ResourceError(format!("Source register not found: {:?}", source_id)))?;
src//resource/manager.rs-378-                
src//resource/manager.rs-379-            let target = self.get_register(target_id)?
src//resource/manager.rs:380:                .ok_or_else(|| Error::ResourceError(format!("Target register not found: {:?}", target_id)))?;
src//resource/manager.rs-381-            
src//resource/manager.rs-382-            // Parse the relationship type and direction
src//resource/manager.rs-383-            use crate::resource::relationship_tracker::{RelationshipType, RelationshipDirection};
src//resource/manager.rs-384-            
src//resource/manager.rs-385-            let rel_type = match relationship_type {
--
src//resource/manager.rs-395-                "child_to_parent" => RelationshipDirection::ChildToParent,
src//resource/manager.rs-396-                "bidirectional" => RelationshipDirection::Bidirectional,
src//resource/manager.rs:397:                _ => return Err(Error::ResourceError(format!("Invalid relationship direction: {}", direction))),
src//resource/manager.rs-398-            };
src//resource/manager.rs-399-            
src//resource/manager.rs-400-            // Record the relationship
src//resource/manager.rs-401-            relationship_tracker.record_relationship_between_registers(
src//resource/manager.rs-402-                &source,
--
src//resource/manager.rs-405-                rel_direction,
src//resource/manager.rs-406-                None,
src//resource/manager.rs:407:            ).map_err(|e| Error::ResourceError(format!("Failed to create relationship: {}", e)))?;
src//resource/manager.rs-408-            
src//resource/manager.rs-409-            Ok(())
src//resource/manager.rs-410-        } else {
src//resource/manager.rs:411:            Err(Error::ResourceError("Relationship tracker not configured".to_string()))
src//resource/manager.rs-412-        }
src//resource/manager.rs-413-    }
src//resource/manager.rs-414-    
src//resource/manager.rs-415-    /// Get related ResourceRegisters
src//resource/manager.rs-416-    pub fn get_related_registers(
--
src//resource/manager.rs-436-                &rel_type,
src//resource/manager.rs-437-                None,
src//resource/manager.rs:438:            ).map_err(|e| Error::ResourceError(format!("Failed to get related resources: {}", e)))?;
src//resource/manager.rs-439-            
src//resource/manager.rs-440-            // Get the registers
src//resource/manager.rs-441-            let mut related_registers = Vec::new();
src//resource/manager.rs-442-            for id in related_ids {
src//resource/manager.rs-443-                if let Some(register) = self.get_register(&id)? {
--
src//resource/manager.rs-448-            Ok(related_registers)
src//resource/manager.rs-449-        } else {
src//resource/manager.rs:450:            Err(Error::ResourceError("Relationship tracker not configured".to_string()))
src//resource/manager.rs-451-        }
src//resource/manager.rs-452-    }
src//resource/manager.rs-453-    
src//resource/manager.rs-454-    /// Transfer a ResourceRegister to another domain
src//resource/manager.rs-455-    pub fn transfer_register(
--
src//resource/manager.rs-461-        // Get the register
src//resource/manager.rs-462-        let register = self.get_register(content_id)?
src//resource/manager.rs:463:            .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", content_id)))?;
src//resource/manager.rs-464-            
src//resource/manager.rs-465-        // Handle the transfer
src//resource/manager.rs-466-        let result = self.boundary_manager.cross_domain_transfer(
src//resource/manager.rs-467-            register,
src//resource/manager.rs-468-            self.domain_id.clone(),
src//resource/manager.rs-469-            target_domain.clone(),
src//resource/manager.rs-470-            quantity,
src//resource/manager.rs:471:        ).map_err(|e| Error::ResourceError(format!("Failed to transfer register: {}", e)))?;
src//resource/manager.rs-472-        
src//resource/manager.rs-473-        // If unified system is available, update through lifecycle manager
src//resource/manager.rs-474-        if let Some(lifecycle_manager) = &self.lifecycle_manager {
src//resource/manager.rs-475-            // Update the source register if quantity was specified (partial transfer)
src//resource/manager.rs-476-            if let Some(qty) = quantity {
--
src//resource/manager.rs-481-                    
src//resource/manager.rs-482-                    lifecycle_manager.update_resource(content_id, updated.clone())
src//resource/manager.rs:483:                        .map_err(|e| Error::ResourceError(format!("Failed to update source register: {}", e)))?;
src//resource/manager.rs-484-                        
src//resource/manager.rs-485-                    // Update our registry
src//resource/manager.rs-486-                    let mut registers = self.registers.write().unwrap();
src//resource/manager.rs-487-                    registers.insert(content_id.clone(), updated);
src//resource/manager.rs-488-                }
--
src//resource/tel.rs-10-use crate::resource::{
src//resource/tel.rs-11-    Register, ContentId, RegisterContents, RegisterState, RegisterOperation, OperationType,
src//resource/tel.rs:12:    OneTimeRegisterSystem, RegisterResult, RegisterError, TimeMap, TimeMapEntry
src//resource/tel.rs-13-};
src//resource/tel.rs-14-use crate::domain::{DomainId, DomainRegistry};
src//resource/tel.rs-15-use crate::crypto::hash::{ContentId, HashOutput, HashFactory, HashError};
src//resource/tel.rs-16-
src//resource/tel.rs-17-// TEL resource types
--
src//resource/tel.rs-237-        let register_id = register.register_id.clone();
src//resource/tel.rs-238-        self.register_system.import_register(register, "tel-import")
src//resource/tel.rs:239:            .map_err(|e| Error::RegisterError(format!("Failed to import register: {}", e)))?;
src//resource/tel.rs-240-        
src//resource/tel.rs-241-        // Map the IDs
src//resource/tel.rs-242-        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
src//resource/tel.rs-243-        
src//resource/tel.rs-244-        Ok(register_id)
--
src//resource/tel.rs-306-                new_register, 
src//resource/tel.rs-307-                "tel-sync"
src//resource/tel.rs:308:            ).map_err(|e| Error::RegisterError(format!("Failed to update register: {}", e)))?;
src//resource/tel.rs-309-        }
src//resource/tel.rs-310-        
src//resource/tel.rs-311-        Ok(())
src//resource/tel.rs-312-    }
src//resource/tel.rs-313-    
--
src//resource/tel.rs-399-                // Consume the register
src//resource/tel.rs-400-                self.register_system.consume_register_by_id(&register_id, "tel-operation", Vec::new())
src//resource/tel.rs:401:                    .map_err(|e| Error::RegisterError(format!("Failed to consume register: {}", e)))?;
src//resource/tel.rs-402-                
src//resource/tel.rs-403-                // Remove the mapping
src//resource/tel.rs-404-                self.mapping.remove_mapping(&tel_resource_id, &register_id)
src//resource/tel.rs-405-            },
src//resource/tel.rs-406-            ResourceOperationType::Transfer => {
--
src//resource/tel.rs-411-                // Lock the register
src//resource/tel.rs-412-                self.register_system.lock_register_by_id(&register_id, "tel-operation")
src//resource/tel.rs:413:                    .map_err(|e| Error::RegisterError(format!("Failed to lock register: {}", e)))
src//resource/tel.rs-414-            },
src//resource/tel.rs-415-            ResourceOperationType::Unlock => {
src//resource/tel.rs-416-                // Unlock the register
src//resource/tel.rs-417-                self.register_system.unlock_register_by_id(&register_id, "tel-operation")
src//resource/tel.rs:418:                    .map_err(|e| Error::RegisterError(format!("Failed to unlock register: {}", e)))
src//resource/tel.rs-419-            },
src//resource/tel.rs-420-        }
src//resource/tel.rs-421-    }
src//resource/tel.rs-422-
src//resource/tel.rs-423-    /// Convert TEL operation to Register operation
--
src//resource/tel.rs-548-            register.contents,
src//resource/tel.rs-549-            transaction_id
src//resource/tel.rs:550:        ).map_err(|e| Error::RegisterError(format!("Failed to create register in domain: {}", e)))?;
src//resource/tel.rs-551-        
src//resource/tel.rs-552-        // Map the IDs
src//resource/tel.rs-553-        self.mapping.map_resource(tel_id.clone(), register.register_id.clone())?;
src//resource/tel.rs-554-        
src//resource/tel.rs-555-        Ok(register.register_id)
--
src//resource/tel.rs-583-            register.contents,
src//resource/tel.rs-584-            transaction_id
src//resource/tel.rs:585:        ).map_err(|e| Error::RegisterError(format!("Failed to create register with time info: {}", e)))?;
src//resource/tel.rs-586-        
src//resource/tel.rs-587-        // Map the IDs
src//resource/tel.rs-588-        self.mapping.map_resource(tel_id.clone(), register.register_id.clone())?;
src//resource/tel.rs-589-        
src//resource/tel.rs-590-        Ok(register.register_id)
--
src//resource/tel.rs-710-            contents,
src//resource/tel.rs-711-            "test",
src//resource/tel.rs:712:        ).map_err(|e| Error::RegisterError(format!("Failed to create register: {}", e)))?;
src//resource/tel.rs-713-        
src//resource/tel.rs-714-        // Export the register to TEL
src//resource/tel.rs-715-        let exported_tel_id = adapter.export_register_to_tel(&register.register_id)?;
src//resource/tel.rs-716-        
src//resource/tel.rs-717-        // Verify mapping
--
src//resource/tel.rs-768-            updated_register,
src//resource/tel.rs-769-            "test-update"
src//resource/tel.rs:770:        ).map_err(|e| Error::RegisterError(format!("Failed to update register: {}", e)))?;
src//resource/tel.rs-771-        
src//resource/tel.rs-772-        // Sync from register to TEL
src//resource/tel.rs-773-        adapter.sync_to_tel(&register_id)?;
src//resource/tel.rs-774-        
src//resource/tel.rs-775-        // Verify the TEL register was updated
```

### Files That May Be Removable After Migration



## Recommendation Summary

After completing the migration to the unified ResourceRegister model, consider removing or consolidating the following types of redundant code:

1. Resource/Register conversion functions (to_register, from_register, etc.)
2. Synchronization logic between Resource and Register models
3. Dual lookups that query both resource and register systems
4. Duplicate validation logic in both resource and register systems
5. Redundant error types (ResourceError and RegisterError can be consolidated)
6. Files dedicated to the separate Resource and Register models

These changes should result in a significantly cleaner codebase with less maintenance overhead.
