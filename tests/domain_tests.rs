use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use tokio::time::sleep;

use causality::domain::adapter::{
    DomainAdapter, DomainInfo, DomainStatus, DomainType, FactProof, FactQuery, ObservedFact,
    ProofType,
};
use causality::domain::{
    DomainId, DomainRegistry, DomainSelector, SelectionCriteria, TimeMap, TimeMapEntry,
    Transaction, TransactionId, TransactionReceipt, TransactionStatus,
};
use causality::error::{Error, Result};
use causality::types::{BlockHash, BlockHeight, Timestamp};

/// Creates a test domain ID with a simple numeric identifier
fn create_test_domain_id(id: u8) -> DomainId {
    DomainId(vec![id])
}

/// Mock domain adapter for testing
struct MockDomainAdapter {
    domain_id: DomainId,
    domain_type: DomainType,
    name: String,
    status: DomainStatus,
    block_height: Arc<Mutex<BlockHeight>>,
    block_hash: Arc<Mutex<BlockHash>>,
    timestamp: Arc<Mutex<Timestamp>>,
    facts: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    connected: Arc<Mutex<bool>>,
}

impl MockDomainAdapter {
    fn new(id: u8, domain_type: DomainType, name: &str) -> Self {
        MockDomainAdapter {
            domain_id: create_test_domain_id(id),
            domain_type,
            name: name.to_string(),
            status: DomainStatus::Active,
            block_height: Arc::new(Mutex::new(BlockHeight(100))),
            block_hash: Arc::new(Mutex::new(BlockHash(
                vec![id, 0, 0, 1].try_into().unwrap_or([0u8; 32]),
            ))),
            timestamp: Arc::new(Mutex::new(Timestamp(1000000))),
            facts: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(Mutex::new(true)),
        }
    }

    fn set_block_height(&self, height: BlockHeight) {
        *self.block_height.lock().unwrap() = height;
    }

    fn set_block_hash(&self, hash: BlockHash) {
        *self.block_hash.lock().unwrap() = hash;
    }

    fn set_timestamp(&self, ts: Timestamp) {
        *self.timestamp.lock().unwrap() = ts;
    }

    fn set_connected(&self, connected: bool) {
        *self.connected.lock().unwrap() = connected;
    }

    fn add_fact(&self, key: &str, value: &[u8]) {
        self.facts
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
    }
}

