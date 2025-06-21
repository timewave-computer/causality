//! Fungible Token Primitive
//!
//! A comprehensive fungible token implementation supporting standard ERC-20-like operations
//! with additional features for cross-chain compatibility and ZK privacy.

use super::*;
use causality_core::system::content_addressing::ContentAddressable;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Fungible token primitive state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FungibleTokenState {
    /// Token metadata
    pub token_info: TokenInfo,
    
    /// Account balances (address -> balance)
    pub balances: BTreeMap<String, u128>,
    
    /// Token allowances (owner -> spender -> amount)
    pub allowances: BTreeMap<String, BTreeMap<String, u128>>,
    
    /// Total supply of tokens
    pub total_supply: u128,
    
    /// Whether token transfers are paused
    pub paused: bool,
    
    /// Minting permissions (address -> can_mint)
    pub minters: BTreeMap<String, bool>,
    
    /// Burning permissions (address -> can_burn)
    pub burners: BTreeMap<String, bool>,
    
    /// Token lockups (address -> locked_until_timestamp)
    pub lockups: BTreeMap<String, u64>,
}

/// Token metadata information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token name
    pub name: String,
    
    /// Token symbol
    pub symbol: String,
    
    /// Number of decimal places
    pub decimals: u8,
    
    /// Maximum total supply (None for unlimited)
    pub max_supply: Option<u128>,
    
    /// Token description
    pub description: Option<String>,
    
    /// Token logo URI
    pub logo_uri: Option<String>,
    
    /// Additional metadata
    pub metadata: BTreeMap<String, Value>,
}

/// Fungible token operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FungibleTokenOperation {
    /// Transfer tokens between accounts
    Transfer {
        from: String,
        to: String,
        amount: u128,
    },
    
    /// Transfer tokens from allowance
    TransferFrom {
        spender: String,
        from: String,
        to: String,
        amount: u128,
    },
    
    /// Approve token allowance
    Approve {
        owner: String,
        spender: String,
        amount: u128,
    },
    
    /// Mint new tokens
    Mint {
        to: String,
        amount: u128,
        minter: String,
    },
    
    /// Burn existing tokens
    Burn {
        from: String,
        amount: u128,
        burner: String,
    },
    
    /// Pause token transfers
    Pause {
        admin: String,
    },
    
    /// Unpause token transfers
    Unpause {
        admin: String,
    },
    
    /// Lock tokens for a specific duration
    Lock {
        owner: String,
        amount: u128,
        unlock_time: u64,
    },
    
    /// Unlock previously locked tokens
    Unlock {
        owner: String,
        amount: u128,
    },
    
    /// Update token metadata
    UpdateMetadata {
        admin: String,
        new_metadata: BTreeMap<String, Value>,
    },
    
    /// Grant minting permission
    GrantMinter {
        admin: String,
        minter: String,
    },
    
    /// Revoke minting permission
    RevokeMinter {
        admin: String,
        minter: String,
    },
    
    /// Grant burning permission
    GrantBurner {
        admin: String,
        burner: String,
    },
    
    /// Revoke burning permission
    RevokeBurner {
        admin: String,
        burner: String,
    },
}

/// Fungible token primitive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FungibleToken {
    /// Unique identifier for this token
    pub id: EntityId,
    
    /// Current state of the token
    pub state: FungibleTokenState,
    
    /// Configuration for this token
    pub config: PrimitiveConfig,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last updated timestamp
    pub updated_at: u64,
}

impl FungibleToken {
    /// Create a new fungible token
    pub fn new(
        token_info: TokenInfo,
        initial_supply: u128,
        initial_owner: String,
        config: PrimitiveConfig,
    ) -> Result<Self> {
        let id = EntityId::default();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        let mut balances = BTreeMap::new();
        if initial_supply > 0 {
            balances.insert(initial_owner.clone(), initial_supply);
        }
        
        let mut minters = BTreeMap::new();
        minters.insert(initial_owner, true);
        
        let state = FungibleTokenState {
            token_info,
            balances,
            allowances: BTreeMap::new(),
            total_supply: initial_supply,
            paused: false,
            minters,
            burners: BTreeMap::new(),
            lockups: BTreeMap::new(),
        };
        
        Ok(Self {
            id,
            state,
            config,
            created_at: timestamp,
            updated_at: timestamp,
        })
    }
    
    /// Get balance for an account
    pub fn balance_of(&self, account: &str) -> u128 {
        self.state.balances.get(account).copied().unwrap_or(0)
    }
    
