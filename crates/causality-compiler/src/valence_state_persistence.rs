// ------------ VALENCE STATE PERSISTENCE ------------
// Purpose: State persistence system for Valence accounts with real storage integration

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::storage_backend::StorageBackendManager;

// Real Almanac types when feature is enabled
#[cfg(feature = "almanac")]
use indexer_storage::{ValenceAccountInfo, ValenceAccountState, ValenceAccountLibrary, Storage};

/// Valence state persistence manager
pub struct ValenceStatePersistence {
    #[allow(dead_code)]
    storage_backend: Arc<StorageBackendManager>,
}

impl ValenceStatePersistence {
    /// Create a new state persistence manager
    pub fn new(storage_backend: Arc<StorageBackendManager>) -> Self {
        Self { storage_backend }
    }

    /// Store Valence account information
    pub async fn store_account(&self, account: CausalityValenceAccount) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let almanac_account = self.convert_to_almanac_account(account)?;
                storage.store_valence_account_instantiation(almanac_account).await?;
            } else {
                return Err(anyhow!("Storage backend not initialized"));
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            log::info!("Mock: Storing Valence account {} on chain {}", account.id, account.chain_id);
        }

        Ok(())
    }

    /// Store Valence account state
    pub async fn store_account_state(&self, account_id: &str, _state: CausalityValenceState) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let almanac_state = self.convert_to_almanac_state(_state)?;
                storage.store_valence_account_state(account_id, almanac_state).await?;
            } else {
                return Err(anyhow!("Storage backend not initialized"));
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            log::info!("Mock: Storing state for Valence account {}", account_id);
        }

        Ok(())
    }

    /// Store library approval
    pub async fn store_library_approval(&self, approval: CausalityLibraryApproval) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let almanac_library = self.convert_to_almanac_library(approval)?;
                storage.store_valence_account_library(almanac_library).await?;
            } else {
                return Err(anyhow!("Storage backend not initialized"));
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            log::info!("Mock: Storing library approval for account {} library {}", approval.account_id, approval.library_address);
        }

        Ok(())
    }

    /// Get Valence account information
    pub async fn get_account(&self, account_id: &str) -> Result<Option<CausalityValenceAccount>> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                if let Some(account) = storage.get_valence_account_info(account_id).await? {
                    Ok(Some(self.convert_from_almanac_account(account)?))
                } else {
                    Ok(None)
                }
            } else {
                Err(anyhow!("Storage backend not initialized"))
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            Ok(Some(CausalityValenceAccount::mock(account_id)))
        }
    }

    /// Get Valence account state
    pub async fn get_account_state(&self, account_id: &str) -> Result<Option<CausalityValenceState>> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                if let Some(state) = storage.get_valence_account_state(account_id).await? {
                    Ok(Some(self.convert_from_almanac_state(state)?))
                } else {
                    Ok(None)
                }
            } else {
                Err(anyhow!("Storage backend not initialized"))
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            Ok(Some(CausalityValenceState::mock(account_id)))
        }
    }

    /// Get approved libraries for an account
    pub async fn get_account_libraries(&self, account_id: &str) -> Result<Vec<CausalityLibraryApproval>> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let libraries = storage.get_valence_account_libraries(account_id).await?;
                let causality_libraries: Result<Vec<_>> = libraries.into_iter()
                    .map(|lib| self.convert_from_almanac_library(lib))
                    .collect();
                causality_libraries
            } else {
                Err(anyhow!("Storage backend not initialized"))
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            Ok(vec![CausalityLibraryApproval::mock(account_id)])
        }
    }

    /// Get accounts by owner
    pub async fn get_accounts_by_owner(&self, owner_address: &str) -> Result<Vec<CausalityValenceAccount>> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let accounts = storage.get_valence_accounts_by_owner(owner_address).await?;
                let causality_accounts: Result<Vec<_>> = accounts.into_iter()
                    .map(|acc| self.convert_from_almanac_account(acc))
                    .collect();
                causality_accounts
            } else {
                Err(anyhow!("Storage backend not initialized"))
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            Ok(vec![CausalityValenceAccount::mock_for_owner(owner_address)])
        }
    }

    /// Update account state
    pub async fn update_account_state(&self, account_id: &str, _updates: StateUpdate) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                // Get current state
                if let Some(mut current_state) = storage.get_valence_account_state(account_id).await? {
                    // Apply updates
                    if let Some(new_owner) = _updates.new_owner {
                        current_state.current_owner = Some(new_owner);
                    }
                    if let Some(pending_owner) = _updates.pending_owner {
                        current_state.pending_owner = Some(pending_owner);
                    }
                    if let Some(expiry) = _updates.pending_owner_expiry {
                        current_state.pending_owner_expiry = Some(expiry);
                    }
                    if let Some(block) = _updates.last_updated_block {
                        current_state.last_updated_block = block;
                    }
                    if let Some(tx) = _updates.last_updated_tx {
                        current_state.last_updated_tx = tx;
                    }

                    // Store updated state
                    storage.store_valence_account_state(account_id, current_state).await?;
                } else {
                    return Err(anyhow!("Account state not found: {}", account_id));
                }
            } else {
                return Err(anyhow!("Storage backend not initialized"));
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            log::info!("Mock: Updating state for account {}", account_id);
        }

        Ok(())
    }

    /// Get account statistics
    pub async fn get_account_statistics(&self) -> Result<AccountStatistics> {
        #[cfg(feature = "almanac")]
        {
            if let Some(storage) = self.storage_backend.storage() {
                let stats = storage.get_valence_account_statistics().await?;
                Ok(AccountStatistics {
                    total_accounts: stats.total_accounts,
                    active_accounts: stats.active_accounts,
                    total_libraries: stats.total_libraries,
                    total_transactions: stats.total_transactions,
                    last_activity_block: stats.last_activity_block,
                })
            } else {
                Err(anyhow!("Storage backend not initialized"))
            }
        }

        #[cfg(not(feature = "almanac"))]
        {
            // Mock implementation for development
            Ok(AccountStatistics::mock())
        }
    }

    /// Convert Causality account to Almanac format
    #[cfg(feature = "almanac")]
    fn convert_to_almanac_account(&self, account: CausalityValenceAccount) -> Result<ValenceAccountInfo> {
        Ok(ValenceAccountInfo {
            id: account.id,
            chain_id: account.chain_id,
            contract_address: account.contract_address,
            created_at_block: account.created_at_block,
            created_at_tx: account.created_at_tx,
            current_owner: account.current_owner,
            pending_owner: account.pending_owner,
            pending_owner_expiry: account.pending_owner_expiry,
            last_updated_block: account.last_updated_block,
            last_updated_tx: account.last_updated_tx,
        })
    }

    /// Convert Almanac account to Causality format
    #[cfg(feature = "almanac")]
    fn convert_from_almanac_account(&self, account: ValenceAccountInfo) -> Result<CausalityValenceAccount> {
        Ok(CausalityValenceAccount {
            id: account.id,
            chain_id: account.chain_id,
            contract_address: account.contract_address,
            created_at_block: account.created_at_block,
            created_at_tx: account.created_at_tx,
            current_owner: account.current_owner,
            pending_owner: account.pending_owner,
            pending_owner_expiry: account.pending_owner_expiry,
            last_updated_block: account.last_updated_block,
            last_updated_tx: account.last_updated_tx,
        })
    }

    /// Convert Causality state to Almanac format
    #[cfg(feature = "almanac")]
    fn convert_to_almanac_state(&self, state: CausalityValenceState) -> Result<ValenceAccountState> {
        Ok(ValenceAccountState {
            account_id: state.account_id,
            current_owner: state.current_owner,
            pending_owner: state.pending_owner,
            pending_owner_expiry: state.pending_owner_expiry,
            last_updated_block: state.last_updated_block,
            last_updated_tx: state.last_updated_tx,
            state_data: state.state_data,
        })
    }

    /// Convert Almanac state to Causality format
    #[cfg(feature = "almanac")]
    fn convert_from_almanac_state(&self, state: ValenceAccountState) -> Result<CausalityValenceState> {
        Ok(CausalityValenceState {
            account_id: state.account_id,
            current_owner: state.current_owner,
            pending_owner: state.pending_owner,
            pending_owner_expiry: state.pending_owner_expiry,
            last_updated_block: state.last_updated_block,
            last_updated_tx: state.last_updated_tx,
            state_data: state.state_data,
        })
    }

    /// Convert Causality library to Almanac format
    #[cfg(feature = "almanac")]
    fn convert_to_almanac_library(&self, library: CausalityLibraryApproval) -> Result<ValenceAccountLibrary> {
        Ok(ValenceAccountLibrary {
            account_id: library.account_id,
            library_address: library.library_address,
            approved_at_block: library.approved_at_block,
            approved_at_tx: library.approved_at_tx,
        })
    }

    /// Convert Almanac library to Causality format
    #[cfg(feature = "almanac")]
    fn convert_from_almanac_library(&self, library: ValenceAccountLibrary) -> Result<CausalityLibraryApproval> {
        Ok(CausalityLibraryApproval {
            account_id: library.account_id,
            library_address: library.library_address,
            approved_at_block: library.approved_at_block,
            approved_at_tx: library.approved_at_tx,
        })
    }
}

