// ------------ VALENCE COPROCESSOR FFI IMPLEMENTATION ------------ 
// Purpose: Rust FFI implementation for OCaml bindings to Valence coprocessor APIs

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use anyhow::{Result, anyhow};

// Import real Valence types when available
#[cfg(feature = "valence")]
use valence_core::{AccountId, LibraryId, TransactionConfig};
#[cfg(feature = "valence")]
use valence_domain_clients::{DomainClient, ExecutionResult};
#[cfg(feature = "valence")]
use valence_coprocessor_client::{CoprocessorClient, AccountCreationRequest, LibraryApprovalRequest};

// Import real Almanac integration
use crate::almanac_ffi::AlmanacRuntimeFFI;

/// Global runtime for async operations
static mut TOKIO_RUNTIME: Option<Runtime> = None;
static mut VALENCE_CLIENT: Option<Arc<ValenceFFIClient>> = None;

/// Initialize the global runtime (called once)
fn ensure_runtime() -> &'static Runtime {
    unsafe {
        TOKIO_RUNTIME.get_or_insert_with(|| {
            Runtime::new().expect("Failed to create Tokio runtime")
        })
    }
}

/// Valence FFI client wrapper
#[derive(Debug)]
pub struct ValenceFFIClient {
    #[cfg(feature = "valence")]
    coprocessor_client: Arc<CoprocessorClient>,
    #[cfg(feature = "valence")]
    domain_clients: HashMap<String, Arc<dyn DomainClient + Send + Sync>>,
    almanac_runtime: AlmanacRuntimeFFI,
    config: ValenceFFIConfig,
}

#[derive(Debug, Clone)]
pub struct ValenceFFIConfig {
    pub coprocessor_endpoint: String,
    pub default_gas_limit: u64,
    pub default_gas_price: u64,
    pub transaction_timeout_seconds: u64,
    pub max_retry_attempts: u32,
}

impl Default for ValenceFFIConfig {
    fn default() -> Self {
        Self {
            coprocessor_endpoint: "http://localhost:8080".to_string(),
            default_gas_limit: 500_000,
            default_gas_price: 20_000_000_000, // 20 gwei
            transaction_timeout_seconds: 300,
            max_retry_attempts: 3,
        }
    }
}

impl ValenceFFIClient {
    #[cfg(feature = "valence")]
    pub async fn new(config: ValenceFFIConfig) -> Result<Self> {
        // Initialize real coprocessor client
        let coprocessor_client = Arc::new(
            CoprocessorClient::new(&config.coprocessor_endpoint)
                .await?
        );
        
        let almanac_runtime = AlmanacRuntimeFFI::new().await?;
        
        Ok(Self {
            coprocessor_client,
            domain_clients: HashMap::new(),
            almanac_runtime,
            config,
        })
    }
    
    #[cfg(not(feature = "valence"))]
    pub async fn new(config: ValenceFFIConfig) -> Result<Self> {
        let almanac_runtime = AlmanacRuntimeFFI::new().await?;
        
        Ok(Self {
            almanac_runtime,
            config,
        })
    }
    
    /// Create a Valence account factory
    pub async fn create_account_factory(
        &self,
        chain_id: &str,
        owner_address: &str,
        initial_libraries: &[String],
    ) -> Result<Value> {
        #[cfg(feature = "valence")]
        {
            let request = AccountCreationRequest {
                chain_id: chain_id.to_string(),
                owner_address: owner_address.to_string(),
                initial_libraries: initial_libraries.iter()
                    .map(|s| LibraryId::from_string(s))
                    .collect::<Result<Vec<_>, _>>()?,
                gas_limit: self.config.default_gas_limit,
                gas_price: self.config.default_gas_price,
            };
            
            let result = self.coprocessor_client.create_account(request).await?;
            
            // Store account info in Almanac
            let account_info = indexer_storage::ValenceAccountInfo {
                id: format!("{}:{}", chain_id, result.account_id),
                chain_id: chain_id.to_string(),
                contract_address: result.account_id.to_string(),
                created_at_block: result.block_number,
                created_at_tx: result.transaction_hash.clone(),
                current_owner: Some(owner_address.to_string()),
                pending_owner: None,
                pending_owner_expiry: None,
                last_updated_block: result.block_number,
                last_updated_tx: result.transaction_hash.clone(),
            };
            
            let libraries = initial_libraries.iter().map(|lib_id| {
                indexer_storage::ValenceAccountLibrary {
                    account_id: format!("{}:{}", chain_id, result.account_id),
                    library_address: lib_id.clone(),
                    approved_at_block: result.block_number,
                    approved_at_tx: result.transaction_hash.clone(),
                }
            }).collect();
            
            self.almanac_runtime.store_valence_account(account_info, libraries).await?;
            
            Ok(json!({
                "account_id": result.account_id.to_string(),
                "transaction_hash": result.transaction_hash,
                "block_number": result.block_number,
                "status": "Confirmed"
            }))
        }
        
        #[cfg(not(feature = "valence"))]
        {
            // Mock implementation for development
            let account_id = format!("account_{}_{}", chain_id, owner_address);
            let tx_hash = format!("0x{:x}", rand::random::<u64>());
            
            Ok(json!({
                "account_id": account_id,
                "transaction_hash": tx_hash,
                "block_number": 12345,
                "status": "Confirmed"
            }))
        }
    }
    
