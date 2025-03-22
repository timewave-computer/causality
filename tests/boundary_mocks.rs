use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use causality::boundary::{
    annotation::{Boundary, BoundaryCrossing, BoundaryType},
    crossing::{AuthType, BoundarySafe, CrossingProtocol, VerificationResult},
    metrics::BoundaryMetrics,
    off_chain::{OffChainComponent, OffChainRequest, OffChainResponse, ServiceAvailability},
    on_chain::{ContractInterface, SubmitResult, TransactionReceipt, TransactionStatus},
    BoundarySystem,
};

/// Mock data structure that can safely cross boundaries
#[derive(Debug, Clone)]
pub struct MockData {
    pub id: u64,
    pub name: String,
    pub value: i32,
    pub metadata: Option<String>,
}

impl BoundarySafe for MockData {
    fn prepare_for_boundary(&self) -> Vec<u8> {
        // Simple serialization for testing
        format!("{}:{}:{}:{}", 
            self.id, 
            self.name, 
            self.value, 
            self.metadata.clone().unwrap_or_default()
        ).into_bytes()
    }

    fn from_boundary(data: Vec<u8>) -> Result<Self, String> {
        let s = String::from_utf8(data).map_err(|e| e.to_string())?;
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.len() < 4 {
            return Err("Invalid MockData format".to_string());
        }
        
        let id = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<i32>().map_err(|e| e.to_string())?;
        let metadata = if parts[3].is_empty() { None } else { Some(parts[3].to_string()) };
        
        Ok(MockData { id, name, value, metadata })
    }
}

/// Mock storage service that implements OffChainComponent
#[derive(Debug, Clone)]
pub struct MockStorageService {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    availability: ServiceAvailability,
    latency_ms: u64,
    error_rate: f64,
}

impl MockStorageService {
    pub fn new(latency_ms: u64, error_rate: f64) -> Self {
        MockStorageService {
            data: Arc::new(Mutex::new(HashMap::new())),
            availability: ServiceAvailability::Available,
            latency_ms,
            error_rate,
        }
    }
    
    pub fn set_availability(&mut self, availability: ServiceAvailability) {
        self.availability = availability;
    }
}

impl OffChainComponent for MockStorageService {
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn check_availability(&self) -> ServiceAvailability {
        self.availability
    }

    #[allow(unused_variables)]
    fn execute_request(&self, request: OffChainRequest) -> Result<OffChainResponse, String> {
        // Simulate latency
        if self.latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.latency_ms));
        }
        
        // Simulate random errors
        if rand::random::<f64>() < self.error_rate {
            return Err("Service error (simulated)".to_string());
        }
        
        match request {
            OffChainRequest::Custom(name, payload) => {
                match name.as_str() {
                    "store" => {
                        if payload.len() < 2 {
                            return Err("Invalid payload for store operation".to_string());
                        }
                        
                        let key_size = payload[0] as usize;
                        if key_size + 1 >= payload.len() {
                            return Err("Invalid key size".to_string());
                        }
                        
                        let key = String::from_utf8(payload[1..key_size + 1].to_vec())
                            .map_err(|e| e.to_string())?;
                        let value = payload[key_size + 1..].to_vec();
                        
                        let mut data = self.data.lock().unwrap();
                        data.insert(key, value);
                        
                        Ok(OffChainResponse::Success(vec![1])) // 1 = success code
                    },
                    "retrieve" => {
                        let key = String::from_utf8(payload).map_err(|e| e.to_string())?;
                        let data = self.data.lock().unwrap();
                        
                        match data.get(&key) {
                            Some(value) => Ok(OffChainResponse::Success(value.clone())),
                            None => Ok(OffChainResponse::Success(vec![0])), // 0 = not found
                        }
                    },
                    "delete" => {
                        let key = String::from_utf8(payload).map_err(|e| e.to_string())?;
                        let mut data = self.data.lock().unwrap();
                        
                        match data.remove(&key) {
                            Some(_) => Ok(OffChainResponse::Success(vec![1])), // 1 = success code
                            None => Ok(OffChainResponse::Success(vec![0])), // 0 = not found
                        }
                    },
                    _ => Err(format!("Unknown operation: {}", name)),
                }
            },
            _ => Err("Unsupported request type".to_string()),
        }
    }
}

