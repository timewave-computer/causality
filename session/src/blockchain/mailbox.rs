// Mailbox: A session-typed account that handles deposits with linear semantics
// When tokens are deposited externally, they only "enter" the system when the counter increments

use crate::layer1::{SessionType, Type};
use crate::layer2::outcome::{Value, StateLocation};
use crate::layer2::effect::{Effect, EffectRow};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// A mailbox is a blockchain account modeled as a session type
/// It maintains linear semantics even when receiving ad hoc deposits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    /// Unique identifier for this mailbox
    pub id: MailboxId,
    
    /// Session type defining the protocol this mailbox follows
    pub session_type: SessionType,
    
    /// Counter tracking linear consumption of deposits
    pub deposit_counter: u64,
    
    /// Pending deposits that haven't been consumed yet
    pub pending_deposits: Vec<PendingDeposit>,
    
    /// Current balance after consumed deposits
    pub consumed_balance: BTreeMap<TokenId, u128>,
    
    /// Constraints on what operations are allowed
    pub constraints: Vec<MailboxConstraint>,
    
    /// Safe deposit mode configuration
    pub safe_deposit_mode: Option<SafeDepositConfig>,
    
    /// Rejected deposits that need to be refunded
    pub rejected_deposits: Vec<RejectedDeposit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MailboxId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDeposit {
    pub token: TokenId,
    pub amount: u128,
    pub depositor: String,
    pub block_height: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailboxConstraint {
    /// Only specific addresses can withdraw
    AllowedWithdrawers(Vec<String>),
    
    /// Maximum withdrawal per operation
    MaxWithdrawalAmount(u128),
    
    /// Time window when withdrawals are allowed
    WithdrawalTimeWindow { start: u64, end: u64 },
    
    /// Required number of confirmations before withdrawal
    RequiredConfirmations(u32),
}

/// Session type for mailbox operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailboxSessionType {
    /// Check for pending deposits and consume them
    ConsumeDeposits {
        continuation: Box<SessionType>,
    },
    
    /// Send tokens from the mailbox
    Send {
        token: Type,
        amount: Type,
        recipient: Type,
        continuation: Box<SessionType>,
    },
    
    /// Query current state
    Query {
        response: Type,
        continuation: Box<SessionType>,
    },
    
    /// End the session
    End,
}

/// Configuration for safe deposit mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeDepositConfig {
    /// Conditions that trigger deposit rejection
    pub rejection_conditions: Vec<DepositCondition>,
    
    /// Whether to automatically refund rejected deposits
    pub auto_refund: bool,
    
    /// Maximum time to hold rejected deposits before mandatory refund
    pub refund_timeout_blocks: u64,
}

/// Conditions that can trigger deposit rejection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepositCondition {
    /// Maximum total balance per token
    DepositCap { token: TokenId, max_amount: u128 },
    
    /// Maximum single deposit amount
    MaxSingleDeposit { max_amount: u128 },
    
    /// Only accept deposits from specific addresses
    AllowedDepositors(Vec<String>),
    
    /// Reject deposits after a certain block height
    DeadlineBlock(u64),
    
    /// Custom condition (would be evaluated by external logic)
    Custom { condition_id: String },
}

/// A rejected deposit that needs to be refunded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedDeposit {
    pub deposit: PendingDeposit,
    pub rejection_reason: RejectionReason,
    pub rejected_at_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RejectionReason {
    DepositCapReached { token: TokenId, cap: u128 },
    DepositTooLarge { amount: u128, max: u128 },
    UnauthorizedDepositor { depositor: String },
    DeadlinePassed { deadline: u64 },
    CustomCondition { condition_id: String },
}

/// A refund that needs to be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refund {
    pub recipient: String,
    pub token: TokenId,
    pub amount: u128,
    pub nonce: u64,
    pub reason: RejectionReason,
}

/// Receipt for a deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositReceipt {
    pub mailbox_id: MailboxId,
    pub depositor: String,
    pub token: TokenId,
    pub amount: u128,
    pub nonce: u64,
    pub status: DepositStatus,
}

/// Status of a deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepositStatus {
    Pending,
    Consumed { at_counter: u64 },
    Rejected { reason: RejectionReason },
    Refunded { at_block: u64 },
}

impl Mailbox {
    /// Create a new mailbox with initial session type
    pub fn new(id: MailboxId, session_type: SessionType) -> Self {
        Mailbox {
            id,
            session_type,
            deposit_counter: 0,
            pending_deposits: Vec::new(),
            consumed_balance: BTreeMap::new(),
            constraints: Vec::new(),
            safe_deposit_mode: None,
            rejected_deposits: Vec::new(),
        }
    }
    