#[async_trait]
impl DomainAdapter for MockDomainAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }

    async fn domain_info(&self) -> Result<DomainInfo> {
        Ok(DomainInfo {
            id: self.domain_id.clone(),
            domain_type: self.domain_type.clone(),
            name: self.name.clone(),
            description: Some(format!("Mock domain {}", self.name)),
            rpc_url: Some("http://localhost:8545".to_string()),
            explorer_url: None,
            chain_id: Some(1),
            native_currency: Some("ETH".to_string()),
            status: self.status.clone(),
            metadata: HashMap::new(),
        })
    }

    async fn current_height(&self) -> Result<BlockHeight> {
        let height = *self
            .block_height
            .lock()
            .map_err(|_| Error::LockError("Failed to acquire block height lock".to_string()))?;
        Ok(height)
    }

    async fn current_hash(&self) -> Result<BlockHash> {
        let hash = self
            .block_hash
            .lock()
            .map_err(|_| Error::LockError("Failed to acquire block hash lock".to_string()))?
            .clone();
        Ok(hash)
    }

    async fn current_timestamp(&self) -> Result<Timestamp> {
        let ts = *self
            .timestamp
            .lock()
            .map_err(|_| Error::LockError("Failed to acquire timestamp lock".to_string()))?;
        Ok(ts)
    }

    async fn observe_fact(&self, query: FactQuery) -> Result<ObservedFact> {
        let facts = self
            .facts
            .lock()
            .map_err(|_| Error::LockError("Failed to acquire facts lock".to_string()))?;

        let key = match query.fact_type.as_str() {
            "balance" => {
                if let Some(account) = query.parameters.get("account") {
                    format!("balance:{}", account)
                } else {
                    return Err(Error::InvalidArgument(
                        "Missing account parameter".to_string(),
                    ));
                }
            }
            "storage" => {
                if let (Some(contract), Some(slot)) = (
                    query.parameters.get("contract"),
                    query.parameters.get("slot"),
                ) {
                    format!("storage:{}:{}", contract, slot)
                } else {
                    return Err(Error::InvalidArgument(
                        "Missing contract or slot parameter".to_string(),
                    ));
                }
            }
            _ => query.fact_type.clone(),
        };

        let data = facts.get(&key).cloned().unwrap_or_else(|| vec![0]);

        Ok(ObservedFact {
            domain_id: self.domain_id.clone(),
            fact_type: query.fact_type,
            block_height: *self.block_height.lock().unwrap(),
            block_hash: self.block_hash.lock().unwrap().clone(),
            timestamp: *self.timestamp.lock().unwrap(),
            data,
            proof: None,
            metadata: HashMap::new(),
        })
    }

    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> {
        // Simulate transaction submission
        let tx_id = TransactionId(vec![self.domain_id.0[0], 0x01, 0x02, 0x03]);

        // Add a fact based on the transaction
        self.add_fact(&format!("tx:{}", hex::encode(&tx_id.0)), &tx.data);

        Ok(tx_id)
    }

    async fn get_transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt> {
        Ok(TransactionReceipt {
            tx_id: tx_id.clone(),
            domain_id: self.domain_id.clone(),
            block_height: Some(*self.block_height.lock().unwrap()),
            block_hash: Some(self.block_hash.lock().unwrap().clone()),
            timestamp: Some(*self.timestamp.lock().unwrap()),
            status: TransactionStatus::Success,
            error: None,
            gas_used: Some(21000),
            metadata: HashMap::new(),
        })
    }

    async fn get_time_map(&self) -> Result<TimeMapEntry> {
        Ok(TimeMapEntry::new(
            self.domain_id.clone(),
            *self.block_height.lock().unwrap(),
            self.block_hash.lock().unwrap().clone(),
            *self.timestamp.lock().unwrap(),
        ))
    }

    async fn verify_block(&self, height: BlockHeight, _hash: &BlockHash) -> Result<bool> {
        // Simple implementation - just check if the height is less than or equal to current height
        let current_height = *self.block_height.lock().unwrap();
        Ok(height <= current_height)
    }

    async fn check_connectivity(&self) -> Result<bool> {
        Ok(*self.connected.lock().unwrap())
    }
}