    /// Get allowance between owner and spender
    pub fn allowance(&self, owner: &str, spender: &str) -> u128 {
        self.state.allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0)
    }
    
    /// Check if an account has locked tokens
    pub fn locked_balance(&self, account: &str) -> u128 {
        let current_time = chrono::Utc::now().timestamp() as u64;
        
        if let Some(&unlock_time) = self.state.lockups.get(account) {
            if current_time < unlock_time {
                return self.balance_of(account);
            }
        }
        
        0
    }
    
    /// Get available (unlocked) balance
    pub fn available_balance(&self, account: &str) -> u128 {
        self.balance_of(account).saturating_sub(self.locked_balance(account))
    }
    
    /// Check if account can mint tokens
    pub fn can_mint(&self, account: &str) -> bool {
        self.state.minters.get(account).copied().unwrap_or(false)
    }
    
    /// Check if account can burn tokens
    pub fn can_burn(&self, account: &str) -> bool {
        self.state.burners.get(account).copied().unwrap_or(false)
    }
    
    /// Validate a transfer operation
    fn validate_transfer(&self, from: &str, amount: u128) -> DeFiResult<()> {
        if self.state.paused {
            return Err(DeFiError::InvalidOperation {
                reason: "Token transfers are paused".to_string(),
            });
        }
        
        let available = self.available_balance(from);
        if available < amount {
            return Err(DeFiError::InsufficientBalance {
                available,
                required: amount,
            });
        }
        
        Ok(())
    }
    
    /// Execute a transfer operation
    fn execute_transfer(&mut self, from: &str, to: &str, amount: u128) -> DeFiResult<()> {
        self.validate_transfer(from, amount)?;
        
        // Update balances
        let from_balance = self.balance_of(from);
        let to_balance = self.balance_of(to);
        
        self.state.balances.insert(from.to_string(), from_balance - amount);
        self.state.balances.insert(to.to_string(), to_balance + amount);
        
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        
        Ok(())
    }
    
    /// Execute a mint operation
    fn execute_mint(&mut self, to: &str, amount: u128, minter: &str) -> DeFiResult<()> {
        if !self.can_mint(minter) {
            return Err(DeFiError::Unauthorized {
                user: minter.to_string(),
                operation: "mint".to_string(),
            });
        }
        
        // Check max supply constraint
        if let Some(max_supply) = self.state.token_info.max_supply {
            if self.state.total_supply + amount > max_supply {
                return Err(DeFiError::InvalidOperation {
                    reason: format!(
                        "Minting {} would exceed max supply of {}",
                        amount, max_supply
                    ),
                });
            }
        }
        
        // Update balances and total supply
        let to_balance = self.balance_of(to);
        self.state.balances.insert(to.to_string(), to_balance + amount);
        self.state.total_supply += amount;
        
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        
        Ok(())
    }
    
    /// Execute a burn operation
    fn execute_burn(&mut self, from: &str, amount: u128, burner: &str) -> DeFiResult<()> {
        if !self.can_burn(burner) && burner != from {
            return Err(DeFiError::Unauthorized {
                user: burner.to_string(),
                operation: "burn".to_string(),
            });
        }
        
        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(DeFiError::InsufficientBalance {
                available: from_balance,
                required: amount,
            });
        }
        
        // Update balances and total supply
        self.state.balances.insert(from.to_string(), from_balance - amount);
        self.state.total_supply -= amount;
        
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        
        Ok(())
    }
}

impl DeFiPrimitive for FungibleToken {
    type Asset = Asset;
    type Operation = FungibleTokenOperation;
    type State = FungibleTokenState;
    
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn state(&self) -> &Self::State {
        &self.state
    }
    
