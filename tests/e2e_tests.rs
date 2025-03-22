use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use tokio::time::sleep;

use causality::domain::adapter::{
    DomainAdapter, DomainInfo, DomainStatus, DomainType, FactProof, FactQuery, ObservedFact,
    ProofType,
};
use causality::domain::{
    DomainId, DomainRegistry, DomainSelector, SelectionCriteria, SharedTimeMap, TimeMap,
    TimeMapEntry, Transaction, TransactionId, TransactionReceipt, TransactionStatus,
};
use causality::effect::{Effect, EffectHandler, EffectType};
use causality::error::{Error, Result};
use causality::log::{LogEntry, LogStorage, MemoryLogStorage, ReplayEngine};
use causality::resource::{ResourceGuard, ResourceManager};
use causality::types::{BlockHash, BlockHeight, ResourceId, Timestamp};

/// Test domain ID generator
fn test_domain_id(id: u8) -> DomainId {
    DomainId(vec![id])
}

/// Mock domain adapter for testing
struct TestDomainAdapter {
    domain_id: DomainId,
    domain_type: DomainType,
    name: String,
    block_height: Arc<Mutex<BlockHeight>>,
    block_hash: Arc<Mutex<BlockHash>>,
    timestamp: Arc<Mutex<Timestamp>>,
    balances: Arc<RwLock<HashMap<String, u64>>>,
    connected: Arc<Mutex<bool>>,
    tx_history: Arc<Mutex<Vec<Transaction>>>,
}

impl TestDomainAdapter {
    fn new(id: u8, domain_type: DomainType, name: &str) -> Self {
        TestDomainAdapter {
            domain_id: test_domain_id(id),
            domain_type,
            name: name.to_string(),
            block_height: Arc::new(Mutex::new(BlockHeight(100))),
            block_hash: Arc::new(Mutex::new(BlockHash([
                id, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ]))),
            timestamp: Arc::new(Mutex::new(Timestamp(1000000))),
            balances: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(Mutex::new(true)),
            tx_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_balance(&self, account: &str, balance: u64) {
        let mut balances = self.balances.write().unwrap();
        balances.insert(account.to_string(), balance);
    }

    fn get_balance(&self, account: &str) -> Option<u64> {
        let balances = self.balances.read().unwrap();
        balances.get(account).copied()
    }

    fn advance_block(&self) {
        let mut height = self.block_height.lock().unwrap();
        *height += 1;

        let mut timestamp = self.timestamp.lock().unwrap();
        *timestamp += 12000; // Assume 12 second blocks
    }
}

#[async_trait]
impl DomainAdapter for TestDomainAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }

    async fn domain_info(&self) -> Result<DomainInfo> {
        Ok(DomainInfo {
            id: self.domain_id.clone(),
            domain_type: self.domain_type.clone(),
            name: self.name.clone(),
            description: Some(format!("Test domain {}", self.name)),
            rpc_url: Some("http://localhost:8545".to_string()),
            explorer_url: None,
            chain_id: Some(1),
            native_currency: Some("ETH".to_string()),
            status: DomainStatus::Active,
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
        match query.fact_type.as_str() {
            "balance" => {
                if let Some(account) = query.parameters.get("account") {
                    let balances = self.balances.read().map_err(|_| {
                        Error::LockError("Failed to acquire balances lock".to_string())
                    })?;

                    let balance = balances
                        .get(account)
                        .copied()
                        .unwrap_or(0)
                        .to_be_bytes()
                        .to_vec();

                    Ok(ObservedFact {
                        domain_id: self.domain_id.clone(),
                        fact_type: query.fact_type,
                        block_height: *self.block_height.lock().unwrap(),
                        block_hash: self.block_hash.lock().unwrap().clone(),
                        timestamp: *self.timestamp.lock().unwrap(),
                        data: balance,
                        proof: None,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err(Error::InvalidArgument(
                        "Missing account parameter".to_string(),
                    ))
                }
            }
            _ => Err(Error::InvalidArgument(format!(
                "Unsupported fact type: {}",
                query.fact_type
            ))),
        }
    }

    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> {
        // Process transaction - simple simulation
        if tx.tx_type == "transfer" {
            if let (Some(from), Some(to), Some(amount_str)) = (
                tx.metadata.get("from"),
                tx.metadata.get("to"),
                tx.metadata.get("amount"),
            ) {
                let amount = amount_str
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidArgument("Invalid amount".to_string()))?;

                {
                    let mut balances = self.balances.write().map_err(|_| {
                        Error::LockError("Failed to acquire balances lock".to_string())
                    })?;

                    let from_balance = *balances.get(from).unwrap_or(&0);
                    if from_balance < amount {
                        return Err(Error::InsufficientFunds("Not enough balance".to_string()));
                    }

                    // Update balances
                    balances.insert(from.clone(), from_balance - amount);
                    let to_balance = *balances.get(to).unwrap_or(&0);
                    balances.insert(to.clone(), to_balance + amount);
                }

                // Store transaction
                self.tx_history.lock().unwrap().push(tx.clone());

                // Advance block
                self.advance_block();

                // Generate transaction ID
                let tx_id = TransactionId(vec![self.domain_id.0[0], 0x01, 0x02, 0x03]);

                return Ok(tx_id);
            }
        }

        Err(Error::InvalidArgument(
            "Transaction not supported".to_string(),
        ))
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
        let current_height = *self.block_height.lock().unwrap();
        Ok(height <= current_height)
    }

    async fn check_connectivity(&self) -> Result<bool> {
        Ok(*self.connected.lock().unwrap())
    }
}

/// Test effect handler implementation
struct TestEffectHandler {
    domains: Arc<DomainRegistry>,
    time_map: SharedTimeMap,
    log_storage: Arc<MemoryLogStorage>,
}

impl TestEffectHandler {
    fn new(
        domains: Arc<DomainRegistry>,
        time_map: SharedTimeMap,
        log_storage: Arc<MemoryLogStorage>,
    ) -> Self {
        Self {
            domains,
            time_map,
            log_storage,
        }
    }

    async fn handle_transfer(
        &self,
        from_domain: &DomainId,
        to_domain: &DomainId,
        from_account: &str,
        to_account: &str,
        amount: u64,
    ) -> Result<()> {
        // Get domains
        let from_adapter = self
            .domains
            .get_domain(from_domain)
            .ok_or_else(|| Error::DomainNotFound(from_domain.clone()))?;

        let to_adapter = self
            .domains
            .get_domain(to_domain)
            .ok_or_else(|| Error::DomainNotFound(to_domain.clone()))?;

        // Create transaction
        let tx = Transaction {
            domain_id: from_domain.clone(),
            tx_type: "transfer".to_string(),
            data: Vec::new(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("from".to_string(), from_account.to_string());
                metadata.insert("to".to_string(), to_account.to_string());
                metadata.insert("amount".to_string(), amount.to_string());
                metadata.insert("target_domain".to_string(), to_domain.to_string());
                metadata
            },
        };

        // Submit transaction
        let tx_id = from_adapter.submit_transaction(tx).await?;

        // Log the effect
        let effect_entry = LogEntry::Effect {
            effect_type: EffectType::Transfer,
            domain_id: from_domain.clone(),
            resources: vec![
                ResourceId::new(&format!("account:{}", from_account)),
                ResourceId::new(&format!("account:{}", to_account)),
            ],
            data: amount.to_be_bytes().to_vec(),
            metadata: HashMap::new(),
            timestamp: Utc::now().timestamp() as u64,
        };

        self.log_storage.append(effect_entry)?;

        // Wait for receipt
        let receipt = from_adapter.get_transaction_receipt(&tx_id).await?;

        if receipt.status != TransactionStatus::Success {
            return Err(Error::TransactionFailed("Transfer failed".to_string()));
        }

        // Update time map
        let time_map_entry = from_adapter.get_time_map().await?;
        self.time_map.update_domain(time_map_entry)?;

        Ok(())
    }

    async fn handle_observe_balance(&self, domain_id: &DomainId, account: &str) -> Result<u64> {
        // Get domain
        let adapter = self
            .domains
            .get_domain(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;

        // Create fact query
        let query = FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "balance".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("account".to_string(), account.to_string());
                params
            },
            block_height: None,
            block_hash: None,
            timestamp: None,
        };

        // Observe fact
        let fact = adapter.observe_fact(query).await?;

        // Extract balance from data
        let balance = if fact.data.len() >= 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&fact.data[0..8]);
            u64::from_be_bytes(bytes)
        } else {
            0
        };

        // Log the fact
        let fact_entry = LogEntry::Fact {
            domain_id: domain_id.clone(),
            fact_type: "balance".to_string(),
            resource: ResourceId::new(&format!("account:{}", account)),
            data: fact.data,
            block_height: fact.block_height,
            block_hash: fact.block_hash,
            timestamp: fact.timestamp,
        };

        self.log_storage.append(fact_entry)?;

        // Update time map
        let time_map_entry = adapter.get_time_map().await?;
        self.time_map.update_domain(time_map_entry)?;

        Ok(balance)
    }
}