#[tokio::test]
async fn test_domain_registry() {
    // Create multiple mock domains
    let eth_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));
    let btc_domain = Arc::new(MockDomainAdapter::new(
        2,
        DomainType::Custom("Bitcoin".into()),
        "Bitcoin",
    ));
    let sol_domain = Arc::new(MockDomainAdapter::new(
        3,
        DomainType::Custom("Solana".into()),
        "Solana",
    ));

    // Create registry and register domains
    let registry = DomainRegistry::new();

    // Register domains
    registry
        .register(eth_domain.clone(), eth_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(btc_domain.clone(), btc_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(sol_domain.clone(), sol_domain.domain_info().await.unwrap())
        .unwrap();

    // Test retrieving domains
    let domains = registry.list_domains();
    assert_eq!(domains.len(), 3);

    // Test getting domain by ID
    let eth_id = create_test_domain_id(1);
    let retrieved_domain = registry.get_domain(&eth_id).unwrap();
    let info = retrieved_domain.domain_info().await.unwrap();
    assert_eq!(info.name, "Ethereum");

    // Test getting domain info
    let all_info = registry.get_all_domain_info().await.unwrap();
    assert_eq!(all_info.len(), 3);
}

#[tokio::test]
async fn test_time_map() {
    // Create a time map and add entries
    let mut time_map = TimeMap::new();

    let domain1 = create_test_domain_id(1);
    let domain2 = create_test_domain_id(2);

    let entry1 = TimeMapEntry::new(domain1.clone(), 100, vec![1, 0, 0, 1], 1000000);

    let entry2 = TimeMapEntry::new(domain2.clone(), 200, vec![2, 0, 0, 2], 2000000);

    time_map.update_domain(entry1);
    time_map.update_domain(entry2);

    // Test basic access methods
    assert_eq!(time_map.domain_count(), 2);
    assert!(time_map.contains_domain(&domain1));
    assert!(time_map.contains_domain(&domain2));

    assert_eq!(time_map.get_height(&domain1), Some(100));
    assert_eq!(time_map.get_height(&domain2), Some(200));

    // Test filtering
    let recent_map = time_map.filter(|entry| entry.height > 150);
    assert_eq!(recent_map.domain_count(), 1);
    assert!(!recent_map.contains_domain(&domain1));
    assert!(recent_map.contains_domain(&domain2));

    // Test merging
    let mut time_map3 = TimeMap::new();
    let domain3 = create_test_domain_id(3);
    let entry3 = TimeMapEntry::new(domain3.clone(), 300, vec![3, 0, 0, 3], 3000000);
    time_map3.update_domain(entry3);

    time_map.merge(&time_map3);
    assert_eq!(time_map.domain_count(), 3);
    assert_eq!(time_map.get_height(&domain3), Some(300));
}

#[tokio::test]
async fn test_observing_facts() {
    // Create a mock domain
    let mock_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));

    // Add some facts
    mock_domain.add_fact("balance:0xuser", &100u64.to_be_bytes());
    mock_domain.add_fact("storage:0xcontract:0", &200u64.to_be_bytes());

    // Test observing facts
    let balance_query = FactQuery {
        domain_id: create_test_domain_id(1),
        fact_type: "balance".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("account".to_string(), "0xuser".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let storage_query = FactQuery {
        domain_id: create_test_domain_id(1),
        fact_type: "storage".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("contract".to_string(), "0xcontract".to_string());
            params.insert("slot".to_string(), "0".to_string());
            params
        },
        block_height: None,
        block_hash: None,
        timestamp: None,
    };

    let balance_fact = mock_domain.observe_fact(balance_query).await.unwrap();
    let balance = u64::from_be_bytes(balance_fact.data.try_into().unwrap());
    assert_eq!(balance, 100);

    let storage_fact = mock_domain.observe_fact(storage_query).await.unwrap();
    let storage_value = u64::from_be_bytes(storage_fact.data.try_into().unwrap());
    assert_eq!(storage_value, 200);
}

#[tokio::test]
async fn test_transaction_submission() {
    // Create a mock domain
    let mock_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));

    // Create a transaction
    let tx = Transaction {
        domain_id: create_test_domain_id(1),
        tx_type: "transfer".to_string(),
        data: vec![1, 2, 3, 4],
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("to".to_string(), "0xrecipient".to_string());
            metadata.insert("amount".to_string(), "100".to_string());
            metadata
        },
    };

    // Submit transaction
    let tx_id = mock_domain.submit_transaction(tx).await.unwrap();

    // Get receipt
    let receipt = mock_domain.get_transaction_receipt(&tx_id).await.unwrap();

    // Verify receipt details
    assert_eq!(receipt.tx_id, tx_id);
    assert_eq!(receipt.domain_id, create_test_domain_id(1));
    assert_eq!(receipt.status, TransactionStatus::Success);
}

