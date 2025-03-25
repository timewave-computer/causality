// Safe state utilities for schema operations
// Original file: src/schema/safe_state.rs

//! Safe State Management
//!
//! This module provides safe state management for schema evolution,
//! ensuring that schema changes only happen when the system is in a safe state.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::schema::{Error, Result, Schema};

/// A safe state strategy for schema evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SafeStateStrategy {
    /// No checks - always considered safe (use with caution!)
    None,
    /// No in-flight operations should exist
    NoInFlightOperations,
    /// No pending returns from cross-program calls
    NoPendingReturns,
    /// No active operators running
    NoActiveOperators,
    /// Custom safe state strategy
    Custom(String),
}

impl Default for SafeStateStrategy {
    fn default() -> Self {
        SafeStateStrategy::NoInFlightOperations
    }
}

/// The status of a safe state check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeStateStatus {
    /// The system is in a safe state
    Safe,
    /// The system is not in a safe state
    Unsafe(String),
    /// The check timed out
    Timeout,
}

/// A callback function to check if a domain is in a safe state
pub type SafeStateCheckFn = fn(&str) -> Result<bool>;

/// Options for safe state management
#[derive(Debug, Clone)]
pub struct SafeStateOptions {
    /// The strategy to use for determining safe state
    pub strategy: SafeStateStrategy,
    /// The timeout for safe state checks
    pub timeout: Duration,
    /// Whether to block until safe state is reached
    pub block_until_safe: bool,
    /// Max blocking duration
    pub max_block_duration: Option<Duration>,
    /// Domains to exclude from safe state checks
    pub excluded_domains: HashSet<String>,
}

impl Default for SafeStateOptions {
    fn default() -> Self {
        SafeStateOptions {
            strategy: SafeStateStrategy::default(),
            timeout: Duration::from_secs(10),
            block_until_safe: true,
            max_block_duration: Some(Duration::from_secs(60)),
            excluded_domains: HashSet::new(),
        }
    }
}

/// A safe state manager for schema evolution
pub struct SafeStateManager {
    /// Options for the safe state manager
    options: SafeStateOptions,
    /// Custom check functions by domain
    custom_checks: RwLock<HashMap<String, SafeStateCheckFn>>,
    /// Operations in flight by domain
    in_flight: RwLock<HashMap<String, u64>>,
    /// Pending returns by domain
    pending_returns: RwLock<HashMap<String, u64>>,
    /// Active operators by domain
    active_operators: RwLock<HashMap<String, u64>>,
}

impl SafeStateManager {
    /// Create a new safe state manager
    pub fn new(options: SafeStateOptions) -> Self {
        SafeStateManager {
            options,
            custom_checks: RwLock::new(HashMap::new()),
            in_flight: RwLock::new(HashMap::new()),
            pending_returns: RwLock::new(HashMap::new()),
            active_operators: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a custom safe state check function for a domain
    pub fn register_check(&self, domain: impl Into<String>, check_fn: SafeStateCheckFn) -> Result<()> {
        let mut checks = self.custom_checks.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on custom checks".to_string())
        })?;
        
        checks.insert(domain.into(), check_fn);
        
        Ok(())
    }
    
