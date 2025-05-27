// causality-indexer-adapter/src/tests.rs
//
// Unit tests for the adapter interfaces

use crate::*;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl std::error::Error for TestError {}

#[derive(Debug)]
struct TestFact {
    id: String,
    chain_id: String,
    resource_ids: Vec<String>,
    block_height: u64,
    timestamp: DateTime<Utc>,
    data: serde_json::Value,
}

#[derive(Debug)]
struct TestSubscription {
    facts: Vec<IndexedFact>,
    current_index: usize,
}

#[async_trait]
impl FactSubscription for TestSubscription {
    type Error = TestError;
    
    async fn next_fact(&mut self) -> Result<Option<IndexedFact>, Self::Error> {
        if self.current_index < self.facts.len() {
            let fact = self.facts[self.current_index].clone();
            self.current_index += 1;
            Ok(Some(fact))
        } else {
            Ok(None)
        }
    }
    
    async fn close(&mut self) -> Result<(), Self::Error> {
        self.facts.clear();
        self.current_index = 0;
        Ok(())
    }
}

#[derive(Debug)]
struct TestAdapter {
    facts: Vec<TestFact>,
}

impl TestAdapter {
    fn new() -> Self {
        Self {
            facts: vec![],
        }
    }
    
    fn add_fact(&mut self, fact: TestFact) {
        self.facts.push(fact);
    }
    
    fn get_indexed_fact(&self, test_fact: &TestFact) -> IndexedFact {
        IndexedFact {
            id: FactId::new(&test_fact.id),
            chain_id: ChainId::new(&test_fact.chain_id),
            resource_ids: test_fact.resource_ids.clone(),
            timestamp: test_fact.timestamp,
            block_height: test_fact.block_height,
            transaction_hash: None,
            data: test_fact.data.clone(),
            metadata: None,
        }
    }
}

#[async_trait]
impl IndexerAdapter for TestAdapter {
    type Error = TestError;
    
    async fn get_facts_by_resource(
        &self, 
        resource_id: &str,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        let mut facts: Vec<IndexedFact> = self.facts
            .iter()
            .filter(|f| f.resource_ids.contains(&resource_id.to_string()))
            .map(|f| self.get_indexed_fact(f))
            .collect();
            
        if !options.ascending {
            facts.sort_by(|a, b| b.block_height.cmp(&a.block_height));
        } else {
            facts.sort_by(|a, b| a.block_height.cmp(&b.block_height));
        }
        
        if let Some(offset) = options.offset {
            facts = facts.into_iter().skip(offset as usize).collect();
        }
        
        if let Some(limit) = options.limit {
            facts = facts.into_iter().take(limit as usize).collect();
        }
        
        Ok(facts)
    }
    
    async fn get_facts_by_chain(
        &self, 
        chain_id: &ChainId, 
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        let mut facts: Vec<IndexedFact> = self.facts
            .iter()
            .filter(|f| f.chain_id == chain_id.0)
            .filter(|f| {
                if let Some(from) = from_height {
                    f.block_height >= from
                } else {
                    true
                }
            })
            .filter(|f| {
                if let Some(to) = to_height {
                    f.block_height <= to
                } else {
                    true
                }
            })
            .map(|f| self.get_indexed_fact(f))
            .collect();
            
        if !options.ascending {
            facts.sort_by(|a, b| b.block_height.cmp(&a.block_height));
        } else {
            facts.sort_by(|a, b| a.block_height.cmp(&b.block_height));
        }
        
        if let Some(offset) = options.offset {
            facts = facts.into_iter().skip(offset as usize).collect();
        }
        
        if let Some(limit) = options.limit {
            facts = facts.into_iter().take(limit as usize).collect();
        }
        
        Ok(facts)
    }
    
    async fn get_fact_by_id(&self, fact_id: &FactId) -> Result<Option<IndexedFact>, Self::Error> {
        let fact = self.facts
            .iter()
            .find(|f| f.id == fact_id.0)
            .map(|f| self.get_indexed_fact(f));
            
        Ok(fact)
    }
    
    async fn subscribe(&self, filter: FactFilter) -> Result<Box<dyn FactSubscription<Error = Self::Error> + Send>, Self::Error> {
        let mut filtered_facts = Vec::new();
        
        for fact in &self.facts {
            let mut include = true;
            
            // Filter by chains
            if let Some(chains) = &filter.chains {
                if !chains.iter().any(|c| c.0 == fact.chain_id) {
                    include = false;
                }
            }
            
            // Filter by resources
            if let Some(resources) = &filter.resources {
                if !fact.resource_ids.iter().any(|r| resources.contains(r)) {
                    include = false;
                }
            }
            
            // Filter by block height
            if let Some(from_height) = filter.from_height {
                if fact.block_height < from_height {
                    include = false;
                }
            }
            
            if let Some(to_height) = filter.to_height {
                if fact.block_height > to_height {
                    include = false;
                }
            }
            
            if include {
                filtered_facts.push(self.get_indexed_fact(fact));
            }
        }
        
        Ok(Box::new(TestSubscription {
            facts: filtered_facts,
            current_index: 0,
        }))
    }
    