    /// Approve a library for a Valence account
    pub async fn approve_library(
        &self,
        chain_id: &str,
        account_id: &str,
        library_id: &str,
    ) -> Result<Value> {
        #[cfg(feature = "valence")]
        {
            let request = LibraryApprovalRequest {
                chain_id: chain_id.to_string(),
                account_id: AccountId::from_string(account_id)?,
                library_id: LibraryId::from_string(library_id)?,
                gas_limit: self.config.default_gas_limit,
                gas_price: self.config.default_gas_price,
            };
            
            let result = self.coprocessor_client.approve_library(request).await?;
            
            // Update Almanac with library approval
            let library_info = indexer_storage::ValenceAccountLibrary {
                account_id: format!("{}:{}", chain_id, account_id),
                library_address: library_id.to_string(),
                approved_at_block: result.block_number,
                approved_at_tx: result.transaction_hash.clone(),
            };
            
            self.almanac_runtime.store_library_approval(library_info).await?;
            
            Ok(json!({
                "account_id": account_id,
                "library_id": library_id,
                "transaction_hash": result.transaction_hash,
                "block_number": result.block_number,
                "status": "Confirmed"
            }))
        }
        
        #[cfg(not(feature = "valence"))]
        {
            // Mock implementation
            let tx_hash = format!("0x{:x}", rand::random::<u64>());
            
            Ok(json!({
                "account_id": account_id,
                "library_id": library_id,
                "transaction_hash": tx_hash,
                "block_number": 12346,
                "status": "Confirmed"
            }))
        }
    }
    
    /// Execute a transaction through a Valence account
    pub async fn execute_transaction(
        &self,
        chain_id: &str,
        account_id: &str,
        transaction_config: &str,
    ) -> Result<Value> {
        #[cfg(feature = "valence")]
        {
            let config: TransactionConfig = serde_json::from_str(transaction_config)?;
            let account = AccountId::from_string(account_id)?;
            
            if let Some(domain_client) = self.domain_clients.get(chain_id) {
                let result = domain_client.execute_transaction(&account, config).await?;
                
                Ok(json!({
                    "transaction_hash": result.transaction_hash,
                    "block_number": result.block_number,
                    "gas_used": result.gas_used,
                    "status": "Confirmed",
                    "logs": result.logs
                }))
            } else {
                Err(anyhow!("No domain client for chain: {}", chain_id))
            }
        }
        
        #[cfg(not(feature = "valence"))]
        {
            // Mock implementation
            let tx_hash = format!("0x{:x}", rand::random::<u64>());
            
            Ok(json!({
                "transaction_hash": tx_hash,
                "block_number": 12347,
                "gas_used": 21000,
                "status": "Confirmed",
                "logs": []
            }))
        }
    }
    
    /// Query account state from Almanac
    pub async fn query_account_state(
        &self,
        chain_id: &str,
        account_id: &str,
    ) -> Result<Value> {
        let full_account_id = format!("{}:{}", chain_id, account_id);
        
        if let Some(state) = self.almanac_runtime.get_valence_account_state(&full_account_id).await? {
            Ok(serde_json::to_value(state)?)
        } else {
            Ok(json!(null))
        }
    }
    
    /// Get account balance
    pub async fn get_account_balance(
        &self,
        chain_id: &str,
        account_id: &str,
        token_address: Option<&str>,
    ) -> Result<String> {
        #[cfg(feature = "valence")]
        {
            if let Some(domain_client) = self.domain_clients.get(chain_id) {
                let account = AccountId::from_string(account_id)?;
                let balance = domain_client.get_account_balance(&account, token_address).await?;
                Ok(balance)
            } else {
                Err(anyhow!("No domain client for chain: {}", chain_id))
            }
        }
        
        #[cfg(not(feature = "valence"))]
        {
            // Mock balance
            Ok("1000000000000000000".to_string()) // 1 ETH in wei
        }
    }
}

// C FFI functions that OCaml calls

