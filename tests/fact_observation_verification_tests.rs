// Integration tests for fact observation and verification
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use causality::domain::adapter::{
    DomainAdapter, DomainInfo, DomainStatus, DomainType, FactQuery, TimeMapEntry, Transaction,
    TransactionId, TransactionReceipt, TransactionStatus,
};
use causality::domain::fact::verification::VerificationResult;
use causality::domain::fact::verifiers::{
    MerkleProofVerifier, SignatureVerifier, VerifierRegistry,
};
use causality::error::{Error, Result};
use causality::log::fact_types::{FactType, RegisterFact};
use causality::types::{BlockHash, BlockHeight, DomainId, Timestamp};

// Mock domain adapter for testing fact observation
#[derive(Debug)]
struct MockAdapter {
    domain_id: DomainId,
    facts: Arc<Mutex<HashMap<String, FactType>>>,
}

impl MockAdapter {
    fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            facts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn register_fact(&self, fact_type: &str, fact: FactType) {
        let mut facts = self.facts.lock().unwrap();
        facts.insert(fact_type.to_string(), fact);
    }
}

#[async_trait]
impl DomainAdapter for MockAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }

    async fn domain_info(&self) -> Result<DomainInfo> {
        Ok(DomainInfo {
            id: self.domain_id.clone(),
            domain_type: DomainType::Custom,
            name: "Mock Domain".to_string(),
            description: Some("Test domain for fact verification".to_string()),
            rpc_url: None,
            explorer_url: None,
            chain_id: None,
            native_currency: None,
            status: DomainStatus::Active,
            metadata: HashMap::new(),
        })
    }

    async fn current_height(&self) -> Result<BlockHeight> {
        Ok(BlockHeight::new(1000))
    }

    async fn current_hash(&self) -> Result<BlockHash> {
        Ok(BlockHash::new(vec![1, 2, 3, 4]))
    }

    async fn current_timestamp(&self) -> Result<Timestamp> {
        Ok(Timestamp::new(1600000000))
    }

    async fn observe_fact(&self, query: FactQuery) -> Result<FactType> {
        let facts = self
            .facts
            .lock()
            .map_err(|_| Error::ConcurrencyError("Failed to acquire lock".to_string()))?;

        if let Some(fact) = facts.get(&query.fact_type) {
            return Ok(fact.clone());
        }

        // Create default facts for testing
        match query.fact_type.as_str() {
            "balance" => Ok(FactType::BalanceFact {
                domain_id: self.domain_id.clone(),
                address: query.parameters.get("address").cloned().unwrap_or_default(),
                amount: "1000".to_string(),
                token: None,
                block_height: Some(1000),
                block_hash: Some(vec![1, 2, 3, 4]),
                timestamp: Some(1600000000),
                proof_data: Some(vec![5, 6, 7, 8]),
                metadata: query.parameters.clone(),
            }),
            "register_create" => {
                let register_id = query
                    .parameters
                    .get("register_id")
                    .cloned()
                    .unwrap_or_else(|| "test-register".to_string());
                let owner = query
                    .parameters
                    .get("owner")
                    .cloned()
                    .unwrap_or_else(|| "test-owner".to_string());

                Ok(FactType::RegisterFact(RegisterFact::RegisterCreation {
                    domain_id: self.domain_id.clone(),
                    register_id,
                    owner,
                    register_type: Some("token".to_string()),
                    initial_value: Some("100".to_string()),
                    block_height: Some(1000),
                    block_hash: Some(vec![1, 2, 3, 4]),
                    timestamp: Some(1600000000),
                    proof_data: Some(vec![5, 6, 7, 8]),
                    metadata: query.parameters.clone(),
                }))
            }
            "register_update" => {
                let register_id = query
                    .parameters
                    .get("register_id")
                    .cloned()
                    .unwrap_or_else(|| "test-register".to_string());

                Ok(FactType::RegisterFact(RegisterFact::RegisterUpdate {
                    domain_id: self.domain_id.clone(),
                    register_id,
                    new_value: "200".to_string(),
                    previous_value: Some("100".to_string()),
                    updater: Some("test-owner".to_string()),
                    block_height: Some(1000),
                    block_hash: Some(vec![1, 2, 3, 4]),
                    timestamp: Some(1600000000),
                    proof_data: Some(vec![5, 6, 7, 8]),
                    metadata: query.parameters.clone(),
                }))
            }
            "register_transfer" => {
                let register_id = query
                    .parameters
                    .get("register_id")
                    .cloned()
                    .unwrap_or_else(|| "test-register".to_string());

                Ok(FactType::RegisterFact(RegisterFact::RegisterTransfer {
                    domain_id: self.domain_id.clone(),
                    register_id,
                    from: "test-owner".to_string(),
                    to: "new-owner".to_string(),
                    block_height: Some(1000),
                    block_hash: Some(vec![1, 2, 3, 4]),
                    timestamp: Some(1600000000),
                    proof_data: Some(vec![5, 6, 7, 8]),
                    metadata: query.parameters.clone(),
                }))
            }
            "block" => Ok(FactType::BlockFact {
                domain_id: self.domain_id.clone(),
                height: Some(1000),
                hash: Some(vec![1, 2, 3, 4]),
                parent_hash: Some(vec![0, 1, 2, 3]),
                timestamp: 1600000000,
                metadata: query.parameters.clone(),
            }),
            _ => Err(Error::UnsupportedFactType(query.fact_type)),
        }
    }

    async fn submit_transaction(&self, _tx: Transaction) -> Result<TransactionId> {
        Ok(TransactionId::new("test-tx-id".to_string()))
    }

    async fn get_transaction_receipt(&self, _tx_id: &TransactionId) -> Result<TransactionReceipt> {
        Ok(TransactionReceipt {
            tx_id: TransactionId::new("test-tx-id".to_string()),
            domain_id: self.domain_id.clone(),
            block_height: Some(BlockHeight::new(1000)),
            block_hash: Some(BlockHash::new(vec![1, 2, 3, 4])),
            timestamp: Some(Timestamp::new(1600000000)),
            status: TransactionStatus::Success,
            error: None,
            gas_used: Some(21000),
            metadata: HashMap::new(),
        })
    }

    async fn get_time_map(&self) -> Result<TimeMapEntry> {
        Ok(TimeMapEntry::new(
            self.domain_id.clone(),
            BlockHeight::new(1000),
            BlockHash::new(vec![1, 2, 3, 4]),
            Timestamp::new(1600000000),
        ))
    }

    async fn verify_block(&self, _height: BlockHeight, _hash: &BlockHash) -> Result<bool> {
        Ok(true)
    }

    async fn check_connectivity(&self) -> Result<bool> {
        Ok(true)
    }
}