/// Mock contract that implements ContractInterface
#[derive(Debug)]
pub struct MockContract {
    pub name: String,
    pub address: String,
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    next_tx_id: Arc<Mutex<u64>>,
    tx_history: Arc<Mutex<HashMap<String, TransactionStatus>>>,
    confirmation_time: u64,
    success_rate: f64,
}

impl MockContract {
    pub fn new(name: String, address: String, confirmation_time: u64, success_rate: f64) -> Self {
        MockContract {
            name,
            address,
            state: Arc::new(Mutex::new(HashMap::new())),
            next_tx_id: Arc::new(Mutex::new(1)),
            tx_history: Arc::new(Mutex::new(HashMap::new())),
            confirmation_time,
            success_rate,
        }
    }
    
    pub fn set_state(&self, key: &str, value: Vec<u8>) {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_string(), value);
    }
    
    pub fn get_state(&self, key: &str) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();
        state.get(key).cloned()
    }
    
    fn generate_tx_id(&self) -> String {
        let mut id = self.next_tx_id.lock().unwrap();
        let tx_id = format!("0x{:064x}", *id);
        *id += 1;
        tx_id
    }
}

impl ContractInterface for MockContract {
    fn contract_address(&self) -> &str {
        &self.address
    }

    fn contract_name(&self) -> &str {
        &self.name
    }

