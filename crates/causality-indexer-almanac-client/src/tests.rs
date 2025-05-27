// causality-indexer-almanac-client/src/tests.rs
//
// Tests for the Almanac client implementation

#[cfg(feature = "client")]
mod client_tests {
    use crate::models;
    use chrono::Utc;
    use serde_json::json;

    // Helper function to create a mock Almanac event
    fn create_mock_event(id: &str, chain_id: &str, block_number: u64) -> models::AlmanacEvent {
        models::AlmanacEvent {
            id: id.to_string(),
            chain_id: chain_id.to_string(),
            address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            related_addresses: Some(vec![
                "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            ]),
            block_number,
            block_hash: "0xblock_hash".to_string(),
            tx_hash: "0xtx_hash".to_string(),
            timestamp: Utc::now(),
            event_type: "Transfer".to_string(),
            data: json!({
                "from": "0x1234",
                "to": "0x5678",
                "amount": "1000000000000000000"
            }),
            metadata: None,
        }
    }

    // Helper function to create a mock chain status
    fn create_mock_chain_status(chain_id: &str) -> models::AlmanacChainStatus {
        models::AlmanacChainStatus {
            chain_id: chain_id.to_string(),
            latest_indexed_height: 12345,
            latest_chain_height: 12350,
            indexing_lag: 5,
            is_healthy: true,
            last_indexed_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_convert_event_to_fact() {
        let event = create_mock_event("test_id", "ethereum:1", 100);
        
        // Convert the AlmanacEvent to an IndexedFact using the From implementation
        let fact = causality_indexer_adapter::IndexedFact::from(event.clone());

        assert_eq!(fact.id.0, "test_id");
        assert_eq!(fact.chain_id.0, "ethereum:1");
        assert_eq!(fact.block_height, 100);
        assert!(fact.resource_ids.contains(&"0x1234567890abcdef1234567890abcdef12345678".to_string()));
    }

    #[tokio::test]
    async fn test_convert_chain_status() {
        let status = create_mock_chain_status("ethereum:1");
        
        // Convert the AlmanacChainStatus to a ChainStatus using the From implementation
        let chain_status = causality_indexer_adapter::ChainStatus::from(status.clone());

        assert_eq!(chain_status.chain_id.0, "ethereum:1");
        assert_eq!(chain_status.latest_indexed_height, 12345);
        assert_eq!(chain_status.latest_chain_height, 12350);
        assert_eq!(chain_status.indexing_lag, 5);
        assert!(chain_status.is_healthy);
    }
}

// Add more tests here as needed 