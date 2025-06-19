// Token: ERC-20 style token with session-typed send/receive operations

use crate::layer1::{SessionType, Type};
use crate::layer2::outcome::Value;
use crate::layer2::effect::{Effect, EffectRow};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// A token represents a fungible asset on a blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Unique identifier for this token (contract address)
    pub id: TokenId,
    
    /// Token metadata
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    
    /// Total supply of the token
    pub total_supply: u128,
    
    /// Balances for each account
    pub balances: BTreeMap<AccountId, u128>,
    
    /// Allowances for spending on behalf of others
    pub allowances: BTreeMap<(AccountId, AccountId), u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TokenId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AccountId(pub String);

/// Session type for token operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenSessionType {
    /// Send tokens to another account
    Send {
        recipient: AccountId,
        amount: u128,
        continuation: Box<SessionType>,
    },
    
    /// Receive tokens from another account
    Receive {
        sender: AccountId,
        continuation: Box<SessionType>,
    },
    
    /// Approve another account to spend tokens
    Approve {
        spender: AccountId,
        amount: u128,
        continuation: Box<SessionType>,
    },
    
    /// Transfer tokens on behalf of another account
    TransferFrom {
        from: AccountId,
        to: AccountId,
        amount: u128,
        continuation: Box<SessionType>,
    },
    
    /// Query balance
    QueryBalance {
        account: AccountId,
        continuation: Box<SessionType>,
    },
    
    /// End the session
    End,
}

/// Token send operation - models linear transfer of tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSend {
    pub token: TokenId,
    pub from: AccountId,
    pub to: AccountId,
    pub amount: u128,
    pub nonce: u64, // For ordering and replay protection
}

/// Token receive operation - completes the linear transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenReceive {
    pub token: TokenId,
    pub from: AccountId,
    pub to: AccountId,
    pub amount: u128,
    pub nonce: u64, // Must match the send
}

impl Token {
    /// Create a new token
    pub fn new(id: TokenId, name: String, symbol: String, decimals: u8) -> Self {
        Token {
            id,
            name,
            symbol,
            decimals,
            total_supply: 0,
            balances: BTreeMap::new(),
            allowances: BTreeMap::new(),
        }
    }
    
    /// Mint new tokens to an account
    pub fn mint(&mut self, to: &AccountId, amount: u128) -> Result<(), TokenError> {
        // Check for overflow
        let new_supply = self.total_supply
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        let new_balance = self.balance_of(to)
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        self.total_supply = new_supply;
        self.balances.insert(to.clone(), new_balance);
        
        Ok(())
    }
    
    /// Get balance of an account
    pub fn balance_of(&self, account: &AccountId) -> u128 {
        self.balances.get(account).copied().unwrap_or(0)
    }
    
    /// Get allowance for a spender
    pub fn allowance(&self, owner: &AccountId, spender: &AccountId) -> u128 {
        self.allowances
            .get(&(owner.clone(), spender.clone()))
            .copied()
            .unwrap_or(0)
    }
    
    /// Transfer tokens (internal - should be called through session type)
    fn transfer_internal(&mut self, from: &AccountId, to: &AccountId, amount: u128) -> Result<(), TokenError> {
        if from == to {
            return Ok(()); // No-op for self-transfer
        }
        
        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        let to_balance = self.balance_of(to);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        // Update balances
        self.balances.insert(from.clone(), from_balance - amount);
        self.balances.insert(to.clone(), new_to_balance);
        
        Ok(())
    }
    
    /// Approve another account to spend tokens
    pub fn approve(&mut self, owner: &AccountId, spender: &AccountId, amount: u128) -> Result<(), TokenError> {
        self.allowances.insert((owner.clone(), spender.clone()), amount);
        Ok(())
    }
    
