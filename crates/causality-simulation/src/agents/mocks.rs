// Purpose: Provides mock agent implementations for simulation testing.

use crate::agent::{AgentId, SimulatedAgent, SimulationAgentConfig};
use causality_core::resource::ResourceId;
use causality_types::crypto_primitives::{ContentHash, HashAlgorithm};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info, warn};
use std::sync::OnceLock;

// Import the core agent implementations
use causality_core::resource::agent::{
    user::{UserAgent, UserAgentBuilder, AuthenticationMethod},
    committee::{CommitteeAgent, CommitteeAgentBuilder, CommitteeConfig},
    agent::AgentBuilder,
    types::AgentState
};

// Helper to create ResourceId from string ID
fn create_resource_id(id_str: &str) -> ResourceId {
    let hash_bytes = blake3::hash(id_str.as_bytes());
    ResourceId::from_hash_bytes(hash_bytes.as_bytes())
}

// --- Mock User Agent ---

#[derive(Debug)]
pub struct MockUserAgent {
    id_str: String,
    resource_id: OnceLock<ResourceId>,
    // Use the core UserAgent instead of reimplementing
    user_agent: Option<UserAgent>,
}

impl MockUserAgent {
    pub fn new(id_str: String) -> Self {
        // Create a simple UserAgent
        let user_agent = UserAgentBuilder::new()
            .with_display_name(&id_str)
            .with_auth_method(AuthenticationMethod::PublicKey { 
                public_key: format!("pk_{}", id_str) 
            })
            .state(AgentState::Active)
            .build()
            .ok();

        Self { 
            id_str, 
            resource_id: OnceLock::new(),
            user_agent,
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
    // Use the core CommitteeAgent instead of reimplementing
    committee_agent: Option<CommitteeAgent>,
}

impl MockCommitteeAgent {
    pub fn new(id_str: String) -> Self {
        // Create a CommitteeConfig
        let committee_config = CommitteeConfig {
            domain: format!("domain_{}", id_str),
            quorum_percentage: 67,
            max_size: 5,
            min_votes: 2,
            protocol_version: "1.0".to_string(),
        };

        // Create a simple CommitteeAgent
        let committee_agent = CommitteeAgentBuilder::new()
            .with_config(committee_config)
            .state(AgentState::Active)
            .build()
            .ok();

        Self { 
            id_str,
            resource_id: OnceLock::new(),
            committee_agent,
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

    // async fn shutdown(&self) -> Result<()> { ... }
}
