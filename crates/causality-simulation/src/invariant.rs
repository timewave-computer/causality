// Purpose: Defines the invariant checking system for monitoring simulation state and reporting violations
//
// This module provides functionality for defining, checking, and reporting invariant violations
// during simulation runs, allowing for automatic detection of rule violations

use std::fmt;
use std::collections::HashMap;
use serde_json::Value;

use crate::replay::{LogEntry, LogEntryType, Result};
use crate::agent::AgentId;
use crate::scenario::InvariantConfig;
use crate::observer::{Observer, LogFilter};
use causality_types::DomainId;

/// Type of invariant being checked
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantType {
    /// No negative balance allowed for any asset
    NoNegativeBalances,
    /// Custom invariant with a name
    Custom(String),
}

impl fmt::Display for InvariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoNegativeBalances => write!(f, "NoNegativeBalances"),
            Self::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Result of an invariant check
#[derive(Debug, Clone)]
pub enum InvariantResult {
    /// Invariant was satisfied
    Satisfied,
    /// Invariant was violated
    Violated {
        /// Type of invariant that was violated
        invariant_type: InvariantType,
        /// Description of the violation
        message: String,
        /// Entry that triggered the violation
        triggering_entry: Option<LogEntry>,
        /// Additional context data for debugging
        context: HashMap<String, Value>,
    },
}

/// Trait defining an invariant checker
pub trait InvariantChecker: Send + Sync + std::fmt::Debug {
    /// Get the type of this invariant
    fn invariant_type(&self) -> InvariantType;
    
    /// Check if the invariant is satisfied given a log entry
    fn check(&self, entry: &LogEntry) -> InvariantResult;
    
    /// Get a filter for log entries that are relevant to this invariant
    fn log_filter(&self) -> Option<LogFilter>;
}

/// Observer for simulation invariants
pub struct InvariantObserver {
    /// List of invariant checkers to apply
    checkers: Vec<Box<dyn InvariantChecker>>,
    /// Callback for reporting violations
    violation_callback: Option<Box<dyn Fn(InvariantResult) + Send + Sync>>,
    /// Track number of violations detected
    violation_count: std::sync::atomic::AtomicUsize,
    /// Filter for log entries
    filter: Option<LogFilter>,
}

impl std::fmt::Debug for InvariantObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvariantObserver")
            .field("checkers_count", &self.checkers.len())
            .field("has_violation_callback", &self.violation_callback.is_some())
            .field("violation_count", &self.violation_count.load(std::sync::atomic::Ordering::Relaxed))
            .field("filter", &self.filter)
            .finish()
    }
}

impl InvariantObserver {
    /// Create a new invariant observer
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
            violation_callback: None,
            violation_count: std::sync::atomic::AtomicUsize::new(0),
            filter: None,
        }
    }
    
    /// Create a new invariant observer from a scenario's invariant config
    pub fn from_config(config: &InvariantConfig) -> Self {
        let mut observer = Self::new();
        
        if let Some(true) = config.no_negative_balances {
            observer.add_checker(Box::new(NoNegativeBalancesChecker::new()));
        }
        
        // Add other checkers based on config
        
        observer
    }
    
    /// Add an invariant checker
    pub fn add_checker(&mut self, checker: Box<dyn InvariantChecker>) -> &mut Self {
        self.checkers.push(checker);
        self
    }
    
    /// Set a callback to be notified of violations
    pub fn set_violation_callback<F>(&mut self, callback: F) -> &mut Self 
    where
        F: Fn(InvariantResult) + Send + Sync + 'static,
    {
        self.violation_callback = Some(Box::new(callback));
        self
    }
    
    /// Get the number of violations detected
    pub fn violation_count(&self) -> usize {
        self.violation_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Handle a violation by reporting it and incrementing the counter
    fn handle_violation(&self, result: InvariantResult) {
        self.violation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if let Some(callback) = &self.violation_callback {
            callback(result);
        }
    }
}