/// Causality Valence account structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityValenceAccount {
    pub id: String,
    pub chain_id: String,
    pub contract_address: String,
    pub created_at_block: u64,
    pub created_at_tx: String,
    pub current_owner: Option<String>,
    pub pending_owner: Option<String>,
    pub pending_owner_expiry: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated_block: u64,
    pub last_updated_tx: String,
}

impl CausalityValenceAccount {
    /// Create a mock account for development
    #[cfg(not(feature = "almanac"))]
    pub fn mock(account_id: &str) -> Self {
        Self {
            id: account_id.to_string(),
            chain_id: "1".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            created_at_block: 12345,
            created_at_tx: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            current_owner: Some("0xa1b2c3d4e5f6789012345678901234567890abcd".to_string()),
            pending_owner: None,
            pending_owner_expiry: None,
            last_updated_block: 12345,
            last_updated_tx: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        }
    }

    /// Create a mock account for a specific owner
    #[cfg(not(feature = "almanac"))]
    pub fn mock_for_owner(owner_address: &str) -> Self {
        let mut account = Self::mock("mock_account_1");
        account.current_owner = Some(owner_address.to_string());
        account
    }
}

/// Causality Valence account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityValenceState {
    pub account_id: String,
    pub current_owner: Option<String>,
    pub pending_owner: Option<String>,
    pub pending_owner_expiry: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated_block: u64,
    pub last_updated_tx: String,
    pub state_data: BTreeMap<String, serde_json::Value>,
}