    async fn get_chain_status(&self, chain_id: &ChainId) -> Result<ChainStatus, Self::Error> {
        // Find the highest block for this chain
        let latest_block = self.facts
            .iter()
            .filter(|f| f.chain_id == chain_id.0)
            .map(|f| f.block_height)
            .max()
            .unwrap_or(0);
            
        // Get the latest timestamp
        let latest_time = self.facts
            .iter()
            .filter(|f| f.chain_id == chain_id.0)
            .map(|f| f.timestamp)
            .max()
            .unwrap_or_else(|| Utc::now());
            
        Ok(ChainStatus {
            chain_id: chain_id.clone(),
            latest_indexed_height: latest_block,
            latest_chain_height: latest_block + 10, // Simulate a small lag
            indexing_lag: 10,
            is_healthy: true,
            last_indexed_at: latest_time,
        })
    }
}

struct TestAdapterFactory {
    adapter: Arc<TestAdapter>,
}

impl TestAdapterFactory {
    fn new(adapter: TestAdapter) -> Self {
        Self {
            adapter: Arc::new(adapter),
        }
    }
}

#[async_trait]
impl IndexerAdapterFactory for TestAdapterFactory {
    type Error = TestError;
    type Adapter = Arc<TestAdapter>;
    
    async fn create(&self) -> Result<Self::Adapter, Self::Error> {
        Ok(self.adapter.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    fn create_test_adapter() -> TestAdapter {
        let mut adapter = TestAdapter::new();
        
        // Add some test facts
        adapter.add_fact(TestFact {
            id: "fact1".to_string(),
            chain_id: "ethereum:1".to_string(),
            resource_ids: vec!["0xabc123".to_string()],
            block_height: 100,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "type": "Transfer",
                "from": "0x111",
                "to": "0x222",
                "amount": "1000"
            }),
        });
        
        adapter.add_fact(TestFact {
            id: "fact2".to_string(),
            chain_id: "ethereum:1".to_string(),
            resource_ids: vec!["0xabc123".to_string()],
            block_height: 200,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "type": "Transfer",
                "from": "0x222",
                "to": "0x333",
                "amount": "500"
            }),
        });
        
        adapter.add_fact(TestFact {
            id: "fact3".to_string(),
            chain_id: "polygon:137".to_string(),
            resource_ids: vec!["0xdef456".to_string()],
            block_height: 300,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "type": "Approval",
                "owner": "0x111",
                "spender": "0x222",
                "amount": "2000"
            }),
        });
        
        adapter
    }
    
    #[test]
    fn test_get_facts_by_resource() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        
        let facts = rt.block_on(async {
            adapter.get_facts_by_resource(
                "0xabc123",
                QueryOptions {
                    limit: None,
                    offset: None,
                    ascending: true,
                }
            ).await.unwrap()
        });
        
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].block_height, 100);
        assert_eq!(facts[1].block_height, 200);
    }
    
    #[test]
    fn test_get_facts_by_chain() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        
        let facts = rt.block_on(async {
            adapter.get_facts_by_chain(
                &ChainId::new("ethereum:1"),
                None,
                None,
                QueryOptions::default(),
            ).await.unwrap()
        });
        
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].chain_id.0, "ethereum:1");
        assert_eq!(facts[1].chain_id.0, "ethereum:1");
    }
    
    #[test]
    fn test_get_fact_by_id() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        
        let fact = rt.block_on(async {
            adapter.get_fact_by_id(&FactId::new("fact2")).await.unwrap()
        });
        
        assert!(fact.is_some());
        let fact = fact.unwrap();
        assert_eq!(fact.id.0, "fact2");
        assert_eq!(fact.block_height, 200);
    }
    
    #[test]
    fn test_subscription() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        
        let filter = FactFilter {
            resources: Some(vec!["0xabc123".to_string()]),
            chains: Some(vec![ChainId::new("ethereum:1")]),
            event_types: None,
            from_height: None,
            to_height: None,
        };
        
        let mut facts = Vec::new();
        
        rt.block_on(async {
            let mut subscription = adapter.subscribe(filter).await.unwrap();
            
            while let Some(fact) = subscription.next_fact().await.unwrap() {
                facts.push(fact);
            }
            
            subscription.close().await.unwrap();
        });
        
        assert_eq!(facts.len(), 2);
        assert!(facts.iter().all(|f| f.chain_id.0 == "ethereum:1"));
        assert!(facts.iter().all(|f| f.resource_ids.contains(&"0xabc123".to_string())));
    }
    
    #[test]
    fn test_chain_status() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        
        let status = rt.block_on(async {
            adapter.get_chain_status(&ChainId::new("ethereum:1")).await.unwrap()
        });
        
        assert_eq!(status.chain_id.0, "ethereum:1");
        assert_eq!(status.latest_indexed_height, 200);
        assert_eq!(status.latest_chain_height, 210);
        assert_eq!(status.indexing_lag, 10);
        assert!(status.is_healthy);
    }
    
    #[test]
    fn test_factory() {
        let rt = Runtime::new().unwrap();
        let adapter = create_test_adapter();
        let factory = TestAdapterFactory::new(adapter);
        
        let adapter_instance = rt.block_on(async {
            factory.create().await.unwrap()
        });
        
        let facts = rt.block_on(async {
            adapter_instance.get_facts_by_chain(
                &ChainId::new("ethereum:1"),
                None,
                None,
                QueryOptions::default(),
            ).await.unwrap()
        });
        
        assert_eq!(facts.len(), 2);
    }
} 