//! Tests for the invariant checking system

#[cfg(test)]
mod tests {
    use crate::invariant::{InvariantObserver, NoNegativeBalancesChecker, InvariantChecker, InvariantResult};
    use crate::replay::{LogEntry, LogEntryType, log_helpers};
    use crate::scenario::InvariantConfig;
    use crate::agent::agent_id;
    use crate::observer::Observer;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use serde_json::json;
    use chrono::Utc;

    #[test]
    fn test_no_negative_balances_invariant() {
        // Create a checker
        let checker = NoNegativeBalancesChecker::new();
        
        // Create test entries
        
        // 1. Valid transaction - positive balance
        let mut metadata1 = HashMap::new();
        metadata1.insert("event_type".to_string(), "transaction".to_string());
        metadata1.insert("asset".to_string(), "ETH".to_string());
        metadata1.insert("amount".to_string(), "10".to_string());
        
        let entry1 = LogEntry {
            id: "test-entry-1".to_string(),
            timestamp: Utc::now(),
            entry_type: LogEntryType::DomainEvent,
            agent_id: Some(agent_id::from_string("alice")),
            domain: Some("ethereum".to_string()),
            payload: json!({"sender": "alice", "receiver": "bob", "value": 10}),
            parent_id: None,
            run_id: Some("test-run".to_string()),
            metadata: metadata1,
            content_hash: "test-hash-1".to_string(),
        };
        
        // 2. Invalid transaction - negative balance
        let mut metadata2 = HashMap::new();
        metadata2.insert("event_type".to_string(), "transaction".to_string());
        metadata2.insert("asset".to_string(), "ETH".to_string());
        metadata2.insert("amount".to_string(), "-150".to_string());
        
        let entry2 = LogEntry {
            id: "test-entry-2".to_string(),
            timestamp: Utc::now(),
            entry_type: LogEntryType::DomainEvent,
            agent_id: Some(agent_id::from_string("alice")),
            domain: Some("ethereum".to_string()),
            payload: json!({"sender": "alice", "receiver": "bob", "value": -150}),
            parent_id: None,
            run_id: Some("test-run".to_string()),
            metadata: metadata2,
            content_hash: "test-hash-2".to_string(),
        };
        
        // Create agent state entries
        
        // 3. Valid agent state - positive balances
        let entry3 = log_helpers::create_agent_state(
            agent_id::from_string("alice"),
            json!({"balances": {"ETH": 100, "USDC": 500}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // 4. Invalid agent state - negative balance
        let entry4 = log_helpers::create_agent_state(
            agent_id::from_string("bob"),
            json!({"balances": {"ETH": -50, "USDC": 200}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // Check valid entries
        let result1 = checker.check(&entry1);
        let result3 = checker.check(&entry3);
        
        // Check invalid entries
        let result2 = checker.check(&entry2);
        let result4 = checker.check(&entry4);
        
        // Assert results
        assert!(matches!(result1, InvariantResult::Satisfied));
        assert!(matches!(result3, InvariantResult::Satisfied));
        
        assert!(matches!(result2, InvariantResult::Violated { .. }));
        assert!(matches!(result4, InvariantResult::Violated { .. }));
    }
    
    #[test]
    fn test_invariant_observer() {
        // Create an observer with the no negative balances checker
        let mut observer = InvariantObserver::new();
        observer.add_checker(Box::new(NoNegativeBalancesChecker::new()));
        
        // Track violation messages
        let violations = Arc::new(Mutex::new(Vec::<String>::new()));
        let violations_clone = violations.clone();
        
        observer.set_violation_callback(move |result| {
            if let InvariantResult::Violated { message, .. } = result {
                violations_clone.lock().unwrap().push(message);
            }
        });
        
        // Create a valid entry
        let valid_entry = log_helpers::create_agent_state(
            agent_id::from_string("alice"),
            json!({"balances": {"ETH": 100, "USDC": 500}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // Create an invalid entry
        let invalid_entry = log_helpers::create_agent_state(
            agent_id::from_string("bob"),
            json!({"balances": {"ETH": -50, "USDC": 200}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // Create a clone for later use
        let invalid_entry_clone = invalid_entry.clone();
        
        // Process entries
        observer.on_log_entry(valid_entry);
        observer.on_log_entry(invalid_entry);
        
        // Check violation count
        assert_eq!(observer.violation_count(), 1);
        
        // Check violation messages
        let messages = violations.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("negative balance"));
    }
    
    #[test]
    fn test_invariant_observer_from_config() {
        // Create config with invariants enabled
        let config = InvariantConfig {
            no_negative_balances: Some(true),
        };
        
        // Create observer from config
        let observer = InvariantObserver::from_config(&config);
        
        // Create a valid entry
        let valid_entry = log_helpers::create_agent_state(
            agent_id::from_string("alice"),
            json!({"balances": {"ETH": 100, "USDC": 500}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // Create an invalid entry
        let invalid_entry = log_helpers::create_agent_state(
            agent_id::from_string("bob"),
            json!({"balances": {"ETH": -50, "USDC": 200}}),
            Some("test-run".to_string()),
        ).unwrap();
        
        // Create a clone for later use
        let invalid_entry_clone = invalid_entry.clone();
        
        // Process entries
        observer.on_log_entry(valid_entry);
        observer.on_log_entry(invalid_entry);
        
        // Check violation count
        assert_eq!(observer.violation_count(), 1);
        
        // Test with invariant disabled
        let config2 = InvariantConfig {
            no_negative_balances: Some(false),
        };
        
        let observer2 = InvariantObserver::from_config(&config2);
        observer2.on_log_entry(invalid_entry_clone);
        
        // No violation should be recorded
        assert_eq!(observer2.violation_count(), 0);
    }
} 