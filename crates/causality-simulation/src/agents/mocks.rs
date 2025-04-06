// Purpose: Provides mock agent implementations for simulation testing.

use crate::agent::{AgentId, SimulatedAgent, SimulationAgentConfig};
use causality_core::resource::ResourceId;
use causality_types::crypto_primitives::{ContentHash, HashAlgorithm};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info, warn};
use std::sync::OnceLock;

// Helper to create ResourceId from string ID
fn create_resource_id(id_str: &str) -> ResourceId {
    let hash_bytes = blake3::hash(id_str.as_bytes());
    let content_hash = ContentHash::new(
        &HashAlgorithm::Blake3.to_string(),
        hash_bytes.as_bytes().to_vec()
    );
    ResourceId::with_name(content_hash, id_str.to_string())
}

// --- Mock User Agent ---

#[derive(Debug)]
pub struct MockUserAgent {
    id_str: String,
    resource_id: OnceLock<ResourceId>,
}

impl MockUserAgent {
    pub fn new(id_str: String) -> Self {
        Self { 
            id_str, 
            resource_id: OnceLock::new(), 
        }
    }
}

#[async_trait]
impl SimulatedAgent for MockUserAgent {
    async fn run(&self, config: SimulationAgentConfig) -> Result<()> {
        tracing::info!(agent_id = %self.id_str, config = ?config, "MockUserAgent running...");
        // Simulate some user activity
        for i in 0..3 {
            tracing::debug!(agent_id = %self.id_str, iteration = i, "User doing work...");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        tracing::info!(agent_id = %self.id_str, "MockUserAgent finished.");
        Ok(())
    }

    fn id(&self) -> &ResourceId {
        self.resource_id.get_or_init(|| create_resource_id(&self.id_str))
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!(agent_id = %self.id_str, "MockUserAgent shutting down.");
        Ok(())
    }
}

// --- Mock Committee Agent ---

/// Mock Committee agent for testing.
#[derive(Debug)]
pub struct MockCommitteeAgent {
    id_str: String,
    resource_id: OnceLock<ResourceId>,
}

impl MockCommitteeAgent {
    pub fn new(id_str: String) -> Self {
        Self { 
            id_str,
            resource_id: OnceLock::new(), 
        }
    }
}

#[async_trait]
impl SimulatedAgent for MockCommitteeAgent {
    async fn run(&self, config: SimulationAgentConfig) -> Result<()> {
        info!(agent_id = %self.id_str, config = ?config, "MockCommitteeAgent running");
        // Simulate some committee activity
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; 
        info!(agent_id = %self.id_str, "MockCommitteeAgent finished.");
        Ok(())
    }

    fn id(&self) -> &ResourceId {
        self.resource_id.get_or_init(|| create_resource_id(&self.id_str))
    }

    // Optional: Override shutdown for specific cleanup if needed
    // async fn shutdown(&self) -> Result<()> { ... }
}