    fn apply_operation(&self, operation: Self::Operation) -> Result<Self::State> {
        let mut new_token = self.clone();
        new_token.validate_operation(&operation)?;
        
        match operation {
            FungibleTokenOperation::Transfer { from, to, amount } => {
                new_token.execute_transfer(&from, &to, amount)?;
            }
            
            FungibleTokenOperation::TransferFrom { spender, from, to, amount } => {
                let allowance = new_token.allowance(&from, &spender);
                if allowance < amount {
                    return Err(DeFiError::InsufficientBalance {
                        available: allowance,
                        required: amount,
                    }.into());
                }
                
                new_token.execute_transfer(&from, &to, amount)?;
                
                // Update allowance
                let new_allowance = allowance - amount;
                new_token.state.allowances
                    .entry(from)
                    .or_default()
                    .insert(spender, new_allowance);
            }
            
            FungibleTokenOperation::Approve { owner, spender, amount } => {
                new_token.state.allowances
                    .entry(owner)
                    .or_default()
                    .insert(spender, amount);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::Mint { to, amount, minter } => {
                new_token.execute_mint(&to, amount, &minter)?;
            }
            
            FungibleTokenOperation::Burn { from, amount, burner } => {
                new_token.execute_burn(&from, amount, &burner)?;
            }
            
            FungibleTokenOperation::Pause { admin } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "pause".to_string(),
                    }.into());
                }
                new_token.state.paused = true;
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::Unpause { admin } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "unpause".to_string(),
                    }.into());
                }
                new_token.state.paused = false;
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::Lock { owner, amount: _, unlock_time } => {
                new_token.state.lockups.insert(owner, unlock_time);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::Unlock { owner, amount: _ } => {
                new_token.state.lockups.remove(&owner);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::UpdateMetadata { admin, new_metadata } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "update_metadata".to_string(),
                    }.into());
                }
                new_token.state.token_info.metadata = new_metadata;
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::GrantMinter { admin, minter } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "grant_minter".to_string(),
                    }.into());
                }
                new_token.state.minters.insert(minter, true);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::RevokeMinter { admin, minter } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "revoke_minter".to_string(),
                    }.into());
                }
                new_token.state.minters.remove(&minter);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::GrantBurner { admin, burner } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "grant_burner".to_string(),
                    }.into());
                }
                new_token.state.burners.insert(burner, true);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
            
            FungibleTokenOperation::RevokeBurner { admin, burner } => {
                if !new_token.config.admins.contains(&admin) {
                    return Err(DeFiError::Unauthorized {
                        user: admin,
                        operation: "revoke_burner".to_string(),
                    }.into());
                }
                new_token.state.burners.remove(&burner);
                new_token.updated_at = chrono::Utc::now().timestamp() as u64;
            }
        }
        
        Ok(new_token.state)
    }
    
    fn validate_operation(&self, operation: &Self::Operation) -> Result<()> {
        match operation {
            FungibleTokenOperation::Transfer { from, amount, .. } => {
                self.validate_transfer(from, *amount)?;
            }
            
            FungibleTokenOperation::TransferFrom { from, amount, .. } => {
                self.validate_transfer(from, *amount)?;
            }
            
            _ => {
                // Other operations are validated in apply_operation
            }
        }
        
        Ok(())
    }
    
    fn primitive_type(&self) -> &'static str {
        "fungible_token"
    }
    
    fn metadata(&self) -> BTreeMap<String, Value> {
        let mut metadata = BTreeMap::new();
        metadata.insert("name".to_string(), Value::String(causality_core::system::Str { value: self.state.token_info.name.clone() }));
        metadata.insert("symbol".to_string(), Value::String(causality_core::system::Str { value: self.state.token_info.symbol.clone() }));
        metadata.insert("decimals".to_string(), Value::Int(self.state.token_info.decimals as u32));
        metadata.insert("total_supply".to_string(), Value::String(causality_core::system::Str { value: self.state.total_supply.to_string() }));
        metadata.insert("paused".to_string(), Value::Bool(self.state.paused));
        metadata.insert("created_at".to_string(), Value::Int(self.created_at as u32));
        metadata.insert("updated_at".to_string(), Value::Int(self.updated_at as u32));
        metadata
    }
}

impl ContentAddressable for FungibleToken {
    fn content_id(&self) -> causality_core::EntityId {
        // Create a deterministic ID based on the token's essential properties
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.state.token_info.name.as_bytes());
        data.extend_from_slice(self.state.token_info.symbol.as_bytes());
        data.push(self.state.token_info.decimals);
        data.extend_from_slice(&self.state.total_supply.to_le_bytes());
        data.push(if self.state.paused { 1 } else { 0 });
        
        // Hash balances for deterministic ordering
        let mut sorted_balances: Vec<_> = self.state.balances.iter().collect();
        sorted_balances.sort_by_key(|(addr, _)| *addr);
        for (addr, balance) in sorted_balances {
            data.extend_from_slice(addr.as_bytes());
            data.extend_from_slice(&balance.to_le_bytes());
        }
        
