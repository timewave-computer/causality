// causality-indexer-client/src/tests.rs
//
// Tests for the Almanac client implementation

use crate::{
    models, AlmanacClient, AlmanacClientConfig, AlmanacClientError, AlmanacHttpClient,
};
use causality_indexer_adapter::{ChainId, FactFilter, FactId, IndexerAdapter, QueryOptions};
use chrono::{TimeZone, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// Helper function to create a mock Almanac event
fn create_mock_event(id: &str, chain_id: &str, block_number: u64) -> models::AlmanacEvent {
    models::AlmanacEvent {
        id: id.to_string(),
        chain_id: chain_id.to_string(),
        address: Some(format!("0x{}", id.repeat(10).chars().take(40).collect::<String>())),
        related_addresses: Some(vec![format!("0x{}", format!("{}", block_number).repeat(10).chars().take(40).collect::<String>())]),
        block_number,
        block_hash: format!("0xblock{}", block_number),
        tx_hash: format!("0xtx{}", id),
        timestamp: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
        event_type: "TestEvent".to_string(),
        data: json!({
            "value": block_number,
            "testId": id
        }),
        metadata: Some(HashMap::new()),
    }
}

// Helper function to create a mock chain status
fn create_mock_chain_status(chain_id: &str) -> models::AlmanacChainStatus {
    models::AlmanacChainStatus {
        chain_id: chain_id.to_string(),
        latest_indexed_height: 100,
        latest_chain_height: 110,
        indexing_lag: 10,
        is_healthy: true,
        last_indexed_at: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
    }
}

#[tokio::test]
async fn test_get_fact_by_id() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup the mock response
    let event = create_mock_event("test1", "ethereum:1", 100);
    
    Mock::given(method("GET"))
        .and(path("/events/test1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&event))
        .mount(&mock_server)
        .await;

    // Create client
    let config = AlmanacClientConfig {
        http_url: mock_server.uri(),
        ws_url: "ws://localhost:8081".to_string(),
        ..Default::default()
    };
    
    let client = AlmanacClient::new(config).unwrap();

    // Call the method
    let fact = client.get_fact_by_id(&FactId::new("test1")).await.unwrap();

    // Verify
    assert!(fact.is_some());
    let fact = fact.unwrap();
    assert_eq!(fact.id.0, "test1");
    assert_eq!(fact.chain_id.0, "ethereum:1");
    assert_eq!(fact.block_height, 100);
}

#[tokio::test]
async fn test_get_facts_by_resource() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup the mock response
    let events = vec![
        create_mock_event("test1", "ethereum:1", 100),
        create_mock_event("test2", "ethereum:1", 101),
    ];
    
    Mock::given(method("GET"))
        .and(path("/resources/0xresource/events"))
        .and(query_param("limit", "10"))
        .and(query_param("order", "desc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&events))
        .mount(&mock_server)
        .await;

    // Create client
    let config = AlmanacClientConfig {
        http_url: mock_server.uri(),
        ws_url: "ws://localhost:8081".to_string(),
        ..Default::default()
    };
    
    let client = AlmanacClient::new(config).unwrap();

    // Call the method
    let facts = client.get_facts_by_resource(
        "0xresource",
        QueryOptions {
            limit: Some(10),
            offset: None,
            ascending: false,
        },
    ).await.unwrap();

    // Verify
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].id.0, "test1");
    assert_eq!(facts[1].id.0, "test2");
}

#[tokio::test]
async fn test_get_facts_by_chain() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup the mock response
    let events = vec![
        create_mock_event("test1", "ethereum:1", 100),
        create_mock_event("test2", "ethereum:1", 101),
    ];
    
    Mock::given(method("GET"))
        .and(path("/chains/ethereum:1/events"))
        .and(query_param("from_block", "95"))
        .and(query_param("to_block", "105"))
        .and(query_param("limit", "10"))
        .and(query_param("order", "asc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&events))
        .mount(&mock_server)
        .await;

    // Create client
    let config = AlmanacClientConfig {
        http_url: mock_server.uri(),
        ws_url: "ws://localhost:8081".to_string(),
        ..Default::default()
    };
    
    let client = AlmanacClient::new(config).unwrap();

    // Call the method
    let facts = client.get_facts_by_chain(
        &ChainId::new("ethereum:1"),
        Some(95),
        Some(105),
        QueryOptions {
            limit: Some(10),
            offset: None,
            ascending: true,
        },
    ).await.unwrap();

    // Verify
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].id.0, "test1");
    assert_eq!(facts[1].id.0, "test2");
}

#[tokio::test]
async fn test_get_chain_status() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup the mock response
    let status = create_mock_chain_status("ethereum:1");
    
    Mock::given(method("GET"))
        .and(path("/chains/ethereum:1/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&status))
        .mount(&mock_server)
        .await;

    // Create client
    let config = AlmanacClientConfig {
        http_url: mock_server.uri(),
        ws_url: "ws://localhost:8081".to_string(),
        ..Default::default()
    };
    
    let client = AlmanacClient::new(config).unwrap();

    // Call the method
    let chain_status = client.get_chain_status(&ChainId::new("ethereum:1")).await.unwrap();

    // Verify
    assert_eq!(chain_status.chain_id.0, "ethereum:1");
    assert_eq!(chain_status.latest_indexed_height, 100);
    assert_eq!(chain_status.latest_chain_height, 110);
    assert_eq!(chain_status.indexing_lag, 10);
    assert!(chain_status.is_healthy);
}

#[tokio::test]
async fn test_http_error_handling() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup the mock response with error
    Mock::given(method("GET"))
        .and(path("/events/not-found"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    // Create client
    let config = AlmanacClientConfig {
        http_url: mock_server.uri(),
        ws_url: "ws://localhost:8081".to_string(),
        max_retries: 1,
        retry_delay_ms: 100,
        ..Default::default()
    };
    
    let client = AlmanacClient::new(config).unwrap();

    // Call the method
    let result = client.get_fact_by_id(&FactId::new("not-found")).await.unwrap();
    
    // Should return None for not found
    assert!(result.is_none());
}

// Note: WebSocket tests would typically require more complex setup with a mock
// WebSocket server. For simplicity, we'll omit them from this basic test suite. 