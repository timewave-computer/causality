// On-chain boundary integration
// Original file: src/boundary/on_chain.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::annotation::{BoundaryType, CrossingType, BoundarySafe};
use super::crossing::{
    BoundaryCrossingProtocol, 
    BoundaryCrossingPayload, 
    BoundaryAuthentication,
    BoundaryCrossingError,
    BoundaryCrossingResult,
};

/// Types of supported on-chain environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnChainEnvironment {
    /// Ethereum Virtual Machine
    EVM,
    /// CosmWasm (Cosmos SDK)
    CosmWasm,
    /// Custom chain environment
    Custom(u32),
}

impl From<OnChainEnvironment> for BoundaryType {
    fn from(env: OnChainEnvironment) -> Self {
        match env {
            OnChainEnvironment::EVM => BoundaryType::EVM,
            OnChainEnvironment::SVM => BoundaryType::SVM,
            OnChainEnvironment::MoveVM => BoundaryType::MoveVM,
            OnChainEnvironment::CosmWasm => BoundaryType::CosmWasm,
            OnChainEnvironment::Custom(id) => BoundaryType::Custom(format!("chain_{}", id)),
        }
    }
}

/// Contract address for different chain environments
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainAddress {
    /// Ethereum address (hex string)
    Ethereum(String),
    /// Solana public key (base58 string)
    Solana(String),
    /// Move address
    Move(String),
    /// CosmWasm address (bech32 string)
    CosmWasm(String),
    /// Custom address format
    Custom(String),
}

impl ChainAddress {
    /// Get the chain environment for this address
    pub fn environment(&self) -> OnChainEnvironment {
        match self {
            ChainAddress::Ethereum(_) => OnChainEnvironment::EVM,
            ChainAddress::CosmWasm(_) => OnChainEnvironment::CosmWasm,
            ChainAddress::Custom(_) => OnChainEnvironment::Custom(0),
        }
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            ChainAddress::Ethereum(addr) => format!("eth:{}", addr),
            ChainAddress::CosmWasm(addr) => format!("cosm:{}", addr),
            ChainAddress::Custom(addr) => format!("custom:{}", addr),
        }
    }
}

/// Contract interface for on-chain components
#[async_trait]
pub trait ContractInterface: Send + Sync + 'static {
    /// Get the chain environment
    fn environment(&self) -> OnChainEnvironment;
    
    /// Get the contract address
    fn address(&self) -> ChainAddress;
    
    /// Call a contract method
    async fn call_method(
        &self,
        method: &str,
        args: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, String>;
    
    /// Submit a transaction to the contract
    async fn submit_transaction(
        &self,
        method: &str,
        args: HashMap<String, Vec<u8>>,
        auth: BoundaryAuthentication,
    ) -> Result<String, String>;
    
    /// Get the contract ABI/interface definition
    fn get_interface(&self) -> String;
}

/// Contract call data for crossing boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallData {
    /// Contract environment
    pub environment: OnChainEnvironment,
    /// Contract address
    pub address: ChainAddress,
    /// Method to call
    pub method: String,
    /// Arguments
    pub args: HashMap<String, Vec<u8>>,
    /// Authentication information
    pub auth: Option<BoundaryAuthentication>,
    /// Call context
    pub context: HashMap<String, String>,
}

impl BoundarySafe for ContractCallData {
    fn target_boundary(&self) -> BoundaryType {
        self.environment.into()
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        // In a real implementation, use a proper serialization format
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize contract call data: {}", e))
    }
}

/// Result of a contract call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    /// Transaction hash or identifier
    pub tx_id: Option<String>,
    /// Result data
    pub data: Vec<u8>,
    /// Call status
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Result context
    pub context: HashMap<String, String>,
}

impl BoundarySafe for ContractCallResult {
    fn target_boundary(&self) -> BoundaryType {
        BoundaryType::OffChain
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize contract call result: {}", e))
    }
}

/// Protocol for on-chain contract calls
pub struct ContractCallProtocol {
    name: String,
    environment: OnChainEnvironment,
    contracts: Arc<RwLock<HashMap<ChainAddress, Arc<dyn ContractInterface>>>>,
}