    #[allow(unused_variables)]
    fn call(&self, method: &str, params: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
        match method {
            "read" => {
                if params.is_empty() {
                    return Err("Missing key parameter".to_string());
                }
                let key = String::from_utf8(params[0].clone()).map_err(|e| e.to_string())?;
                let state = self.state.lock().unwrap();
                
                match state.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(vec![]),
                }
            },
            "balance" => {
                if params.is_empty() {
                    return Err("Missing address parameter".to_string());
                }
                let address = String::from_utf8(params[0].clone()).map_err(|e| e.to_string())?;
                // Mock balance calculation - just return the length of the address as bytes
                Ok(vec![address.len() as u8])
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    #[allow(unused_variables)]
    fn submit_transaction(&self, method: &str, params: Vec<Vec<u8>>, value: Option<u64>) -> Result<SubmitResult, String> {
        // Generate transaction ID
        let tx_id = self.generate_tx_id();
        
        // For write operations, we'll process them immediately for testing
        match method {
            "write" => {
                if params.len() < 2 {
                    return Err("Missing key or value parameters".to_string());
                }
                
                let key = String::from_utf8(params[0].clone()).map_err(|e| e.to_string())?;
                let value = params[1].clone();
                
                // Store in state based on success rate
                let success = rand::random::<f64>() < self.success_rate;
                
                let mut tx_history = self.tx_history.lock().unwrap();
                if success {
                    let mut state = self.state.lock().unwrap();
                    state.insert(key, value);
                    tx_history.insert(tx_id.clone(), TransactionStatus::Pending);
                } else {
                    tx_history.insert(tx_id.clone(), TransactionStatus::Failed);
                }
            },
            "transfer" => {
                // Just record the transaction for now
                let success = rand::random::<f64>() < self.success_rate;
                let mut tx_history = self.tx_history.lock().unwrap();
                
                if success {
                    tx_history.insert(tx_id.clone(), TransactionStatus::Pending);
                } else {
                    tx_history.insert(tx_id.clone(), TransactionStatus::Failed);
                }
            },
            _ => return Err(format!("Unknown method: {}", method)),
        }
        
        Ok(SubmitResult {
            transaction_id: tx_id,
            estimated_confirmation_time: self.confirmation_time,
        })
    }

    fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, String> {
        let tx_history = self.tx_history.lock().unwrap();
        
        match tx_history.get(transaction_id) {
            Some(status) => {
                match status {
                    TransactionStatus::Pending => {
                        // Simulate confirmation after the specified time
                        // In a real implementation, this would check if enough time has passed
                        if rand::random::<bool>() {
                            Ok(TransactionStatus::Confirmed(TransactionReceipt {
                                block_number: 12345,
                                gas_used: 21000,
                                status: true,
                            }))
                        } else {
                            Ok(TransactionStatus::Pending)
                        }
                    },
                    _ => Ok(status.clone()),
                }
            },
            None => Err(format!("Transaction not found: {}", transaction_id)),
        }
    }
}

/// Creates a simulated authentication token for testing
pub fn create_mock_auth_token(user_id: &str, expiry_seconds: u64) -> Vec<u8> {
    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + expiry_seconds;
    
    format!("MOCK_AUTH:{}:{}", user_id, expiry).into_bytes()
}

/// Verifies a mock authentication token
pub fn verify_mock_auth_token(token: &[u8]) -> VerificationResult {
    let token_str = match std::str::from_utf8(token) {
        Ok(s) => s,
        Err(_) => return VerificationResult::Invalid("Invalid token format".to_string()),
    };
    
    let parts: Vec<&str> = token_str.split(':').collect();
    if parts.len() != 3 || parts[0] != "MOCK_AUTH" {
        return VerificationResult::Invalid("Invalid token structure".to_string());
    }
    
    let expiry = match parts[2].parse::<u64>() {
        Ok(e) => e,
        Err(_) => return VerificationResult::Invalid("Invalid expiry format".to_string()),
    };
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if expiry < now {
        return VerificationResult::Expired;
    }
    
    VerificationResult::Valid(parts[1].to_string())
}

/// Boundary crossing helper with mocked authentication
#[derive(Debug, Clone)]
pub struct MockBoundaryCrosser;

impl MockBoundaryCrosser {
    #[allow(dead_code)]
    #[Boundary(BoundaryType::System)]
    pub fn cross_system_boundary<T: BoundarySafe>(
        data: &T,
        auth_token: Option<Vec<u8>>,
        boundary_system: &BoundarySystem,
    ) -> Result<T, String> {
        let protocol = CrossingProtocol {
            from_boundary: BoundaryType::System,
            to_boundary: BoundaryType::External,
            auth_type: match auth_token {
                Some(_) => AuthType::Token,
                None => AuthType::None,
            },
        };
        
        boundary_system.cross_boundary(data, protocol, auth_token)
    }
    
    #[allow(dead_code)]
    #[Boundary(BoundaryType::OnChain)]
    pub fn cross_chain_boundary<T: BoundarySafe>(
        data: &T,
        auth_token: Option<Vec<u8>>,
        boundary_system: &BoundarySystem,
    ) -> Result<T, String> {
        let protocol = CrossingProtocol {
            from_boundary: BoundaryType::OnChain,
            to_boundary: BoundaryType::OffChain,
            auth_type: match auth_token {
                Some(_) => AuthType::Token,
                None => AuthType::None,
            },
        };
        
        boundary_system.cross_boundary(data, protocol, auth_token)
    }
}

/// Factory for creating mock boundary testing components
pub struct MockComponentFactory;

impl MockComponentFactory {
    pub fn create_storage_service(latency_ms: u64, error_rate: f64) -> MockStorageService {
        MockStorageService::new(latency_ms, error_rate)
    }
    
    pub fn create_mock_contract(name: &str, confirmation_time: u64, success_rate: f64) -> MockContract {
        // Generate pseudo-random contract address
        let address = format!("0x{:040x}", rand::random::<u64>());
        MockContract::new(name.to_string(), address, confirmation_time, success_rate)
    }
    
    pub fn create_boundary_system(collect_metrics: bool) -> BoundarySystem {
        let metrics = if collect_metrics {
            Some(Arc::new(Mutex::new(BoundaryMetrics::new())))
        } else {
            None
        };
        
        BoundarySystem::new(metrics)
    }
    
    pub fn create_test_data(id: u64, name: &str, value: i32) -> MockData {
        MockData {
            id,
            name: name.to_string(),
            value,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_data_boundary_safe() {
        let data = MockData {
            id: 42,
            name: "Test".to_string(),
            value: 100,
            metadata: Some("meta".to_string()),
        };
        
        let serialized = data.prepare_for_boundary();
        let deserialized = MockData::from_boundary(serialized).unwrap();
        
        assert_eq!(data.id, deserialized.id);
        assert_eq!(data.name, deserialized.name);
        assert_eq!(data.value, deserialized.value);
        assert_eq!(data.metadata, deserialized.metadata);
    }
    
    #[test]
    fn test_mock_storage_service() {
        let service = MockStorageService::new(0, 0.0);
        
        // Store data
        let store_request = OffChainRequest::Custom(
            "store".to_string(),
            {
                let key = "test-key";
                let value = vec![1, 2, 3, 4];
                let mut payload = vec![key.len() as u8];
                payload.extend_from_slice(key.as_bytes());
                payload.extend_from_slice(&value);
                payload
            }
        );
        
        let result = service.execute_request(store_request).unwrap();
        assert!(matches!(result, OffChainResponse::Success(_)));
        
        // Retrieve data
        let retrieve_request = OffChainRequest::Custom(
            "retrieve".to_string(),
            "test-key".as_bytes().to_vec(),
        );
        
        let result = service.execute_request(retrieve_request).unwrap();
        if let OffChainResponse::Success(data) = result {
            assert_eq!(data, vec![1, 2, 3, 4]);
        } else {
            panic!("Expected successful response");
        }
    }
    
    #[test]
    fn test_mock_contract() {
        let contract = MockContract::new(
            "TestContract".to_string(),
            "0x1234567890".to_string(),
            2,
            1.0,
        );
        
        // Set state directly
        contract.set_state("test-key", vec![5, 6, 7, 8]);
        
        // Read state through call
        let result = contract.call("read", vec!["test-key".as_bytes().to_vec()]).unwrap();
        assert_eq!(result, vec![5, 6, 7, 8]);
        
        // Write state through transaction
        let tx_result = contract.submit_transaction(
            "write",
            vec!["new-key".as_bytes().to_vec(), vec![9, 10, 11]],
            None,
        ).unwrap();
        
        assert!(!tx_result.transaction_id.is_empty());
        assert_eq!(tx_result.estimated_confirmation_time, 2);
        
        // Check transaction status
        let status = contract.get_transaction_status(&tx_result.transaction_id).unwrap();
        // Note: status might be Pending or Confirmed depending on the random behavior
        assert!(matches!(status, TransactionStatus::Pending) || 
                matches!(status, TransactionStatus::Confirmed(_)));
    }
    
    #[test]
    fn test_mock_auth_token() {
        let token = create_mock_auth_token("user123", 3600);
        let verification = verify_mock_auth_token(&token);
        
        assert!(matches!(verification, VerificationResult::Valid(_)));
        if let VerificationResult::Valid(user_id) = verification {
            assert_eq!(user_id, "user123");
        }
        
        // Test expired token
        let expired_token = "MOCK_AUTH:user123:1000".as_bytes().to_vec();
        let verification = verify_mock_auth_token(&expired_token);
        assert!(matches!(verification, VerificationResult::Expired));
        
        // Test invalid token
        let invalid_token = "INVALID".as_bytes().to_vec();
        let verification = verify_mock_auth_token(&invalid_token);
        assert!(matches!(verification, VerificationResult::Invalid(_)));
    }
} 