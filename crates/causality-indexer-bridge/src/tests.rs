// causality-indexer-bridge/src/tests.rs
//
// Tests for the Almanac bridge implementation

use crate::{AlmanacBridge, AlmanacBridgeError, AlmanacBridgeFactory};
use causality_indexer_adapter::{ChainId, FactFilter, FactId, IndexerAdapter, QueryOptions};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex};
use std::time::Duration;

// A mock Almanac event for testing
#[derive(Clone)]
struct MockAlmanacEvent {
    id: String,
    chain: String,
    address: Option<String>,
    block_number: u64,
    block_hash: String,
    tx_hash: String,
    timestamp: SystemTime,
    event_type: String,
    raw_data: Vec<u8>,
}

impl std::fmt::Debug for MockAlmanacEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockAlmanacEvent")
            .field("id", &self.id)
            .field("chain", &self.chain)
            .field("address", &self.address)
            .field("block_number", &self.block_number)
            .field("block_hash", &self.block_hash)
            .field("tx_hash", &self.tx_hash)
            .field("timestamp", &self.timestamp)
            .field("event_type", &self.event_type)
            .field("raw_data", &format!("<{} bytes>", self.raw_data.len()))
            .finish()
    }
}

impl std::any::Any for MockAlmanacEvent {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl indexer_core::event::Event for MockAlmanacEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn chain(&self) -> &str {
        &self.chain
    }

    fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    fn block_number(&self) -> u64 {
        self.block_number
    }

    fn block_hash(&self) -> &str {
        &self.block_hash
    }

    fn tx_hash(&self) -> &str {
        &self.tx_hash
    }

    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// A mock Almanac storage implementation for testing
struct MockAlmanacStorage {
    events: Vec<Box<dyn indexer_core::event::Event>>,
    latest_blocks: std::collections::HashMap<String, u64>,
}

impl MockAlmanacStorage {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            latest_blocks: std::collections::HashMap::new(),
        }
    }

    fn add_event(&mut self, event: Box<dyn indexer_core::event::Event>) {
        let chain = event.chain().to_string();
        let block_number = event.block_number();
        
        // Update latest block if needed
        let latest_block = self.latest_blocks.entry(chain).or_insert(0);
        if block_number > *latest_block {
            *latest_block = block_number;
        }
        
        self.events.push(event);
    }
    
    async fn get_chains(&self) -> indexer_core::Result<Vec<String>> {
        // Return all chains we have in our latest_blocks map
        Ok(self.latest_blocks.keys().cloned().collect())
    }
}

#[async_trait::async_trait]
impl indexer_storage::Storage for MockAlmanacStorage {
    async fn store_event(&self, _chain: &str, _event: Box<dyn indexer_core::event::Event>) -> indexer_core::Result<()> {
        // Not implemented for this test
        Ok(())
    }
    
    async fn get_events(&self, chain: &str, from_block: u64, to_block: u64) -> indexer_core::Result<Vec<Box<dyn indexer_core::event::Event>>> {
        let filtered_events = self.events.iter()
            .filter(|event| {
                let matches_chain = if chain == "*" {
                    true
                } else {
                    event.chain() == chain
                };
                
                let block_num = event.block_number();
                let in_range = block_num >= from_block && block_num <= to_block;
                
                matches_chain && in_range
            })
            .cloned()
            .collect();
        
        Ok(filtered_events)
    }
    
    async fn get_latest_block(&self, chain: &str) -> indexer_core::Result<u64> {
        Ok(*self.latest_blocks.get(chain).unwrap_or(&0))
    }
    
    async fn get_latest_block_with_status(&self, chain: &str, _status: indexer_core::BlockStatus) -> indexer_core::Result<u64> {
        self.get_latest_block(chain).await
    }
    
    async fn mark_block_processed(&self, _chain: &str, _block_number: u64, _tx_hash: &str, _status: indexer_core::BlockStatus) -> indexer_core::Result<()> {
        // Not implemented for this test
        Ok(())
    }

    async fn update_block_status(&self, _chain: &str, _block_number: u64, _status: indexer_core::BlockStatus) -> indexer_core::Result<()> {
        // Not implemented for this test
        Ok(())
    }
    