#[tokio::test]
async fn test_domain_selection() {
    // Create a domain registry with multiple domains
    let mut registry = DomainRegistry::new();

    let eth_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));
    let btc_domain = Arc::new(MockDomainAdapter::new(
        2,
        DomainType::Custom("Bitcoin".into()),
        "Bitcoin",
    ));
    let sol_domain = Arc::new(MockDomainAdapter::new(
        3,
        DomainType::Custom("Solana".into()),
        "Solana",
    ));

    registry
        .register(eth_domain.clone(), eth_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(btc_domain.clone(), btc_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(sol_domain.clone(), sol_domain.domain_info().await.unwrap())
        .unwrap();

    // Create a shared time map
    let time_map = causality::domain::SharedTimeMap::new();

    // Update time map with domain entries
    for domain in [eth_domain.clone(), btc_domain.clone(), sol_domain.clone()] {
        let entry = domain.get_time_map().await.unwrap();
        time_map.update_domain(entry).unwrap();
    }

    // Create domain selector
    let registry_arc = Arc::new(registry);
    let mut selector = DomainSelector::new(registry_arc, time_map);

    // Update metrics (normally this would be done automatically based on actual metrics)
    for (id, (reliability, latency)) in [
        (create_test_domain_id(1), (0.9, 100)), // ETH: high reliability, medium latency
        (create_test_domain_id(2), (0.95, 200)), // BTC: highest reliability, high latency
        (create_test_domain_id(3), (0.8, 50)),  // SOL: lower reliability, low latency
    ] {
        let metrics = causality::domain::DomainMetrics {
            domain_id: id,
            reliability,
            avg_latency: latency,
            cost_factor: 1.0,
            features: HashSet::new(),
            last_update: Utc::now(),
        };
        selector.update_metrics(metrics);
    }

    // Test selecting with default criteria (should pick highest reliability)
    let result = selector
        .select_domain("transfer", &SelectionCriteria::default())
        .await
        .unwrap();
    assert!(result.is_some());
    let selected = result.unwrap();
    assert_eq!(selected.domain_id, create_test_domain_id(2)); // BTC has highest reliability

    // Test selecting with max latency criteria
    let latency_criteria = SelectionCriteria {
        max_latency: Some(75),
        ..Default::default()
    };
    let result = selector
        .select_domain("transfer", &latency_criteria)
        .await
        .unwrap();
    assert!(result.is_some());
    let selected = result.unwrap();
    assert_eq!(selected.domain_id, create_test_domain_id(3)); // SOL has lowest latency

    // Test selecting with specific domain types
    let mut required_types = HashSet::new();
    required_types.insert(DomainType::EVM);

    let type_criteria = SelectionCriteria {
        required_types: Some(required_types),
        ..Default::default()
    };

    let result = selector
        .select_domain("transfer", &type_criteria)
        .await
        .unwrap();
    assert!(result.is_some());
    let selected = result.unwrap();
    assert_eq!(selected.domain_id, create_test_domain_id(1)); // ETH is the only Ethereum type

    // Test selecting multiple domains
    let result = selector
        .select_multiple_domains("transfer", &SelectionCriteria::default(), 2)
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    // Should be ordered by score (highest first)
    assert_eq!(result[0].domain_id, create_test_domain_id(2)); // BTC (highest reliability)
    assert_eq!(result[1].domain_id, create_test_domain_id(1)); // ETH (second highest reliability)

    // Test domain connectivity affecting selection
    eth_domain.set_connected(false);
    btc_domain.set_connected(false);

    // Now only SOL should be available
    let result = selector
        .select_domain("transfer", &SelectionCriteria::default())
        .await
        .unwrap();
    assert!(result.is_some());
    let selected = result.unwrap();
    assert_eq!(selected.domain_id, create_test_domain_id(3)); // SOL is the only connected domain
}

// Test domain monitoring and recovery
#[tokio::test]
async fn test_domain_monitoring() {
    // Create mock domains
    let eth_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));
    let btc_domain = Arc::new(MockDomainAdapter::new(
        2,
        DomainType::Custom("Bitcoin".into()),
        "Bitcoin",
    ));

    // Create registry and register domains
    let mut registry = DomainRegistry::new();
    registry
        .register(eth_domain.clone(), eth_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(btc_domain.clone(), btc_domain.domain_info().await.unwrap())
        .unwrap();

    // Create time map
    let time_map = causality::domain::SharedTimeMap::new();

    // Simulate domain updates
    for domain in [eth_domain.clone(), btc_domain.clone()] {
        let entry = domain.get_time_map().await.unwrap();
        time_map.update_domain(entry).unwrap();
    }

    // Test disconnection and recovery
    eth_domain.set_connected(false);

    // Check connectivity
    let eth_connected = eth_domain.check_connectivity().await.unwrap();
    let btc_connected = btc_domain.check_connectivity().await.unwrap();

    assert!(!eth_connected);
    assert!(btc_connected);

    // Test time map updates continue for connected domains
    btc_domain.set_block_height(250);
    btc_domain.set_timestamp(2500000);

    // Update time map
    let entry = btc_domain.get_time_map().await.unwrap();
    time_map.update_domain(entry).unwrap();

    // Verify time map has updated for BTC but not for ETH
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&create_test_domain_id(2)), Some(250));
    assert_eq!(map.get_height(&create_test_domain_id(1)), Some(100)); // Unchanged

    // Simulate recovery
    eth_domain.set_connected(true);
    eth_domain.set_block_height(150);
    eth_domain.set_timestamp(1500000);

    // Update time map after recovery
    let entry = eth_domain.get_time_map().await.unwrap();
    time_map.update_domain(entry).unwrap();

    // Verify time map has updated for ETH
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&create_test_domain_id(1)), Some(150));
}