    /// Transfer tokens on behalf of another account
    pub fn transfer_from(&mut self, spender: &AccountId, from: &AccountId, to: &AccountId, amount: u128) -> Result<(), TokenError> {
        let allowance = self.allowance(from, spender);
        if allowance < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        
        // Perform the transfer
        self.transfer_internal(from, to, amount)?;
        
        // Update allowance
        let new_allowance = allowance - amount;
        if new_allowance == 0 {
            self.allowances.remove(&(from.clone(), spender.clone()));
        } else {
            self.allowances.insert((from.clone(), spender.clone()), new_allowance);
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TokenError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Insufficient allowance")]
    InsufficientAllowance,
    
    #[error("Arithmetic overflow")]
    Overflow,
    
    #[error("Invalid recipient")]
    InvalidRecipient,
}

/// Effect for token operations
pub enum TokenEffect<A, R> {
    /// Send tokens (creates a pending send)
    TokenSend {
        send: TokenSend,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
    
    /// Receive tokens (completes a pending send)
    TokenReceive {
        receive: TokenReceive,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
    
    /// Approve spending
    TokenApprove {
        token: TokenId,
        owner: AccountId,
        spender: AccountId,
        amount: u128,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
    
    /// Query token balance
    TokenBalance {
        token: TokenId,
        account: AccountId,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
}

/// Linear token transfer protocol
/// Ensures tokens are neither created nor destroyed during transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearTokenTransfer {
    /// The send operation that initiated the transfer
    pub send: TokenSend,
    
    /// Whether the transfer has been completed
    pub completed: bool,
    
    /// Block height when the send was initiated
    pub initiated_at: u64,
    
    /// Maximum block height for completion (timeout)
    pub timeout_at: u64,
}

impl LinearTokenTransfer {
    /// Create a new linear transfer
    pub fn new(send: TokenSend, current_block: u64, timeout_blocks: u64) -> Self {
        LinearTokenTransfer {
            send,
            completed: false,
            initiated_at: current_block,
            timeout_at: current_block + timeout_blocks,
        }
    }
    
    /// Complete the transfer with a matching receive
    pub fn complete(&mut self, receive: &TokenReceive) -> Result<(), TokenError> {
        // Verify the receive matches the send
        if self.send.token != receive.token ||
           self.send.from != receive.from ||
           self.send.to != receive.to ||
           self.send.amount != receive.amount ||
           self.send.nonce != receive.nonce {
            return Err(TokenError::InvalidRecipient);
        }
        
        self.completed = true;
        Ok(())
    }
    
    /// Check if the transfer has timed out
    pub fn is_timed_out(&self, current_block: u64) -> bool {
        current_block > self.timeout_at
    }
}

/// Convert token session type to standard session type
impl From<TokenSessionType> for SessionType {
    fn from(tst: TokenSessionType) -> SessionType {
        match tst {
            TokenSessionType::Send { amount, continuation, .. } => {
                SessionType::Send(
                    Box::new(Type::Int), // Amount
                    continuation,
                )
            }
            TokenSessionType::Receive { continuation, .. } => {
                SessionType::Receive(
                    Box::new(Type::Int), // Amount
                    continuation,
                )
            }
            TokenSessionType::Approve { continuation, .. } => {
                SessionType::Send(
                    Box::new(Type::Bool), // Success
                    continuation,
                )
            }
            TokenSessionType::TransferFrom { continuation, .. } => {
                SessionType::Send(
                    Box::new(Type::Bool), // Success
                    continuation,
                )
            }
            TokenSessionType::QueryBalance { continuation, .. } => {
                SessionType::Send(
                    Box::new(Type::Int), // Balance
                    continuation,
                )
            }
            TokenSessionType::End => SessionType::End,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_transfer() {
        let mut token = Token::new(
            TokenId("TEST".to_string()),
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
        );
        
        let alice = AccountId("alice".to_string());
        let bob = AccountId("bob".to_string());
        
        // Mint tokens to alice
        token.mint(&alice, 1000).unwrap();
        assert_eq!(token.balance_of(&alice), 1000);
        assert_eq!(token.total_supply, 1000);
        
        // Transfer to bob
        token.transfer_internal(&alice, &bob, 300).unwrap();
        assert_eq!(token.balance_of(&alice), 700);
        assert_eq!(token.balance_of(&bob), 300);
        assert_eq!(token.total_supply, 1000); // Total supply unchanged
    }
    
    #[test]
    fn test_linear_transfer() {
        let alice = AccountId("alice".to_string());
        let bob = AccountId("bob".to_string());
        
        // Create a send operation
        let send = TokenSend {
            token: TokenId("TEST".to_string()),
            from: alice.clone(),
            to: bob.clone(),
            amount: 100,
            nonce: 1,
        };
        
        // Create linear transfer
        let mut transfer = LinearTokenTransfer::new(send.clone(), 100, 50);
        assert!(!transfer.completed);
        
        // Create matching receive
        let receive = TokenReceive {
            token: TokenId("TEST".to_string()),
            from: alice,
            to: bob,
            amount: 100,
            nonce: 1,
        };
        
        // Complete the transfer
        transfer.complete(&receive).unwrap();
        assert!(transfer.completed);
    }
    
    #[test]
    fn test_token_allowance() {
        let mut token = Token::new(
            TokenId("TEST".to_string()),
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
        );
        
        let alice = AccountId("alice".to_string());
        let bob = AccountId("bob".to_string());
        let charlie = AccountId("charlie".to_string());
        
        // Mint tokens to alice
        token.mint(&alice, 1000).unwrap();
        
        // Alice approves bob to spend 500
        token.approve(&alice, &bob, 500).unwrap();
        assert_eq!(token.allowance(&alice, &bob), 500);
        
        // Bob transfers 300 from alice to charlie
        token.transfer_from(&bob, &alice, &charlie, 300).unwrap();
        
        assert_eq!(token.balance_of(&alice), 700);
        assert_eq!(token.balance_of(&charlie), 300);
        assert_eq!(token.allowance(&alice, &bob), 200); // Allowance reduced
    }
} 