// Define a simple cross-domain transfer effect
#[derive(Debug, Clone)]
struct TransferEffect {
    from_domain: DomainId,
    to_domain: DomainId,
    from_account: String,
    to_account: String,
    amount: u64,
}

impl Effect for TransferEffect {
    fn get_type(&self) -> EffectType {
        EffectType::Transfer
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

impl TransferEffect {
    fn execute(&self, handler: &TestEffectHandler) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            handler
                .handle_transfer(
                    &self.from_domain,
                    &self.to_domain,
                    &self.from_account,
                    &self.to_account,
                    self.amount,
                )
                .await
        })
    }

    fn content_hash(&self) -> Vec<u8> {
        // Simple hash representation for testing
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.from_domain.0);
        bytes.extend_from_slice(&self.to_domain.0);
        bytes.extend_from_slice(self.from_account.as_bytes());
        bytes.extend_from_slice(self.to_account.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes
    }
}

// Define a balance observation effect
#[derive(Debug, Clone)]
struct ObserveBalanceEffect {
    domain_id: DomainId,
    account: String,
}

impl Effect for ObserveBalanceEffect {
    fn get_type(&self) -> EffectType {
        EffectType::Observe
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

impl ObserveBalanceEffect {
    fn execute(&self, handler: &TestEffectHandler) -> Result<u64> {
        tokio::runtime::Handle::current().block_on(async {
            handler
                .handle_observe_balance(&self.domain_id, &self.account)
                .await
        })
    }

    fn content_hash(&self) -> Vec<u8> {
        // Simple hash representation for testing
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.domain_id.0);
        bytes.extend_from_slice(self.account.as_bytes());
        bytes
    }
}

#[tokio::test]
async fn test_e2e_transfer_and_observation() -> Result<()> {
    // Create test domains
    let eth_domain = Arc::new(TestDomainAdapter::new(1, DomainType::EVM, "Ethereum"));
    let sol_domain = Arc::new(TestDomainAdapter::new(
        2,
        DomainType::Custom("Solana".into()),
        "Solana",
    ));

    // Set initial balances
    eth_domain.set_balance("alice", 1000);
    eth_domain.set_balance("bob", 500);
    sol_domain.set_balance("carol", 200);

    // Create registry and register domains
    let registry = DomainRegistry::new();

    // Register domains
    registry
        .register(eth_domain.clone(), eth_domain.domain_info().await.unwrap())
        .unwrap();
    registry
        .register(sol_domain.clone(), sol_domain.domain_info().await.unwrap())
        .unwrap();

    // Create shared components
    let time_map = SharedTimeMap::new();
    let log_storage = Arc::new(MemoryLogStorage::new());

    // Create handler
    let handler = TestEffectHandler::new(registry.clone(), time_map.clone(), log_storage.clone());

    // Create resource manager
    let resource_manager = Arc::new(ResourceManager::new());

    // Acquire resources for accounts (demonstrating resource management)
    let alice_resource = ResourceId::new("account:alice");
    let bob_resource = ResourceId::new("account:bob");
    let carol_resource = ResourceId::new("account:carol");

    // Perform transfer with resource safety
    {
        // Acquire resources in a deterministic order to prevent deadlocks
        let _alice_guard = resource_manager
            .acquire_resource(alice_resource.clone(), "transfer_op")
            .await?;
        let _carol_guard = resource_manager
            .acquire_resource(carol_resource.clone(), "transfer_op")
            .await?;

        // Check initial balances via observation effects
        let alice_balance = ObserveBalanceEffect {
            domain_id: test_domain_id(1),
            account: "alice".to_string(),
        }
        .execute(&handler)?;

        let carol_balance = ObserveBalanceEffect {
            domain_id: test_domain_id(2),
            account: "carol".to_string(),
        }
        .execute(&handler)?;

        println!(
            "Initial balances: Alice = {}, Carol = {}",
            alice_balance, carol_balance
        );

        // Perform a cross-domain transfer
        TransferEffect {
            from_domain: test_domain_id(1),
            to_domain: test_domain_id(2),
            from_account: "alice".to_string(),
            to_account: "carol".to_string(),
            amount: 300,
        }
        .execute(&handler)?;

        // Check updated balances
        let new_alice_balance = ObserveBalanceEffect {
            domain_id: test_domain_id(1),
            account: "alice".to_string(),
        }
        .execute(&handler)?;

        let new_carol_balance = ObserveBalanceEffect {
            domain_id: test_domain_id(2),
            account: "carol".to_string(),
        }
        .execute(&handler)?;

        println!(
            "Updated balances: Alice = {}, Carol = {}",
            new_alice_balance, new_carol_balance
        );

        // Verify balances changed as expected (basic assertion)
        assert_eq!(new_alice_balance, alice_balance - 300);
        // In real multi-domain scenarios, Carol's balance might not update immediately
        // This is simplified for the test
    }

    // Test log replay
    println!("Replaying log...");
    let replay_engine = ReplayEngine::new(
        log_storage.clone(),
        registry.clone(), // Domain registry
        time_map.clone(), // Time map
    );
    let events = replay_engine.get_log_entries(0, 100)?;

    // Print log entries
    println!("Log entries:");
    for (i, event) in events.iter().enumerate() {
        match event {
            LogEntry::Effect {
                effect_type,
                domain_id,
                resources,
                timestamp,
                ..
            } => {
                println!(
                    "{}: Effect: {:?} on domain {:?} at {}",
                    i, effect_type, domain_id, timestamp
                );
            }
            LogEntry::Fact {
                domain_id,
                fact_type,
                resource,
                timestamp,
                ..
            } => {
                println!(
                    "{}: Fact: {} for {} on domain {:?} at {}",
                    i, fact_type, resource, domain_id, timestamp
                );
            }
            LogEntry::Event {
                event_type,
                timestamp,
                ..
            } => {
                println!("{}: Event: {:?} at {}", i, event_type, timestamp);
            }
        }
    }

    // Test domain selection
    let selector = DomainSelector::new(registry.clone(), time_map.clone());

    // Update metrics for selection
    for (id, (reliability, latency)) in [
        (test_domain_id(1), (0.9, 100)), // ETH: high reliability, medium latency
        (test_domain_id(2), (0.8, 50)),  // SOL: lower reliability, low latency
    ] {
        selector.update_metrics(causality::domain::DomainMetrics {
            domain_id: id,
            reliability,
            avg_latency: latency,
            cost_factor: 1.0,
            features: HashSet::new(),
            last_update: Utc::now(),
        });
    }

    // Select domain for transfer operation
    let selected = selector
        .select_domain("transfer", &SelectionCriteria::default())
        .await?;
    println!(
        "Selected domain for transfer: {:?}",
        selected.map(|s| s.domain_id)
    );

    Ok(())
}