impl Observer for InvariantObserver {
    fn on_log_entry(&self, entry: LogEntry) {
        // Check if the entry matches our filter
        if let Some(filter) = &self.filter {
            if !filter.matches(&entry) {
                return;
            }
        }
        
        for checker in &self.checkers {
            // Check if this entry is relevant to this checker
            if let Some(filter) = checker.log_filter() {
                if !filter.matches(&entry) {
                    continue;
                }
            }
            
            // Check the invariant
            let result = checker.check(&entry);
            
            // Report violation if any
            if let InvariantResult::Violated { .. } = result {
                self.handle_violation(result.clone());
            }
        }
    }
    
    fn on_simulation_start(&self, _run_id: &str) {
        // Reset violation counter on start
        self.violation_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn on_simulation_end(&self, _run_id: &str) {
        // Could report final count here if needed
    }
    
    fn apply_filter(&mut self, filter: LogFilter) {
        self.filter = Some(filter);
    }
}

/// Checker for no negative balances invariant
#[derive(Debug, Clone)]
pub struct NoNegativeBalancesChecker {
    /// Track asset balances per agent
    balances: HashMap<(AgentId, String), i64>, // (agent_id, asset) -> balance
}

impl NoNegativeBalancesChecker {
    /// Create a new checker for no negative balances
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }
    
    /// Update balance tracking based on a transaction
    fn update_balance(&mut self, agent_id: &AgentId, asset: &str, delta: i64) -> InvariantResult {
        let key = (agent_id.clone(), asset.to_string());
        let current = *self.balances.get(&key).unwrap_or(&0);
        let new_balance = current + delta;
        
        self.balances.insert(key.clone(), new_balance);
        
        if new_balance < 0 {
            let mut context = HashMap::new();
            context.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
            context.insert("asset".to_string(), Value::String(asset.to_string()));
            context.insert("balance".to_string(), Value::Number(new_balance.into()));
            
            InvariantResult::Violated {
                invariant_type: InvariantType::NoNegativeBalances,
                message: format!("Agent {} has negative balance of {} for asset {}", agent_id, new_balance, asset),
                triggering_entry: None,
                context,
            }
        } else {
            InvariantResult::Satisfied
        }
    }
}

impl InvariantChecker for NoNegativeBalancesChecker {
    fn invariant_type(&self) -> InvariantType {
        InvariantType::NoNegativeBalances
    }
    