        causality_core::EntityId::from_content(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token() -> FungibleToken {
        let token_info = TokenInfo {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            max_supply: Some(1_000_000_000_000_000_000_000_000), // 1M tokens
            description: Some("A test token".to_string()),
            logo_uri: None,
            metadata: BTreeMap::new(),
        };
        
        let mut config = PrimitiveConfig::default();
        config.admins.push("admin".to_string());
        
        FungibleToken::new(
            token_info,
            1_000_000_000_000_000_000_000, // 1000 tokens
            "owner".to_string(),
            config,
        ).unwrap()
    }
    
    #[test]
    fn test_token_creation() {
        let token = create_test_token();
        
        assert_eq!(token.state.token_info.name, "Test Token");
        assert_eq!(token.state.token_info.symbol, "TEST");
        assert_eq!(token.state.token_info.decimals, 18);
        assert_eq!(token.state.total_supply, 1_000_000_000_000_000_000_000);
        assert_eq!(token.balance_of("owner"), 1_000_000_000_000_000_000_000);
        assert!(token.can_mint("owner"));
    }
    
    #[test]
    fn test_transfer() {
        let token = create_test_token();
        
        let transfer_op = FungibleTokenOperation::Transfer {
            from: "owner".to_string(),
            to: "recipient".to_string(),
            amount: 100_000_000_000_000_000_000, // 100 tokens
        };
        
        let new_state = token.apply_operation(transfer_op).unwrap();
        
        assert_eq!(
            new_state.balances.get("owner").unwrap(),
            &900_000_000_000_000_000_000
        );
        assert_eq!(
            new_state.balances.get("recipient").unwrap(),
            &100_000_000_000_000_000_000
        );
    }
    
    #[test]
    fn test_approve_and_transfer_from() {
        let mut token = create_test_token();
        
        // First approve
        let approve_op = FungibleTokenOperation::Approve {
            owner: "owner".to_string(),
            spender: "spender".to_string(),
            amount: 200_000_000_000_000_000_000, // 200 tokens
        };
        
        token.state = token.apply_operation(approve_op).unwrap();
        assert_eq!(token.allowance("owner", "spender"), 200_000_000_000_000_000_000);
        
        // Then transfer from
        let transfer_from_op = FungibleTokenOperation::TransferFrom {
            spender: "spender".to_string(),
            from: "owner".to_string(),
            to: "recipient".to_string(),
            amount: 150_000_000_000_000_000_000, // 150 tokens
        };
        
        token.state = token.apply_operation(transfer_from_op).unwrap();
        
        assert_eq!(token.allowance("owner", "spender"), 50_000_000_000_000_000_000);
        assert_eq!(token.balance_of("recipient"), 150_000_000_000_000_000_000);
        assert_eq!(token.balance_of("owner"), 850_000_000_000_000_000_000);
    }
    
    #[test]
    fn test_mint() {
        let mut token = create_test_token();
        
        let mint_op = FungibleTokenOperation::Mint {
            to: "recipient".to_string(),
            amount: 500_000_000_000_000_000_000, // 500 tokens
            minter: "owner".to_string(),
        };
        
        token.state = token.apply_operation(mint_op).unwrap();
        
        assert_eq!(token.state.total_supply, 1_500_000_000_000_000_000_000);
        assert_eq!(token.balance_of("recipient"), 500_000_000_000_000_000_000);
    }
    
    #[test]
    fn test_burn() {
        let mut token = create_test_token();
        
        // Grant burn permission to owner
        let grant_burner_op = FungibleTokenOperation::GrantBurner {
            admin: "admin".to_string(),
            burner: "owner".to_string(),
        };
        token.state = token.apply_operation(grant_burner_op).unwrap();
        
        let burn_op = FungibleTokenOperation::Burn {
            from: "owner".to_string(),
            amount: 100_000_000_000_000_000_000, // 100 tokens
            burner: "owner".to_string(),
        };
        
        token.state = token.apply_operation(burn_op).unwrap();
        
        assert_eq!(token.state.total_supply, 900_000_000_000_000_000_000);
        assert_eq!(token.balance_of("owner"), 900_000_000_000_000_000_000);
    }
    
    #[test]
    fn test_pause_unpause() {
        let mut token = create_test_token();
        
        // Pause
        let pause_op = FungibleTokenOperation::Pause {
            admin: "admin".to_string(),
        };
        token.state = token.apply_operation(pause_op).unwrap();
        assert!(token.state.paused);
        
        // Try to transfer while paused (should fail)
        let transfer_op = FungibleTokenOperation::Transfer {
            from: "owner".to_string(),
            to: "recipient".to_string(),
            amount: 100_000_000_000_000_000_000,
        };
        assert!(token.apply_operation(transfer_op).is_err());
        
        // Unpause
        let unpause_op = FungibleTokenOperation::Unpause {
            admin: "admin".to_string(),
        };
        token.state = token.apply_operation(unpause_op).unwrap();
        assert!(!token.state.paused);
    }
    
    #[test]
    fn test_insufficient_balance() {
        let token = create_test_token();
        
        let transfer_op = FungibleTokenOperation::Transfer {
            from: "owner".to_string(),
            to: "recipient".to_string(),
            amount: 2_000_000_000_000_000_000_000, // More than available
        };
        
        assert!(token.apply_operation(transfer_op).is_err());
    }
    
    #[test]
    fn test_content_addressability() {
        let token1 = create_test_token();
        let token2 = create_test_token();
        
        // Tokens with same state should have same hash
        assert_eq!(token1.content_id(), token2.content_id());
        
        // Different tokens should have different hashes
        let mut token3 = token1.clone();
        token3.state.total_supply += 1;
        assert_ne!(token1.content_id(), token3.content_id());
    }
} 