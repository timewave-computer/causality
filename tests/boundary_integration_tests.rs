wuse std::collections::HashMap;
use std::sync::{Arc, Mutex};

use causality::boundary::{
    BoundarySystem,
    BoundarySystemConfig,
    BoundaryType,
    CrossingType,
    BoundarySafe,
    BoundaryAuthentication,
    BoundaryCrossingError,
    BoundaryCrossingProtocol,
    BoundaryCrossingPayload,
    OnChainEnvironment,
    ChainAddress,
    OffChainComponentType,
    ComponentId,
    ComponentConfig,
    ConnectionDetails,
    SecuritySettings,
    boundary_system,
};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

// Helper macro for running tokio tests
#[macro_export]
macro_rules! boundary_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                $body.await;
            });
        }
    };
}

/// Test data structure that can cross boundaries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestData {
    id: String,
    value: i32,
    metadata: HashMap<String, String>,
}

impl BoundarySafe for TestData {
    fn target_boundary(&self) -> BoundaryType {
        BoundaryType::InsideSystem
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize TestData: {}", e))
    }
}

/// Test on-chain component for integration testing
struct TestContract {
    address: ChainAddress,
    calls: Mutex<Vec<String>>,
    values: Mutex<HashMap<String, i32>>,
}

impl TestContract {
    fn new(address: &str) -> Self {
        Self {
            address: ChainAddress::Ethereum(address.to_string()),
            calls: Mutex::new(Vec::new()),
            values: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl causality::boundary::ContractInterface for TestContract {
    fn environment(&self) -> OnChainEnvironment {
        OnChainEnvironment::EVM
    }
    
    fn address(&self) -> ChainAddress {
        self.address.clone()
    }
    
    async fn call_method(
        &self,
        method: &str,
        args: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, String> {
        // Record this call
        self.calls.lock().unwrap().push(format!("call:{}", method));
        
        match method {
            "getValue" => {
                let key = args.get("key")
                    .ok_or_else(|| "Missing key parameter".to_string())?;
                let key = String::from_utf8(key.clone())
                    .map_err(|e| format!("Invalid key: {}", e))?;
                
                let values = self.values.lock().unwrap();
                let value = values.get(&key).copied().unwrap_or(0);
                
                Ok(value.to_be_bytes().to_vec())
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
    
    async fn submit_transaction(
        &self,
        method: &str,
        args: HashMap<String, Vec<u8>>,
        _auth: BoundaryAuthentication,
    ) -> Result<String, String> {
        // Record this transaction
        self.calls.lock().unwrap().push(format!("tx:{}", method));
        
        match method {
            "setValue" => {
                let key = args.get("key")
                    .ok_or_else(|| "Missing key parameter".to_string())?;
                let key = String::from_utf8(key.clone())
                    .map_err(|e| format!("Invalid key: {}", e))?;
                
                let value_bytes = args.get("value")
                    .ok_or_else(|| "Missing value parameter".to_string())?;
                
                let value = if value_bytes.len() == 4 {
                    let mut bytes = [0; 4];
                    bytes.copy_from_slice(value_bytes);
                    i32::from_be_bytes(bytes)
                } else {
                    return Err("Invalid value format".to_string());
                };
                
                let mut values = self.values.lock().unwrap();
                values.insert(key, value);
                
                Ok("0xabcdef1234567890".to_string())
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
    
    fn get_interface(&self) -> String {
        r#"{
            "methods": {
                "getValue": {
                    "inputs": [{"name": "key", "type": "string"}],
                    "outputs": [{"name": "value", "type": "int32"}]
                },
                "setValue": {
                    "inputs": [
                        {"name": "key", "type": "string"},
                        {"name": "value", "type": "int32"}
                    ],
                    "outputs": []
                }
            }
        }"#.to_string()
    }
}

/// Test off-chain component for integration testing
struct TestService {
    config: ComponentConfig,
    data: Mutex<HashMap<String, TestData>>,
    operations: Mutex<Vec<String>>,
}

impl TestService {
    fn new(id: &str, version: &str) -> Self {
        let component_id = ComponentId::new(
            OffChainComponentType::ApiService,
            id,
            version,
        );
        
        let connection = ConnectionDetails {
            host: "localhost".to_string(),
            port: Some(8080),
            protocol: "http".to_string(),
            path: Some("/api".to_string()),
            params: HashMap::new(),
        };
        
        let security = SecuritySettings {
            auth_type: "token".to_string(),
            credentials: Some({
                let mut creds = HashMap::new();
                creds.insert("token".to_string(), "test-token".to_string());
                creds
            }),
            tls_enabled: false,
            verify_cert: false,
            rate_limit: Some(10),
            timeout_seconds: 5,
        };
        
        let config = ComponentConfig {
            id: component_id,
            connection,
            security,
            settings: HashMap::new(),
        };
        
        Self {
            config,
            data: Mutex::new(HashMap::new()),
            operations: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl causality::boundary::OffChainComponent for TestService {
    fn id(&self) -> &ComponentId {
        &self.config.id
    }
    
    fn component_type(&self) -> OffChainComponentType {
        OffChainComponentType::ApiService
    }
    
    fn config(&self) -> &ComponentConfig {
        &self.config
    }
    
    async fn initialize(&self) -> Result<(), String> {
        self.operations.lock().unwrap().push("initialize".to_string());
        Ok(())
    }
    
    async fn is_available(&self) -> bool {
        self.operations.lock().unwrap().push("is_available".to_string());
        true
    }
    
    async fn execute_request(
        &self,
        operation: &str,
        params: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, String> {
        self.operations.lock().unwrap().push(format!("execute:{}", operation));
        
        match operation {
            "getData" => {
                let id = params.get("id")
                    .ok_or_else(|| "Missing id parameter".to_string())?;
                let id = String::from_utf8(id.clone())
                    .map_err(|e| format!("Invalid id: {}", e))?;
                
                let data = self.data.lock().unwrap();
                if let Some(item) = data.get(&id) {
                    serde_json::to_vec(item)
                        .map_err(|e| format!("Failed to serialize data: {}", e))
                } else {
                    Err(format!("Data not found: {}", id))
                }
            },
            "saveData" => {
                let data_json = params.get("data")
                    .ok_or_else(|| "Missing data parameter".to_string())?;
                let data: TestData = serde_json::from_slice(data_json)
                    .map_err(|e| format!("Invalid data: {}", e))?;
                
                let mut data_map = self.data.lock().unwrap();
                data_map.insert(data.id.clone(), data.clone());
                
                Ok(b"success".to_vec())
            },
            _ => Err(format!("Unknown operation: {}", operation)),
        }
    }
    
    async fn close(&self) -> Result<(), String> {
        self.operations.lock().unwrap().push("close".to_string());
        Ok(())
    }
}

// Integration tests for boundary system

// Test for basic boundary crossing
boundary_test!(test_basic_boundary_crossing, async {
    // Initialize boundary system
    let boundary_system = BoundarySystem::new();
    
    // Create test data
    let test_data = TestData {
        id: "test1".to_string(),
        value: 42,
        metadata: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        },
    };
    
    // Get crossing registry
    let registry = boundary_system.crossing_registry();
    
    // Find inside to outside protocol
    let protocol = registry.find_protocol_for_boundaries(
        BoundaryType::InsideSystem,
        BoundaryType::OutsideSystem,
    ).expect("Protocol should exist");
    
    // Prepare data for crossing
    let payload = protocol.prepare_outgoing(
        &test_data,
        BoundaryAuthentication::None,
    ).expect("Should prepare payload");
    
    // Process the crossing
    let result_data = registry.process_crossing(
        protocol.name(),
        payload,
    ).expect("Should process crossing");
    
    // Deserialize the result
    let result: TestData = TestData::from_crossing(&result_data)
        .expect("Should deserialize result");
    
    // Verify the result matches original data
    assert_eq!(result, test_data);
});

// Test for on-chain component integration
#[cfg(feature = "on_chain")]
boundary_test!(test_on_chain_integration, async {
    // Initialize boundary system
    let boundary_system = BoundarySystem::new();
    
    // Create and register test contract
    let test_contract = Arc::new(TestContract::new("0x1234567890abcdef"));
    
    let on_chain_adapter = boundary_system
        .on_chain_adapter(OnChainEnvironment::EVM)
        .expect("EVM adapter should exist");
    
    on_chain_adapter.register_contract(test_contract.clone());
    
    // Test reading value
    let mut args = HashMap::new();
    args.insert("key".to_string(), b"test_key".to_vec());
    
    let result = on_chain_adapter.call_contract_method(
        test_contract.address(),
        "getValue",
        args,
    ).await.expect("Call should succeed");
    
    assert_eq!(result.success, true);
    assert_eq!(result.data.len(), 4); // i32 is 4 bytes
    
    // Test writing value
    let mut args = HashMap::new();
    args.insert("key".to_string(), b"test_key".to_vec());
    args.insert("value".to_string(), 123i32.to_be_bytes().to_vec());
    
    let result = on_chain_adapter.submit_contract_transaction(
        test_contract.address(),
        "setValue",
        args,
        BoundaryAuthentication::Capability("test_capability".to_string()),
    ).await.expect("Transaction should succeed");
    
    assert_eq!(result.success, true);
    assert!(result.tx_id.is_some());
    
    // Verify the value was set
    let mut args = HashMap::new();
    args.insert("key".to_string(), b"test_key".to_vec());
    
    let result = on_chain_adapter.call_contract_method(
        test_contract.address(),
        "getValue",
        args,
    ).await.expect("Call should succeed");
    
    assert_eq!(result.success, true);
    
    let value_bytes = result.data;
    let mut bytes = [0; 4];
    bytes.copy_from_slice(&value_bytes);
    let value = i32::from_be_bytes(bytes);
    
    assert_eq!(value, 123);
});

// Test for off-chain component integration
#[cfg(feature = "off_chain")]
boundary_test!(test_off_chain_integration, async {
    // Initialize boundary system
    let boundary_system = BoundarySystem::new();
    
    // Create and register test service
    let test_service = Arc::new(TestService::new("test-api", "1.0"));
    
    boundary_system.register_off_chain_component(test_service.clone());
    
    // Initialize the boundary system
    boundary_system.initialize().await.expect("Initialization should succeed");
    
    // Get the off-chain registry
    let off_chain_registry = boundary_system.off_chain_registry();
    
    // Get the adapter
    let adapter = off_chain_registry.adapter();
    
    // Create test data
    let test_data = TestData {
        id: "test2".to_string(),
        value: 99,
        metadata: {
            let mut map = HashMap::new();
            map.insert("test".to_string(), "data".to_string());
            map
        },
    };
    
    // Save data
    let mut params = HashMap::new();
    params.insert("data".to_string(), serde_json::to_vec(&test_data).unwrap());
    
    let result = adapter.execute_operation(
        test_service.id().clone(),
        "saveData",
        params,
        Some(BoundaryAuthentication::Capability("api_capability".to_string())),
    ).await.expect("Operation should succeed");
    
    assert_eq!(result.success, true);
    
    // Get data
    let mut params = HashMap::new();
    params.insert("id".to_string(), b"test2".to_vec());
    
    let result = adapter.execute_operation(
        test_service.id().clone(),
        "getData",
        params,
        None,
    ).await.expect("Operation should succeed");
    
    assert_eq!(result.success, true);
    
    // Deserialize result
    let retrieved_data: TestData = serde_json::from_slice(&result.data)
        .expect("Should deserialize data");
    
    // Verify data
    assert_eq!(retrieved_data, test_data);
});

// Test for boundary authentication failures
boundary_test!(test_authentication_failures, async {
    // Initialize boundary system with custom config
    let config = BoundarySystemConfig {
        // Ensure rate limiting is enabled for this test
        enable_rate_limiting: true,
        enable_size_limiting: true,
        max_payload_size: 1024,
        enable_metrics: true,
        default_auth_method: "capability".to_string(),
        #[cfg(feature = "on_chain")]
        supported_on_chain_environments: vec![OnChainEnvironment::EVM],
    };
    
    let boundary_system = BoundarySystem::with_config(config);
    
    // Get crossing registry
    let registry = boundary_system.crossing_registry();
    
    // Find outside to inside protocol
    let protocol = registry.find_protocol_for_boundaries(
        BoundaryType::OutsideSystem,
        BoundaryType::InsideSystem,
    ).expect("Protocol should exist");
    
    // Create test data
    let test_data = TestData {
        id: "test3".to_string(),
        value: 42,
        metadata: HashMap::new(),
    };
    
    // Try with no authentication (should fail for outside to inside)
    let payload_result = protocol.prepare_outgoing(
        &test_data,
        BoundaryAuthentication::None,
    );
    
    assert!(payload_result.is_ok(), "Payload preparation should succeed");
    
    let payload = payload_result.unwrap();
    
    let crossing_result = registry.process_crossing(
        protocol.name(),
        payload,
    );
    
    // Should fail due to authentication
    assert!(crossing_result.is_err(), "Crossing should fail due to missing authentication");
    
    if let Err(BoundaryCrossingError::AuthenticationFailed(_)) = crossing_result {
        // Expected error
    } else {
        panic!("Expected AuthenticationFailed error");
    }
    
    // Try with valid authentication
    let payload = protocol.prepare_outgoing(
        &test_data,
        BoundaryAuthentication::Capability("valid_capability".to_string()),
    ).expect("Should prepare payload");
    
    let crossing_result = registry.process_crossing(
        protocol.name(),
        payload,
    );
    
    // Should succeed
    assert!(crossing_result.is_ok(), "Crossing should succeed with valid authentication");
});

// Test for size limits
boundary_test!(test_size_limits, async {
    // Initialize boundary system with small size limit
    let config = BoundarySystemConfig {
        enable_rate_limiting: true,
        enable_size_limiting: true,
        max_payload_size: 10, // Very small limit
        enable_metrics: true,
        default_auth_method: "capability".to_string(),
        #[cfg(feature = "on_chain")]
        supported_on_chain_environments: vec![OnChainEnvironment::EVM],
    };
    
    let boundary_system = BoundarySystem::with_config(config);
    
    // Get crossing registry
    let registry = boundary_system.crossing_registry();
    
    // Find inside to outside protocol
    let protocol = registry.find_protocol_for_boundaries(
        BoundaryType::InsideSystem,
        BoundaryType::OutsideSystem,
    ).expect("Protocol should exist");
    
    // Create large test data
    let test_data = TestData {
        id: "test4".to_string(),
        value: 42,
        metadata: {
            let mut map = HashMap::new();
            // Add enough data to exceed the size limit
            map.insert("large_key".to_string(), "a".repeat(100));
            map
        },
    };
    
    // Try to prepare data for crossing
    let payload_result = protocol.prepare_outgoing(
        &test_data,
        BoundaryAuthentication::None,
    );
    
    // Should fail due to size limit
    assert!(payload_result.is_err(), "Payload preparation should fail due to size limit");
    
    if let Err(BoundaryCrossingError::SizeLimitExceeded) = payload_result {
        // Expected error
    } else {
        panic!("Expected SizeLimitExceeded error");
    }
});

// Test for metrics collection
boundary_test!(test_metrics_collection, async {
    // Create fresh boundary system
    let boundary_system = BoundarySystem::new();
    
    // Reset metrics to start fresh
    boundary_system.reset_metrics();
    
    // Perform a few boundary crossings
    let registry = boundary_system.crossing_registry();
    let protocol = registry.find_protocol_for_boundaries(
        BoundaryType::InsideSystem,
        BoundaryType::OutsideSystem,
    ).expect("Protocol should exist");
    
    for i in 0..5 {
        let test_data = TestData {
            id: format!("metrics_test_{}", i),
            value: i,
            metadata: HashMap::new(),
        };
        
        let payload = protocol.prepare_outgoing(
            &test_data,
            BoundaryAuthentication::None,
        ).expect("Should prepare payload");
        
        registry.process_crossing(
            protocol.name(),
            payload,
        ).expect("Should process crossing");
    }
    
    // Export metrics
    let metrics_json = boundary_system.export_metrics();
    
    // Parse metrics
    let metrics: serde_json::Value = serde_json::from_str(&metrics_json)
        .expect("Should parse metrics JSON");
    
    // Check if we have the expected metrics
    let boundary_crossings = &metrics["boundary_crossings"];
    assert!(boundary_crossings.is_object(), "Should have boundary_crossings metric");
    
    let inside_to_outside = &boundary_crossings["inside_to_outside"];
    assert!(inside_to_outside.is_object(), "Should have inside_to_outside metric");
    
    let count = inside_to_outside["count"].as_u64().unwrap_or(0);
    assert_eq!(count, 5, "Should have 5 boundary crossings");
});

// Integration test framework setup function
async fn setup_integration_test_environment() -> Arc<BoundarySystem> {
    // Initialize boundary system
    let boundary_system = BoundarySystem::new();
    
    // Register test components
    #[cfg(feature = "on_chain")]
    {
        let test_contract = Arc::new(TestContract::new("0x1234567890abcdef"));
        let on_chain_adapter = boundary_system
            .on_chain_adapter(OnChainEnvironment::EVM)
            .expect("EVM adapter should exist");
        on_chain_adapter.register_contract(test_contract);
    }
    
    #[cfg(feature = "off_chain")]
    {
        let test_service = Arc::new(TestService::new("test-api", "1.0"));
        boundary_system.register_off_chain_component(test_service);
    }
    
    // Initialize
    boundary_system.initialize().await.expect("Initialization should succeed");
    
    Arc::new(boundary_system)
} 