    fn check(&self, entry: &LogEntry) -> InvariantResult {
        // Clone self to allow mutation during check
        let mut checker = Self {
            balances: self.balances.clone(),
        };
        
        // Check for relevant entry types
        match entry.entry_type {
            LogEntryType::DomainEvent => {
                // Process transaction events
                if let Some(event_type) = entry.metadata.get("event_type") {
                    if event_type == "transaction" {
                        if let Some(agent_id) = &entry.agent_id {
                            // Extract transaction details from payload
                            if let Some(asset) = entry.metadata.get("asset") {
                                if let Some(amount_str) = entry.metadata.get("amount") {
                                    if let Ok(amount) = amount_str.parse::<i64>() {
                                        return checker.update_balance(agent_id, asset, amount);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            LogEntryType::AgentState => {
                // Check for balance updates in agent state
                if let Some(agent_id) = &entry.agent_id {
                    if let Some(balances) = entry.payload.get("balances") {
                        if let Some(balances_obj) = balances.as_object() {
                            for (asset, balance_value) in balances_obj {
                                if let Some(balance) = balance_value.as_i64() {
                                    if balance < 0 {
                                        let mut context = HashMap::new();
                                        context.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
                                        context.insert("asset".to_string(), Value::String(asset.to_string()));
                                        context.insert("balance".to_string(), Value::Number(balance.into()));
                                        
                                        return InvariantResult::Violated {
                                            invariant_type: InvariantType::NoNegativeBalances,
                                            message: format!("Agent {} has negative balance of {} for asset {}", 
                                                           agent_id, balance, asset),
                                            triggering_entry: Some(entry.clone()),
                                            context,
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        InvariantResult::Satisfied
    }
    
    fn log_filter(&self) -> Option<LogFilter> {
        // Filter for transaction events and agent state updates
        Some(LogFilter::new()
            .with_entry_type(LogEntryType::DomainEvent)
            .with_entry_type(LogEntryType::AgentState))
    }
}

/// Factory for creating invariant checkers
pub struct InvariantCheckerFactory;

impl InvariantCheckerFactory {
    /// Create checker for NoNegativeBalances invariant
    pub fn create_no_negative_balances_checker() -> Box<dyn InvariantChecker> {
        Box::new(NoNegativeBalancesChecker::new())
    }
    
    /// Create all configured checkers based on invariant config
    pub fn create_all_from_config(config: &InvariantConfig) -> Vec<Box<dyn InvariantChecker>> {
        let mut checkers = Vec::new();
        
        if let Some(true) = config.no_negative_balances {
            checkers.push(Self::create_no_negative_balances_checker());
        }
        
        // Add other checkers as they're implemented
        
        checkers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use crate::agent::agent_id;
    
    fn create_test_entry(
        entry_type: LogEntryType,
        agent_id: Option<&str>,
        payload: Value,
        metadata: HashMap<String, String>,
    ) -> LogEntry {
        LogEntry {
            id: "test-entry".to_string(),
            timestamp: Utc::now(),
            entry_type,
            agent_id: agent_id.map(|id| agent_id::from_string(id)),
            domain: Some("test-domain".to_string()),
            payload,
            parent_id: None,
            run_id: Some("test-run".to_string()),
            metadata,
            content_hash: "test-hash".to_string(),
        }
    }
    
    #[test]
    fn test_no_negative_balances_checker() {
        let checker = NoNegativeBalancesChecker::new();
        
        // Create agent state entry with positive balance
        let mut metadata = HashMap::new();
        let payload = serde_json::json!({
            "balances": {
                "USD": 100,
                "EUR": 50
            }
        });
        let entry_ok = create_test_entry(
            LogEntryType::AgentState,
            Some("agent1"),
            payload,
            metadata.clone(),
        );
        
        // Check should pass
        let result = checker.check(&entry_ok);
        assert!(matches!(result, InvariantResult::Satisfied));
        
        // Create agent state entry with negative balance
        let payload_negative = serde_json::json!({
            "balances": {
                "USD": -10,
                "EUR": 50
            }
        });
        let entry_negative = create_test_entry(
            LogEntryType::AgentState,
            Some("agent1"),
            payload_negative,
            metadata.clone(),
        );
        
        // Check should fail
        let result = checker.check(&entry_negative);
        assert!(matches!(result, InvariantResult::Violated { .. }));
        
        // Create transaction event
        metadata.insert("event_type".to_string(), "transaction".to_string());
        metadata.insert("asset".to_string(), "USD".to_string());
        metadata.insert("amount".to_string(), "-150".to_string());
        let entry_tx = create_test_entry(
            LogEntryType::DomainEvent,
            Some("agent1"),
            serde_json::json!({"transaction_id": "tx1"}),
            metadata,
        );
        
        // Check transaction
        let result = checker.check(&entry_tx);
        assert!(matches!(result, InvariantResult::Violated { .. }));
    }
    
    #[test]
    fn test_invariant_observer() {
        // Create an observer
        let mut observer = InvariantObserver::new();
        observer.add_checker(Box::new(NoNegativeBalancesChecker::new()));
        
        // Track violations
        let violations = Arc::new(Mutex::new(Vec::new()));
        let violations_clone = violations.clone();
        
        observer.set_violation_callback(move |result| {
            violations_clone.lock().unwrap().push(result);
        });
        
        // Create a log entry with negative balance
        let metadata = HashMap::new();
        let payload = serde_json::json!({
            "balances": {
                "USD": -50
            }
        });
        let entry = create_test_entry(
            LogEntryType::AgentState,
            Some("agent1"),
            payload,
            metadata,
        );
        
        // Process the entry
        observer.on_log_entry(entry);
        
        // Check that a violation was recorded
        assert_eq!(observer.violation_count(), 1);
        assert_eq!(violations.lock().unwrap().len(), 1);
    }
} 