    /// Increment the in-flight operations count for a domain
    pub fn increment_in_flight(&self, domain: &str) -> Result<()> {
        let mut in_flight = self.in_flight.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on in-flight operations".to_string())
        })?;
        
        let count = in_flight.entry(domain.to_string()).or_insert(0);
        *count += 1;
        
        Ok(())
    }
    
    /// Decrement the in-flight operations count for a domain
    pub fn decrement_in_flight(&self, domain: &str) -> Result<()> {
        let mut in_flight = self.in_flight.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on in-flight operations".to_string())
        })?;
        
        if let Some(count) = in_flight.get_mut(domain) {
            if *count > 0 {
                *count -= 1;
            }
        }
        
        Ok(())
    }
    
    /// Increment the pending returns count for a domain
    pub fn increment_pending_returns(&self, domain: &str) -> Result<()> {
        let mut pending = self.pending_returns.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on pending returns".to_string())
        })?;
        
        let count = pending.entry(domain.to_string()).or_insert(0);
        *count += 1;
        
        Ok(())
    }
    
    /// Decrement the pending returns count for a domain
    pub fn decrement_pending_returns(&self, domain: &str) -> Result<()> {
        let mut pending = self.pending_returns.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on pending returns".to_string())
        })?;
        
        if let Some(count) = pending.get_mut(domain) {
            if *count > 0 {
                *count -= 1;
            }
        }
        
        Ok(())
    }
    
    /// Increment the active operators count for a domain
    pub fn increment_active_operators(&self, domain: &str) -> Result<()> {
        let mut active = self.active_operators.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on active operators".to_string())
        })?;
        
        let count = active.entry(domain.to_string()).or_insert(0);
        *count += 1;
        
        Ok(())
    }
    
    /// Decrement the active operators count for a domain
    pub fn decrement_active_operators(&self, domain: &str) -> Result<()> {
        let mut active = self.active_operators.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on active operators".to_string())
        })?;
        
        if let Some(count) = active.get_mut(domain) {
            if *count > 0 {
                *count -= 1;
            }
        }
        
        Ok(())
    }
    
    /// Check if a domain is in a safe state
    pub fn check_domain(&self, domain: &str) -> Result<SafeStateStatus> {
        // Skip check for excluded domains
        if self.options.excluded_domains.contains(domain) {
            return Ok(SafeStateStatus::Safe);
        }
        
        match self.options.strategy {
            SafeStateStrategy::None => {
                // Always considered safe
                Ok(SafeStateStatus::Safe)
            },
            SafeStateStrategy::NoInFlightOperations => {
                // Check if there are any in-flight operations
                let in_flight = self.in_flight.read().map_err(|_| {
                    Error::Migration("Failed to acquire read lock on in-flight operations".to_string())
                })?;
                
                if let Some(count) = in_flight.get(domain) {
                    if *count > 0 {
                        return Ok(SafeStateStatus::Unsafe(format!(
                            "Domain {} has {} in-flight operations",
                            domain, count
                        )));
                    }
                }
                
                Ok(SafeStateStatus::Safe)
            },
            SafeStateStrategy::NoPendingReturns => {
                // Check if there are any pending returns
                let pending = self.pending_returns.read().map_err(|_| {
                    Error::Migration("Failed to acquire read lock on pending returns".to_string())
                })?;
                
                if let Some(count) = pending.get(domain) {
                    if *count > 0 {
                        return Ok(SafeStateStatus::Unsafe(format!(
                            "Domain {} has {} pending returns",
                            domain, count
                        )));
                    }
                }
                
                Ok(SafeStateStatus::Safe)
            },
            SafeStateStrategy::NoActiveOperators => {
                // Check if there are any active operators
                let active = self.active_operators.read().map_err(|_| {
                    Error::Migration("Failed to acquire read lock on active operators".to_string())
                })?;
                
                if let Some(count) = active.get(domain) {
                    if *count > 0 {
                        return Ok(SafeStateStatus::Unsafe(format!(
                            "Domain {} has {} active operators",
                            domain, count
                        )));
                    }
                }
                
                Ok(SafeStateStatus::Safe)
            },
            SafeStateStrategy::Custom(ref name) => {
                // Run custom check function
                let checks = self.custom_checks.read().map_err(|_| {
                    Error::Migration("Failed to acquire read lock on custom checks".to_string())
                })?;
                
                if let Some(check_fn) = checks.get(name) {
                    match check_fn(domain) {
                        Ok(true) => Ok(SafeStateStatus::Safe),
                        Ok(false) => Ok(SafeStateStatus::Unsafe(format!(
                            "Custom check '{}' failed for domain {}",
                            name, domain
                        ))),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(Error::Migration(format!(
                        "No custom check function registered for '{}'",
                        name
                    )))
                }
            },
        }
    }
    
    /// Check if all domains are in a safe state
    pub fn check_all_domains(&self, domains: &[String]) -> Result<HashMap<String, SafeStateStatus>> {
        let mut results = HashMap::new();
        
        for domain in domains {
            let start = Instant::now();
            let timeout = self.options.timeout;
            
            // Check with timeout
            let result = std::thread::spawn(move || {
                self.check_domain(domain)
            })
            .join()
            .unwrap_or_else(|_| {
                Err(Error::Migration("Safe state check thread panicked".to_string()))
            });
            
            // Check for timeout
            if start.elapsed() > timeout {
                results.insert(domain.clone(), SafeStateStatus::Timeout);
            } else if let Ok(status) = result {
                results.insert(domain.clone(), status);
            } else {
                results.insert(
                    domain.clone(),
                    SafeStateStatus::Unsafe("Check failed".to_string()),
                );
            }
        }
        
        Ok(results)
    }
    
    /// Wait until a domain is in a safe state
    pub fn wait_until_safe(&self, domain: &str) -> Result<SafeStateStatus> {
        if !self.options.block_until_safe {
            return self.check_domain(domain);
        }
        
        let start = Instant::now();
        let max_duration = self.options.max_block_duration.unwrap_or(Duration::from_secs(60));
        
        loop {
            let status = self.check_domain(domain)?;
            
            match status {
                SafeStateStatus::Safe => return Ok(status),
                _ => {
                    // Check if we've exceeded max duration
                    if start.elapsed() > max_duration {
                        return Ok(SafeStateStatus::Timeout);
                    }
                    
                    // Sleep for a bit before retrying
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
    
    /// Check if it's safe to perform a schema migration
    pub fn can_migrate(&self, schema: &Schema) -> Result<bool> {
        // First check if the schema's domain is in a safe state
        let domain = schema.name.clone();
        
        match self.check_domain(&domain)? {
            SafeStateStatus::Safe => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// Ensure it's safe to perform a schema migration, blocking if necessary
    pub fn ensure_can_migrate(&self, schema: &Schema) -> Result<()> {
        let domain = schema.name.clone();
        
        match self.wait_until_safe(&domain)? {
            SafeStateStatus::Safe => Ok(()),
            SafeStateStatus::Unsafe(reason) => Err(Error::Migration(format!(
                "Cannot migrate schema '{}': {}",
                schema.name, reason
            ))),
            SafeStateStatus::Timeout => Err(Error::Migration(format!(
                "Timeout waiting for safe state for schema '{}'",
                schema.name
            ))),
        }
    }
}

/// A shared safe state manager that can be used throughout the application
pub struct SharedSafeStateManager {
    /// The shared manager
    manager: Arc<SafeStateManager>,
}

impl SharedSafeStateManager {
    /// Create a new shared safe state manager
    pub fn new(options: SafeStateOptions) -> Self {
        SharedSafeStateManager {
            manager: Arc::new(SafeStateManager::new(options)),
        }
    }
    
    /// Get a reference to the manager
    pub fn manager(&self) -> Arc<SafeStateManager> {
        self.manager.clone()
    }
}

impl Default for SharedSafeStateManager {
    fn default() -> Self {
        Self::new(SafeStateOptions::default())
    }
}

/// A transaction for schema updates that ensures safe state
pub struct SchemaTransaction<'a> {
    /// The schema being updated
    schema: &'a mut Schema,
    /// The safe state manager
    safe_state: &'a SafeStateManager,
    /// Whether the transaction has been committed
    committed: bool,
    /// The original schema state
    original_state: Schema,
}

impl<'a> SchemaTransaction<'a> {
    /// Create a new schema transaction
    pub fn new(schema: &'a mut Schema, safe_state: &'a SafeStateManager) -> Result<Self> {
        // Create a clone of the schema for rollback
        let original_state = schema.clone();
        
        // Ensure it's safe to start a transaction
        safe_state.ensure_can_migrate(schema)?;
        
        Ok(SchemaTransaction {
            schema,
            safe_state,
            committed: false,
            original_state,
        })
    }
    
    /// Get a reference to the schema
    pub fn schema(&self) -> &Schema {
        self.schema
    }
    
    /// Get a mutable reference to the schema
    pub fn schema_mut(&mut self) -> &mut Schema {
        self.schema
    }
    
    /// Commit the transaction
    pub fn commit(mut self) -> Result<()> {
        // Check if it's still safe to commit
        self.safe_state.ensure_can_migrate(self.schema)?;
        
        // Mark as committed
        self.committed = true;
        
        Ok(())
    }
    
    /// Validate that the schema is still valid after changes
    pub fn validate(&self) -> Result<()> {
        // Simple validation: make sure the schema has at least one field
        if self.schema.fields.is_empty() {
            return Err(Error::Validation(
                "Schema must have at least one field".to_string()
            ));
        }
        
        Ok(())
    }
}

impl<'a> Drop for SchemaTransaction<'a> {
    fn drop(&mut self) {
        // If not committed, roll back changes
        if !self.committed {
            *self.schema = self.original_state.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_state_manager() -> Result<()> {
        // Create a safe state manager
        let options = SafeStateOptions {
            strategy: SafeStateStrategy::NoInFlightOperations,
            ..Default::default()
        };
        let manager = SafeStateManager::new(options);
        
        // Create a test domain
        let domain = "test_domain";
        
        // Initially, there should be no in-flight operations
        assert_eq!(manager.check_domain(domain)?, SafeStateStatus::Safe);
        
        // Add an in-flight operation
        manager.increment_in_flight(domain)?;
        
        // Now there should be an in-flight operation
        assert!(matches!(manager.check_domain(domain)?, SafeStateStatus::Unsafe(_)));
        
        // Remove the in-flight operation
        manager.decrement_in_flight(domain)?;
        
        // Now it should be safe again
        assert_eq!(manager.check_domain(domain)?, SafeStateStatus::Safe);
        
        Ok(())
    }
    
    #[test]
    fn test_schema_transaction() -> Result<()> {
        // Create a safe state manager
        let options = SafeStateOptions {
            strategy: SafeStateStrategy::None, // Always safe
            ..Default::default()
        };
        let manager = SafeStateManager::new(options);
        
        // Create a schema
        let mut schema = Schema::new("TestSchema", "1.0.0")?;
        schema.add_field(crate::schema::SchemaField::new(
            "name",
            crate::schema::SchemaType::String,
            true,
        ));
        
        // Create a transaction
        let mut transaction = SchemaTransaction::new(&mut schema, &manager)?;
        
        // Modify the schema
        transaction.schema_mut().add_field(crate::schema::SchemaField::new(
            "age",
            crate::schema::SchemaType::Integer,
            true,
        ));
        
        // Commit the transaction
        transaction.commit()?;
        
        // Verify the changes were applied
        assert!(schema.fields.contains_key("age"));
        
        Ok(())
    }
    
    #[test]
    fn test_transaction_rollback() -> Result<()> {
        // Create a safe state manager
        let options = SafeStateOptions {
            strategy: SafeStateStrategy::None, // Always safe
            ..Default::default()
        };
        let manager = SafeStateManager::new(options);
        
        // Create a schema
        let mut schema = Schema::new("TestSchema", "1.0.0")?;
        schema.add_field(crate::schema::SchemaField::new(
            "name",
            crate::schema::SchemaType::String,
            true,
        ));
        
        // Create a transaction without committing
        {
            let mut transaction = SchemaTransaction::new(&mut schema, &manager)?;
            
            // Modify the schema
            transaction.schema_mut().add_field(crate::schema::SchemaField::new(
                "age",
                crate::schema::SchemaType::Integer,
                true,
            ));
            
            // Transaction will be dropped without committing
        }
        
        // Verify the changes were rolled back
        assert!(!schema.fields.contains_key("age"));
        
        Ok(())
    }
} 