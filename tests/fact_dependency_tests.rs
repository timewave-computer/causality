// End-to-end tests for fact dependencies
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use causality::domain::adapter::{
    DomainAdapter, DomainInfo, DomainStatus, DomainType, FactQuery, TimeMapEntry, Transaction,
    TransactionId, TransactionReceipt, TransactionStatus,
};
use causality::domain::fact::verification::VerificationResult;
use causality::effects::dependency::{DependencySet, EffectDependency, FactDependency};
use causality::effects::snapshot::FactSnapshot;
use causality::effects::{Effect, EffectId, EffectStatus};
use causality::error::{Error, Result};
use causality::log::fact_types::{FactType, RegisterFact};
use causality::types::{BlockHash, BlockHeight, DomainId, Timestamp};

// Mock domain adapter
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

    fn register_fact(&self, key: &str, fact: FactType) {
        let mut facts = self.facts.lock().unwrap();
        facts.insert(key.to_string(), fact);
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
            description: Some("Test domain for fact dependencies".to_string()),
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

        let fact_key = format!(
            "{}:{}",
            query.fact_type,
            query.parameters.get("id").unwrap_or(&"default".to_string())
        );

        if let Some(fact) = facts.get(&fact_key) {
            return Ok(fact.clone());
        }

        Err(Error::FactNotFound(format!("Fact not found: {}", fact_key)))
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

// Mock effect with dependencies
#[derive(Debug, Clone)]
struct MockEffect {
    id: EffectId,
    dependencies: DependencySet,
    status: EffectStatus,
}

impl MockEffect {
    fn new(id: &str) -> Self {
        Self {
            id: EffectId::new(id),
            dependencies: DependencySet::new(),
            status: EffectStatus::Pending,
        }
    }

    fn add_fact_dependency(&mut self, fact: &FactType, required: bool) {
        self.dependencies.add_fact_dependency(FactDependency {
            fact: fact.clone(),
            required,
            metadata: HashMap::new(),
        });
    }

    fn add_effect_dependency(&mut self, effect_id: &EffectId, required: bool) {
        self.dependencies.add_effect_dependency(EffectDependency {
            effect_id: effect_id.clone(),
            required,
            metadata: HashMap::new(),
        });
    }

    fn set_status(&mut self, status: EffectStatus) {
        self.status = status;
    }
}

// Checks if all dependencies are satisfied
fn are_dependencies_satisfied(
    effect: &MockEffect,
    snapshot: &FactSnapshot,
    completed_effects: &[EffectId],
) -> bool {
    // Check if all required fact dependencies are in the snapshot
    for fact_dep in effect.dependencies.fact_dependencies() {
        if fact_dep.required && !snapshot.contains_fact(&fact_dep.fact) {
            return false;
        }
    }

    // Check if all required effect dependencies are in the completed effects
    for effect_dep in effect.dependencies.effect_dependencies() {
        if effect_dep.required && !completed_effects.contains(&effect_dep.effect_id) {
            return false;
        }
    }

    true
}

// Create a balance fact
fn create_balance_fact(domain_id: &DomainId, address: &str, amount: &str) -> FactType {
    FactType::BalanceFact {
        domain_id: domain_id.clone(),
        address: address.to_string(),
        amount: amount.to_string(),
        token: None,
        block_height: Some(1000),
        block_hash: Some(vec![1, 2, 3, 4]),
        timestamp: Some(1600000000),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    }
}

// Create a register fact
fn create_register_fact(domain_id: &DomainId, register_id: &str, owner: &str) -> FactType {
    FactType::RegisterFact(RegisterFact::RegisterCreation {
        domain_id: domain_id.clone(),
        register_id: register_id.to_string(),
        owner: owner.to_string(),
        register_type: Some("token".to_string()),
        initial_value: Some("100".to_string()),
        block_height: Some(1000),
        block_hash: Some(vec![1, 2, 3, 4]),
        timestamp: Some(1600000000),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    })
}

#[tokio::test]
async fn test_fact_dependencies() -> Result<()> {
    // Create domain
    let domain_id = DomainId::new("test-domain");
    let adapter = Arc::new(MockAdapter::new(domain_id.clone()));

    // Register some facts in the mock adapter
    let alice_balance = create_balance_fact(&domain_id, "alice", "1000");
    let bob_balance = create_balance_fact(&domain_id, "bob", "2000");
    let register1 = create_register_fact(&domain_id, "register1", "alice");

    adapter.register_fact("balance:alice", alice_balance.clone());
    adapter.register_fact("balance:bob", bob_balance.clone());
    adapter.register_fact("register_create:register1", register1.clone());

    // Create a fact snapshot
    let mut snapshot = FactSnapshot::new();

    // Add facts to the snapshot
    snapshot.add_fact(alice_balance.clone());
    snapshot.add_fact(register1.clone());
    // Note: bob_balance is intentionally not added yet

    // Create effects with dependencies
    let mut effect1 = MockEffect::new("effect1");
    let mut effect2 = MockEffect::new("effect2");
    let mut effect3 = MockEffect::new("effect3");

    // Effect1 depends on alice_balance (satisfied) and register1 (satisfied)
    effect1.add_fact_dependency(&alice_balance, true);
    effect1.add_fact_dependency(&register1, true);

    // Effect2 depends on bob_balance (not satisfied yet)
    effect2.add_fact_dependency(&bob_balance, true);

    // Effect3 depends on effect1 (will be satisfied) and bob_balance (not satisfied yet)
    effect3.add_effect_dependency(&effect1.id, true);
    effect3.add_fact_dependency(&bob_balance, true);

    // Check initial dependencies
    let completed_effects = vec![];

    assert!(
        are_dependencies_satisfied(&effect1, &snapshot, &completed_effects),
        "Effect1 dependencies should be satisfied"
    );

    assert!(
        !are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should not be satisfied yet"
    );

    assert!(
        !are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should not be satisfied yet (missing effect1 and bob_balance)"
    );

    // Mark effect1 as completed
    effect1.set_status(EffectStatus::Completed);
    let completed_effects = vec![effect1.id.clone()];

    // Effect3 still needs bob_balance
    assert!(
        !are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should not be satisfied yet (missing bob_balance)"
    );

    // Add bob_balance to the snapshot
    snapshot.add_fact(bob_balance.clone());

    // Now effect2 and effect3 dependencies should be satisfied
    assert!(
        are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should now be satisfied"
    );

    assert!(
        are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should now be satisfied"
    );

    Ok(())
}

#[tokio::test]
async fn test_fact_dependency_chain() -> Result<()> {
    // Create domain
    let domain_id = DomainId::new("test-domain");
    let adapter = Arc::new(MockAdapter::new(domain_id.clone()));

    // Create a chain of register facts
    let register1 = create_register_fact(&domain_id, "register1", "alice");

    // Create register update fact
    let register1_update = FactType::RegisterFact(RegisterFact::RegisterUpdate {
        domain_id: domain_id.clone(),
        register_id: "register1".to_string(),
        new_value: "200".to_string(),
        previous_value: Some("100".to_string()),
        updater: Some("alice".to_string()),
        block_height: Some(1001),
        block_hash: Some(vec![2, 3, 4, 5]),
        timestamp: Some(1600000100),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    });

    // Create register transfer fact
    let register1_transfer = FactType::RegisterFact(RegisterFact::RegisterTransfer {
        domain_id: domain_id.clone(),
        register_id: "register1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        block_height: Some(1002),
        block_hash: Some(vec![3, 4, 5, 6]),
        timestamp: Some(1600000200),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    });

    // Register facts in the adapter
    adapter.register_fact("register_create:register1", register1.clone());
    adapter.register_fact("register_update:register1", register1_update.clone());
    adapter.register_fact("register_transfer:register1", register1_transfer.clone());

    // Create a fact snapshot
    let mut snapshot = FactSnapshot::new();

    // Create effects with chained dependencies
    // Each effect depends on the previous register state
    let mut effect1 = MockEffect::new("effect1"); // Create register
    let mut effect2 = MockEffect::new("effect2"); // Update register (depends on creation)
    let mut effect3 = MockEffect::new("effect3"); // Transfer register (depends on update)

    effect1.add_fact_dependency(&register1, true);

    effect2.add_fact_dependency(&register1, true); // Depends on register creation
    effect2.add_fact_dependency(&register1_update, true);
    effect2.add_effect_dependency(&effect1.id, true);

    effect3.add_fact_dependency(&register1_update, true); // Depends on register update
    effect3.add_fact_dependency(&register1_transfer, true);
    effect3.add_effect_dependency(&effect2.id, true);

    // Add the first fact to the snapshot
    snapshot.add_fact(register1.clone());

    // Check initial dependencies
    let completed_effects = vec![];

    assert!(
        are_dependencies_satisfied(&effect1, &snapshot, &completed_effects),
        "Effect1 dependencies should be satisfied"
    );

    assert!(
        !are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should not be satisfied yet"
    );

    assert!(
        !are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should not be satisfied yet"
    );

    // Mark effect1 as completed and add register update to snapshot
    effect1.set_status(EffectStatus::Completed);
    snapshot.add_fact(register1_update.clone());
    let completed_effects = vec![effect1.id.clone()];

    assert!(
        are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should now be satisfied"
    );

    assert!(
        !are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should not be satisfied yet"
    );

    // Mark effect2 as completed and add register transfer to snapshot
    effect2.set_status(EffectStatus::Completed);
    snapshot.add_fact(register1_transfer.clone());
    let completed_effects = vec![effect1.id.clone(), effect2.id.clone()];

    assert!(
        are_dependencies_satisfied(&effect3, &snapshot, &completed_effects),
        "Effect3 dependencies should now be satisfied"
    );

    Ok(())
}

#[tokio::test]
async fn test_optional_dependencies() -> Result<()> {
    // Create domain
    let domain_id = DomainId::new("test-domain");
    let adapter = Arc::new(MockAdapter::new(domain_id.clone()));

    // Create facts
    let alice_balance = create_balance_fact(&domain_id, "alice", "1000");
    let bob_balance = create_balance_fact(&domain_id, "bob", "2000");

    // Register facts in the adapter
    adapter.register_fact("balance:alice", alice_balance.clone());
    adapter.register_fact("balance:bob", bob_balance.clone());

    // Create a fact snapshot
    let mut snapshot = FactSnapshot::new();
    snapshot.add_fact(alice_balance.clone());

    // Create effect with one required and one optional dependency
    let mut effect = MockEffect::new("effect1");
    effect.add_fact_dependency(&alice_balance, true); // Required
    effect.add_fact_dependency(&bob_balance, false); // Optional

    // Check dependencies - should pass because optional dependencies don't block
    let completed_effects = vec![];

    assert!(
        are_dependencies_satisfied(&effect, &snapshot, &completed_effects),
        "Effect dependencies should be satisfied with optional dependencies missing"
    );

    // Add the optional dependency
    snapshot.add_fact(bob_balance.clone());

    // Should still be satisfied
    assert!(
        are_dependencies_satisfied(&effect, &snapshot, &completed_effects),
        "Effect dependencies should still be satisfied with optional dependencies present"
    );

    Ok(())
}

#[tokio::test]
async fn test_fact_dependency_conflicts() -> Result<()> {
    // Create domain
    let domain_id = DomainId::new("test-domain");
    let adapter = Arc::new(MockAdapter::new(domain_id.clone()));

    // Create two conflicting register facts (different owners for same register)
    let register1_alice = create_register_fact(&domain_id, "register1", "alice");

    let register1_bob = FactType::RegisterFact(RegisterFact::RegisterCreation {
        domain_id: domain_id.clone(),
        register_id: "register1".to_string(),
        owner: "bob".to_string(), // Different owner!
        register_type: Some("token".to_string()),
        initial_value: Some("100".to_string()),
        block_height: Some(1000),
        block_hash: Some(vec![1, 2, 3, 4]),
        timestamp: Some(1600000000),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    });

    // Register facts in the adapter
    adapter.register_fact("register_create:alice", register1_alice.clone());
    adapter.register_fact("register_create:bob", register1_bob.clone());

    // Create a fact snapshot
    let mut snapshot = FactSnapshot::new();

    // Create effects dependent on different versions of the same register
    let mut effect1 = MockEffect::new("effect1");
    let mut effect2 = MockEffect::new("effect2");

    effect1.add_fact_dependency(&register1_alice, true);
    effect2.add_fact_dependency(&register1_bob, true);

    // Add alice's register to the snapshot
    snapshot.add_fact(register1_alice.clone());

    // Check dependencies
    let completed_effects = vec![];

    assert!(
        are_dependencies_satisfied(&effect1, &snapshot, &completed_effects),
        "Effect1 dependencies should be satisfied"
    );

    assert!(
        !are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should not be satisfied"
    );

    // Now replace with bob's register
    snapshot = FactSnapshot::new(); // Clear snapshot
    snapshot.add_fact(register1_bob.clone());

    assert!(
        !are_dependencies_satisfied(&effect1, &snapshot, &completed_effects),
        "Effect1 dependencies should no longer be satisfied"
    );

    assert!(
        are_dependencies_satisfied(&effect2, &snapshot, &completed_effects),
        "Effect2 dependencies should now be satisfied"
    );

    Ok(())
}