    /// Add a constraint to this mailbox
    pub fn add_constraint(&mut self, constraint: MailboxConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Record an external deposit (doesn't consume it)
    pub fn record_deposit(&mut self, deposit: PendingDeposit) {
        self.pending_deposits.push(deposit);
    }
    
    /// Consume pending deposits, incrementing the counter
    /// This is where deposits "enter" the linear system
    pub fn consume_deposits(&mut self) -> Vec<ConsumedDeposit> {
        let mut consumed = Vec::new();
        
        // Take all pending deposits
        let deposits = std::mem::take(&mut self.pending_deposits);
        
        for deposit in deposits {
            // Increment counter for each deposit consumed
            self.deposit_counter += 1;
            
            // Add to consumed balance
            *self.consumed_balance
                .entry(deposit.token.clone())
                .or_insert(0) += deposit.amount;
            
            consumed.push(ConsumedDeposit {
                deposit,
                consumption_index: self.deposit_counter,
            });
        }
        
        consumed
    }
    
    /// Check if a withdrawal is allowed given constraints
    pub fn can_withdraw(&self, token: &TokenId, amount: u128, recipient: &str) -> bool {
        // Check balance
        if self.consumed_balance.get(token).copied().unwrap_or(0) < amount {
            return false;
        }
        
        // Check all constraints
        for constraint in &self.constraints {
            match constraint {
                MailboxConstraint::AllowedWithdrawers(allowed) => {
                    if !allowed.contains(&recipient.to_string()) {
                        return false;
                    }
                }
                MailboxConstraint::MaxWithdrawalAmount(max) => {
                    if amount > *max {
                        return false;
                    }
                }
                // Add other constraint checks as needed
                _ => {}
            }
        }
        
        true
    }
    
    /// Execute a withdrawal if allowed
    pub fn withdraw(&mut self, token: &TokenId, amount: u128, recipient: &str) -> Result<(), MailboxError> {
        if !self.can_withdraw(token, amount, recipient) {
            return Err(MailboxError::WithdrawalNotAllowed);
        }
        
        // Deduct from balance
        let balance = self.consumed_balance.get_mut(token)
            .ok_or(MailboxError::InsufficientBalance)?;
        
        if *balance < amount {
            return Err(MailboxError::InsufficientBalance);
        }
        
        *balance -= amount;
        Ok(())
    }
    
    /// Generate a unique nonce for deposits
    fn generate_nonce(&self) -> u64 {
        // Simple nonce generation based on deposit counter and timestamp simulation
        self.deposit_counter + (self.pending_deposits.len() as u64) + 1
    }
    
    /// Receive a deposit (standard mode without safe checks)
    pub fn receive_deposit(
        &mut self,
        depositor: String,
        token: TokenId,
        amount: u128,
        block_height: u64,
    ) -> Result<DepositReceipt, MailboxError> {
        let nonce = self.generate_nonce();
        let deposit = PendingDeposit {
            token: token.clone(),
            amount,
            depositor: depositor.clone(),
            block_height,
            nonce,
        };
        
        self.record_deposit(deposit);
        
        Ok(DepositReceipt {
            mailbox_id: self.id.clone(),
            depositor,
            token,
            amount,
            nonce,
            status: DepositStatus::Pending,
        })
    }
    
    /// Create a new mailbox with safe deposit mode
    pub fn new_safe(
        id: MailboxId,
        session_type: SessionType,
        safe_config: SafeDepositConfig,
    ) -> Self {
        Self {
            id,
            session_type,
            deposit_counter: 0,
            pending_deposits: Vec::new(),
            consumed_balance: BTreeMap::new(),
            constraints: Vec::new(),
            safe_deposit_mode: Some(safe_config),
            rejected_deposits: Vec::new(),
        }
    }
    
    /// Check if a deposit should be rejected based on safe deposit conditions
    pub fn should_reject_deposit(
        &self,
        deposit: &PendingDeposit,
        current_block: u64,
    ) -> Option<RejectionReason> {
        let config = self.safe_deposit_mode.as_ref()?;
        
        for condition in &config.rejection_conditions {
            match condition {
                DepositCondition::DepositCap { token, max_amount } => {
                    if &deposit.token == token {
                        let current_balance = self.consumed_balance.get(token).unwrap_or(&0);
                        let pending_amount: u128 = self.pending_deposits
                            .iter()
                            .filter(|d| &d.token == token)
                            .map(|d| d.amount)
                            .sum();
                        
                        if current_balance + pending_amount + deposit.amount > *max_amount {
                            return Some(RejectionReason::DepositCapReached {
                                token: token.clone(),
                                cap: *max_amount,
                            });
                        }
                    }
                }
                
                DepositCondition::MaxSingleDeposit { max_amount } => {
                    if deposit.amount > *max_amount {
                        return Some(RejectionReason::DepositTooLarge {
                            amount: deposit.amount,
                            max: *max_amount,
                        });
                    }
                }
                
                DepositCondition::AllowedDepositors(allowed) => {
                    if !allowed.contains(&deposit.depositor) {
                        return Some(RejectionReason::UnauthorizedDepositor {
                            depositor: deposit.depositor.clone(),
                        });
                    }
                }
                
                DepositCondition::DeadlineBlock(deadline) => {
                    if current_block > *deadline {
                        return Some(RejectionReason::DeadlinePassed {
                            deadline: *deadline,
                        });
                    }
                }
                
                DepositCondition::Custom { condition_id } => {
                    // Custom conditions would be evaluated externally
                    // For now, we'll skip them
                    continue;
                }
            }
        }
        
        None
    }
    
    /// Receive a deposit with safe mode checking
    pub fn receive_deposit_safe(
        &mut self,
        depositor: String,
        token: TokenId,
        amount: u128,
        current_block: u64,
    ) -> Result<DepositReceipt, MailboxError> {
        let deposit = PendingDeposit {
            depositor,
            token,
            amount,
            block_height: current_block,
            nonce: self.generate_nonce(),
        };
        
        // Check if deposit should be rejected
        if let Some(rejection_reason) = self.should_reject_deposit(&deposit, current_block) {
            self.rejected_deposits.push(RejectedDeposit {
                deposit: deposit.clone(),
                rejection_reason: rejection_reason.clone(),
                rejected_at_block: current_block,
            });
            
            return Err(MailboxError::DepositRejected {
                reason: format!("{:?}", rejection_reason),
            });
        }
        
        // If not rejected, process normally
        self.receive_deposit(
            deposit.depositor,
            deposit.token,
            deposit.amount,
            deposit.block_height,
        )
    }
    
    /// Get refunds that need to be processed
    pub fn get_pending_refunds(&self, current_block: u64) -> Vec<Refund> {
        let config = match &self.safe_deposit_mode {
            Some(c) => c,
            None => return Vec::new(),
        };
        
        self.rejected_deposits
            .iter()
            .filter(|rejected| {
                // Check if refund is due
                config.auto_refund || 
                (current_block - rejected.rejected_at_block) >= config.refund_timeout_blocks
            })
            .map(|rejected| Refund {
                recipient: rejected.deposit.depositor.clone(),
                token: rejected.deposit.token.clone(),
                amount: rejected.deposit.amount,
                nonce: rejected.deposit.nonce,
                reason: rejected.rejection_reason.clone(),
            })
            .collect()
    }
    
    /// Process refunds and remove them from rejected list
    pub fn process_refunds(&mut self, processed_nonces: &[u64]) {
        self.rejected_deposits.retain(|rejected| {
            !processed_nonces.contains(&rejected.deposit.nonce)
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumedDeposit {
    pub deposit: PendingDeposit,
    pub consumption_index: u64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MailboxError {
    #[error("Withdrawal not allowed by constraints")]
    WithdrawalNotAllowed,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid session state")]
    InvalidSessionState,
    
    #[error("Deposit rejected")]
    DepositRejected { reason: String },
}

/// Effect for mailbox operations
pub enum MailboxEffect<A, R> {
    /// Check and consume any pending deposits
    ConsumeDeposits {
        mailbox: MailboxId,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
    
    /// Send tokens from mailbox
    SendFromMailbox {
        mailbox: MailboxId,
        token: TokenId,
        amount: u128,
        recipient: String,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
    
    /// Query mailbox state
    QueryMailbox {
        mailbox: MailboxId,
        _phantom: std::marker::PhantomData<(A, R)>,
    },
}

/// Convert mailbox session type to standard session type
impl From<MailboxSessionType> for SessionType {
    fn from(mst: MailboxSessionType) -> SessionType {
        match mst {
            MailboxSessionType::ConsumeDeposits { continuation } => {
                SessionType::Receive(
                    Box::new(Type::Int), // Number of deposits consumed
                    continuation,
                )
            }
            MailboxSessionType::Send { continuation, .. } => {
                SessionType::Send(
                    Box::new(Type::Unit), // Simplified for now
                    continuation,
                )
            }
            MailboxSessionType::Query { response, continuation } => {
                SessionType::Send(
                    Box::new(response),
                    continuation,
                )
            }
            MailboxSessionType::End => SessionType::End,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mailbox_linear_consumption() {
        let mut mailbox = Mailbox::new(
            MailboxId("test".to_string()),
            SessionType::End,
        );
        
        // External deposits don't affect counter
        mailbox.record_deposit(PendingDeposit {
            token: TokenId("ETH".to_string()),
            amount: 100,
            depositor: "alice".to_string(),
            block_height: 1,
            nonce: 0,
        });
        
        assert_eq!(mailbox.deposit_counter, 0);
        assert_eq!(mailbox.consumed_balance.len(), 0);
        
        // Consuming deposits increments counter
        let consumed = mailbox.consume_deposits();
        
        assert_eq!(mailbox.deposit_counter, 1);
        assert_eq!(consumed.len(), 1);
        assert_eq!(mailbox.consumed_balance[&TokenId("ETH".to_string())], 100);
    }
    
    #[test]
    fn test_mailbox_constraints() {
        let mut mailbox = Mailbox::new(
            MailboxId("test".to_string()),
            SessionType::End,
        );
        
        // Add constraint
        mailbox.add_constraint(MailboxConstraint::AllowedWithdrawers(
            vec!["bob".to_string()]
        ));
        
        // Deposit and consume
        mailbox.record_deposit(PendingDeposit {
            token: TokenId("ETH".to_string()),
            amount: 100,
            depositor: "alice".to_string(),
            block_height: 1,
            nonce: 0,
        });
        mailbox.consume_deposits();
        
        // Check constraints
        assert!(!mailbox.can_withdraw(&TokenId("ETH".to_string()), 50, "alice"));
        assert!(mailbox.can_withdraw(&TokenId("ETH".to_string()), 50, "bob"));
    }
    
    #[test]
    fn test_mailbox_safe_deposit_mode() {
        // Create mailbox with deposit cap
        let safe_config = SafeDepositConfig {
            rejection_conditions: vec![
                DepositCondition::DepositCap {
                    token: TokenId("ETH".to_string()),
                    max_amount: 1000,
                },
                DepositCondition::MaxSingleDeposit { max_amount: 500 },
            ],
            auto_refund: true,
            refund_timeout_blocks: 100,
        };
        
        let mut mailbox = Mailbox::new_safe(
            MailboxId("safe_test".to_string()),
            SessionType::End,
            safe_config,
        );
        
        // First deposit should succeed
        let result = mailbox.receive_deposit_safe(
            "alice".to_string(),
            TokenId("ETH".to_string()),
            400,
            10,
        );
        assert!(result.is_ok());
        
        // Second deposit should succeed (total 800)
        let result = mailbox.receive_deposit_safe(
            "bob".to_string(),
            TokenId("ETH".to_string()),
            400,
            20,
        );
        assert!(result.is_ok());
        
        // Third deposit should be rejected (would exceed cap)
        let result = mailbox.receive_deposit_safe(
            "charlie".to_string(),
            TokenId("ETH".to_string()),
            300,
            30,
        );
        assert!(result.is_err());
        assert_eq!(mailbox.rejected_deposits.len(), 1);
        
        // Large single deposit should be rejected
        let result = mailbox.receive_deposit_safe(
            "dave".to_string(),
            TokenId("ETH".to_string()),
            600,
            40,
        );
        assert!(result.is_err());
        assert_eq!(mailbox.rejected_deposits.len(), 2);
        
        // Check pending refunds
        let refunds = mailbox.get_pending_refunds(50);
        assert_eq!(refunds.len(), 2);
        assert_eq!(refunds[0].recipient, "charlie");
        assert_eq!(refunds[0].amount, 300);
        assert_eq!(refunds[1].recipient, "dave");
        assert_eq!(refunds[1].amount, 600);
        
        // Process refunds
        let nonces: Vec<u64> = refunds.iter().map(|r| r.nonce).collect();
        mailbox.process_refunds(&nonces);
        assert_eq!(mailbox.rejected_deposits.len(), 0);
    }
    
    #[test]
    fn test_mailbox_allowed_depositors() {
        let safe_config = SafeDepositConfig {
            rejection_conditions: vec![
                DepositCondition::AllowedDepositors(vec![
                    "alice".to_string(),
                    "bob".to_string(),
                ]),
            ],
            auto_refund: false,
            refund_timeout_blocks: 50,
        };
        
        let mut mailbox = Mailbox::new_safe(
            MailboxId("whitelist_test".to_string()),
            SessionType::End,
            safe_config,
        );
        
        // Allowed depositor should succeed
        let result = mailbox.receive_deposit_safe(
            "alice".to_string(),
            TokenId("ETH".to_string()),
            100,
            10,
        );
        assert!(result.is_ok());
        
        // Unauthorized depositor should be rejected
        let result = mailbox.receive_deposit_safe(
            "charlie".to_string(),
            TokenId("ETH".to_string()),
            100,
            20,
        );
        assert!(result.is_err());
        
        // Check refunds - should be empty (auto_refund is false)
        let refunds = mailbox.get_pending_refunds(30);
        assert_eq!(refunds.len(), 0);
        
        // After timeout, refund should be available
        let refunds = mailbox.get_pending_refunds(71);
        assert_eq!(refunds.len(), 1);
        assert_eq!(refunds[0].recipient, "charlie");
    }
} 