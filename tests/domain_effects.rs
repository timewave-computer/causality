#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{DomainId, DomainType},
        effect::{Effect, EffectId, EffectStatus},
        time::{TimeMap, TimePoint},
    };

    // ... existing test cases ...

    #[test]
    fn test_cross_domain_operations() {
        let mut registry = DomainRegistry::new();
        
        // Register multiple domains
        let evm_domain = DomainId::new("evm_chain");
        let cosmwasm_domain = DomainId::new("cosmwasm_chain");
        let zk_domain = DomainId::new("zk_chain");
        
        registry.register_domain(evm_domain, DomainType::Evm).unwrap();
        registry.register_domain(cosmwasm_domain, DomainType::CosmWasm).unwrap();
        registry.register_domain(zk_domain, DomainType::Zk).unwrap();
        
        // Create cross-domain operation
        let operation_id = registry.create_cross_domain_operation(
            vec![evm_domain, cosmwasm_domain, zk_domain],
            300, // 5 minute timeout
        ).unwrap();
        
        // Submit transactions for each domain
        let evm_tx = Effect::new(
            EffectId::new("evm_tx"),
            evm_domain,
            EffectStatus::Pending,
            TimePoint::now(),
        );
        let cosmwasm_tx = Effect::new(
            EffectId::new("cosmwasm_tx"),
            cosmwasm_domain,
            EffectStatus::Pending,
            TimePoint::now(),
        );
        let zk_tx = Effect::new(
            EffectId::new("zk_tx"),
            zk_domain,
            EffectStatus::Pending,
            TimePoint::now(),
        );
        
        // Mark domains as ready
        registry.set_domain_ready(operation_id, evm_domain, evm_tx.id).unwrap();
        registry.set_domain_ready(operation_id, cosmwasm_domain, cosmwasm_tx.id).unwrap();
        registry.set_domain_ready(operation_id, zk_domain, zk_tx.id).unwrap();
        
        // Verify operation is committed
        let operation = registry.get_operation(operation_id).unwrap();
        assert_eq!(operation.status, OperationStatus::Committed);
        
        // Verify all transactions are executed
        assert_eq!(registry.get_effect_status(evm_tx.id), EffectStatus::Executed);
        assert_eq!(registry.get_effect_status(cosmwasm_tx.id), EffectStatus::Executed);
        assert_eq!(registry.get_effect_status(zk_tx.id), EffectStatus::Executed);
    }
    
    #[test]
    fn test_cross_domain_operation_timeout() {
        let mut registry = DomainRegistry::new();
        
        // Register domains
        let evm_domain = DomainId::new("evm_chain");
        let cosmwasm_domain = DomainId::new("cosmwasm_chain");
        
        registry.register_domain(evm_domain, DomainType::Evm).unwrap();
        registry.register_domain(cosmwasm_domain, DomainType::CosmWasm).unwrap();
        
        // Create operation with short timeout
        let operation_id = registry.create_cross_domain_operation(
            vec![evm_domain, cosmwasm_domain],
            1, // 1 second timeout
        ).unwrap();
        
        // Submit first transaction
        let evm_tx = Effect::new(
            EffectId::new("evm_tx"),
            evm_domain,
            EffectStatus::Pending,
            TimePoint::now(),
        );
        registry.set_domain_ready(operation_id, evm_domain, evm_tx.id).unwrap();
        
        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // Verify operation is aborted
        let operation = registry.get_operation(operation_id).unwrap();
        assert_eq!(operation.status, OperationStatus::Aborted);
        
        // Verify transaction is rolled back
        assert_eq!(registry.get_effect_status(evm_tx.id), EffectStatus::Failed);
    }
    
    #[test]
    fn test_error_handling() {
        let mut registry = DomainRegistry::new();
        
        // Test invalid domain registration
        let result = registry.register_domain(DomainId::new(""), DomainType::Evm);
        assert!(result.is_err());
        
        // Test duplicate domain registration
        let domain_id = DomainId::new("test_chain");
        registry.register_domain(domain_id, DomainType::Evm).unwrap();
        let result = registry.register_domain(domain_id, DomainType::Evm);
        assert!(result.is_err());
        
        // Test invalid operation creation
        let result = registry.create_cross_domain_operation(vec![], 300);
        assert!(result.is_err());
        
        // Test invalid domain in operation
        let result = registry.create_cross_domain_operation(
            vec![DomainId::new("nonexistent")],
            300,
        );
        assert!(result.is_err());
        
        // Test invalid effect submission
        let result = registry.submit_effect(
            EffectId::new("test"),
            DomainId::new("nonexistent"),
            EffectStatus::Pending,
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_performance_benchmarks() {
        let mut registry = DomainRegistry::new();
        
        // Register test domain
        let domain_id = DomainId::new("test_chain");
        registry.register_domain(domain_id, DomainType::Evm).unwrap();
        
        // Measure effect submission performance
        let start = std::time::Instant::now();
        for i in 0..1000 {
            let effect_id = EffectId::new(&format!("effect_{}", i));
            registry.submit_effect(effect_id, domain_id, EffectStatus::Pending).unwrap();
        }
        let submission_duration = start.elapsed();
        
        // Measure effect status lookup performance
        let start = std::time::Instant::now();
        for i in 0..1000 {
            let effect_id = EffectId::new(&format!("effect_{}", i));
            registry.get_effect_status(effect_id);
        }
        let lookup_duration = start.elapsed();
        
        // Measure operation creation performance
        let start = std::time::Instant::now();
        for i in 0..100 {
            let operation_id = registry.create_cross_domain_operation(
                vec![domain_id],
                300,
            ).unwrap();
            
            let effect_id = EffectId::new(&format!("op_effect_{}", i));
            registry.set_domain_ready(operation_id, domain_id, effect_id).unwrap();
        }
        let operation_duration = start.elapsed();
        
        // Log performance metrics
        println!("Effect submission: {:?}", submission_duration);
        println!("Effect lookup: {:?}", lookup_duration);
        println!("Operation creation: {:?}", operation_duration);
        
        // Verify performance meets requirements
        assert!(submission_duration.as_millis() < 5000); // 5 seconds for 1000 submissions
        assert!(lookup_duration.as_millis() < 1000); // 1 second for 1000 lookups
        assert!(operation_duration.as_millis() < 10000); // 10 seconds for 100 operations
    }
} 