#[tokio::test]
async fn test_cross_domain_synchronization() {
    // Create mock domains with different initial states
    let eth_domain = Arc::new(MockDomainAdapter::new(1, DomainType::EVM, "Ethereum"));
    let btc_domain = Arc::new(MockDomainAdapter::new(
        2,
        DomainType::Custom("Bitcoin".into()),
        "Bitcoin",
    ));

    eth_domain.set_block_height(100);
    eth_domain.set_timestamp(1000000);

    btc_domain.set_block_height(50);
    btc_domain.set_timestamp(900000);

    // Manually sync domains and build time map
    let time_map = causality::domain::SharedTimeMap::new();

    // First sync
    for domain in [eth_domain.clone(), btc_domain.clone()] {
        let entry = domain.get_time_map().await.unwrap();
        time_map.update_domain(entry).unwrap();
    }

    // Verify initial state
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&create_test_domain_id(1)), Some(100));
    assert_eq!(map.get_height(&create_test_domain_id(2)), Some(50));

    // Progress the domains
    sleep(Duration::from_millis(100)).await;

    eth_domain.set_block_height(110);
    eth_domain.set_timestamp(1100000);

    btc_domain.set_block_height(55);
    btc_domain.set_timestamp(950000);

    // Second sync
    for domain in [eth_domain.clone(), btc_domain.clone()] {
        let entry = domain.get_time_map().await.unwrap();
        time_map.update_domain(entry).unwrap();
    }

    // Verify updated state
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&create_test_domain_id(1)), Some(110));
    assert_eq!(map.get_height(&create_test_domain_id(2)), Some(55));

    // Check time map history
    let current_version = time_map.current_version().unwrap();
    assert!(
        current_version > 1,
        "Time map version should have increased"
    );

    // Progress again but only for one domain
    sleep(Duration::from_millis(100)).await;

    eth_domain.set_block_height(120);
    eth_domain.set_timestamp(1200000);

    // Third sync - only ETH
    let entry = eth_domain.get_time_map().await.unwrap();
    time_map.update_domain(entry).unwrap();

    // Verify partially updated state
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&create_test_domain_id(1)), Some(120));
    assert_eq!(map.get_height(&create_test_domain_id(2)), Some(55)); // Unchanged
}