    async fn get_events_with_status(&self, chain: &str, from_block: u64, to_block: u64, _status: indexer_core::BlockStatus) -> indexer_core::Result<Vec<Box<dyn indexer_core::event::Event>>> {
        self.get_events(chain, from_block, to_block).await
    }
    
    async fn reorg_chain(&self, _chain: &str, _from_block: u64) -> indexer_core::Result<()> {
        // Not implemented for this test
        Ok(())
    }
    
    // The rest of the methods are not needed for this test
    // Add any other required methods if tests need them
    
    async fn store_valence_account_instantiation(
        &self,
        _account_info: indexer_storage::ValenceAccountInfo,
        _initial_libraries: Vec<indexer_storage::ValenceAccountLibrary>,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn store_valence_library_approval(
        &self,
        _account_id: &str,
        _library_info: indexer_storage::ValenceAccountLibrary,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn store_valence_library_removal(
        &self,
        _account_id: &str,
        _library_address: &str,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn store_valence_ownership_update(
        &self,
        _account_id: &str,
        _new_owner: Option<String>,
        _new_pending_owner: Option<String>,
        _new_pending_expiry: Option<u64>,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn store_valence_execution(
        &self,
        _execution_info: indexer_storage::ValenceAccountExecution,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn get_valence_account_state(&self, _account_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceAccountState>> {
        unimplemented!("Not needed for this test")
    }
    
    async fn set_valence_account_state(&self, _account_id: &str, _state: &indexer_storage::ValenceAccountState) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn delete_valence_account_state(&self, _account_id: &str) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn set_historical_valence_account_state(
        &self,
        _account_id: &str,
        _block_number: u64,
        _state: &indexer_storage::ValenceAccountState,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn get_historical_valence_account_state(
        &self,
        _account_id: &str,
        _block_number: u64,
    ) -> indexer_core::Result<Option<indexer_storage::ValenceAccountState>> {
        unimplemented!("Not needed for this test")
    }
    
    async fn delete_historical_valence_account_state(
        &self,
        _account_id: &str,
        _block_number: u64,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn set_latest_historical_valence_block(
        &self,
        _account_id: &str,
        _block_number: u64,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn get_latest_historical_valence_block(&self, _account_id: &str) -> indexer_core::Result<Option<u64>> {
        unimplemented!("Not needed for this test")
    }

    async fn delete_latest_historical_valence_block(&self, _account_id: &str) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_processor_instantiation(
        &self,
        _processor_info: indexer_storage::ValenceProcessorInfo,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_processor_config_update(
        &self,
        _processor_id: &str,
        _config: indexer_storage::ValenceProcessorConfig,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_processor_message(
        &self,
        _message: indexer_storage::ValenceProcessorMessage,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn update_valence_processor_message_status(
        &self,
        _message_id: &str,
        _new_status: indexer_storage::ValenceMessageStatus,
        _processed_block: Option<u64>,
        _processed_tx: Option<&str>,
        _retry_count: Option<u32>,
        _next_retry_block: Option<u64>,
        _gas_used: Option<u64>,
        _error: Option<String>,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_processor_state(&self, _processor_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceProcessorState>> {
        unimplemented!("Not needed for this test")
    }

    async fn set_valence_processor_state(&self, _processor_id: &str, _state: &indexer_storage::ValenceProcessorState) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn set_historical_valence_processor_state(
        &self,
        _processor_id: &str,
        _block_number: u64,
        _state: &indexer_storage::ValenceProcessorState,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn get_historical_valence_processor_state(
        &self,
        _processor_id: &str,
        _block_number: u64,
    ) -> indexer_core::Result<Option<indexer_storage::ValenceProcessorState>> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_authorization_instantiation(
        &self,
        _auth_info: indexer_storage::ValenceAuthorizationInfo,
        _initial_policy: Option<indexer_storage::ValenceAuthorizationPolicy>,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_authorization_policy(
        &self,
        _policy: indexer_storage::ValenceAuthorizationPolicy,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn update_active_authorization_policy(
        &self,
        _auth_id: &str,
        _policy_id: &str,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_authorization_grant(
        &self,
        _grant: indexer_storage::ValenceAuthorizationGrant,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn revoke_valence_authorization_grant(
        &self,
        _auth_id: &str,
        _grantee: &str,
        _resource: &str,
        _revoked_at_block: u64,
        _revoked_at_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_authorization_request(
        &self,
        _request: indexer_storage::ValenceAuthorizationRequest,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn update_valence_authorization_request_decision(
        &self,
        _request_id: &str,
        _decision: indexer_storage::ValenceAuthorizationDecision,
        _processed_block: Option<u64>,
        _processed_tx: Option<&str>,
        _reason: Option<String>,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_library_instantiation(
        &self,
        _library_info: indexer_storage::ValenceLibraryInfo,
        _initial_version: Option<indexer_storage::ValenceLibraryVersion>,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_library_version(
        &self,
        _version: indexer_storage::ValenceLibraryVersion,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn update_active_library_version(
        &self,
        _library_id: &str,
        _version: u32,
        _update_block: u64,
        _update_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn store_valence_library_usage(
        &self,
        _usage: indexer_storage::ValenceLibraryUsage,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn revoke_valence_library_approval(
        &self,
        _library_id: &str,
        _account_id: &str,
        _revoked_at_block: u64,
        _revoked_at_tx: &str,
    ) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_library_state(&self, _library_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceLibraryState>> {
        unimplemented!("Not needed for this test")
    }

    async fn set_valence_library_state(&self, _library_id: &str, _state: &indexer_storage::ValenceLibraryState) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_library_versions(&self, _library_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryVersion>> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_library_approvals(&self, _library_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryApproval>> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_libraries_for_account(&self, _account_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryApproval>> {
        unimplemented!("Not needed for this test")
    }

    async fn get_valence_library_usage_history(
        &self,
        _library_id: &str,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryUsage>> {
        unimplemented!("Not needed for this test")
    }

    async fn set_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn get_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
        unimplemented!("Not needed for this test")
    }
    
    async fn set_historical_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
        unimplemented!("Not needed for this test")
    }
    
    async fn get_historical_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
        unimplemented!("Not needed for this test")
    }
}

struct MockStorage {
    events: Mutex<Vec<Box<dyn indexer_core::event::Event>>>,
    chains: Mutex<HashMap<String, usize>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            chains: Mutex::new(HashMap::new()),
        }
    }

    fn add_event(&self, event: Box<dyn indexer_core::event::Event>) {
        let mut events = self.events.lock().unwrap();
        let mut chains = self.chains.lock().unwrap();
        
        // Update chain height
        let current_height = chains.entry(event.chain().to_string()).or_insert(0);
        if let Some(height) = event.block_number() {
            if height as usize > *current_height {
                *current_height = height as usize;
            }
        }
        
        events.push(event);
    }
}

impl indexer_storage::Storage for MockStorage {
    fn get_event(&self, id: &indexer_storage::EventId) -> Result<Option<Box<dyn indexer_core::event::Event>>, indexer_storage::StorageError> {
        let events = self.events.lock().unwrap();
        let event = events.iter().find(|e| e.id() == id.0).cloned();
        Ok(event)
    }

    fn get_events_by_resource(
        &self,
        resource_id: &indexer_storage::ResourceId,
        _filter: Option<&indexer_storage::ResourceFilter>,
    ) -> Result<Vec<Box<dyn indexer_core::event::Event>>, indexer_storage::StorageError> {
        let events = self.events.lock().unwrap();
        let matching_events = events
            .iter()
            .filter(|e| e.chain() == resource_id.0 && e.address().map(|a| a == resource_id.1).unwrap_or(false))
            .cloned()
            .collect();
        Ok(matching_events)
    }

    fn get_events_by_block(
        &self,
        chain: &str,
        block_number: u64,
    ) -> Result<Vec<Box<dyn indexer_core::event::Event>>, indexer_storage::StorageError> {
        let events = self.events.lock().unwrap();
        let matching_events = events
            .iter()
            .filter(|e| {
                e.chain() == chain && e.block_number() == Some(block_number)
            })
            .cloned()
            .collect();
        Ok(matching_events)
    }

    fn get_blocks_in_range(
        &self,
        chain: &str,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<u64>, indexer_storage::StorageError> {
        let events = self.events.lock().unwrap();
        let mut blocks = HashSet::new();
        
        for event in events.iter() {
            if event.chain() == chain {
                if let Some(block) = event.block_number() {
                    if block >= from_block && block <= to_block {
                        blocks.insert(block);
                    }
                }
            }
        }
        
        let mut block_vec: Vec<u64> = blocks.into_iter().collect();
        block_vec.sort();
        Ok(block_vec)
    }

    fn get_chain_head(&self, chain: &str) -> Result<Option<u64>, indexer_storage::StorageError> {
        let chains = self.chains.lock().unwrap();
        Ok(chains.get(chain).map(|&h| h as u64))
    }

    fn get_all_resource_ids(&self) -> Result<Vec<indexer_storage::ResourceId>, indexer_storage::StorageError> {
        let events = self.events.lock().unwrap();
        let mut resources = HashSet::new();
        
        for event in events.iter() {
            for resource in event.address().map(|a| indexer_storage::ResourceId::new(event.chain().to_string(), a.to_string())).into_iter() {
                resources.insert(resource);
            }
        }
        
        Ok(resources.into_iter().collect())
    }

    fn get_all_chains(&self) -> Result<Vec<String>, indexer_storage::StorageError> {
        let chains = self.chains.lock().unwrap();
        Ok(chains.keys().cloned().collect())
    }
}

fn create_test_event(
    id: &str,
    chain: &str,
    resources: Vec<&str>,
    block: Option<u64>,
    timestamp: DateTime<Utc>,
) -> Box<dyn indexer_core::event::Event> {
    Box::new(MockAlmanacEvent {
        id: id.to_string(),
        chain: chain.to_string(),
        address: None,
        block_number: block.unwrap_or_default(),
        block_hash: "0xabc".to_string(),
        tx_hash: "0xdef".to_string(),
        timestamp,
        event_type: "Transfer".to_string(),
        raw_data: serde_json::to_vec(&serde_json::json!({
            "from": "0x111",
            "to": "0x222",
            "amount": "1000"
        })).unwrap(),
    })
}

#[tokio::test]
async fn test_get_facts_by_chain() {
    // Create mock storage
    let storage = Arc::new(MockStorage::new());
    
    // Add test events
    storage.add_event(create_test_event(
        "ev1",
        "ethereum:1",
    let event2 = Box::new(MockAlmanacEvent {
        id: "event2".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x456".to_string()),
        block_number: 200,
        block_hash: "0xbcd".to_string(),
        tx_hash: "0xefg".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Approval".to_string(),
        raw_data: serde_json::to_vec(&serde_json::json!({
            "owner": "0x333",
            "spender": "0x444",
            "amount": "500"
        })).unwrap(),
    }) as Box<dyn indexer_core::event::Event>;
    
    // Add events to storage
    storage.add_event(event1);
    storage.add_event(event2);
    
    // Create bridge
    let bridge = AlmanacBridge::new(Arc::new(storage));
    
    // Test get_facts_by_chain
    let chain_id = ChainId::new("ethereum:1");
    let facts = bridge.get_facts_by_chain(
        &chain_id, 
        Some(0), 
        Some(1000), 
        QueryOptions {
            limit: None,
            offset: None,
            ascending: true,
        }
    ).await.unwrap();
    
    // Verify results
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].id.0, "event1");
    assert_eq!(facts[0].chain_id.0, "ethereum:1");
    assert_eq!(facts[0].resource_ids[0], "0x123");
    assert_eq!(facts[0].block_height, 100);
    
    assert_eq!(facts[1].id.0, "event2");
    assert_eq!(facts[1].chain_id.0, "ethereum:1");
    assert_eq!(facts[1].resource_ids[0], "0x456");
    assert_eq!(facts[1].block_height, 200);
}

#[tokio::test]
async fn test_get_chain_status() {
    // Create mock storage with some events
    let mut storage = MockAlmanacStorage::new();
    
    // Add events to set the latest block
    let event = Box::new(MockAlmanacEvent {
        id: "event1".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x123".to_string()),
        block_number: 1000,
        block_hash: "0xabc".to_string(),
        tx_hash: "0xdef".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Transfer".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    storage.add_event(event);
    
    // Create bridge
    let bridge = AlmanacBridge::new(Arc::new(storage));
    
    // Test get_chain_status
    let chain_id = ChainId::new("ethereum:1");
    let status = bridge.get_chain_status(&chain_id).await.unwrap();
    
    // Verify results
    assert_eq!(status.chain_id.0, "ethereum:1");
    assert_eq!(status.latest_indexed_height, 1000);
    assert_eq!(status.latest_chain_height, 1010); // Mock adds 10 to simulate lag
    assert_eq!(status.indexing_lag, 10);
    assert!(status.is_healthy);
}

#[tokio::test]
async fn test_get_facts_by_resource() {
    // Create mock storage with some events
    let mut storage = MockAlmanacStorage::new();
    
    // Create mock events with the same address
    let event1 = Box::new(MockAlmanacEvent {
        id: "event1".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x123".to_string()),
        block_number: 100,
        block_hash: "0xabc".to_string(),
        tx_hash: "0xdef".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Transfer".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    let event2 = Box::new(MockAlmanacEvent {
        id: "event2".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x123".to_string()), // Same address
        block_number: 200,
        block_hash: "0xbcd".to_string(),
        tx_hash: "0xefg".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Approval".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    let event3 = Box::new(MockAlmanacEvent {
        id: "event3".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x456".to_string()), // Different address
        block_number: 300,
        block_hash: "0xcde".to_string(),
        tx_hash: "0xfgh".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Transfer".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    // Add events to storage
    storage.add_event(event1);
    storage.add_event(event2);
    storage.add_event(event3);
    
    // Create bridge
    let bridge = AlmanacBridge::new(Arc::new(storage));
    
    // Test get_facts_by_resource
    let facts = bridge.get_facts_by_resource(
        "0x123", 
        QueryOptions {
            limit: None,
            offset: None,
            ascending: true,
        }
    ).await.unwrap();
    
    // Verify results
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].id.0, "event1");
    assert_eq!(facts[0].resource_ids[0], "0x123");
    
    assert_eq!(facts[1].id.0, "event2");
    assert_eq!(facts[1].resource_ids[0], "0x123");
}

#[tokio::test]
async fn test_get_fact_by_id() {
    // Create mock storage with some events
    let mut storage = MockAlmanacStorage::new();
    
    // Create mock events with different IDs
    let event1 = Box::new(MockAlmanacEvent {
        id: "event1".to_string(),
        chain: "ethereum".to_string(),
        address: Some("0x123".to_string()),
        block_number: 100,
        block_hash: "0xabc".to_string(),
        tx_hash: "0xdef".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Transfer".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    let event2 = Box::new(MockAlmanacEvent {
        id: "special-event-id".to_string(), // This is the one we'll search for
        chain: "ethereum".to_string(),
        address: Some("0x456".to_string()),
        block_number: 200,
        block_hash: "0xbcd".to_string(),
        tx_hash: "0xefg".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Approval".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    let event3 = Box::new(MockAlmanacEvent {
        id: "event3".to_string(),
        chain: "cosmos".to_string(), // Different chain
        address: Some("cosmos123".to_string()),
        block_number: 300,
        block_hash: "cosmos-hash-123".to_string(),
        tx_hash: "cosmos-tx-123".to_string(),
        timestamp: SystemTime::now(),
        event_type: "Transfer".to_string(),
        raw_data: vec![],
    }) as Box<dyn indexer_core::event::Event>;
    
    // Add events to storage
    storage.add_event(event1);
    storage.add_event(event2);
    storage.add_event(event3);
    
    // Create bridge
    let bridge = AlmanacBridge::new(Arc::new(storage));
    
    // Test get_fact_by_id with an existing ID
    let fact_id = FactId::new("special-event-id");
    let fact_result = bridge.get_fact_by_id(&fact_id).await.unwrap();
    
    // Verify we found the correct fact
    assert!(fact_result.is_some());
    let fact = fact_result.unwrap();
    assert_eq!(fact.id.0, "special-event-id");
    assert_eq!(fact.chain_id.0, "ethereum:0"); // Default ChainId format
    assert_eq!(fact.resource_ids[0], "0x456");
    assert_eq!(fact.block_height, 200);
    
    // Test get_fact_by_id with a non-existent ID
    let nonexistent_id = FactId::new("nonexistent-id");
    let nonexistent_result = bridge.get_fact_by_id(&nonexistent_id).await.unwrap();
    
    // Verify we get None for a non-existent ID
    assert!(nonexistent_result.is_none());
} 