#[no_mangle]
pub extern "C" fn caml_valence_create_account_factory(
    chain_id: *const c_char,
    owner_address: *const c_char,
    initial_libraries: *const *const c_char,
    library_count: c_int,
) -> *mut c_char {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let chain_id = unsafe { CStr::from_ptr(chain_id) }.to_string_lossy();
        let owner_address = unsafe { CStr::from_ptr(owner_address) }.to_string_lossy();
        
        let mut libraries = Vec::new();
        for i in 0..library_count {
            let lib_ptr = unsafe { *initial_libraries.offset(i as isize) };
            let lib_str = unsafe { CStr::from_ptr(lib_ptr) }.to_string_lossy();
            libraries.push(lib_str.to_string());
        }
        
        let client = unsafe { VALENCE_CLIENT.as_ref() }
            .ok_or_else(|| anyhow!("Valence client not initialized"))?;
            
        client.create_account_factory(&chain_id, &owner_address, &libraries).await
    });
    
    match result {
        Ok(value) => {
            let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
            CString::new(json_str).unwrap().into_raw()
        }
        Err(e) => {
            let error_json = json!({"error": e.to_string()});
            let json_str = serde_json::to_string(&error_json).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string());
            CString::new(json_str).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn caml_valence_approve_library(
    chain_id: *const c_char,
    account_id: *const c_char,
    library_id: *const c_char,
) -> *mut c_char {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let chain_id = unsafe { CStr::from_ptr(chain_id) }.to_string_lossy();
        let account_id = unsafe { CStr::from_ptr(account_id) }.to_string_lossy();
        let library_id = unsafe { CStr::from_ptr(library_id) }.to_string_lossy();
        
        let client = unsafe { VALENCE_CLIENT.as_ref() }
            .ok_or_else(|| anyhow!("Valence client not initialized"))?;
            
        client.approve_library(&chain_id, &account_id, &library_id).await
    });
    
    match result {
        Ok(value) => {
            let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
            CString::new(json_str).unwrap().into_raw()
        }
        Err(e) => {
            let error_json = json!({"error": e.to_string()});
            let json_str = serde_json::to_string(&error_json).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string());
            CString::new(json_str).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn caml_valence_execute_transaction(
    chain_id: *const c_char,
    account_id: *const c_char,
    transaction_config: *const c_char,
) -> *mut c_char {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let chain_id = unsafe { CStr::from_ptr(chain_id) }.to_string_lossy();
        let account_id = unsafe { CStr::from_ptr(account_id) }.to_string_lossy();
        let transaction_config = unsafe { CStr::from_ptr(transaction_config) }.to_string_lossy();
        
        let client = unsafe { VALENCE_CLIENT.as_ref() }
            .ok_or_else(|| anyhow!("Valence client not initialized"))?;
            
        client.execute_transaction(&chain_id, &account_id, &transaction_config).await
    });
    
    match result {
        Ok(value) => {
            let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
            CString::new(json_str).unwrap().into_raw()
        }
        Err(e) => {
            let error_json = json!({"error": e.to_string()});
            let json_str = serde_json::to_string(&error_json).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string());
            CString::new(json_str).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn caml_valence_query_account_state(
    chain_id: *const c_char,
    account_id: *const c_char,
) -> *mut c_char {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let chain_id = unsafe { CStr::from_ptr(chain_id) }.to_string_lossy();
        let account_id = unsafe { CStr::from_ptr(account_id) }.to_string_lossy();
        
        let client = unsafe { VALENCE_CLIENT.as_ref() }
            .ok_or_else(|| anyhow!("Valence client not initialized"))?;
            
        client.query_account_state(&chain_id, &account_id).await
    });
    
    match result {
        Ok(value) => {
            let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
            CString::new(json_str).unwrap().into_raw()
        }
        Err(e) => {
            let error_json = json!({"error": e.to_string()});
            let json_str = serde_json::to_string(&error_json).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string());
            CString::new(json_str).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn caml_valence_get_account_balance(
    chain_id: *const c_char,
    account_id: *const c_char,
    token_address: *const c_char, // Can be null
) -> *mut c_char {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let chain_id = unsafe { CStr::from_ptr(chain_id) }.to_string_lossy();
        let account_id = unsafe { CStr::from_ptr(account_id) }.to_string_lossy();
        let token_address = if token_address.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(token_address) }.to_string_lossy())
        };
        
        let client = unsafe { VALENCE_CLIENT.as_ref() }
            .ok_or_else(|| anyhow!("Valence client not initialized"))?;
            
        client.get_account_balance(&chain_id, &account_id, token_address.as_deref()).await
    });
    
    match result {
        Ok(balance) => CString::new(balance).unwrap().into_raw(),
        Err(e) => CString::new(format!("Error: {}", e)).unwrap().into_raw(),
    }
}

/// Initialize the Valence FFI client (called once from OCaml)
#[no_mangle]
pub extern "C" fn caml_valence_initialize() -> c_int {
    let rt = ensure_runtime();
    
    let result = rt.block_on(async {
        let config = ValenceFFIConfig::default();
        let client = Arc::new(ValenceFFIClient::new(config).await?);
        
        unsafe {
            VALENCE_CLIENT = Some(client);
        }
        
        Ok::<(), anyhow::Error>(())
    });
    
    match result {
        Ok(()) => 0, // Success
        Err(_) => -1, // Error
    }
}

/// Cleanup function
#[no_mangle]
pub extern "C" fn caml_valence_cleanup() {
    unsafe {
        VALENCE_CLIENT = None;
    }
} 