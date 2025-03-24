// Register Fact Observer Module
//
// This module provides functionality to observe register operations
// and log them as facts in the causality unified log system.

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::log::fact::{FactLogger, FactMetadata};
use crate::log::fact_types::{FactType, RegisterFact};
use crate::resource::register::{Register, RegisterId, RegisterState, OperationType};
use crate::types::{TraceId, DomainId, BlockHeight};

/// Observer for register operations that logs them as facts
pub struct RegisterFactObserver {
    /// The fact logger used to log register facts
    logger: Arc<FactLogger>,
    /// The observer name
    observer_name: String,
    /// The domain ID for this observer
    domain_id: DomainId,
}

impl RegisterFactObserver {
    /// Create a new register fact observer
    pub fn new(logger: Arc<FactLogger>, observer_name: String, domain_id: DomainId) -> Self {
        Self {
            logger,
            observer_name,
            domain_id,
        }
    }
    
    /// Observe register creation
    pub fn observe_register_creation(
        &self,
        register: &Register,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterCreation {
            register_id: register.register_id.clone(),
            initial_data: register.contents.as_bytes().to_vec(),
            owner: register.owner.to_string(),
            domain: register.domain.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-creation".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:creation:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register update
    pub fn observe_register_update(
        &self,
        register: &Register,
        previous_data: &[u8],
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterUpdate {
            register_id: register.register_id.clone(),
            new_data: register.contents.as_bytes().to_vec(),
            previous_version: hex::encode(previous_data),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-update".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:update:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register consumption
    pub fn observe_register_consumption(
        &self,
        register: &Register,
        transaction_id: &str,
        nullifier: &str,
        successors: Vec<RegisterId>,
        block_height: BlockHeight,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterConsumption {
            register_id: register.register_id.clone(),
            transaction_id: transaction_id.to_string(),
            nullifier: nullifier.to_string(),
            successors: successors.clone(),
            block_height,
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-consumption".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:consumption:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register state change
    pub fn observe_register_state_change(
        &self,
        register: &Register,
        previous_state: RegisterState,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterStateChange {
            register_id: register.register_id.clone(),
            previous_state,
            new_state: register.state.clone(),
            reason: reason.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-state-change".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:state_change:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register ownership transfer
    pub fn observe_register_ownership_transfer(
        &self,
        register: &Register,
        previous_owner: &str,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterOwnershipTransfer {
            register_id: register.register_id.clone(),
            previous_owner: previous_owner.to_string(),
            new_owner: register.owner.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-ownership-transfer".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:ownership_transfer:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register lock
    pub fn observe_register_lock(
        &self,
        register: &Register,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterLock {
            register_id: register.register_id.clone(),
            reason: reason.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-lock".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:lock:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register unlock
    pub fn observe_register_unlock(
        &self,
        register: &Register,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterUnlock {
            register_id: register.register_id.clone(),
            reason: reason.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-unlock".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:unlock:{}", register.register_id),
            Some(register.register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe register nullifier creation
    pub fn observe_register_nullifier_creation(
        &self,
        register_id: &RegisterId,
        nullifier: &str,
        block_height: BlockHeight,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact
        let register_fact = RegisterFact::RegisterNullifierCreation {
            register_id: register_id.clone(),
            nullifier: nullifier.to_string(),
            block_height,
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-nullifier-creation".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:nullifier_creation:{}", register_id),
            Some(register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
    
    /// Observe domain integration for a register
    pub fn observe_register_domain_integration(
        &self,
        register_id: &RegisterId,
        domain_id: &DomainId,
        transaction_id: &str,
    ) -> Result<()> {
        // Create a trace ID for this observation
        let trace_id = TraceId::from_str(transaction_id);
        
        // Create register fact for domain transfer as an approximation
        let register_fact = RegisterFact::RegisterTransfer {
            register_id: register_id.clone(),
            source_domain: "register-system".to_string(),
            target_domain: domain_id.to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        // Create metadata
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("register-domain-integration".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:domain_integration:{}", register_id),
            Some(register_id.clone()),
            &fact_type,
            Some(metadata),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::log::MemoryLogStorage;
    use crate::resource::register::{RegisterContents};
    
    fn create_test_register() -> Register {
        Register::new(
            RegisterId::new_unique(),
            crate::types::Address::new("owner1"),
            crate::types::Domain::new("domain1"),
            RegisterContents::with_string("test contents"),
            std::collections::HashMap::new(),
            1000,
            1000,
        )
    }
    
    fn create_test_observer() -> RegisterFactObserver {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let domain_id = DomainId::new("test-domain");
        let logger = Arc::new(FactLogger::new(storage, domain_id.clone()));
        
        RegisterFactObserver::new(
            logger,
            "test-observer".to_string(),
            domain_id,
        )
    }
    
    #[test]
    fn test_observe_register_creation() {
        let observer = create_test_observer();
        let register = create_test_register();
        
        let result = observer.observe_register_creation(&register, "test-tx");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_observe_register_consumption() {
        let observer = create_test_observer();
        let register = create_test_register();
        let nullifier = "test-nullifier";
        let block_height = 1000;
        
        let result = observer.observe_register_consumption(
            &register,
            "test-tx",
            nullifier,
            Vec::new(),
            block_height,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_observe_register_state_change() {
        let observer = create_test_observer();
        let register = create_test_register();
        
        let result = observer.observe_register_state_change(
            &register,
            RegisterState::Active,
            "Testing state change",
            "test-tx",
        );
        assert!(result.is_ok());
    }
} 
