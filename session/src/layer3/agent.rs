// Layer 3 Agent model - autonomous actors with capabilities

use crate::layer3::capability::Capability;
use std::collections::BTreeMap;

/// Unique identifier for an agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(name: &str) -> Self {
        AgentId(name.to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent lifecycle status
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Active,
    Suspended,
    Terminated,
}

/// Agent state - local storage for agent data
#[derive(Debug, Clone, PartialEq)]
pub struct AgentState {
    /// Key-value store for agent's local state
    data: BTreeMap<String, String>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentState {
    pub fn new() -> Self {
        AgentState {
            data: BTreeMap::new(),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
    
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
}

/// An agent is an autonomous entity with capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub id: AgentId,
    pub capabilities: Vec<Capability>,
    pub supervisor: Option<AgentId>,
    pub state: AgentState,
    pub status: AgentStatus,
}

impl Agent {
    /// Create a new agent
    pub fn new(name: &str) -> Self {
        Agent {
            id: AgentId::new(name),
            capabilities: Vec::new(),
            supervisor: None,
            state: AgentState::new(),
            status: AgentStatus::Active,
        }
    }
    
    /// Create a new supervised agent
    pub fn with_supervisor(name: &str, supervisor: AgentId) -> Self {
        Agent {
            id: AgentId::new(name),
            capabilities: Vec::new(),
            supervisor: Some(supervisor),
            state: AgentState::new(),
            status: AgentStatus::Active,
        }
    }
    
    /// Add a capability to the agent
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }
    
    /// Check if agent has a capability that allows an effect
    pub fn can_perform(&self, effect_name: &str) -> bool {
        self.capabilities.iter().any(|cap| cap.allows_effect(effect_name))
    }
    
    /// Get all capabilities that allow a specific effect
    pub fn capabilities_for(&self, effect_name: &str) -> Vec<&Capability> {
        self.capabilities.iter()
            .filter(|cap| cap.allows_effect(effect_name))
            .collect()
    }
    
    /// Suspend the agent
    pub fn suspend(&mut self) {
        self.status = AgentStatus::Suspended;
    }
    
    /// Terminate the agent
    pub fn terminate(&mut self) {
        self.status = AgentStatus::Terminated;
    }
    
    /// Check if agent is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, AgentStatus::Active)
    }
}

/// Agent registry - manages all agents in the system
pub struct AgentRegistry {
    agents: BTreeMap<AgentId, Agent>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        AgentRegistry {
            agents: BTreeMap::new(),
        }
    }
    
    /// Register a new agent
    pub fn register(&mut self, agent: Agent) -> Result<(), String> {
        if self.agents.contains_key(&agent.id) {
            return Err(format!("Agent {:?} already registered", agent.id));
        }
        self.agents.insert(agent.id.clone(), agent);
        Ok(())
    }
    
    /// Get an agent by ID
    pub fn get(&self, id: &AgentId) -> Option<&Agent> {
        self.agents.get(id)
    }
    
    /// Lookup an agent by ID (alias for get)
    pub fn lookup(&self, id: &AgentId) -> Option<&Agent> {
        self.get(id)
    }
    
    /// Get a mutable reference to an agent
    pub fn get_mut(&mut self, id: &AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(id)
    }
    
    /// Remove an agent from the registry
    pub fn unregister(&mut self, id: &AgentId) -> Option<Agent> {
        self.agents.remove(id)
    }
    
    /// List all agent IDs in deterministic order
    pub fn list_agents(&self) -> Vec<&AgentId> {
        self.agents.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_id() {
        let id1 = AgentId::new("Alice");
        let id2 = AgentId::new("Alice");
        let id3 = AgentId::new("Bob");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_agent_creation() {
        let agent_id = AgentId::new("Alice");
        let agent = Agent::new("Alice");
        
        assert_eq!(agent.id, agent_id);
        assert!(agent.capabilities.is_empty());
        assert!(agent.supervisor.is_none());
        assert!(agent.is_active());
    }
    
    #[test]
    fn test_agent_capabilities() {
        let mut agent = Agent::new("Bob");
        
        // Add rate-limited API capability
        let api_cap = Capability::rate_limited_api(100);
        agent.add_capability(api_cap);
        
        // Add data access capability
        let data_cap = Capability::data_access(
            vec!["users".to_string(), "orders".to_string()],
            vec![],
        );
        agent.add_capability(data_cap);
        
        // Test capability checks
        assert!(agent.can_perform("api_call"));
        assert!(agent.can_perform("read_users"));
        assert!(agent.can_perform("write_orders"));
        assert!(!agent.can_perform("read_audit_log"));
        
        // Test getting capabilities for an effect
        let api_caps = agent.capabilities_for("api_call");
        assert_eq!(api_caps.len(), 1);
    }
    
    #[test] 
    fn test_agent_state() {
        let mut agent = Agent::new("Carol");
        
        agent.state.set("balance".to_string(), "100".to_string());
        assert_eq!(agent.state.get("balance"), Some(&"100".to_string()));
        
        agent.state.set("balance".to_string(), "200".to_string());
        assert_eq!(agent.state.get("balance"), Some(&"200".to_string()));
    }
    
    #[test]
    fn test_agent_lifecycle() {
        let mut agent = Agent::new("Dave");
        
        assert!(agent.is_active());
        
        agent.suspend();
        assert!(!agent.is_active());
        assert!(matches!(agent.status, AgentStatus::Suspended));
        
        agent.terminate();
        assert!(!agent.is_active());
        assert!(matches!(agent.status, AgentStatus::Terminated));
    }
    
    #[test]
    fn test_agent_registry() {
        let mut registry = AgentRegistry::new();
        
        let alice = Agent::new("Alice");
        let bob = Agent::new("Bob");
        
        registry.register(alice.clone()).unwrap();
        registry.register(bob.clone()).unwrap();
        
        assert_eq!(registry.get(&alice.id), Some(&alice));
        assert_eq!(registry.get(&bob.id), Some(&bob));
        
        let agents = registry.list_agents();
        assert_eq!(agents.len(), 2);
        
        registry.unregister(&alice.id);
        assert_eq!(registry.get(&alice.id), None);
    }
}