impl ContractCallProtocol {
    /// Create a new contract call protocol
    pub fn new(name: &str, environment: OnChainEnvironment) -> Self {
        Self {
            name: name.to_string(),
            environment,
            contracts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a contract interface
    pub fn register_contract(&self, contract: Arc<dyn ContractInterface>) {
        let mut contracts = self.contracts.write().unwrap();
        contracts.insert(contract.address(), contract);
    }
    
    /// Get a contract interface by address
    pub fn get_contract(&self, address: &ChainAddress) -> Option<Arc<dyn ContractInterface>> {
        let contracts = self.contracts.read().unwrap();
        contracts.get(address).cloned()
    }
    
    /// Execute a contract call
    async fn execute_contract_call(
        &self,
        call_data: &ContractCallData,
    ) -> Result<ContractCallResult, String> {
        // Get the contract interface
        let contract = self.get_contract(&call_data.address)
            .ok_or_else(|| format!("Contract not found: {:?}", call_data.address))?;
        
        // Check if the environment matches
        if contract.environment() != call_data.environment {
            return Err(format!(
                "Environment mismatch: expected {:?}, got {:?}",
                contract.environment(),
                call_data.environment
            ));
        }
        
        // Execute the call
        let result = if call_data.auth.is_some() {
            // This is a transaction
            match contract.submit_transaction(
                &call_data.method,
                call_data.args.clone(),
                call_data.auth.clone().unwrap_or(BoundaryAuthentication::None),
            ).await {
                Ok(tx_id) => ContractCallResult {
                    tx_id: Some(tx_id),
                    data: Vec::new(),
                    success: true,
                    error: None,
                    context: HashMap::new(),
                },
                Err(e) => ContractCallResult {
                    tx_id: None,
                    data: Vec::new(),
                    success: false,
                    error: Some(e),
                    context: HashMap::new(),
                },
            }
        } else {
            // This is a read-only call
            match contract.call_method(&call_data.method, call_data.args.clone()).await {
                Ok(data) => ContractCallResult {
                    tx_id: None,
                    data,
                    success: true,
                    error: None,
                    context: HashMap::new(),
                },
                Err(e) => ContractCallResult {
                    tx_id: None,
                    data: Vec::new(),
                    success: false,
                    error: Some(e),
                    context: HashMap::new(),
                },
            }
        };
        
        Ok(result)
    }
}

#[async_trait]
impl BoundaryCrossingProtocol for ContractCallProtocol {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn source_boundary(&self) -> BoundaryType {
        BoundaryType::OffChain
    }
    
    fn destination_boundary(&self) -> BoundaryType {
        self.environment.into()
    }
    
    async fn verify_authentication(
        &self,
        payload: &BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<bool> {
        // Extract the call data
        let call_data: ContractCallData = serde_json::from_slice(&payload.data)
            .map_err(|e| BoundaryCrossingError::InvalidPayload(format!("Invalid call data: {}", e)))?;
        
        // For simplicity, assume authentication is valid
        // In a real implementation, verify signatures or capabilities
        Ok(true)
    }
    
    async fn process_incoming(
        &self,
        payload: BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<Vec<u8>> {
        // Extract the call data
        let call_data: ContractCallData = serde_json::from_slice(&payload.data)
            .map_err(|e| BoundaryCrossingError::InvalidPayload(format!("Invalid call data: {}", e)))?;
        
        // Execute the contract call
        let result = self.execute_contract_call(&call_data).await
            .map_err(|e| BoundaryCrossingError::ProtocolError(e))?;
        
        // Serialize the result
        let result_data = serde_json::to_vec(&result)
            .map_err(|e| BoundaryCrossingError::InternalError(format!("Failed to serialize result: {}", e)))?;
        
        Ok(result_data)
    }
    
    async fn prepare_outgoing<T: BoundarySafe>(
        &self,
        data: &T,
        auth: BoundaryAuthentication,
    ) -> BoundaryCrossingResult<BoundaryCrossingPayload> {
        // For outgoing calls, data should be ContractCallData
        let serialized_data = data.prepare_for_crossing();
        
        // Create a new payload
        let payload = BoundaryCrossingPayload::new(
            BoundaryType::OffChain,
            self.environment.into(),
            CrossingType::OffChainToOnChain,
            serialized_data,
            auth,
        );
        
        Ok(payload)
    }
}

/// Adapter for making contract calls across boundaries
pub struct ContractCallAdapter {
    protocol: Arc<ContractCallProtocol>,
}

impl ContractCallAdapter {
    /// Create a new contract call adapter
    pub fn new(protocol: Arc<ContractCallProtocol>) -> Self {
        Self { protocol }
    }
    
    /// Call a contract method
    pub async fn call_contract_method(
        &self,
        address: ChainAddress,
        method: &str,
        args: HashMap<String, Vec<u8>>,
    ) -> Result<ContractCallResult, String> {
        // Create the call data
        let call_data = ContractCallData {
            environment: address.environment(),
            address,
            method: method.to_string(),
            args,
            auth: None,
            context: HashMap::new(),
        };
        
        // Prepare the outgoing payload
        let payload = self.protocol.prepare_outgoing(&call_data, BoundaryAuthentication::None).await
            .map_err(|e| format!("Failed to prepare outgoing call: {}", e))?;
        
        // Process the call
        let result_data = self.protocol.process_incoming(payload).await
            .map_err(|e| format!("Failed to process call: {}", e))?;
        
        // Deserialize the result
        let result: ContractCallResult = serde_json::from_slice(&result_data)
            .map_err(|e| format!("Failed to deserialize result: {}", e))?;
        
        Ok(result)
    }
    
    /// Submit a transaction to a contract
    pub async fn submit_contract_transaction(
        &self,
        address: ChainAddress,
        method: &str,
        args: HashMap<String, Vec<u8>>,
        auth: BoundaryAuthentication,
    ) -> Result<ContractCallResult, String> {
        // Create the call data
        let call_data = ContractCallData {
            environment: address.environment(),
            address,
            method: method.to_string(),
            args,
            auth: Some(auth.clone()),
            context: HashMap::new(),
        };
        
        // Prepare the outgoing payload
        let payload = self.protocol.prepare_outgoing(&call_data, auth).await
            .map_err(|e| format!("Failed to prepare outgoing transaction: {}", e))?;
        
        // Process the transaction
        let result_data = self.protocol.process_incoming(payload).await
            .map_err(|e| format!("Failed to process transaction: {}", e))?;
        
        // Deserialize the result
        let result: ContractCallResult = serde_json::from_slice(&result_data)
            .map_err(|e| format!("Failed to deserialize result: {}", e))?;
        
        Ok(result)
    }
} 