impl CausalityValenceState {
    /// Create a mock state for development
    #[cfg(not(feature = "almanac"))]
    pub fn mock(account_id: &str) -> Self {
        let mut state_data = BTreeMap::new();
        state_data.insert("balance".to_string(), serde_json::json!("1000000000000000000"));
        state_data.insert("nonce".to_string(), serde_json::json!(1));

        Self {
            account_id: account_id.to_string(),
            current_owner: Some("0xa1b2c3d4e5f6789012345678901234567890abcd".to_string()),
            pending_owner: None,
            pending_owner_expiry: None,
            last_updated_block: 12345,
            last_updated_tx: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            state_data,
        }
    }
}

/// Causality library approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityLibraryApproval {
    pub account_id: String,
    pub library_address: String,
    pub approved_at_block: u64,
    pub approved_at_tx: String,
}

impl CausalityLibraryApproval {
    /// Create a mock library approval for development
    #[cfg(not(feature = "almanac"))]
    pub fn mock(account_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            library_address: "0xb2c3d4e5f6789012345678901234567890abcdef".to_string(),
            approved_at_block: 12346,
            approved_at_tx: "0xbcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890a".to_string(),
        }
    }
}

/// State update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub new_owner: Option<String>,
    pub pending_owner: Option<String>,
    pub pending_owner_expiry: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated_block: Option<u64>,
    pub last_updated_tx: Option<String>,
}

/// Account statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatistics {
    pub total_accounts: u64,
    pub active_accounts: u64,
    pub total_libraries: u64,
    pub total_transactions: u64,
    pub last_activity_block: u64,
}

impl AccountStatistics {
    /// Create mock statistics for development
    #[cfg(not(feature = "almanac"))]
    pub fn mock() -> Self {
        Self {
            total_accounts: 100,
            active_accounts: 75,
            total_libraries: 25,
            total_transactions: 1000,
            last_activity_block: 12345,
        }
    }
} 