// Mock verifier for testing
#[derive(Debug)]
struct MockVerifier {
    id: String,
    verification_result: Arc<Mutex<bool>>,
}

impl MockVerifier {
    fn new(id: &str, result: bool) -> Self {
        Self {
            id: id.to_string(),
            verification_result: Arc::new(Mutex::new(result)),
        }
    }

    fn set_verification_result(&self, result: bool) {
        let mut verification_result = self.verification_result.lock().unwrap();
        *verification_result = result;
    }
}

#[async_trait]
impl causality::domain::fact::verifiers::FactVerifier for MockVerifier {
    async fn verify(&self, _fact: &FactType) -> VerificationResult {
        let result = *self.verification_result.lock().unwrap();

        if result {
            VerificationResult {
                verified: true,
                confidence: 1.0,
                method: Some(self.id.clone()),
                metadata: HashMap::new(),
            }
        } else {
            VerificationResult {
                verified: false,
                confidence: 0.0,
                method: Some(self.id.clone()),
                metadata: HashMap::new(),
            }
        }
    }

    fn supports_fact_type(&self, _fact_type: &FactType) -> bool {
        true // Support all fact types for testing
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[tokio::test]
async fn test_fact_observation() -> Result<()> {
    // Create a mock adapter
    let domain_id = DomainId::new("test-domain");
    let adapter = MockAdapter::new(domain_id.clone());

    // Create fact queries
    let balance_query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "balance".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("address".to_string(), "test-address".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let register_create_query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "register_create".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("register_id".to_string(), "test-register".to_string());
            params.insert("owner".to_string(), "test-owner".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    // Observe facts
    let balance_fact = adapter.observe_fact(balance_query).await?;
    let register_fact = adapter.observe_fact(register_create_query).await?;

    // Verify balance fact
    match balance_fact {
        FactType::BalanceFact {
            address, amount, ..
        } => {
            assert_eq!(address, "test-address");
            assert_eq!(amount, "1000");
        }
        _ => panic!("Expected BalanceFact, got: {:?}", balance_fact),
    }

    // Verify register fact
    match register_fact {
        FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id, owner, ..
        }) => {
            assert_eq!(register_id, "test-register");
            assert_eq!(owner, "test-owner");
        }
        _ => panic!(
            "Expected RegisterFact::RegisterCreation, got: {:?}",
            register_fact
        ),
    }

    Ok(())
}

#[tokio::test]
async fn test_fact_verification() -> Result<()> {
    // Create a mock adapter
    let domain_id = DomainId::new("test-domain");
    let adapter = MockAdapter::new(domain_id.clone());

    // Create verifier registry
    let mut registry = VerifierRegistry::new();

    // Add mock verifiers
    let mock_verifier = Arc::new(MockVerifier::new("mock-verifier", true));
    registry.register_verifier(mock_verifier.clone());

    // Create fact query
    let query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "register_create".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("register_id".to_string(), "test-register".to_string());
            params.insert("owner".to_string(), "test-owner".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    // Observe fact
    let fact = adapter.observe_fact(query).await?;

    // Verify fact
    let verification_result = registry.verify(&fact).await;
    assert!(verification_result.verified);
    assert_eq!(verification_result.confidence, 1.0);

    // Change verification result and try again
    mock_verifier.set_verification_result(false);
    let verification_result = registry.verify(&fact).await;
    assert!(!verification_result.verified);
    assert_eq!(verification_result.confidence, 0.0);

    Ok(())
}

#[tokio::test]
async fn test_register_fact_lifecycle() -> Result<()> {
    // Create a mock adapter
    let domain_id = DomainId::new("test-domain");
    let adapter = MockAdapter::new(domain_id.clone());

    // Register ID and parameters
    let register_id = "test-register-lifecycle";
    let owner = "test-owner";
    let new_owner = "new-owner";

    // Create register
    let create_query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "register_create".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("register_id".to_string(), register_id.to_string());
            params.insert("owner".to_string(), owner.to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let create_fact = adapter.observe_fact(create_query).await?;

    // Update register
    let update_query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "register_update".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("register_id".to_string(), register_id.to_string());
            params.insert("new_value".to_string(), "200".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let update_fact = adapter.observe_fact(update_query).await?;

    // Transfer register
    let transfer_query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "register_transfer".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("register_id".to_string(), register_id.to_string());
            params.insert("from".to_string(), owner.to_string());
            params.insert("to".to_string(), new_owner.to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let transfer_fact = adapter.observe_fact(transfer_query).await?;

    // Verify the full lifecycle
    match create_fact {
        FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id: r_id,
            owner: r_owner,
            ..
        }) => {
            assert_eq!(r_id, register_id);
            assert_eq!(r_owner, owner);
        }
        _ => panic!(
            "Expected RegisterFact::RegisterCreation, got: {:?}",
            create_fact
        ),
    }

    match update_fact {
        FactType::RegisterFact(RegisterFact::RegisterUpdate {
            register_id: r_id,
            new_value,
            ..
        }) => {
            assert_eq!(r_id, register_id);
            assert_eq!(new_value, "200");
        }
        _ => panic!(
            "Expected RegisterFact::RegisterUpdate, got: {:?}",
            update_fact
        ),
    }

    match transfer_fact {
        FactType::RegisterFact(RegisterFact::RegisterTransfer {
            register_id: r_id,
            from,
            to,
            ..
        }) => {
            assert_eq!(r_id, register_id);
            assert_eq!(from, owner);
            assert_eq!(to, new_owner);
        }
        _ => panic!(
            "Expected RegisterFact::RegisterTransfer, got: {:?}",
            transfer_fact
        ),
    }

    Ok(())
}
