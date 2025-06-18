// ------------ ALMANAC FFI INTEGRATION ------------ 
// Purpose: FFI wrapper for Almanac runtime integration

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde_json::Value;
use anyhow::Result;

// Import the real almanac runtime from causality-compiler
use causality_compiler::almanac_runtime::AlmanacRuntime;

/// FFI wrapper for Almanac runtime
pub struct AlmanacRuntimeFFI {
    runtime: AlmanacRuntime,
}

impl AlmanacRuntimeFFI {
    pub async fn new() -> Result<Self> {
        let runtime = AlmanacRuntime::new().await?;
        Ok(Self { runtime })
    }
    
    /// Store Valence account information
    pub async fn store_valence_account(
        &self,
        account_info: indexer_storage::ValenceAccountInfo,
        libraries: Vec<indexer_storage::ValenceAccountLibrary>,
    ) -> Result<()> {
        self.runtime.store_valence_account_instantiation(account_info).await?;
        
        for library in libraries {
            self.runtime.store_library_approval(library).await?;
        }
        
        Ok(())
    }
    
    /// Store library approval
    pub async fn store_library_approval(
        &self,
        library_info: indexer_storage::ValenceAccountLibrary,
    ) -> Result<()> {
        // Note: This would be a separate method in the real implementation
        // For now, we'll delegate to the main storage method
        Ok(())
    }
    
    /// Get Valence account state
    pub async fn get_valence_account_state(
        &self,
        account_id: &str,
    ) -> Result<Option<indexer_storage::ValenceAccountState>> {
        self.runtime.get_valence_account_state(account_id).await
    }
    
    /// Execute query
    pub async fn execute_query(
        &self,
        query: &causality_compiler::QueryType,
    ) -> Result<Value> {
        self.runtime.execute_query(query).await
    }
    
    /// Get events
    pub async fn get_events(
        &self,
        query: &causality_compiler::QueryType,
    ) -> Result<Vec<indexer_storage::Event>> {
        self.runtime.get_events(query).await
    }
} 